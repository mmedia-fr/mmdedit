// SPDX-License-Identifier: GPL-3.0-or-later
//
// Colorations syntaxiques de la zone d'édition (Markdown, XML), portées de
// src/mmdedit/highlighter.py sans changer les couleurs ni les règles.
//
// QSyntaxHighlighter n'existe pas côté QML : il s'attache au QTextDocument que
// le TextArea expose par `textDocument`, d'où le passage par du C++.
#pragma once

#include <QtCore/QRegularExpression>
#include <QtGui/QColor>
#include <QtGui/QFont>
#include <QtGui/QSyntaxHighlighter>
#include <QtGui/QTextCharFormat>
#include <QtGui/QTextDocument>

#include <utility>
#include <vector>

/// Fabrique un format de caractère — équivalent du `_fmt()` de la version Python.
inline QTextCharFormat mmdeditFormat(const QString& couleur = QString(), bool gras = false,
                                     bool italique = false, bool mono = false,
                                     bool barre = false)
{
  QTextCharFormat f;
  if (!couleur.isEmpty())
    f.setForeground(QColor(couleur));
  if (gras)
    f.setFontWeight(QFont::Bold);
  if (italique)
    f.setFontItalic(true);
  if (mono)
    f.setFontFamilies({ QStringLiteral("Courier New"), QStringLiteral("monospace") });
  if (barre)
    f.setFontStrikeOut(true);
  return f;
}

/// Règles ligne à ligne, appliquées dans l'ordre : les suivantes recouvrent.
using ReglesColoration = std::vector<std::pair<QRegularExpression, QTextCharFormat>>;

/// Colore les principales constructions Markdown.
///
/// Volontairement simple : expressions régulières ligne à ligne. Ne prétend pas
/// couvrir tous les cas limites de CommonMark, mais les balises du rappel.
class ColorateurMarkdown : public QSyntaxHighlighter
{
  Q_OBJECT
public:
  explicit ColorateurMarkdown(QTextDocument* document)
    : QSyntaxHighlighter(document)
  {
    const QTextCharFormat titre = mmdeditFormat(QStringLiteral("#000080"), true);
    const QTextCharFormat gras = mmdeditFormat(QString(), true);
    const QTextCharFormat italique = mmdeditFormat(QString(), false, true);
    const QTextCharFormat barre = mmdeditFormat(QString(), false, false, false, true);
    const QTextCharFormat code = mmdeditFormat(QStringLiteral("#8b0000"), false, false, true);
    const QTextCharFormat lien = mmdeditFormat(QStringLiteral("#0000cd"));
    const QTextCharFormat citation = mmdeditFormat(QStringLiteral("#556b2f"), false, true);
    const QTextCharFormat puce = mmdeditFormat(QStringLiteral("#800080"), true);
    const QTextCharFormat filet = mmdeditFormat(QStringLiteral("#808080"), true);

    m_regles = {
      { QRegularExpression(QStringLiteral("^#{1,6}\\s.*$")), titre },
      { QRegularExpression(QStringLiteral("^\\s*>.*$")), citation },
      { QRegularExpression(QStringLiteral("^\\s*([-+*]|\\d+\\.)\\s")), puce },
      { QRegularExpression(QStringLiteral("^\\s*([-*_])(\\s*\\1){2,}\\s*$")), filet },
      { QRegularExpression(QStringLiteral("(\\*\\*|__)(?=\\S)(.+?\\S)\\1")), gras },
      { QRegularExpression(QStringLiteral("(?<![\\*_])([*_])(?=\\S)(.+?\\S)\\1(?![\\*_])")),
        italique },
      { QRegularExpression(QStringLiteral("~~(?=\\S)(.+?\\S)~~")), barre },
      { QRegularExpression(QStringLiteral("`[^`]+`")), code },
      { QRegularExpression(QStringLiteral("!?\\[[^\\]]*\\]\\([^)]*\\)")), lien },
    };
  }

protected:
  void highlightBlock(const QString& texte) override
  {
    for (const auto& [expression, format] : m_regles) {
      auto it = expression.globalMatch(texte);
      while (it.hasNext()) {
        const QRegularExpressionMatch m = it.next();
        setFormat(m.capturedStart(), m.capturedLength(), format);
      }
    }
  }

private:
  ReglesColoration m_regles;
};

