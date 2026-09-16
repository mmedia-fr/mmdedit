// SPDX-License-Identifier: GPL-3.0-or-later
//
// Ce que QML ne sait pas faire sur un TextArea, et qui exige le QTextDocument
// qui se trouve derrière : annulation en un bloc, coloration syntaxique, export
// PDF. Trois services, un seul objet, parce qu'ils partagent ce même accès.
#pragma once

#include <QtCore/QFileInfo>
#include <QtCore/QObject>
#include <QtCore/QPointer>
#include <QtCore/QString>
#include <QtCore/QUrl>
#include <QtGui/QPageSize>
#include <QtGui/QPdfWriter>
#include <QtGui/QTextCursor>
#include <QtGui/QTextDocument>
#include <QtQuick/QQuickTextDocument>

#include "colorateur.h"

class PontTexte : public QObject
{
  Q_OBJECT
public:
  using QObject::QObject;

  /// Remplace [debut, fin) par `remplacement`, en une seule opération annulable.
  ///
  /// QML n'offre que `remove()` puis `insert()`, deux opérations distinctes : un
  /// Ctrl+Z après un clic sur « Titre 2 » rendait un état intermédiaire — la
  /// ligne retirée, pas encore réinsérée. Un QTextCursor encadré par
  /// beginEditBlock()/endEditBlock() n'en fait qu'une.
  Q_INVOKABLE void appliquer(QQuickTextDocument* document, int debut, int fin,
                             const QString& remplacement) const
  {
    QTextDocument* doc = document ? document->textDocument() : nullptr;
    if (!doc)
      return;
    const int dernier = doc->characterCount() - 1;
    QTextCursor curseur(doc);
    curseur.beginEditBlock();
    curseur.setPosition(qBound(0, debut, dernier));
    curseur.setPosition(qBound(0, fin, dernier), QTextCursor::KeepAnchor);
    curseur.insertText(remplacement);
    curseur.endEditBlock();
  }

  /// Attache la coloration correspondant au format, ou la retire.
  ///
  /// Un seul colorateur à la fois : l'ancien est détruit, ce qui le détache du
  /// document (l'équivalent du `setDocument(None)` de la version PySide6).
  Q_INVOKABLE void colorer(QQuickTextDocument* document, const QString& format)
  {
    QTextDocument* doc = document ? document->textDocument() : nullptr;
    delete m_colorateur;
    m_colorateur = nullptr;
    if (!doc)
      return;
    if (format == QStringLiteral("markdown"))
      m_colorateur = new ColorateurMarkdown(doc);
    else if (format == QStringLiteral("xml"))
      m_colorateur = new ColorateurXml(doc);
  }

  /// Écrit le texte en PDF, rendu comme dans l'aperçu. Rend faux en cas d'échec.
  ///
  /// Le document d'impression est monté à part : celui de l'éditeur porte du
  /// texte brut, alors que le PDF doit recevoir le Markdown rendu.
  Q_INVOKABLE bool exporterPdf(const QUrl& fichier, const QString& texte, bool markdown) const
  {
    const QString chemin = fichier.isLocalFile() ? fichier.toLocalFile() : fichier.toString();
    if (chemin.isEmpty())
      return false;

    QTextDocument document;
    if (markdown)
      document.setMarkdown(texte);
    else
      document.setPlainText(texte);

    QPdfWriter ecrivain(chemin);
    ecrivain.setPageSize(QPageSize(QPageSize::A4));
    ecrivain.setCreator(QStringLiteral("MMdedit"));
    document.print(&ecrivain);
    // QPdfWriter n'ouvre le fichier qu'à l'impression et ne signale pas l'échec :
    // c'est le fichier obtenu qui fait foi.
    return QFileInfo(chemin).size() > 0;
  }

private:
  // Détruit avec l'objet, ou remplacé par colorer().
  QPointer<QSyntaxHighlighter> m_colorateur;
};
