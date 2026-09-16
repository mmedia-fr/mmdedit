#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Installe l'APK sur l'émulateur courant, le lance, et vérifie qu'il tient debout.
#
# Ce contrôle vit dans un fichier, et non dans le workflow : l'action
# android-emulator-runner exécute son « script » **ligne par ligne, chacune dans
# un shell distinct** — une variable posée à la première ligne est perdue à la
# suivante, ce qui fait échouer en silence toute séquence écrite naïvement.
set -euo pipefail

PAQUET=fr.mmedia.mmdedit
ATTENTE=20

apk=$(find _build/android-build -name '*.apk' | head -n 1)
if [ -z "$apk" ]; then
  echo "aucun APK construit sous _build/android-build" >&2
  exit 1
fi
echo "APK : $apk"

adb install -r "$apk"
adb logcat -c
adb shell monkey -p "$PAQUET" -c android.intent.category.LAUNCHER 1
sleep "$ATTENTE"

pid=$(adb shell pidof "$PAQUET" || true)
adb logcat -d -t 500 > logcat.txt

if [ -z "$pid" ]; then
  echo "MMdedit ne tourne plus $ATTENTE s apres son lancement :" >&2
  grep -iE 'mmdedit|AndroidRuntime|FATAL' logcat.txt | tail -40 >&2
  exit 1
fi

echo "MMdedit tourne (pid $pid)"
