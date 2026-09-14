#!/usr/bin/env bash
# Installe MMdedit sur un poste Linux, pour l'utilisateur courant.
#
#   ./install-linux.sh                installe (ou met a jour)
#   ./install-linux.sh --desinstaller retire tout ce qui a ete pose
#
# Aucun privilege root : tout est place dans le profil de l'utilisateur.
# PySide6 est installe dans un environnement virtuel dedie, sans toucher au
# Python du systeme ni aux paquets de la distribution.
#
# MMdedit — lecteur / editeur Markdown simple.
# Copyright (C) 2026 M-Media — GNU GPL v3 ou ulterieure (voir LICENSE).

set -euo pipefail

APP="MMdedit"
VERSION="0.2.2"
PREFIXE="${XDG_DATA_HOME:-$HOME/.local/share}/mmdedit"
BIN="$HOME/.local/bin"
LANCEUR="$BIN/mmdedit"
BUREAU="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP="$BUREAU/mmdedit.desktop"
ICONES="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"

ICI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rouge()  { printf '\033[31m%s\033[0m\n' "$*"; }
vert()   { printf '\033[32m%s\033[0m\n' "$*"; }
info()   { printf '  %s\n' "$*"; }

# --------------------------------------------------------------------------
# Desinstallation
# --------------------------------------------------------------------------
desinstaller() {
    echo "Desinstallation de $APP..."
    rm -rf "$PREFIXE"                        && info "programme retire"
    rm -f  "$LANCEUR"                        && info "lanceur retire"
    rm -f  "$DESKTOP"                        && info "entree de menu retiree"
    find "$ICONES" -name 'mmdedit.png' -delete 2>/dev/null || true
    info "icones retirees"

    command -v update-desktop-database >/dev/null 2>&1 && \
        update-desktop-database "$BUREAU" 2>/dev/null || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 && \
        gtk-update-icon-cache -f -t "$ICONES" 2>/dev/null || true

    vert "$APP desinstalle."
    exit 0
}

[ "${1:-}" = "--desinstaller" ] && desinstaller

# --------------------------------------------------------------------------
# Prerequis
# --------------------------------------------------------------------------
echo "Installation de $APP $VERSION"
echo

if ! command -v python3 >/dev/null 2>&1; then
    rouge "ECHEC : python3 est introuvable."
    echo  "  Debian/Ubuntu : sudo apt install python3 python3-venv"
    echo  "  Fedora        : sudo dnf install python3"
    exit 1
fi

PY_VERSION="$(python3 -c 'import sys; print("%d.%d" % sys.version_info[:2])')"
info "python3 $PY_VERSION"

if ! python3 -c 'import venv' >/dev/null 2>&1; then
    rouge "ECHEC : le module venv est absent."
    echo  "  Debian/Ubuntu : sudo apt install python3-venv"
    exit 1
fi

# Les sources doivent accompagner ce script : soit a cote (arborescence du
# depot), soit dans l'archive livree avec lui.
if [ -d "$ICI/../src/mmdedit" ]; then
    SOURCES="$(cd "$ICI/.." && pwd)"
elif [ -d "$ICI/src/mmdedit" ]; then
    SOURCES="$ICI"
else
    rouge "ECHEC : sources introuvables (src/mmdedit)."
    echo  "  Ce script doit etre lance depuis l'archive des sources de MMdedit."
    exit 1
fi
info "sources : $SOURCES"

# --------------------------------------------------------------------------
# Environnement virtuel + dependances
# --------------------------------------------------------------------------
echo
echo "[1/4] Environnement virtuel"
rm -rf "$PREFIXE/venv"
mkdir -p "$PREFIXE"
python3 -m venv "$PREFIXE/venv"
info "cree dans $PREFIXE/venv"

echo "[2/4] Installation de PySide6 (telechargement, patientez)"
"$PREFIXE/venv/bin/python" -m pip install --quiet --upgrade pip
if ! "$PREFIXE/venv/bin/python" -m pip install --quiet "PySide6>=6.6"; then
    rouge "ECHEC : installation de PySide6 impossible."
    echo  "  Verifiez la connexion reseau, ou installez les dependances Qt"
    echo  "  systeme : libgl1, libxkbcommon-x11-0, libegl1."
    exit 1
