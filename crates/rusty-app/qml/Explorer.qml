import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The file explorer: the vault's real folder tree, folders first, with the actions
// Obsidian offers on a right click (new note, new folder, rename, move, delete). A
// click opens a page; folders fold. Expanded folders are remembered by the window.
Item {
    id: explorer
    required property var backend
    required property var theme
    property var tree: null
    property var expanded: ({})
    property string currentSlug: ""
    property var rows: []
    property string renaming: ""
    property var pending: ({})
    property string notice: ""
    readonly property string currentFolder: currentSlug.indexOf("/") >= 0 ? currentSlug.slice(0, currentSlug.lastIndexOf("/")) : ""

    signal openPage(string slug)
    signal created(string slug)
    signal expandedEdited()
    signal requestBookmark(var row)
    // The bookmarked files and folders, shown above the tree; the window owns the list.
    property var favorites: []
    signal openFavorite(var bookmark)
    signal removeFavorite(var bookmark)
    // Folder roots on the machine (TICKET-016): the window owns the list, the disk is
    // read through `folders`, and a listing is cached until Refresh or a root change.
    required property var folders
    property var roots: []
    property var agents: []
    property var agentNames: ({})
    property var listing: ({})
    signal openFile(string path)
    signal openAgentAt(string program, string cwd)
    signal addRootRequested()
    signal removeRoot(string path)
    function isDirRow(r) { return r !== null && r !== undefined && (r.kind === "folder" || r.kind === "dir" || r.kind === "root") }
    function entriesOf(dir) {
        if (listing[dir] === undefined) { const l = listing; try { l[dir] = JSON.parse(folders.list(dir)) } catch (e) { l[dir] = [] } listing = l }
        return listing[dir]
    }
    function walkDisk(dir, depth, out) {
        for (const e of entriesOf(dir)) {
            out.push({ name: e.name, path: e.path, kind: e.kind === "folder" ? "dir" : "disk", depth: depth })
            if (e.kind === "folder" && expanded[e.path]) walkDisk(e.path, depth + 1, out)
        }
    }
    function refreshDisk() { listing = ({}); rebuild() }
    onRootsChanged: refreshDisk()
    function expandPath(path) { if (!expanded[path]) toggle(path) }
    function copyText(t) { copier.text = t; copier.selectAll(); copier.copy(); copier.text = "" }
    TextEdit { id: copier; visible: false; width: 1; height: 1 }

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function rebuild() {
        const out = []
        if (tree) walk(tree.children, 0, out)
        if (roots.length > 0) out.push({ name: "Folders", path: "", kind: "section", depth: 0 })
        for (const r of roots) {
            out.push({ name: r.name, path: r.path, kind: "root", depth: 0 })
            if (expanded[r.path]) walkDisk(r.path, 1, out)
        }
        rows = out
    }
    function walk(children, depth, out) {
        for (const c of children) {
            out.push({ name: c.name, path: c.path, kind: c.kind, depth: depth, pages: c.pages, hasChildren: c.children.length > 0 })
            if (c.kind === "folder" && expanded[c.path]) walk(c.children, depth + 1, out)
        }
    }
    onTreeChanged: rebuild()
    function toggle(path) { const e = expanded; e[path] = !e[path]; expanded = e; rebuild(); expandedEdited() }
    function collapseAll() { expanded = ({}); rebuild(); expandedEdited() }
    function reveal(slug) {
        const parts = slug.split("/")
        const e = expanded
        let p = ""
        for (let i = 0; i < parts.length - 1; i++) { p = p.length > 0 ? p + "/" + parts[i] : parts[i]; e[p] = true }
        expanded = e
        rebuild()
        expandedEdited()
        const i = rows.findIndex(function (r) { return r.path === slug })
        if (i >= 0) list.positionViewAtIndex(i, ListView.Contain)
    }
    // A bookmarked folder: open it and its parents, and bring its row into view.
    function revealFolder(path) {
        const parts = path.split("/")
        const e = expanded
        let p = ""
        for (let i = 0; i < parts.length; i++) { p = p.length > 0 ? p + "/" + parts[i] : parts[i]; e[p] = true }
        expanded = e
        rebuild()
        expandedEdited()
        const i = rows.findIndex(function (r) { return r.path === path })
        if (i >= 0) { list.currentIndex = i; list.positionViewAtIndex(i, ListView.Contain) }
    }
    function folders() {
        const out = [""]
        function w(children) { for (const c of children) if (c.kind === "folder") { out.push(c.path); w(c.children) } }
        if (tree) w(tree.children)
        return out
    }
    function folderOf(row) {
        if (row.kind === "folder") return row.path
        return row.path.indexOf("/") >= 0 ? row.path.slice(0, row.path.lastIndexOf("/")) : ""
    }
    function newNote(folder) { ask("brain_new_page", { folder: folder }, "created") }
    function newFolder(parent, name) {
        const clean = name.trim().replace(/^\/+|\/+$/g, "")
        if (clean.length === 0) return
        ask("brain_new_folder", { path: parent.length > 0 ? parent + "/" + clean : clean }, "folder")
    }
    function isDiskRow(r) { return r !== null && r !== undefined && (r.kind === "dir" || r.kind === "disk") }
    // A disk write answers `{ok, path}` or `{ok, error}`; the tree follows the disk on
    // success and the reason shows in the notice otherwise (TICKET-019).
    function diskResult(json) {
        try { const r = JSON.parse(json); if (r.ok) { notice = ""; refreshDisk() } else notice = r.error }
        catch (e) { notice = String(e) }
    }
    function newDiskFile(dir, name) { diskResult(folders.createFile(dir, name)) }
    function newDiskFolder(dir, name) { diskResult(folders.createDir(dir, name)) }
    function moveDisk(path, into) { diskResult(folders.moveEntry(path, into)) }
    // The root that holds `path`, or "" — a drag never crosses roots or reaches the vault.
    function rootOf(path) { for (const r of roots) if (path === r.path || path.startsWith(r.path + "/")) return r.path; return "" }
    function rename(row, newName) {
        if (isDiskRow(row)) { diskResult(folders.renameEntry(row.path, newName)); return }
        const clean = newName.trim().replace(/\//g, "-")
        if (clean.length === 0 || clean === row.name) return
        const dir = row.path.indexOf("/") >= 0 ? row.path.slice(0, row.path.lastIndexOf("/") + 1) : ""
        ask("brain_rename", { from: row.path, to: dir + clean }, "renamed")
    }
    function moveTo(row, folder) { ask("brain_rename", { from: row.path, to: folder + "/" }, "renamed") }
    function remove(row) {
        if (isDiskRow(row)) { diskResult(folders.trash(row.path)); return }
        if (row.kind === "folder") ask("brain_delete_folder", { path: row.path }, "deleted")
        else ask("brain_delete_page", { slug: row.path }, "deleted")
    }
    function rowAt(i) { return i >= 0 && i < rows.length ? rows[i] : null }

    Connections {
        target: explorer.backend
        function onResult(id, tool, json, ok) {
            const kind = explorer.pending[id]
            if (kind === undefined) return
            const p = explorer.pending; delete p[id]; explorer.pending = p
            if (!ok) { explorer.notice = tool + ": " + json; return }
            explorer.notice = ""
            if (kind === "created") explorer.created(JSON.parse(json))
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Obsidian's nav header: new note, new folder, collapse all.
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 4
            spacing: 2
            NavButton { icon: "new-note"; tip: "New note"; onClicked: explorer.newNote(explorer.currentFolder) }
            NavButton { icon: "new-folder"; tip: "New folder"; onClicked: folderDialog.openFor(explorer.currentFolder) }
            NavButton { icon: "collapse"; tip: "Collapse all"; onClicked: explorer.collapseAll() }
            NavButton { icon: "plus"; tip: "Add a folder from the machine"; onClicked: explorer.addRootRequested() }
            Item { Layout.fillWidth: true }
        }

        // Favorites: the bookmarked files and folders, gathered above the tree. A click
        // opens one; a right-click removes it.
        ColumnLayout {
            visible: explorer.favorites.length > 0
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 6
            spacing: 0
            Text { text: "Favorites"; color: explorer.theme.muted; font.pixelSize: Math.round(10 * explorer.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.bottomMargin: 2 }
            Repeater {
                model: explorer.favorites
                delegate: Rectangle {
                    id: fav
                    required property var modelData
                    Layout.fillWidth: true
                    height: Math.round(22 * explorer.theme.scale)
                    radius: explorer.theme.radius
                    color: favHover.hovered ? explorer.theme.hover : "transparent"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        anchors.rightMargin: 6
                        spacing: 6
                        Text { text: "★"; color: explorer.theme.gold; font.pixelSize: Math.round(11 * explorer.theme.scale) }
                        Icon { name: fav.modelData.kind === "folder" ? "folder" : "file"; color: explorer.theme.muted; size: 13 }
                        Text { text: fav.modelData.title; color: explorer.theme.foreground; font.pixelSize: Math.round(12 * explorer.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                    }
                    HoverHandler { id: favHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { acceptedButtons: Qt.LeftButton; onTapped: explorer.openFavorite(fav.modelData) }
                    TapHandler { acceptedButtons: Qt.RightButton; onTapped: explorer.removeFavorite(fav.modelData) }
                    ToolTip.visible: favHover.hovered
                    ToolTip.text: fav.modelData.path + " (right-click removes)"
                    ToolTip.delay: 600
                }
            }
            Rectangle { Layout.fillWidth: true; height: 1; color: explorer.theme.line; Layout.topMargin: 6 }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: 4
            clip: true
            model: explorer.rows
            spacing: 0
            keyNavigationEnabled: true
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            property int dragFrom: -1
            Keys.onReturnPressed: { const r = explorer.rowAt(currentIndex); if (r) { if (r.kind === "page") explorer.openPage(r.path); else if (r.kind === "disk") explorer.openFile(r.path); else if (explorer.isDirRow(r)) explorer.toggle(r.path) } }
            // F2 and Delete act on the current row whichever half of the list it is in.
            Keys.onPressed: (event) => {
                const r = explorer.rowAt(currentIndex)
                if (!r || r.kind === "section" || r.kind === "root" || r.kind === "file") return
                if (event.key === Qt.Key_F2) { explorer.renaming = r.path; event.accepted = true }
                else if (event.key === Qt.Key_Delete) { deleteDialog.openFor(r); event.accepted = true }
            }
            Keys.onRightPressed: { const r = explorer.rowAt(currentIndex); if (explorer.isDirRow(r) && !explorer.expanded[r.path]) explorer.toggle(r.path) }
            Keys.onLeftPressed: { const r = explorer.rowAt(currentIndex); if (explorer.isDirRow(r) && explorer.expanded[r.path]) explorer.toggle(r.path) }
            delegate: Item {
                id: row
                required property int index
                required property var modelData
                width: list.width
                height: 24
                readonly property bool active: modelData.kind === "page" && modelData.path === explorer.currentSlug
                readonly property bool isRenaming: explorer.renaming.length > 0 && explorer.renaming === modelData.path
                readonly property bool isDir: explorer.isDirRow(modelData)
                readonly property bool isSection: modelData.kind === "section"
                Rectangle {
                    anchors.fill: parent
                    radius: explorer.theme.radius
                    color: row.active ? explorer.theme.active : (rowHover.hovered || list.currentIndex === row.index ? explorer.theme.hover : "transparent")
                    Rectangle { visible: row.active; anchors.left: parent.left; width: 1; height: parent.height; color: explorer.theme.accent }
                }
                // Indent guides for nested rows.
                Repeater {
                    model: row.modelData.depth
                    delegate: Rectangle {
                        required property int index
                        x: 14 + 8 + index * 17
                        y: 0
                        width: 1
                        height: row.height
                        color: explorer.theme.line
                        opacity: 0.5
                    }
                }
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10 + row.modelData.depth * 17
                    anchors.rightMargin: 10
                    spacing: 4
                    Icon {
                        visible: row.isDir
                        name: explorer.expanded[row.modelData.path] ? "chevron-down" : "chevron-right"
                        color: explorer.theme.accentSoft
                        size: 12
                    }
                    Text { visible: row.isDir; text: "▰"; color: row.modelData.kind === "root" ? explorer.theme.accent : explorer.theme.gold; font.pixelSize: Math.round(9 * explorer.theme.scale) }
                    Text { visible: row.isSection; text: row.modelData.name; color: explorer.theme.muted; font.pixelSize: Math.round(10 * explorer.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.fillWidth: true; Layout.topMargin: 6 }
                    Text { visible: !row.isDir && !row.isSection; text: row.modelData.kind === "page" ? (row.active ? "◆" : "◇") : "◈"; color: row.modelData.kind === "page" ? (row.active ? explorer.theme.accent : explorer.theme.alive) : explorer.theme.muted; font.pixelSize: Math.round(10 * explorer.theme.scale); Layout.preferredWidth: 12 }
                    Text {
                        visible: !row.isRenaming && !row.isSection
                        Layout.fillWidth: true
                        text: row.modelData.name
                        color: row.active ? explorer.theme.bright : explorer.theme.muted
                        font.pixelSize: Math.round(12 * explorer.theme.scale)
                        elide: Text.ElideRight
                    }
                    TextField {
                        id: renameField
                        visible: row.isRenaming
                        Layout.fillWidth: true
                        font.pixelSize: Math.round(13 * explorer.theme.scale)
                        text: row.modelData.name
                        onVisibleChanged: if (visible) { forceActiveFocus(); selectAll() }
                        onAccepted: { explorer.rename(row.modelData, text); explorer.renaming = "" }
                        Keys.onEscapePressed: explorer.renaming = ""
                        onActiveFocusChanged: if (!activeFocus && row.isRenaming) explorer.renaming = ""
                    }
                    Text { visible: row.modelData.kind === "folder" && row.modelData.pages !== undefined; text: String(row.modelData.pages).padStart(2, "0"); color: explorer.theme.faint; font.pixelSize: Math.round(10 * explorer.theme.scale) }
                    Text {
                        visible: row.modelData.kind === "file" || row.modelData.kind === "disk"
                        text: row.modelData.name.indexOf(".") >= 0 ? row.modelData.name.slice(row.modelData.name.lastIndexOf(".") + 1).toUpperCase() : ""
                        color: explorer.theme.faint
                        font.pixelSize: Math.round(9 * explorer.theme.scale)
                        font.letterSpacing: 0.5
                    }
                }
                HoverHandler { id: rowHover }
                // Drag a file or folder on disk onto a folder or root under the same root
                // to move it there; the handler moves nothing itself, and a press that
                // becomes a drag never fires the tap below.
                DragHandler {
                    id: rowDrag
                    enabled: explorer.isDiskRow(row.modelData)
                    target: null
                    xAxis.enabled: false
                    onActiveChanged: {
                        if (active) { list.dragFrom = row.index; return }
                        const from = list.dragFrom
                        list.dragFrom = -1
                        if (from < 0) return
                        const p = row.mapToItem(list.contentItem, 0, rowDrag.centroid.position.y)
                        const to = list.indexAt(4, p.y)
                        const src = explorer.rowAt(from); const dst = explorer.rowAt(to)
                        if (!src || !dst || to === from) return
                        if (dst.kind !== "dir" && dst.kind !== "root") return
                        if (dst.path === src.path.slice(0, src.path.lastIndexOf("/"))) return
                        const root = explorer.rootOf(src.path)
                        if (root.length === 0 || explorer.rootOf(dst.path) !== root) return
                        explorer.moveDisk(src.path, dst.path)
                    }
                }
                TapHandler {
                    acceptedButtons: Qt.LeftButton
                    onTapped: {
                        list.currentIndex = row.index
                        if (row.modelData.kind === "page") explorer.openPage(row.modelData.path)
                        else if (row.modelData.kind === "disk") explorer.openFile(row.modelData.path)
                        else if (row.isDir) explorer.toggle(row.modelData.path)
                    }
                }
                TapHandler {
                    acceptedButtons: Qt.RightButton
                    onTapped: {
                        list.currentIndex = row.index
                        if (row.isSection) return
                        if (row.modelData.kind === "root" || row.modelData.kind === "dir" || row.modelData.kind === "disk") { diskMenu.row = row.modelData; diskMenu.popup() }
                        else { rowMenu.row = row.modelData; rowMenu.popup() }
                    }
                }
            }
        }
        Text { visible: explorer.notice.length > 0; text: explorer.notice; color: explorer.theme.muted; font.pixelSize: Math.round(11 * explorer.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.margins: 8 }
    }

    Menu {
        id: rowMenu
        property var row: null
        MenuItem { text: "New note"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: explorer.newNote(explorer.folderOf(rowMenu.row)) }
        MenuItem { text: "New folder"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: folderDialog.openFor(explorer.folderOf(rowMenu.row)) }
        MenuSeparator {}
        MenuItem { text: "Rename…"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: explorer.renaming = rowMenu.row.path }
        MenuItem { text: "Move to…"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: moveDialog.openFor(rowMenu.row) }
        MenuItem { text: "Bookmark…"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: explorer.requestBookmark(rowMenu.row) }
        MenuSeparator {}
        MenuItem { text: "Delete"; enabled: rowMenu.row !== null && rowMenu.row.kind !== "file"; onTriggered: deleteDialog.openFor(rowMenu.row) }
    }

    // The menu of a folder root, a folder under it, or a file on disk.
    Menu {
        id: diskMenu
        property var row: null
        readonly property bool dirRow: row !== null && (row.kind === "dir" || row.kind === "root")
        readonly property string dir: row === null ? "" : (dirRow ? row.path : row.path.slice(0, row.path.lastIndexOf("/")))
        Instantiator {
            model: explorer.agents
            delegate: MenuItem {
                required property string modelData
                text: "Open " + (explorer.agentNames[modelData] || modelData) + " here"
                onTriggered: explorer.openAgentAt(modelData, diskMenu.dir)
            }
            onObjectAdded: (i, o) => diskMenu.insertItem(i, o)
            onObjectRemoved: (i, o) => diskMenu.removeItem(o)
        }
        MenuItem { text: "Open a shell here"; onTriggered: explorer.openAgentAt("shell", diskMenu.dir) }
        MenuSeparator {}
        MenuItem { text: "New file…"; onTriggered: diskNameDialog.openFor("file", diskMenu.dir) }
        MenuItem { text: "New folder…"; onTriggered: diskNameDialog.openFor("folder", diskMenu.dir) }
        MenuItem { text: "Rename…"; visible: explorer.isDiskRow(diskMenu.row); height: visible ? implicitHeight : 0; onTriggered: explorer.renaming = diskMenu.row.path }
        MenuItem { text: "Move to…"; visible: explorer.isDiskRow(diskMenu.row); height: visible ? implicitHeight : 0; onTriggered: diskMoveDialog.openFor(diskMenu.row) }
        MenuItem { text: "Delete"; visible: explorer.isDiskRow(diskMenu.row); height: visible ? implicitHeight : 0; onTriggered: deleteDialog.openFor(diskMenu.row) }
        MenuSeparator {}
        MenuItem { text: "Open"; visible: diskMenu.row !== null && diskMenu.row.kind === "disk"; height: visible ? implicitHeight : 0; onTriggered: explorer.openFile(diskMenu.row.path) }
        MenuItem { text: "Open outside"; visible: diskMenu.row !== null && diskMenu.row.kind === "disk"; height: visible ? implicitHeight : 0; onTriggered: explorer.folders.openExternally(diskMenu.row.path) }
        MenuItem { text: "Copy path"; onTriggered: explorer.copyText(diskMenu.row.path) }
        MenuItem { text: "Reveal in the file manager"; onTriggered: explorer.folders.openExternally(diskMenu.dir) }
        MenuSeparator {}
        MenuItem { text: "Refresh"; onTriggered: explorer.refreshDisk() }
        MenuItem { text: "Remove this root"; visible: diskMenu.row !== null && diskMenu.row.kind === "root"; height: visible ? implicitHeight : 0; onTriggered: explorer.removeRoot(diskMenu.row.path) }
    }

    Dialog {
        id: folderDialog
        title: "New folder"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok | Dialog.Cancel
        property string parentFolder: ""
        function openFor(folder) { parentFolder = folder; folderName.text = ""; open(); folderName.forceActiveFocus() }
        onAccepted: explorer.newFolder(parentFolder, folderName.text)
        ColumnLayout {
            spacing: 8
            Label { text: folderDialog.parentFolder.length > 0 ? "Inside " + folderDialog.parentFolder : "At the vault root" }
            TextField { id: folderName; Layout.preferredWidth: 320; placeholderText: "Folder name"; onAccepted: folderDialog.accept() }
        }
    }

    Dialog {
        id: moveDialog
        title: "Move to folder"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok | Dialog.Cancel
        property var row: null
        function openFor(r) { row = r; targetBox.model = explorer.folders().map(function (f) { return f.length > 0 ? f : "/" }); targetBox.currentIndex = 0; open() }
        onAccepted: if (row) explorer.moveTo(row, targetBox.currentText === "/" ? "" : targetBox.currentText)
        ColumnLayout {
            spacing: 8
            Label { text: moveDialog.row ? "Move " + moveDialog.row.path + " to" : "" }
            ComboBox { id: targetBox; Layout.preferredWidth: 320 }
        }
    }

    Dialog {
        id: deleteDialog
        title: "Delete"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Yes | Dialog.No
        property var row: null
        function openFor(r) { row = r; open() }
        onAccepted: if (row) explorer.remove(row)
        Label { text: deleteDialog.row ? (explorer.isDiskRow(deleteDialog.row) ? "Move " + deleteDialog.row.path + " to the trash? A file manager can restore it." : "Delete " + deleteDialog.row.path + "? It moves to archive/ and can be restored by hand.") : "" }
    }

    // A name for a new file or folder on disk (TICKET-019).
    Dialog {
        id: diskNameDialog
        title: what === "folder" ? "New folder" : "New file"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok | Dialog.Cancel
        property string what: "file"
        property string dir: ""
        function openFor(kind, folder) { what = kind; dir = folder; diskName.text = ""; open(); diskName.forceActiveFocus() }
        onAccepted: { if (what === "folder") explorer.newDiskFolder(dir, diskName.text); else explorer.newDiskFile(dir, diskName.text) }
        ColumnLayout {
            spacing: 8
            Label { text: "Inside " + diskNameDialog.dir; elide: Text.ElideMiddle; Layout.preferredWidth: 320 }
            TextField { id: diskName; Layout.preferredWidth: 320; placeholderText: diskNameDialog.what === "folder" ? "Folder name" : "File name"; onAccepted: diskNameDialog.accept() }
        }
    }

    // Move a file or folder on disk by typing where; the keyboard's path beside the drag.
    Dialog {
        id: diskMoveDialog
        title: "Move to folder"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok | Dialog.Cancel
        property var row: null
        function openFor(r) { row = r; diskTarget.text = r.path.slice(0, r.path.lastIndexOf("/")); open(); diskTarget.forceActiveFocus(); diskTarget.selectAll() }
        onAccepted: if (row) explorer.moveDisk(row.path, diskTarget.text.trim())
        ColumnLayout {
            spacing: 8
            Label { text: diskMoveDialog.row ? "Move " + diskMoveDialog.row.name + " into" : ""; elide: Text.ElideMiddle; Layout.preferredWidth: 360 }
            TextField { id: diskTarget; Layout.preferredWidth: 360; placeholderText: "An existing folder"; onAccepted: diskMoveDialog.accept() }
        }
    }

    function openMenuFor(slug) {
        const i = rows.findIndex(function (r) { return r.path === slug })
        if (i >= 0) { rowMenu.row = rows[i]; rowMenu.popup() }
    }
    function moveDialogFor(slug) {
        const i = rows.findIndex(function (r) { return r.path === slug })
        moveDialog.openFor(i >= 0 ? rows[i] : { path: slug, kind: "page", name: slug.slice(slug.lastIndexOf("/") + 1) })
    }
    function deleteDialogFor(slug) {
        const i = rows.findIndex(function (r) { return r.path === slug })
        deleteDialog.openFor(i >= 0 ? rows[i] : { path: slug, kind: "page", name: slug.slice(slug.lastIndexOf("/") + 1) })
    }

    component NavButton: Rectangle {
        id: nb
        property string icon
        property string tip: ""
        signal clicked()
        width: 26
        height: 26
        radius: 5
        color: nbHover.hovered ? explorer.theme.hover : "transparent"
        Icon { anchors.centerIn: parent; name: nb.icon; color: explorer.theme.muted; size: 16 }
        HoverHandler { id: nbHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: nb.clicked() }
        ToolTip.visible: nbHover.hovered && nb.tip.length > 0
        ToolTip.text: nb.tip
        ToolTip.delay: 600
    }
}
