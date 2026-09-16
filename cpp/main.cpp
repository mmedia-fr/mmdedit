// SPDX-License-Identifier: GPL-3.0-or-later
#include <QtCore/QCoreApplication>
#include <QtCore/QFileInfo>
#include <QtCore/QString>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtCore/QVariant>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlContext>
#include <QtQuick/QQuickWindow>
#include <QtQml/qqml.h>

#include "pont_texte.h"
#include <QtGui/QImage>

#include <cstdio>
#include <cstring>

int main(int argc, char* argv[])
{
  bool smoke = false;
  // Capture de la fenêtre dans un fichier PNG, puis sortie : sert à juger
  // l'interface depuis une machine sans écran (la plateforme « offscreen » suffit).
  QString capture;
  // Fichier passé en ligne de commande : c'est ainsi que Windows lance MMdedit
  // depuis « Ouvrir avec » (la commande enregistrée est « MMdedit.exe "%1" »).
  QString fichier;
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--smoke") == 0)
      smoke = true;
    else if (std::strcmp(argv[i], "--capture") == 0 && i + 1 < argc)
      capture = QString::fromLocal8Bit(argv[++i]);
    else if (fichier.isEmpty())
      fichier = QString::fromLocal8Bit(argv[i]);
  }

  QGuiApplication app(argc, argv);
  QCoreApplication::setApplicationName(QStringLiteral("MMdedit"));
  QCoreApplication::setOrganizationName(QStringLiteral("M-Media"));
  QCoreApplication::setOrganizationDomain(QStringLiteral("mmedia.fr"));
  QCoreApplication::setApplicationVersion(QStringLiteral(MMDEDIT_VERSION));
  QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/assets/mmdedit.ico")));

  // Type natif exposé à QML : il donne accès au QTextDocument du TextArea, hors
  // de portée du QML et du noyau Rust. URI distinct de celui du module cxx-qt.
  qmlRegisterType<PontTexte>("fr.mmedia.mmdedit.natif", 1, 0, "PontTexte");

  QQmlApplicationEngine engine;
  // L'URL est vide si aucun fichier n'est donné, ou si le chemin ne désigne rien :
  // ouvrir à l'aveugle afficherait une erreur de lecture au démarrage.
  QUrl fichierInitial;
  if (!fichier.isEmpty() && QFileInfo::exists(fichier))
    fichierInitial = QUrl::fromLocalFile(QFileInfo(fichier).absoluteFilePath());
  engine.rootContext()->setContextProperty(QStringLiteral("fichierInitial"), fichierInitial);

  // Chargement par URL qrc plutôt que loadFromModule(), absent de Qt 6.4 (Debian 12).
  const QUrl url(QStringLiteral("qrc:/qt/qml/fr/mmedia/mmdedit/qml/Main.qml"));
  QObject::connect(
    &engine, &QQmlApplicationEngine::objectCreated, &app,
    [url](QObject* obj, const QUrl& objUrl) {
      if (!obj && url == objUrl)
        QCoreApplication::exit(1);
    },
    Qt::QueuedConnection);
  engine.load(url);

  if (smoke) {
    // Contrôle de fabrication : la fenêtre QML est créée et l'objet Rust répond.
    QTimer::singleShot(0, &app, [&engine, fichierInitial] {
      if (engine.rootObjects().isEmpty()) {
        std::fprintf(stderr, "smoke: aucune fenetre creee\n");
        QCoreApplication::exit(1);
        return;
      }
      QObject* racine = engine.rootObjects().first();
      const QString noyau = racine->property("noyau").toString();
      // Le noyau doit aussi répondre sur ce qu'il porte désormais : le document.
      QVariant comptes;
      const bool appel = QMetaObject::invokeMethod(
        racine, "statistiquesDeControle", Q_RETURN_ARG(QVariant, comptes));
      // Un fichier donné en argument doit être arrivé dans l'éditeur : c'est le
      // chemin complet — lecture, décodage, affichage — et non le seul noyau.
      QVariant charge;
      QMetaObject::invokeMethod(racine, "texteCourant", Q_RETURN_ARG(QVariant, charge));
      std::printf("smoke: %s | %s | %lld caracteres lus\n", qPrintable(noyau),
                  qPrintable(comptes.toString()),
                  static_cast<long long>(charge.toString().size()));
      std::fflush(stdout);
      if (!noyau.startsWith(QStringLiteral("mmdedit_core"))) {
        QCoreApplication::exit(2);
        return;
      }
      if (!appel || !comptes.toString().contains(QStringLiteral("2 mots"))) {
        QCoreApplication::exit(3);
        return;
      }
      if (!fichierInitial.isEmpty() && charge.toString().isEmpty()) {
        QCoreApplication::exit(4);
        return;
      }
      // Édition assistée : le contrôle est écrit en QML, là où il peut agir sur
      // le TextArea comme le ferait un clic sur la barre d'outils.
      QVariant edition;
      QMetaObject::invokeMethod(racine, "controleEdition", Q_RETURN_ARG(QVariant, edition));
      std::printf("smoke: edition %s\n", qPrintable(edition.toString()));
      std::fflush(stdout);
      QCoreApplication::exit(edition.toString() == QStringLiteral("ok") ? 0 : 7);
    });
  }

  if (!capture.isEmpty()) {
    // Deux passages d'événements avant la saisie : la fenêtre doit être peinte,
    // et l'aperçu n'est rendu qu'après le minuteur d'anti-rebond de 200 ms.
    QTimer::singleShot(600, &app, [&engine, capture] {
      if (engine.rootObjects().isEmpty()) {
        QCoreApplication::exit(1);
        return;
      }
      auto* fenetre = qobject_cast<QQuickWindow*>(engine.rootObjects().first());
      if (!fenetre) {
        std::fprintf(stderr, "capture: la racine n'est pas une fenetre\n");
        QCoreApplication::exit(5);
        return;
      }
      const QImage image = fenetre->grabWindow();
      QCoreApplication::exit(image.save(capture) ? 0 : 6);
    });
  }
  return app.exec();
}
