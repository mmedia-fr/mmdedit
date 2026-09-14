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

"""Feuilles de style (apparences) et textes statiques.

Trois apparences sont proposees : « classique » (look Windows 9x, l'aspect
historique de MMdedit), « moderne » (plat, aux couleurs M-Media) et
« systeme » (aucune feuille de style : le style natif de la plateforme).
Sur macOS, le look Windows 9x jure avec le reste du bureau : le defaut y est
« systeme », qui rend l'application conforme aux autres applications Mac.
"""

import sys

# QSS appliqué à toute l'application : gris institutionnel, reliefs
# biseautés, champs texte enfoncés. Volontairement sobre et daté.
OLDSCHOOL_QSS = """
QMainWindow, QDialog, QWidget {
    background-color: #d4d0c8;
    color: #000000;
    font-family: "MS Shell Dlg 2", "Tahoma", "Sans Serif";
    font-size: 9pt;
}
QMenuBar {
    background-color: #d4d0c8;
    border-bottom: 1px solid #808080;
}
QMenuBar::item:selected {
    background-color: #000080;
    color: #ffffff;
}
QMenu {
    background-color: #d4d0c8;
    border: 2px outset #ffffff;
}
QMenu::item:selected {
    background-color: #000080;
    color: #ffffff;
}
QPlainTextEdit {
    background-color: #ffffff;
    color: #000000;
    border: 2px inset #808080;
    font-family: "Courier New", "Consolas", monospace;
    font-size: 11pt;
    selection-background-color: #000080;
    selection-color: #ffffff;
}
QTextBrowser {
    background-color: #ffffff;
    color: #000000;
    border: 2px inset #808080;
}
QToolBar {
    background-color: #d4d0c8;
    border: 0px;
    border-bottom: 1px solid #808080;
    spacing: 2px;
    padding: 2px;
}
QToolButton {
    background-color: #d4d0c8;
    border: 2px outset #ffffff;
    padding: 1px 5px;
    min-width: 20px;
}
QToolButton:pressed, QToolButton:checked {
    border: 2px inset #808080;
    background-color: #c0bcb4;
}
QPushButton {
    background-color: #d4d0c8;
    border: 2px outset #ffffff;
    padding: 3px 12px;
    min-width: 60px;
}
QPushButton:pressed {
    border: 2px inset #808080;
}
QPushButton:focus {
    border: 2px outset #ffffff;
}
QLineEdit {
    background-color: #ffffff;
    border: 2px inset #808080;
    padding: 1px 2px;
}
QCheckBox {
    spacing: 5px;
}
QStatusBar {
    background-color: #d4d0c8;
    border-top: 1px solid #808080;
}
QStatusBar::item {
    border: none;
}
QLabel#statusCell {
    border: 1px inset #808080;
    padding: 1px 6px;
    background-color: #d4d0c8;
}
QSplitter::handle {
    background-color: #d4d0c8;
}
"""

