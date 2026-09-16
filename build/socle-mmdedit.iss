; Installeur Windows de MMdedit — socle Rust / Qt 6 (Inno Setup 6).
;
; Compilation :
;   ISCC.exe /DAppVersion=0.4.0 build\socle-mmdedit.iss
;
; Prérequis : dist\MMdedit\ doit contenir MMdedit.exe et les bibliothèques Qt
; déposées par windeployqt (cf. le workflow « socle », étape « Paquet Windows »).
; Sortie    : dist\MMdedit-<version>-setup.exe
;
; Différence avec build\mmdedit.iss, qui empaquetait la version PySide6 : celle-ci
; n'est plus un exécutable unique produit par PyInstaller, mais un dossier — le
; programme et le Qt dont il dépend. Tout le reste (identifiant, associations,
; privilèges) est repris à l'identique, pour qu'une machine déjà pourvue de la
; version PySide6 reçoive celle-ci comme une mise à jour et non comme un second
; programme : c'est l'AppId, inchangé, qui le garantit.

#ifndef AppVersion
  #define AppVersion "0.4.0"
#endif
#define AppName        "MMdedit"
#define AppPublisher   "M-Media"
#define AppExe         "MMdedit.exe"
#define ProgId         "MMedia.MMdedit.1"

[Setup]
AppId={{8E5A1C6F-2B47-4D93-9A18-3F7C5D0E4B21}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName} {#AppVersion}
OutputDir=..\dist
OutputBaseFilename={#AppName}-{#AppVersion}-setup
SetupIconFile=..\assets\mmdedit.ico
; Licence presentee a l'installation (exigence morale du copyleft : l'utilisateur
; doit savoir sous quels termes il recoit le programme).
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Installation dans le profil par défaut (aucune élévation exigée) ; l'assistant
; propose l'installation pour tous les utilisateurs si on dispose de l'admin.
PrivilegesRequired=lowest
; « commandline » déclare /ALLUSERS et /CURRENTUSER recevables en ligne de
; commande : tout déploiement automatisé en dépend (il s'exécute sous SYSTEM et
; exige une installation pour la machine entière).
PrivilegesRequiredOverridesAllowed=commandline dialog
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
DisableProgramGroupPage=yes
ShowLanguageDialog=no

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le &Bureau"; \
    GroupDescription: "Raccourcis :"; Flags: unchecked
Name: "associer"; Description: "Proposer {#AppName} dans « Ouvrir avec » pour les fichiers .md, .markdown et .txt"; \
    GroupDescription: "Intégration à Windows :"

[Files]
; Le dossier entier : le programme, les bibliothèques Qt et les greffons QML.
Source: "..\dist\MMdedit\*"; DestDir: "{app}"; \
    Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\README.md";      DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "..\LICENSE";        DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}";  Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; --- Type de fichier propre à MMdedit -------------------------------------
Root: HKA; Subkey: "Software\Classes\{#ProgId}"; \
    ValueType: string; ValueName: ""; ValueData: "Document Markdown"; \
    Flags: uninsdeletekey; Tasks: associer
Root: HKA; Subkey: "Software\Classes\{#ProgId}\DefaultIcon"; \
    ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; \
    Tasks: associer
Root: HKA; Subkey: "Software\Classes\{#ProgId}\shell\open\command"; \
    ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; \
    Tasks: associer

; --- Ajout à « Ouvrir avec », sans voler l'association par défaut ----------
Root: HKA; Subkey: "Software\Classes\.md\OpenWithProgids"; \
    ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; \
    Flags: uninsdeletevalue; Tasks: associer
Root: HKA; Subkey: "Software\Classes\.markdown\OpenWithProgids"; \
    ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; \
    Flags: uninsdeletevalue; Tasks: associer
Root: HKA; Subkey: "Software\Classes\.txt\OpenWithProgids"; \
    ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; \
    Flags: uninsdeletevalue; Tasks: associer

; --- Application enregistrée (onglet « Ouvrir avec » complet) --------------
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}"; \
    ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#AppName}"; \
    Flags: uninsdeletekey; Tasks: associer
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\shell\open\command"; \
    ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; \
    Tasks: associer

[Run]
Filename: "{app}\{#AppExe}"; Description: "Lancer {#AppName}"; \
    Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: dirifempty; Name: "{app}"
