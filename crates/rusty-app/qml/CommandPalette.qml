import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// Ctrl+P: every command with its key, filtered as you type; Enter runs the chosen one.
Popup {
    id: pop
    required property var theme
    property var commands: []
    property var matches: []
    property int selected: 0

    modal: true
    focus: true
    dim: false
    padding: 0
    width: Math.min(620, parent.width - 80)
    x: Math.round((parent.width - width) / 2)
    y: 72
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    background: Rectangle { color: pop.theme.panel; border.color: pop.theme.accent; border.width: 1; radius: pop.theme.radius }

    function show() { field.text = ""; refilter(); open(); field.forceActiveFocus() }
    onCommandsChanged: refilter()
    function refilter() {
        const q = field.text.trim().toLowerCase()
        const enabled = commands.filter(function (c) { return c.enabled === undefined || c.enabled })
        if (q.length === 0) { matches = enabled; selected = 0; return }
        const words = q.split(/\s+/)
        matches = enabled.filter(function (c) {
            const name = c.name.toLowerCase()
            return words.every(function (w) { return name.indexOf(w) >= 0 })
        })
        selected = 0
    }
    function run() {
        if (matches.length === 0) return
        const c = matches[selected]
        pop.close()
        c.run()
    }

    contentItem: ColumnLayout {
        spacing: 0
        TextField {
            id: field
            Layout.fillWidth: true
            Layout.margins: 8
            placeholderText: "Select a command…"
            font.pixelSize: 15
            onTextChanged: pop.refilter()
            onAccepted: pop.run()
            Keys.onDownPressed: if (pop.matches.length > 0) pop.selected = Math.min(pop.selected + 1, pop.matches.length - 1)
            Keys.onUpPressed: if (pop.matches.length > 0) pop.selected = Math.max(pop.selected - 1, 0)
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: pop.theme.line; opacity: 0.6 }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(contentHeight, 440)
            clip: true
            model: pop.matches
            currentIndex: pop.selected
            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
            delegate: Rectangle {
                required property int index
                required property var modelData
                width: list.width
                height: 36
                color: pop.selected === index ? pop.theme.active : "transparent"
                RowLayout {
                    anchors.fill: parent; anchors.leftMargin: 14; anchors.rightMargin: 14
                    Text { text: modelData.name; color: pop.theme.foreground; font.pixelSize: 14; elide: Text.ElideRight; Layout.fillWidth: true }
                    Text { text: modelData.keys || ""; color: pop.theme.faint; font.pixelSize: 12 }
                }
                HoverHandler { onHoveredChanged: if (hovered) pop.selected = index }
                TapHandler { onTapped: { pop.selected = index; pop.run() } }
            }
        }
        Text { visible: pop.matches.length === 0; text: "No matching command"; color: pop.theme.faint; font.pixelSize: 12; Layout.margins: 12 }
    }
}
