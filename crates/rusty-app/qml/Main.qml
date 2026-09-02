import QtCore
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
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

    // Remembered between runs: window size and the tab that was open.
    Settings {
        id: ui
        category: "window"
        property int width: 1500
        property int height: 950
        property int lastTab: 0
    }
    onWidthChanged: if (visible) ui.width = width
    onHeightChanged: if (visible) ui.height = height

    // Agent tabs come first in the rail and the stack; the pages follow.
    readonly property var pageNames: ["Tasks", "Brain", "Notes", "Memory", "Skills", "Secrets", "Settings"]
    readonly property int tabCount: tabs.count + pageNames.length
    function pageIndex(i) { return tabs.count + i }
    function currentIsAgent() { return stack.currentIndex < tabs.count }

    // name, session, program, cwd are saved; unread and title live only while running.
    ListModel { id: tabs }

    Component.onCompleted: {
        win.width = ui.width
        win.height = ui.height
        theme.watch()
        backend.start()
        const saved = JSON.parse(terminals.load())
        for (const t of saved)
            tabs.append({ name: t.name, session: t.session, program: t.program, cwd: t.cwd || "", unread: false, title: "" })
        const wanted = theme.startTab >= 0 ? theme.startTab : ui.lastTab
        stack.currentIndex = Math.max(0, Math.min(wanted, tabCount - 1))
    }

    function saveTabs() {
        const out = []
        for (let i = 0; i < tabs.count; i++) {
            const t = tabs.get(i)
            out.push({ name: t.name, session: t.session, program: t.program, cwd: t.cwd })
        }
        terminals.save(JSON.stringify(out))
    }

    function takenSessions() {
        const names = []
        for (let i = 0; i < tabs.count; i++) names.push(tabs.get(i).session)
        for (const s of terminals.sessions()) names.push(s)
        return names
    }

    function addTab(name, program, session, cwd) {
        const label = name.trim().length > 0 ? name.trim() : program.charAt(0).toUpperCase() + program.slice(1)
        const sess = session.length > 0 ? session : terminals.sessionName(label, takenSessions())
        tabs.append({ name: label, session: sess, program: program, cwd: cwd.trim(), unread: false, title: "" })
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

    function folderPath(url) {
        let s = String(url)
        if (s.startsWith("file://")) s = s.slice(7)
        return decodeURIComponent(s)
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
        required property int index
        required property string session
        required property string program
        required property string cwd
        readonly property bool isCurrent: stack.currentIndex === index
        readonly property string startDir: cwd.length > 0 ? cwd : theme.homeDir
        onIsCurrentChanged: if (isCurrent) { tabs.setProperty(index, "unread", false); term.forceActiveFocus() }
        QMLTermWidget {
            id: term
            anchors.fill: parent
            font.family: theme.termFont
            font.pointSize: 11
            colorScheme: theme.termScheme
            session: QMLTermSession {
                id: termSession
                initialWorkingDirectory: tab.startDir
                shellProgram: "tmux"
                // After the session is up, tmux forwards the pane title (what Claude Code or
                // Codex set) to us, which the rail shows under the tab name.
                shellProgramArgs: ["new-session", "-A", "-s", tab.session, "-c", tab.startDir, terminals.commandFor(tab.program),
                                   ";", "set-option", "-t", tab.session, "set-titles", "on",
                                   ";", "set-option", "-t", tab.session, "set-titles-string", "#T"]
            }
            // An empty session name would make tmux attach to whatever session it used last.
            Component.onCompleted: { if (tab.session.length > 0) termSession.startShellProgram(); term.forceActiveFocus() }
            QMLTermScrollbar { terminal: term; width: 8; Rectangle { anchors.fill: parent; color: theme.accent; opacity: 0.4; radius: 4 } }
        }
        function focusTerminal() { term.forceActiveFocus() }
        function markUnread() {
            if (tab.isCurrent) return
            if (!tabs.get(tab.index).unread && theme.debug) console.log("rusty: unread tab", tab.index, tabs.get(tab.index).name)
            tabs.setProperty(tab.index, "unread", true)
        }
        // A bell in a tab that is not showing, or while the window is not focused, gets a
        // desktop notification; a bell in the tab you are looking at does not.
        function rang() {
            tab.markUnread()
            if (!tab.isCurrent || !win.active) terminals.notify(tabs.get(tab.index).name, "rang the bell")
            if (theme.debug) console.log("rusty: bell in tab", tab.index)
        }
        // Codex and Claude Code announce attention through the title ("[ ! ] Action
        // Required", "needs your input"); the bell does not reach us through tmux. A tab
        // that is not showing, or a window that is not focused, turns that into a
        // desktop notification, once per title.
        property string lastAlert: ""
        property double lastAlertAt: 0
        function titled() {
            const t = termSession.title || ""
            tabs.setProperty(tab.index, "title", t)
            if (theme.debug) console.log("rusty: title of tab", tab.index, JSON.stringify(t))
            const wantsYou = /\[ ! \]|action required|needs your|waiting for you|permission/i.test(t)
            // Codex blinks "[ ! ]" and "[ . ]" in front of the same message; compare without
            // that frame, and never more than once a minute per tab.
            const key = t.replace(/^\[ . \]\s*/, "")
            const now = Date.now()
            if (wantsYou && key !== lastAlert && now - lastAlertAt > 60000 && (!tab.isCurrent || !win.active)) {
                lastAlert = key
                lastAlertAt = now
                terminals.notify(tabs.get(tab.index).name, key)
                if (theme.debug) console.log("rusty: attention in tab", tab.index)
            }
            if (!wantsYou) lastAlert = ""
        }
        // The widget and its session each expose some of these; unknown ones are ignored.
        Connections {
            target: termSession
            ignoreUnknownSignals: true
            function onTitleChanged() { tab.titled() }
            function onActivity() { tab.markUnread() }
            function onBell() { tab.rang() }
            function onReceivedData() { tab.markUnread() }
        }
        Connections {
            target: term
            ignoreUnknownSignals: true
            // imagePainted fires whenever the emulation has new output, shown or not.
            function onImagePainted() { tab.markUnread() }
            function onActivity() { tab.markUnread() }
            function onBell() { tab.rang() }
            function onNotifyBell() { tab.rang() }
        }
    }

    component RailItem: Rectangle {
        id: item
        property string label
        property string subtitle: ""
        property int stackIndex
        property bool agent: false
        property bool unread: false
        signal activated()
        signal menuRequested()
        Layout.fillWidth: true
        height: subtitle.length > 0 ? 40 : 34
        radius: 6
        color: stack.currentIndex === stackIndex ? theme.accent : (hover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
        ColumnLayout {
            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 12
            anchors.right: dot.visible ? dot.left : parent.right; anchors.rightMargin: 8
            spacing: 0
            Text {
                text: item.label
                elide: Text.ElideRight
                Layout.fillWidth: true
                color: stack.currentIndex === item.stackIndex ? theme.background : theme.foreground
                font.pixelSize: 14
            }
            Text {
                visible: item.subtitle.length > 0
                text: item.subtitle
                elide: Text.ElideRight
                Layout.fillWidth: true
                color: stack.currentIndex === item.stackIndex ? theme.background : theme.foreground
                opacity: 0.6
                font.pixelSize: 10
            }
        }
        Rectangle {
            id: dot
            visible: item.unread
            width: 8; height: 8; radius: 4
            anchors.right: parent.right; anchors.rightMargin: 10; anchors.verticalCenter: parent.verticalCenter
            color: stack.currentIndex === item.stackIndex ? theme.background : theme.accent
        }
        HoverHandler { id: hover }
        TapHandler { acceptedButtons: Qt.LeftButton; onTapped: item.activated() }
        TapHandler { acceptedButtons: Qt.RightButton; enabled: item.agent; onTapped: item.menuRequested() }
        ToolTip.visible: hover.hovered && item.subtitle.length > 0
        ToolTip.text: item.subtitle
        ToolTip.delay: 600
    }

    Menu {
        id: tabMenu
        property int tabIndex: -1
        MenuItem { text: "Rename…"; onTriggered: renameDialog.openFor(tabMenu.tabIndex) }
        MenuItem { text: "Close tab (keep session)"; onTriggered: closeTab(tabMenu.tabIndex, false) }
        MenuItem { text: "Close tab and end session"; onTriggered: closeTab(tabMenu.tabIndex, true) }
    }

    FolderDialog {
        id: folderDialog
        title: "Working directory"
        onAccepted: cwdField.text = folderPath(selectedFolder)
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
            cwdField.text = ""
            open()
            nameField.forceActiveFocus()
        }
        onAccepted: addTab(nameField.text, programBox.currentText, sessionBox.currentIndex > 0 ? sessionBox.currentText : "", cwdField.text)
        ColumnLayout {
            spacing: 10
            Label { text: "Name (optional)" }
            TextField { id: nameField; Layout.preferredWidth: 320; placeholderText: "Claude 2, Codex for droost, Scratch shell"; onAccepted: newTabDialog.accept() }
            Label { text: "Agent" }
            ComboBox { id: programBox; Layout.preferredWidth: 320 }
            Label { text: "Session" }
            ComboBox { id: sessionBox; Layout.preferredWidth: 320 }
            Label { text: "Working directory" }
            RowLayout {
                spacing: 6
                TextField { id: cwdField; Layout.preferredWidth: 240; placeholderText: theme.homeDir; onAccepted: newTabDialog.accept() }
                Button { text: "Browse…"; onClicked: { folderDialog.currentFolder = "file://" + (cwdField.text.length > 0 ? cwdField.text : theme.homeDir); folderDialog.open() } }
            }
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
                        required property string title
                        required property bool unread
                        label: name
                        subtitle: (title.length === 0 || title === theme.hostName || title.startsWith(theme.hostName + ":")) ? "" : title
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
                ui.lastTab = currentIndex
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
                } else if (currentIndex === pageIndex(4)) {
                    skillsPage.focusEntry()
                } else if (currentIndex === pageIndex(5)) {
                    secretsPage.focusEntry()
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
            SkillsPage { id: skillsPage; backend: backend; theme: theme }
            SecretsPage { id: secretsPage; backend: backend; theme: theme }
            SettingsPage { backend: backend; theme: theme; terminals: terminals }
        }
    }
}
