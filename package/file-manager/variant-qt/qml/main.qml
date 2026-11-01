// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Minimal placeholder UI proving the cxx-qt pipeline and a native Wayland window.
// The module URI and version must match QmlModule::new(...) in build.rs.

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Window 2.15

import dev.monochromatic.file_manager 1.0

ApplicationWindow {
    id: root
    width: 900
    height: 600
    visible: true
    title: qsTr("Monochromatic File Manager (Qt)")
    color: palette.window

    // The Rust-backed QObject registered by cxx-qt.
    AppBridge {
        id: bridge
    }

    Column {
        anchors.centerIn: parent
        spacing: 16

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: bridge.greeting
            color: palette.text
            font.pixelSize: 24
        }

        Button {
            anchors.horizontalCenter: parent.horizontalCenter
            text: qsTr("Log to Rust")
            onClicked: bridge.logFromQml("hello from QML")
        }
    }
}
