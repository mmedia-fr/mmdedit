// SPDX-License-Identifier: GPL-3.0-or-later
// Boîte de lecture seule : sert au rappel des balises (Markdown) et à la
// fenêtre « À propos » (texte enrichi). Le contenu est sélectionnable — on
// vient souvent y copier une balise.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: racine

    property string contenu: ""
    property int formatTexte: TextEdit.MarkdownText

    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(700, parent ? parent.width - 40 : 700)
    height: Math.min(560, parent ? parent.height - 40 : 560)
    standardButtons: Dialog.Close

    ScrollView {
        id: cadre
        anchors.fill: parent
        clip: true
        // Largeur imposée ici plutôt que sur le TextArea : lier la largeur de
        // l'enfant à celle disponible boucle sur implicitWidth.
        contentWidth: availableWidth

        TextArea {
            readOnly: true
            selectByMouse: true
            textFormat: racine.formatTexte
            wrapMode: TextEdit.Wrap
            text: racine.contenu
            onLinkActivated: function (lien) { Qt.openUrlExternally(lien) }
        }
    }
}
