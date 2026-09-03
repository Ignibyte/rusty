import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// Search across the vault (full text, and vectors when a provider is set), results
// with the matching snippet. Enter opens the chosen result; arrows move. The query
// takes Obsidian's operators (path:, file:, tag:, type:); the two chips beside the
// field switch match case and regular expressions, the third bookmarks the search.
Item {
    id: pane
    required property var backend
    required property var theme
    property var results: []
    property int hit: -1
    property string notice: ""
    property var pending: ({})
    property bool matchCase: false
    property bool useRegex: false
    readonly property string query: field.text
    signal openPage(string slug)
    signal bookmarkSearch(string query)

    component ModeChip: Rectangle {
        id: chip
        property string label: ""
        property string icon: ""
        property string tip: ""
        property bool active: false
        signal clicked()
        width: 26
        height: 26
        radius: 5
        color: chip.active ? pane.theme.active : (chipHover.hovered ? pane.theme.hover : "transparent")
        opacity: enabled ? 1 : 0.4
        Text { visible: chip.label.length > 0; anchors.centerIn: parent; text: chip.label; color: chip.active ? pane.theme.foreground : pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); font.family: pane.theme.termFont }
        Icon { visible: chip.icon.length > 0; anchors.centerIn: parent; name: chip.icon; color: pane.theme.muted; size: 14 }
        HoverHandler { id: chipHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: if (chip.enabled) chip.clicked() }
        ToolTip.visible: chipHover.hovered && chip.tip.length > 0
        ToolTip.text: chip.tip
        ToolTip.delay: 600
    }

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function search(q) {
        if (q.trim().length === 0) { results = []; hit = -1; return }
        ask("brain_search", { query: q.trim(), limit: 60, case_sensitive: matchCase, regex: useRegex }, "search")
    }
    function searchFor(q) { field.text = q; search(q); field.forceActiveFocus() }
    function focusEntry() { field.forceActiveFocus(); field.selectAll() }
    function openHit(i) { if (i >= 0 && i < results.length) pane.openPage(results[i].slug) }
    function styled(snippet) {
        return String(snippet).replace(/</g, "&lt;").replace(/&lt;b>/g, "<b>").replace(/&lt;\/b>/g, "</b>")
    }

    Timer { id: debounce; interval: 250; onTriggered: pane.search(field.text) }

    Connections {
        target: pane.backend
        function onResult(id, tool, json, ok) {
            const kind = pane.pending[id]
            if (kind === undefined) return
            const p = pane.pending; delete p[id]; pane.pending = p
            if (!ok) { pane.notice = tool + ": " + json; return }
            pane.notice = ""
            pane.results = JSON.parse(json)
            pane.hit = pane.results.length > 0 ? 0 : -1
        }
        function onDataChanged() { if (field.text.trim().length > 0) debounce.restart() }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            spacing: 4
            TextField {
                id: field
                Layout.fillWidth: true
                placeholderText: "Search…"
                font.pixelSize: Math.round(13 * pane.theme.scale)
                onTextChanged: debounce.restart()
                onAccepted: pane.openHit(pane.hit >= 0 ? pane.hit : 0)
                Keys.onEscapePressed: text = ""
                Keys.onDownPressed: if (pane.results.length > 0) pane.hit = Math.min(pane.hit + 1, pane.results.length - 1)
                Keys.onUpPressed: if (pane.results.length > 0) pane.hit = Math.max(pane.hit - 1, 0)
            }
            ModeChip { label: "Aa"; tip: "Match case"; active: pane.matchCase; onClicked: { pane.matchCase = !pane.matchCase; pane.search(field.text) } }
            ModeChip { label: ".*"; tip: "Regular expression"; active: pane.useRegex; onClicked: { pane.useRegex = !pane.useRegex; pane.search(field.text) } }
            ModeChip { icon: "bookmark"; tip: "Bookmark this search"; enabled: field.text.trim().length > 0; onClicked: pane.bookmarkSearch(field.text.trim()) }
        }
        Text {
            visible: field.text.trim().length === 0
            text: "Narrow with path: file: tag: type:  ·  quotes keep spaces  ·  a leading - excludes"
            color: pane.theme.faint
            font.pixelSize: Math.round(11 * pane.theme.scale)
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
        Text {
            visible: field.text.trim().length > 0
            text: pane.results.length === 0 ? "No matches" : pane.results.length + (pane.results.length === 1 ? " result" : " results")
            color: pane.theme.faint
            font.pixelSize: Math.round(11 * pane.theme.scale)
        }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: pane.results
            spacing: 2
            currentIndex: pane.hit
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                required property int index
                required property var modelData
                width: list.width
                height: col.implicitHeight + 12
                radius: 4
                color: pane.hit === index ? pane.theme.active : (hover.hovered ? pane.theme.hover : "transparent")
                ColumnLayout {
                    id: col
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 6
                    spacing: 2
                    Text { text: modelData.title; color: pane.theme.foreground; font.pixelSize: Math.round(13 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                    Text { text: modelData.slug; color: pane.theme.faint; font.pixelSize: Math.round(11 * pane.theme.scale); elide: Text.ElideMiddle; Layout.fillWidth: true }
                    Text { text: pane.styled(modelData.snippet); textFormat: Text.StyledText; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); wrapMode: Text.Wrap; maximumLineCount: 3; elide: Text.ElideRight; Layout.fillWidth: true }
                }
                HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: pane.openPage(modelData.slug) }
            }
        }
        Text { visible: pane.notice.length > 0; text: pane.notice; color: pane.theme.muted; font.pixelSize: Math.round(11 * pane.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true }
    }
}
