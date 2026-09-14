"""Formats non-Markdown, persistance de la sélection, icône."""

import os
import sys

from PySide6.QtGui import QTextCursor
from PySide6.QtWidgets import QApplication

from mmdedit.highlighter import MarkdownHighlighter, XmlHighlighter
from mmdedit.mainwindow import MainWindow, app_icon, resource_path

app = QApplication(sys.argv)
w = MainWindow()
w.show()

ICI = os.path.dirname(os.path.abspath(__file__))
ok = []


def check(label, cond, detail=""):
    ok.append(bool(cond))
    print(f"  {'OK   ' if cond else 'ECHEC'} {label}{(' — ' + detail) if detail else ''}")


def ecrire(nom, contenu):
    p = os.path.join(ICI, nom)
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(contenu)
    return p


def selectionner(texte, debut, fin):
    w.editor.setPlainText(texte)
    c = w.editor.textCursor()
    c.setPosition(debut)
    c.setPosition(fin, QTextCursor.KeepAnchor)
    w.editor.setTextCursor(c)


print("Icone — chemin resolu")
check("resource_path trouve le .ico", os.path.isfile(resource_path("assets", "mmdedit.ico")))
check("icone non nulle", not app_icon().isNull())
check("icone sur la fenetre", not w.windowIcon().isNull())
check("tailles multiples", len(app_icon().availableSizes()) >= 5,
      f"{len(app_icon().availableSizes())} tailles")

print("Ouverture d'un .md — apercu affiche, coloration Markdown")
md = ecrire("t.md", "# Titre\n\ntexte\n")
w.open_file(md)
check("format = markdown", w._format() == "markdown", w._format())
check("apercu visible", w.preview.isVisible())
check("highlighter Markdown", isinstance(w._highlighter, MarkdownHighlighter))
check("libelle barre d'etat", w._format_label.text() == "Markdown", w._format_label.text())

print("Ouverture d'un .xml — apercu masque, coloration XML")
xml = ecrire("t.xml", '<?xml version="1.0"?>\n<!-- commentaire -->\n<racine attr="valeur">\n  <enfant>texte &amp; suite</enfant>\n</racine>\n')
w.open_file(xml)
check("format = xml", w._format() == "xml", w._format())
check("apercu MASQUE", not w.preview.isVisible())
check("editeur visible", w.editor.isVisible())
check("highlighter XML", isinstance(w._highlighter, XmlHighlighter))
check("libelle barre d'etat", w._format_label.text() == "XML", w._format_label.text())

print("Ouverture d'un .csv — apercu masque, pas de coloration Markdown")
csv = ecrire("t.csv", "col_a;col_b\n# pas un titre;valeur\n")
w.open_file(csv)
check("format = texte", w._format() == "texte", w._format())
check("apercu MASQUE", not w.preview.isVisible())
check("aucun highlighter", w._highlighter is None)
check("libelle barre d'etat", w._format_label.text() == "Texte", w._format_label.text())

print("Retour a un .md — apercu retabli")
w.open_file(md)
check("apercu de nouveau visible", w.preview.isVisible())
check("highlighter Markdown reattache", isinstance(w._highlighter, MarkdownHighlighter))

print("Nouveau document — Markdown, apercu retabli")
w.open_file(csv)
w.new_file()
check("format = markdown", w._format() == "markdown")
check("apercu visible", w.preview.isVisible())

print("Selection conservee apres mise en forme")
selectionner("mot", 0, 3)
w._toggle_wrap("**")
c = w.editor.textCursor()
check("texte = **mot**", w.editor.toPlainText() == "**mot**", repr(w.editor.toPlainText()))
check("selection conservee", c.hasSelection())
check("selection = mot (hors marqueurs)", c.selectedText() == "mot", repr(c.selectedText()))

print("Reclic immediat = retrait, sans reselectionner")
w._toggle_wrap("**")
c = w.editor.textCursor()
check("texte = mot", w.editor.toPlainText() == "mot", repr(w.editor.toPlainText()))
check("selection toujours la", c.hasSelection())
check("selection = mot", c.selectedText() == "mot", repr(c.selectedText()))

print("Enchainement gras puis italique sur la meme selection")
selectionner("mot", 0, 3)
w._toggle_wrap("**")
w._toggle_wrap("*")
check("texte = ***mot***", w.editor.toPlainText() == "***mot***", repr(w.editor.toPlainText()))
check("selection = mot", w.editor.textCursor().selectedText() == "mot",
      repr(w.editor.textCursor().selectedText()))

print("Lien conserve aussi la selection")
selectionner("cible", 0, 5)
w._wrap("[", "](https://)")
check("texte = [cible](https://)", w.editor.toPlainText() == "[cible](https://)",
      repr(w.editor.toPlainText()))
check("selection = cible", w.editor.textCursor().selectedText() == "cible",
      repr(w.editor.textCursor().selectedText()))

print("Coloration XML — commentaire multiligne")
h = XmlHighlighter(w.editor.document())
check("etats de bloc definis", h.ETAT_COMMENTAIRE != h.ETAT_CDATA)
h.setDocument(None)

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} controles OK")
sys.exit(0 if all(ok) else 1)
