; Installeur Windows de MMdedit (Inno Setup 6).
;
; Compilation :
;   build\build.bat installeur      (construit l'exe puis l'installeur)
;   ou : ISCC.exe build\mmdedit.iss
;
; Prérequis : dist\MMdedit.exe doit exister (cf. build\build.bat).
; Sortie    : dist\MMdedit-<version>-setup.exe
;
; Choix d'intégration : MMdedit s'ajoute à la liste « Ouvrir avec » des
; fichiers .md/.markdown/.txt via OpenWithProgids. Il ne s'impose PAS comme
; application par défaut — l'association existante de l'utilisateur est
; conservée. Windows reste seul juge du défaut, comme il se doit depuis
; Windows 8.

#define AppName        "MMdedit"
#define AppVersion     "0.3.0"
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
SetupIconFile=..\src\mmdedit\assets\mmdedit.ico
; Licence presentee a l'installation (exigence morale du copyleft : l'utilisateur
; doit savoir sous quels termes il recoit le programme).
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Installation dans le profil par défaut (aucune élévation exigée) ; l'assistant
; propose l'installation pour tous les utilisateurs si on dispose de l'admin.
; Sans « lowest », Inno réclamerait l'élévation d'office.
PrivilegesRequired=lowest
; « commandline » déclare /ALLUSERS et /CURRENTUSER recevables en ligne de
; commande. Tout déploiement automatisé en dépend : il s'exécute sous SYSTEM et
; exige une installation machine-wide — sans quoi le programme atterrirait dans
; le profil de SYSTEM, invisible pour les utilisateurs et inopérant sur un
; serveur de bureaux à distance multi-session.
; Mesuré sur banc le 2026-07-24 : avec « dialog » seul, /VERYSILENT /ALLUSERS
; installait déjà correctement dans {commonpf} (Inno tranche de lui-même quand
; le dialogue est supprimé et que les droits admin sont là). La déclaration
; explicite ne corrige donc pas un défaut constaté : elle fixe le contrat au
; lieu de le laisser dépendre d'un arbitrage implicite non documenté pour ce cas.
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
Source: "..\dist\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md";      DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "..\LICENSE";        DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}";  Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; --- Type de fichier propre à MMdedit -------------------------------------
; Décrit une fois, réutilisé par chaque extension via OpenWithProgids.
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
; uninsdeletekey porte sur la clé RACINE Applications\MMdedit.exe : posé sur la
; seule sous-clé « command », il laissait la clé parente derrière lui.
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
