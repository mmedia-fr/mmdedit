"""Un fichier fraichement ouvert doit-il etre marque comme modifie ? Non."""

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
    ("t2.md", "# Titre\n\n**gras** et `code`\n"),
    ("t2.xml", '<?xml version="1.0"?>\n<a attr="v">texte</a>\n'),
    ("t2.csv", "a;b\n1;2\n"),
]

w = MainWindow()
w.show()

ok = []
for nom, contenu in cas:
    p = ecrire(nom, contenu)
    w.open_file(p)
    juste_apres = w.editor.document().isModified()
    titre_apres = w.windowTitle()
    pump(400)  # laisse la coloration syntaxique s'appliquer
    apres_coloration = w.editor.document().isModified()
    titre_final = w.windowTitle()

    bon = (not juste_apres) and (not apres_coloration)
    ok.append(bon)
    print(f"  {'OK   ' if bon else 'ECHEC'} {nom}")
    print(f"        juste apres open_file : modifie={juste_apres} titre={titre_apres!r}")
    print(f"        apres coloration      : modifie={apres_coloration} titre={titre_final!r}")

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} formats sans faux 'modifie'")
sys.exit(0 if all(ok) else 1)
