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

    // One agent = one tmux session. Embedded mode shows it in the widget; alacritty mode
    // opens a real Alacritty window on the same session, so switching keeps the conversation.
    component AgentTab: Item {
        id: tab
        property string sessionName
        property string program
        property string label

        Loader {
            anchors.fill: parent
            active: settings.terminalMode === "embedded"
            sourceComponent: QMLTermWidget {
                id: term
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

        ColumnLayout {
            anchors.centerIn: parent
            visible: settings.terminalMode !== "embedded"
            spacing: 14
            width: 520
            Text { text: tab.label + " runs in Alacritty"; color: omarchyTheme.foreground; font.pixelSize: 22; font.bold: true }
            Text {
                text: "tmux session " + tab.sessionName + ". The window is a view on it; close the window and the conversation keeps going. Switch back to the embedded terminal in Settings and you land in the same session."
                color: omarchyTheme.foreground; opacity: 0.75; font.pixelSize: 14; wrapMode: Text.WordWrap; Layout.fillWidth: true
            }
            Button {
                text: settings.isOpen(tab.sessionName) ? "Focus the Alacritty window" : "Open " + tab.label + " in Alacritty"
                onClicked: { settings.launchOrFocus(tab.sessionName, tab.program); refresh.restart() }
                Timer { id: refresh; interval: 1500; onTriggered: parent.text = settings.isOpen(tab.sessionName) ? "Focus the Alacritty window" : "Open " + tab.label + " in Alacritty" }
            }
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
            // Settings: the one setting the prototype has.
            Rectangle {
                color: omarchyTheme.background
                ColumnLayout {
                    anchors.left: parent.left; anchors.top: parent.top; anchors.margins: 32
                    spacing: 12
                    Text { text: "Agent terminals"; color: omarchyTheme.foreground; font.pixelSize: 22; font.bold: true }
                    Text { text: "Where Claude and Codex run. Both choices attach to the same tmux sessions."; color: omarchyTheme.foreground; opacity: 0.75; font.pixelSize: 14 }
                    RadioButton {
                        text: "Embedded terminal, styled with the Omarchy theme's Alacritty colours"
                        checked: settings.terminalMode === "embedded"
                        onToggled: if (checked) settings.terminalMode = "embedded"
                        contentItem: Text { text: parent.text; color: omarchyTheme.foreground; font.pixelSize: 14; leftPadding: parent.indicator.width + 8; verticalAlignment: Text.AlignVCenter }
                    }
                    RadioButton {
                        text: "Alacritty windows, opened and focused from the tab"
                        checked: settings.terminalMode === "alacritty"
                        onToggled: if (checked) settings.terminalMode = "alacritty"
                        contentItem: Text { text: parent.text; color: omarchyTheme.foreground; font.pixelSize: 14; leftPadding: parent.indicator.width + 8; verticalAlignment: Text.AlignVCenter }
                    }
                    Text { text: "font " + termFont + "  ·  scheme " + termScheme + "  ·  " + configPath; color: omarchyTheme.foreground; opacity: 0.5; font.pixelSize: 12; Layout.topMargin: 12 }
                }
            }
        }
    }
}