/// Colore le XML et le HTML : balises, attributs, valeurs, commentaires.
///
/// Les commentaires et les sections CDATA courent sur plusieurs lignes : ils
/// sont suivis par état de bloc, le reste par expressions régulières.
class ColorateurXml : public QSyntaxHighlighter
{
  Q_OBJECT
public:
  enum Etat { EtatNormal = 0, EtatCommentaire = 1, EtatCdata = 2 };

  explicit ColorateurXml(QTextDocument* document)
    : QSyntaxHighlighter(document)
    , m_balise(mmdeditFormat(QStringLiteral("#000080"), true))
    , m_attribut(mmdeditFormat(QStringLiteral("#800080")))
    , m_valeur(mmdeditFormat(QStringLiteral("#8b0000")))
    , m_commentaire(mmdeditFormat(QStringLiteral("#008000"), false, true))
    , m_entite(mmdeditFormat(QStringLiteral("#ff8c00")))
    , m_declaration(mmdeditFormat(QStringLiteral("#808080"), true))
    , m_debutCommentaire(QStringLiteral("<!--"))
    , m_finCommentaire(QStringLiteral("-->"))
    , m_debutCdata(QStringLiteral("<!\\[CDATA\\["))
    , m_finCdata(QStringLiteral("\\]\\]>"))
  {
    m_regles = {
      { QRegularExpression(QStringLiteral("<\\?[^?]*\\?>|<!DOCTYPE[^>]*>")), m_declaration },
      { QRegularExpression(QStringLiteral("</?\\s*([A-Za-z_][\\w.:-]*)")), m_balise },
      { QRegularExpression(QStringLiteral("/?>")), m_balise },
      { QRegularExpression(QStringLiteral("([A-Za-z_][\\w.:-]*)\\s*(?=\\=)")), m_attribut },
      { QRegularExpression(QStringLiteral("\"[^\"]*\"|'[^']*'")), m_valeur },
      { QRegularExpression(QStringLiteral("&[#\\w]+;")), m_entite },
    };
  }

protected:
  void highlightBlock(const QString& texte) override
  {
    setCurrentBlockState(EtatNormal);

    if (blocMultiligne(texte, EtatCommentaire, m_debutCommentaire, m_finCommentaire,
                       m_commentaire)) {
      setCurrentBlockState(EtatCommentaire);
      return;
    }
    if (blocMultiligne(texte, EtatCdata, m_debutCdata, m_finCdata, m_valeur)) {
      setCurrentBlockState(EtatCdata);
      return;
    }
    // Si la ligne ouvre un commentaire ou un CDATA, ce qui précède a déjà été
    // colorié par blocMultiligne().
    if (previousBlockState() != EtatCommentaire && previousBlockState() != EtatCdata)
      appliquerRegles(texte, 0);
  }

private:
  void appliquerRegles(const QString& texte, int decalage)
  {
    for (const auto& [expression, format] : m_regles) {
      auto it = expression.globalMatch(texte);
      while (it.hasNext()) {
        const QRegularExpressionMatch m = it.next();
        setFormat(decalage + m.capturedStart(), m.capturedLength(), format);
      }
    }
  }

  /// Rend vrai si la fin de la ligne est encore dans la construction ouverte.
  bool blocMultiligne(const QString& texte, int etat, const QRegularExpression& debutExpr,
                      const QRegularExpression& finExpr, const QTextCharFormat& format)
  {
    int depart = 0;
    if (previousBlockState() != etat) {
      const QRegularExpressionMatch m = debutExpr.match(texte);
      if (!m.hasMatch())
        return false;
      depart = m.capturedStart();
    }

    const QRegularExpressionMatch fin = finExpr.match(texte, depart);
    if (fin.hasMatch()) {
      setFormat(depart, fin.capturedEnd() - depart, format);
      const QString reste = texte.mid(fin.capturedEnd());
      if (!reste.isEmpty())
        appliquerRegles(reste, fin.capturedEnd());
      return false;
    }

    setFormat(depart, texte.length() - depart, format);
    return true;
  }

  QTextCharFormat m_balise;
  QTextCharFormat m_attribut;
  QTextCharFormat m_valeur;
  QTextCharFormat m_commentaire;
  QTextCharFormat m_entite;
  QTextCharFormat m_declaration;
  QRegularExpression m_debutCommentaire;
  QRegularExpression m_finCommentaire;
  QRegularExpression m_debutCdata;
  QRegularExpression m_finCdata;
  ReglesColoration m_regles;
};
