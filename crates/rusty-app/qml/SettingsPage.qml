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

            Text { text: "Stored by rusty-mcp"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 16 }
            Text {
                visible: page.settings.length === 0
                text: page.backend.connected ? "No settings stored yet." : page.backend.status
                color: page.theme.foreground; opacity: 0.6; font.pixelSize: 13
            }
            Repeater {
                model: page.settings
                delegate: RowLayout {
                    required property var modelData
                    Layout.fillWidth: true
                    spacing: 12
                    Text { text: modelData.key; color: page.theme.foreground; font.pixelSize: 14; Layout.preferredWidth: 220; elide: Text.ElideRight }
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
