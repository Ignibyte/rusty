import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QMLTermWidget
import dev.ignibyte.rusty

ApplicationWindow {
    id: win
    visible: true
    width: 1500
    height: 950
    title: "Rusty"
    color: theme.background
    palette.window: theme.background
    palette.windowText: theme.foreground
    palette.base: Qt.darker(theme.background, 1.15)
    palette.alternateBase: theme.background
    palette.text: theme.foreground
    palette.button: Qt.lighter(theme.background, 1.35)
    palette.buttonText: theme.foreground
    palette.highlight: theme.accent
    palette.highlightedText: theme.background
    palette.placeholderText: Qt.rgba(1, 1, 1, 0.35)

    Theme { id: theme }
    Terminals { id: terminals }
    Backend { id: backend }

    // Agent tabs come first in the rail and the stack; the pages follow.
    readonly property var pageNames: ["Tasks", "Brain", "Notes", "Memory", "Skills", "Settings"]
    readonly property int tabCount: tabs.count + pageNames.length
    function pageIndex(i) { return tabs.count + i }
    function currentIsAgent() { return stack.currentIndex < tabs.count }

    ListModel { id: tabs }

    Component.onCompleted: {
        theme.watch()
        backend.start()
        const saved = JSON.parse(terminals.load())
        for (const t of saved)
            tabs.append({ name: t.name, session: t.session, program: t.program })
        stack.currentIndex = Math.min(theme.startTab, tabCount - 1)
    }

    function saveTabs() {
        const out = []
        for (let i = 0; i < tabs.count; i++) {
            const t = tabs.get(i)
            out.push({ name: t.name, session: t.session, program: t.program })
        }
        terminals.save(JSON.stringify(out))
    }

    function takenSessions() {
        const names = []
        for (let i = 0; i < tabs.count; i++) names.push(tabs.get(i).session)
        for (const s of terminals.sessions()) names.push(s)
        return names
    }

    function addTab(name, program, session) {
        const label = name.trim().length > 0 ? name.trim() : program.charAt(0).toUpperCase() + program.slice(1)
        const sess = session.length > 0 ? session : terminals.sessionName(label, takenSessions())
        tabs.append({ name: label, session: sess, program: program })
        saveTabs()
        stack.currentIndex = tabs.count - 1
    }

    function closeTab(i, endSession) {
        if (i < 0 || i >= tabs.count) return
        const sess = tabs.get(i).session
        tabs.remove(i)
        saveTabs()
        if (endSession) terminals.endSession(sess)
        if (stack.currentIndex >= tabCount) stack.currentIndex = tabCount - 1
    }

    function renameTab(i, name) {
        if (i < 0 || i >= tabs.count || name.trim().length === 0) return
        tabs.setProperty(i, "name", name.trim())
        saveTabs()
    }

    // Ctrl+PgUp / Ctrl+PgDn cycle; Ctrl+Shift+T / Ctrl+Shift+W add and close agent tabs.
    // None of these are used by Claude Code or Codex.
    Shortcut { sequences: ["Ctrl+PgDown"]; onActivated: stack.currentIndex = (stack.currentIndex + 1) % tabCount }
    Shortcut { sequences: ["Ctrl+PgUp"]; onActivated: stack.currentIndex = (stack.currentIndex + tabCount - 1) % tabCount }
    Shortcut { sequences: ["Ctrl+Shift+T"]; onActivated: newTabDialog.openFresh() }
    // Only live on agent tabs: otherwise they would swallow the keys before a page
    // (the task list uses F2 to rename a task) ever sees them.
    Shortcut { sequences: ["Ctrl+Shift+W"]; enabled: currentIsAgent(); onActivated: closeTab(stack.currentIndex, false) }
    Shortcut { sequences: ["F2"]; enabled: currentIsAgent(); onActivated: renameDialog.openFor(stack.currentIndex) }

    // One agent tab = one tmux session in the built-in terminal, coloured like Alacritty.
    // Closing the tab or the window leaves the session running.
    component AgentTab: Item {
        id: tab
        required property string session
        required property string program
        QMLTermWidget {
            id: term
            anchors.fill: parent
            font.family: theme.termFont
            font.pointSize: 11
            colorScheme: theme.termScheme
            session: QMLTermSession {
                id: termSession
                initialWorkingDirectory: theme.homeDir
                shellProgram: "tmux"
                shellProgramArgs: ["new-session", "-A", "-s", tab.session, terminals.commandFor(tab.program)]
            }
            // An empty session name would make tmux attach to whatever session it used last.
            Component.onCompleted: { if (tab.session.length > 0) termSession.startShellProgram(); term.forceActiveFocus() }
            QMLTermScrollbar { terminal: term; width: 8; Rectangle { anchors.fill: parent; color: theme.accent; opacity: 0.4; radius: 4 } }
        }
        function focusTerminal() { term.forceActiveFocus() }
    }

    component RailItem: Rectangle {
        id: item
        property string label
        property int stackIndex
        property bool agent: false
        signal activated()
        signal menuRequested()
        Layout.fillWidth: true
        height: 34
        radius: 6
        color: stack.currentIndex === stackIndex ? theme.accent : (hover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
        Text {
            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 12
            anchors.right: parent.right; anchors.rightMargin: 8
            text: item.label
            elide: Text.ElideRight
            color: stack.currentIndex === stackIndex ? theme.background : theme.foreground
            font.pixelSize: 14
        }
        HoverHandler { id: hover }
        TapHandler { acceptedButtons: Qt.LeftButton; onTapped: item.activated() }
        TapHandler { acceptedButtons: Qt.RightButton; enabled: item.agent; onTapped: item.menuRequested() }
    }

    Menu {
        id: tabMenu
        property int tabIndex: -1
        MenuItem { text: "Rename…"; onTriggered: renameDialog.openFor(tabMenu.tabIndex) }
        MenuItem { text: "Close tab (keep session)"; onTriggered: closeTab(tabMenu.tabIndex, false) }
        MenuItem { text: "Close tab and end session"; onTriggered: closeTab(tabMenu.tabIndex, true) }
    }

    Dialog {
        id: newTabDialog
        title: "New terminal"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        function openFresh() {
            programBox.model = terminals.programs()
            programBox.currentIndex = 0
            sessionBox.model = ["new session"].concat(terminals.sessions())
            sessionBox.currentIndex = 0
            nameField.text = ""
            open()
            nameField.forceActiveFocus()
        }
        onAccepted: addTab(nameField.text, programBox.currentText, sessionBox.currentIndex > 0 ? sessionBox.currentText : "")
        ColumnLayout {
            spacing: 10
            Label { text: "Name (optional)" }
            TextField { id: nameField; Layout.preferredWidth: 320; placeholderText: "Claude 2, Codex for droost, Scratch shell"; onAccepted: newTabDialog.accept() }
            Label { text: "Agent" }
            ComboBox { id: programBox; Layout.preferredWidth: 320 }
            Label { text: "Session" }
            ComboBox { id: sessionBox; Layout.preferredWidth: 320 }
        }
    }

    Dialog {
        id: renameDialog
        title: "Rename tab"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        property int tabIndex: -1
        function openFor(i) { tabIndex = i; renameField.text = tabs.get(i).name; open(); renameField.forceActiveFocus(); renameField.selectAll() }
        onAccepted: renameTab(tabIndex, renameField.text)
        TextField { id: renameField; width: 320; onAccepted: renameDialog.accept() }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillHeight: true
            width: 172
            color: theme.background
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 4
                Text { text: "Rusty"; color: theme.accent; font.pixelSize: 20; font.bold: true; Layout.bottomMargin: 10; Layout.leftMargin: 6 }
                Repeater {
                    model: tabs
                    delegate: RailItem {
                        required property int index
                        required property string name
                        label: name
                        stackIndex: index
                        agent: true
                        onActivated: stack.currentIndex = index
                        onMenuRequested: { tabMenu.tabIndex = index; tabMenu.popup() }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    height: 28
                    radius: 6
                    color: addHover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent"
                    Text { anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 12; text: "+ terminal"; color: theme.foreground; opacity: 0.7; font.pixelSize: 13 }
                    HoverHandler { id: addHover }
                    TapHandler { onTapped: newTabDialog.openFresh() }
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: theme.accent; opacity: 0.25; Layout.topMargin: 6; Layout.bottomMargin: 6 }
                Repeater {
                    model: pageNames
                    delegate: RailItem {
                        required property int index
                        required property string modelData
                        label: modelData
                        stackIndex: pageIndex(index)
                        onActivated: stack.currentIndex = pageIndex(index)
                    }
                }
                Item { Layout.fillHeight: true }
                Text { text: "theme " + theme.accent; color: theme.foreground; opacity: 0.5; font.pixelSize: 11; Layout.leftMargin: 6 }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: theme.accent; opacity: 0.25 }

        StackLayout {
            id: stack
            Layout.fillWidth: true
            Layout.fillHeight: true
            onCurrentIndexChanged: {
                if (currentIsAgent()) {
                    const item = agentTabs.itemAt(currentIndex)
                    if (item) item.focusTerminal()
                } else if (currentIndex === pageIndex(0)) {
                    tasksPage.focusAdd()
                } else if (currentIndex === pageIndex(1)) {
                    brainPage.focusEntry()
                } else if (currentIndex === pageIndex(2)) {
                    notesPage.focusEntry()
                } else if (currentIndex === pageIndex(3)) {
                    memoryPage.focusEntry()
                }
            }
            Repeater {
                id: agentTabs
                model: tabs
                delegate: AgentTab {}
            }
            TasksPage { id: tasksPage; backend: backend; theme: theme }
            BrainPage { id: brainPage; backend: backend; theme: theme }
            NotesPage { id: notesPage; backend: backend; theme: theme }
            MemoryPage { id: memoryPage; backend: backend; theme: theme }
            Repeater {
                model: ["Skills"]
                delegate: Rectangle {
                    required property string modelData
                    color: theme.background
                    Text { anchors.centerIn: parent; text: modelData + " tab arrives with M5"; color: theme.foreground; font.pixelSize: 16 }
                }
            }
            SettingsPage { backend: backend; theme: theme; terminals: terminals }
        }
    }
}
