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
Name: addpath; Description: "Add wsup to PATH"

[Code]

function NeedsAddPath(Param: string): Boolean;
var
  Path: string;
begin
  RegQueryStringValue(HKCU, 'Environment', 'Path', Path);

  Result := Pos(Uppercase(ExpandConstant('{app}')),
                Uppercase(Path)) = 0;
end;

[Registry]
Root: HKCU; Subkey: "Environment"; \
ValueType: expandsz; ValueName: "Path"; \
ValueData: "{olddata};{app}"; \
Tasks: addpath; Check: NeedsAddPath('')