fi
info "PySide6 $("$PREFIXE/venv/bin/python" -c 'import PySide6; print(PySide6.__version__)')"

# --------------------------------------------------------------------------
# Programme
# --------------------------------------------------------------------------
echo "[3/4] Copie du programme"
rm -rf "$PREFIXE/src"
cp -r "$SOURCES/src" "$PREFIXE/src"
cp -f "$SOURCES/run_mmdedit.py" "$PREFIXE/"
[ -f "$SOURCES/LICENSE" ] && cp -f "$SOURCES/LICENSE" "$PREFIXE/"
[ -f "$SOURCES/README.md" ] && cp -f "$SOURCES/README.md" "$PREFIXE/"
info "installe dans $PREFIXE"

mkdir -p "$BIN"
cat > "$LANCEUR" <<EOF
#!/usr/bin/env bash
# Lanceur de $APP — genere par install-linux.sh, ne pas editer.
exec "$PREFIXE/venv/bin/python" "$PREFIXE/run_mmdedit.py" "\$@"
EOF
chmod +x "$LANCEUR"
info "lanceur : $LANCEUR"

# --------------------------------------------------------------------------
# Integration au bureau
# --------------------------------------------------------------------------
echo "[4/4] Integration au bureau"

# L'icone est livree au format .ico : on en extrait des PNG aux tailles
# standard, seul format compris par les environnements de bureau Linux.
ICO="$PREFIXE/src/mmdedit/assets/mmdedit.ico"
if [ -f "$ICO" ]; then
    # QT_QPA_PLATFORM=offscreen : sans cela, QGuiApplication tente de charger le
    # greffon « xcb » et avorte sur une machine sans serveur graphique (poste en
    # SSH, conteneur, serveur). L'echec est rattrape : une icone est cosmetique,
    # elle ne doit pas faire echouer l'installation entiere.
    if ! QT_QPA_PLATFORM=offscreen "$PREFIXE/venv/bin/python" - "$ICO" "$ICONES" <<'PY'
import os
import sys

from PySide6.QtGui import QGuiApplication, QImageReader

ico, racine = sys.argv[1], sys.argv[2]
app = QGuiApplication([])          # requis pour manipuler des images
lecteur = QImageReader(ico)
poses = 0
for i in range(lecteur.imageCount()):
    lecteur.jumpToImage(i)
    image = lecteur.read()
    if image.isNull():
        continue
    taille = image.width()
    dossier = os.path.join(racine, f"{taille}x{taille}", "apps")
    os.makedirs(dossier, exist_ok=True)
    if image.save(os.path.join(dossier, "mmdedit.png")):
        poses += 1
print(f"  {poses} icones posees")
PY
    then
        info "icones non generees (sans consequence sur le fonctionnement)"
    fi
else
    info "icone absente, ignoree"
fi

mkdir -p "$BUREAU"
cat > "$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=$APP
GenericName=Éditeur Markdown
Comment=Lecteur / éditeur Markdown simple
Exec=$LANCEUR %f
Icon=mmdedit
Terminal=false
Categories=Utility;TextEditor;Development;
MimeType=text/markdown;text/x-markdown;text/plain;application/xml;text/xml;text/csv;
Keywords=markdown;md;texte;editeur;
StartupNotify=true
EOF
info "entree de menu : $DESKTOP"

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$BUREAU" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && \
    gtk-update-icon-cache -f -t "$ICONES" 2>/dev/null || true

echo
vert "$APP $VERSION installe."
echo
echo "  Lancer         : mmdedit [fichier]"
echo "  Menu           : chercher « $APP » dans les applications"
echo "  Desinstaller   : $0 --desinstaller"
echo
case ":$PATH:" in
    *":$BIN:"*) ;;
    *) echo "  Note : $BIN n'est pas dans votre PATH."
       echo "         Ajoutez : export PATH=\"\$PATH:$BIN\"" ;;
esac
