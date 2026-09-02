import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QMLTermWidget

ApplicationWindow {
    id: win
    visible: true
    width: 1500
    height: 950
    title: "Rusty (prototype)"
    color: omarchyTheme.background

    readonly property var tabNames: ["Claude", "Codex", "Tasks", "Brain", "Notes", "Memory", "Skills", "Settings"]

    // Ctrl+PgUp / Ctrl+PgDn cycle tabs; neither is used by Claude Code or Codex.
    Shortcut { sequences: ["Ctrl+PgDown"]; onActivated: stack.currentIndex = (stack.currentIndex + 1) % tabNames.length }
    Shortcut { sequences: ["Ctrl+PgUp"]; onActivated: stack.currentIndex = (stack.currentIndex + tabNames.length - 1) % tabNames.length }

    // One agent tab = one tmux session in the built-in terminal, coloured like Alacritty.
    // Closing the window leaves the session running.
    component AgentTab: Item {
        id: tab
        property string sessionName
        property string program
        property string label
        QMLTermWidget {
            id: term
            anchors.fill: parent
            font.family: termFont
            font.pointSize: 11
            colorScheme: termScheme
            session: QMLTermSession {
                id: termSession
                initialWorkingDirectory: homeDir
                shellProgram: "tmux"
                shellProgramArgs: ["new-session", "-A", "-s", tab.sessionName, tab.program]
            }
            Component.onCompleted: { termSession.startShellProgram(); term.forceActiveFocus() }
            QMLTermScrollbar { terminal: term; width: 8; Rectangle { anchors.fill: parent; color: omarchyTheme.accent; opacity: 0.4; radius: 4 } }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillHeight: true
            width: 172
            color: omarchyTheme.background
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 4
                Text { text: "Rusty"; color: omarchyTheme.accent; font.pixelSize: 20; font.bold: true; Layout.bottomMargin: 10; Layout.leftMargin: 6 }
                Repeater {
                    model: tabNames
                    delegate: Rectangle {
                        required property int index
                        required property string modelData
                        Layout.fillWidth: true
                        height: 34
                        radius: 6
                        color: stack.currentIndex === index ? omarchyTheme.accent : (hover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
                        Text {
                            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 12
                            text: modelData
                            color: stack.currentIndex === index ? omarchyTheme.background : omarchyTheme.foreground
                            font.pixelSize: 14
                        }
                        HoverHandler { id: hover }
                        TapHandler { onTapped: stack.currentIndex = index }
                    }
                }
                Item { Layout.fillHeight: true }
                Text { text: "theme " + omarchyTheme.accent; color: omarchyTheme.foreground; opacity: 0.5; font.pixelSize: 11; Layout.leftMargin: 6 }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: omarchyTheme.accent; opacity: 0.25 }

        StackLayout {
            id: stack
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: startTab
            AgentTab { sessionName: "rusty-claude"; program: "claude"; label: "Claude" }
            AgentTab { sessionName: "rusty-codex"; program: "codex"; label: "Codex" }
            Repeater {
                model: ["Tasks", "Brain", "Notes", "Memory", "Skills"]
                delegate: Rectangle {
                    required property string modelData
                    color: omarchyTheme.background
                    Text { anchors.centerIn: parent; text: modelData + " tab: data comes from rusty-mcp over local HTTP (M2, M3)"; color: omarchyTheme.foreground; font.pixelSize: 16 }
                }
            }
            // Settings: kept as a page; the first real settings arrive with M2 (paths, theme,
            // terminal font) and M4 (embedding provider).
            Rectangle {
                color: omarchyTheme.background
                ColumnLayout {
                    anchors.left: parent.left; anchors.top: parent.top; anchors.margins: 32
                    spacing: 10
                    Text { text: "Settings"; color: omarchyTheme.foreground; font.pixelSize: 22; font.bold: true }
                    Text { text: "Nothing to configure yet. Settings land with the features that need them: paths and theme (M2), embedding provider (M4), skills and secrets (M5)."; color: omarchyTheme.foreground; opacity: 0.75; font.pixelSize: 14; wrapMode: Text.WordWrap; Layout.preferredWidth: 640 }
                    Text { text: "terminal font " + termFont + "  ·  colour scheme " + termScheme + "  ·  theme accent " + omarchyTheme.accent; color: omarchyTheme.foreground; opacity: 0.5; font.pixelSize: 12; Layout.topMargin: 8 }
                }
            }
        }
    }
}
