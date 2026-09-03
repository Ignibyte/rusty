import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// One page in a document tab, laid out as Obsidian lays out a note: the view header
// (history, breadcrumb, reading toggle, menu), the inline title, the properties block,
// then the page as the back end rendered it, or its whole source in the editor with
// highlighting. Edits autosave after a pause and on Ctrl+S; the tab keeps its own
// navigation history.
Item {
    id: note
    required property var backend
    required property var theme
    property string slug: ""
    property bool isCurrent: false
    property bool editing: false

    // Read by the tab strip, the right pane and the status bar.
    property string title: ""
    property string pageType: ""
    property var links: null
    property var outline: []
    property var properties: []
    property var unresolved: []
    property int words: 0
    property int characters: 0
    property int taskCount: 0
    property string raw: ""
    property string html: ""
    property var chunks: []
    property string notice: ""
    property bool loaded: false
    property bool missing: false
    property bool dirty: false
    property var history: []
    property int historyIndex: -1
    property var pending: ({})
    property bool applying: false
    property bool addingProperty: false
    readonly property int backlinkCount: links ? links.backlinks.length : 0
    // The `updated` property's value, for the meta line.
    readonly property string updatedText: { const p = properties.find(function (x) { return x.key === "updated" }); return p && p.value !== undefined && p.value !== null ? String(p.value) : "" }
    // How connected the page is: nodes at one, two and three or more links away.
    property var graphInfo: null
    function summariseGraph(g) {
        const adj = {}
        for (const e of g.edges || []) { (adj[e.from] = adj[e.from] || []).push(e.to); (adj[e.to] = adj[e.to] || []).push(e.from) }
        const depth = {}; depth[slug] = 0
        const queue = [slug]
        while (queue.length > 0) { const n = queue.shift(); for (const m of adj[n] || []) if (depth[m] === undefined) { depth[m] = depth[n] + 1; queue.push(m) } }
        let direct = 0, related = 0, distant = 0
        for (const n of g.nodes || []) { const d = depth[n.id]; if (d === 1) direct++; else if (d === 2) related++; else if (d !== 0) distant++ }
        return { nodes: (g.nodes || []).length, direct: direct, related: related, distant: distant }
    }
    readonly property string folder: slug.indexOf("/") >= 0 ? slug.slice(0, slug.lastIndexOf("/")) : ""
    readonly property string fileName: slug.slice(slug.lastIndexOf("/") + 1)
    readonly property int contentWidth: Math.max(320, Math.min(flick.width - 64, 720))

    signal navigated(string slug, string title)
    signal openTag(string tag)
    signal requestMove(string slug)
    signal requestDelete(string slug)
    signal requestLocalGraph(string slug)
    signal requestBookmark(string slug, string title)
    property bool bookmarked: false
    // A heading a bookmark asked for before the page had rendered.
    property string pendingHeading: ""

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }

    // The renderer's colours, all from the theme.
    function style() {
        const t = JSON.parse(theme.tokens || "{}")
        return {
            text: theme.foreground, muted: theme.muted, link: theme.link, unresolved: theme.faint,
            accent: theme.accent, code: theme.code, code_bg: theme.codeBg, mono: theme.termFont,
            mark_bg: t.mark || theme.accent, line: theme.line, tag: theme.tag,
            red: t.red, green: t.green, yellow: t.yellow, blue: t.blue, magenta: t.magenta, cyan: t.cyan,
            headings: [t.h1, t.h2, t.h3, t.h4, t.h5, t.h6], size: 15,
            bright: theme.bright, gold: theme.gold, alive: theme.alive, accent_soft: theme.accentSoft,
            panel3: theme.panel3, line_bright: theme.lineBright, marks: true, code_head: true
        }
    }

    // Navigate this tab to a page (pushed on the history).
    function open(s) {
        if (s === slug && loaded) return
        if (dirty) save()
        editing = false
        const h = history.slice(0, historyIndex + 1)
        h.push(s)
        history = h
        historyIndex = h.length - 1
        slug = s
        loaded = false
        missing = false
        load()
    }
    function goBack() { if (historyIndex > 0) { historyIndex--; jump(history[historyIndex]) } }
    function goForward() { if (historyIndex < history.length - 1) { historyIndex++; jump(history[historyIndex]) } }
    function jump(s) { if (dirty) save(); editing = false; slug = s; loaded = false; load() }
    function load() {
        if (slug.length === 0) return
        ask("brain_render", { slug: slug, style: style() }, "render")
        ask("brain_graph", { around: slug, depth: 3 }, "graph")
        ask("brain_get_links", { slug: slug }, "links")
    }
    function reload() { if (!dirty) load() }
    function save() {
        if (!dirty || !editing) return
        dirty = false
        raw = editor.text
        ask("brain_write_page", { slug: slug, content: editor.text }, "saved")
    }
    function toggleEditing() {
        if (editing) { save(); editing = false }
        else { editing = true; editor.forceActiveFocus() }
    }
    function editTitle() { titleField.forceActiveFocus(); titleField.selectAll() }
    function renameTo(name) {
        const clean = name.trim().replace(/\//g, "-")
        if (clean.length === 0 || clean === fileName) return
        ask("brain_rename", { from: slug, to: (folder.length > 0 ? folder + "/" : "") + clean }, "renamed")
    }
    function createFromLink(name) { ask("brain_new_page", { folder: "", name: name }, "created") }

    // Properties edit the frontmatter through the back end; the page re-renders after.
    function setProperty(key, value) { ask("brain_set_property", { slug: slug, key: key, value: value }, "property") }
    function removeProperty(key) { ask("brain_remove_property", { slug: slug, key: key }, "property") }
    function startAddProperty() { if (editing) toggleEditing(); addingProperty = true }
    function addProperty(key, type) {
        const k = key.trim()
        if (k.length === 0) return
        let value = ""
        if (type === "List") value = []
        else if (type === "Number") value = 0
        else if (type === "Checkbox") value = false
        else if (type === "Date") value = new Date().toISOString().slice(0, 10)
        addingProperty = false
        setProperty(k, value)
    }
    // Lists arrive from JSON as sequence types, not JS arrays.
    function isList(v) { return v !== null && typeof v === "object" && typeof v.length === "number" }
    function listOf(v) { const out = []; if (isList(v)) for (let i = 0; i < v.length; i++) out.push(String(v[i])); return out }
    function kindOf(v) {
        if (isList(v)) return "list"
        if (typeof v === "number") return "number"
        if (typeof v === "boolean") return "bool"
        if (typeof v === "string" && /^\d{4}-\d{2}-\d{2}/.test(v)) return "date"
        if (typeof v === "object" && v !== null) return "object"
        return "text"
    }

    // Links the renderer wrote carry the rusty: scheme.
    function onLink(link) {
        if (link.startsWith("rusty:page/")) {
            const t = link.slice(11)
            const hash = t.indexOf("#")
            note.open(hash >= 0 ? t.slice(0, hash) : t)
        } else if (link.startsWith("rusty:new/")) {
            createFromLink(decodeURIComponent(link.slice(10)))
        } else if (link.startsWith("rusty:task/")) {
            toggleTask(parseInt(link.slice(11)))
        } else if (link.startsWith("rusty:tag/")) {
            note.openTag(link.slice(10))
        } else if (link.startsWith("#")) {
            // A footnote or heading anchor: nothing to scroll to yet.
        } else {
            Qt.openUrlExternally(link)
        }
    }

    // Flip the n-th task box in the source, counted the way the renderer counts them.
    function toggleTask(n) {
        const lines = raw.split("\n")
        const box = /^(\s*(?:>\s*)*(?:[-*+]|\d+[.)])\s+\[)([ xX])(\])/
        let seen = 0
        let fence = false
        for (let i = 0; i < lines.length; i++) {
            const t = lines[i].trimStart()
            if (t.startsWith("```") || t.startsWith("~~~")) { fence = !fence; continue }
            if (fence) continue
            const m = lines[i].match(box)
            if (!m) continue
            if (seen === n) {
                lines[i] = lines[i].replace(box, function (all, a, b, c) { return a + (b === " " ? "x" : " ") + c })
                break
            }
            seen++
        }
        const content = lines.join("\n")
        if (content === raw) return
        raw = content
        ask("brain_write_page", { slug: slug, content: content }, "toggled")
    }

    // A bookmark asks for a heading by its text; the scroll waits for the outline.
    function scrollToHeadingText(text) {
        const i = outline.findIndex(function (h) { return h.text === text })
        if (i >= 0) { pendingHeading = ""; scrollToHeading(i) } else pendingHeading = text
    }
    // The outline pane asks for a heading: scroll the reading view or move the cursor.
    function scrollToHeading(i) {
        if (i < 0 || i >= outline.length) return
        const h = outline[i]
        if (editing) {
            const lines = editor.text.split("\n")
            let pos = 0
            for (let k = 0; k < h.line && k < lines.length; k++) pos += lines[k].length + 1
            editor.cursorPosition = pos
            editor.forceActiveFocus()
            ensureCursorVisible()
        } else {
            const offset = chunks.length > 0 && chunks[0].startsWith("<!--") === false && chunks[0].startsWith("<h") ? 0 : 1
            const item = chunkRepeater.itemAt(Math.min(i + offset, chunkRepeater.count - 1))
            if (item) flick.contentY = Math.max(0, Math.min(readingColumn.y + item.y - 12, flick.contentHeight - flick.height))
        }
    }
    function ensureCursorVisible() {
        if (!editing) return
        const r = editor.cursorRectangle
        const top = editor.mapToItem(flick.contentItem, r.x, r.y).y
        const bottom = top + r.height
        if (top < flick.contentY + 8) flick.contentY = Math.max(0, top - 8)
        else if (bottom > flick.contentY + flick.height - 8) flick.contentY = bottom - flick.height + 8
    }

    onIsCurrentChanged: if (isCurrent && editing) editor.forceActiveFocus()

    Timer { id: autosave; interval: 1500; onTriggered: note.save() }

    Connections {
        target: note.backend
        function onResult(id, tool, json, ok) {
            const kind = note.pending[id]
            if (kind === undefined) return
            const p = note.pending; delete p[id]; note.pending = p
            if (!ok) {
                note.notice = tool + ": " + json
                if (kind === "render") { note.loaded = true; note.missing = true }
                return
            }
            note.notice = ""
            switch (kind) {
            case "render": {
                const data = JSON.parse(json)
                if (data === null) { note.missing = true; note.loaded = true; note.title = note.fileName; note.navigated(note.slug, note.title); return }
                note.title = data.title
                note.pageType = data.page_type
                note.properties = data.properties
                note.outline = data.outline
                if (note.pendingHeading.length > 0) Qt.callLater(function () { note.scrollToHeadingText(note.pendingHeading) })
                note.unresolved = data.unresolved
                note.words = data.words
                note.characters = data.characters
                note.taskCount = data.tasks
                note.html = data.html
                note.chunks = data.html.split("<!--h-->").filter(function (c) { return c.length > 0 })
                if (data.raw !== note.raw) note.raw = data.raw
                if (!note.dirty && editor.text !== data.raw) { note.applying = true; editor.text = data.raw; note.applying = false }
                note.loaded = true
                note.missing = false
                note.navigated(note.slug, note.title)
                break
            }
            case "links": note.links = JSON.parse(json); break
            case "graph": note.graphInfo = note.summariseGraph(JSON.parse(json)); break
            case "saved": note.load(); break
            case "toggled": note.load(); break
            case "created": note.open(JSON.parse(json)); break
            case "property": note.load(); break
            case "renamed": { const r = JSON.parse(json); note.slug = r.to; note.history[note.historyIndex] = r.to; note.load(); break }
            }
        }
        function onDataChanged() { note.reload() }
    }

    Component.onCompleted: if (slug.length > 0 && history.length === 0) { history = [slug]; historyIndex = 0; load() }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // The view header: history, breadcrumb, reading toggle and the menu.
        Item {
            Layout.fillWidth: true
            height: 36
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 2
                HeaderButton { icon: "arrow-left"; enabled: note.historyIndex > 0; tip: "Back"; onClicked: note.goBack() }
                HeaderButton { icon: "arrow-right"; enabled: note.historyIndex < note.history.length - 1; tip: "Forward"; onClicked: note.goForward() }
                Item { Layout.fillWidth: true }
                Text { visible: note.folder.length > 0; text: note.folder.replace(/\//g, " / "); color: note.theme.muted; font.pixelSize: 10; elide: Text.ElideMiddle; Layout.maximumWidth: 300 }
                Text { visible: note.folder.length > 0; text: "/"; color: note.theme.muted; font.pixelSize: 10 }
                Text { text: note.fileName; color: note.theme.accent; font.pixelSize: 10; elide: Text.ElideMiddle; Layout.maximumWidth: 360 }
                Text { visible: note.dirty; text: "•"; color: note.theme.accent; font.pixelSize: 14 }
                Item { Layout.fillWidth: true }
                Text {
                    text: note.editing ? "[ EDIT ]" : "[ READ ]"
                    color: readHover.hovered ? note.theme.accent : note.theme.muted
                    font.pixelSize: 10
                    font.letterSpacing: 1
                    HoverHandler { id: readHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: note.toggleEditing() }
                    ToolTip.visible: readHover.hovered
                    ToolTip.text: note.editing ? "Reading view (Ctrl+E)" : "Edit (Ctrl+E)"
                    ToolTip.delay: 600
                }
                HeaderButton { icon: "more"; tip: "More options"; onClicked: moreMenu.popup() }
            }
            Menu {
                id: moreMenu
                MenuItem { text: note.editing ? "Reading view" : "Edit source"; onTriggered: note.toggleEditing() }
                MenuItem { text: "Rename…"; onTriggered: note.editTitle() }
                MenuItem { text: "Move file to…"; onTriggered: note.requestMove(note.slug) }
                MenuItem { text: "Open local graph"; onTriggered: note.requestLocalGraph(note.slug) }
                MenuItem { text: note.bookmarked ? "Remove bookmark" : "Bookmark…"; onTriggered: note.requestBookmark(note.slug, note.title) }
                MenuSeparator {}
                MenuItem { text: "Delete file"; onTriggered: note.requestDelete(note.slug) }
            }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: note.theme.line; opacity: 0.6 }

        // Waiting, missing, or the page.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.centerIn: parent
                visible: !note.loaded || note.missing
                spacing: 8
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: !note.backend.connected ? "waiting for rusty-mcp" : (note.missing ? "No page at " + note.slug : "loading " + note.slug)
                    color: note.theme.muted
                    font.pixelSize: 14
                }
                Button { visible: note.missing && note.backend.connected; Layout.alignment: Qt.AlignHCenter; text: "Create it"; onClicked: note.ask("brain_new_page", { folder: note.folder, name: note.fileName }, "created") }
                Text { visible: note.notice.length > 0; text: note.notice; color: note.theme.muted; font.pixelSize: 12; Layout.alignment: Qt.AlignHCenter }
            }

            // The mock's legend: how connected this page is; a click opens the local graph.
            Rectangle {
                z: 2
                visible: note.loaded && !note.missing && !note.editing && note.graphInfo !== null && flick.width - note.contentWidth > 440
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: 22
                width: 190
                implicitHeight: legendCol.implicitHeight + 20
                color: note.theme.panel
                opacity: 0.94
                border.width: 1
                border.color: note.theme.line
                ColumnLayout {
                    id: legendCol
                    anchors.fill: parent
                    anchors.margins: 10
                    spacing: 6
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "Local graph"; color: note.theme.gold; font.pixelSize: 9; font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                        Item { Layout.fillWidth: true }
                        Text { text: (note.graphInfo ? note.graphInfo.nodes : 0) + " nodes"; color: note.theme.muted; font.pixelSize: 9; font.capitalization: Font.AllUppercase }
                    }
                    Repeater {
                        model: [["direct links", "direct", note.theme.accent], ["related notes", "related", note.theme.alive], ["distant nodes", "distant", note.theme.lineBright]]
                        delegate: RowLayout {
                            required property var modelData
                            spacing: 8
                            Rectangle { width: 28; height: 1; color: modelData[2] }
                            Text { text: modelData[0]; color: note.theme.muted; font.pixelSize: 9; Layout.fillWidth: true }
                            Text { text: String(note.graphInfo ? note.graphInfo[modelData[1]] : 0).padStart(2, "0"); color: note.theme.foreground; font.pixelSize: 9 }
                        }
                    }
                }
                HoverHandler { cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: note.requestLocalGraph(note.slug) }
            }

            Flickable {
                id: flick
                anchors.fill: parent
                visible: note.loaded && !note.missing
                contentWidth: width
                contentHeight: body.implicitHeight + 120
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

                ColumnLayout {
                    id: body
                    x: Math.round((flick.width - note.contentWidth) / 2)
                    y: 24
                    width: note.contentWidth
                    spacing: 0

                    // The mock's meta line: the alive dot, when the page changed, its backlinks.
                    RowLayout {
                        visible: !note.editing
                        spacing: 10
                        Text { text: "●"; color: note.theme.alive; font.pixelSize: 9 }
                        Text { text: "Live note"; color: note.theme.alive; font.pixelSize: 9; font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.leftMargin: -4 }
                        Text { visible: note.updatedText.length > 0; text: "Modified " + note.updatedText; color: note.theme.muted; font.pixelSize: 9; font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                        Text { text: "·"; color: note.theme.muted; font.pixelSize: 9 }
                        Text { text: note.backlinkCount + (note.backlinkCount === 1 ? " backlink" : " backlinks"); color: note.theme.muted; font.pixelSize: 9; font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                    }
                    Item { visible: !note.editing; height: 16 }
                    RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    Text { text: "#"; color: note.theme.accent; font.pixelSize: 28 }
                    // The inline title: Enter renames the file, as in Obsidian.
                    TextInput {
                        id: titleField
                        Layout.fillWidth: true
                        text: note.title
                        color: note.theme.bright
                        font.pixelSize: 28
                        font.weight: Font.Medium
                        selectByMouse: true
                        selectionColor: note.theme.accent
                        selectedTextColor: note.theme.background
                        wrapMode: TextInput.Wrap
                        onEditingFinished: if (text !== note.title) note.renameTo(text)
                        Keys.onEscapePressed: { text = note.title; note.forceActiveFocus() }
                        Keys.onReturnPressed: note.forceActiveFocus()
                    }
                    }
                    Item { height: 14 }

                    // Properties, editable in place as Obsidian's are: a value edits by its
                    // type, a row can go, "Add property" adds a key of a chosen type.
                    ColumnLayout {
                        Layout.fillWidth: true
                        visible: !note.editing
                        spacing: 2
                        RowLayout {
                            spacing: 6
                            visible: note.properties.length > 0
                            Text { text: "Properties"; color: note.theme.muted; font.pixelSize: 13 }
                            Text { text: note.properties.length; color: note.theme.faint; font.pixelSize: 12 }
                        }
                        Repeater {
                            model: note.properties
                            delegate: PropertyRow {}
                        }
                        RowLayout {
                            visible: note.addingProperty
                            spacing: 6
                            TextField {
                                id: newKey
                                Layout.preferredWidth: 160
                                placeholderText: "Property name"
                                font.pixelSize: 13
                                onAccepted: note.addProperty(text, newType.currentText)
                                Keys.onEscapePressed: note.addingProperty = false
                                onVisibleChanged: if (visible) { text = ""; forceActiveFocus() }
                            }
                            ComboBox { id: newType; model: ["Text", "List", "Number", "Checkbox", "Date"]; Layout.preferredWidth: 120; font.pixelSize: 13 }
                            Button { text: "Add"; onClicked: note.addProperty(newKey.text, newType.currentText) }
                        }
                        Text {
                            visible: !note.addingProperty
                            text: "+ Add property"
                            color: addHover.hovered ? note.theme.foreground : note.theme.faint
                            font.pixelSize: 13
                            HoverHandler { id: addHover; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: note.startAddProperty() }
                        }
                        Item { height: 12 }
                    }

                    // Reading view: one rich-text block per top-level section, so the
                    // outline can scroll to a heading.
                    ColumnLayout {
                        id: readingColumn
                        Layout.fillWidth: true
                        visible: !note.editing
                        spacing: 0
                        Repeater {
                            id: chunkRepeater
                            model: note.chunks
                            delegate: Text {
                                required property var modelData
                                Layout.fillWidth: true
                                text: modelData
                                textFormat: Text.RichText
                                wrapMode: Text.WordWrap
                                color: note.theme.foreground
                                linkColor: note.theme.link
                                font.pixelSize: 15
                                lineHeight: 1.5
                                onLinkActivated: (link) => note.onLink(link)
                                HoverHandler { cursorShape: parent.hoveredLink.length > 0 ? Qt.PointingHandCursor : Qt.ArrowCursor }
                            }
                        }
                        Text { visible: note.chunks.length === 0; text: "Empty page"; color: note.theme.faint; font.pixelSize: 14; font.italic: true }
                    }

                    // The mock's footer: who links here.
                    RowLayout {
                        visible: !note.editing && note.backlinkCount > 0
                        Layout.fillWidth: true
                        Layout.topMargin: 28
                        spacing: 13
                        Text { text: "Linked from"; color: note.theme.accent; font.pixelSize: 10; font.letterSpacing: 1; font.capitalization: Font.AllUppercase }
                        Repeater {
                            model: note.links ? note.links.backlinks.slice(0, 6) : []
                            delegate: Text {
                                required property var modelData
                                text: modelData.from_slug ? modelData.from_slug.slice(modelData.from_slug.lastIndexOf("/") + 1) : ""
                                color: bfHover.hovered ? note.theme.bright : note.theme.alive
                                font.pixelSize: 10
                                HoverHandler { id: bfHover; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: note.open(modelData.from_slug) }
                            }
                        }
                        Item { Layout.fillWidth: true }
                    }

                    // Source editor: the whole file, highlighted, autosaved.
                    TextArea {
                        id: editor
                        visible: note.editing
                        Layout.fillWidth: true
                        wrapMode: TextEdit.Wrap
                        textFormat: TextEdit.PlainText
                        font.family: note.theme.termFont
                        font.pointSize: 11
                        color: note.theme.foreground
                        selectionColor: note.theme.accent
                        selectedTextColor: note.theme.background
                        selectByMouse: true
                        persistentSelection: true
                        padding: 0
                        background: null
                        tabStopDistance: 32
                        onTextChanged: if (note.editing && note.loaded && !note.applying) { note.dirty = true; autosave.restart() }
                        onCursorRectangleChanged: note.ensureCursorVisible()
                        onActiveFocusChanged: if (!activeFocus && note.dirty) note.save()
                        Keys.onPressed: (event) => {
                            if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_S) { note.save(); event.accepted = true }
                            else if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_E) { note.toggleEditing(); event.accepted = true }
                        }
                    }
                    MarkdownHighlighter { target: editor.textDocument; tokens: note.theme.tokens; monoFamily: note.theme.termFont }

                    Text { visible: note.notice.length > 0; text: note.notice; color: note.theme.muted; font.pixelSize: 12; wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.topMargin: 12 }
                }
            }
        }
    }

    // One property: an icon for its type, the key, and a value editor by type.
    component PropertyRow: RowLayout {
        id: prow
        required property var modelData
        readonly property string key: modelData.key
        readonly property var value: modelData.value
        readonly property string kind: note.kindOf(modelData.value)
        Layout.fillWidth: true
        spacing: 8
        Icon {
            name: prow.kind === "list" ? "list" : prow.kind === "number" ? "hash" : prow.kind === "bool" ? "check-square" : prow.kind === "date" ? "calendar" : "text"
            color: note.theme.faint
            size: 15
            Layout.alignment: Qt.AlignTop
            Layout.topMargin: 6
        }
        Text { text: prow.key; color: note.theme.muted; font.pixelSize: 14; Layout.preferredWidth: 130; Layout.alignment: Qt.AlignTop; Layout.topMargin: 4; elide: Text.ElideRight }
        // List: chips, each removable; tags are also links to their search.
        Flow {
            Layout.fillWidth: true
            spacing: 6
            visible: prow.kind === "list"
            Repeater {
                model: prow.kind === "list" ? note.listOf(prow.value) : []
                delegate: Rectangle {
                    id: chip
                    required property int index
                    required property string modelData
                    radius: 10
                    color: chipHover.hovered ? note.theme.active : note.theme.hover
                    width: chipText.implicitWidth + 30
                    height: 22
                    Text { id: chipText; anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 8; text: chip.modelData; color: prow.key === "tags" ? note.theme.tag : note.theme.foreground; font.pixelSize: 12 }
                    Icon { anchors.right: parent.right; anchors.rightMargin: 5; anchors.verticalCenter: parent.verticalCenter; name: "close"; color: note.theme.faint; size: 11; TapHandler { onTapped: { const l = note.listOf(prow.value); l.splice(chip.index, 1); note.setProperty(prow.key, l) } } }
                    HoverHandler { id: chipHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: if (prow.key === "tags") note.openTag(chip.modelData.replace(/^#/, "")) }
                }
            }
            TextField {
                id: chipAdd
                width: Math.max(60, implicitWidth)
                height: 22
                font.pixelSize: 12
                placeholderText: "add"
                background: Rectangle { color: chipAdd.activeFocus ? note.theme.hover : "transparent"; radius: 10; border.color: note.theme.line; border.width: chipAdd.activeFocus ? 1 : 0 }
                onAccepted: { const v = text.trim(); if (v.length > 0) { const l = note.listOf(prow.value); l.push(v); text = ""; note.setProperty(prow.key, l) } }
            }
        }
        CheckBox {
            visible: prow.kind === "bool"
            checked: prow.kind === "bool" ? prow.value : false
            onToggled: note.setProperty(prow.key, checked)
        }
        TextField {
            id: valueField
            visible: prow.kind === "text" || prow.kind === "date" || prow.kind === "number"
            Layout.fillWidth: true
            font.pixelSize: 14
            color: note.theme.foreground
            text: prow.kind === "text" || prow.kind === "date" || prow.kind === "number" ? String(prow.value) : ""
            placeholderText: prow.kind === "date" ? "YYYY-MM-DD" : "Empty"
            background: Rectangle { color: valueField.activeFocus || valueHover.hovered ? note.theme.hover : "transparent"; radius: 4 }
            HoverHandler { id: valueHover }
            onEditingFinished: {
                const t = text.trim()
                if (t === String(prow.value)) return
                if (prow.kind === "number") { const n = Number(t); if (!isNaN(n)) note.setProperty(prow.key, n) }
                else note.setProperty(prow.key, t)
            }
        }
        Text {
            visible: prow.kind === "object"
            Layout.fillWidth: true
            text: JSON.stringify(prow.value)
            color: note.theme.foreground
            font.pixelSize: 13
            wrapMode: Text.Wrap
        }
        Rectangle {
            width: 20; height: 20; radius: 4
            color: rmHover.hovered ? note.theme.hover : "transparent"
            Layout.alignment: Qt.AlignTop
            Layout.topMargin: 4
            Icon { anchors.centerIn: parent; name: "close"; color: note.theme.faint; size: 12 }
            HoverHandler { id: rmHover; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: note.removeProperty(prow.key) }
            ToolTip.visible: rmHover.hovered
            ToolTip.text: "Remove property"
            ToolTip.delay: 600
        }
    }

    component HeaderButton: Rectangle {
        id: hb
        property string icon
        property string tip: ""
        signal clicked()
        width: 26
        height: 26
        radius: 5
        color: hbHover.hovered && enabled ? note.theme.hover : "transparent"
        opacity: enabled ? 1 : 0.35
        Icon { anchors.centerIn: parent; name: hb.icon; color: note.theme.muted; size: 16 }
        HoverHandler { id: hbHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: if (hb.enabled) hb.clicked() }
        ToolTip.visible: hbHover.hovered && hb.tip.length > 0
        ToolTip.text: hb.tip
        ToolTip.delay: 600
    }
}
