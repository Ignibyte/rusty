import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The right sidebar: backlinks with their context lines, outgoing links (unresolved
// ones create the page), the outline, and an agent terminal beside the note.
Item {
    id: pane
    required property var backend
    required property var theme
    required property var terminals
    property var note: null
    property var titles: ({})
    property string current: "backlinks"
    property bool windowActive: true
    property var programs: []
    property string program: ""
    signal openPage(string slug)
    signal createPage(string name)
    signal paneChanged(string name)

    function titleOf(slug) { return titles[slug] || slug.slice(slug.lastIndexOf("/") + 1) }
    function focusAgent() { if (current === "agent") agentTerm.focusTerminal() }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 4
            spacing: 2
            PaneTab { icon: "link"; name: "backlinks"; tip: "Backlinks" }
            PaneTab { icon: "outgoing"; name: "outgoing"; tip: "Outgoing links" }
            PaneTab { icon: "outline"; name: "outline"; tip: "Outline" }
            PaneTab { icon: "agent"; name: "agent"; tip: "Agent beside the note" }
            Item { Layout.fillWidth: true }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: pane.theme.line; opacity: 0.6; Layout.topMargin: 4 }

        // Backlinks
        ColumnLayout {
            visible: pane.current === "backlinks"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 4
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: 12; elide: Text.ElideRight; Layout.fillWidth: true }
            RowLayout {
                visible: pane.note !== null
                Text { text: "Linked mentions"; color: pane.theme.foreground; font.pixelSize: 13 }
                Rectangle { radius: 8; color: pane.theme.hover; width: countText.implicitWidth + 12; height: 18; Text { id: countText; anchors.centerIn: parent; text: pane.note ? pane.note.backlinkCount : 0; color: pane.theme.muted; font.pixelSize: 11 } }
            }
            ListView {
                id: backlinkList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 4
                model: pane.note && pane.note.links ? pane.note.links.backlinks : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property var modelData
                    width: backlinkList.width
                    height: bcol.implicitHeight + 12
                    radius: 4
                    color: bHover.hovered ? pane.theme.hover : "transparent"
                    ColumnLayout {
                        id: bcol
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 6
                        spacing: 2
                        Text { text: pane.titleOf(modelData.from_slug); color: pane.theme.foreground; font.pixelSize: 13; elide: Text.ElideRight; Layout.fillWidth: true }
                        Text { text: modelData.context; color: pane.theme.muted; font.pixelSize: 12; wrapMode: Text.Wrap; maximumLineCount: 3; elide: Text.ElideRight; Layout.fillWidth: true }
                    }
                    HoverHandler { id: bHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: pane.openPage(modelData.from_slug) }
                }
            }
            Text { visible: pane.note !== null && pane.note.backlinkCount === 0; text: "No backlinks found."; color: pane.theme.faint; font.pixelSize: 12 }
        }

        // Outgoing links
        ColumnLayout {
            visible: pane.current === "outgoing"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 4
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: 12; elide: Text.ElideRight; Layout.fillWidth: true }
            Text { visible: pane.note !== null; text: "Links"; color: pane.theme.foreground; font.pixelSize: 13 }
            ListView {
                id: outList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                model: pane.note && pane.note.links ? pane.note.links.outbound : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property var modelData
                    width: outList.width
                    height: 28
                    radius: 4
                    color: oHover.hovered ? pane.theme.hover : "transparent"
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 6; anchors.rightMargin: 6
                        spacing: 6
                        Icon { name: modelData.resolved ? "link" : "unlink"; color: modelData.resolved ? pane.theme.link : pane.theme.faint; size: 14 }
                        Text { text: modelData.resolved ? pane.titleOf(modelData.to_slug) : modelData.to_slug; color: modelData.resolved ? pane.theme.foreground : pane.theme.faint; font.pixelSize: 13; elide: Text.ElideMiddle; Layout.fillWidth: true }
                        Text { visible: !modelData.resolved; text: "create"; color: pane.theme.link; font.pixelSize: 11 }
                    }
                    HoverHandler { id: oHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: modelData.resolved ? pane.openPage(modelData.to_slug) : pane.createPage(modelData.to_slug) }
                }
            }
            Text { visible: pane.note !== null && pane.note.links !== null && pane.note.links.outbound.length === 0; text: "No links."; color: pane.theme.faint; font.pixelSize: 12 }
        }

        // Outline
        ColumnLayout {
            visible: pane.current === "outline"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 2
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: 12; elide: Text.ElideRight; Layout.fillWidth: true }
            ListView {
                id: outlineList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: pane.note ? pane.note.outline : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property int index
                    required property var modelData
                    width: outlineList.width
                    height: 26
                    radius: 4
                    color: hHover.hovered ? pane.theme.hover : "transparent"
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 8 + (modelData.level - 1) * 14
                        anchors.right: parent.right
                        text: modelData.text
                        color: modelData.level === 1 ? pane.theme.foreground : pane.theme.muted
                        font.pixelSize: 13
                        elide: Text.ElideRight
                    }
                    HoverHandler { id: hHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: if (pane.note) pane.note.scrollToHeading(index) }
                }
            }
            Text { visible: pane.note !== null && pane.note.outline.length === 0; text: "No headings."; color: pane.theme.faint; font.pixelSize: 12 }
        }

        // The agent pane: one terminal that stays with the sidebar.
        ColumnLayout {
            visible: pane.current === "agent"
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: 6
                spacing: 6
                Text { text: "Agent"; color: pane.theme.muted; font.pixelSize: 12 }
                ComboBox {
                    id: programBox
                    Layout.fillWidth: true
                    model: pane.programs
                    font.pixelSize: 12
                    onActivated: pane.program = currentText
                    Component.onCompleted: { const i = pane.programs.indexOf(pane.program); currentIndex = i >= 0 ? i : 0; if (pane.program.length === 0 && pane.programs.length > 0) pane.program = pane.programs[0] }
                }
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: pane.theme.background
                AgentTerminal {
                    id: agentTerm
                    anchors.fill: parent
                    visible: pane.current === "agent" && pane.program.length > 0
                    theme: pane.theme
                    terminals: pane.terminals
                    program: pane.program
                    session: pane.program.length > 0 ? "rusty-pane-" + pane.program : ""
                    isCurrent: pane.current === "agent"
                    ready: pane.current === "agent"
                    windowActive: pane.windowActive
                }
            }
        }
    }

    component PaneTab: Rectangle {
        id: pt
        property string icon
        property string name
        property string tip: ""
        width: 28
        height: 26
        radius: 5
        color: pane.current === name ? pane.theme.active : (ptHover.hovered ? pane.theme.hover : "transparent")
        Icon { anchors.centerIn: parent; name: pt.icon; color: pane.current === pt.name ? pane.theme.foreground : pane.theme.muted; size: 16 }
        HoverHandler { id: ptHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: { pane.current = pt.name; pane.paneChanged(pt.name) } }
        ToolTip.visible: ptHover.hovered && pt.tip.length > 0
        ToolTip.text: pt.tip
        ToolTip.delay: 600
    }
}
