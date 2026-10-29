// Minimal level-1 repro: a single ListView with reuseItems + a large model, scrolled
// then torn down. Tests whether reuseItems teardown crashes with NO nesting.
import QtQuick
import QtQuick.Window

Window {
    visible: true
    width: 400
    height: 600
    property real p: 0
    ListView {
        anchors.fill: parent
        model: 100000
        reuseItems: true
        cacheBuffer: 720
        contentY: parent.p * Math.max(0, contentHeight - height)
        delegate: Item {
            required property int index
            width: ListView.view.width
            height: 18
            Text { text: "row " + parent.index; font.pixelSize: 10 }
        }
    }
    NumberAnimation on p { from: 0; to: 1; duration: 2000; loops: 3; running: true }
    Timer { interval: 6500; running: true; onTriggered: Qt.quit() }
}
