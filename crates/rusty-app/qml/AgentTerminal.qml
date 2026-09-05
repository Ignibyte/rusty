import QtQuick
import QtQuick.Controls
import QMLTermWidget
import dev.ignibyte.rusty

// One agent terminal: a tmux session in the built-in terminal widget, coloured like
// Alacritty. Used by terminal tabs and by the right pane. Closing the tab or the window
// leaves the session running.
Item {
    id: tab
    required property var theme
    required property var terminals
    property string session: ""
    property string program: "shell"
    property string cwd: ""
    property bool isCurrent: false
    property bool windowActive: true
    // The session starts when the terminal is first meant to be seen, not at load.
    property bool ready: true
    property bool started: false
    readonly property string startDir: cwd.length > 0 ? cwd : theme.homeDir
    // The terminal's own title (what Claude Code or Codex set), hostname default hidden.
    property string title: ""
    signal unread()
    signal attention(string message)
    // Whether there is a selection to copy; the widget says so through copyAvailable.
    property bool copyAvailable: false

    onIsCurrentChanged: if (isCurrent) term.forceActiveFocus()
    onReadyChanged: if (ready) start()
    // An empty session name would make tmux attach to whatever session it used last.
    function start() { if (!started && session.length > 0) { started = true; termSession.startShellProgram() } }
    function focusTerminal() { term.forceActiveFocus() }

    QMLTermWidget {
        id: term
        objectName: "term"
        anchors.fill: parent
        font.family: tab.theme.termFont
        font.pointSize: 11
        colorScheme: tab.theme.termScheme
        session: QMLTermSession {
            id: termSession
            initialWorkingDirectory: tab.startDir
            shellProgram: "tmux"
            // After the session is up, tmux forwards the pane title (what Claude Code or
            // Codex set) to us, which the tab shows under its name.
            shellProgramArgs: ["new-session", "-A", "-s", tab.session, "-c", tab.startDir, tab.terminals.commandFor(tab.program),
                               ";", "set-option", "-t", tab.session, "set-titles", "on",
                               ";", "set-option", "-t", tab.session, "set-titles-string", "#T"]
        }
        Component.onCompleted: { if (tab.ready) tab.start(); if (tab.isCurrent) term.forceActiveFocus() }
        QMLTermScrollbar { terminal: term; width: 8; Rectangle { anchors.fill: parent; color: tab.theme.accent; opacity: 0.4; radius: 4 } }
        // The terminal convention: the shifted chords copy and paste, so plain Ctrl+C
        // stays the interrupt. Attached before the widget's own handling, because the
        // workspace's shortcuts stand down while a terminal has focus and would never
        // see these.
        Keys.priority: Keys.BeforeItem
        Keys.onPressed: (event) => {
            const chord = (event.modifiers & Qt.ControlModifier) && (event.modifiers & Qt.ShiftModifier)
            if (!chord) return
            if (event.key === Qt.Key_C) { term.copyClipboard(); event.accepted = true }
            else if (event.key === Qt.Key_V) { term.pasteClipboard(); event.accepted = true }
        }
        // Middle click pastes the primary selection, right click opens the menu. A
        // handler rather than a MouseArea: it never touches the wheel, it accepts only
        // these two buttons so a left-button drag still selects, and its exclusive grab
        // keeps the widget's own middle-click handling from pasting a second time.
        TapHandler {
            acceptedButtons: Qt.MiddleButton | Qt.RightButton
            gesturePolicy: TapHandler.ReleaseWithinBounds
            onTapped: (point, button) => {
                if (button === Qt.MiddleButton) term.pasteSelection()
                else termMenu.popup()
            }
        }
    }

    Menu {
        id: termMenu
        MenuItem { text: "Copy"; enabled: tab.copyAvailable; onTriggered: term.copyClipboard() }
        MenuItem { text: "Paste"; onTriggered: term.pasteClipboard() }
    }

    function markUnread() { if (!tab.isCurrent) tab.unread() }
    // A bell in a tab that is not showing, or while the window is not focused, gets a
    // desktop notification; a bell in the tab you are looking at does not.
    function rang() {
        tab.markUnread()
        if (!tab.isCurrent || !tab.windowActive) tab.attention("rang the bell")
        if (tab.theme.debug) console.log("rusty: bell in", tab.session)
    }
    // Codex and Claude Code announce attention through the title ("[ ! ] Action
    // Required", "needs your input"); the bell does not reach us through tmux. A tab
    // that is not showing, or a window that is not focused, turns that into a desktop
    // notification, once per title and never more than once a minute.
    property string lastAlert: ""
    property double lastAlertAt: 0
    function titled() {
        const t = termSession.title || ""
        const hidden = t.length === 0 || t === tab.theme.hostName || t.startsWith(tab.theme.hostName + ":")
        tab.title = hidden ? "" : t
        if (tab.theme.debug) console.log("rusty: title of", tab.session, JSON.stringify(t))
        const wantsYou = /\[ ! \]|action required|needs your|waiting for you|permission/i.test(t)
        const key = t.replace(/^\[ . \]\s*/, "")
        const now = Date.now()
        if (wantsYou && key !== lastAlert && now - lastAlertAt > 60000 && (!tab.isCurrent || !tab.windowActive)) {
            lastAlert = key
            lastAlertAt = now
            tab.attention(key)
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
        function onCopyAvailable(available) { tab.copyAvailable = available }
        function onActivity() { tab.markUnread() }
        function onBell() { tab.rang() }
        function onNotifyBell() { tab.rang() }
    }
}
