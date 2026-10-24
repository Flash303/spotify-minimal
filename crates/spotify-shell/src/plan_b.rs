use crate::config::SPOTIFY_URL;

const CANDIDATES: [&str; 6] = [
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    "chrome",
    "msedge",
];

pub fn launch() -> Result<(), String> {
    if let Ok(binary) = std::env::var("SPOTIFY_PLAN_B_BIN") {
        return spawn(&binary);
    }
    let mut failures = 0;
    for binary in CANDIDATES {
        match spawn(binary) {
            Ok(()) => return Ok(()),
            Err(_) => failures += 1,
        }
    }
    Err(format!(
        "no browser found ({failures} attempts): install Chrome, Chromium or Edge, or set SPOTIFY_PLAN_B_BIN"
    ))
}

fn spawn(binary: &str) -> Result<(), String> {
    match std::process::Command::new(binary)
        .arg(format!("--app={SPOTIFY_URL}"))
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("{binary} : {error}")),
    }
}
