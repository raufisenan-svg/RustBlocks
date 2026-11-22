#define MyAppName "RustBlocks"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "RustBlocks"
#define MyAppExeName "rustblocks.exe"

[Setup]
AppId={{D9A2D7C4-3B0B-4F9A-9A12-6A0D4B61C102}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\RustBlocks
DefaultGroupName=RustBlocks
OutputDir=dist
OutputBaseFilename=RustBlocks-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "target\x86_64-pc-windows-msvc\release\rustblocks.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\RustBlocks"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\RustBlocks"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch RustBlocks"; Flags: postinstall nowait skipifsilent
