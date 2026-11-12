// SPDX-License-Identifier: GPL-3.0-or-later
#include <QtCore/QCoreApplication>
#include <QtCore/QDir>
#include <QtCore/QFile>
#include <QtCore/QFileInfo>
#include <QtCore/QSettings>
#include <QtCore/QTemporaryFile>
#include <QtCore/QThread>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtCore/QVariant>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#if QT_CONFIG(sessionmanager)
#  include <QtGui/QSessionManager>
#endif
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlContext>
#include <QtQuick/QQuickWindow>
#include <QtQuickControls2/QQuickStyle>
#include <QtQml/qqml.h>

#include <QtGui/QImage>

namespace {
// Le test de fumée fabrique son propre .docx d'import : un ZIP stocké sans
// compression — entête locale, répertoire central et somme CRC apposées à la
// main, les deux pages du format suffisent à l'import (étape 6 de la
// feuille de route du 12/11).

void ajout16(QByteArray& flux, quint16 valeur)
{
  flux.append(char(quint8(valeur & 0xFF)));
  flux.append(char(quint8(valeur >> 8)));
}

void ajout32(QByteArray& flux, quint32 valeur)
{
  for (int decalage = 0; decalage < 32; decalage += 8) {
    flux.append(char(quint8(valeur >> decalage)));
  }
}

quint32 crc32(const QByteArray& donnees)
{
  quint32 crc = 0xFFFFFFFFu;
  for (const char octet : donnees) {
    crc ^= quint8(octet);
    for (int bit = 0; bit < 8; ++bit) {
      const quint32 masque = (crc & 1u) ? 0xEDB88320u : 0u;
      crc = (crc >> 1) ^ masque;
    }
  }
  return crc ^ 0xFFFFFFFFu;
}

QByteArray zipUnMembre(const QByteArray& nom, const QByteArray& contenu)
{
  const quint32 somme = crc32(contenu);
  const quint16 taille = quint16(contenu.size());
  const quint16 nomTaille = quint16(nom.size());
  QByteArray local;
  local.append("PK\x03\x04");
  ajout16(local, 20);        // version requise
  ajout16(local, 0);         // attributs généraux
  ajout16(local, 0);        // stocké, sans compression
  ajout16(local, 0);        // heure de modification
  ajout16(local, 0);        // date de modification
  ajout32(local, somme);
  ajout32(local, taille);
  ajout32(local, taille);
  ajout16(local, nomTaille);
  ajout16(local, 0);        // attributs étendus : aucun
  local.append(nom);
  local.append(contenu);
  const quint32 departLocale = 0;

  QByteArray central;
  central.append("PK\x01\x02");
  ajout16(central, 20);     // version créatrice
  ajout16(central, 20);     // version requise
  ajout16(central, 0);      // attributs généraux
  ajout16(central, 0);     // stocké
  ajout16(central, 0);      // heure de modification
  ajout16(central, 0);      // date de modification
  ajout32(central, somme);
  ajout32(central, taille);
  ajout32(central, taille);
  ajout16(central, nomTaille);
  ajout16(central, 0);      // attributs étendus : aucun
  ajout16(central, 0);      // commentaire : aucun
  ajout16(central, 0);      // numéro de disque de départ
  ajout16(central, 0);      // attributs internes
  ajout32(central, 0);      // attributs externes
  ajout32(central, departLocale);
  central.append(nom);

  QByteArray fin;
  fin.append("PK\x05\x06");
  ajout16(fin, 0);          // disque courant
  ajout16(fin, 0);          // disque du répertoire
  ajout16(fin, 1);          // membres sur ce disque
  ajout16(fin, 1);          // membres au total
  ajout32(fin, quint32(central.size()));
  ajout32(fin, quint32(departLocale + local.size()));
  ajout16(fin, 0);          // commentaire : aucun

  QByteArray archive;
  archive.append(local);
  archive.append(central);
  archive.append(fin);
  return archive;
}

}  // namespace

#include "installeur.h"
#include "pont_texte.h"
#include "presse_papier.h"

