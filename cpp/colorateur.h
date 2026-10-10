// SPDX-License-Identifier: GPL-3.0-or-later
//
// Colorations syntaxiques de la zone d'édition. Markdown et XML sont portés de
// src/mmdedit/highlighter.py sans changer les couleurs ni les règles ; les
// configurations, le code et les journaux s'y sont ajoutés, sur la même palette.
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

/// Base des colorations faites de règles ligne à ligne.
///
/// Même principe que ColorateurMarkdown : les règles s'appliquent dans l'ordre,
/// les suivantes recouvrant les précédentes. Les colorations qui suivent n'en
/// diffèrent que par leurs règles, d'où cette base commune.
class ColorateurParRegles : public QSyntaxHighlighter
{
  Q_OBJECT
public:
  explicit ColorateurParRegles(QTextDocument* document)
    : QSyntaxHighlighter(document)
  {
  }

protected:
  void highlightBlock(const QString& texte) override { appliquerRegles(texte, 0); }

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

  ReglesColoration m_regles;
};

/// Expression régulière insensible à la casse — les mots-clés d'un script batch
/// ou d'une requête SQL s'écrivent aussi bien en majuscules qu'en minuscules.
inline QRegularExpression mmdeditExprCasseIndifferente(const QString& motif)
{
  return QRegularExpression(motif, QRegularExpression::CaseInsensitiveOption);
}

/// Colore les fichiers de configuration et de données : JSON, YAML, TOML, INI.
///
/// Un seul jeu de règles pour les quatre : leurs constructions communes — clé,
/// chaîne, nombre, littéral, commentaire, section — suffisent à les lire, et
/// une coloration par format multiplierait le code sans rien apprendre de plus.
class ColorateurDonnees : public ColorateurParRegles
{
public:
  explicit ColorateurDonnees(QTextDocument* document)
    : ColorateurParRegles(document)
  {
    const QTextCharFormat section = mmdeditFormat(QStringLiteral("#000080"), true);
    const QTextCharFormat clef = mmdeditFormat(QStringLiteral("#800080"));
    const QTextCharFormat chaine = mmdeditFormat(QStringLiteral("#8b0000"));
    const QTextCharFormat nombre = mmdeditFormat(QStringLiteral("#ff8c00"));
    const QTextCharFormat litteral = mmdeditFormat(QStringLiteral("#000080"), true);
    const QTextCharFormat commentaire = mmdeditFormat(QStringLiteral("#008000"), false, true);

    m_regles = {
      // Section d'un fichier INI ou table TOML.
      { QRegularExpression(QStringLiteral("^\\s*\\[[^\\]]+\\]\\s*$")), section },
      // Clé, jusqu'au deux-points ou au signe égal qui la termine.
      { QRegularExpression(QStringLiteral("^\\s*[\"']?[\\w.$/\\- ]+[\"']?\\s*(?=[:=])")), clef },
      { QRegularExpression(QStringLiteral("\"[^\"\\\\]*(\\\\.[^\"\\\\]*)*\"|'[^'\\n]*'")), chaine },
      { QRegularExpression(QStringLiteral("\\b-?\\d+(\\.\\d+)?([eE][+-]?\\d+)?\\b")), nombre },
      { mmdeditExprCasseIndifferente(
          QStringLiteral("\\b(true|false|null|none|yes|no|on|off)\\b")), litteral },
      // Un commentaire commence une ligne ou suit une espace : « https:// » et
      // un « # » collé à un mot ne sont donc pas pris pour tels.
      { QRegularExpression(QStringLiteral("(^|\\s)(#|;|//).*$")), commentaire },
    };
  }
};

/// Colore les scripts et le code : shell, PowerShell, batch, Python, Rust, C,
/// JavaScript, SQL…
///
/// Les mots-clés sont une liste commune, prise sans égard à la casse : il ne
/// s'agit pas d'analyser un langage, mais de faire ressortir la structure d'un
/// fichier qu'on relit. Les commentaires /* … */ courent sur plusieurs lignes
/// et sont donc suivis par état de bloc, comme ceux du XML.
class ColorateurCode : public ColorateurParRegles
{
public:
  enum Etat { EtatNormal = 0, EtatCommentaireBloc = 1 };

