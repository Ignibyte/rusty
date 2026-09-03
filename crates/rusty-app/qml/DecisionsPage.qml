import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Decisions view (TICKET-018): the follow-ups due first, then every decision with
// its status and dates; a click opens the page. One tool feeds it, brain_due.
Item {
    id: page
    required property var backend
    required property var theme
    signal openPage(string slug)

    property var due: []
    property var all: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("brain_due", { days: 0 }, "due") }
    function label(status) {
        return status === "decided" ? "decided" : status === "kept" ? "kept" : status === "revised" ? "revised" : status === "superseded" ? "superseded" : (status || "")
    }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            const r = JSON.parse(json)
            page.due = r.due || []
            page.all = r.all || []
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    component Row: Rectangle {
        id: row
        property var item: ({})
        Layout.fillWidth: true
        implicitHeight: Math.round(34 * page.theme.scale)
        radius: page.theme.radius
        color: rowHover.hovered ? page.theme.hover : "transparent"
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            spacing: 12
            Text { text: "◆"; color: page.theme.accent; font.pixelSize: Math.round(11 * page.theme.scale) }
            Text { text: row.item.title || row.item.slug || ""; color: page.theme.bright; font.pixelSize: Math.round(13 * page.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
            Text { text: page.label(row.item.status); color: row.item.status === "superseded" ? page.theme.faint : row.item.status === "decided" ? page.theme.alive : page.theme.muted; font.pixelSize: Math.round(11 * page.theme.scale) }
            Text { text: row.item.decided ? "decided " + row.item.decided : ""; color: page.theme.faint; font.pixelSize: Math.round(11 * page.theme.scale) }
            Text { text: row.item.follow_up_by ? "follow up by " + row.item.follow_up_by : ""; color: row.item.overdue ? page.theme.gold : page.theme.muted; font.pixelSize: Math.round(11 * page.theme.scale) }
        }
        HoverHandler { id: rowHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: page.openPage(row.item.slug) }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 10
        Text { text: "Decisions"; color: page.theme.foreground; font.pixelSize: Math.round(22 * page.theme.scale); font.bold: true }
        Text {
            text: "Ask, Decide, Follow up. An agent consults the brain (brain_ask), records what it chose and why as a page under decisions/ linked to what it read (brain_decide), and comes back to say how it went (brain_follow_up). The follow-ups due are listed first."
            color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(13 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true
        }
        Text { visible: page.due.length > 0; text: "Due"; color: page.theme.gold; font.pixelSize: Math.round(11 * page.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.topMargin: 6 }
        Repeater { model: page.due; delegate: Row { required property var modelData; item: modelData } }
        Text { text: page.all.length + (page.all.length === 1 ? " decision" : " decisions"); color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(11 * page.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.topMargin: 6 }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: page.all
            spacing: 2
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Row { required property var modelData; item: modelData; width: list.width }
            Text { anchors.centerIn: parent; visible: page.all.length === 0; text: page.backend.connected ? "No decisions yet. brain_ask, then brain_decide, writes the first one." : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(13 * page.theme.scale) }
        }
        Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
    }
}
