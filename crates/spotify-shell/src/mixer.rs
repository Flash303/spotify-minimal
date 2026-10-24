#![allow(unsafe_code)]

use std::collections::HashMap;
use std::time::Duration;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IAudioSessionControl, IAudioSessionControl2, IAudioSessionManager2,
    IMMDeviceEnumerator, MMDeviceEnumerator, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::core::{Interface, PCWSTR};

const DISPLAY_NAME: &str = "spotify-minimal";
const WEBVIEW_EXE: &str = "msedgewebview2.exe";

pub fn ensure_labels() {
    if let Err(error) = std::thread::Builder::new()
        .name("mixer".to_owned())
        .spawn(watch)
    {
        tracing::warn!("unable to rename Windows mixer: {error}");
    }
}

fn watch() {
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    apply_once();
    loop {
        std::thread::sleep(Duration::from_secs(2));
        apply_once();
    }
}

fn apply_once() {
    if let Err(error) = relabel_sessions() {
        tracing::debug!("mixer rename deferred: {error}");
    }
}

fn relabel_sessions() -> windows::core::Result<()> {
    let owned = unsafe { GetCurrentProcessId() };
    let processes = snapshot();
    let icon = std::env::current_exe()
        .ok()
        .map(|path| path.to_string_lossy().into_owned());
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let endpoints = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
    let total = unsafe { endpoints.GetCount()? };
    for index in 0..total {
        let manager: IAudioSessionManager2 =
            unsafe { endpoints.Item(index)?.Activate(CLSCTX_ALL, None)? };
        let sessions = unsafe { manager.GetSessionEnumerator()? };
        let count = unsafe { sessions.GetCount()? };
        for position in 0..count {
            let control = unsafe { sessions.GetSession(position)? };
            relabel_session(&control, owned, &processes, icon.as_deref());
        }
    }
    Ok(())
}

fn relabel_session(
    control: &IAudioSessionControl,
    owned: u32,
    processes: &HashMap<u32, (u32, String)>,
    icon: Option<&str>,
) {
    let managed = control
        .cast::<IAudioSessionControl2>()
        .and_then(|process| unsafe { process.GetProcessId() })
        .map(|pid| is_managed(pid, owned, processes))
        .unwrap_or(false);
    if !managed {
        return;
    }
    if current_text(control, true) != DISPLAY_NAME {
        let wide = to_wide(DISPLAY_NAME);
        let _ =
            unsafe { control.SetDisplayName(PCWSTR::from_raw(wide.as_ptr()), std::ptr::null()) };
    }
    if let Some(path) = icon {
        if current_text(control, false) != path {
            let wide = to_wide(path);
            let _ =
                unsafe { control.SetIconPath(PCWSTR::from_raw(wide.as_ptr()), std::ptr::null()) };
        }
    }
}

fn current_text(control: &IAudioSessionControl, name: bool) -> String {
    let raw = unsafe {
        if name {
            control.GetDisplayName()
        } else {
            control.GetIconPath()
        }
    };
    match raw {
        Ok(text) => String::from_utf16_lossy(trim_nul(unsafe { text.as_wide() })),
        Err(_) => String::new(),
    }
}

fn trim_nul(wide: &[u16]) -> &[u16] {
    let mut end = wide.len();
    while end > 0 && wide[end - 1] == 0 {
        end -= 1;
    }
    &wide[..end]
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn is_managed(pid: u32, owned: u32, processes: &HashMap<u32, (u32, String)>) -> bool {
    match processes.get(&pid) {
        Some((_, exe)) => exe == WEBVIEW_EXE && descendant_of(pid, owned, processes),
        None => false,
    }
}

fn descendant_of(pid: u32, ancestor: u32, processes: &HashMap<u32, (u32, String)>) -> bool {
    let mut current = pid;
    for _ in 0..64 {
        if current == ancestor {
            return true;
        }
        match processes.get(&current) {
            Some((parent, _)) if *parent != current => current = *parent,
            _ => return false,
        }
    }
    false
}

fn snapshot() -> HashMap<u32, (u32, String)> {
    let mut processes = HashMap::new();
    let snap = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(handle) => handle,
        Err(_) => return processes,
    };
    unsafe {
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|slot| *slot == 0)
                    .unwrap_or(260);
                let exe = String::from_utf16_lossy(&entry.szExeFile[..end]).to_ascii_lowercase();
                processes.insert(entry.th32ProcessID, (entry.th32ParentProcessID, exe));
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    processes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(entries: &[(u32, u32, &str)]) -> HashMap<u32, (u32, String)> {
        entries
            .iter()
            .map(|(pid, parent, exe)| (*pid, (*parent, exe.to_string())))
            .collect()
    }

    #[test]
    fn walks_up_ancestors() {
        let procs = table(&[
            (1, 0, "spotify-minimal.exe"),
            (5, 1, "msedgewebview2.exe"),
            (10, 5, "msedgewebview2.exe"),
        ]);
        assert!(is_managed(10, 1, &procs));
        assert!(is_managed(5, 1, &procs));
        assert!(!is_managed(10, 2, &procs));
        assert!(!is_managed(1, 1, &procs));
        assert!(!is_managed(99, 1, &procs));
    }

    #[test]
    fn ignores_other_apps() {
        let procs = table(&[(1, 0, "spotify-minimal.exe"), (7, 4, "msedgewebview2.exe")]);
        assert!(!is_managed(7, 1, &procs));
    }

    #[test]
    fn no_endless_loop() {
        let procs = table(&[(7, 7, "msedgewebview2.exe")]);
        assert!(!is_managed(7, 1, &procs));
    }
}
