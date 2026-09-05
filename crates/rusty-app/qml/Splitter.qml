import QtQuick

// A vertical drag handle between two panes. The owner binds `value` and clamps it with
// `min` and `max`; the handle reports where the pointer wants the value and never writes
// it itself, so one component serves the sidebars and any page with a split.
//
// Drags are measured in scene coordinates. This item moves as the pane it sits on
// resizes, so a delta taken in its own frame would be measured from a moving origin and
// would only track while the pointer stayed inside it (BF-rusty-moving-frame-delta-001).
Item {
    id: sp
    required property var theme
    property real value: 0
    property real min: 0
    property real max: 100000
    // A pane on the right grows as the pointer moves left.
    property bool invert: false
    signal moved(real value)
    width: 7
    Rectangle { anchors.centerIn: parent; width: 1; height: parent.height; color: sp.theme.line }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.SplitHCursor
        property real startX: 0
        property real startValue: 0
        onPressed: (mouse) => { startX = sp.mapToItem(null, mouse.x, 0).x; startValue = sp.value }
        onPositionChanged: (mouse) => {
            if (!pressed) return
            const delta = sp.mapToItem(null, mouse.x, 0).x - startX
            const next = startValue + (sp.invert ? -delta : delta)
            sp.moved(Math.max(sp.min, Math.min(sp.max, next)))
        }
    }
}
