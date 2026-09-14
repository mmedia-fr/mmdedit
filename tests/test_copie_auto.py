"""Copie automatique de la selection, avec protection du presse-papier."""

import sys
import time

from PySide6.QtCore import QEventLoop, QTimer
from PySide6.QtGui import QTextCursor
from PySide6.QtWidgets import QApplication

from mmdedit import mainwindow as mw
from mmdedit.mainwindow import MainWindow

app = QApplication(sys.argv)
presse_papier = QApplication.clipboard()
w = MainWindow()
w.show()

ok = []


def check(label, cond, detail=""):
    ok.append(bool(cond))
    print(f"  {'OK   ' if cond else 'ECHEC'} {label}{(' — ' + detail) if detail else ''}")


def pump(ms):
    loop = QEventLoop()
    QTimer.singleShot(ms, loop.quit)
    loop.exec()


def selectionner(debut, fin):
    c = w.editor.textCursor()
    c.setPosition(debut)
    c.setPosition(fin, QTextCursor.KeepAnchor)
    w.editor.setTextCursor(c)
    pump(500)  # au-dela du delai de stabilisation


print("Reglages")
check("option activee par defaut", w.act_copie_auto.isChecked())
check("delai de protection = 60 s", mw.DELAI_PROTECTION_S == 60, str(mw.DELAI_PROTECTION_S))

print("Presse-papier vierge : la selection est copiee")
presse_papier.clear()
pump(150)
w._presse_papier_horodatage = float("-inf")  # rien a proteger
w.editor.setPlainText("alpha beta gamma")
pump(150)
selectionner(0, 5)
check("presse-papier = alpha", presse_papier.text() == "alpha", repr(presse_papier.text()))

print("Nouvelle selection : elle remplace la precedente (copie interne non protegee)")
selectionner(6, 10)
check("presse-papier = beta", presse_papier.text() == "beta", repr(presse_papier.text()))

print("Depot etranger recent : la selection ne l'ecrase pas")
presse_papier.setText("CONTENU IMPORTANT")
pump(150)
check("depot horodate", w._presse_papier_horodatage > float("-inf"))
check("protection active", w._presse_papier_protege())
selectionner(11, 16)
check("presse-papier intact", presse_papier.text() == "CONTENU IMPORTANT",
      repr(presse_papier.text()))

print("Depot etranger perime (>60 s) : la selection reprend la main")
w._presse_papier_horodatage = time.monotonic() - (mw.DELAI_PROTECTION_S + 1)
check("protection expiree", not w._presse_papier_protege())
selectionner(0, 5)
check("presse-papier = alpha", presse_papier.text() == "alpha", repr(presse_papier.text()))

print("Option desactivee : plus aucune copie")
w.act_copie_auto.setChecked(False)
presse_papier.setText("FIXE")
pump(150)
w._presse_papier_horodatage = float("-inf")
selectionner(6, 10)
check("presse-papier inchange", presse_papier.text() == "FIXE", repr(presse_papier.text()))
w.act_copie_auto.setChecked(True)

print("Selection vide : ne vide pas le presse-papier")
presse_papier.setText("GARDE")
pump(150)
w._presse_papier_horodatage = float("-inf")
c = w.editor.textCursor()
c.setPosition(3)
w.editor.setTextCursor(c)
pump(500)
check("presse-papier conserve", presse_papier.text() == "GARDE", repr(presse_papier.text()))

print("Selection multiligne : les retours a la ligne sont reels")
w.editor.setPlainText("ligne un\nligne deux")
pump(150)
w._presse_papier_horodatage = float("-inf")
selectionner(0, 18)
t = presse_papier.text()
check("contient un \\n reel", "\n" in t, repr(t))
check("aucun separateur Qt residuel",
      mw.PARAGRAPHE_QT not in t and mw.LIGNE_QT not in t, repr(t))

print("Echo multiple d'une meme ecriture : pas d'auto-protection")
# Windows emet souvent plusieurs dataChanged pour un seul setText. Le signal
# surnumeraire etait pris pour un depot etranger : MMdedit se protegeait
# alors contre sa propre ecriture, et la selection suivante etait refusee.
w.editor.setPlainText("alpha beta gamma")
pump(150)
w._presse_papier_horodatage = float("-inf")
selectionner(0, 5)
w._on_presse_papier_change()          # signal surnumeraire, meme contenu
check("aucune protection declenchee", not w._presse_papier_protege())
check("origine = saisie", w._origine_presse_papier == "saisie",
      w._origine_presse_papier)
selectionner(6, 10)
check("selection suivante copiee", presse_papier.text() == "beta",
      repr(presse_papier.text()))

print("Selection dans l'apercu rendu : copiee elle aussi")
w.editor.setPlainText("**gras** et suite")
pump(400)                              # laisse le rendu se reconstruire
w._presse_papier_horodatage = float("-inf")
rendu = w.preview.toPlainText()
check("apercu rendu sans marqueurs", "gras" in rendu and "**" not in rendu,
      repr(rendu))
c = w.preview.textCursor()
c.setPosition(0)
c.setPosition(4, QTextCursor.KeepAnchor)
w.preview.setTextCursor(c)
pump(500)
check("presse-papier = gras", presse_papier.text() == "gras",
      repr(presse_papier.text()))
check("origine = rendu", w._origine_presse_papier == "rendu",
      w._origine_presse_papier)

print("Cellule d'etat : origine affichee")
check("cellule mentionne le rendu", "rendu" in w._presse_papier_label.text(),
      repr(w._presse_papier_label.text()))

print("Ctrl+C suit la vue qui a le focus")
w.editor.setPlainText("un deux trois")
pump(150)
w._presse_papier_horodatage = float("-inf")
presse_papier.setText("AVANT")
pump(150)
w.preview.setFocus()
pump(100)
if w.preview.hasFocus():
    c = w.preview.textCursor()
    c.setPosition(0)
    c.setPosition(2, QTextCursor.KeepAnchor)
    w.preview.setTextCursor(c)
    pump(50)
    w.act_copy.trigger()
    pump(150)
    check("Ctrl+C copie depuis l'apercu",
          presse_papier.text() == w.preview.textCursor().selectedText(),
          repr(presse_papier.text()))
    check("origine = rendu", w._origine_presse_papier == "rendu",
          w._origine_presse_papier)
else:
    # Le mode offscreen n'accorde pas toujours le focus clavier : la
    # verification est alors reportee sur la suite en rendu reel.
    print("  IGNORE focus indisponible en mode offscreen")

print("Ctrl+C protege le presse-papier (depot explicite)")
w.editor.setFocus()
pump(50)
w._presse_papier_horodatage = float("-inf")
selectionner(0, 2)
w.act_copy.trigger()
pump(150)
check("protection active apres Ctrl+C", w._presse_papier_protege())

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} controles OK")
sys.exit(0 if all(ok) else 1)
