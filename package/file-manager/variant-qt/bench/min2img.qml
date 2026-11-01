// The recommended app structure: 2-level, mixed column types (directory list vs preview)
// selected by visibility, NO Loader. This is what the file manager would actually use.
import QtQuick
import QtQuick.Window

Window {
    id: root
    visible: true
    width: 900
    height: 600
    property real p: 0
    ListView {
        anchors.fill: parent
        orientation: ListView.Horizontal
        model: 300
        reuseItems: true
        cacheBuffer: 600
        contentX: root.p * Math.max(0, contentWidth - width)
        delegate: Item {
            id: col
            required property int index
            width: 200
            height: ListView.view.height
            readonly property bool isDir: (col.index % 2) === 0
            // Directory column: a real nested list (level 2). Preview column: an Image, no list.
            ListView {
                anchors.fill: parent
                visible: col.isDir
                enabled: col.isDir
                model: col.isDir ? 100000 : 0
                reuseItems: true
                cacheBuffer: 720
                contentY: root.p * Math.max(0, contentHeight - height)
                delegate: Item {
                    required property int index
                    width: ListView.view.width
                    height: 18
                    Text { text: "r" + parent.index; font.pixelSize: 10 }
                }
            }
            Rectangle {
                anchors.fill: parent
                visible: !col.isDir
                color: "#223344"
                Text { anchors.centerIn: parent; text: "preview " + col.index; color: "white" }
            }
        }
    }
    NumberAnimation on p { from: 0; to: 1; duration: 2000; loops: 3; running: true }
    Timer { interval: 6500; running: true; onTriggered: Qt.quit() }
}
