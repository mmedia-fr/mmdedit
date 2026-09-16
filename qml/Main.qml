// SPDX-License-Identifier: GPL-3.0-or-later
// Fenêtre principale de MMdedit — éditeur à gauche, aperçu rendu à droite.
//
// Écrit pour Qt 6.4 : c'est la version de la machine de développement (Debian 12),
// et ce qui s'y compile se compile aussi sur le Qt 6.8 de l'intégration continue.
// D'où « Qt.labs.settings » plutôt que le module QtCore des versions récentes.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Qt.labs.settings
import fr.mmedia.mmdedit

ApplicationWindow {
    id: fenetre

    width: 1000
    height: 640
    visible: true
    title: (modifie ? "*" : "") + doc.nom + " — MMdedit"

    // Le document a-t-il changé depuis sa dernière écriture ? QML en est seul
    // juge : c'est le TextArea qui édite le texte, le noyau ne le détient pas.
    property bool modifie: false
    // Ce qu'il faut faire une fois la question « enregistrer ? » tranchée.
    property string apresEnregistrement: ""
    // Vrai pendant qu'on remplace le texte de l'éditeur par programme : sans ce
    // drapeau, le chargement d'un fichier marquerait aussitôt le document modifié.
    property bool chargementEnCours: false

    // Objet témoin de la liaison Rust : le test de fumée lit cette propriété.
    Socle { id: socle }
    readonly property string noyau: socle.noyau

    Document { id: doc }

    Settings {
        category: "fenetre"
        property alias x: fenetre.x
        property alias y: fenetre.y
        property alias largeur: fenetre.width
        property alias hauteur: fenetre.height
    }

    // ----------------------------------------------------------------- actions
    Action {
        id: actNouveau
        text: qsTr("&Nouveau")
        shortcut: StandardKey.New
        onTriggered: fenetre.avecEnregistrement("nouveau")
    }
    Action {
        id: actOuvrir
        text: qsTr("&Ouvrir…")
        shortcut: StandardKey.Open
        onTriggered: fenetre.avecEnregistrement("ouvrir")
    }
    Action {
        id: actEnregistrer
        text: qsTr("&Enregistrer")
        shortcut: StandardKey.Save
        onTriggered: fenetre.enregistrer()
    }
    Action {
        id: actEnregistrerSous
        text: qsTr("Enregistrer &sous…")
        shortcut: StandardKey.SaveAs
        onTriggered: fenetre.enregistrerSous()
    }
    Action {
        id: actQuitter
        text: qsTr("&Quitter")
        shortcut: StandardKey.Quit
        onTriggered: fenetre.close()
    }
    Action {
        id: actAnnuler
        text: qsTr("&Annuler")
        shortcut: StandardKey.Undo
        enabled: editeur.canUndo
        onTriggered: editeur.undo()
    }
    Action {
        id: actRetablir
        text: qsTr("&Rétablir")
        shortcut: StandardKey.Redo
        enabled: editeur.canRedo
        onTriggered: editeur.redo()
    }
    Action {
        id: actCouper
        text: qsTr("Co&uper")
        shortcut: StandardKey.Cut
        onTriggered: editeur.cut()
    }
    Action {
        id: actCopier
        text: qsTr("&Copier")
        shortcut: StandardKey.Copy
        // Copier depuis l'aperçu doit marcher aussi : sans cela, un Ctrl+C fait
        // dans le rendu n'écrivait rien et le presse-papier gardait l'ancien contenu.
        onTriggered: apercu.selectedText.length > 0 ? apercu.copy() : editeur.copy()
    }
    Action {
        id: actColler
        text: qsTr("Co&ller")
        shortcut: StandardKey.Paste
        onTriggered: editeur.paste()
    }
    Action {
        id: actVoirEditeur
        text: qsTr("Afficher l'&éditeur")
        checkable: true
        checked: true
        onTriggered: fenetre.basculerVue(actVoirEditeur, actVoirApercu)
    }
    Action {
        id: actVoirApercu
        text: qsTr("Afficher l'&aperçu")
        checkable: true
        checked: true
        onTriggered: fenetre.basculerVue(actVoirApercu, actVoirEditeur)
    }

    menuBar: MenuBar {
        Menu {
            title: qsTr("&Fichier")
            MenuItem { action: actNouveau }
            MenuItem { action: actOuvrir }
            MenuSeparator {}
            MenuItem { action: actEnregistrer }
            MenuItem { action: actEnregistrerSous }
            MenuSeparator {}
            MenuItem { action: actQuitter }
        }
        Menu {
            title: qsTr("&Édition")
            MenuItem { action: actAnnuler }
            MenuItem { action: actRetablir }
            MenuSeparator {}
            MenuItem { action: actCouper }
            MenuItem { action: actCopier }
            MenuItem { action: actColler }
        }
        Menu {
            title: qsTr("&Affichage")
            MenuItem { action: actVoirEditeur }
            MenuItem { action: actVoirApercu }
        }
    }

    // ------------------------------------------------------------------- vues
    SplitView {
        id: partage
        anchors.fill: parent
        orientation: Qt.Horizontal

        ScrollView {
            id: volet_editeur
            SplitView.preferredWidth: parent.width / 2
            SplitView.fillWidth: !volet_apercu.visible
            visible: actVoirEditeur.checked

            TextArea {
                id: editeur
                textFormat: TextEdit.PlainText
                wrapMode: TextEdit.NoWrap
                selectByMouse: true
                persistentSelection: true
                font.family: "monospace"
                font.pointSize: 11
                onTextChanged: {
                    if (!fenetre.chargementEnCours)
                        fenetre.modifie = true
                    minuteurApercu.restart()
                }
            }
        }

        ScrollView {
            id: volet_apercu
            SplitView.preferredWidth: parent.width / 2
            SplitView.fillWidth: !volet_editeur.visible
            visible: actVoirApercu.checked && doc.estMarkdown()

            TextArea {
                id: apercu
                readOnly: true
                selectByMouse: true
                persistentSelection: true
                textFormat: TextEdit.MarkdownText
                wrapMode: TextEdit.Wrap
                // Sans largeur imposée, le document rendu prend sa largeur naturelle
                // et les tableaux Markdown sortent écrasés sur quelques pixels.
                width: volet_apercu.availableWidth
                // La largeur du document n'est lue qu'au chargement du texte : un
                // redimensionnement doit donc refaire le rendu, pas seulement la
                // mise à l'échelle (cf. rafraichir()).
                onWidthChanged: minuteurApercu.restart()
                // Le rendu suit le texte avec un temps de retard (minuteurApercu) :
                // le recalculer à chaque frappe fait ramer dès quelques pages.
            }
        }
    }

    // Anti-rebond du rendu, comme la version PySide6 : 200 ms après la frappe.
    Timer {
        id: minuteurApercu
        interval: 200
        repeat: false
        onTriggered: fenetre.rafraichir()
    }

    // ------------------------------------------------------------- barre d'état
    footer: ToolBar {
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 16

            Label {
                id: messageEtat
                text: ""
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
            Label { text: doc.libelle }
            Label { text: doc.encodage }
            Label { id: compteurs; text: doc.statistiques("") }
        }
    }

    // Message passager en barre d'état (« Enregistré. »).
    Timer {
        id: minuteurMessage
        interval: 3000
        onTriggered: messageEtat.text = ""
    }

    // ------------------------------------------------------------- glisser-déposer
    DropArea {
        anchors.fill: parent
        onDropped: function (depot) {
            if (depot.hasUrls && depot.urls.length > 0) {
                fenetre.urlAOuvrir = depot.urls[0]
                fenetre.avecEnregistrement("deposer")
            }
        }
    }
    property url urlAOuvrir: ""

    // ------------------------------------------------------------------ dialogues
    FileDialog {
        id: dlgOuvrir
        title: qsTr("Ouvrir un fichier")
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTr("Markdown (*.md *.markdown)"), qsTr("Texte (*.txt)"),
                      qsTr("CSV (*.csv)"), qsTr("Tous les fichiers (*)")]
        onAccepted: fenetre.charger(selectedFile)
    }

    FileDialog {
        id: dlgEnregistrer
        title: qsTr("Enregistrer sous")
        fileMode: FileDialog.SaveFile
        defaultSuffix: "md"
        nameFilters: [qsTr("Markdown (*.md *.markdown)"), qsTr("Texte (*.txt)"),
                      qsTr("CSV (*.csv)"), qsTr("Tous les fichiers (*)")]
        onAccepted: {
            if (doc.enregistrer(selectedFile, editeur.text)) {
                fenetre.modifie = false
                fenetre.flash(qsTr("Enregistré."))
                fenetre.poursuivre()
            }
        }
    }

    MessageDialog {
        id: dlgModifie
        title: "MMdedit"
        text: qsTr("Le document « %1 » a été modifié.").arg(doc.nom)
        informativeText: qsTr("Voulez-vous enregistrer les modifications ?")
        buttons: MessageDialog.Save | MessageDialog.Discard | MessageDialog.Cancel
        onButtonClicked: function (bouton) {
            if (bouton === MessageDialog.Save) {
                // poursuivre() est appelé par l'écriture, une fois qu'elle a réussi.
                fenetre.enregistrer()
            } else if (bouton === MessageDialog.Discard) {
                fenetre.modifie = false
                fenetre.poursuivre()
            } else {
                fenetre.apresEnregistrement = ""
            }
        }
    }

    MessageDialog {
        id: dlgErreur
        title: "MMdedit"
        text: doc.erreur
        buttons: MessageDialog.Ok
        onButtonClicked: doc.oublierErreur()
    }

    // Une erreur du noyau se voit aussitôt, d'où qu'elle vienne.
    Connections {
        target: doc
        function onErreurChanged() {
            if (doc.erreur.length > 0)
                dlgErreur.open()
        }
    }

    // -------------------------------------------------------------------- logique
    function flash(message) {
        messageEtat.text = message
        minuteurMessage.restart()
    }

    // Largeur de l'aperçu lors du dernier rendu : sert à savoir s'il faut forcer
    // une nouvelle mise en page.
    property real largeurRendue: 0

    function rafraichir() {
        var rendu = doc.estMarkdown() ? editeur.text : ""
        // Qt ne remet en page qu'au (re)chargement du texte : à texte inchangé et
        // largeur nouvelle, il faut le vider d'abord, sans quoi un tableau garde
        // les largeurs de colonnes calculées pour l'ancienne fenêtre.
        if (rendu === apercu.text && apercu.width !== largeurRendue)
            apercu.text = ""
        apercu.text = rendu
        largeurRendue = apercu.width
        compteurs.text = doc.statistiques(editeur.text)
    }

    // Au moins une des deux vues reste visible.
    function basculerVue(bascule, autre) {
        if (!bascule.checked && !autre.checked)
            bascule.checked = true
    }

    // Applique au document ouvert ce que son format impose : pas d'aperçu rendu
    // hors Markdown, et l'éditeur reprend alors toute la fenêtre.
    function appliquerFormat() {
        actVoirApercu.checked = doc.estMarkdown()
        if (!actVoirApercu.checked)
            actVoirEditeur.checked = true
    }

    function charger(url) {
        chargementEnCours = true
        var contenu = doc.ouvrir(url)
        if (doc.erreur.length === 0) {
            editeur.text = contenu
            modifie = false
            appliquerFormat()
        }
        chargementEnCours = false
        rafraichir()
    }

    function nouveau() {
        chargementEnCours = true
        doc.nouveau()
        editeur.text = ""
        modifie = false
        appliquerFormat()
        chargementEnCours = false
        rafraichir()
    }

    function enregistrer() {
        if (doc.chemin.length === 0) {
            enregistrerSous()
            return
        }
        if (doc.enregistrer(doc.chemin, editeur.text)) {
            modifie = false
            flash(qsTr("Enregistré."))
            poursuivre()
        }
    }

    function enregistrerSous() {
        if (doc.chemin.length > 0)
            dlgEnregistrer.currentFile = "file://" + doc.chemin
        dlgEnregistrer.open()
    }

    // Demande s'il faut enregistrer, puis exécute l'action demandée.
    function avecEnregistrement(action) {
        apresEnregistrement = action
        if (modifie)
            dlgModifie.open()
        else
            poursuivre()
    }

    function poursuivre() {
        var action = apresEnregistrement
        apresEnregistrement = ""
        if (action === "nouveau")
            nouveau()
        else if (action === "ouvrir")
            dlgOuvrir.open()
        else if (action === "deposer")
            charger(urlAOuvrir)
        else if (action === "quitter")
            Qt.quit()
    }

    onClosing: function (fermeture) {
        if (modifie) {
            fermeture.accepted = false
            avecEnregistrement("quitter")
        }
    }

    // Contrôle de fabrication : le test de fumée appelle cette fonction pour
    // vérifier que le noyau Rust répond, et pas seulement qu'il s'instancie.
    function statistiquesDeControle() {
        return doc.statistiques("deux mots")
    }

    // Idem : ce que l'éditeur porte réellement, pour vérifier qu'un fichier donné
    // en argument est bien arrivé jusqu'à lui.
    function texteCourant() {
        return editeur.text
    }

    Component.onCompleted: {
        // « MMdedit fichier.md », et le « Ouvrir avec » de Windows qui s'y ramène.
        if (typeof fichierInitial !== "undefined" && fichierInitial.toString().length > 0)
            charger(fichierInitial)
        else
            rafraichir()
    }
}
