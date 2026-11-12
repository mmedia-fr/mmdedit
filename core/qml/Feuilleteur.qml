// SPDX-License-Identifier: GPL-3.0-or-later
// Feuilleteur PDF, en lecture seule (étape 5 de la feuille de route du 12/11).
//
// Vit dans son propre fichier parce qu'Android n'a pas de module Qt PDF : il
// n'est pas déclaré dans Main.qml — « import QtQuick.Pdf » casserait le
// démarrage sous Android dès l'analyse — mais chargé par un Loader quand un
// PDF s'ouvre, bureau seulement (cf. « feuilleteurDisponible » dans Main.qml).
import QtQuick
import QtQuick.Pdf

PdfMultiPageView {
    id: feuilletteur

    // La vue ne crée pas son document : « document » est une propriété exigée
    // de PdfMultiPageView, et c'est par elle que la source ouverte remonte.
    document: interne

    /// Pages chargées du PDF courant. Un alias ne traverse qu'un seul
    /// identifiant : la cible se nommerait « racine.interne.pageCount » et
    /// se réduit à « interne.pageCount ».
    readonly property alias pages: interne.pageCount
    /// Cause d'un échec de chargement, en clair : « error » côté QtPdf est
    /// une énumération muette (nombre), pas un message.
    readonly property string erreur: {
        switch (interne.error) {
        case 1: return qsTr("erreur inconnue")
        case 2: return qsTr("données pas encore disponibles")
        case 3: return qsTr("fichier introuvable")
        case 4: return qsTr("format invalide")
        case 5: return qsTr("mot de passe incorrect")
        case 6: return qsTr("schéma de sécurité non pris en charge")
        }
        return ""
    }
    /// État du document : 0 nul, 1 chargement, 2 prêt, 3 déchargement,
    /// 4 erreur — l'ordre de QPdfDocument::Status, que Main.qml ne nomme
    /// pas sans l'import du module.
    readonly property int etat: interne.status

    /// Ouvre un PDF ; remplacer l'URL ouvre le nouveau fichier.
    function ouvrir(url) {
        interne.source = url
    }

    /// Referme : le document PDF est relâché, la vue se vide.
    function fermer() {
        interne.source = ""
    }

    PdfDocument {
        id: interne
    }
}
