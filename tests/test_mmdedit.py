"""Vérification fonctionnelle de MMdedit sous Qt (offscreen).

L'aperçu est anti-rebondi (QTimer 200 ms) : il faut faire tourner la boucle
d'événements, un simple processEvents() ne suffit pas.
"""

import os
import sys

from PySide6.QtCore import QEventLoop, QTimer
from PySide6.QtWidgets import QApplication

from mmdedit import __version__
from mmdedit.mainwindow import APP_NAME, MainWindow

app = QApplication(sys.argv)


def pump(ms):
    """Fait tourner la boucle d'événements pendant ms millisecondes."""
    loop = QEventLoop()
    QTimer.singleShot(ms, loop.quit)
    loop.exec()


ok = []


def check(label, condition, detail=""):
    ok.append(bool(condition))
    print(f"  {'OK ' if condition else 'ECHEC'}  {label}{(' — ' + detail) if detail else ''}")


w = MainWindow()
w.show()
pump(50)

print("Identite")
check("APP_NAME = MMdedit", APP_NAME == "MMdedit", APP_NAME)
check("version renseignee", __version__.count(".") == 2, __version__)
check("titre fenetre", w.windowTitle() == "Sans titre — MMdedit", w.windowTitle())

print("Apercu temps reel (anti-rebond 200 ms)")
w.editor.setPlainText("# Titre\n\nTexte **gras** et `code`.\n\n- item 1\n- item 2\n")
pump(400)
html = w.preview.toHtml()
check("titre h1 rendu", "<h1" in html)
check("liste rendue", "<li" in html)
check("gras rendu", "font-weight:700" in html or "<b" in html or "bold" in html)

print("Compteur")
txt = w._counter.text()
check("compteur alimente", any(c.isdigit() for c in txt), repr(txt))

print("Bascule des vues")
w.act_show_preview.setChecked(False)
w._toggle_preview(False)
pump(50)
check("apercu masque", not w.preview.isVisible())
check("editeur visible", w.editor.isVisible())
w.act_show_editor.setChecked(False)
w._toggle_editor(False)
pump(50)
check("derniere vue non masquable", w.editor.isVisible() or w.preview.isVisible())
w.act_show_preview.setChecked(True)
w._toggle_preview(True)
pump(50)

print("Fichier non-Markdown affiche en brut")
tmp = os.path.join(os.path.dirname(os.path.abspath(__file__)), "essai.csv")
with open(tmp, "w", encoding="utf-8") as fh:
    fh.write("col_a;col_b\n# pas un titre;valeur\n")
w.open_file(tmp)
pump(400)
check("csv en texte brut", "<h1" not in w.preview.toHtml())
check("titre reflete le fichier", "essai.csv" in w.windowTitle(), w.windowTitle())

print("Encodage")
lat = os.path.join(os.path.dirname(os.path.abspath(__file__)), "latin.txt")
with open(lat, "wb") as fh:
    fh.write("Éléments àéîôû — cp1252\n".encode("cp1252"))
w.open_file(lat)
pump(200)
check("repli cp1252", "Éléments" in w.editor.toPlainText(), w.editor.toPlainText().strip()[:30])

md = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ecrit.md")
w.editor.setPlainText("# Accentué\n\nLigne deux.\n")
w._write_to(md)
with open(md, "rb") as fh:
    raw = fh.read()
check("ecriture UTF-8 sans BOM", not raw.startswith(b"\xef\xbb\xbf"))
check("fins de ligne CRLF", b"\r\n" in raw)

print("Export PDF (rendu direct, hors boite de dialogue)")
from PySide6.QtGui import QTextDocument
from PySide6.QtPrintSupport import QPrinter

pdf = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sortie.pdf")
doc = QTextDocument()
doc.setMarkdown("# Titre\n\nTexte.\n")
pr = QPrinter(QPrinter.HighResolution)
pr.setOutputFormat(QPrinter.PdfFormat)
pr.setOutputFileName(pdf)
doc.print_(pr)
check("pdf produit", os.path.exists(pdf) and os.path.getsize(pdf) > 500,
      f"{os.path.getsize(pdf) if os.path.exists(pdf) else 0} octets")

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} controles OK")
sys.exit(0 if all(ok) else 1)
