// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick
import QtQuick.Controls
import fr.mmedia.mmdedit

ApplicationWindow {
    id: fenetre
    width: 720
    height: 480
    visible: true
    title: "MMdedit"

    readonly property Socle socle: Socle {}
    readonly property string noyau: socle.noyau

    Label {
        anchors.centerIn: parent
        horizontalAlignment: Text.AlignHCenter
        text: fenetre.noyau + "\n" + fenetre.socle.plateforme()
    }
}
