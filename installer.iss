[Setup]
AppName=wsup
AppVersion={#MyAppVersion}
DefaultDirName={pf}\wsup
DefaultGroupName=wsup
OutputBaseFilename=wsup-setup
Compression=lzma
SolidCompression=yes

[Files]
Source: "target\x86_64-pc-windows-msvc\release\wsup.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\wsup"; Filename: "{app}\wsup.exe"

[Tasks]
Name: "addpath"; Description: "Add wsup to PATH"; Flags: checkedonce

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; \
ValueName: "Path"; ValueData: "{olddata};{app}"; \
Tasks: addpath