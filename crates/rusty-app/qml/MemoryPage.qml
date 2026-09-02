import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Memory tab: what Rusty remembers about you. Add a line at the top, pick a row to
// edit or delete it, filter by category.
Item {
    id: page
    required property var backend
    required property var theme

    property var memories: []
    property var categories: []
    property string filter: ""
    property var selected: null
    property string notice: ""
    property var pending: ({})

    readonly property var importances: ["low", "normal", "high"]

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("list_memories", {}, "list") }
    function add(content, category, importance) {
        if (content.trim().length === 0) return
        ask("store_memory", { content: content.trim(), category: category.trim().length > 0 ? category.trim() : "context", importance: importance }, "stored")
    }
    function update(m, content, category, importance) {
        ask("update_memory", { id: m.id, content: content.trim(), category: category.trim(), importance: importance }, "updated")
    }
    function remove(m) { ask("delete_memory", { id: m.id }, "deleted") }
    function focusEntry() { addField.forceActiveFocus() }
    function shown() { return filter.length > 0 ? memories.filter(m => m.category === filter) : memories }

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
                list.sort((a, b) => b.updated_at - a.updated_at)
                page.memories = list
                const cats = {}
                for (const m of list) cats[m.category] = true
                page.categories = Object.keys(cats).sort()
                if (page.selected) page.selected = list.find(m => m.id === page.selected.id) || null
            } else if (kind === "stored") { addField.text = ""; page.refresh() }
            else if (kind === "updated" || kind === "deleted") { if (kind === "deleted") page.selected = null; page.refresh() }
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    Dialog {
        id: confirmDelete
        title: "Delete memory"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: if (page.selected) page.remove(page.selected)
        Label { text: "Forget this?\n\n" + (page.selected ? page.selected.content : ""); wrapMode: Text.WordWrap; width: 420 }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            TextField { id: addField; Layout.fillWidth: true; placeholderText: "Remember something and press Enter"; onAccepted: page.add(text, addCategory.editText, addImportance.currentText) }
            // "context" stays first so a fresh line lands there unless another category is picked.
            ComboBox { id: addCategory; editable: true; model: ["context"].concat(page.categories.filter(c => c !== "context")); Layout.preferredWidth: 150 }
            ComboBox { id: addImportance; model: page.importances; currentIndex: 1; Layout.preferredWidth: 100 }
        }
        RowLayout {
            Layout.fillWidth: true
            Text { text: page.memories.length + " memories"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; Layout.fillWidth: true }
            Text { text: "filter"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12 }
            ComboBox { model: ["all"].concat(page.categories); Layout.preferredWidth: 150; onActivated: (i) => page.filter = i === 0 ? "" : currentText }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 12
            ListView {
                id: list
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: page.shown()
                spacing: 3
                ScrollBar.vertical: ScrollBar {}
                delegate: Rectangle {
                    required property var modelData
                    width: list.width
                    height: rowCol.implicitHeight + 14
                    radius: 6
                    color: page.selected && page.selected.id === modelData.id ? Qt.rgba(1, 1, 1, 0.1) : (rowHover.hovered ? Qt.rgba(1, 1, 1, 0.04) : "transparent")
                    ColumnLayout {
                        id: rowCol
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 7
                        spacing: 3
                        Text { text: modelData.content; color: page.theme.foreground; font.pixelSize: 14; wrapMode: Text.WordWrap; Layout.fillWidth: true; maximumLineCount: 3; elide: Text.ElideRight }
                        RowLayout {
                            spacing: 8
                            Rectangle { radius: 3; color: page.theme.accent; opacity: 0.85; width: catText.implicitWidth + 10; height: 18; Text { id: catText; anchors.centerIn: parent; text: modelData.category; color: page.theme.background; font.pixelSize: 11 } }
                            Text { text: modelData.importance; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 11 }
                            Text { text: modelData.source; color: page.theme.foreground; opacity: 0.4; font.pixelSize: 11 }
                            Text { text: new Date(modelData.updated_at * 1000).toLocaleDateString(Qt.locale(), Locale.ShortFormat); color: page.theme.foreground; opacity: 0.4; font.pixelSize: 11 }
                        }
                    }
                    HoverHandler { id: rowHover }
                    TapHandler { onTapped: { page.selected = modelData; editContent.text = modelData.content; editCategory.editText = modelData.category; editImportance.currentIndex = Math.max(0, page.importances.indexOf(modelData.importance)) } }
                }
                Text { anchors.centerIn: parent; visible: page.shown().length === 0; text: page.backend.connected ? "Nothing remembered yet" : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: 14 }
            }

            // Editor for the selected memory
            Rectangle {
                Layout.preferredWidth: 380
                Layout.fillHeight: true
                radius: 8
                color: Qt.darker(page.theme.background, 1.1)
                visible: page.selected !== null
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8
                    Text { text: "Edit"; color: page.theme.foreground; font.pixelSize: 14; font.bold: true }
                    TextArea {
                        id: editContent
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        wrapMode: TextEdit.Wrap
                        color: page.theme.foreground
                        background: Rectangle { color: Qt.darker(page.theme.background, 1.2); radius: 6 }
                    }
                    RowLayout {
                        spacing: 8
                        ComboBox { id: editCategory; editable: true; model: page.categories; Layout.fillWidth: true }
                        ComboBox { id: editImportance; model: page.importances; Layout.preferredWidth: 100 }
                    }
                    RowLayout {
                        spacing: 8
                        Button { text: "Save"; highlighted: true; onClicked: page.update(page.selected, editContent.text, editCategory.editText, editImportance.currentText) }
                        Button { text: "Delete"; onClicked: confirmDelete.open() }
                        Item { Layout.fillWidth: true }
                        Button { text: "Close"; flat: true; onClicked: page.selected = null }
                    }
                }
            }
        }
        Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
    }
}