# Apparence « moderne » : surfaces plates, bordures fines, coins legerement
# arrondis, aux couleurs de la charte M-Media — rouge #e61e29, cyan #21abe3,
# anthracite #414042. Le cyan sert de couleur de selection (lisible sur de
# longues plages de texte) ; le rouge marque l'element actif ou survole.
MODERNE_QSS = """
QMainWindow, QDialog, QWidget {
    background-color: #f6f7f8;
    color: #231f20;
    font-family: "Segoe UI", "Noto Sans", "Helvetica Neue", "Sans Serif";
    font-size: 10pt;
}
QMenuBar {
    background-color: #ffffff;
    border-bottom: 1px solid #e2e4e7;
    padding: 2px;
}
QMenuBar::item {
    padding: 4px 10px;
    border-radius: 4px;
}
QMenuBar::item:selected {
    background-color: #e61e29;
    color: #ffffff;
}
QMenu {
    background-color: #ffffff;
    border: 1px solid #d6d9dd;
    padding: 4px;
}
QMenu::item {
    padding: 5px 22px 5px 22px;
    border-radius: 4px;
}
QMenu::item:selected {
    background-color: #e61e29;
    color: #ffffff;
}
QMenu::separator {
    height: 1px;
    background-color: #e2e4e7;
    margin: 4px 8px;
}
QPlainTextEdit {
    background-color: #ffffff;
    color: #231f20;
    border: 1px solid #d6d9dd;
    border-radius: 6px;
    padding: 4px;
    font-family: "Cascadia Mono", "Consolas", "Noto Sans Mono", monospace;
    font-size: 11pt;
    selection-background-color: #21abe3;
    selection-color: #ffffff;
}
QTextBrowser {
    background-color: #ffffff;
    color: #231f20;
    border: 1px solid #d6d9dd;
    border-radius: 6px;
    padding: 4px;
    selection-background-color: #21abe3;
    selection-color: #ffffff;
}
QToolBar {
    background-color: #ffffff;
    border: 0px;
    border-bottom: 1px solid #e2e4e7;
    spacing: 4px;
    padding: 4px;
}
QToolButton {
    background-color: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    padding: 3px 8px;
    min-width: 20px;
}
QToolButton:hover {
    background-color: #f0f1f3;
    border: 1px solid #d6d9dd;
}
QToolButton:pressed, QToolButton:checked {
    background-color: #e61e29;
    color: #ffffff;
    border: 1px solid #c61821;
}
QPushButton {
    background-color: #ffffff;
    border: 1px solid #c9ced4;
    border-radius: 5px;
    padding: 5px 14px;
    min-width: 72px;
}
QPushButton:hover {
    border: 1px solid #21abe3;
}
QPushButton:pressed {
    background-color: #f0f1f3;
}
QPushButton:focus {
    border: 1px solid #21abe3;
}
QLineEdit {
    background-color: #ffffff;
    border: 1px solid #c9ced4;
    border-radius: 5px;
    padding: 4px 6px;
    selection-background-color: #21abe3;
    selection-color: #ffffff;
}
QLineEdit:focus {
    border: 1px solid #21abe3;
}
QCheckBox {
    spacing: 6px;
}
QStatusBar {
    background-color: #ffffff;
    border-top: 1px solid #e2e4e7;
}
QStatusBar::item {
    border: none;
}
QLabel#statusCell {
    border: 1px solid #e2e4e7;
    border-radius: 4px;
    padding: 1px 8px;
    background-color: #f6f7f8;
    color: #414042;
}
QSplitter::handle {
    background-color: #e2e4e7;
    width: 3px;
}
QScrollBar:vertical, QScrollBar:horizontal {
    background: #f6f7f8;
    border: none;
    width: 11px;
    height: 11px;
}
QScrollBar::handle:vertical, QScrollBar::handle:horizontal {
    background: #c9ced4;
    border-radius: 5px;
    min-height: 24px;
    min-width: 24px;
}
QScrollBar::handle:hover {
    background: #a8afb7;
}
QScrollBar::add-line, QScrollBar::sub-line {
    height: 0px;
    width: 0px;
}
"""

# Apparences proposees par le menu « Affichage > Apparence ».
# cle -> (libelle du menu, feuille de style ; None = style natif, sans QSS).
THEMES = {
    "classique": ("&Classique (Windows 9x)", OLDSCHOOL_QSS),
    "moderne": ("&Moderne (M-Media)", MODERNE_QSS),
    "systeme": ("&Systeme (aspect natif)", None),
}

# Sur macOS, l'aspect natif est le defaut attendu ; ailleurs, MMdedit garde
# son look historique tant que l'utilisateur n'en decide pas autrement.
THEME_PAR_DEFAUT = "systeme" if sys.platform == "darwin" else "classique"


# Mémo affiché par « Aide > Rappel des balises ». Rendu tel quel dans un
# QTextBrowser en Markdown, il sert donc aussi de démonstration.
TAGS_HELP = """\
# Rappel des balises Markdown

## Titres
`# Titre 1`  ·  `## Titre 2`  ·  `### Titre 3`

## Emphase
`**gras**`  ·  `*italique*`  ·  `` `code` ``  ·  `~~barré~~`

## Listes
- `- élément` (liste à puces)
- `1. élément` (liste numérotée)
- Indenter de deux espaces pour un sous-niveau.

## Liens et images
`[texte](https://exemple.fr)`
`![texte alternatif](chemin/image.png)`

## Citation
`> ligne citée`

## Bloc de code
Entourer par trois accents graves ``` sur leur propre ligne.

## Séparateur horizontal
`---`

## Tableau
`| Colonne A | Colonne B |`
`| --- | --- |`
`| valeur 1 | valeur 2 |`
"""
