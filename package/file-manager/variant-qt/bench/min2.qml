// Minimal level-2 repro: a horizontal ListView of columns, each column a vertical
// ListView (reuseItems on both). This is the shape of a normal column view, so if this
// crashes on teardown the app cannot dodge the bug by using only two levels.
import QtQuick
import QtQuick.Window

Window {
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
        contentX: parent.p * Math.max(0, contentWidth - width)
        delegate: Item {
            required property int index
            width: 200
            height: ListView.view.height
            ListView {
                anchors.fill: parent
                model: 100000
                reuseItems: true
                cacheBuffer: 720
                contentY: parent.parent.p * Math.max(0, contentHeight - height)
                delegate: Item {
                    required property int index
                    width: ListView.view.width
                    height: 18
                    Text { text: "r" + parent.index; font.pixelSize: 10 }
                }
            }
        }
    }
    NumberAnimation on p { from: 0; to: 1; duration: 2000; loops: 3; running: true }
    Timer { interval: 6500; running: true; onTriggered: Qt.quit() }
}
