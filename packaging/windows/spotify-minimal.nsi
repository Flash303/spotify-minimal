Unicode True
SetCompressor /SOLID lzma
RequestExecutionLevel user

!include "MUI2.nsh"

!define APPNAME "spotify-minimal"
!ifndef VERSION
!define VERSION "0.1.0"
!endif
!define EXE "spotify-minimal.exe"
!define SRC "${__FILEDIR__}\..\..\target\release\${EXE}"
!define ICON "${__FILEDIR__}\..\..\assets\icon.ico"

Name "${APPNAME} ${VERSION}"
OutFile "${__FILEDIR__}\..\..\target\${APPNAME}-setup-${VERSION}.exe"
InstallDir "$LOCALAPPDATA\Programs\${APPNAME}"
Icon "${ICON}"
UninstallIcon "${ICON}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${__FILEDIR__}\..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Application" SectionMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "${SRC}"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  CreateShortcut "$SMPROGRAMS\${APPNAME}.lnk" "$INSTDIR\${EXE}" "" "$INSTDIR\${EXE}" 0
  CreateShortcut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\${EXE}" "" "$INSTDIR\${EXE}" 0
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME} ${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\${EXE},0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "spotify-minimal contributors"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSION}"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$SMPROGRAMS\${APPNAME}.lnk"
  Delete "$DESKTOP\${APPNAME}.lnk"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
  DeleteRegKey /ifempty HKCU "Software\spotify-minimal"
SectionEnd
