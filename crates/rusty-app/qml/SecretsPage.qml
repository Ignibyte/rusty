import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Secrets tab: names in the vault, set or replace a value, delete. A value is typed
// once and never shown again; the server never returns one.
Item {
    id: page
    required property var backend
    required property var theme

    property var names: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("secret_list", {}, "list") }
    function set(key, value) {
        if (key.trim().length === 0 || value.length === 0) { notice = "a key and a value are both needed"; return }
        ask("secret_set", { key: key.trim(), value: value }, "set")
    }
    function remove(key) { ask("secret_delete", { key: key }, "deleted") }
    function focusEntry() { keyField.forceActiveFocus() }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            if (kind === "list") {
                const list = JSON.parse(json)
                page.names = list.map(e => typeof e === "string" ? e : (e.key || e.name || JSON.stringify(e))).sort()
            } else if (kind === "set") { keyField.text = ""; valueField.text = ""; page.notice = "stored"; page.refresh() }
            else if (kind === "deleted") page.refresh()
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
        Text { text: "Keys for providers and services. A value is written once and never displayed; set it again to replace it. `openai_api_key` (or `OPENAI_API_KEY`) is what OpenAI embeddings use when `embedding_provider` is `openai`."; color: page.theme.foreground; opacity: 0.7; font.pixelSize: Math.round(13 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            TextField { id: keyField; Layout.preferredWidth: 260; placeholderText: "key, e.g. openai_api_key" }
            TextField { id: valueField; Layout.fillWidth: true; placeholderText: "value"; echoMode: TextInput.Password; onAccepted: page.set(keyField.text, text) }
            Button { text: "Set"; highlighted: true; onClicked: page.set(keyField.text, valueField.text) }
        }
        Text { text: page.names.length + " in the vault"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true; Layout.topMargin: 6 }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: page.names
            spacing: 2
            delegate: Rectangle {
                required property string modelData
                width: list.width
                height: 36
                radius: 6
                color: hover.hovered ? Qt.rgba(1, 1, 1, 0.05) : "transparent"
                RowLayout {
                    anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 6
                    Text { text: modelData; color: page.theme.foreground; font.pixelSize: Math.round(14 * page.theme.scale); font.family: page.theme.termFont; Layout.fillWidth: true; elide: Text.ElideRight }
                    Text { text: "••••••••"; color: page.theme.foreground; opacity: 0.4; font.pixelSize: Math.round(12 * page.theme.scale) }
                    Button { text: "Replace"; flat: true; onClicked: { keyField.text = modelData; valueField.forceActiveFocus() } }
                    Button { text: "Delete"; flat: true; onClicked: { confirmDelete.key = modelData; confirmDelete.open() } }
                }
                HoverHandler { id: hover }
            }
            Text { anchors.centerIn: parent; visible: page.names.length === 0; text: page.backend.connected ? "The vault is empty" : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(14 * page.theme.scale) }
        }
        Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
    }
}
