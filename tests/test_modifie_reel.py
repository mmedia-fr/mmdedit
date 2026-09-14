"""Meme controle qu'en offscreen, mais avec un vrai rendu a l'ecran.

La coloration syntaxique ne s'execute qu'au moment ou le widget est peint :
en offscreen elle peut ne jamais tourner, ce qui masque un faux marquage
« modifie ».
"""

import os
import sys

from PySide6.QtCore import QEventLoop, QTimer
from PySide6.QtWidgets import QApplication

from mmdedit.mainwindow import MainWindow

app = QApplication(sys.argv)
ICI = os.path.dirname(os.path.abspath(__file__))


def pump(ms):
    loop = QEventLoop()
    QTimer.singleShot(ms, loop.quit)
    loop.exec()


def ecrire(nom, contenu):
    p = os.path.join(ICI, nom)
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(contenu)
    return p


cas = [
    ("r.md", "# Titre\n\n**gras**, `code`, [lien](http://x)\n\n- a\n- b\n"),
    ("r.xml", '<?xml version="1.0"?>\n<!-- c -->\n<a attr="v">t &amp; u</a>\n'),
    ("r.csv", "a;b\n1;2\n"),
]

w = MainWindow()
w.show()
pump(600)  # laisse la fenetre se peindre reellement

resultats = []
for nom, contenu in cas:
    p = ecrire(nom, contenu)
    w.open_file(p)
    pump(900)  # peinture + coloration + anti-rebond de l'apercu
    modifie = w.editor.document().isModified()
    resultats.append((nom, modifie, w.windowTitle()))
    print(f"  {nom} : modifie={modifie} titre={w.windowTitle()!r}")

w.close()
mauvais = [n for n, m, _ in resultats if m]
print()
if mauvais:
    print(f"ECHEC : faux marquage 'modifie' sur {', '.join(mauvais)}")
    sys.exit(1)
print("OK : aucun faux marquage 'modifie' en rendu reel")
sys.exit(0)
