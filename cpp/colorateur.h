// SPDX-License-Identifier: GPL-3.0-or-later
//
// Colorations syntaxiques de la zone d'édition. Markdown et XML sont portés de
// src/mmdedit/highlighter.py sans changer les couleurs ni les règles ; les
// configurations, le code et les journaux s'y sont ajoutés, sur la même palette.
// Sous une apparence à fond sombre (« nuit »), chaque teinte a son équivalent
// éclairci — le rôle ne change pas, la teinte suit le fond.
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

namespace mmdedit {

/// Rôles colorés : ce qui est mis en valeur, indépendamment de la teinte. Un
/// même rôle garde sa fonction d'un fond à l'autre — titre, commentaire,
/// chaîne… — seule la couleur change.
enum class Role {
  Titre,        ///< Titre Markdown, section de configuration, mot-clé de code.
  Code,         ///< Code en ligne Markdown, valeur d'attribut XML, chaîne.
  Lien,         ///< Lien Markdown.
  Citation,     ///< Ligne citée Markdown.
  Puce,         ///< Début d'élément de liste.
  Filet,        ///< Séparateur horizontal, déclaration XML.
  Balise,       ///< Balise XML.
  Attribut,     ///< Attribut XML, variable de script.
  Valeur,       ///< Valeur d'attribut, chaîne de configuration.
  Commentaire,  ///< Commentaire — XML, configuration, code.
  Entite,       ///< Entité XML, nombre et horodatage mis en valeur.
  Section,      ///< Section d'un fichier INI ou d'une table TOML.
  Clef,         ///< Clé de configuration.
  Nombre,       ///< Nombre.
  Litteral,     ///< Littéral (true, null, none…).
  Variable,     ///< Variable de shell ou de batch.
  Horodatage,   ///< Horodatage de journal.
  Erreur,       ///< Niveau de gravité d'erreur.
  Alerte,       ///< Niveau de gravité d'alerte.
  Information,  ///< Niveau de gravité d'information.
  MiseAuPoint,  ///< Niveau de gravité bas (debug, trace).
};

/// La teinte d'un rôle sous un fond clair ou sombre. Les teintes claires sont
/// celles du portage (héritées de la version PySide6) ; les sombres sont leurs
/// équivalents éclaircis, tenus à 4,5:1 au moins sur le fond `#24272e` que
/// l'apparence « nuit » donne à la zone d'édition.
inline QString couleur(Role role, bool sombre)
{
  switch (role) {
  case Role::Titre:       return sombre ? QStringLiteral("#91cbf2") : QStringLiteral("#000080");
  case Role::Code:        return sombre ? QStringLiteral("#ff9d8f") : QStringLiteral("#8b0000");
  case Role::Lien:       return sombre ? QStringLiteral("#5cc6f2") : QStringLiteral("#0000cd");
  case Role::Citation:    return sombre ? QStringLiteral("#a3c98a") : QStringLiteral("#556b2f");
  case Role::Puce:       return sombre ? QStringLiteral("#d2a8ff") : QStringLiteral("#800080");
  case Role::Filet:       return sombre ? QStringLiteral("#9aa8b6") : QStringLiteral("#808080");
  case Role::Balise:      return sombre ? QStringLiteral("#91cbf2") : QStringLiteral("#000080");
  case Role::Attribut:    return sombre ? QStringLiteral("#d2a8ff") : QStringLiteral("#800080");
  case Role::Valeur:      return sombre ? QStringLiteral("#ff9d8f") : QStringLiteral("#8b0000");
  case Role::Commentaire: return sombre ? QStringLiteral("#8ccf8c") : QStringLiteral("#008000");
  case Role::Entite:      return sombre ? QStringLiteral("#ffb86c") : QStringLiteral("#ff8c00");
  case Role::Section:     return sombre ? QStringLiteral("#91cbf2") : QStringLiteral("#000080");
  case Role::Clef:        return sombre ? QStringLiteral("#d2a8ff") : QStringLiteral("#800080");
  case Role::Nombre:      return sombre ? QStringLiteral("#ffb86c") : QStringLiteral("#ff8c00");
  case Role::Litteral:    return sombre ? QStringLiteral("#91cbf2") : QStringLiteral("#000080");
  case Role::Variable:    return sombre ? QStringLiteral("#d2a8ff") : QStringLiteral("#800080");
  case Role::Horodatage:  return sombre ? QStringLiteral("#9aa8b6") : QStringLiteral("#808080");
  case Role::Erreur:      return sombre ? QStringLiteral("#ff7b72") : QStringLiteral("#b22222");
  case Role::Alerte:      return sombre ? QStringLiteral("#ffb86c") : QStringLiteral("#ff8c00");
  case Role::Information:  return sombre ? QStringLiteral("#91cbf2") : QStringLiteral("#000080");
  case Role::MiseAuPoint: return sombre ? QStringLiteral("#9aa8b6") : QStringLiteral("#808080");
  }
  return QString();
}

} // namespace mmdedit

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
  explicit ColorateurMarkdown(QTextDocument* document, bool sombre)
    : QSyntaxHighlighter(document)
  {
    const QTextCharFormat titre = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Titre, sombre), true);
    const QTextCharFormat gras = mmdeditFormat(QString(), true);
    const QTextCharFormat italique = mmdeditFormat(QString(), false, true);
    const QTextCharFormat barre = mmdeditFormat(QString(), false, false, false, true);
    const QTextCharFormat code = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Code, sombre), false, false, true);
    const QTextCharFormat lien = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Lien, sombre));
    const QTextCharFormat citation = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Citation, sombre), false, true);
    const QTextCharFormat puce = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Puce, sombre), true);
    const QTextCharFormat filet = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Filet, sombre), true);

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

  explicit ColorateurXml(QTextDocument* document, bool sombre)
    : QSyntaxHighlighter(document)
    , m_balise(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Balise, sombre), true))
    , m_attribut(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Attribut, sombre)))
    , m_valeur(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Valeur, sombre)))
    , m_commentaire(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Commentaire, sombre), false, true))
    , m_entite(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Entite, sombre)))
    , m_declaration(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Filet, sombre), true))
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
  explicit ColorateurDonnees(QTextDocument* document, bool sombre)
    : ColorateurParRegles(document)
  {
    const QTextCharFormat section = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Section, sombre), true);
    const QTextCharFormat clef = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Clef, sombre));
    const QTextCharFormat chaine = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Valeur, sombre));
    const QTextCharFormat nombre = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Nombre, sombre));
    const QTextCharFormat litteral = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Litteral, sombre), true);
    const QTextCharFormat commentaire = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Commentaire, sombre), false, true);

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

  explicit ColorateurCode(QTextDocument* document, bool sombre)
    : ColorateurParRegles(document)
    , m_commentaire(mmdeditFormat(mmdedit::couleur(mmdedit::Role::Commentaire, sombre), false, true))
    , m_debutBloc(QStringLiteral("(^|\\s)/\\*"))
    , m_finBloc(QStringLiteral("\\*/"))
  {
    const QTextCharFormat motCle = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Titre, sombre), true);
    const QTextCharFormat chaine = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Code, sombre));
    const QTextCharFormat nombre = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Nombre, sombre));
    const QTextCharFormat variable = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Variable, sombre));

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
  explicit ColorateurJournal(QTextDocument* document, bool sombre)
    : ColorateurParRegles(document)
  {
    const QTextCharFormat horodatage = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Horodatage, sombre));
    const QTextCharFormat erreur = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Erreur, sombre), true);
    const QTextCharFormat alerte = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Alerte, sombre), true);
    const QTextCharFormat information = mmdeditFormat(mmdedit::couleur(mmdedit::Role::Information, sombre));
    const QTextCharFormat mise_au_point = mmdeditFormat(mmdedit::couleur(mmdedit::Role::MiseAuPoint, sombre), false, true);

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
