"""Vérifie les bascules de mise en forme et l'icône de MMdedit."""

import os
import sys

from PySide6.QtGui import QTextCursor
from PySide6.QtWidgets import QApplication

from mmdedit.mainwindow import MainWindow, app_icon, resource_path

app = QApplication(sys.argv)
w = MainWindow()

ok = []


def check(label, condition, detail=""):
    ok.append(bool(condition))
    print(f"  {'OK   ' if condition else 'ECHEC'} {label}{(' — ' + detail) if detail else ''}")


def selectionner(texte, debut, fin):
    w.editor.setPlainText(texte)
    c = w.editor.textCursor()
    c.setPosition(debut)
    c.setPosition(fin, QTextCursor.KeepAnchor)
    w.editor.setTextCursor(c)


print("Icone")
chemin = resource_path("assets", "mmdedit.ico")
check("fichier .ico present", os.path.isfile(chemin), chemin)
check("icone chargee", not app_icon().isNull())
check("icone posee sur la fenetre", not w.windowIcon().isNull())

print("Bascule gras — pose puis retrait")
selectionner("mot", 0, 3)
w._toggle_wrap("**")
check("pose", w.editor.toPlainText() == "**mot**", repr(w.editor.toPlainText()))
# La sélection couvre maintenant « mot » sans les marqueurs : cas 2 (extérieur).
selectionner("**mot**", 2, 5)
w._toggle_wrap("**")
check("retrait (marqueurs exterieurs)", w.editor.toPlainText() == "mot", repr(w.editor.toPlainText()))
# Sélection incluant les marqueurs : cas 1 (intérieur).
selectionner("**mot**", 0, 7)
w._toggle_wrap("**")
check("retrait (marqueurs inclus)", w.editor.toPlainText() == "mot", repr(w.editor.toPlainText()))

print("Bascule italique — ne doit pas confondre avec le gras")
selectionner("mot", 0, 3)
w._toggle_wrap("*")
check("pose italique", w.editor.toPlainText() == "*mot*", repr(w.editor.toPlainText()))
selectionner("*mot*", 1, 4)
w._toggle_wrap("*")
check("retrait italique", w.editor.toPlainText() == "mot", repr(w.editor.toPlainText()))

print("Bascule barre et code")
selectionner("mot", 0, 3)
w._toggle_wrap("~~")
check("pose barre", w.editor.toPlainText() == "~~mot~~", repr(w.editor.toPlainText()))
selectionner("~~mot~~", 0, 7)
w._toggle_wrap("~~")
check("retrait barre", w.editor.toPlainText() == "mot", repr(w.editor.toPlainText()))
selectionner("mot", 0, 3)
w._toggle_wrap("`")
check("pose code", w.editor.toPlainText() == "`mot`", repr(w.editor.toPlainText()))

print("Sans selection : pose la paire, curseur au milieu")
w.editor.setPlainText("")
c = w.editor.textCursor()
c.setPosition(0)
w.editor.setTextCursor(c)
w._toggle_wrap("**")
pos = w.editor.textCursor().position()
check("paire posee", w.editor.toPlainText() == "****", repr(w.editor.toPlainText()))
check("curseur au milieu", pos == 2, str(pos))

print("Bascule des prefixes de ligne")
w.editor.setPlainText("Une ligne")
w._toggle_prefix("# ")
check("pose Titre 1", w.editor.toPlainText() == "# Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("# ")
check("retrait Titre 1", w.editor.toPlainText() == "Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("## ")
check("pose Titre 2", w.editor.toPlainText() == "## Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("### ")
check("remplacement T2 -> T3", w.editor.toPlainText() == "### Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("# ")
check("remplacement T3 -> T1", w.editor.toPlainText() == "# Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("- ")
check("remplacement titre -> liste", w.editor.toPlainText() == "- Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("1. ")
check("remplacement liste -> numerotee", w.editor.toPlainText() == "1. Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("> ")
check("remplacement numerotee -> citation", w.editor.toPlainText() == "> Une ligne", repr(w.editor.toPlainText()))
w._toggle_prefix("> ")
check("retrait citation", w.editor.toPlainText() == "Une ligne", repr(w.editor.toPlainText()))

print("Prefixe sur la ligne du curseur uniquement")
w.editor.setPlainText("ligne un\nligne deux")
c = w.editor.textCursor()
c.setPosition(12)  # dans « ligne deux »
w.editor.setTextCursor(c)
w._toggle_prefix("# ")
check("2e ligne seule prefixee", w.editor.toPlainText() == "ligne un\n# ligne deux", repr(w.editor.toPlainText()))

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} controles OK")
sys.exit(0 if all(ok) else 1)
