// SPDX-License-Identifier: GPL-3.0-or-later
#include <QtCore/QCoreApplication>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtCore/QVariant>
#include <QtGui/QGuiApplication>
#include <QtQml/QQmlApplicationEngine>

#include <cstdio>
#include <cstring>

int main(int argc, char* argv[])
{
  bool smoke = false;
  for (int i = 1; i < argc; ++i)
    if (std::strcmp(argv[i], "--smoke") == 0)
      smoke = true;

  QGuiApplication app(argc, argv);
  QCoreApplication::setApplicationName(QStringLiteral("MMdedit"));
  QCoreApplication::setOrganizationDomain(QStringLiteral("mmedia.fr"));
  QCoreApplication::setApplicationVersion(QStringLiteral(MMDEDIT_VERSION));

  QQmlApplicationEngine engine;
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
    QTimer::singleShot(0, &app, [&engine] {
      if (engine.rootObjects().isEmpty()) {
        std::fprintf(stderr, "smoke: aucune fenetre creee\n");
        QCoreApplication::exit(1);
        return;
      }
      const QString noyau = engine.rootObjects().first()->property("noyau").toString();
      std::printf("smoke: %s\n", qPrintable(noyau));
      std::fflush(stdout);
      QCoreApplication::exit(noyau.startsWith(QStringLiteral("mmdedit_core")) ? 0 : 2);
    });
  }
  return app.exec();
}
