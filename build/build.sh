#!/usr/bin/env bash
# Construction de l'exécutable Linux / macOS MMdedit (PyInstaller, onefile).
# PyInstaller ne fait pas de compilation croisée : à lancer sur la plateforme cible.
# Sortie : dist/MMdedit (à la racine du projet, pas dans build/).

set -euo pipefail

cd "$(dirname "$0")/.."

PYTHON="${PYTHON:-python3}"

echo "[1/3] Installation des dépendances..."
"$PYTHON" -m pip install --disable-pip-version-check -q -r requirements.txt pyinstaller

echo "[2/3] Nettoyage des sorties précédentes..."
rm -f dist/MMdedit
rm -rf build/MMdedit

# Catalogues de traduction Qt : sans eux, les boutons standard des boîtes de
# dialogue restent en anglais dans l'exécutable.
QT_TR="$("$PYTHON" -c "import PySide6,os;print(os.path.join(os.path.dirname(PySide6.__file__),'translations'))")"

echo "[3/3] Construction..."
"$PYTHON" -m PyInstaller \
  --noconfirm \
  --onefile \
  --windowed \
  --name MMdedit \
  --paths src \
  --icon "$(pwd)/src/mmdedit/assets/mmdedit.ico" \
  --add-data "$(pwd)/src/mmdedit/assets:mmdedit/assets" \
  --add-data "$(pwd)/LICENSE:mmdedit" \
  --add-data "${QT_TR}/qtbase_fr.qm:mmdedit/translations" \
  --add-data "${QT_TR}/qt_fr.qm:mmdedit/translations" \
  --workpath build/MMdedit \
  --specpath build \
  run_mmdedit.py

echo
echo "Terminé : $(pwd)/dist/MMdedit"
