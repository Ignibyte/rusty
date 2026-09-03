import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The mock's top bar: the brand, the command button and one small glyph per agent CLI
// found on the machine, the vault's state and size in the middle, and memory, CPU and
// the clock on the right. The readings come from `Desk`; the agents and their glyphs
// come from the window, which owns the tabs and the pane the signals open. The Hyprland
// workspace strip went with TICKET-011: waybar shows the workspaces already.
Rectangle {
    id: bar
    required property var theme
    required property var desk
    required property var backend
    // Programs in launch order (`terminals.programs()`), the window's glyph and name maps.
    property var agents: []
    property var agentGlyphs: ({})
    property var agentNames: ({})
    property int pages: 0
    property string vaultName: ""
    signal quit()
    signal commandRequested()
    signal agentRequested(string program)
    signal agentPaneRequested(string program)

    implicitHeight: 33
    color: bar.theme.panel2
    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: bar.theme.line }

    component Micro: Text {
        color: bar.theme.muted
        font.pixelSize: 10
        font.letterSpacing: 1.2
        font.capitalization: Font.AllUppercase
    }

    // A small button at the bar's size: an icon or a glyph, a hover frame, a tooltip, and
    // a left click and a right click as separate signals.
    component BarButton: Item {
        id: bb
        property string icon: ""
        property string glyph: ""
        property string tip: ""
        signal clicked()
        signal rightClicked()
        readonly property bool lit: bbHover.hovered
        width: Math.max(20, content.implicitWidth + 10)
        height: 19
        Rectangle {
            anchors.fill: parent
            radius: 3
            color: bb.lit ? bar.theme.panel3 : "transparent"
            border.width: 1
            border.color: bb.lit ? bar.theme.lineBright : "transparent"
        }
        Row {
            id: content
            anchors.centerIn: parent
            spacing: 4
            Icon { visible: bb.icon.length > 0; anchors.verticalCenter: parent.verticalCenter; name: bb.icon; color: bb.lit ? bar.theme.accent : bar.theme.muted; size: 12 }
            Text { visible: bb.glyph.length > 0; anchors.verticalCenter: parent.verticalCenter; text: bb.glyph; color: bb.lit ? bar.theme.accent : bar.theme.muted; font.pixelSize: 12 }
        }
        HoverHandler { id: bbHover; cursorShape: Qt.PointingHandCursor; enabled: bar.theme.shotPath.length === 0 }
        TapHandler { acceptedButtons: Qt.LeftButton; onTapped: bb.clicked() }
        TapHandler { acceptedButtons: Qt.RightButton; onTapped: bb.rightClicked() }
        ToolTip.visible: bbHover.hovered && bb.tip.length > 0
        ToolTip.text: bb.tip
        ToolTip.delay: 500
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 13
        spacing: 14

        // Brand.
        Text { text: "▰"; color: bar.theme.gold; font.pixelSize: 10 }
        Micro { text: "Omarchy // Rusty"; color: bar.theme.accent; font.bold: true; font.letterSpacing: 1.6; Layout.leftMargin: -9 }

        // The command layer, then the agents: click for a new tab, right-click for the
        // pane beside the note.
        Row {
            spacing: 2
            BarButton { icon: "command"; tip: "Command palette (Ctrl+P)"; onClicked: bar.commandRequested() }
            Repeater {
                model: bar.agents
                delegate: BarButton {
                    required property string modelData
                    glyph: bar.agentGlyphs[modelData] || "▸"
                    tip: (bar.agentNames[modelData] || modelData) + ": click for a new tab, right-click for the agent pane"
                    onClicked: bar.agentRequested(modelData)
                    onRightClicked: bar.agentPaneRequested(modelData)
                }
            }
        }

        Item { Layout.fillWidth: true }

        // The vault.
        Text { text: "●"; color: bar.backend.connected ? bar.theme.alive : bar.theme.red; font.pixelSize: 12 }
        Micro {
            text: (bar.backend.connected ? "Vault online" : "Vault offline") + " // " + (bar.pages > 0 ? bar.pages + " pages" : "local-first knowledge system")
            Layout.leftMargin: -6
        }

        Item { Layout.fillWidth: true }

        // The machine.
        Micro { visible: bar.desk.memory.length > 0; text: "mem " + bar.desk.memory }
        Micro { visible: bar.desk.cpu.length > 0; text: "cpu " + bar.desk.cpu }
        Micro { text: bar.desk.clock; color: bar.theme.bright; font.bold: true }
        Text {
            text: "⏻"
            color: powerHover.hovered ? bar.theme.accent : bar.theme.muted
            font.pixelSize: 12
            HoverHandler { id: powerHover; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: bar.quit() }
            ToolTip.visible: powerHover.hovered
            ToolTip.text: "Quit Rusty (terminals keep running in tmux)"
            ToolTip.delay: 600
        }
    }
}
