# MMdedit

Lecteur et éditeur Markdown léger, dans l'esprit du bloc-notes, pour Windows,
Linux et macOS. Logiciel libre sous **GNU GPL v3**.

---

## Ce qui le distingue

### Un presse-papier qui retient ce que vous sélectionnez

Sur les systèmes X11, sélectionner du texte suffit à le rendre collable :
c'est la *sélection primaire*, et elle n'existe pas sous Windows. MMdedit la
rétablit — **toute sélection part automatiquement au presse-papier**, sans
`Ctrl+C`, dans l'éditeur comme dans l'aperçu. Sélectionner dans l'aperçu copie
le **texte rendu**, débarrassé des marqueurs Markdown.

Deux garde-fous évitent l'effet de bord habituel de ce confort :

- **Protection de 60 secondes.** Si le presse-papier a reçu autre chose depuis
  moins d'une minute — un `Ctrl+C`, ou une copie faite dans une autre
  application — une sélection ne l'écrase pas. La barre d'état affiche le temps
  restant. Passé ce délai, le contenu est tenu pour périmé et la sélection
  reprend la main.
- **Origine affichée en permanence**, dans la dernière cellule de la barre
  d'état : `extérieur` (déposé par une autre application), `saisie` (venu de la
  zone d'édition) ou `rendu` (venu de l'aperçu), suivie du compte à rebours
  tant qu'il court. On sait donc toujours ce que l'on s'apprête à coller.

La fonction s'active et se désactive dans le menu *Édition* ; elle est active
par défaut.

### Lire et écrire du Markdown, et pas seulement

**Édition et aperçu côte à côte, rendu en temps réel.** Chaque vue se masque
indépendamment (menu *Affichage*), l'une des deux restant toujours visible.

MMdedit ouvre **tout fichier texte**. Le format est déduit de l'extension et
affiché en barre d'état :

| Format | Extensions | Comportement |
|---|---|---|
| **Markdown** | `.md`, `.markdown` | coloration Markdown, aperçu rendu affiché |
| **XML / HTML** | `.xml`, `.xsd`, `.xsl`, `.svg`, `.html`, `.plist`, `.csproj`… | coloration XML (balises, attributs, valeurs, commentaires multi-lignes, CDATA, entités), aperçu masqué |
| **Texte** | `.txt`, `.csv`… | sans coloration, aperçu masqué |

L'aperçu reste rouvrable à la main par le menu *Affichage*.

---

## Les autres fonctions

- **Barre d'outils de mise en forme** — gras, italique, barré, code, titres,
  listes, citation, lien, image, et un rappel des balises dans le menu *Aide*.
  Les boutons sont des **bascules** : recliquer retire le marquage, et passer
  d'un niveau de titre à un autre remplace le précédent. Raccourcis `Ctrl+B` et
  `Ctrl+I`.
- **La sélection est conservée** après application d'un style, marqueurs
  exclus : on enchaîne les styles ou on annule d'un second clic sans
  resélectionner. Les marqueurs imbriqués sont distingués — appliquer
  l'italique sur du gras donne `***texte***`, et non `*texte*`.
- **Coloration syntaxique** dans la zone d'édition.
- **Rechercher / Remplacer**, avec respect de la casse et mot entier.
- **Copier / Couper / Coller suivent la vue qui a le focus.** L'aperçu étant en
  lecture seule, seul *Copier* y agit.
- **Apparence au choix** (*Affichage > Apparence*), appliquée à chaud et
  mémorisée : **Classique (Windows 9x)**, défaut sur Windows et Linux ;
  **Moderne**, surfaces plates et coins arrondis ; **Système**, aucune feuille
  de style, l'aspect natif de la plateforme — défaut sur macOS.
- **Réglages conservés d'une session à l'autre** : taille et position de la
  fenêtre, partage éditeur / aperçu, copie automatique, apparence. Écrits par
  QSettings (`HKCU\Software\M-Media\MMdedit` sous Windows,
  `~/.config/M-Media/MMdedit.conf` ailleurs).
- **Compteur** mots / caractères / lignes en barre d'état.
- **Export PDF** du rendu.
- **Avertissement de sauvegarde** à la fermeture si des modifications sont en
  cours. Aucune sauvegarde automatique.
- Glisser-déposer d'un fichier sur la fenêtre, et ouverture par argument
  (`MMdedit fichier.md`).

## Encodage

- Lecture : UTF-8 en priorité, repli automatique cp1252 puis latin-1.
- Écriture : UTF-8 sans BOM, fins de ligne normalisées en CRLF.

## Exécution depuis les sources

```bash
python -m pip install -r requirements.txt
python run_mmdedit.py           # ou : python -m mmdedit  (depuis src/)
```

## Installation sous Windows

`dist\MMdedit.exe` seul est **autonome** et n'écrit rien dans le registre :
aucune association, aucun raccourci. L'intégration passe par l'installeur
`dist\MMdedit-<version>-setup.exe` (Inno Setup), qui pose :

- le programme dans `%LOCALAPPDATA%\Programs\MMdedit` (aucune élévation
  requise ; l'assistant propose l'installation pour tous les utilisateurs si
  l'on dispose des droits d'administration) ;
- un raccourci au menu Démarrer, et sur le Bureau si la case est cochée ;
- MMdedit dans la liste **« Ouvrir avec »** des `.md`, `.markdown` et `.txt` ;
- une entrée de désinstallation dans *Programmes et fonctionnalités*.

L'installeur **ne s'impose pas comme application par défaut** : il s'ajoute à
`OpenWithProgids` sans toucher au `UserChoice` de l'utilisateur. Choisir
MMdedit comme application par défaut reste une action explicite, via *Ouvrir
avec > Toujours utiliser cette application*.

