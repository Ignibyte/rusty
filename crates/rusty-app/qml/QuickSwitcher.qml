import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// Ctrl+O: find a page by a few letters of its title or path, or create one by name.
Popup {
    id: pop
    required property var theme
    property var pages: []
    property var matches: []
    property int selected: 0
    signal openPage(string slug)
    signal createPage(string name)

    modal: true
    focus: true
    dim: false
    padding: 0
    width: Math.min(620, parent.width - 80)
    x: Math.round((parent.width - width) / 2)
    y: 72
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    background: Rectangle { color: pop.theme.surface; border.color: pop.theme.line; border.width: 1; radius: 8 }

    function show() { field.text = ""; refilter(); open(); field.forceActiveFocus() }

    // Subsequence match with a bonus for word starts and runs; -1 when a letter is missing.
    function score(hay, q) {
        hay = hay.toLowerCase()
        let from = 0
        let s = 0
        for (const ch of q) {
            const idx = hay.indexOf(ch, from)
            if (idx < 0) return -1
            if (idx === from && from > 0) s += 3
            if (idx === 0 || " /-_.".indexOf(hay[idx - 1]) >= 0) s += 2
            s += 1
            from = idx + 1
        }
        return s - hay.length / 200
    }
    function refilter() {
        const q = field.text.trim().toLowerCase()
        if (q.length === 0) { matches = pages.slice(0, 30); selected = 0; return }
        const scored = []
        for (const p of pages) {
            const a = score(p.title, q)
            const b = score(p.slug, q)
            const best = Math.max(a, b)
            if (best >= 0) scored.push({ page: p, score: best + (a >= 0 ? 1 : 0) })
        }
        scored.sort(function (x, y) { return y.score - x.score })
        matches = scored.slice(0, 50).map(function (s) { return s.page })
        selected = 0
    }
    function choose(create) {
        const name = field.text.trim()
        if (!create && matches.length > 0) pop.openPage(matches[selected].slug)
        else if (name.length > 0) pop.createPage(name)
        pop.close()
    }
    readonly property bool exact: matches.some(function (m) { return m.title.toLowerCase() === field.text.trim().toLowerCase() })

    contentItem: ColumnLayout {
        spacing: 0
        TextField {
            id: field
            Layout.fillWidth: true
            Layout.margins: 8
            placeholderText: "Find or create a note…"
            font.pixelSize: 15
            onTextChanged: pop.refilter()
            onAccepted: pop.choose(false)
            Keys.onDownPressed: if (pop.matches.length > 0) pop.selected = Math.min(pop.selected + 1, pop.matches.length - 1)
            Keys.onUpPressed: if (pop.matches.length > 0) pop.selected = Math.max(pop.selected - 1, 0)
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Return && (event.modifiers & Qt.ShiftModifier)) { pop.choose(true); event.accepted = true }
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: pop.theme.line; opacity: 0.6 }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(contentHeight, 420)
            clip: true
            model: pop.matches
            currentIndex: pop.selected
            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
            delegate: Rectangle {
                required property int index
                required property var modelData
                width: list.width
                height: 40
                color: pop.selected === index ? pop.theme.active : "transparent"
                RowLayout {
                    anchors.fill: parent; anchors.leftMargin: 14; anchors.rightMargin: 14
                    spacing: 10
                    Text { text: modelData.title; color: pop.theme.foreground; font.pixelSize: 14; elide: Text.ElideRight; Layout.fillWidth: true }
                    Text { text: modelData.slug; color: pop.theme.faint; font.pixelSize: 12; elide: Text.ElideMiddle; Layout.maximumWidth: 260 }
                }
                HoverHandler { onHoveredChanged: if (hovered) pop.selected = index }
                TapHandler { onTapped: { pop.selected = index; pop.choose(false) } }
            }
        }
        Rectangle {
            Layout.fillWidth: true
            height: 34
            color: "transparent"
            visible: field.text.trim().length > 0 && !pop.exact
            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: 14
                text: (pop.matches.length === 0 ? "Enter" : "Shift+Enter") + " to create \"" + field.text.trim() + "\""
                color: pop.theme.muted
                font.pixelSize: 12
            }
        }
        Text {
            visible: pop.matches.length === 0 && field.text.trim().length === 0
            text: "No pages yet"
            color: pop.theme.faint
            font.pixelSize: 12
            Layout.margins: 12
        }
    }
}
