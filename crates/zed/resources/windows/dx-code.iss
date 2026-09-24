; Minimal Dx Code installer (size-focused).
; Ships the release exe + WebView2 loader + assets/web tree (served from disk
; next to the exe by web_preview's find_assets_web_root). No code signing,
; no ConPTY/AGS vendor DLLs, no explorer injector/CLI companions yet — those
; are bundle parity follow-ups, intentionally excluded for the size baseline.
#define MyAppName "Dx Code"
#define MyAppVersion "1.8.0"
#define MyAppExeName "dx-code.exe"
#define RepoRoot SourcePath + "\..\..\..\.."

[Setup]
AppId={{ea8373d3-5b82-4d29-a61b-0bf8f9665025}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={autopf}\DxCode
DefaultGroupName=Dx Code
OutputDir={#RepoRoot}\target
OutputBaseFilename=DxCode-Setup-x86_64
Compression=lzma2/max
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile={#SourcePath}\app-icon-dev.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=lowest
WizardStyle=modern

[Files]
Source: "{#RepoRoot}\target\release\dx-code.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\target\release\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\..\assets\web\*"; DestDir: "{app}\assets\web"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Dx Code"; Filename: "{app}\{#MyAppExeName}"
