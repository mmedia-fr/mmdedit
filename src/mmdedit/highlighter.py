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

"""Colorations syntaxiques de la zone d'édition (Markdown, XML).

Un highlighter se détache de son document par ``setDocument(None)`` — c'est
ainsi que la fenêtre principale bascule de coloration quand on ouvre un
fichier d'un autre format.
"""

from PySide6.QtCore import QRegularExpression
from PySide6.QtGui import QColor, QFont, QSyntaxHighlighter, QTextCharFormat


def _fmt(color=None, bold=False, italic=False, mono=False, strike=False):
    f = QTextCharFormat()
    if color is not None:
        f.setForeground(QColor(color))
    if bold:
        f.setFontWeight(QFont.Bold)
    if italic:
        f.setFontItalic(True)
    if mono:
        f.setFontFamily("Courier New")
    if strike:
        f.setFontStrikeOut(True)
    return f


class MarkdownHighlighter(QSyntaxHighlighter):
    """Colore les principales constructions Markdown.

    Volontairement simple : basé sur des expressions régulières ligne à
    ligne. Ne prétend pas couvrir tous les cas limites de CommonMark, mais
    les balises courantes listées dans le rappel.
    """

    def __init__(self, document):
        super().__init__(document)

        heading = _fmt(color="#000080", bold=True)
        bold = _fmt(bold=True)
        italic = _fmt(italic=True)
        strike = _fmt(strike=True)
        code = _fmt(color="#8b0000", mono=True)
        link = _fmt(color="#0000cd")
        quote = _fmt(color="#556b2f", italic=True)
        listmark = _fmt(color="#800080", bold=True)
        rule = _fmt(color="#808080", bold=True)

        # Ordre important : les règles suivantes s'appliquent successivement.
        self._rules = [
            # Titres  # ... jusqu'à ######
            (QRegularExpression(r"^#{1,6}\s.*$"), heading),
            # Citation  > ...
            (QRegularExpression(r"^\s*>.*$"), quote),
            # Marqueur de liste en début de ligne (- + * ou 1.)
            (QRegularExpression(r"^\s*([-+*]|\d+\.)\s"), listmark),
            # Séparateur horizontal --- *** ___
            (QRegularExpression(r"^\s*([-*_])(\s*\1){2,}\s*$"), rule),
            # Gras  **texte**  ou __texte__
            (QRegularExpression(r"(\*\*|__)(?=\S)(.+?\S)\1"), bold),
            # Italique  *texte*  ou _texte_
            (QRegularExpression(r"(?<![\*_])([*_])(?=\S)(.+?\S)\1(?![\*_])"), italic),
            # Barré  ~~texte~~
            (QRegularExpression(r"~~(?=\S)(.+?\S)~~"), strike),
            # Code en ligne  `texte`
            (QRegularExpression(r"`[^`]+`"), code),
            # Lien / image  [texte](url)
            (QRegularExpression(r"!?\[[^\]]*\]\([^)]*\)"), link),
        ]

    def highlightBlock(self, text):
        for expr, fmt in self._rules:
            it = expr.globalMatch(text)
            while it.hasNext():
                m = it.next()
                self.setFormat(m.capturedStart(), m.capturedLength(), fmt)


class XmlHighlighter(QSyntaxHighlighter):
    """Colore le XML et le HTML : balises, attributs, valeurs, commentaires.

    Les commentaires <!-- --> et les sections CDATA peuvent s'étendre sur
    plusieurs lignes : ils sont traités par état de bloc, le reste par
    expressions régulières ligne à ligne.
    """

    # États de bloc pour les constructions multi-lignes.
    ETAT_NORMAL = 0
    ETAT_COMMENTAIRE = 1
    ETAT_CDATA = 2

    def __init__(self, document):
        super().__init__(document)

        self._f_balise = _fmt(color="#000080", bold=True)
        self._f_attribut = _fmt(color="#800080")
        self._f_valeur = _fmt(color="#8b0000")
        self._f_commentaire = _fmt(color="#008000", italic=True)
        self._f_entite = _fmt(color="#ff8c00")
        self._f_declaration = _fmt(color="#808080", bold=True)

        self._rules = [
            # Déclaration <?xml ... ?> et doctype <!DOCTYPE ...>
            (QRegularExpression(r"<\?[^?]*\?>|<!DOCTYPE[^>]*>"), self._f_declaration),
            # Nom de balise, ouvrante ou fermante
            (QRegularExpression(r"</?\s*([A-Za-z_][\w.:-]*)"), self._f_balise),
            (QRegularExpression(r"/?>"), self._f_balise),
            # Attribut =
            (QRegularExpression(r"([A-Za-z_][\w.:-]*)\s*(?=\=)"), self._f_attribut),
            # Valeur entre guillemets simples ou doubles
            (QRegularExpression(r"\"[^\"]*\"|'[^']*'"), self._f_valeur),
            # Entités &amp; &#233;
            (QRegularExpression(r"&[#\w]+;"), self._f_entite),
        ]

        self._debut_commentaire = QRegularExpression(r"<!--")
        self._fin_commentaire = QRegularExpression(r"-->")
        self._debut_cdata = QRegularExpression(r"<!\[CDATA\[")
        self._fin_cdata = QRegularExpression(r"\]\]>")

    def _bloc_multiligne(self, text, etat, debut_expr, fin_expr, fmt):
        """Colore une construction pouvant courir sur plusieurs lignes.

        Retourne True si la fin de la ligne est encore dans la construction.
        """
        if self.previousBlockState() == etat:
            depart = 0
        else:
            m = debut_expr.match(text)
            if not m.hasMatch():
                return False
            depart = m.capturedStart()

        m_fin = fin_expr.match(text, depart)
        if m_fin.hasMatch():
            longueur = m_fin.capturedEnd() - depart
            self.setFormat(depart, longueur, fmt)
            # Le reste de la ligne après la fermeture est du XML ordinaire.
            reste = text[m_fin.capturedEnd():]
            if reste:
                self._appliquer_regles(reste, decalage=m_fin.capturedEnd())
            return False

        self.setFormat(depart, len(text) - depart, fmt)
        return True

    def _appliquer_regles(self, text, decalage=0):
        for expr, fmt in self._rules:
            it = expr.globalMatch(text)
            while it.hasNext():
                m = it.next()
                self.setFormat(decalage + m.capturedStart(), m.capturedLength(), fmt)

    def highlightBlock(self, text):
        self.setCurrentBlockState(self.ETAT_NORMAL)

        if self._bloc_multiligne(
            text, self.ETAT_COMMENTAIRE,
            self._debut_commentaire, self._fin_commentaire, self._f_commentaire,
        ):
            self.setCurrentBlockState(self.ETAT_COMMENTAIRE)
            return

        if self._bloc_multiligne(
            text, self.ETAT_CDATA,
            self._debut_cdata, self._fin_cdata, self._f_valeur,
        ):
            self.setCurrentBlockState(self.ETAT_CDATA)
            return

        # Ligne ordinaire : si elle ouvre un commentaire ou un CDATA, la partie
        # qui précède a déjà été coloriée par _bloc_multiligne.
        if self.previousBlockState() not in (self.ETAT_COMMENTAIRE, self.ETAT_CDATA):
            self._appliquer_regles(text)
