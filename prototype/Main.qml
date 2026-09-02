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

    // One terminal tab = one tmux session; closing the window leaves the session running.
    component AgentTab: Item {
        property string sessionName
        property string program
        QMLTermWidget {
            id: term
            anchors.fill: parent
            font.family: "JetBrainsMono Nerd Font Mono"
            font.pointSize: 11
            colorScheme: "Linux"
            session: QMLTermSession {
                id: termSession
                initialWorkingDirectory: homeDir
                shellProgram: "tmux"
                shellProgramArgs: ["new-session", "-A", "-s", sessionName, program]
            }
            Component.onCompleted: { termSession.startShellProgram(); term.forceActiveFocus() }
            QMLTermScrollbar { terminal: term; width: 8; Rectangle { anchors.fill: parent; color: omarchyTheme.accent; opacity: 0.4; radius: 4 } }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            height: 40
            color: omarchyTheme.background
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 12
                spacing: 4
                Repeater {
                    model: ["Claude", "Codex", "Tasks", "Brain", "Notes", "Memory", "Skills", "Settings"]
                    delegate: Rectangle {
                        required property int index
                        required property string modelData
                        width: label.implicitWidth + 24
                        height: 30
                        radius: 6
                        color: stack.currentIndex === index ? omarchyTheme.accent : "transparent"
                        Text {
                            id: label
                            anchors.centerIn: parent
                            text: modelData
                            color: stack.currentIndex === index ? omarchyTheme.background : omarchyTheme.foreground
                            font.pixelSize: 14
                        }
                        MouseArea { anchors.fill: parent; onClicked: stack.currentIndex = index }
                    }
                }
                Item { Layout.fillWidth: true }
                Text { text: "Omarchy theme: " + omarchyTheme.accent; color: omarchyTheme.foreground; opacity: 0.6; font.pixelSize: 12; Layout.rightMargin: 12 }
            }
        }

        StackLayout {
            id: stack
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: 0
            AgentTab { sessionName: "rusty-claude"; program: "claude" }
            AgentTab { sessionName: "rusty-codex"; program: "codex" }
            Repeater {
                model: ["Tasks", "Brain", "Notes", "Memory", "Skills", "Settings"]
                delegate: Rectangle {
                    required property string modelData
                    color: omarchyTheme.background
                    Text {
                        anchors.centerIn: parent
                        text: modelData + " tab: data comes from rusty-mcp over local HTTP (M2, M3)"
                        color: omarchyTheme.foreground
                        font.pixelSize: 16
                    }
                }
            }
        }
    }
}
