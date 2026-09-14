#!/usr/bin/env bash
# Installe MMdedit sur un Mac, pour l'utilisateur courant.
#
#   ./install-macos.sh                installe (ou met a jour)
#   ./install-macos.sh --desinstaller retire tout ce qui a ete pose
#
# Aucun privilege administrateur : un bundle ~/Applications/MMdedit.app est
# cree, adosse a un environnement virtuel Python dedie. PySide6 fournit Qt,
# rien n'est installe a l'echelle du systeme.
#
# MMdedit — lecteur / editeur Markdown simple.
# Copyright (C) 2026 M-Media — GNU GPL v3 ou ulterieure (voir LICENSE).

set -euo pipefail

APP="MMdedit"
VERSION="0.2.2"
BUNDLE="$HOME/Applications/$APP.app"
SUPPORT="$HOME/Library/Application Support/$APP"

ICI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rouge() { printf '\033[31m%s\033[0m\n' "$*"; }
vert()  { printf '\033[32m%s\033[0m\n' "$*"; }
info()  { printf '  %s\n' "$*"; }

# --------------------------------------------------------------------------
# Desinstallation
# --------------------------------------------------------------------------
desinstaller() {
    echo "Desinstallation de $APP..."
    rm -rf "$BUNDLE"  && info "application retiree"
    rm -rf "$SUPPORT" && info "donnees de support retirees"
    # Oblige le Finder a oublier les associations de l'application disparue.
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
        -kill -r -domain local -domain user >/dev/null 2>&1 || true
    vert "$APP desinstalle."
    exit 0
}

[ "${1:-}" = "--desinstaller" ] && desinstaller

# --------------------------------------------------------------------------
# Prerequis
# --------------------------------------------------------------------------
echo "Installation de $APP $VERSION"
echo

[ "$(uname -s)" = "Darwin" ] || { rouge "ECHEC : ce script est prevu pour macOS."; exit 1; }

# Le python3 d'Apple (/usr/bin/python3) declenche l'installation des outils en
# ligne de commande et convient ; Homebrew est prefere s'il est present.
PYTHON=""
for candidat in /opt/homebrew/bin/python3 /usr/local/bin/python3 /usr/bin/python3; do
    [ -x "$candidat" ] && { PYTHON="$candidat"; break; }
done
[ -n "$PYTHON" ] || { rouge "ECHEC : python3 introuvable. Installez-le : brew install python"; exit 1; }

PY_VERSION="$("$PYTHON" -c 'import sys; print("%d.%d" % sys.version_info[:2])')"
info "python3 $PY_VERSION ($PYTHON)"

ARCH="$(uname -m)"
info "architecture : $ARCH"

if [ -d "$ICI/../src/mmdedit" ]; then
    SOURCES="$(cd "$ICI/.." && pwd)"
elif [ -d "$ICI/src/mmdedit" ]; then
    SOURCES="$ICI"
else
    rouge "ECHEC : sources introuvables (src/mmdedit)."
    exit 1
fi
info "sources : $SOURCES"

# --------------------------------------------------------------------------
# Environnement virtuel + dependances
# --------------------------------------------------------------------------
echo
echo "[1/4] Environnement virtuel"
rm -rf "$SUPPORT/venv"
mkdir -p "$SUPPORT"
"$PYTHON" -m venv "$SUPPORT/venv"
info "cree dans $SUPPORT/venv"

echo "[2/4] Installation de PySide6 (telechargement, patientez)"
"$SUPPORT/venv/bin/python" -m pip install --quiet --upgrade pip
if ! "$SUPPORT/venv/bin/python" -m pip install --quiet "PySide6>=6.6"; then
    rouge "ECHEC : installation de PySide6 impossible."
    echo  "  Sur Mac Intel ancien, PySide6 peut exiger macOS 11 ou superieur."
    exit 1
fi
info "PySide6 $("$SUPPORT/venv/bin/python" -c 'import PySide6; print(PySide6.__version__)')"

