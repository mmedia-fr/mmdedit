// SPDX-License-Identifier: GPL-3.0-or-later
//
// Ce que QML ne sait pas faire sur un TextArea, et qui exige le QTextDocument
// qui se trouve derrière.
//
// Motif : QML n'offre que `remove()` puis `insert()`, deux opérations distinctes
// dans la pile d'annulation. Un Ctrl+Z après un clic sur « Titre 2 » rendait
// alors un état intermédiaire — la ligne retirée, pas encore réinsérée. Un
// QTextCursor encadré par beginEditBlock()/endEditBlock() n'en fait qu'une.
#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QTextCursor>
#include <QtGui/QTextDocument>
#include <QtQuick/QQuickTextDocument>

class PontTexte : public QObject
{
  Q_OBJECT
public:
  using QObject::QObject;

  /// Remplace [debut, fin) par `remplacement`, en une seule opération annulable.
  Q_INVOKABLE void appliquer(QQuickTextDocument* document, int debut, int fin,
                             const QString& remplacement) const
  {
    QTextDocument* doc = document ? document->textDocument() : nullptr;
    if (!doc)
      return;
    QTextCursor curseur(doc);
    curseur.beginEditBlock();
    curseur.setPosition(qBound(0, debut, doc->characterCount() - 1));
    curseur.setPosition(qBound(0, fin, doc->characterCount() - 1), QTextCursor::KeepAnchor);
    curseur.insertText(remplacement);
    curseur.endEditBlock();
  }
};
