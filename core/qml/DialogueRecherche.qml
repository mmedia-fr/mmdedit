// SPDX-License-Identifier: GPL-3.0-or-later
// Rechercher / Remplacer — non modal, comme dans la version PySide6 : il reste
// ouvert pendant que l'on continue d'éditer.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import fr.mmedia.mmdedit

Dialog {
    id: racine

    // Le moteur de recherche (noyau Rust) et la zone de texte visée.
    property Edition moteur
    property var cible
    // Message à afficher en barre d'état (nombre de remplacements, échec).
    signal information(string texte)

    title: qsTr("Rechercher / Remplacer")
    modal: false
    closePolicy: Popup.CloseOnEscape
    standardButtons: Dialog.Close

    function ouvrir() {
        open()
        champRecherche.forceActiveFocus()
        champRecherche.selectAll()
    }

    // Cherche à partir du bord courant de la sélection, et signale l'échec.
    function chercher(enArriere) {
        if (champRecherche.text.length === 0 || !cible)
            return false
        var depuis = enArriere ? cible.selectionStart : cible.selectionEnd
        var trouve = moteur.chercher(cible.text, champRecherche.text, depuis,
                                     boiteCasse.checked, boiteMotEntier.checked, enArriere)
        if (trouve < 0) {
            information(qsTr("« %1 » introuvable.").arg(champRecherche.text))
            return false
        }
        cible.select(trouve, trouve + champRecherche.text.length)
        cible.forceActiveFocus()
        return true
    }

    // Remplace la sélection si elle correspond au motif, puis va à la suivante.
    function remplacer() {
        if (!cible || champRecherche.text.length === 0)
            return
        var selection = cible.selectedText
        var correspond = boiteCasse.checked
                ? selection === champRecherche.text
                : selection.toLowerCase() === champRecherche.text.toLowerCase()
        if (correspond) {
            var debut = cible.selectionStart
            cible.remove(debut, cible.selectionEnd)
            cible.insert(debut, champRemplacement.text)
            cible.select(debut, debut + champRemplacement.text.length)
        }
        chercher(false)
    }

    function remplacerTout() {
        if (!cible || champRecherche.text.length === 0)
            return
        var resultat = moteur.remplacerTout(cible.text, champRecherche.text,
                                            champRemplacement.text,
                                            boiteCasse.checked, boiteMotEntier.checked)
        if (resultat.compte > 0) {
            // Remplacer le contenu sans réassigner « text » : l'affectation
            // directe viderait la pile d'annulation.
            var position = cible.cursorPosition
            cible.remove(0, cible.length)
            cible.insert(0, resultat.texte)
            cible.cursorPosition = Math.min(position, cible.length)
        }
        information(qsTr("%1 remplacement(s) effectué(s).").arg(resultat.compte))
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        GridLayout {
            columns: 2
            columnSpacing: 8
            Layout.fillWidth: true

            Label { text: qsTr("Rechercher :") }
            TextField {
                id: champRecherche
                Layout.fillWidth: true
                Layout.minimumWidth: 240
                onAccepted: racine.chercher(false)
            }
            Label { text: qsTr("Remplacer par :") }
            TextField {
                id: champRemplacement
                Layout.fillWidth: true
                onAccepted: racine.remplacer()
            }
        }

        RowLayout {
            spacing: 16
            CheckBox { id: boiteCasse; text: qsTr("Respecter la casse") }
            CheckBox { id: boiteMotEntier; text: qsTr("Mot entier") }
        }

        RowLayout {
            spacing: 8
            Button { text: qsTr("Suivant"); onClicked: racine.chercher(false) }
            Button { text: qsTr("Précédent"); onClicked: racine.chercher(true) }
            Button { text: qsTr("Remplacer"); onClicked: racine.remplacer() }
            Button { text: qsTr("Remplacer tout"); onClicked: racine.remplacerTout() }
        }
    }
}