#ifdef Q_OS_WIN
#  include <windows.h>  // GetCommandLineW / CommandLineToArgvW
#  include <shellapi.h>
#endif

#include <cstdio>

/// Arguments du programme, en Unicode et sans perte.
///
/// Sous Windows, la seule source fiable est la ligne de commande native :
/// `argv` est encodé dans la page de code ANSI. Ailleurs, argv est déjà de
/// l'UTF-8 et QCoreApplication::arguments() convient.
static QStringList argumentsUnicode()
{
#ifdef Q_OS_WIN
  int nombre = 0;
  LPWSTR* natifs = CommandLineToArgvW(GetCommandLineW(), &nombre);
  if (!natifs)
    return QCoreApplication::arguments();
  QStringList liste;
  liste.reserve(nombre);
  for (int i = 0; i < nombre; ++i)
    liste.append(QString::fromWCharArray(natifs[i]));
  LocalFree(natifs);
  return liste;
#else
  return QCoreApplication::arguments();
#endif
}

/// Réglage valant pour toute la machine : la variable d'environnement d'abord,
/// puis la clé de portée machine — `HKLM\Software\M-Media\MMdedit` sous Windows,
/// `/etc/xdg/M-Media/MMdedit.conf` sous Linux. Même règle que MMail.
static QString reglageMachine(const char* variable, const QString& cle)
{
  const QString valeur = qEnvironmentVariable(variable);
  if (!valeur.isEmpty())
    return valeur;
  QSettings machine(QSettings::NativeFormat, QSettings::SystemScope,
                    QCoreApplication::organizationName(), QCoreApplication::applicationName());
  return machine.value(cle).toString().trimmed();
}

