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
    // Scripts as commands (TICKET-010): a `*.sh` beside a skill; view, edit, run.
    property var scripts: []
    property var selectedScript: null
    signal runScript(string path, string name)
    property var findings: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("skill_list", { include_pending: true }, "list"); ask("script_list", { include_pending: true }, "scripts") }
    function select(s) {
        selected = s
        selectedScript = null
        findings = []
        if (s) { descField.text = s.description; bodyArea.text = s.body }
    }
    function selectScript(s) {
        selected = null
        selectedScript = s
        findings = []
        bodyArea.text = ""
        if (s) ask("script_view", { name: s.skill + "/" + s.name }, "script")
    }
    function save() {
        if (selectedScript) { ask("script_update", { name: selectedScript.skill + "/" + selectedScript.name, body: bodyArea.text }, "saved"); return }
        if (!selected) return
        ask("skill_update", { name: selected.name, description: descField.text, body: bodyArea.text }, "saved")
    }
    function run() { if (selectedScript) runScript(selectedScript.path, selectedScript.name) }
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
            case "scripts": {
                page.scripts = JSON.parse(json)
                break
            }
            case "script": {
                const r = JSON.parse(json)
                if (page.selectedScript && r.script && r.script.path === page.selectedScript.path) bodyArea.text = r.text
                break
            }
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
                    Text { text: page.skills.length + " skills"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true; Layout.fillWidth: true }
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
                                Text { text: modelData.name; font.pixelSize: Math.round(14 * page.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                                Rectangle { visible: modelData.status === "pending"; radius: 3; color: page.theme.foreground; opacity: 0.85; width: pendingText.implicitWidth + 8; height: 16; Text { id: pendingText; anchors.centerIn: parent; text: "pending"; color: page.theme.background; font.pixelSize: Math.round(10 * page.theme.scale) } }
                                Text { visible: modelData.origin === "auto"; text: "auto"; font.pixelSize: Math.round(10 * page.theme.scale); opacity: 0.7; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                            }
                            Text { text: modelData.description; font.pixelSize: Math.round(11 * page.theme.scale); opacity: 0.7; elide: Text.ElideRight; Layout.fillWidth: true; color: page.selected && page.selected.name === modelData.name ? page.theme.background : page.theme.foreground }
                        }
                        HoverHandler { id: hover }
                        TapHandler { onTapped: page.select(modelData) }
                    }
                }
                // Scripts: a `*.sh` beside a skill, run as `rusty <name>`.
                Text { visible: page.scripts.length > 0; text: "Scripts"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true; Layout.topMargin: 6 }
                ListView {
                    id: scriptList
                    visible: page.scripts.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.min(Math.round(180 * page.theme.scale), page.scripts.length * Math.round(36 * page.theme.scale))
                    clip: true
                    model: page.scripts
                    spacing: 2
                    delegate: Rectangle {
                        required property var modelData
                        width: scriptList.width
                        height: Math.round(34 * page.theme.scale)
                        radius: 6
                        color: page.selectedScript && page.selectedScript.path === modelData.path ? page.theme.accent : (scriptHover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
                        RowLayout {
                            anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 8
                            spacing: 6
                            Text { text: "$"; color: page.selectedScript && page.selectedScript.path === modelData.path ? page.theme.background : page.theme.gold; font.pixelSize: Math.round(12 * page.theme.scale); font.family: page.theme.termFont }
                            Text { text: modelData.name; font.pixelSize: Math.round(13 * page.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true; color: page.selectedScript && page.selectedScript.path === modelData.path ? page.theme.background : page.theme.foreground }
                            Text { text: modelData.skill; font.pixelSize: Math.round(10 * page.theme.scale); opacity: 0.7; color: page.selectedScript && page.selectedScript.path === modelData.path ? page.theme.background : page.theme.foreground }
                            Text { visible: modelData.status === "pending"; text: "pending"; font.pixelSize: Math.round(10 * page.theme.scale); color: page.theme.gold }
                        }
                        HoverHandler { id: scriptHover }
                        TapHandler { onTapped: page.selectScript(modelData) }
                    }
                }
                Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: Math.round(11 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: page.theme.accent; opacity: 0.25 }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Text { anchors.centerIn: parent; visible: page.selected === null && page.selectedScript === null; text: page.backend.connected ? "Pick a skill, or make a new one" : page.backend.status; color: page.theme.foreground; opacity: 0.5; font.pixelSize: Math.round(14 * page.theme.scale) }
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 8
                visible: page.selected !== null || page.selectedScript !== null
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Text { text: page.selectedScript ? "$ rusty " + page.selectedScript.name : (page.selected ? page.selected.name : ""); color: page.theme.foreground; font.pixelSize: Math.round(22 * page.theme.scale); font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight }
                    Button { text: "Run"; visible: page.selectedScript !== null; enabled: page.selectedScript !== null && page.selectedScript.status === "active"; highlighted: true; onClicked: page.run() }
                    Button { text: "Scan"; visible: page.selected !== null; onClicked: page.scan() }
                    Button { text: "Approve"; visible: page.selected && page.selected.status === "pending"; highlighted: true; onClicked: page.approve(false) }
                    Button { text: "Approve anyway"; visible: page.selected && page.selected.status === "pending" && page.findings.length > 0; onClicked: page.approve(true) }
                    Button { text: "Reject"; visible: page.selected && page.selected.status === "pending"; onClicked: page.reject() }
                    Button { text: "Save"; highlighted: true; onClicked: page.save() }
                    Button { text: "Delete"; visible: page.selected !== null; onClicked: confirmDelete.open() }
                }
                Text {
                    text: page.selectedScript ? (page.selectedScript.status + "  ·  " + page.selectedScript.skill + "  ·  " + page.selectedScript.path + (page.selectedScript.status === "pending" ? "  ·  approve the skill before it runs" : "")) : page.selected ? (page.selected.status + "  ·  " + page.selected.origin + "  ·  " + page.selected.path) : ""
                    color: page.theme.foreground; opacity: 0.55; font.pixelSize: Math.round(12 * page.theme.scale); elide: Text.ElideMiddle; Layout.fillWidth: true
                }
                Text { visible: page.selected !== null; text: "Description"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true }
                TextField { id: descField; visible: page.selected !== null; Layout.fillWidth: true }
                Text { text: page.selectedScript ? "Script" : "Body"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true }
                TextArea {
                    id: bodyArea
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    wrapMode: TextEdit.Wrap
                    font.family: page.theme.termFont
                    font.pointSize: 10.5 * page.theme.scale
                    color: page.theme.foreground
                    background: Rectangle { color: Qt.darker(page.theme.background, 1.15); radius: 6; border.color: page.theme.accent; border.width: 1 }
                    Keys.onPressed: (event) => { if (event.key === Qt.Key_S && (event.modifiers & Qt.ControlModifier)) { page.save(); event.accepted = true } }
                }
                ColumnLayout {
                    visible: page.findings.length > 0
                    spacing: 2
                    Text { text: "Scan findings"; color: page.theme.accent; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true }
                    Repeater {
                        model: page.findings
                        delegate: Text { required property string modelData; text: "• " + modelData; color: page.theme.foreground; font.pixelSize: Math.round(12 * page.theme.scale); wrapMode: Text.WordWrap; Layout.fillWidth: true }
                    }
                }
            }
        }
    }
}