Installation et désinstallation silencieuses **pour l'utilisateur courant** :

```bat
MMdedit-<version>-setup.exe /VERYSILENT /MERGETASKS=associer
"%LOCALAPPDATA%\Programs\MMdedit\unins000.exe" /VERYSILENT
```

Installation silencieuse **pour tous les utilisateurs** — la forme à retenir
pour un déploiement automatisé, qui s'exécute généralement sous le compte
SYSTEM :

```bat
MMdedit-<version>-setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /ALLUSERS /MERGETASKS=associer
"%ProgramFiles%\MMdedit\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES
```

`/ALLUSERS` n'est pas cosmétique : sans lui, `PrivilegesRequired=lowest`
installe dans le profil du compte appelant — celui de SYSTEM lors d'un
déploiement, invisible pour les utilisateurs et inopérant sur un serveur de
bureaux à distance multi-session.

Depuis 0.2.2, le script déclare `PrivilegesRequiredOverridesAllowed=commandline
dialog`, qui rend `/ALLUSERS` explicitement recevable. Mesure faite le
2026-07-24 : avec `dialog` seul (versions ≤ 0.2.1), `/VERYSILENT /ALLUSERS`
installait **déjà** dans `C:\Program Files\MMdedit` avec les clés HKLM — Inno
tranche de lui-même quand le dialogue est supprimé et que les droits
d'administration sont présents. La déclaration explicite ne répare donc rien :
elle fixe le contrat plutôt que de le laisser reposer sur un arbitrage
implicite et non documenté pour ce cas.

## Installation sous Linux et macOS

Il n'existe pas de binaire pour ces plateformes : PyInstaller ne fait pas de
compilation croisée. L'installation part des **sources**, prise en charge de
bout en bout par un script — rien à compiler à la main.

```bash
tar xzf MMdedit-<version>-sources.tar.gz && cd mmdedit
./packaging/install-linux.sh      # ou install-macos.sh
```

Les deux scripts créent un environnement virtuel dédié, y installent PySide6,
copient le programme et posent l'intégration au bureau, **sans privilège
administrateur** ni modification du Python du système. Chacun accepte
`--desinstaller` pour tout retirer.

| | Linux | macOS |
|---|---|---|
| Programme | `~/.local/share/mmdedit` | `~/Library/Application Support/MMdedit` |
| Lancement | `mmdedit [fichier]` | `~/Applications/MMdedit.app` |
| Intégration | entrée de menu `.desktop` + icônes hicolor (7 tailles) | bundle `.app` + icône `.icns` + types de documents |

**État de validation** : le script Linux a été **exécuté sur une Debian 12** —
installation, lancement, ouverture de fichier, désinstallation sans résidu. Le
script macOS n'a **pas** pu être exécuté faute de Mac ; sa syntaxe est validée
et sa structure identique, mais son premier passage réel reste à faire.

L'application n'étant ni signée ni notariée, macOS bloque le premier
lancement : clic droit sur l'application puis « Ouvrir », une seule fois.

## Construction de l'exécutable

PyInstaller ne fait pas de compilation croisée : construire sur la plateforme
cible.

- Windows : `build\build.bat` → `dist\MMdedit.exe`, puis
  `build\build.bat installeur` pour enchaîner sur l'installeur. Requiert Inno
  Setup 6 (`winget install JRSoftware.InnoSetup`).
- Linux / macOS : `build/build.sh` → `dist/MMdedit`.

Le code source est identique sur les trois plateformes, PySide6 étant
multiplateforme. Seul l'empaquetage est à relancer par système.

Les binaires **ne sont pas versionnés** : un exécutable de ~47 Mo committé à
chaque version gonflerait l'historique git définitivement, sans diff possible.
`dist/` est ignoré, et les binaires sont publiés en **release GitHub**.

## Icône

`build/make_icon.py` régénère `src/mmdedit/assets/mmdedit.ico` à partir d'un
logo passé en argument : il isole le glyphe rouge par détection de teinte, le
centre sur un carré transparent et écrit sept résolutions (16 à 256 px). À
relancer seulement si le logo change.

```bash
python build/make_icon.py chemin/du/logo.png
```

## Vérifications

```bat
python tests\run_tests.py
```

Six suites pilotent une vraie fenêtre Qt en mode *offscreen* — aucune fenêtre
n'apparaît — et exercent le rendu, l'encodage, l'export PDF, la bascule des
vues, la barre d'outils, les formats, le presse-papier et les réglages
persistants. Elles ont mis au jour trois défauts réels : boutons de titres
inopérants sous PySide6 6.11, icône absente de l'exécutable empaqueté, italique
appliqué sur du gras qui le dégradait.

## Licence

**GNU General Public License version 3** ou ultérieure — copyleft. Texte
intégral dans [`LICENSE`](LICENSE), rappelé dans *Aide > À propos* (bouton
« Afficher les détails ») et présenté à l'installation.

Concrètement : quiconque reçoit le programme peut l'utiliser, l'étudier, le
modifier et le redistribuer, **à condition d'accorder les mêmes libertés**,
code source inclus, sous la même licence.

Seule la version anglaise de la GPL a valeur juridique — la FSF ne publie que
des traductions *non officielles*, à titre informatif. C'est pourquoi `LICENSE`
est en anglais, le sens de la licence étant expliqué en français dans *À
propos*.

Qt et PySide6 sont sous **LGPL v3**, compatible avec la GPL v3. L'exécutable
`--onefile` embarquant Qt, sa redistribution est couverte par cette
combinaison.

## Pile technique

PySide6 (Qt). Rendu Markdown et export PDF assurés nativement par Qt
(`QTextDocument.setMarkdown`, `QPrinter`), sans dépendance supplémentaire.