int main(int argc, char* argv[])
{
  QGuiApplication app(argc, argv);
  QCoreApplication::setApplicationName(QStringLiteral("MMdedit"));
  QCoreApplication::setOrganizationName(QStringLiteral("M-Media"));
  QCoreApplication::setOrganizationDomain(QStringLiteral("mmedia.fr"));
  QCoreApplication::setApplicationVersion(QStringLiteral(MMDEDIT_VERSION));
  QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/assets/mmdedit.ico")));

  bool smoke = false;
  // Capture de la fenêtre dans un fichier PNG, puis sortie : sert à juger
  // l'interface depuis une machine sans écran (la plateforme « offscreen » suffit).
  QString capture;
  // Fichier passé en ligne de commande : c'est ainsi que Windows lance MMdedit
  // depuis « Ouvrir avec » (la commande enregistrée est « MMdedit.exe "%1" »).
  QString fichier;
  // Les arguments ne sont pas lus dans argv : sous Windows, il arrive dans la
  // page de code ANSI, qui remplace par « ? » tout ce qu'elle ne sait pas
  // représenter — un fichier « Réunion été.md » n'était alors jamais ouvert.
  // QCoreApplication::arguments() ne suffit pas : mesuré sur le banc, il rend
  // encore la version dégradée. On demande donc la ligne de commande Unicode à
  // Windows lui-même.
  const QStringList arguments = argumentsUnicode();
  for (int i = 1; i < arguments.size(); ++i) {
    const QString argument = arguments.at(i);
    if (argument == QStringLiteral("--smoke"))
      smoke = true;
    else if (argument == QStringLiteral("--capture") && i + 1 < arguments.size())
      capture = arguments.at(++i);
    else if (fichier.isEmpty())
      fichier = argument;
  }

  // Type natif exposé à QML : il donne accès au QTextDocument du TextArea, hors
  // de portée du QML et du noyau Rust. URI distinct de celui du module cxx-qt.
  qmlRegisterType<PontTexte>("fr.mmedia.mmdedit.natif", 1, 0, "PontTexte");
  qmlRegisterType<PressePapier>("fr.mmedia.mmdedit.natif", 1, 0, "PressePapier");
  // Installation d'une mise à jour à la fermeture (cf. core/src/mise_a_jour.rs).
  qmlRegisterType<Installeur>("fr.mmedia.mmdedit.natif", 1, 0, "Installeur");

  // Fusion plutôt que le style natif : c'est le seul à honorer une palette sur
  // les quatre cibles, et les apparences de MMdedit (classique, moderne) n'ont
  // pas d'autre moyen de s'appliquer depuis QML — la version PySide6 les posait
  // par feuille de style Qt, sans équivalent ici.
  QQuickStyle::setStyle(QStringLiteral("Fusion"));

  QQmlApplicationEngine engine;
  // Un fichier donné en argument est transmis tel quel, même s'il n'existe pas :
  // le noyau rend alors « Lecture impossible » et l'utilisateur voit pourquoi sa
  // fenêtre est vide. L'ignorer en silence masquait le défaut d'encodage des
  // arguments — un nom accentué n'était tout simplement jamais ouvert.
  QUrl fichierInitial;
  if (!fichier.isEmpty())
    fichierInitial = QUrl::fromLocalFile(QFileInfo(fichier).absoluteFilePath());
  engine.rootContext()->setContextProperty(QStringLiteral("fichierInitial"), fichierInitial);
  // En contrôle de fabrication, l'interface ne sort pas sur le réseau de sa
  // propre initiative.
  engine.rootContext()->setContextProperty(QStringLiteral("modeControle"), smoke);
  // Avis de nouvelle version : coupé pour la machine (« AvisVersion » à 0) sur
  // un parc dont l'administrateur déploie lui-même les mises à jour.
  engine.rootContext()->setContextProperty(
    QStringLiteral("avisVersion"),
    reglageMachine("MMDEDIT_AVIS_VERSION", QStringLiteral("AvisVersion")) != QStringLiteral("0"));
  // Commodité de développement (cf. core/src/mise_a_jour.rs) : la vérification
  // part aussitôt, pour qu'une capture montre le bandeau.
  engine.rootContext()->setContextProperty(QStringLiteral("essaiAvis"),
                                           qEnvironmentVariableIsSet("MMDEDIT_AVIS_ESSAI"));
  // Scénario d'essai de la mise à jour assistée (« mise-a-jour »,
  // « mise-a-jour-fermeture ») : MMdedit choisit lui-même, comme le ferait un
  // clic, pour que l'intégration continue l'éprouve de bout en bout.
  engine.rootContext()->setContextProperty(QStringLiteral("scenarioEssai"),
                                           qEnvironmentVariable("MMDEDIT_SCENARIO"));

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
    QTimer::singleShot(0, &app, [&engine, fichierInitial, fichier] {
      if (engine.rootObjects().isEmpty()) {
        std::fprintf(stderr, "smoke: aucune fenetre creee\n");
        QCoreApplication::exit(1);
        return;
      }
      QObject* racine = engine.rootObjects().first();
      // Un fichier qu'on vient d'ouvrir n'est pas modifié. La coloration Markdown,
      // appliquée par Qt après le chargement, le faisait croire : astérisque au
      // titre et « enregistrer ? » à la fermeture d'un document seulement lu
      // (0.4.2 et avant). Les événements en attente passent d'abord, puisque
      // c'est parmi eux que la coloration arrive.
      QCoreApplication::processEvents();
      const bool modifieALOuverture = racine->property("modifie").toBool();
      const QString noyau = racine->property("noyau").toString();
      // Le noyau doit aussi répondre sur ce qu'il porte désormais : le document.
      QVariant comptes;
      const bool appel = QMetaObject::invokeMethod(
        racine, "statistiquesDeControle", Q_RETURN_ARG(QVariant, comptes));
      // Un fichier donné en argument doit être arrivé dans l'éditeur : c'est le
      // chemin complet — lecture, décodage, affichage — et non le seul noyau.
      QVariant charge;
      QMetaObject::invokeMethod(racine, "texteCourant", Q_RETURN_ARG(QVariant, charge));
      std::printf("smoke: %s | %s | %lld caracteres lus\n", noyau.toUtf8().constData(),
                  comptes.toString().toUtf8().constData(),
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
        std::fprintf(stderr, "smoke: le fichier donne en argument n'est pas arrive dans l'editeur\n");
        std::fprintf(stderr, "smoke: argument recu = %s\n", fichier.toUtf8().constData());
        QCoreApplication::exit(4);
        return;
      }
      if (!fichierInitial.isEmpty() && modifieALOuverture) {
        std::fprintf(stderr, "smoke: le document est marque modifie des l'ouverture\n");
        QCoreApplication::exit(9);
        return;
      }
      // Édition assistée : le contrôle est écrit en QML, là où il peut agir sur
      // le TextArea comme le ferait un clic sur la barre d'outils.
      QVariant edition;
      QMetaObject::invokeMethod(racine, "controleEdition", Q_RETURN_ARG(QVariant, edition));
      std::printf("smoke: edition %s\n", edition.toString().toUtf8().constData());
      std::fflush(stdout);
      if (edition.toString() != QStringLiteral("ok")) {
        QCoreApplication::exit(7);
        return;
      }

      // Export PDF : contrôlé ici, où l'on peut relire le fichier produit.
      QTemporaryFile sortie(QDir::tempPath() + QStringLiteral("/mmdedit-XXXXXX.pdf"));
      sortie.open();
      const QString cheminPdf = sortie.fileName();
      sortie.close();
      PontTexte pont;
      const bool ecrit = pont.exporterPdf(QUrl::fromLocalFile(cheminPdf),
                                          QStringLiteral("# Titre\n\nUn paragraphe."), true);
      QFile relu(cheminPdf);
      const bool entete = relu.open(QIODevice::ReadOnly) && relu.read(4) == QByteArray("%PDF");
      std::printf("smoke: pdf ecrit=%d entete=%d taille=%lld\n", ecrit, entete,
                  static_cast<long long>(QFileInfo(cheminPdf).size()));
      std::fflush(stdout);

      // Feuilleteur (lecture seule) : le PDF tout juste écrit s'y rouvre sans
      // que le moindre texte soit lu par le noyau, et ses pages arrivent.
      QVariant ouvert;
      QMetaObject::invokeMethod(racine, "ouvrirPdf", Q_RETURN_ARG(QVariant, ouvert),
                                Q_ARG(QVariant, QUrl::fromLocalFile(cheminPdf)));
      // Sous Android, le module QtPdf n'existe pas : le feuilleteur ne s'y
      // instancie jamais, aucune page n'y peut arriver. Le contrôle attend
      // alors l'avertissement tenant lieu de vue, pas les pages.
      const bool avecPages = racine && racine->property("feuilleteurDisponible").toBool();
      int pages = 0;
      for (int i = 0; i < 300 && avecPages && pages <= 0; ++i) {
        QThread::msleep(10);
        QCoreApplication::processEvents();
        if (racine)
          pages = racine->property("pagesPdf").toInt();
      }
      QVariant etatFeuilleteur;
      QMetaObject::invokeMethod(racine, "controleFeuilleteur",
                                Q_RETURN_ARG(QVariant, etatFeuilleteur),
                                Q_ARG(QVariant, false));
      std::printf("smoke: feuilleteur ouvert=%d pages=%d etat=%s\n", ouvert.toBool(), pages,
                  etatFeuilleteur.toString().toUtf8().constData());
      std::fflush(stdout);
      if (!ouvert.toBool() || (avecPages && pages <= 0)
          || etatFeuilleteur.toString() != QStringLiteral("ok")) {
        std::fprintf(stderr, "smoke: le feuilleteur n'a pas ouvert le PDF (pages=%d)\n", pages);
        QCoreApplication::exit(10);
        return;
      }
      // Retour à l'édition : « Nouveau » referme le feuilleteur et rend ses
      // commandes.
      QMetaObject::invokeMethod(racine, "nouveau");
      for (int i = 0; i < 10; ++i) {
        QThread::msleep(10);
        QCoreApplication::processEvents();
      }
      QMetaObject::invokeMethod(racine, "controleFeuilleteur",
                                Q_RETURN_ARG(QVariant, etatFeuilleteur),
                                Q_ARG(QVariant, true));
      if (etatFeuilleteur.toString() != QStringLiteral("ok")) {
        std::fprintf(stderr, "smoke: le feuilleteur reste ouvert apres Nouveau : %s\n",
                     etatFeuilleteur.toString().toUtf8().constData());
        QCoreApplication::exit(11);
        return;
      }
      // Import bureautique (étape 6 de la feuille de route du 12/11) : un
      // .docx minimal, fabriqué sur-le-champ en ZIP stocké, s'ouvre converti
      // en Markdown — lecture seule, l'original ne saurait s'écrire dessus.
      QTemporaryFile docx(QDir::tempPath() + QStringLiteral("/mmdedit-XXXXXX.docx"));
      docx.open();
      const QByteArray xmlDocx = QByteArrayLiteral(R"(<?xml version="1.0" encoding="UTF-8"?>)"
                                                   R"(<w:document xmlns:w="ex"><w:body>)"
                                                   R"(<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr>)"
                                                   R"(<w:r><w:t>Titre importé</w:t></w:r></w:p>)"
                                                   R"(<w:p><w:r><w:t>Paragraphe importé.</w:t></w:r></w:p>)"
                                                   R"(</w:body></w:document>)");
      docx.write(zipUnMembre(QByteArrayLiteral("word/document.xml"), xmlDocx));
      docx.close();
      const QString cheminDocx = docx.fileName();
      QMetaObject::invokeMethod(racine, "charger",
                                Q_ARG(QVariant, QUrl::fromLocalFile(cheminDocx)));
      for (int i = 0; i < 20; ++i) {
        QThread::msleep(10);
        QCoreApplication::processEvents();
      }
      QVariant texteImporte;
      QMetaObject::invokeMethod(racine, "texteCourant",
                                Q_RETURN_ARG(QVariant, texteImporte));
      const QString attenduImport = QStringLiteral("# Titre importé\n\nParagraphe importé.");
      std::printf("smoke: import bureautique attendu=%d\n",
                  texteImporte.toString() == attenduImport ? 1 : 0);
      std::fflush(stdout);
      QVariant etatBureautique;
      QMetaObject::invokeMethod(racine, "controleBureautique",
                                Q_RETURN_ARG(QVariant, etatBureautique),
                                Q_ARG(QVariant, false));
      if (texteImporte.toString() != attenduImport
          || etatBureautique.toString() != QStringLiteral("ok")) {
        std::fprintf(stderr, "smoke: l'import bureautique a echoue : %s | %s\n",
                     texteImporte.toString().toUtf8().constData(),
                     etatBureautique.toString().toUtf8().constData());
        QCoreApplication::exit(12);
        return;
      }
      // « Nouveau » rend l'édition ordinaire : plus d'import en cours.
      QMetaObject::invokeMethod(racine, "nouveau");
      for (int i = 0; i < 10; ++i) {
        QThread::msleep(10);
        QCoreApplication::processEvents();
      }
      QMetaObject::invokeMethod(racine, "controleBureautique",
                                Q_RETURN_ARG(QVariant, etatBureautique),
                                Q_ARG(QVariant, true));
      if (etatBureautique.toString() != QStringLiteral("ok")) {
        std::fprintf(stderr, "smoke: l'import bureautique survit a Nouveau : %s\n",
                     etatBureautique.toString().toUtf8().constData());
        QCoreApplication::exit(13);
        return;
      }
      QCoreApplication::exit(ecrit && entete ? 0 : 8);
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
#if QT_CONFIG(sessionmanager)
  // Arrêt ou déconnexion : pas d'installation lancée maintenant (cf.
  // cpp/installeur.h).
  QObject::connect(&app, &QGuiApplication::commitDataRequest, &app,
                   [](QSessionManager&) { Installeur::sessionFinie(); });
#endif
  const int code = app.exec();
  // Mise à jour retenue pour la fermeture : son installeur part maintenant, et
  // attend la fin de ce processus pour remplacer les fichiers.
  Installeur::executer();
  return code;
}
