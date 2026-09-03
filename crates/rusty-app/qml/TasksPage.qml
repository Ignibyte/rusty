import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Tasks tab: lists on the left, the chosen list's tasks on the right. Everything
// goes through the back end's task tools; the page keeps no state the server does not.
// Keyboard first: type in the add box and press Enter, arrows move, Space toggles,
// F2 renames, Delete archives, Ctrl+Up/Down reorder (or drag the handle), Escape returns to
// the add box.
Item {
    id: page
    required property var backend
    required property var theme

    property int groupId: -1
    property string groupName: ""
    property bool showArchived: false
    property var groups: []
    property var tasks: []
    property string notice: ""

    function refreshGroups() { backend.call("list_task_groups", "{}") }
    function refreshTasks() {
        if (groupId >= 0)
            backend.call("list_tasks", JSON.stringify({ group_id: groupId, include_archived: showArchived }))
        else
            tasks = []
    }
    function refresh() { refreshGroups(); refreshTasks() }

    function selectGroup(id, name) { groupId = id; groupName = name; refreshTasks() }
    function addTask(title) {
        if (groupId < 0 || title.trim().length === 0) return
        backend.call("create_task", JSON.stringify({ group_id: groupId, title: title.trim() }))
    }
    function toggle(task) { backend.call("toggle_task", JSON.stringify({ id: task.id })) }
    function archiveOrRestore(task) { backend.call(task.archived ? "unarchive_task" : "archive_task", JSON.stringify({ id: task.id })) }
    function rename(task, title) {
        if (title.trim().length === 0 || title.trim() === task.title) return
        backend.call("update_task_title", JSON.stringify({ id: task.id, title: title.trim() }))
    }
    function remove(task) { backend.call("delete_task", JSON.stringify({ id: task.id })) }
    function move(from, to) {
        if (to < 0 || to >= tasks.length || from === to) return
        const ids = tasks.map(t => t.id)
        const [moved] = ids.splice(from, 1)
        ids.splice(to, 0, moved)
        backend.call("reorder_tasks", JSON.stringify({ group_id: groupId, task_ids: ids }))
        taskList.currentIndex = to
    }
    function addGroup(name) {
        if (name.trim().length === 0) return
        backend.call("create_task_group", JSON.stringify({ name: name.trim() }))
    }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            if (!ok) { page.notice = tool + ": " + json; return }
            switch (tool) {
            case "list_task_groups": {
                const list = JSON.parse(json)
                const firstLoad = page.groups.length === 0
                page.groups = list
                if (list.length === 0) { page.groupId = -1; page.groupName = ""; page.tasks = [] }
                else if (!list.some(g => g.id === page.groupId)) page.selectGroup(list[0].id, list[0].name)
                else page.groupName = list.find(g => g.id === page.groupId).name
                if (firstLoad && page.visible) page.focusAdd()
                break
            }
            case "list_tasks": {
                const keep = taskList.currentIndex
                page.tasks = JSON.parse(json)
                taskList.currentIndex = Math.min(keep, page.tasks.length - 1)
                break
            }
            case "create_task_group":
            case "rename_task_group":
            case "delete_task_group":
                page.refreshGroups(); break
            case "create_task":
            case "toggle_task":
            case "archive_task":
            case "unarchive_task":
            case "update_task_title":
            case "delete_task":
            case "reorder_tasks":
                page.refreshTasks(); break
            }
            page.notice = ""
        }
        function onDataChanged() { page.refresh() }
    }
    function focusAdd() { if (addField.enabled) addField.forceActiveFocus(); else taskList.forceActiveFocus() }
    Component.onCompleted: if (backend.connected) refresh()

    Dialog {
        id: confirmGroupDelete
        title: "Delete list"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        property int targetId: -1
        property string targetName: ""
        onAccepted: page.backend.call("delete_task_group", JSON.stringify({ group_id: targetId }))
        Label { text: "Delete the list \"" + confirmGroupDelete.targetName + "\" and every task in it?"; wrapMode: Text.WordWrap; width: 360 }
    }

    Dialog {
        id: renameGroupDialog
        title: "Rename list"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel
        property int targetId: -1
        function openFor(g) { targetId = g.id; groupNameField.text = g.name; open(); groupNameField.forceActiveFocus(); groupNameField.selectAll() }
        onAccepted: if (groupNameField.text.trim().length > 0) page.backend.call("rename_task_group", JSON.stringify({ group_id: targetId, name: groupNameField.text.trim() }))
        TextField { id: groupNameField; width: 320; onAccepted: renameGroupDialog.accept() }
    }

    Menu {
        id: groupMenu
        property var group: null
        MenuItem { text: "Rename…"; onTriggered: renameGroupDialog.openFor(groupMenu.group) }
        MenuItem { text: "Delete list…"; onTriggered: { confirmGroupDelete.targetId = groupMenu.group.id; confirmGroupDelete.targetName = groupMenu.group.name; confirmGroupDelete.open() } }
    }

    Menu {
        id: taskMenu
        property var task: null
        property int row: -1
        MenuItem { text: "Rename (F2)"; onTriggered: taskList.startEdit(taskMenu.row) }
        MenuItem { text: taskMenu.task && taskMenu.task.archived ? "Restore" : "Archive (Delete)"; onTriggered: page.archiveOrRestore(taskMenu.task) }
        MenuItem { text: "Delete for good"; onTriggered: page.remove(taskMenu.task) }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Lists
        Rectangle {
            Layout.preferredWidth: 240
            Layout.fillHeight: true
            color: Qt.darker(page.theme.background, 1.08)
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 6
                Text { text: "Lists"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(12 * page.theme.scale); font.bold: true }
                ListView {
                    id: groupList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.groups
                    spacing: 2
                    delegate: Rectangle {
                        required property var modelData
                        width: groupList.width
                        height: 32
                        radius: 6
                        color: modelData.id === page.groupId ? page.theme.accent : (groupHover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
                        Text {
                            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 10; anchors.right: parent.right; anchors.rightMargin: 8
                            text: modelData.name
                            elide: Text.ElideRight
                            color: modelData.id === page.groupId ? page.theme.background : page.theme.foreground
                            font.pixelSize: Math.round(14 * page.theme.scale)
                        }
                        HoverHandler { id: groupHover }
                        TapHandler { acceptedButtons: Qt.LeftButton; onTapped: page.selectGroup(modelData.id, modelData.name) }
                        TapHandler { acceptedButtons: Qt.RightButton; onTapped: { groupMenu.group = modelData; groupMenu.popup() } }
                    }
                }
                TextField {
                    id: newGroupField
                    Layout.fillWidth: true
                    placeholderText: "+ new list"
                    onAccepted: { page.addGroup(text); text = "" }
                }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: page.theme.accent; opacity: 0.25 }

        // Tasks of the chosen list
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: 16
                Layout.bottomMargin: 0
                Text { text: page.groupName.length > 0 ? page.groupName : "No list yet"; color: page.theme.foreground; font.pixelSize: Math.round(22 * page.theme.scale); font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight }
                CheckBox { text: "show archived"; checked: page.showArchived; onToggled: { page.showArchived = checked; page.refreshTasks() } }
            }
            TextField {
                id: addField
                Layout.fillWidth: true
                Layout.leftMargin: 16
                Layout.rightMargin: 16
                enabled: page.groupId >= 0
                placeholderText: page.groupId >= 0 ? "Add a task and press Enter" : "Create a list first"
                onAccepted: { page.addTask(text); text = "" }
                Keys.onEscapePressed: taskList.forceActiveFocus()
                Keys.onDownPressed: { if (taskList.currentIndex < 0 && page.tasks.length > 0) taskList.currentIndex = 0; taskList.forceActiveFocus() }
            }
            ListView {
                id: taskList
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: 16
                Layout.rightMargin: 16
                clip: true
                model: page.tasks
                spacing: 2
                focus: true
                keyNavigationEnabled: true
                onActiveFocusChanged: if (activeFocus && currentIndex < 0 && page.tasks.length > 0) currentIndex = 0
                property int editingRow: -1
                property int dragFrom: -1
                function startEdit(row) { if (row >= 0 && row < page.tasks.length) { currentIndex = row; editingRow = row } }
                function dropIndex(contentY) {
                    let to = indexAt(4, contentY)
                    if (to < 0) to = contentY < 0 ? 0 : page.tasks.length - 1
                    return to
                }
                Keys.onPressed: (event) => {
                    if (currentIndex < 0 || currentIndex >= page.tasks.length) { if (event.key === Qt.Key_Escape) addField.forceActiveFocus(); return }
                    const task = page.tasks[currentIndex]
                    if (event.key === Qt.Key_Space) { page.toggle(task); event.accepted = true }
                    else if (event.key === Qt.Key_F2) { startEdit(currentIndex); event.accepted = true }
                    else if (event.key === Qt.Key_Delete) { page.archiveOrRestore(task); event.accepted = true }
                    else if (event.key === Qt.Key_Escape) { addField.forceActiveFocus(); event.accepted = true }
                    else if (event.key === Qt.Key_Up && (event.modifiers & Qt.ControlModifier)) { page.move(currentIndex, currentIndex - 1); event.accepted = true }
                    else if (event.key === Qt.Key_Down && (event.modifiers & Qt.ControlModifier)) { page.move(currentIndex, currentIndex + 1); event.accepted = true }
                }
                delegate: Rectangle {
                    id: row
                    required property int index
                    required property var modelData
                    width: taskList.width
                    height: 36
                    radius: 6
                    color: taskList.currentIndex === index ? Qt.rgba(1, 1, 1, 0.08) : (rowHover.hovered ? Qt.rgba(1, 1, 1, 0.04) : "transparent")
                    border.width: (taskList.currentIndex === index && taskList.activeFocus) || taskList.dragFrom === index ? 1 : 0
                    border.color: page.theme.accent
                    opacity: modelData.archived ? 0.5 : 1
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 4
                        anchors.rightMargin: 8
                        spacing: 6
                        // Drag handle: reorders without fighting the list's own scrolling.
                        Text {
                            id: grip
                            text: "⋮⋮"
                            color: page.theme.foreground
                            opacity: gripHover.hovered || rowDrag.active ? 0.9 : 0.3
                            font.pixelSize: Math.round(13 * page.theme.scale)
                            Layout.preferredWidth: 16
                            horizontalAlignment: Text.AlignHCenter
                            HoverHandler { id: gripHover; cursorShape: Qt.SizeVerCursor }
                            DragHandler {
                                id: rowDrag
                                target: null
                                xAxis.enabled: false
                                onActiveChanged: {
                                    if (active) { taskList.dragFrom = row.index; taskList.currentIndex = row.index; return }
                                    if (taskList.dragFrom < 0) return
                                    const p = grip.mapToItem(taskList.contentItem, 0, rowDrag.centroid.position.y)
                                    page.move(taskList.dragFrom, taskList.dropIndex(p.y))
                                    taskList.dragFrom = -1
                                }
                            }
                        }
                        CheckBox {
                            checked: modelData.completed
                            onToggled: page.toggle(modelData)
                            focusPolicy: Qt.NoFocus
                        }
                        Text {
                            visible: taskList.editingRow !== row.index
                            Layout.fillWidth: true
                            text: modelData.title
                            elide: Text.ElideRight
                            color: page.theme.foreground
                            font.pixelSize: Math.round(15 * page.theme.scale)
                            font.strikeout: modelData.completed
                        }
                        TextField {
                            id: editField
                            visible: taskList.editingRow === row.index
                            Layout.fillWidth: true
                            text: modelData.title
                            onVisibleChanged: if (visible) { text = modelData.title; forceActiveFocus(); selectAll() }
                            onAccepted: { page.rename(modelData, text); taskList.editingRow = -1; taskList.forceActiveFocus() }
                            Keys.onEscapePressed: { taskList.editingRow = -1; taskList.forceActiveFocus() }
                        }
                        Text { visible: modelData.archived; text: "archived"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: Math.round(11 * page.theme.scale) }
                    }
                    HoverHandler { id: rowHover }
                    TapHandler {
                        acceptedButtons: Qt.LeftButton
                        onTapped: { taskList.currentIndex = row.index; taskList.forceActiveFocus() }
                        onDoubleTapped: taskList.startEdit(row.index)
                    }
                    TapHandler { acceptedButtons: Qt.RightButton; onTapped: { taskMenu.task = modelData; taskMenu.row = row.index; taskList.currentIndex = row.index; taskMenu.popup() } }
                }
                Text {
                    anchors.centerIn: parent
                    visible: page.tasks.length === 0 && page.groupId >= 0
                    text: page.showArchived ? "Nothing here" : "Nothing open. Type above to add a task."
                    color: page.theme.foreground
                    opacity: 0.5
                    font.pixelSize: Math.round(14 * page.theme.scale)
                }
            }
            Text {
                Layout.leftMargin: 16
                Layout.bottomMargin: 10
                text: page.notice.length > 0 ? page.notice : (page.backend.connected ? "" : page.backend.status)
                color: page.theme.foreground
                opacity: 0.6
                font.pixelSize: Math.round(12 * page.theme.scale)
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
        }
    }
}
