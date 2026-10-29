// Qt strip benchmark, OPTIMIZED for fast scroll, mirroring the Slint spike scale:
//   1200 columns, ~14400 panes, up to 100000 rows/pane, 384x256 previews.
// Fast full-range DIAGONAL scroll (columns horizontally, panes+rows vertically), the
// way a column-strip user actually flings. Optimizations over the naive version:
//   - reuseItems:true on every ListView  -> recycle delegates instead of create/destroy
//   - cacheBuffer > 0                     -> prefetch a screen of delegates
//   - Loader.asynchronous:true           -> incubate pane content off the frame's hot path
// Clean FPS via frameSwapped, logged once/second (no per-frame I/O).
import QtQuick
import QtQuick.Window

Window {
    id: root
    visible: true
    width: 1600
    height: 1000
    title: "Qt strip: fast diagonal scroll (optimized) + previews"
    color: "#0f1216"

    property int aliveColumns: 0
    property int alivePanes: 0
    property int aliveRows: 0
    property int alivePreviews: 0
    property int maxAlivePanes: 0
    property int maxAliveRows: 0
    property int maxAlivePreviews: 0
    property real panesAddressable: 0
    property real rowsAddressable: 0
    property real hphase: 0
    property real vphase: 0
    property int frameCount: 0
    onFrameSwapped: root.frameCount++

    readonly property int columnCount: 1200
    readonly property int colWidth: 260
    readonly property int paneHeight: 300
    readonly property int rowHeight: 18
    readonly property int poolSize: 256

    function paneCount(col) { return (col % 3 === 0) ? 26 : 5; }
    function isPreview(col, pane) { return ((col + pane) % 2) === 0; }
    function rowCount(col, pane) {
        return (((col * 31 + pane * 17) % 7) === 0)
            ? 100000
            : (50 + ((col * 13 + pane * 7) % 4950));
    }
    function previewSource(col, pane) {
        var k = 1 + (((col * 7 + pane * 13) % root.poolSize + root.poolSize) % root.poolSize);
        return "imgs/img_" + ("00" + k).slice(-3) + ".png";
    }

    Component.onCompleted: {
        var panes = 0, rows = 0;
        for (var c = 0; c < columnCount; ++c) {
            var pc = paneCount(c);
            panes += pc;
            for (var p = 0; p < pc; ++p) if (!isPreview(c, p)) rows += rowCount(c, p);
        }
        panesAddressable = panes;
        rowsAddressable = rows;
        console.log("[bench] addressable columns=" + columnCount
            + " panes=" + panes + " directory-rows=" + rows);
    }

    ListView {
        id: strip
        anchors.fill: parent
        orientation: ListView.Horizontal
        model: root.columnCount
        reuseItems: true
        cacheBuffer: root.colWidth * 3
        interactive: false
        // Fast full-range horizontal fling across all 1200 columns.
        contentX: root.hphase * Math.max(0, contentWidth - width)

        delegate: Item {
            id: colItem
            required property int index
            width: root.colWidth
            height: strip.height
            Component.onCompleted: root.aliveColumns++
            Component.onDestruction: root.aliveColumns--

            ListView {
                anchors.fill: parent
                clip: true
                model: root.paneCount(colItem.index)
                reuseItems: true
                cacheBuffer: root.paneHeight * 2
                interactive: false
                contentY: root.vphase * Math.max(0, contentHeight - height)

                delegate: Item {
                    id: paneItem
                    required property int index
                    width: root.colWidth
                    height: root.paneHeight
                    readonly property bool preview: root.isPreview(colItem.index, paneItem.index)
                    Component.onCompleted: {
                        root.alivePanes++;
                        if (root.alivePanes > root.maxAlivePanes) root.maxAlivePanes = root.alivePanes;
                    }
                    Component.onDestruction: root.alivePanes--

                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 2
                        color: "#1b2330"
                        border.color: "#2d3a4f"

                        Text {
                            id: paneHeader
                            width: parent.width
                            height: 16
                            leftPadding: 4
                            text: (paneItem.preview ? "preview  " : "dir  ")
                                + "c" + colItem.index + " p" + paneItem.index
                            color: paneItem.preview ? "#ffd08f" : "#8fb3ff"
                            font.pixelSize: 10
                            elide: Text.ElideRight
                        }

                        // Async incubation: pane content builds off the frame hot path.
                        Loader {
                            active: paneItem.preview
                            asynchronous: true
                            anchors.top: paneHeader.bottom
                            width: parent.width
                            height: parent.height - paneHeader.height
                            onLoaded: { root.alivePreviews++; if (root.alivePreviews > root.maxAlivePreviews) root.maxAlivePreviews = root.alivePreviews; }
                            Component.onDestruction: if (status === Loader.Ready) root.alivePreviews--
                            sourceComponent: Component {
                                Item {
                                    Rectangle { anchors.fill: parent; color: "#20303f" }
                                    Image {
                                        anchors.fill: parent
                                        source: root.previewSource(colItem.index, paneItem.index)
                                        sourceSize.width: 384
                                        sourceSize.height: 256
                                        asynchronous: true
                                        cache: false
                                        fillMode: Image.PreserveAspectCrop
                                    }
                                }
                            }
                        }

                        Loader {
                            active: !paneItem.preview
                            asynchronous: true
                            anchors.top: paneHeader.bottom
                            width: parent.width
                            height: parent.height - paneHeader.height
                            sourceComponent: Component {
                                ListView {
                                    clip: true
                                    model: root.rowCount(colItem.index, paneItem.index)
                                    reuseItems: true
                                    cacheBuffer: root.rowHeight * 40
                                    interactive: false
                                    contentY: root.vphase * Math.max(0, contentHeight - height)
                                    delegate: Item {
                                        id: rowItem
                                        required property int index
                                        width: ListView.view.width
                                        height: root.rowHeight
                                        Component.onCompleted: {
                                            root.aliveRows++;
                                            if (root.aliveRows > root.maxAliveRows) root.maxAliveRows = root.aliveRows;
                                        }
                                        Component.onDestruction: root.aliveRows--
                                        Text {
                                            anchors.verticalCenter: parent.verticalCenter
                                            leftPadding: 6
                                            text: "row " + rowItem.index
                                            color: "#cdd6e4"
                                            font.pixelSize: 10
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    SequentialAnimation on hphase {
        loops: Animation.Infinite
        running: true
        NumberAnimation { from: 0; to: 1; duration: 12000; easing.type: Easing.InOutSine }
        NumberAnimation { from: 1; to: 0; duration: 12000; easing.type: Easing.InOutSine }
    }
    SequentialAnimation on vphase {
        loops: Animation.Infinite
        running: true
        NumberAnimation { from: 0; to: 1; duration: 4500; easing.type: Easing.InOutSine }
        NumberAnimation { from: 1; to: 0; duration: 4500; easing.type: Easing.InOutSine }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.margins: 8
        width: hud.implicitWidth + 16
        height: hud.implicitHeight + 12
        color: "#000000dd"
        border.color: "#3d5a80"
        Text {
            id: hud
            anchors.centerIn: parent
            color: "#e0f0ff"
            font.pixelSize: 12
            text: "fps " + root.frameCount + "   ALIVE panes " + root.alivePanes
                + " (max " + root.maxAlivePanes + ")   previews " + root.alivePreviews
                + "   rows " + root.aliveRows + " (max " + root.maxAliveRows + ")\n"
                + "ADDRESSABLE panes " + Math.round(root.panesAddressable)
                + "   directory-rows " + Math.round(root.rowsAddressable)
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            console.log("[bench] fps=" + root.frameCount
                + " alive panes=" + root.alivePanes + "(max " + root.maxAlivePanes + ")"
                + " previews=" + root.alivePreviews + "(max " + root.maxAlivePreviews + ")"
                + " rows=" + root.aliveRows + "(max " + root.maxAliveRows + ")");
            root.frameCount = 0;
        }
    }

    Timer {
        interval: 16000
        running: true
        repeat: false
        onTriggered: Qt.quit()
    }
}
