"""Reglages persistants : geometrie, partage des vues, options, apparence."""

import os
import sys
import tempfile

from PySide6.QtCore import QEventLoop, QSettings, QTimer
from PySide6.QtWidgets import QApplication

from mmdedit import mainwindow as mw
from mmdedit.mainwindow import MainWindow

app = QApplication(sys.argv)

ok = []


def check(label, cond, detail=""):
    ok.append(bool(cond))
    print(f"  {'OK   ' if cond else 'ECHEC'} {label}{(' — ' + detail) if detail else ''}")


def pump(ms):
    loop = QEventLoop()
    QTimer.singleShot(ms, loop.quit)
    loop.exec()


def fichier_neuf(nom):
    return os.path.join(tempfile.mkdtemp(prefix="mmdedit-tests-"), nom)


# Les reglages sont injectes : jamais ceux de la machine qui execute les tests.
CHEMIN = fichier_neuf("reglages.ini")


def reglages():
    return QSettings(CHEMIN, QSettings.IniFormat)


print("Premiere session : reglages modifies, puis fermeture de la fenetre")
w1 = MainWindow(settings=reglages())
w1.show()
pump(100)
# Taille volontairement inferieure a l'ecran virtuel du mode offscreen
# (800x600) : au-dela, restoreGeometry ramene la fenetre a l'ecran et la
# comparaison porterait sur la contrainte, pas sur le reglage restaure.
w1.resize(700, 480)
w1.splitter.setSizes([300, 700])
w1.act_copie_auto.setChecked(False)
w1._choisir_theme("moderne")
pump(100)
check("apparence appliquee a chaud", "#e61e29" in (app.styleSheet() or ""))
w1.close()
pump(100)
check("fichier de reglages ecrit", os.path.isfile(CHEMIN), CHEMIN)

print("Seconde session : la fenetre revient telle qu'elle a ete laissee")
w2 = MainWindow(settings=reglages())
w2.show()
pump(100)
check("taille restauree", (w2.width(), w2.height()) == (700, 480),
      f"{w2.width()}x{w2.height()}")
gauche, droite = w2.splitter.sizes()[0], w2.splitter.sizes()[1]
check("partage des vues restaure", gauche < droite, f"{gauche} / {droite}")
check("copie automatique restauree (decochee)", not w2.act_copie_auto.isChecked())
check("apparence restauree", w2._theme == "moderne", w2._theme)
check("menu coche sur l'apparence active", w2._actions_theme["moderne"].isChecked())

print("Apparence systeme : aucune feuille de style imposee")
w2._choisir_theme("systeme")
pump(50)
check("feuille de style vide", not app.styleSheet(), repr(app.styleSheet()[:40]))
check("defaut macOS = systeme",
      mw.THEME_PAR_DEFAUT == ("systeme" if sys.platform == "darwin" else "classique"),
      mw.THEME_PAR_DEFAUT)

print("Apparence classique : look historique")
w2._choisir_theme("classique")
pump(50)
check("gris Windows 9x present", "#d4d0c8" in app.styleSheet())
w2.close()
pump(50)

print("Reglages absents : valeurs par defaut, sans erreur")
w3 = MainWindow(settings=QSettings(fichier_neuf("vide.ini"), QSettings.IniFormat))
w3.show()
pump(100)
check("copie automatique activee par defaut", w3.act_copie_auto.isChecked())
check("apparence par defaut", w3._theme == mw.THEME_PAR_DEFAUT, w3._theme)
check("cellule presse-papier presente",
      "Presse-papiers" in w3._presse_papier_label.text(),
      repr(w3._presse_papier_label.text()))
w3.close()

print()
print(f"RESULTAT : {sum(ok)}/{len(ok)} controles OK")
sys.exit(0 if all(ok) else 1)
