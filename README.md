# MMdedit

Lecteur et éditeur Markdown léger, dans l'esprit du bloc-notes, pour Windows,
Linux et Android. Logiciel libre sous **GNU GPL v3**.

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
  mémorisée : **Classique (Windows 9x)**, **Moderne** (M-Media), ou **Système**,
  qui reprend la palette du bureau. Les contrôles sont dessinés par le style
  Fusion de Qt sur toutes les plateformes.
- **Réglages conservés d'une session à l'autre** : géométrie de la fenêtre,
  copie automatique, apparence.
- **Compteur** mots / caractères / lignes en barre d'état.
- **Export PDF** du rendu.
- **Avertissement de sauvegarde** à la fermeture si des modifications sont en
  cours. Aucune sauvegarde automatique.
- Glisser-déposer d'un fichier sur la fenêtre, et ouverture par argument
  (`MMdedit fichier.md`).

## Encodage

- Lecture : UTF-8 en priorité, repli strict CP1252 puis Latin-1.
- Écriture : UTF-8 sans BOM, fins de ligne normalisées en CRLF.
- Les positions échangées avec Qt se comptent en unités UTF-16 : un document
  contenant un emoji ne décale pas la mise en forme.

## Téléchargement

Chaque version est publiée en **release GitHub**, avec ses sources :

| Plateforme | Paquet |
|---|---|
| Windows | `MMdedit-<version>-setup.exe` (Inno Setup) |
| Linux | `MMdedit-<version>-x86_64.AppImage` |
| Android | `MMdedit-<version>-android-arm64.apk` |

macOS est construit et contrôlé en intégration continue, par portabilité du
code, mais n'est pas distribué.

## Installation sous Windows

L'installeur pose :

- le programme et le Qt dont il dépend dans `%LOCALAPPDATA%\Programs\MMdedit`
  (aucune élévation requise ; l'assistant propose l'installation pour tous les
  utilisateurs si l'on dispose des droits d'administration) ;
- un raccourci au menu Démarrer, et sur le Bureau si la case est cochée ;
- MMdedit dans la liste **« Ouvrir avec »** des `.md`, `.markdown` et `.txt` ;
- une entrée de désinstallation dans *Programmes et fonctionnalités*.

L'installeur **ne s'impose pas comme application par défaut** : il s'ajoute à
`OpenWithProgids` sans toucher au `UserChoice` de l'utilisateur.

Son identifiant (`AppId`) est celui des versions PySide6 antérieures à 0.4 :
un poste qui en porte une reçoit celle-ci comme une mise à jour.

Installation silencieuse **pour tous les utilisateurs** — la forme à retenir
pour un déploiement automatisé, généralement exécuté sous SYSTEM :

```bat
MMdedit-<version>-setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /ALLUSERS /MERGETASKS=associer
"%ProgramFiles%\MMdedit\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES
```

`/ALLUSERS` n'est pas cosmétique : sans lui, `PrivilegesRequired=lowest`
installe dans le profil du compte appelant — celui de SYSTEM lors d'un
déploiement, invisible des utilisateurs et inopérant sur un serveur de bureaux
à distance.

## Installation sous Linux

```bash
chmod +x MMdedit-<version>-x86_64.AppImage
./MMdedit-<version>-x86_64.AppImage [fichier]
```

L'AppImage exige une glibc 2.35 ou plus récente : Debian 12, Ubuntu 22.04 et
leurs successeurs. Celle de la 0.4.1, fabriquée sur un système plus récent,
exigeait la glibc 2.39 et ne démarrait pas sur Debian 12.

## Installation sous Android

L'APK s'installe après avoir autorisé les sources inconnues pour l'application
qui l'ouvre (navigateur, gestionnaire de fichiers).

## Construction

Prérequis : Rust ≥ 1.85, Qt ≥ 6.4 (Core, Gui, Qml, Quick, QuickControls2,
QuickDialogs2, LabsSettings), CMake ≥ 3.24, Ninja. La liaison Rust ↔ Qt passe
par [cxx-qt](https://github.com/KDAB/cxx-qt) 0.10, récupéré par CMake.

```bash
cmake -S . -B _build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build _build
```

Sous Android, configurer avec le `qt-cmake` du Qt Android, `ANDROID_SDK_ROOT`,
`ANDROID_NDK_ROOT` et `QT_HOST_PATH`, puis construire la cible `apk`. La recette
complète de chaque cible — dont l'empaquetage — est dans
`.github/workflows/socle.yml`.

## Vérifications

```bash
cargo test --manifest-path core/Cargo.toml                      # noyau
QT_QPA_PLATFORM=offscreen ctest --test-dir _build --output-on-failure  # fumée
```

Le test de fumée (`mmdedit --smoke`) exerce une vraie fenêtre sans écran :
encadrement, bascules, annulation, recherche, remplacement, copie automatique
et sa protection, les trois apparences, puis relit le PDF produit.
`mmdedit --capture fichier.png` rend la fenêtre en image.

L'intégration continue le fait tourner **sur le paquet déployé** (dossier
windeployqt, AppImage), ouvre un fichier au nom et au contenu accentués, et
installe puis lance l'APK sur un émulateur Android 34.

## Arborescence

| Dossier | Contenu |
|---|---|
| `core/` | noyau Rust (`mmdedit_core`) et interface QML (`core/qml/`) |
| `cpp/` | point d'entrée et objets natifs Qt (texte, colorateur, presse-papier) |
| `build/` | installeur Inno Setup et ressources Windows |
| `packaging/` | fichier `.desktop`, contrôle de l'APK sur émulateur |
| `assets/` | icônes |

## Licence

**GNU General Public License version 3** ou ultérieure — copyleft. Texte
intégral dans [`LICENSE`](LICENSE), présenté à l'installation.

Quiconque reçoit le programme peut l'utiliser, l'étudier, le modifier et le
redistribuer, **à condition d'accorder les mêmes libertés**, code source
inclus, sous la même licence.

Qt est sous **LGPL v3**, compatible avec la GPL v3 ; cxx-qt est sous licence
MIT ou Apache 2.0.

## Historique

Jusqu'à la 0.3.0, MMdedit était écrit en Python (PySide6) et empaqueté par
PyInstaller. La cible Android, que Qt for Python ne dessert pas de façon
fiable, a imposé le passage à Rust / Qt 6 en 0.4.0. Le code PySide6 reste
consultable sous l'étiquette `v0.2.2`.
