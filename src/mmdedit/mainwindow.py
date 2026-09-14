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

"""Fenêtre principale de MMdedit."""

import os
import sys
import time

from PySide6.QtCore import (
    QLibraryInfo,
    QLocale,
    QSettings,
    Qt,
    QTimer,
    QTranslator,
)
from PySide6.QtGui import (
    QAction,
    QActionGroup,
    QIcon,
    QKeySequence,
    QTextCursor,
    QTextDocument,
)
from PySide6.QtPrintSupport import QPrinter
from PySide6.QtWidgets import (
    QApplication,
    QFileDialog,
    QLabel,
    QMainWindow,
    QMessageBox,
    QPlainTextEdit,
    QSplitter,
    QTextBrowser,
    QToolBar,
    QWidget,
)

from . import __version__
from .finddialog import FindReplaceDialog
from .highlighter import MarkdownHighlighter, XmlHighlighter
from .style import TAGS_HELP, THEME_PAR_DEFAUT, THEMES

APP_NAME = "MMdedit"
# Editeur au sens QSettings : determine ou sont ecrits les reglages
# (registre HKCU\Software\M-Media\MMdedit sous Windows, ~/.config ailleurs).
ORG_NAME = "M-Media"

# Filtres de la boîte de dialogue Ouvrir / Enregistrer.
FILE_FILTERS = (
    "Markdown (*.md *.markdown);;"
    "Texte (*.txt);;"
    "CSV (*.csv);;"
    "Tous les fichiers (*)"
)

# Extensions considérées comme Markdown pour l'aperçu rendu. Les autres
# fichiers texte sont affichés tels quels (texte brut) dans l'aperçu.
MARKDOWN_EXT = {".md", ".markdown"}

# Formats balisés : coloration XML et aperçu inutile (le rendu Markdown n'a
# aucun sens sur du XML — on veut voir la structure du fichier).
XML_EXT = {
    ".xml", ".xsd", ".xsl", ".xslt", ".svg", ".rss", ".atom",
    ".html", ".htm", ".xhtml", ".plist", ".config", ".csproj", ".props",
}

# Copie automatique de la sélection dans le presse-papier (façon sélection
# primaire X11, absente de Windows). Un contenu déposé dans le presse-papier
# par un tiers — ou par un Ctrl+C — est protégé pendant ce délai : la copie
# automatique ne l'écrase pas tant qu'il est « frais ».
# QTextCursor.selectedText() rend les fins de ligne sous forme de
# separateurs Unicode, pas de \n. Constantes nommees : ces caracteres
# sont invisibles dans un editeur et se perdent au premier copier-coller.
PARAGRAPHE_QT = "\u2029"
LIGNE_QT = "\u2028"

# Traducteurs Qt : doivent rester r\u00e9f\u00e9renc\u00e9s au niveau du module, sinon le
# ramasse-miettes les emporte et les libell\u00e9s repassent en anglais.
_traducteurs = []

COPIE_AUTO_PAR_DEFAUT = True
DELAI_PROTECTION_S = 60

# Fenetre pendant laquelle un depot dans le presse-papier identique a ce que
# nous venons d'y ecrire est reconnu comme notre propre echo. Windows emet
# souvent plusieurs dataChanged pour une seule ecriture (notification
# systeme, plus les ouvertures du presse-papier par les gestionnaires
# tiers) : un drapeau consomme par le premier signal laissait le second
# passer pour un depot etranger, et MMdedit se protegeait alors contre sa
# propre ecriture pendant DELAI_PROTECTION_S.
DELAI_ECHO_S = 1.0

# Libelles de la cellule d'etat indiquant d'ou vient le contenu du
# presse-papier : depot exterieur (autre application, ou Ctrl+C), copie
# venue de la zone de saisie, ou copie venue de l'apercu rendu.
ORIGINES = {
    "inconnue": "\u2014",
    "externe": "ext\u00e9rieur",
    "saisie": "saisie",
    "rendu": "rendu",
}

# Rafraichissement du compte a rebours affiche en barre d'etat.
DELAI_RAFRAICHISSEMENT_MS = 1000

# La sélection est copiée quand elle se stabilise, pas à chaque pixel du
# glisser de souris.
DELAI_STABILISATION_MS = 300


def resource_path(*parts):
    """Chemin d'une ressource, que l'on tourne depuis les sources ou un onefile.

    PyInstaller extrait les données embarquées dans un dossier temporaire exposé
    par sys._MEIPASS. Attention : « --add-data src\\mmdedit\\assets;mmdedit/assets »
    les place sous <_MEIPASS>/mmdedit/assets, pas <_MEIPASS>/assets — d'où les
    deux bases essayées. Chercher au seul premier emplacement faisait échouer
    l'icône dans l'exécutable empaqueté alors qu'elle marchait depuis les sources.
    """
    bases = [os.path.dirname(os.path.abspath(__file__))]
    meipass = getattr(sys, "_MEIPASS", None)
    if meipass:
        bases = [os.path.join(meipass, "mmdedit"), meipass] + bases

    for base in bases:
        chemin = os.path.join(base, *parts)
        if os.path.exists(chemin):
            return chemin
    return os.path.join(bases[0], *parts)


def app_icon():
    """Icône de l'application (« M » M-Media), ou icône vide si absente."""
    chemin = resource_path("assets", "mmdedit.ico")
    return QIcon(chemin) if os.path.isfile(chemin) else QIcon()


def read_text_auto(path):
    """Lit un fichier texte en tentant UTF-8 puis en repli cp1252.

    Retourne (texte, encodage_utilisé). Le repli couvre les anciens
    fichiers .txt/.csv produits sous Windows.
    """
    raw = open(path, "rb").read()
    for enc in ("utf-8", "cp1252"):
        try:
            return raw.decode(enc), enc
        except UnicodeDecodeError:
            continue
    # Dernier recours : latin-1 ne lève jamais d'erreur.
    return raw.decode("latin-1"), "latin-1"


def write_text_utf8(path, text):
    """Écrit en UTF-8 sans BOM, fins de ligne normalisées en CRLF."""
    normalized = text.replace("\r\n", "\n").replace("\r", "\n")
    normalized = normalized.replace("\n", "\r\n")
    with open(path, "wb") as fh:
        fh.write(normalized.encode("utf-8"))


class MainWindow(QMainWindow):
    def __init__(self, settings=None):
        """settings : QSettings a utiliser ; celui de l'application par defaut.

        Le parametre existe pour les tests, qui doivent ecrire dans un
        fichier temporaire plutot que dans les reglages reels de la machine.
        """
        super().__init__()
        self._path = None
        self._find_dialog = None
        self._settings = settings if settings is not None else QSettings(
            ORG_NAME, APP_NAME
        )
        self._theme = THEME_PAR_DEFAUT
        self.setWindowIcon(app_icon())

        # --- widgets centraux -----------------------------------------
        self.editor = QPlainTextEdit()
        self.editor.setLineWrapMode(QPlainTextEdit.NoWrap)
        self.preview = QTextBrowser()
        self.preview.setOpenExternalLinks(True)

        self._highlighter = MarkdownHighlighter(self.editor.document())

        self.splitter = QSplitter(Qt.Horizontal)
        self.splitter.addWidget(self.editor)
        self.splitter.addWidget(self.preview)
        self.splitter.setSizes([500, 500])
        self.setCentralWidget(self.splitter)

        # --- aperçu temps réel (léger anti-rebond) --------------------
        self._preview_timer = QTimer(self)
        self._preview_timer.setInterval(200)
        self._preview_timer.setSingleShot(True)
        self._preview_timer.timeout.connect(self._render_preview)
        self.editor.textChanged.connect(self._on_text_changed)

        self.editor.document().modificationChanged.connect(
            self._update_title
        )
        self.editor.cursorPositionChanged.connect(self._update_counts)

        # --- copie automatique de la sélection ------------------------
        # Horodatage du dernier dépôt protégé dans le presse-papier ;
        # -inf signifie « rien à protéger ».
        self._presse_papier_horodatage = float("-inf")
        # Origine du contenu courant du presse-papier, affichée en barre
        # d'état : « externe », « saisie », « rendu » ou « inconnue ».
        self._origine_presse_papier = "inconnue"
        # Dernier texte ecrit par la copie automatique, et l'instant de
        # cette ecriture : servent a reconnaitre notre propre echo.
        self._dernier_texte_interne = None
        self._instant_ecriture_interne = float("-inf")
        # Origine annoncee juste avant un Ctrl+C explicite, consommee par le
        # signal dataChanged qui suit (il arrive de maniere asynchrone).
        self._origine_attendue = None
        self._instant_origine_attendue = float("-inf")
        # Vue ayant declare la derniere sélection : « saisie » ou « rendu ».
        self._source_selection = "saisie"

        self._selection_timer = QTimer(self)
        self._selection_timer.setInterval(DELAI_STABILISATION_MS)
        self._selection_timer.setSingleShot(True)
        self._selection_timer.timeout.connect(self._copier_selection_auto)
        # Les deux vues sont des sources de sélection : l'apercu rendu est
        # sélectionnable a la souris, et c'est de la qu'on copie une phrase
        # mise en forme plutot que sa source Markdown.
        self.editor.selectionChanged.connect(
            lambda: self._on_selection_changed("saisie")
        )
        self.preview.selectionChanged.connect(
            lambda: self._on_selection_changed("rendu")
        )

        # Compte a rebours de la protection, affiche en barre d'etat.
        self._protection_timer = QTimer(self)
        self._protection_timer.setInterval(DELAI_RAFRAICHISSEMENT_MS)
        self._protection_timer.timeout.connect(self._maj_cellule_presse_papier)

        presse_papier = QApplication.clipboard()
        if presse_papier is not None:
            presse_papier.dataChanged.connect(self._on_presse_papier_change)

        self._build_actions()
        self._build_menus()
        self._build_toolbar()
        self._build_statusbar()

        self.setAcceptDrops(True)
        self.resize(1000, 640)   # taille de repli, avant tout reglage enregistre
        self._restaurer_reglages()
        self._update_title()
        self._update_counts()
        self._maj_cellule_presse_papier()
        self._render_preview()

    # ------------------------------------------------------------------
    # Construction de l'interface
    # ------------------------------------------------------------------
    def _build_actions(self):
        self.act_new = QAction("&Nouveau", self, shortcut=QKeySequence.New)
        self.act_open = QAction("&Ouvrir…", self, shortcut=QKeySequence.Open)
        self.act_save = QAction("&Enregistrer", self, shortcut=QKeySequence.Save)
        self.act_save_as = QAction(
            "Enregistrer &sous…", self, shortcut=QKeySequence.SaveAs
        )
        self.act_export_pdf = QAction("Exporter en &PDF…", self)
        self.act_quit = QAction("&Quitter", self, shortcut=QKeySequence.Quit)

        self.act_new.triggered.connect(self.new_file)
        self.act_open.triggered.connect(self.open_file)
        self.act_save.triggered.connect(self.save_file)
        self.act_save_as.triggered.connect(self.save_file_as)
        self.act_export_pdf.triggered.connect(self.export_pdf)
        self.act_quit.triggered.connect(self.close)

        self.act_undo = QAction("&Annuler", self, shortcut=QKeySequence.Undo)
        self.act_redo = QAction("&Rétablir", self, shortcut=QKeySequence.Redo)
        self.act_cut = QAction("Co&uper", self, shortcut=QKeySequence.Cut)
        self.act_copy = QAction("&Copier", self, shortcut=QKeySequence.Copy)
        self.act_paste = QAction("Co&ller", self, shortcut=QKeySequence.Paste)
        self.act_find = QAction(
            "&Rechercher / Remplacer…", self, shortcut=QKeySequence.Find
        )

        self.act_undo.triggered.connect(self.editor.undo)
        self.act_redo.triggered.connect(self.editor.redo)
        # Ces raccourcis sont portes par la fenetre (Qt.WindowShortcut) : ils
        # sont donc traites avant que la frappe n'atteigne la vue qui a le
        # focus. Cables en dur sur l'editeur, ils faisaient echouer en silence
        # tout Ctrl+C fait depuis l'apercu — editor.copy() sans sélection
        # n'ecrit rien, et le collage suivant rendait le contenu precedent.
        self.act_cut.triggered.connect(self._couper_focus)
        self.act_copy.triggered.connect(self._copier_focus)
        self.act_paste.triggered.connect(self._coller_focus)
        self.act_find.triggered.connect(self.show_find)

        self.act_copie_auto = QAction(
            "Copier la &sélection automatiquement", self, checkable=True
        )
        self.act_copie_auto.setChecked(COPIE_AUTO_PAR_DEFAUT)
        self.act_copie_auto.setToolTip(
            "Place toute sélection dans le presse-papier, sauf si celui-ci a "
            f"reçu autre chose depuis moins de {DELAI_PROTECTION_S} secondes"
        )
        self.act_copie_auto.setStatusTip(self.act_copie_auto.toolTip())
        self.act_copie_auto.toggled.connect(
            lambda coche: self._settings.setValue("edition/copie_auto", coche)
        )

        # Affichage : bascule de visibilité de chaque vue.
        self.act_show_editor = QAction("Afficher l'&éditeur", self, checkable=True)
        self.act_show_preview = QAction("Afficher l'&aperçu", self, checkable=True)
        self.act_show_editor.setChecked(True)
        self.act_show_preview.setChecked(True)
        self.act_show_editor.toggled.connect(self._toggle_editor)
        self.act_show_preview.toggled.connect(self._toggle_preview)

        # Apparence : un choix exclusif, applique a chaud et memorise.
        self._groupe_theme = QActionGroup(self)
        self._groupe_theme.setExclusive(True)
        self._actions_theme = {}
        for cle, (libelle, _qss) in THEMES.items():
            act = QAction(libelle, self, checkable=True)
            act.setStatusTip(f"Apparence {libelle.replace('&', '')}")
            act.triggered.connect(lambda _=False, c=cle: self._choisir_theme(c))
            self._groupe_theme.addAction(act)
            self._actions_theme[cle] = act

        self.act_tags = QAction("&Rappel des balises", self)
        self.act_about = QAction("À &propos", self)
        self.act_tags.triggered.connect(self.show_tags_help)
        self.act_about.triggered.connect(self.show_about)

    def _build_menus(self):
        mb = self.menuBar()
        m_file = mb.addMenu("&Fichier")
        m_file.addAction(self.act_new)
        m_file.addAction(self.act_open)
        m_file.addSeparator()
        m_file.addAction(self.act_save)
        m_file.addAction(self.act_save_as)
        m_file.addSeparator()
        m_file.addAction(self.act_export_pdf)
        m_file.addSeparator()
        m_file.addAction(self.act_quit)

        m_edit = mb.addMenu("&Édition")
        m_edit.addAction(self.act_undo)
        m_edit.addAction(self.act_redo)
        m_edit.addSeparator()
        m_edit.addAction(self.act_cut)
        m_edit.addAction(self.act_copy)
        m_edit.addAction(self.act_paste)
        m_edit.addSeparator()
        m_edit.addAction(self.act_find)
        m_edit.addSeparator()
        m_edit.addAction(self.act_copie_auto)

        m_view = mb.addMenu("&Affichage")
        m_view.addAction(self.act_show_editor)
        m_view.addAction(self.act_show_preview)
        m_view.addSeparator()
        m_apparence = m_view.addMenu("A&pparence")
        for cle in THEMES:
            m_apparence.addAction(self._actions_theme[cle])

        m_help = mb.addMenu("&Aide")
        m_help.addAction(self.act_tags)
        m_help.addAction(self.act_about)

    def _build_toolbar(self):
        tb = QToolBar("Mise en forme")
        # saveState() identifie les barres par leur objectName : sans lui, Qt
        # avertit et l'etat de la fenetre n'est pas restaure.
        tb.setObjectName("barreMiseEnForme")
        tb.setMovable(False)
        self.addToolBar(tb)

        # Encadrements — bascules : un second clic retire le marquage.
        # (libellé, info-bulle, marqueur)
        wraps = [
            ("Gras", "Gras — **texte** (Ctrl+B)", "**", "Ctrl+B"),
            ("Italique", "Italique — *texte* (Ctrl+I)", "*", "Ctrl+I"),
            ("Barré", "Barré — ~~texte~~", "~~", None),
            ("Code", "Code en ligne — `texte`", "`", None),
        ]
        for label, tip, marker, raccourci in wraps:
            act = QAction(label, self)
            act.setToolTip(tip)
            act.setStatusTip(tip)
            if raccourci:
                act.setShortcut(QKeySequence(raccourci))
            act.triggered.connect(lambda _=False, m=marker: self._toggle_wrap(m))
            tb.addAction(act)
            self.addAction(act)  # rend le raccourci actif hors focus barre d'outils
        tb.addSeparator()

        # Préfixes de ligne — bascules également : recliquer retire le préfixe,
        # et passer d'un niveau de titre à un autre remplace le précédent.
        prefixes = [
            ("Titre 1", "Titre de niveau 1 — # texte", "# "),
            ("Titre 2", "Titre de niveau 2 — ## texte", "## "),
            ("Titre 3", "Titre de niveau 3 — ### texte", "### "),
            ("Liste", "Liste à puces — - élément", "- "),
            ("Numérotée", "Liste numérotée — 1. élément", "1. "),
            ("Citation", "Citation — > texte", "> "),
        ]
        for label, tip, prefix in prefixes:
            act = QAction(label, self)
            act.setToolTip(tip)
            act.setStatusTip(tip)
            act.triggered.connect(lambda _=False, p=prefix: self._toggle_prefix(p))
            tb.addAction(act)
        tb.addSeparator()

        act_link = QAction("Lien", self)
        act_link.setToolTip("Insérer un lien — [texte](url)")
        act_link.setStatusTip("Insérer un lien — [texte](url)")
        act_link.triggered.connect(lambda: self._wrap("[", "](https://)"))
        tb.addAction(act_link)

        act_img = QAction("Image", self)
        act_img.setToolTip("Insérer une image — ![texte](chemin)")
        act_img.setStatusTip("Insérer une image — ![texte](chemin)")
        act_img.triggered.connect(lambda: self._wrap("![", "](chemin.png)"))
        tb.addAction(act_img)

    def _build_statusbar(self):
        self.sb = self.statusBar()
        self._counter = QLabel()
        self._counter.setObjectName("statusCell")
        self._format_label = QLabel("Markdown")
        self._format_label.setObjectName("statusCell")
        self._format_label.setToolTip(
            "Format détecté d'après l'extension : détermine la coloration "
            "et l'affichage de l'aperçu"
        )
        self._encoding_label = QLabel("UTF-8")
        self._encoding_label.setObjectName("statusCell")
        # Origine du contenu du presse-papier — cellule la plus a droite.
        self._presse_papier_label = QLabel()
        self._presse_papier_label.setObjectName("statusCell")
        self._presse_papier_label.setToolTip(
            "D'ou vient le contenu du presse-papier :\n"
            "  exterieur — depose par une autre application ;\n"
            "  saisie — copie depuis la zone d'edition ;\n"
            "  rendu — copie depuis l'apercu.\n"
            "Le compte a rebours est le delai pendant lequel la copie "
            "automatique de la sélection ne l'ecrasera pas."
        )
        self.sb.addPermanentWidget(self._format_label)
        self.sb.addPermanentWidget(self._encoding_label)
        self.sb.addPermanentWidget(self._counter)
        self.sb.addPermanentWidget(self._presse_papier_label)

    # ------------------------------------------------------------------
    # Aperçu et compteurs
    # ------------------------------------------------------------------
    def _on_text_changed(self):
        self._preview_timer.start()

    def _render_preview(self):
        text = self.editor.toPlainText()
        if self._is_markdown():
            self.preview.setMarkdown(text)
        else:
            self.preview.setPlainText(text)
        self._update_counts()

    def _is_markdown(self):
        return self._format() == "markdown"

    def _format(self):
        """Format du document courant : « markdown », « xml » ou « texte »."""
        if self._path is None:
            return "markdown"  # nouveau document : traité comme Markdown
        ext = os.path.splitext(self._path)[1].lower()
        if ext in MARKDOWN_EXT:
            return "markdown"
        if ext in XML_EXT:
            return "xml"
        return "texte"

    def _appliquer_format(self):
        """Adapte coloration et aperçu au format du fichier ouvert.

        Le rendu Markdown n'a de sens que pour du Markdown : sur du XML, du CSV
        ou du texte brut, l'aperçu est masqué et l'éditeur occupe la fenêtre.
        L'utilisateur peut toujours le rouvrir par le menu Affichage.
        """
        fmt = self._format()

        # Coloration : un seul highlighter attaché à la fois.
        if self._highlighter is not None:
            self._highlighter.setDocument(None)
        if fmt == "markdown":
            self._highlighter = MarkdownHighlighter(self.editor.document())
        elif fmt == "xml":
            self._highlighter = XmlHighlighter(self.editor.document())
        else:
            self._highlighter = None

        # Aperçu : affiché pour le Markdown, masqué pour le reste.
        montrer = fmt == "markdown"
        self.act_show_preview.setChecked(montrer)
        self._toggle_preview(montrer)
        if not montrer and not self.act_show_editor.isChecked():
            # L'éditeur doit rester visible s'il n'y a plus d'aperçu.
            self.act_show_editor.setChecked(True)
            self._toggle_editor(True)

        self._format_label.setText(
            {"markdown": "Markdown", "xml": "XML", "texte": "Texte"}[fmt]
        )

    def _update_counts(self):
        text = self.editor.toPlainText()
        chars = len(text)
        words = len(text.split())
        lines = text.count("\n") + 1 if text else 0
        self._counter.setText(
            f"{words} mots · {chars} caractères · {lines} lignes"
        )

    # ------------------------------------------------------------------
    # Édition assistée
    # ------------------------------------------------------------------
    def _wrap(self, prefix, suffix):
        cursor = self.editor.textCursor()
        selected = cursor.selectedText()
        debut = cursor.selectionStart()
        cursor.insertText(f"{prefix}{selected}{suffix}")
        if selected:
            # Le texte reste sélectionné, marqueurs exclus.
            self._selectionner(debut + len(prefix), len(selected))
        else:
            # Replace le curseur entre les marqueurs.
            pos = cursor.position() - len(suffix)
            cursor.setPosition(pos)
            self.editor.setTextCursor(cursor)
        self.editor.setFocus()

    # ------------------------------------------------------------------
    # Copie automatique de la sélection, et origine du presse-papier
    # ------------------------------------------------------------------
    def _vue_focus(self):
        """Vue visée par Couper / Copier / Coller : celle qui a le focus."""
        return self.preview if self.preview.hasFocus() else self.editor

    def _copier_focus(self):
        vue = self._vue_focus()
        # L'origine est annoncée avant la copie : le signal dataChanged qui
        # suit arrive de manière asynchrone sous Windows, trop tard pour que
        # l'on sache encore quelle vue a fourni le texte.
        self._annoncer_origine("rendu" if vue is self.preview else "saisie")
        vue.copy()

    def _couper_focus(self):
        if self.preview.hasFocus():
            return  # l'aperçu est en lecture seule : rien à couper
        self._annoncer_origine("saisie")
        self.editor.cut()

    def _coller_focus(self):
        if self.preview.hasFocus():
            return  # rien à coller dans l'aperçu
        self.editor.paste()

    def _annoncer_origine(self, origine):
        """Déclare l'origine du dépôt que la copie explicite va provoquer."""
        self._origine_attendue = origine
        self._instant_origine_attendue = time.monotonic()

    def _on_presse_papier_change(self):
        """Le presse-papier a changé : notre écho, ou un dépôt à protéger ?

        Un dépôt étranger — Ctrl+C ici, ou copie depuis une autre application —
        démarre le délai de protection. Les écritures de la copie automatique,
        elles, ne doivent pas se protéger elles-mêmes, sinon la première
        sélection bloquerait toutes les suivantes.

        L'identification se fait par le contenu, et non par un drapeau à usage
        unique : Windows émet souvent plusieurs dataChanged pour une seule
        écriture, et le second signal était alors pris pour un dépôt étranger.
        """
        presse_papier = QApplication.clipboard()
        texte = presse_papier.text() if presse_papier is not None else ""
        origine = self._origine_annoncee()
        # Une copie explicite (Ctrl+C) est toujours un dépôt à protéger, même
        # lorsqu'elle reprend à l'identique ce que la copie automatique venait
        # d'écrire : l'utilisateur a demandé à figer ce contenu.
        if origine is None and self._est_notre_echo(texte):
            return

        self._presse_papier_horodatage = time.monotonic()
        self._origine_presse_papier = origine or "externe"
        self._maj_cellule_presse_papier()
        self._protection_timer.start()

    def _est_notre_echo(self, texte):
        """Vrai si ce dépôt est celui que la copie automatique vient d'écrire."""
        if self._dernier_texte_interne is None:
            return False
        if texte != self._dernier_texte_interne:
            return False
        return time.monotonic() - self._instant_ecriture_interne < DELAI_ECHO_S

    def _origine_annoncee(self):
        """Origine déclarée par une copie explicite récente, ou None.

        L'annonce n'est pas consommée par le premier signal : les dataChanged
        surnuméraires de la même écriture doivent conserver cette origine, et
        non retomber sur « extérieur ». Elle expire d'elle-même.
        """
        if self._origine_attendue is None:
            return None
        if time.monotonic() - self._instant_origine_attendue >= DELAI_ECHO_S:
            self._origine_attendue = None
            return None
        return self._origine_attendue

    def _on_selection_changed(self, source):
        if not self.act_copie_auto.isChecked():
            return
        self._source_selection = source
        self._selection_timer.start()

    def _presse_papier_protege(self):
        """Vrai si le presse-papier contient un dépôt récent à ne pas écraser."""
        age = time.monotonic() - self._presse_papier_horodatage
        return age < DELAI_PROTECTION_S

    def _restant_protection(self):
        """Secondes restantes de protection, arrondies au supérieur ; 0 sinon."""
        if not self._presse_papier_protege():
            return 0
        reste = DELAI_PROTECTION_S - (
            time.monotonic() - self._presse_papier_horodatage
        )
        return max(1, int(reste + 0.999))

    def _selection_a_copier(self):
        """(texte, origine) de la sélection à copier, ou ("", None).

        La vue qui vient de déclarer sa sélection est consultée en premier ;
        l'autre sert de repli, car un re-rendu de l'aperçu émet lui aussi
        selectionChanged, avec une sélection vide.
        """
        if self._source_selection == "rendu":
            ordre = ("rendu", "saisie")
        else:
            ordre = ("saisie", "rendu")
        for origine in ordre:
            if origine == "rendu" and not self.preview.isVisible():
                continue
            vue = self.preview if origine == "rendu" else self.editor
            texte = vue.textCursor().selectedText()
            if texte:
                return texte, origine
        return "", None

    def _copier_selection_auto(self):
        if not self.act_copie_auto.isChecked():
            return

        texte, origine = self._selection_a_copier()
        if not texte:
            return

        if self._presse_papier_protege():
            self.flash_status(
                "Sélection non copiée : presse-papier occupé encore "
                f"{self._restant_protection()} s.",
                2000,
            )
            return

        presse_papier = QApplication.clipboard()
        if presse_papier is None:
            return

        # QTextCursor.selectedText() sépare les paragraphes par U+2029 et les
        # lignes par U+2028 : sans conversion, un collage ailleurs perdrait les
        # retours a la ligne. Echappements explicites : ces caracteres sont
        # invisibles dans un editeur et se perdent au moindre copier-coller.
        texte = texte.replace(PARAGRAPHE_QT, "\n").replace(LIGNE_QT, "\n")

        self._dernier_texte_interne = texte
        self._instant_ecriture_interne = time.monotonic()
        presse_papier.setText(texte)
        self._origine_presse_papier = origine
        self._maj_cellule_presse_papier()
        self.flash_status(f"Sélection copiée ({len(texte)} caractères).", 1500)

    def _maj_cellule_presse_papier(self):
        """Cellule d'état : origine du contenu, et compte à rebours s'il y a lieu."""
        # La connexion au presse-papier est etablie avant la barre d'etat :
        # un depot survenu pendant la construction ne doit pas echouer ici.
        if getattr(self, "_presse_papier_label", None) is None:
            return
        libelle = ORIGINES.get(self._origine_presse_papier, ORIGINES["inconnue"])
        restant = self._restant_protection()
        if restant:
            self._presse_papier_label.setText(
                f"Presse-papiers : {libelle} ({restant} s)"
            )
        else:
            self._presse_papier_label.setText(f"Presse-papiers : {libelle}")
            self._protection_timer.stop()

    def _selectionner(self, debut, longueur):
        """Sélectionne longueur caractères à partir de debut, dans l'éditeur."""
        cursor = self.editor.textCursor()
        cursor.setPosition(debut)
        cursor.setPosition(debut + longueur, QTextCursor.KeepAnchor)
        self.editor.setTextCursor(cursor)

    @staticmethod
    def _repetitions(texte, index, caractere, pas):
        """Compte les caractere consécutifs depuis index, dans le sens pas.

        Sert à distinguer « * » (italique) d'un « * » appartenant à un « ** »
        (gras) : sans ce comptage, appliquer l'italique sur du gras retirait un
        astérisque de chaque côté au lieu d'imbriquer les deux styles.
        """
        n = 0
        i = index
        while 0 <= i < len(texte) and texte[i] == caractere:
            n += 1
            i += pas
        return n

    def _toggle_wrap(self, marker):
        """Pose ou retire un encadrement (**, *, ~~, `) — bascule.

        Trois cas : sélection déjà encadrée (on retire), sélection encadrée par
        des marqueurs situés juste à l'extérieur (on retire aussi), sinon on
        pose. Sans sélection, on pose la paire et on place le curseur au milieu.

        Le texte reste sélectionné après l'opération, marqueurs exclus : on peut
        donc recliquer pour annuler le style sans avoir à resélectionner.
        """
        cursor = self.editor.textCursor()
        selected = cursor.selectedText()
        n = len(marker)

        if selected:
            debut, fin = cursor.selectionStart(), cursor.selectionEnd()
            texte = self.editor.toPlainText()
            car = marker[0]

            # Cas 1 : les marqueurs sont dans la sélection. On exige un nombre
            # EXACT de marqueurs, sinon « * » sur « **mot** » retirerait un
            # astérisque de chaque côté au lieu d'imbriquer l'italique.
            if (
                len(selected) >= 2 * n
                and self._repetitions(selected, 0, car, 1) == n
                and self._repetitions(selected, len(selected) - 1, car, -1) == n
            ):
                interieur = selected[n:-n]
                cursor.insertText(interieur)
                self._selectionner(debut, len(interieur))
                self.editor.setFocus()
                return

            # Cas 2 : les marqueurs encadrent la sélection, à l'extérieur —
            # même exigence de correspondance exacte.
            if (
                debut >= n
                and texte[debut - n : debut] == marker
                and texte[fin : fin + n] == marker
                and self._repetitions(texte, debut - 1, car, -1) == n
                and self._repetitions(texte, fin, car, 1) == n
            ):
                cursor.beginEditBlock()
                cursor.setPosition(debut - n)
                cursor.setPosition(fin + n, QTextCursor.KeepAnchor)
                cursor.insertText(selected)
                cursor.endEditBlock()
                self._selectionner(debut - n, len(selected))
                self.editor.setFocus()
                return

            # Cas 3 : pose. On reselectionne le texte entre les marqueurs.
            cursor.insertText(f"{marker}{selected}{marker}")
            self._selectionner(debut + n, len(selected))
            self.editor.setFocus()
            return

        self._wrap(marker, marker)

    def _toggle_prefix(self, prefix):
        """Pose, remplace ou retire un préfixe de ligne — bascule.

        Recliquer sur le même préfixe le retire ; passer d'un niveau de titre à
        un autre (ou d'une liste à une citation) remplace le préfixe existant.
        """
        # Préfixes reconnus, du plus long au plus court pour que « ### » soit
        # testé avant « ## » et « # ».
        connus = ("### ", "## ", "# ", "- ", "1. ", "> ")

        cursor = self.editor.textCursor()
        cursor.beginEditBlock()
        # Les énumérations doivent être qualifiées : PySide6 6.11 ne les expose
        # plus sur l'instance (cursor.StartOfLine lève AttributeError).
        cursor.movePosition(QTextCursor.StartOfLine)
        cursor.movePosition(QTextCursor.EndOfLine, QTextCursor.KeepAnchor)
        ligne = cursor.selectedText()

        actuel = next((p for p in sorted(connus, key=len, reverse=True)
                       if ligne.startswith(p)), None)

        if actuel == prefix:
            nouvelle = ligne[len(actuel):]          # retrait
        elif actuel is not None:
            nouvelle = prefix + ligne[len(actuel):]  # remplacement
        else:
            nouvelle = prefix + ligne               # pose

        cursor.insertText(nouvelle)
        cursor.endEditBlock()
        self.editor.setFocus()

    # ------------------------------------------------------------------
    # Bascule des vues (au moins une reste visible)
    # ------------------------------------------------------------------
    def _toggle_editor(self, visible):
        if not visible and not self.act_show_preview.isChecked():
            self.act_show_editor.setChecked(True)
            return
        self.editor.setVisible(visible)

    def _toggle_preview(self, visible):
        if not visible and not self.act_show_editor.isChecked():
            self.act_show_preview.setChecked(True)
            return
        self.preview.setVisible(visible)

    # ------------------------------------------------------------------
    # Gestion des fichiers
    # ------------------------------------------------------------------
    def new_file(self):
        if not self._maybe_save():
            return
        self.editor.clear()
        self._path = None
        self._appliquer_format()  # nouveau document = Markdown, aperçu rétabli
        self.editor.document().setModified(False)
        self._encoding_label.setText("UTF-8")
        self._update_title()
        self._render_preview()

    def open_file(self, path=None):
        if not self._maybe_save():
            return
        if not path:
            path, _ = QFileDialog.getOpenFileName(
                self, "Ouvrir un fichier", "", FILE_FILTERS
            )
        if not path:
            return
        try:
            text, enc = read_text_auto(path)
        except OSError as exc:
            QMessageBox.critical(self, APP_NAME, f"Lecture impossible :\n{exc}")
            return
        self._path = path
        # Le format doit être appliqué AVANT de poser le texte : le highlighter
        # à attacher dépend de l'extension, et on évite ainsi une colorisation
        # Markdown fugace sur un fichier XML.
        self._appliquer_format()
        self.editor.setPlainText(text)
        self.editor.document().setModified(False)
        self._encoding_label.setText(enc.upper())
        self._update_title()
        self._render_preview()

    def save_file(self):
        if self._path is None:
            return self.save_file_as()
        return self._write_to(self._path)

    def save_file_as(self):
        path, _ = QFileDialog.getSaveFileName(
            self, "Enregistrer sous", self._path or "", FILE_FILTERS
        )
        if not path:
            return False
        return self._write_to(path)

    def _write_to(self, path):
        try:
            write_text_utf8(path, self.editor.toPlainText())
        except OSError as exc:
            QMessageBox.critical(self, APP_NAME, f"Écriture impossible :\n{exc}")
            return False
        ancien_format = self._format()
        self._path = path
        # « Enregistrer sous » peut changer l'extension, donc le format.
        if self._format() != ancien_format:
            self._appliquer_format()
        self.editor.document().setModified(False)
        self._encoding_label.setText("UTF-8")
        self._update_title()
        self._render_preview()
        self.flash_status("Enregistré.")
        return True

    def export_pdf(self):
        default = ""
        if self._path:
            default = os.path.splitext(self._path)[0] + ".pdf"
        path, _ = QFileDialog.getSaveFileName(
            self, "Exporter en PDF", default, "PDF (*.pdf)"
        )
        if not path:
            return
        doc = QTextDocument()
        if self._is_markdown():
            doc.setMarkdown(self.editor.toPlainText())
        else:
            doc.setPlainText(self.editor.toPlainText())
        printer = QPrinter(QPrinter.HighResolution)
        printer.setOutputFormat(QPrinter.PdfFormat)
        printer.setOutputFileName(path)
        doc.print_(printer)
        self.flash_status("PDF exporté.")

    def _maybe_save(self):
        """Retourne True si l'on peut continuer, False si annulation."""
        if not self.editor.document().isModified():
            return True
        name = os.path.basename(self._path) if self._path else "Sans titre"
        resp = QMessageBox.question(
            self,
            APP_NAME,
            f"Le document « {name} » a été modifié.\n"
            "Voulez-vous enregistrer les modifications ?",
            QMessageBox.Save | QMessageBox.Discard | QMessageBox.Cancel,
            QMessageBox.Save,
        )
        if resp == QMessageBox.Save:
            return self.save_file()
        if resp == QMessageBox.Cancel:
            return False
        return True  # Discard

    # ------------------------------------------------------------------
    # Recherche / Remplacement
    # ------------------------------------------------------------------
    def show_find(self):
        if self._find_dialog is None:
            self._find_dialog = FindReplaceDialog(self.editor, self)
        self._find_dialog.show()
        self._find_dialog.raise_()
        self._find_dialog.focus_find()

    # ------------------------------------------------------------------
    # Aide
    # ------------------------------------------------------------------
    def show_tags_help(self):
        box = QMessageBox(self)
        box.setWindowTitle("Rappel des balises")
        box.setTextFormat(Qt.MarkdownText)
        box.setText(TAGS_HELP)
        box.exec()

    def show_about(self):
        boite = QMessageBox(self)
        boite.setWindowTitle("À propos")
        boite.setIconPixmap(app_icon().pixmap(64, 64))
        boite.setTextFormat(Qt.RichText)
        boite.setText(
            f"<b>{APP_NAME}</b> {__version__}<br><br>"
            "Lecteur / éditeur Markdown — outil M-Media.<br>"
            "PySide6, rendu Markdown et export PDF natifs Qt."
        )
        boite.setInformativeText(
            "<p><b>Licence</b> — Ce programme est un <b>logiciel libre</b>, "
            "distribué selon les termes de la <b>Licence Publique Générale GNU, "
            "version 3</b> ou ultérieure.</p>"
            "<p>Il est fourni <b>sans aucune garantie</b>, dans la mesure permise "
            "par la loi. Vous êtes libre de l'utiliser, de l'étudier, de le "
            "modifier et de le redistribuer, à condition d'accorder les mêmes "
            "libertés à ceux à qui vous le transmettez, code source inclus.</p>"
            "<p>Texte complet : bouton « Afficher les détails », fichier "
            "<code>LICENSE</code>, ou "
            '<a href="https://www.gnu.org/licenses/gpl-3.0.html">'
            "gnu.org/licenses/gpl-3.0</a>. Traduction française non officielle : "
            '<a href="https://www.gnu.org/licenses/quick-guide-gplv3.fr.html">'
            "guide rapide de la GPLv3</a>.</p>"
            "<p>Qt et PySide6 sont distribués sous licence LGPL v3 par le "
            "Qt Project.</p>"
        )
        boite.setDetailedText(self._texte_licence())
        boite.exec()

    @staticmethod
    def _texte_licence():
        """Texte intégral de la licence, ou message de repli s'il manque."""
        chemin = resource_path("LICENSE")
        if not os.path.isfile(chemin):
            # Hors empaquetage, le fichier est à la racine du projet.
            chemin = os.path.join(
                os.path.dirname(os.path.dirname(os.path.dirname(
                    os.path.abspath(__file__)))),
                "LICENSE",
            )
        try:
            with open(chemin, encoding="utf-8") as fh:
                return fh.read()
        except OSError:
            return (
                "Le fichier LICENSE est introuvable.\n\n"
                "Texte complet de la GNU GPL version 3 :\n"
                "https://www.gnu.org/licenses/gpl-3.0.txt"
            )

    # ------------------------------------------------------------------
    # Divers
    # ------------------------------------------------------------------
    def flash_status(self, message, msec=3000):
        self.sb.showMessage(message, msec)

    def _update_title(self, *_):
        name = os.path.basename(self._path) if self._path else "Sans titre"
        star = "*" if self.editor.document().isModified() else ""
        self.setWindowTitle(f"{star}{name} — {APP_NAME}")

    # ------------------------------------------------------------------
    # Réglages persistants (géométrie, apparence, options)
    # ------------------------------------------------------------------
    def _restaurer_reglages(self):
        """Rétablit la fenêtre telle qu'elle était à la dernière fermeture.

        La visibilité des vues n'est volontairement pas mémorisée : elle est
        déduite du format du fichier ouvert (cf. _appliquer_format), et un
        réglage enregistré la contredirait à chaque ouverture.
        """
        reglages = self._settings

        geometrie = reglages.value("fenetre/geometrie")
        if geometrie is not None:
            self.restoreGeometry(geometrie)
        etat = reglages.value("fenetre/etat")
        if etat is not None:
            self.restoreState(etat)
        partage = reglages.value("fenetre/splitter")
        if partage is not None:
            self.splitter.restoreState(partage)

        self.act_copie_auto.setChecked(
            reglages.value("edition/copie_auto", COPIE_AUTO_PAR_DEFAUT, type=bool)
        )
        self._appliquer_theme(
            reglages.value("apparence/theme", THEME_PAR_DEFAUT, type=str)
        )

    def _enregistrer_reglages(self):
        """Enregistre l'état de la fenêtre — appelé à la fermeture."""
        reglages = self._settings
        reglages.setValue("fenetre/geometrie", self.saveGeometry())
        reglages.setValue("fenetre/etat", self.saveState())
        reglages.setValue("fenetre/splitter", self.splitter.saveState())
        reglages.setValue("edition/copie_auto", self.act_copie_auto.isChecked())
        reglages.setValue("apparence/theme", self._theme)
        reglages.sync()

    def _appliquer_theme(self, nom):
        """Applique une apparence à chaud, sans redémarrage.

        La feuille de style est posée sur l'application entière : elle couvre
        donc aussi les boîtes de dialogue. Un thème vide (« systeme ») rend la
        main au style natif de la plateforme — c'est le défaut sur macOS.
        """
        if nom not in THEMES:
            nom = THEME_PAR_DEFAUT
        self._theme = nom
        app = QApplication.instance()
        if app is not None:
            app.setStyleSheet(THEMES[nom][1] or "")
        action = self._actions_theme.get(nom)
        if action is not None:
            action.setChecked(True)

    def _choisir_theme(self, nom):
        """Choix explicite de l'utilisateur : appliqué et mémorisé aussitôt."""
        self._appliquer_theme(nom)
        self._settings.setValue("apparence/theme", self._theme)
        self.flash_status(f"Apparence : {THEMES[self._theme][0].replace('&', '')}.")

    # Glisser-déposer d'un fichier sur la fenêtre.
    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()

    def dropEvent(self, event):
        urls = event.mimeData().urls()
        if urls:
            self.open_file(urls[0].toLocalFile())

    def closeEvent(self, event):
        if self._maybe_save():
            self._enregistrer_reglages()
            event.accept()
        else:
            event.ignore()


def charger_traductions_qt(app):
    """Traduit les libellés fournis par Qt lui-même (boutons standard, etc.).

    L'interface de MMdedit est écrite en français, mais les widgets standard de
    Qt — « OK », « Show Details… », les boutons des boîtes de dialogue, le menu
    contextuel de l'éditeur — restent en anglais tant que les catalogues de
    traduction ne sont pas chargés. Ils sont livrés avec PySide6.

    La langue suit celle du système ; à défaut de catalogue, on retombe sur
    l'anglais sans erreur. Les traducteurs doivent rester référencés, sinon le
    ramasse-miettes les emporte et la traduction disparaît.
    """
    global _traducteurs
    _traducteurs = []

    dossiers = [
        QLibraryInfo.path(QLibraryInfo.TranslationsPath),
        resource_path("translations"),
    ]
    langue = QLocale.system().name()  # ex. « fr_FR »

    for catalogue in ("qtbase", "qt"):
        for dossier in dossiers:
            if not dossier or not os.path.isdir(dossier):
                continue
            traducteur = QTranslator()
            if traducteur.load(f"{catalogue}_{langue}", dossier) or traducteur.load(
                f"{catalogue}_{langue.split('_')[0]}", dossier
            ):
                app.installTranslator(traducteur)
                _traducteurs.append(traducteur)
                break


def main():
    app = QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    app.setOrganizationName(ORG_NAME)  # emplacement des reglages QSettings
    app.setWindowIcon(app_icon())
    if sys.platform == "win32":
        # Sans AppUserModelID propre, Windows regroupe la fenêtre sous l'icône
        # de python.exe dans la barre des tâches au lieu de la nôtre.
        try:
            import ctypes

            ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(
                f"MMedia.{APP_NAME}.{__version__}"
            )
        except Exception:  # noqa: BLE001 — purement cosmétique, jamais bloquant
            pass
    charger_traductions_qt(app)
    # L'apparence n'est plus imposee ici : MainWindow applique celle que
    # l'utilisateur a choisie (menu Affichage > Apparence), enregistree
    # d'une session a l'autre.
    win = MainWindow()
    # Ouverture par argument : MMdedit fichier.md
    if len(sys.argv) > 1 and os.path.isfile(sys.argv[1]):
        win.open_file(sys.argv[1])
    win.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
