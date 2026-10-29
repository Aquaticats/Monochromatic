// 3 levels (columns -> panes -> rows) but NO Loaders anywhere. Isolates whether the
// third level of nested delegate models alone triggers the crash.
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
            required property int index
            width: 200
            height: ListView.view.height
            ListView {
                anchors.fill: parent
                model: 20
                reuseItems: true
                cacheBuffer: 400
                contentY: root.p * Math.max(0, contentHeight - height)
                delegate: Item {
                    required property int index
                    width: 200
                    height: 150
                    ListView {
                        anchors.fill: parent
                        model: 100000
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
                }
            }
        }
    }
    NumberAnimation on p { from: 0; to: 1; duration: 2000; loops: 3; running: true }
    Timer { interval: 6500; running: true; onTriggered: Qt.quit() }
}
