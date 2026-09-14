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

"""Dialogue non modal Rechercher / Remplacer."""

from PySide6.QtCore import Qt
from PySide6.QtGui import QTextCursor, QTextDocument
from PySide6.QtWidgets import (
    QCheckBox,
    QDialog,
    QGridLayout,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPushButton,
    QVBoxLayout,
)


class FindReplaceDialog(QDialog):
    """Recherche et remplacement dans un QPlainTextEdit.

    Non modal : reste ouvert pendant l'édition. Agit sur l'éditeur passé
    au constructeur.
    """

    def __init__(self, editor, parent=None):
        super().__init__(parent)
        self._editor = editor
        self.setWindowTitle("Rechercher / Remplacer")
        self.setModal(False)

        self._find_edit = QLineEdit()
        self._replace_edit = QLineEdit()
        self._case_box = QCheckBox("Respecter la casse")
        self._word_box = QCheckBox("Mot entier")

        grid = QGridLayout()
        grid.addWidget(QLabel("Rechercher :"), 0, 0)
        grid.addWidget(self._find_edit, 0, 1)
        grid.addWidget(QLabel("Remplacer par :"), 1, 0)
        grid.addWidget(self._replace_edit, 1, 1)

        opts = QHBoxLayout()
        opts.addWidget(self._case_box)
        opts.addWidget(self._word_box)
        opts.addStretch(1)

        btn_next = QPushButton("Suivant")
        btn_prev = QPushButton("Précédent")
        btn_replace = QPushButton("Remplacer")
        btn_replace_all = QPushButton("Remplacer tout")
        btn_close = QPushButton("Fermer")

        btn_next.clicked.connect(lambda: self._find(forward=True))
        btn_prev.clicked.connect(lambda: self._find(forward=False))
        btn_replace.clicked.connect(self._replace_one)
        btn_replace_all.clicked.connect(self._replace_all)
        btn_close.clicked.connect(self.close)

        buttons = QHBoxLayout()
        for b in (btn_next, btn_prev, btn_replace, btn_replace_all, btn_close):
            buttons.addWidget(b)

        layout = QVBoxLayout(self)
        layout.addLayout(grid)
        layout.addLayout(opts)
        layout.addLayout(buttons)

        self._find_edit.returnPressed.connect(lambda: self._find(forward=True))
        btn_next.setDefault(True)

    # -- helpers ---------------------------------------------------------

    def _flags(self, forward=True):
        flags = QTextDocument.FindFlags()
        if not forward:
            flags |= QTextDocument.FindBackward
        if self._case_box.isChecked():
            flags |= QTextDocument.FindCaseSensitively
        if self._word_box.isChecked():
            flags |= QTextDocument.FindWholeWords
        return flags

    def _find(self, forward=True):
        needle = self._find_edit.text()
        if not needle:
            return False
        found = self._editor.find(needle, self._flags(forward))
        if not found:
            # Reboucler depuis le début (ou la fin) du document.
            cursor = self._editor.textCursor()
            cursor.movePosition(
                QTextCursor.Start if forward else QTextCursor.End
            )
            self._editor.setTextCursor(cursor)
            found = self._editor.find(needle, self._flags(forward))
        return found

    def _replace_one(self):
        cursor = self._editor.textCursor()
        if cursor.hasSelection() and (
            cursor.selectedText() == self._find_edit.text()
            or (
                not self._case_box.isChecked()
                and cursor.selectedText().lower()
                == self._find_edit.text().lower()
            )
        ):
            cursor.insertText(self._replace_edit.text())
        self._find(forward=True)

    def _replace_all(self):
        needle = self._find_edit.text()
        if not needle:
            return
        editor = self._editor
        doc = editor.document()
        count = 0
        cursor = QTextCursor(doc)
        cursor.beginEditBlock()
        find_cursor = QTextCursor(doc)
        find_cursor.movePosition(QTextCursor.Start)
        while True:
            find_cursor = doc.find(needle, find_cursor, self._flags(True))
            if find_cursor.isNull():
                break
            find_cursor.insertText(self._replace_edit.text())
            count += 1
        cursor.endEditBlock()
        if self.parent() is not None and hasattr(self.parent(), "flash_status"):
            self.parent().flash_status(f"{count} remplacement(s) effectué(s).")

    def focus_find(self):
        self._find_edit.setFocus()
        self._find_edit.selectAll()
