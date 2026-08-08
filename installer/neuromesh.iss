; NeuroMesh Windows installer (Inno Setup 6).
; Build locally:  ISCC.exe installer\neuromesh.iss
; CI overrides the version:  ISCC.exe /DMyAppVersion=x.y.z installer\neuromesh.iss

#ifndef MyAppVersion
  #define MyAppVersion "0.1.2"
#endif
#define MyAppName "NeuroMesh"
#define MyAppExeName "neuromesh.exe"

[Setup]
AppId={{A7F3D2C4-9B1E-4E8A-B5D6-3C2F8E7A1B90}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=NeuroMesh contributors
AppPublisherURL=https://github.com/SC0R9I0N/neuromesh
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; No admin required: per-user install to {localappdata}\Programs by default,
; but an elevated install to Program Files is offered via the dialog.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=NeuroMesh-Setup-{#MyAppVersion}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Tasks]
; Checked by default — desktop shortcut is part of the standard install.
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent
