#define AppName "Commandblock"
#define AppVersion "1.0.0"
#define AppExeName "Commandblock.exe"

[Setup]
AppId={{A5721B0D-80D0-466B-8B8B-7E43D0678721}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#AppExeName}
SetupIconFile=..\assets\buff-command-block.ico
OutputDir=..\dist
OutputBaseFilename=Commandblock-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern

[Dirs]
Name: "{userappdata}\Commandblock"
Name: "{code:ModelStoreDir}"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked
Name: "ollamasignin"; Description: "เข้าสู่ระบบ Ollama (สำหรับโมเดล Cloud — ข้ามได้)"; GroupDescription: "Ollama Cloud:"; Flags: unchecked

[Files]
Source: "..\target\release\commandblock.exe"; DestDir: "{app}"; DestName: "Commandblock.exe"; Flags: ignoreversion
Source: "..\target\release\commandblock-connector.exe"; DestDir: "{app}"; DestName: "commandblock-connector.exe"; Flags: ignoreversion
Source: "..\target\release\commandblock-updater.exe"; DestDir: "{app}"; DestName: "commandblock-updater.exe"; Flags: ignoreversion
Source: "..\docs\SETUP-GUIDE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "config-template.json"; DestDir: "{userappdata}\Commandblock"; DestName: "config.json"; Flags: onlyifdoesntexist ignoreversion
Source: "payload\ollama\OllamaSetup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall ignoreversion
Source: "payload\models\*"; DestDir: "{code:ModelStoreDir}"; Flags: recursesubdirs createallsubdirs onlyifdoesntexist ignoreversion
Source: "payload\LICENSES\*"; DestDir: "{app}\licenses"; Flags: recursesubdirs createallsubdirs ignoreversion
Source: "payload\SHA256SUMS.txt"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Commandblock"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"
Name: "{autoprograms}\คู่มือการตั้งค่า Commandblock"; Filename: "{sys}\notepad.exe"; Parameters: """{app}\SETUP-GUIDE.txt"""
Name: "{autoprograms}\ใบอนุญาต DeepSeek Coder"; Filename: "{sys}\notepad.exe"; Parameters: """{app}\licenses\DEEPSEEK-CODER.txt"""
Name: "{autodesktop}\Commandblock"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"; Tasks: desktopicon

[Run]
Filename: "{tmp}\OllamaSetup.exe"; Description: "ติดตั้ง Ollama สำหรับ DeepSeek Coder"; Flags: waituntilterminated skipifsilent; Check: NeedsOllamaInstall
Filename: "{code:OllamaExe}"; Parameters: "show deepseek-coder:1.3b"; Flags: runhidden waituntilterminated skipifsilent; Check: HasOllama
Filename: "{sys}\notepad.exe"; Parameters: """{app}\SETUP-GUIDE.txt"""; Description: "เปิดคู่มือการตั้งค่า Commandblock"; Flags: nowait postinstall skipifsilent
Filename: "{code:OllamaExe}"; Parameters: "signin"; Description: "เข้าสู่ระบบ Ollama (สำหรับโมเดล Cloud — ข้ามได้)"; Flags: nowait postinstall skipifsilent unchecked; Tasks: ollamasignin; Check: HasOllama
Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"; Description: "Launch Commandblock"; Flags: nowait postinstall skipifsilent

[Code]
const
  RequiredModelStoreBytes = 2147483648;
  RequiredTempBytes = 3221225472;

function ModelStoreDir(Param: String): String;
begin
  Result := GetEnv('OLLAMA_MODELS');
  if Result = '' then
    Result := ExpandConstant('{userprofile}\.ollama\models');
end;

function OllamaExe(Param: String): String;
begin
  Result := ExpandConstant('{localappdata}\Programs\Ollama\ollama.exe');
end;

function HasOllama(): Boolean;
begin
  Result := FileExists(OllamaExe(''));
end;

function NeedsOllamaInstall(): Boolean;
begin
  Result := not HasOllama();
end;

function HasEnoughSpace(const Path: String; const Required: Int64): Boolean;
var
  FreeSpace, TotalSpace: Int64;
begin
  if not GetSpaceOnDisk64(Path, FreeSpace, TotalSpace) then begin
    Result := True;
    exit;
  end;
  Result := FreeSpace >= Required;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  if not HasEnoughSpace(ModelStoreDir(''), RequiredModelStoreBytes) then begin
    Result := 'พื้นที่ดิสก์สำหรับโมเดล Ollama ไม่พอ ต้องมีพื้นที่ว่างอย่างน้อย 2 GB ที่ ' + ModelStoreDir('');
    exit;
  end;
  if NeedsOllamaInstall() and not HasEnoughSpace(ExpandConstant('{tmp}'), RequiredTempBytes) then begin
    Result := 'พื้นที่ชั่วคราวสำหรับติดตั้ง Ollama ไม่พอ ต้องมีพื้นที่ว่างอย่างน้อย 3 GB';
  end;
end;