  explicit ColorateurCode(QTextDocument* document)
    : ColorateurParRegles(document)
    , m_commentaire(mmdeditFormat(QStringLiteral("#008000"), false, true))
    , m_debutBloc(QStringLiteral("(^|\\s)/\\*"))
    , m_finBloc(QStringLiteral("\\*/"))
  {
    const QTextCharFormat motCle = mmdeditFormat(QStringLiteral("#000080"), true);
    const QTextCharFormat chaine = mmdeditFormat(QStringLiteral("#8b0000"));
    const QTextCharFormat nombre = mmdeditFormat(QStringLiteral("#ff8c00"));
    const QTextCharFormat variable = mmdeditFormat(QStringLiteral("#800080"));

    m_regles = {
      { mmdeditExprCasseIndifferente(QStringLiteral(
          "\\b(if|else|elif|elseif|fi|then|for|foreach|while|do|done|case|esac|switch|break|"
          "continue|return|function|func|fn|def|lambda|class|struct|enum|impl|trait|interface|"
          "import|from|use|require|include|let|const|var|val|static|public|private|protected|"
          "new|delete|try|catch|except|finally|throw|raise|begin|end|echo|print|write|param|"
          "local|export|exit|goto|call|and|or|not|in|is|as|pass|yield|await|async|match|loop|"
          "mut|pub|self|this|null|nil|none|true|false|void|int|float|double|bool|string|char|"
          "select|insert|update|delete|where|join|inner|outer|left|right|group|order|by|having|"
          "limit|values|set|create|alter|drop|table|index|view|into|distinct|union)\\b")),
        motCle },
      // Variables de shell (« $nom », « ${nom} ») et de batch (« %nom% »).
      { QRegularExpression(QStringLiteral("\\$\\{?\\w+\\}?|%\\w+%")), variable },
      { QRegularExpression(QStringLiteral("\\b-?\\d+(\\.\\d+)?([eE][+-]?\\d+)?\\b")), nombre },
      { QRegularExpression(
          QStringLiteral("\"[^\"\\\\]*(\\\\.[^\"\\\\]*)*\"|'[^'\\n]*'|`[^`\\n]*`")), chaine },
      { QRegularExpression(QStringLiteral("(^|\\s)(#|//).*$")), m_commentaire },
      // Le « -- » de SQL n'est reconnu qu'en tête de ligne : ailleurs, c'est le
      // séparateur d'options d'une commande (« cp -- fichier »), pas un commentaire.
      { QRegularExpression(QStringLiteral("^\\s*--.*$")), m_commentaire },
      { mmdeditExprCasseIndifferente(QStringLiteral("^\\s*(rem|::)\\s.*$")), m_commentaire },
    };
  }

protected:
  void highlightBlock(const QString& texte) override
  {
    setCurrentBlockState(EtatNormal);

    int depart = 0;
    if (previousBlockState() == EtatCommentaireBloc) {
      const QRegularExpressionMatch fin = m_finBloc.match(texte);
      if (!fin.hasMatch()) {
        setFormat(0, texte.length(), m_commentaire);
        setCurrentBlockState(EtatCommentaireBloc);
        return;
      }
      setFormat(0, fin.capturedEnd(), m_commentaire);
      depart = fin.capturedEnd();
    }

    const QString reste = texte.mid(depart);
    appliquerRegles(reste, depart);

    // Un bloc ouvert et non refermé sur cette ligne court sur la suivante.
    const QRegularExpressionMatch ouverture = m_debutBloc.match(texte, depart);
    if (ouverture.hasMatch() && !m_finBloc.match(texte, ouverture.capturedEnd()).hasMatch()) {
      setFormat(ouverture.capturedStart(), texte.length() - ouverture.capturedStart(),
                m_commentaire);
      setCurrentBlockState(EtatCommentaireBloc);
    }
  }

private:
  QTextCharFormat m_commentaire;
  QRegularExpression m_debutBloc;
  QRegularExpression m_finBloc;
};

/// Colore un journal : niveau de gravité et horodatage.
///
/// C'est le niveau qu'on cherche des yeux en parcourant un fichier de journal ;
/// l'horodatage, lui, est mis en retrait pour ne pas encombrer la lecture.
class ColorateurJournal : public ColorateurParRegles
{
public:
  explicit ColorateurJournal(QTextDocument* document)
    : ColorateurParRegles(document)
  {
    const QTextCharFormat horodatage = mmdeditFormat(QStringLiteral("#808080"));
    const QTextCharFormat erreur = mmdeditFormat(QStringLiteral("#b22222"), true);
    const QTextCharFormat alerte = mmdeditFormat(QStringLiteral("#ff8c00"), true);
    const QTextCharFormat information = mmdeditFormat(QStringLiteral("#000080"));
    const QTextCharFormat mise_au_point = mmdeditFormat(QStringLiteral("#808080"), false, true);

    m_regles = {
      // ISO 8601 (« 2026-10-10 09:41:06,253 ») puis forme syslog (« Oct 10 09:41:06 »).
      { QRegularExpression(QStringLiteral(
          "^\\s*\\[?\\d{4}-\\d{2}-\\d{2}[T ]\\d{2}:\\d{2}:\\d{2}([.,]\\d+)?\\]?")), horodatage },
      { QRegularExpression(QStringLiteral(
          "^[A-Z][a-z]{2}\\s+\\d{1,2}\\s+\\d{2}:\\d{2}:\\d{2}")), horodatage },
      { mmdeditExprCasseIndifferente(QStringLiteral(
          "\\b(error|erreur|fatal|critical|crit|severe|failed|failure|échec|echec)\\b")),
        erreur },
      { mmdeditExprCasseIndifferente(
          QStringLiteral("\\b(warn|warning|attention|avertissement)\\b")), alerte },
      { mmdeditExprCasseIndifferente(QStringLiteral("\\b(info|notice)\\b")), information },
      { mmdeditExprCasseIndifferente(QStringLiteral("\\b(debug|trace|verbose)\\b")),
        mise_au_point },
    };
  }
};
