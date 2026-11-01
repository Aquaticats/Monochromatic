// 2 levels, but the inner ListView is wrapped in an async Loader. Isolates whether the
// async Loader (not the 3rd level) is the crash trigger.
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
            Loader {
                anchors.fill: parent
                asynchronous: false
                sourceComponent: Component {
                    ListView {
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
