import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Secrets tab: names in the vault, set or replace a value, delete; and, behind a PIN
// the back end keeps, reveal one value at a time, edit it in place, copy it. The PIN
// protects this screen, not the file: `~/.rusty/.secret` stays owner-readable for the
// back end and the agents, and the server hands a value out only against a live unlock
// token that lives in this page's memory (TICKET-015).
Item {
    id: page
    required property var backend
    required property var theme

    property var names: []
    property string notice: ""
    property var pending: ({})
    // The lock as the server reports it, and this page's own unlock.
    property bool pinSet: false
    property bool unlocked: false
    property int lockedOutSeconds: 0
    property string token: ""
    property string revealedKey: ""
    property string revealedValue: ""
    property bool changingPin: false

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("secret_pin_status", {}, "status"); ask("secret_list", {}, "list") }
    function set(key, value) {
        if (key.trim().length === 0 || value.length === 0) { notice = "a key and a value are both needed"; return }
        ask("secret_set", { key: key.trim(), value: value }, "set")
    }
    function remove(key) { ask("secret_delete", { key: key }, "deleted") }
    function setPin(pin, again) {
        if (pin.length < 6) { notice = "a PIN needs at least 6 characters"; return }
        if (pin !== again) { notice = "the two PINs differ"; return }
        ask("secret_pin_set", page.pinSet ? { pin: pin, token: page.token } : { pin: pin }, "pinset")
    }
    function unlock(pin) { if (pin.length === 0) return; ask("secret_unlock", { pin: pin }, "unlock") }
    function lock() { const was = page.unlocked; lockLocal(); if (was) ask("secret_lock", {}, "lock") }
    function lockLocal() { page.unlocked = false; page.token = ""; page.revealedKey = ""; page.revealedValue = ""; page.changingPin = false; expiry.stop() }
    function reveal(key) {
        if (!page.unlocked) return
        if (page.revealedKey === key) { page.revealedKey = ""; page.revealedValue = ""; return }
        ask("secret_reveal", { key: key, token: page.token }, "reveal")
    }
    function update(key, value) { if (!page.unlocked || value.length === 0) return; ask("secret_update", { key: key, value: value, token: page.token }, "update") }
    function copyValue(text) { copier.text = text; copier.selectAll(); copier.copy(); copier.text = ""; page.notice = "copied" }
    function focusEntry() { if (page.pinSet && !page.unlocked) unlockField.forceActiveFocus(); else keyField.forceActiveFocus() }

    // The unlock ends on its own after the life the server gave it.
    Timer { id: expiry; repeat: false; onTriggered: page.lock() }
    // A lockout counts down on the server; ask again every few seconds until it ends.
    Timer { running: page.lockedOutSeconds > 0; interval: 5000; repeat: true; onTriggered: page.ask("secret_pin_status", {}, "status") }
    // The clipboard has no QML API of its own; a hidden editor does the copy.
    TextEdit { id: copier; visible: false; width: 1; height: 1 }
    // Losing the window relocks.
    readonly property bool windowActive: Window.active
    onWindowActiveChanged: if (!windowActive && page.unlocked) page.lock()

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) {
                page.notice = tool + ": " + json
                if (kind === "reveal" || kind === "update") page.lockLocal()
                if (kind === "unlock") ask("secret_pin_status", {}, "status")
                return
            }
            page.notice = ""
            if (kind === "list") {
                const list = JSON.parse(json)
                page.names = list.map(e => typeof e === "string" ? e : (e.key || e.name || JSON.stringify(e))).sort()
            } else if (kind === "status") {
                const s = JSON.parse(json)
                page.pinSet = !!s.set
                page.lockedOutSeconds = s.locked_out_seconds || 0
                if (!s.unlocked && page.unlocked) page.lockLocal()
            } else if (kind === "set") { keyField.text = ""; valueField.text = ""; page.notice = "stored"; page.refresh() }
            else if (kind === "deleted") page.refresh()
            else if (kind === "pinset") { pinField.text = ""; pinAgain.text = ""; page.lockLocal(); page.notice = "PIN set; unlock with it"; page.refresh() }
            else if (kind === "unlock") {
                const u = JSON.parse(json)
                const seconds = u.expires_in_seconds || 300
                page.token = u.token; page.unlocked = true; unlockField.text = ""
                expiry.interval = Math.max(1000, seconds * 1000); expiry.restart()
                page.notice = "unlocked for " + Math.max(1, Math.round(seconds / 60)) + " min"
            }
            else if (kind === "reveal") { const r = JSON.parse(json); page.revealedKey = r.key; page.revealedValue = r.value }
            else if (kind === "update") { page.notice = "updated"; page.revealedKey = ""; page.revealedValue = "" }
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    Dialog {
        id: confirmDelete
        title: "Delete secret"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        property string key: ""
        onAccepted: page.remove(key)
        Label { text: "Remove \"" + confirmDelete.key + "\" from the vault?"; width: 360; wrapMode: Text.WordWrap }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 12
        Text { text: "Secrets"; color: page.theme.foreground; font.pixelSize: Math.round(22 * page.theme.scale); font.bold: true }
        Text {
            text: "Keys for providers and services. A value is written once and shown only behind the PIN; set it again to replace it. `openai_api_key` (or `OPENAI_API_KEY`) is the one the embeddings read. The PIN protects this screen: the file stays owner-readable for the back end and the agents. Never type the PIN to an agent."
            color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(13 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true
        }

        // The lock: set a PIN, unlock with it, lock again, change it.
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: lockColumn.implicitHeight + 24
            radius: page.theme.radius
            color: page.theme.panel2
            border.width: 1
            border.color: page.unlocked ? page.theme.alive : page.theme.line
            ColumnLayout {
                id: lockColumn
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8
                RowLayout {
                    spacing: 10
                    Text {
                        text: !page.pinSet ? "No PIN yet" : page.unlocked ? "Unlocked" : (page.lockedOutSeconds > 0 ? "Locked out for " + page.lockedOutSeconds + " s" : "Locked")
                        color: page.unlocked ? page.theme.alive : page.theme.foreground
                        font.pixelSize: Math.round(13 * page.theme.scale); font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Button { visible: page.unlocked; text: page.changingPin ? "Keep PIN" : "Change PIN"; flat: true; onClicked: page.changingPin = !page.changingPin }
                    Button { visible: page.unlocked; text: "Lock"; onClicked: page.lock() }
                }
                // Unlock.
                RowLayout {
                    visible: page.pinSet && !page.unlocked
                    spacing: 8
                    TextField { id: unlockField; Layout.preferredWidth: 220; placeholderText: "PIN"; echoMode: TextInput.Password; enabled: page.lockedOutSeconds === 0; onAccepted: page.unlock(text) }
                    Button { text: "Unlock"; highlighted: true; enabled: page.lockedOutSeconds === 0; onClicked: page.unlock(unlockField.text) }
                    Text { text: "reveals, edits and copies values for a few minutes"; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
                }
                // Set or change.
                RowLayout {
                    visible: !page.pinSet || page.changingPin
                    spacing: 8
                    TextField { id: pinField; Layout.preferredWidth: 220; placeholderText: page.pinSet ? "new PIN (6+ characters)" : "PIN (6+ characters)"; echoMode: TextInput.Password }
                    TextField { id: pinAgain; Layout.preferredWidth: 220; placeholderText: "again"; echoMode: TextInput.Password; onAccepted: page.setPin(pinField.text, text) }
                    Button { text: page.pinSet ? "Change PIN" : "Set PIN"; highlighted: !page.pinSet; onClicked: page.setPin(pinField.text, pinAgain.text) }
                    Text { visible: !page.pinSet; text: "digits or a passphrase; it protects this screen only"; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            TextField { id: keyField; Layout.preferredWidth: 260; placeholderText: "key, e.g. openai_api_key" }
            TextField { id: valueField; Layout.fillWidth: true; placeholderText: "value"; echoMode: TextInput.Password; onAccepted: page.set(keyField.text, text) }
            Button { text: "Set"; highlighted: true; onClicked: page.set(keyField.text, valueField.text) }
        }
        Text { text: page.names.length + " in the vault"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: page.names
            spacing: 2
            delegate: Rectangle {
                id: row
                required property string modelData
                readonly property bool shown: page.revealedKey === modelData
                width: list.width
                height: rowColumn.implicitHeight + 12
                radius: 6
                color: hover.hovered ? Qt.rgba(1, 1, 1, 0.05) : "transparent"
                ColumnLayout {
                    id: rowColumn
                    anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                    anchors.leftMargin: 10; anchors.rightMargin: 6; anchors.topMargin: 6
                    spacing: 4
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: row.modelData; color: page.theme.foreground; font.pixelSize: Math.round(14 * page.theme.scale); font.family: page.theme.termFont; Layout.fillWidth: true; elide: Text.ElideRight }
                        Text { visible: !row.shown; text: "••••••••"; color: page.theme.foreground; opacity: 0.4; font.pixelSize: Math.round(12 * page.theme.scale) }
                        Button { visible: page.unlocked; text: row.shown ? "Hide" : "Reveal"; flat: true; onClicked: page.reveal(row.modelData) }
                        Button { visible: row.shown; text: "Copy"; flat: true; onClicked: page.copyValue(page.revealedValue) }
                        Button { text: "Replace"; flat: true; onClicked: { keyField.text = row.modelData; valueField.forceActiveFocus() } }
                        Button { text: "Delete"; flat: true; onClicked: { confirmDelete.key = row.modelData; confirmDelete.open() } }
                    }
                    // The value, one row at a time, and an edit saved on Enter.
                    RowLayout {
                        visible: row.shown
                        Layout.fillWidth: true
                        spacing: 8
                        TextEdit { text: page.revealedValue; readOnly: true; selectByMouse: true; color: page.theme.accent; font.pixelSize: Math.round(13 * page.theme.scale); font.family: page.theme.termFont; wrapMode: Text.WrapAnywhere; Layout.fillWidth: true }
                        TextField { Layout.preferredWidth: 260; placeholderText: "new value, Enter saves"; echoMode: TextInput.Password; onAccepted: { page.update(row.modelData, text); text = "" } }
                    }
                }
                HoverHandler { id: hover }
            }
            Text { anchors.centerIn: parent; visible: page.names.length === 0; text: page.backend.connected ? "The vault is empty" : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(13 * page.theme.scale) }
        }
        Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
    }
}
