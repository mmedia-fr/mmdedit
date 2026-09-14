# MMdedit — lecteur / editeur Markdown simple.
# Copyright (C) 2026 M-Media
#
# Ce programme est un logiciel libre : vous pouvez le redistribuer et/ou le
# modifier selon les termes de la Licence Publique Generale GNU telle que
# publiee par la Free Software Foundation, soit la version 3 de la Licence,
# soit (a votre choix) toute version ulterieure.
#
# Ce programme est distribue dans l'espoir qu'il sera utile, mais SANS AUCUNE
# GARANTIE, sans meme la garantie implicite de QUALITE MARCHANDE ou
# D'ADEQUATION A UN USAGE PARTICULIER. Voyez la Licence Publique Generale GNU
# pour plus de details.
#
# Vous devriez avoir recu une copie de la Licence Publique Generale GNU avec
# ce programme (fichier LICENSE). Sinon, voyez <https://www.gnu.org/licenses/>.

"""Génère l'icône de MMdedit à partir d'un logo.

Isole le « M » rouge du logo (détection de la teinte rouge, boîte englobante),
le centre sur un carré transparent avec une marge, puis écrit un .ico
multi-résolutions (PNG embarqués, format Vista+).

Usage :
    python build/make_icon.py chemin/logo.png

Sortie : src/mmdedit/assets/mmdedit.ico
"""

import os
import struct
import sys

from PySide6.QtCore import QBuffer, QByteArray, Qt
from PySide6.QtGui import QColor, QGuiApplication, QImage, QPainter

TAILLES = (16, 24, 32, 48, 64, 128, 256)
MARGE = 0.08  # marge relative autour du glyphe

ICI = os.path.dirname(os.path.abspath(__file__))
PROJET = os.path.dirname(ICI)
SORTIE = os.path.join(PROJET, "src", "mmdedit", "assets", "mmdedit.ico")

_app = None  # référence retenue sur le QGuiApplication (cf. main)


def est_rouge(couleur):
    """Vrai pour le rouge M-Media (#ED1C24 et voisins), faux pour noir/cyan/blanc."""
    r, v, b = couleur.red(), couleur.green(), couleur.blue()
    return r > 120 and r > v * 2 and r > b * 2


def boite_du_rouge(image):
    """Boîte englobante des pixels rouges, ou None."""
    x_min, y_min, x_max, y_max = image.width(), image.height(), -1, -1
    for y in range(image.height()):
        for x in range(image.width()):
            px = image.pixelColor(x, y)
            if px.alpha() > 32 and est_rouge(px):
                x_min = min(x_min, x)
                y_min = min(y_min, y)
                x_max = max(x_max, x)
                y_max = max(y_max, y)
    if x_max < 0:
        return None
    return x_min, y_min, x_max - x_min + 1, y_max - y_min + 1


def glyphe_carre(image, boite):
    """Recadre sur la boîte et centre sur un carré transparent avec marge."""
    x, y, larg, haut = boite
    decoupe = image.copy(x, y, larg, haut)

    cote = int(max(larg, haut) * (1 + 2 * MARGE))
    carre = QImage(cote, cote, QImage.Format_ARGB32)
    carre.fill(Qt.transparent)

    p = QPainter(carre)
    p.setRenderHint(QPainter.SmoothPixmapTransform)
    p.drawImage((cote - larg) // 2, (cote - haut) // 2, decoupe)
    p.end()
    return carre


def png_octets(image, taille):
    """Rend l'image à la taille voulue et retourne ses octets PNG."""
    vignette = image.scaled(
        taille, taille, Qt.KeepAspectRatio, Qt.SmoothTransformation
    )
    # Le QByteArray doit être référencé : passé en temporaire, il est détruit
    # et QBuffer écrit dans de la mémoire libérée (crash).
    octets = QByteArray()
    tampon = QBuffer(octets)
    tampon.open(QBuffer.WriteOnly)
    vignette.save(tampon, "PNG")
    tampon.close()
    return bytes(octets)


def ecrire_ico(chemin, images_png):
    """Écrit un .ico dont chaque entrée est un PNG (format Vista+)."""
    nb = len(images_png)
    entetes = b""
    donnees = b""
    decalage = 6 + 16 * nb

    for taille, brut in images_png:
        # 0 signifie 256 dans le format ICO.
        octet_taille = 0 if taille >= 256 else taille
        entetes += struct.pack(
            "<BBBBHHII",
            octet_taille,   # largeur
            octet_taille,   # hauteur
            0,              # palette
            0,              # réservé
            1,              # plans
            32,             # bits par pixel
            len(brut),      # taille des données
            decalage,       # décalage
        )
        donnees += brut
        decalage += len(brut)

    with open(chemin, "wb") as fh:
        fh.write(struct.pack("<HHH", 0, 1, nb))  # ICONDIR : réservé, type=icône, nb
        fh.write(entetes)
        fh.write(donnees)


def main():
    if len(sys.argv) < 2:
        print("Usage : python build/make_icon.py chemin/logo.png", file=sys.stderr)
        return 1

    source = sys.argv[1]
    if not os.path.isfile(source):
        print(f"Logo introuvable : {source}", file=sys.stderr)
        return 1

    # La référence doit être conservée : un QGuiApplication collecté par le
    # ramasse-miettes fait planter Qt (STATUS_STACK_BUFFER_OVERRUN).
    global _app
    _app = QGuiApplication(sys.argv)

    logo = QImage(source)
    if logo.isNull():
        print(f"Lecture impossible : {source}", file=sys.stderr)
        return 1
    logo = logo.convertToFormat(QImage.Format_ARGB32)

    boite = boite_du_rouge(logo)
    if boite is None:
        print("Aucun pixel rouge trouvé dans le logo.", file=sys.stderr)
        return 1
    print(f"Glyphe rouge détecté : x={boite[0]} y={boite[1]} l={boite[2]} h={boite[3]}")

    carre = glyphe_carre(logo, boite)
    os.makedirs(os.path.dirname(SORTIE), exist_ok=True)
    ecrire_ico(SORTIE, [(t, png_octets(carre, t)) for t in TAILLES])

    print(f"Écrit : {SORTIE} ({os.path.getsize(SORTIE)} octets, {len(TAILLES)} résolutions)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
