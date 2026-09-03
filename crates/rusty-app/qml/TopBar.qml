import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The mock's top bar: the brand, Hyprland's workspaces with the active one lit, the
// vault's state and size in the middle, and memory, CPU and the clock on the right.
// Every reading comes from `Desk`; off Hyprland the strip is the mock's static one.
Rectangle {
    id: bar
    required property var theme
    required property var desk
    required property var backend
    property int pages: 0
    property string vaultName: ""
    signal quit()

    implicitHeight: 33
    color: bar.theme.panel2
    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: bar.theme.line }

    readonly property var workspaceIds: { try { const v = JSON.parse(bar.desk.workspaces); return Array.isArray(v) ? v : [1, 2, 3, 4] } catch (e) { return [1, 2, 3, 4] } }

    component Micro: Text {
        color: bar.theme.muted
        font.pixelSize: 10
        font.letterSpacing: 1.2
        font.capitalization: Font.AllUppercase
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 13
        spacing: 14

        // Brand.
        Text { text: "▰"; color: bar.theme.gold; font.pixelSize: 10 }
        Micro { text: "Omarchy // Rusty"; color: bar.theme.accent; font.bold: true; font.letterSpacing: 1.6; Layout.leftMargin: -9 }

        // Workspaces.
        Row {
            spacing: 6
            Repeater {
                model: bar.workspaceIds
                delegate: Rectangle {
                    required property var modelData
                    readonly property bool lit: modelData === bar.desk.activeWorkspace
                    width: Math.max(19, wsText.implicitWidth + 10)
                    height: 17
                    color: lit ? bar.theme.accent : bar.theme.panel3
                    border.width: 1
                    border.color: lit ? bar.theme.accent : bar.theme.line
                    Text { id: wsText; anchors.centerIn: parent; text: modelData; color: lit ? bar.theme.background : bar.theme.muted; font.pixelSize: 10 }
                    HoverHandler { cursorShape: bar.desk.hyprland ? Qt.PointingHandCursor : Qt.ArrowCursor }
                    TapHandler { onTapped: bar.desk.switchWorkspace(modelData) }
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
