import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Settings tab. Two halves: what this machine gives the app (theme, font, scheme,
// tabs file, back end), and the settings rusty-mcp stores, editable in place.
Item {
    id: page
    required property var backend
    required property var theme
    required property var terminals

    property var settings: []
    property string notice: ""

    // Every setting the back end reads, with what it means and what it defaults to.
    readonly property var known: [
        { key: "brain_vault_path", about: "The brain vault folder (an Obsidian vault). Restart the service after changing it.", fallback: "~/.rusty/brain" },
        { key: "notes_path", about: "The notes folder the notes tools use.", fallback: "~/.rusty/notes" },
        { key: "embedding_provider", about: "auto (Ollama when it answers), ollama, openai (needs openai_api_key in Secrets), or off.", fallback: "auto" },
        { key: "embedding_model", about: "Overrides the provider's default model (nomic-embed-text, text-embedding-3-small).", fallback: "provider default" },
        { key: "ollama_url", about: "Where Ollama listens.", fallback: "http://127.0.0.1:11434" },
        { key: "skills_enabled", about: "Whether the skills store is served to agents.", fallback: "true" },
        { key: "skills_path", about: "Where skills live (active/ and staging/).", fallback: "~/.rusty/skills" },
        { key: "brain_auto_enrich", about: "Enrich captured pages automatically.", fallback: "false" },
        { key: "default_workflow", about: "The default agent workflow name.", fallback: "deep" }
    ]
    function storedValue(key) { const e = settings.find(s => s.key === key); return e ? e.value : "" }
    function others() { const names = known.map(k => k.key); return settings.filter(s => names.indexOf(s.key) < 0) }

    function refresh() { backend.call("settings_list", "{}") }
    function setValue(key, value) {
        if (key.trim().length === 0) return
        backend.call("settings_set", JSON.stringify({ key: key.trim(), value: value }))
    }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            if (!tool.startsWith("settings_")) return
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            if (tool === "settings_list") page.settings = JSON.parse(json)
            else page.refresh()
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    Flickable {
        anchors.fill: parent
        anchors.margins: 32
        contentHeight: column.implicitHeight
        clip: true
        ColumnLayout {
            id: column
            width: parent.width
            spacing: 12

            Text { text: "Settings"; color: page.theme.foreground; font.pixelSize: 22; font.bold: true }

            Text { text: "This machine"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 8 }
            Text {
                text: page.theme.facts
                    + "\ntabs file: " + page.terminals.tabsPath
                    + "\nback end: " + page.backend.url + " (" + page.backend.status + ")"
                color: page.theme.foreground; opacity: 0.75; font.pixelSize: 13
                wrapMode: Text.WrapAnywhere; Layout.fillWidth: true
            }
            Text {
                text: "Ctrl+Shift+T new terminal · Ctrl+Shift+W close tab · F2 rename · Ctrl+PgUp/PgDn switch tabs"
                color: page.theme.foreground; opacity: 0.5; font.pixelSize: 12
            }
            Button { text: "Re-read theme"; onClicked: page.theme.reload() }

            Text { text: "Settings rusty-mcp reads"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 16 }
            Text { visible: !page.backend.connected; text: page.backend.status; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 13 }
            Repeater {
                model: page.known
                delegate: ColumnLayout {
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 2
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 12
                        Text { text: modelData.key; color: page.theme.foreground; font.pixelSize: 14; font.family: page.theme.termFont; Layout.preferredWidth: 220; elide: Text.ElideRight }
                        TextField {
                            id: knownField
                            Layout.preferredWidth: 420
                            text: page.storedValue(modelData.key)
                            placeholderText: modelData.fallback
                            onAccepted: if (text !== page.storedValue(modelData.key)) page.setValue(modelData.key, text)
                        }
                        Text { text: knownField.text !== page.storedValue(modelData.key) ? "Enter saves" : (page.storedValue(modelData.key).length === 0 ? "default" : ""); color: page.theme.accent; font.pixelSize: 11 }
                    }
                    Text { text: modelData.about; color: page.theme.foreground; opacity: 0.55; font.pixelSize: 11; Layout.leftMargin: 232; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                }
            }

            Text { visible: page.others().length > 0; text: "Other stored keys"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 12 }
            Repeater {
                model: page.others()
                delegate: RowLayout {
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 12
                    Text { text: modelData.key; color: page.theme.foreground; font.pixelSize: 14; font.family: page.theme.termFont; Layout.preferredWidth: 220; elide: Text.ElideRight }
                    TextField {
                        id: valueField
                        Layout.preferredWidth: 420
                        text: modelData.value
                        placeholderText: modelData.value === "•••" ? "hidden; type a new value to replace it" : ""
                        onAccepted: if (text !== modelData.value && text !== "•••") page.setValue(modelData.key, text)
                    }
                    Text { text: valueField.text !== modelData.value ? "Enter saves" : ""; color: page.theme.accent; font.pixelSize: 11 }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 8
                spacing: 12
                TextField { id: newKey; Layout.preferredWidth: 220; placeholderText: "new key" }
                TextField { id: newValue; Layout.preferredWidth: 420; placeholderText: "value"; onAccepted: { page.setValue(newKey.text, text); newKey.text = ""; text = "" } }
                Button { text: "Set"; onClicked: { page.setValue(newKey.text, newValue.text); newKey.text = ""; newValue.text = "" } }
            }
            Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
        }
    }
}