echo "[3/4] Copie du programme"
rm -rf "$SUPPORT/src"
cp -r "$SOURCES/src" "$SUPPORT/src"
cp -f "$SOURCES/run_mmdedit.py" "$SUPPORT/"
[ -f "$SOURCES/LICENSE" ]   && cp -f "$SOURCES/LICENSE" "$SUPPORT/"
[ -f "$SOURCES/README.md" ] && cp -f "$SOURCES/README.md" "$SUPPORT/"
info "installe dans $SUPPORT"

# --------------------------------------------------------------------------
# Bundle .app
# --------------------------------------------------------------------------
echo "[4/4] Construction du bundle"
rm -rf "$BUNDLE"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"

cat > "$BUNDLE/Contents/MacOS/$APP" <<EOF
#!/usr/bin/env bash
# Lanceur de $APP — genere par install-macos.sh, ne pas editer.
exec "$SUPPORT/venv/bin/python" "$SUPPORT/run_mmdedit.py" "\$@"
EOF
chmod +x "$BUNDLE/Contents/MacOS/$APP"

# Icone : conversion du .ico en .icns via un iconset intermediaire.
ICO="$SUPPORT/src/mmdedit/assets/mmdedit.ico"
if [ -f "$ICO" ] && command -v iconutil >/dev/null 2>&1; then
    ICONSET="$(mktemp -d)/mmdedit.iconset"
    mkdir -p "$ICONSET"
    # offscreen : evite d'exiger une session graphique pour convertir l'icone
    # (installation par SSH, par exemple). L'echec eventuel est rattrape.
    QT_QPA_PLATFORM=offscreen "$SUPPORT/venv/bin/python" - "$ICO" "$ICONSET" <<'PY' || true
import os
import sys

from PySide6.QtGui import QGuiApplication, QImageReader

ico, iconset = sys.argv[1], sys.argv[2]
app = QGuiApplication([])
lecteur = QImageReader(ico)
# Noms de fichiers imposes par iconutil.
attendus = {16: "16x16", 32: "32x32", 128: "128x128", 256: "256x256"}
for i in range(lecteur.imageCount()):
    lecteur.jumpToImage(i)
    image = lecteur.read()
    if image.isNull() or image.width() not in attendus:
        continue
    nom = attendus[image.width()]
    image.save(os.path.join(iconset, f"icon_{nom}.png"))
    # Variante @2x attendue pour l'affichage Retina.
    double = attendus.get(image.width() // 2)
    if double:
        image.save(os.path.join(iconset, f"icon_{double}@2x.png"))
PY
    if iconutil -c icns "$ICONSET" -o "$BUNDLE/Contents/Resources/mmdedit.icns" 2>/dev/null; then
        info "icone convertie en .icns"
        ICONE="<key>CFBundleIconFile</key><string>mmdedit</string>"
    else
        info "conversion d'icone echouee, bundle sans icone"
        ICONE=""
    fi
    rm -rf "$(dirname "$ICONSET")"
else
    info "iconutil absent ou icone manquante, bundle sans icone"
    ICONE=""
fi

cat > "$BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$APP</string>
    <key>CFBundleDisplayName</key><string>$APP</string>
    <key>CFBundleExecutable</key><string>$APP</string>
    <key>CFBundleIdentifier</key><string>fr.mmedia.mmdedit</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    $ICONE
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key><string>Document Markdown</string>
            <key>CFBundleTypeRole</key><string>Editor</string>
            <key>LSHandlerRank</key><string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>net.daringfireball.markdown</string>
                <string>public.plain-text</string>
                <string>public.xml</string>
                <string>public.comma-separated-values-text</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF
info "bundle : $BUNDLE"

# Declare l'application aupres du Finder sans attendre son rafraichissement.
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
    -f "$BUNDLE" >/dev/null 2>&1 || true

echo
vert "$APP $VERSION installe."
echo
echo "  Lancer         : ouvrir $BUNDLE"
echo "                   ou Finder > Applications (dossier personnel)"
echo "  Ouvrir avec    : clic droit sur un .md > Ouvrir avec > $APP"
echo "  Desinstaller   : $0 --desinstaller"
echo
echo "  Note : l'application n'est pas signee ni notariee par Apple."
echo "         Au premier lancement, macOS peut la bloquer : faites un clic"
echo "         droit sur l'application puis « Ouvrir », et confirmez."
