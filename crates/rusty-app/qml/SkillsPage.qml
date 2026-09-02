import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Skills tab: the Agent Skills Rusty ships to its agents. Active skills are live;
// pending ones wait in staging until approved. The editor edits description and body in
// place, runs the safety scan, approves, rejects, deletes.
Item {
    id: page
    required property var backend
    required property var theme

    property var skills: []
    property var selected: null
    property var findings: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("skill_list", { include_pending: true }, "list") }
    function select(s) {
        selected = s
        findings = []
        if (s) { descField.text = s.description; bodyArea.text = s.body }
    }
    function save() {
        if (!selected) return
        ask("skill_update", { name: selected.name, description: descField.text, body: bodyArea.text }, "saved")
    }
    function scan() { if (selected) ask("skill_scan", { name: selected.name }, "scan") }
    function approve(force) { if (selected) ask("skill_approve", { name: selected.name, force: force }, "approved") }
    function reject() { if (selected) ask("skill_reject", { name: selected.name }, "rejected") }
    function remove() { if (selected) ask("skill_delete", { name: selected.name }, "deleted") }
    function create(name, description, body, staged) {
        if (name.trim().length === 0) { notice = "a skill needs a name (lowercase, digits, hyphens)"; return }
        ask("skill_create", { name: name.trim(), description: description.trim(), body: body, pending: staged, force: false }, "created")
    }
    function focusEntry() { if (!selected) newName.forceActiveFocus() }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            switch (kind) {
            case "list": {
                const list = JSON.parse(json)
                list.sort((a, b) => (a.status === b.status ? a.name.localeCompare(b.name) : (a.status === "pending" ? -1 : 1)))
                page.skills = list
                if (page.selected) {
                    const again = list.find(s => s.name === page.selected.name)
                    if (again) { page.selected = again } else page.select(null)
                }
                break
            }
            case "saved": {
                const r = JSON.parse(json)
                page.findings = r.findings || []
                page.notice = page.findings.length > 0 ? "saved; the scan has findings" : "saved"
                page.refresh(); break
            }
            case "scan": {
                page.findings = JSON.parse(json)
                page.notice = page.findings.length === 0 ? "scan clean" : ""
                break
            }
            case "created": { newName.text = ""; newDesc.text = ""; newBody.text = ""; newDialog.close(); page.refresh(); break }
            case "approved": case "rejected": case "deleted": { if (kind !== "approved") page.select(null); page.refresh(); break }
            }
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    Dialog {
        id: confirmDelete
        title: "Delete skill"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: page.remove()
        Label { text: "Delete \"" + (page.selected ? page.selected.name : "") + "\" from disk?"; width: 360; wrapMode: Text.WordWrap }
    }

    Dialog {
        id: newDialog
        title: "New skill"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: page.create(newName.text, newDesc.text, newBody.text, newStaged.checked)
        ColumnLayout {
            spacing: 8
            Label { text: "Name (directory and invocation name)" }
            TextField { id: newName; Layout.preferredWidth: 480; placeholderText: "tidy-commits"; onAccepted: newDialog.accept() }
            Label { text: "Description (when the agent should use it)" }
            TextField { id: newDesc; Layout.preferredWidth: 480; onAccepted: newDialog.accept() }
            Label { text: "Body (markdown)" }
            TextArea { id: newBody; Layout.preferredWidth: 480; Layout.preferredHeight: 200; wrapMode: TextEdit.Wrap; text: "# Skill\n\nSteps the agent follows.\n" }
            CheckBox { id: newStaged; text: "stage for approval instead of activating"; checked: false }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 300
            Layout.fillHeight: true
            color: Qt.darker(page.theme.background, 1.08)
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 6
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: page.skills.length + " skills"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.fillWidth: true }
                    Button { text: "New"; onClicked: { newDialog.open(); newName.forceActiveFocus() } }
                }
                ListView {
                    id: list
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.skills
                    spacing: 2
                    ScrollBar.vertical: ScrollBar {}
                    delegate: Rectangle {
                        required property var modelData
                        width: list.width
                        height: 44
                        radius: 6
                        color: page.selected && page.selected.name === modelData.name ? page.theme.accent : (hover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
                        ColumnLayout {
                            anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 8; anchors.topMargin: 5; anchors.bottomMargin: 5
                            spacing: 1
                            RowLayout {
                                Text { text: modelData.name; font.pixelSize: 14; elide: Text.ElideRight; Layout.fillWidth: true; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                                Rectangle { visible: modelData.status === "pending"; radius: 3; color: page.theme.foreground; opacity: 0.85; width: pendingText.implicitWidth + 8; height: 16; Text { id: pendingText; anchors.centerIn: parent; text: "pending"; color: page.theme.background; font.pixelSize: 10 } }
                                Text { visible: modelData.origin === "auto"; text: "auto"; font.pixelSize: 10; opacity: 0.7; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                            }
                            Text { text: modelData.description; font.pixelSize: 11; opacity: 0.7; elide: Text.ElideRight; Layout.fillWidth: true; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                        }
                        HoverHandler { id: hover }
                        TapHandler { onTapped: page.select(modelData) }
                    }
                }
                Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: page.theme.accent; opacity: 0.25 }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Text { anchors.centerIn: parent; visible: page.selected === null; text: page.backend.connected ? "Pick a skill, or make a new one" : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: 14 }
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 8
                visible: page.selected !== null
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Text { text: page.selected ? page.selected.name : ""; color: page.theme.foreground; font.pixelSize: 22; font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight }
                    Button { text: "Scan"; onClicked: page.scan() }
                    Button { text: "Approve"; visible: page.selected && page.selected.status === "pending"; highlighted: true; onClicked: page.approve(false) }
                    Button { text: "Approve anyway"; visible: page.selected && page.selected.status === "pending" && page.findings.length > 0; onClicked: page.approve(true) }
                    Button { text: "Reject"; visible: page.selected && page.selected.status === "pending"; onClicked: page.reject() }
                    Button { text: "Save"; highlighted: true; onClicked: page.save() }
                    Button { text: "Delete"; onClicked: confirmDelete.open() }
                }
                Text {
                    text: page.selected ? (page.selected.status + "  ·  " + page.selected.origin + "  ·  " + page.selected.path) : ""
                    color: page.theme.foreground; opacity: 0.55; font.pixelSize: 12; elide: Text.ElideMiddle; Layout.fillWidth: true
                }
                Text { text: "Description"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true }
                TextField { id: descField; Layout.fillWidth: true }
                Text { text: "Body"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true }
                TextArea {
                    id: bodyArea
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    wrapMode: TextEdit.Wrap
                    font.family: page.theme.termFont
                    font.pointSize: 10.5
                    color: page.theme.foreground
                    background: Rectangle { color: Qt.darker(page.theme.background, 1.15); radius: 6; border.color: page.theme.accent; border.width: 1 }
                    Keys.onPressed: (event) => { if (event.key === Qt.Key_S && (event.modifiers & Qt.ControlModifier)) { page.save(); event.accepted = true } }
                }
                ColumnLayout {
                    visible: page.findings.length > 0
                    spacing: 2
                    Text { text: "Scan findings"; color: page.theme.accent; font.pixelSize: 12; font.bold: true }
                    Repeater {
                        model: page.findings
                        delegate: Text { required property string modelData; text: "• " + modelData; color: page.theme.foreground; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
                    }
                }
            }
        }
    }
}
