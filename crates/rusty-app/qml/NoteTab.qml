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
    // Live preview (TICKET-028): the page as parts (the frontmatter first, then one per
    // section) lined up with the rendered blocks; a click turns one section into a
    // source editor while the rest stay rendered. `editMode` is the user's ("live" or
    // "source"); `tools` splits the page by the Rust rule.
    property string editMode: "live"
    // Named apart from the window's `tools` id: an unqualified `tools` inside the tab
    // would find this property first and bind it to itself.
    property var sectionTools: null
    readonly property bool live: editing && editMode === "live"
    property var parts: []
    readonly property bool partsMatch: parts.length > 0 && parts.length - 1 === chunks.length
    property int liveIndex: -1
    property bool liveWhole: false
    property bool renderHeld: false
    readonly property bool sourceVisible: editing && (editMode !== "live" || liveWhole)
    function splitParts() { parts = sectionTools && raw.length > 0 ? sectionTools.pageSections(raw) : (raw.length > 0 ? [raw] : []) }
    // The page assembled from the parts, the open section's text in place of its part.
    function assemble() {
        const p = parts.slice()
        if (liveIndex >= 0 && partsMatch) {
            const item = chunkRepeater.itemAt(liveIndex)
            if (item) p[liveIndex + 1] = item.sourceText()
        }
        return p.join("")
    }
    function editSection(index, fraction) {
        if (!live) return
        if (!partsMatch) { liveWhole = true; editor.forceActiveFocus(); return }
        if (liveIndex >= 0 && liveIndex !== index) commitSection()
        liveIndex = index
        const item = chunkRepeater.itemAt(index)
        if (item) item.openSource(parts[index + 1], fraction === undefined ? 0 : fraction)
    }
    // The open section goes back into its part; a held render runs.
    function commitSection() {
        if (liveIndex < 0) return
        const item = chunkRepeater.itemAt(liveIndex)
        if (item && partsMatch) { const p = parts.slice(); p[liveIndex + 1] = item.sourceText(); parts = p }
        liveIndex = -1
        if (dirty) save()
        if (renderHeld) { renderHeld = false; load() }
    }

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
    // The deliberate tag path (TICKET-024): the vault's tags for completion, the tags
    // row's field for the palette and the pane, and one function that writes the list.
    property var tags: []
    property var tagAdd: null
    property bool pendingTagFocus: false
    property string pendingTagText: ""
    function propertyValue(key) { for (const p of properties) if (p.key === key) return p.value; return undefined }
    function tagRowReady(field) {
        tagAdd = field
        if (pendingTagFocus) { pendingTagFocus = false; field.forceActiveFocus(); field.text = pendingTagText; pendingTagText = "" }
    }
    // The page's tags as a list, a scalar `tags: x` counting as one.
    function tagList() { const v = propertyValue("tags"); return isList(v) ? listOf(v) : (typeof v === "string" && v.trim().length > 0 ? [v.trim()] : []) }
    function ownTags() { return tagList().map(function (t) { return t.replace(/^#/, "").toLowerCase() }) }
    // The vault's tags holding `q` (no case), the page's own left out, by count then
    // name, eight at most; an empty `q` lists the most used.
    function tagCompletions(q) {
        const query = q.trim().replace(/^#/, "").toLowerCase()
        const own = ownTags()
        return tags.filter(function (t) { const l = t.tag.toLowerCase(); return l.indexOf(query) >= 0 && own.indexOf(l) < 0 })
            .sort(function (a, b) { return b.count - a.count || a.tag.localeCompare(b.tag) })
            .slice(0, 8)
    }
    function tagThePage(tag) {
        const clean = tag.trim().replace(/^#/, "")
        if (clean.length === 0 || ownTags().indexOf(clean.toLowerCase()) >= 0) return
        const l = tagList(); l.push(clean)
        setProperty("tags", l)
    }
    // Put the cursor in the tags row's field, adding the property first when the page
    // has none; the row reports itself through `tagRowReady` once it exists.
    function focusTagAdd(prefill) {
        if (editing) toggleEditing()
        const text = prefill === undefined ? "" : String(prefill)
        if (propertyValue("tags") === undefined) { pendingTagFocus = true; pendingTagText = text; setProperty("tags", []); return }
        if (tagAdd) { tagAdd.forceActiveFocus(); tagAdd.text = text }
    }
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
            headings: [t.h1, t.h2, t.h3, t.h4, t.h5, t.h6], size: Math.round(15 * note.theme.scale),
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
    function reload() { if (dirty) return; if (liveIndex >= 0) { renderHeld = true; return } load() }
    // The reading view is rendered at the base size, so a size change renders it again.
    Connections { target: note.theme; function onScaleChanged() { note.reload() } }
    function save() {
        if (!dirty || !editing) return
        dirty = false
        raw = sourceVisible ? editor.text : assemble()
        ask("brain_write_page", { slug: slug, content: raw }, "saved")
    }
    function toggleEditing() {
        if (editing) { commitSection(); save(); editing = false; liveWhole = false }
        else { editing = true; liveWhole = false; if (editMode !== "live") editor.forceActiveFocus() }
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
        // Tags is the `tags` list the index reads; a page that has one gets its field.
        if (type === "Tags") { addingProperty = false; focusTagAdd(); return }
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
        if (sourceVisible) {
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
    function ensureCursorVisible() { if (sourceVisible) ensureVisibleIn(editor) }
    function ensureVisibleIn(ed) {
        const r = ed.cursorRectangle
        const top = ed.mapToItem(flick.contentItem, r.x, r.y).y
        const bottom = top + r.height
        if (top < flick.contentY + 8) flick.contentY = Math.max(0, top - 8)
        else if (bottom > flick.contentY + flick.height - 8) flick.contentY = bottom - flick.height + 8
    }

    onIsCurrentChanged: if (isCurrent && sourceVisible) editor.forceActiveFocus()

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
                if (!note.dirty) note.splitParts()
                note.loaded = true
                note.missing = false
                note.navigated(note.slug, note.title)
                break
            }
            case "links": note.links = JSON.parse(json); break
            case "graph": note.graphInfo = note.summariseGraph(JSON.parse(json)); break
            case "saved": if (note.liveIndex >= 0) note.renderHeld = true; else note.load(); break
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
                Text { visible: note.folder.length > 0; text: note.folder.replace(/\//g, " / "); color: note.theme.muted; font.pixelSize: Math.round(10 * note.theme.scale); elide: Text.ElideMiddle; Layout.maximumWidth: 300 }
                Text { visible: note.folder.length > 0; text: "/"; color: note.theme.muted; font.pixelSize: Math.round(10 * note.theme.scale) }
                Text { text: note.fileName; color: note.theme.accent; font.pixelSize: Math.round(10 * note.theme.scale); elide: Text.ElideMiddle; Layout.maximumWidth: 360 }
                Text { visible: note.dirty; text: "•"; color: note.theme.accent; font.pixelSize: Math.round(14 * note.theme.scale) }
                Item { Layout.fillWidth: true }
                // The favorite star: the page's bookmark, toggled here or with Ctrl+D.
                Text {
                    text: note.bookmarked ? "★" : "☆"
                    color: note.bookmarked || starHover.hovered ? note.theme.gold : note.theme.muted
                    font.pixelSize: Math.round(14 * note.theme.scale)
                    HoverHandler { id: starHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: note.requestBookmark(note.slug, note.title) }
                    ToolTip.visible: starHover.hovered
                    ToolTip.text: note.bookmarked ? "Remove from favorites (Ctrl+D)" : "Add to favorites (Ctrl+D)"
                    ToolTip.delay: 600
                }
                Text {
                    text: note.editing ? (note.sourceVisible ? "[ EDIT ]" : "[ LIVE ]") : "[ READ ]"
                    color: readHover.hovered ? note.theme.accent : note.theme.muted
                    font.pixelSize: Math.round(10 * note.theme.scale)
                    font.letterSpacing: 1
                    HoverHandler { id: readHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: note.toggleEditing() }
                    ToolTip.visible: readHover.hovered
                    ToolTip.text: note.editing ? "Reading view (Ctrl+E)" : (note.editMode === "live" ? "Live preview: click a section to edit it (Ctrl+E)" : "Edit the source (Ctrl+E)")
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
                MenuItem { text: note.bookmarked ? "Remove from favorites" : "Add to favorites"; onTriggered: note.requestBookmark(note.slug, note.title) }
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
                    font.pixelSize: Math.round(14 * note.theme.scale)
                }
                Button { visible: note.missing && note.backend.connected; Layout.alignment: Qt.AlignHCenter; text: "Create it"; onClicked: note.ask("brain_new_page", { folder: note.folder, name: note.fileName }, "created") }
                Text { visible: note.notice.length > 0; text: note.notice; color: note.theme.muted; font.pixelSize: Math.round(12 * note.theme.scale); Layout.alignment: Qt.AlignHCenter }
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
                        Text { text: "Local graph"; color: note.theme.gold; font.pixelSize: Math.round(9 * note.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                        Item { Layout.fillWidth: true }
                        Text { text: (note.graphInfo ? note.graphInfo.nodes : 0) + " nodes"; color: note.theme.muted; font.pixelSize: Math.round(9 * note.theme.scale); font.capitalization: Font.AllUppercase }
                    }
                    Repeater {
                        model: [["direct links", "direct", note.theme.accent], ["related notes", "related", note.theme.alive], ["distant nodes", "distant", note.theme.lineBright]]
                        delegate: RowLayout {
                            required property var modelData
                            spacing: 8
                            Rectangle { width: 28; height: 1; color: modelData[2] }
                            Text { text: modelData[0]; color: note.theme.muted; font.pixelSize: Math.round(9 * note.theme.scale); Layout.fillWidth: true }
                            Text { text: String(note.graphInfo ? note.graphInfo[modelData[1]] : 0).padStart(2, "0"); color: note.theme.foreground; font.pixelSize: Math.round(9 * note.theme.scale) }
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
                        Text { text: "●"; color: note.theme.alive; font.pixelSize: Math.round(9 * note.theme.scale) }
                        Text { text: "Live note"; color: note.theme.alive; font.pixelSize: Math.round(9 * note.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase; Layout.leftMargin: -4 }
                        Text { visible: note.updatedText.length > 0; text: "Modified " + note.updatedText; color: note.theme.muted; font.pixelSize: Math.round(9 * note.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                        Text { text: "·"; color: note.theme.muted; font.pixelSize: Math.round(9 * note.theme.scale) }
                        Text { text: note.backlinkCount + (note.backlinkCount === 1 ? " backlink" : " backlinks"); color: note.theme.muted; font.pixelSize: Math.round(9 * note.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
                    }
                    Item { visible: !note.editing; height: 16 }
                    RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    Text { text: "#"; color: note.theme.accent; font.pixelSize: Math.round(28 * note.theme.scale) }
                    // The inline title: Enter renames the file, as in Obsidian.
                    TextInput {
                        id: titleField
                        Layout.fillWidth: true
                        text: note.title
                        color: note.theme.bright
                        font.pixelSize: Math.round(28 * note.theme.scale)
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
                            Text { text: "Properties"; color: note.theme.muted; font.pixelSize: Math.round(13 * note.theme.scale) }
                            Text { text: note.properties.length; color: note.theme.faint; font.pixelSize: Math.round(12 * note.theme.scale) }
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
                                enabled: newType.currentText !== "Tags"
                                font.pixelSize: Math.round(13 * note.theme.scale)
                                onAccepted: note.addProperty(text, newType.currentText)
                                Keys.onEscapePressed: note.addingProperty = false
                                onVisibleChanged: if (visible) { text = ""; forceActiveFocus() }
                            }
                            ComboBox { id: newType; model: ["Text", "List", "Number", "Checkbox", "Date", "Tags"]; Layout.preferredWidth: 120; font.pixelSize: Math.round(13 * note.theme.scale); onCurrentTextChanged: if (currentText === "Tags") newKey.text = "tags" }
                            Button { text: "Add"; onClicked: note.addProperty(newKey.text, newType.currentText) }
                        }
                        Text {
                            visible: !note.addingProperty
                            text: "+ Add property"
                            color: addHover.hovered ? note.theme.foreground : note.theme.faint
                            font.pixelSize: Math.round(13 * note.theme.scale)
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
                        visible: !note.sourceVisible
                        spacing: 0
                        Repeater {
                            id: chunkRepeater
                            model: note.chunks
                            // One section: its rendered block, and in live preview the
                            // source editor that takes its place while it is open.
                            delegate: Item {
                                id: block
                                required property int index
                                required property var modelData
                                readonly property bool open: note.live && note.liveIndex === index
                                Layout.fillWidth: true
                                implicitHeight: open ? sectionEditor.implicitHeight + 8 : rendered.implicitHeight
                                function sourceText() { return sectionEditor.text }
                                function openSource(text, fraction) {
                                    note.applying = true
                                    sectionEditor.text = text
                                    note.applying = false
                                    sectionEditor.forceActiveFocus()
                                    const lines = text.split("\n")
                                    const target = Math.min(lines.length - 1, Math.max(0, Math.floor(fraction * lines.length)))
                                    let pos = 0
                                    for (let k = 0; k < target; k++) pos += lines[k].length + 1
                                    sectionEditor.cursorPosition = pos
                                }
                                Text {
                                    id: rendered
                                    visible: !block.open
                                    width: parent.width
                                    text: block.modelData
                                    textFormat: Text.RichText
                                    wrapMode: Text.WordWrap
                                    color: note.theme.foreground
                                    linkColor: note.theme.link
                                    font.pixelSize: Math.round(15 * note.theme.scale)
                                    lineHeight: 1.5
                                    onLinkActivated: (link) => note.onLink(link)
                                    HoverHandler { cursorShape: rendered.hoveredLink.length > 0 ? Qt.PointingHandCursor : (note.live ? Qt.IBeamCursor : Qt.ArrowCursor) }
                                    // In live preview a click on the text (not on a link) opens the section.
                                    TapHandler {
                                        enabled: note.live
                                        onTapped: (eventPoint) => { if (rendered.hoveredLink.length === 0) note.editSection(block.index, eventPoint.position.y / Math.max(1, rendered.height)) }
                                    }
                                }
                                TextArea {
                                    id: sectionEditor
                                    visible: block.open
                                    width: parent.width
                                    wrapMode: TextEdit.Wrap
                                    textFormat: TextEdit.PlainText
                                    font.family: note.theme.termFont
                                    font.pointSize: 11 * note.theme.scale
                                    color: note.theme.foreground
                                    selectionColor: note.theme.accent
                                    selectedTextColor: note.theme.background
                                    selectByMouse: true
                                    padding: 4
                                    background: Rectangle { color: "transparent"; border.color: note.theme.line; border.width: 1; radius: 4 }
                                    tabStopDistance: 32
                                    onTextChanged: if (block.open && note.loaded && !note.applying) { note.dirty = true; autosave.restart() }
                                    onCursorRectangleChanged: if (block.open) note.ensureVisibleIn(sectionEditor)
                                    onActiveFocusChanged: if (!activeFocus && block.open) note.commitSection()
                                    Keys.onPressed: (event) => {
                                        if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_S) { note.save(); event.accepted = true }
                                        else if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_E) { note.toggleEditing(); event.accepted = true }
                                        else if (event.key === Qt.Key_Escape) { note.commitSection(); event.accepted = true }
                                    }
                                }
                                MarkdownHighlighter { target: sectionEditor.textDocument; tokens: note.theme.tokens; monoFamily: note.theme.termFont }
                            }
                        }
                        Text { visible: note.chunks.length === 0; text: "Empty page"; color: note.theme.faint; font.pixelSize: Math.round(14 * note.theme.scale); font.italic: true }
                    }

                    // The mock's footer: who links here.
                    RowLayout {
                        visible: !note.sourceVisible && note.backlinkCount > 0
                        Layout.fillWidth: true
                        Layout.topMargin: 28
                        spacing: 13
                        Text { text: "Linked from"; color: note.theme.accent; font.pixelSize: Math.round(10 * note.theme.scale); font.letterSpacing: 1; font.capitalization: Font.AllUppercase }
                        Repeater {
                            model: note.links ? note.links.backlinks.slice(0, 6) : []
                            delegate: Text {
                                required property var modelData
                                text: modelData.from_slug ? modelData.from_slug.slice(modelData.from_slug.lastIndexOf("/") + 1) : ""
                                color: bfHover.hovered ? note.theme.bright : note.theme.alive
                                font.pixelSize: Math.round(10 * note.theme.scale)
                                HoverHandler { id: bfHover; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: note.open(modelData.from_slug) }
                            }
                        }
                        Item { Layout.fillWidth: true }
                    }

                    // Source editor: the whole file, highlighted, autosaved.
                    TextArea {
                        id: editor
                        visible: note.sourceVisible
                        Layout.fillWidth: true
                        wrapMode: TextEdit.Wrap
                        textFormat: TextEdit.PlainText
                        font.family: note.theme.termFont
                        font.pointSize: 11 * note.theme.scale
                        color: note.theme.foreground
                        selectionColor: note.theme.accent
                        selectedTextColor: note.theme.background
                        selectByMouse: true
                        persistentSelection: true
                        padding: 0
                        background: null
                        tabStopDistance: 32
                        onTextChanged: if (note.sourceVisible && note.loaded && !note.applying) { note.dirty = true; autosave.restart() }
                        onCursorRectangleChanged: note.ensureCursorVisible()
                        onActiveFocusChanged: if (!activeFocus && note.dirty) note.save()
                        Keys.onPressed: (event) => {
                            if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_S) { note.save(); event.accepted = true }
                            else if ((event.modifiers & Qt.ControlModifier) && event.key === Qt.Key_E) { note.toggleEditing(); event.accepted = true }
                        }
                    }
                    MarkdownHighlighter { target: editor.textDocument; tokens: note.theme.tokens; monoFamily: note.theme.termFont }

                    Text { visible: note.notice.length > 0; text: note.notice; color: note.theme.muted; font.pixelSize: Math.round(12 * note.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.topMargin: 12 }
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
        Component.onCompleted: if (key === "tags" && kind === "list") note.tagRowReady(chipAdd)
        Component.onDestruction: if (note.tagAdd === chipAdd) note.tagAdd = null
        Icon {
            name: prow.kind === "list" ? "list" : prow.kind === "number" ? "hash" : prow.kind === "bool" ? "check-square" : prow.kind === "date" ? "calendar" : "text"
            color: note.theme.faint
            size: 15
            Layout.alignment: Qt.AlignTop
            Layout.topMargin: 6
        }
        Text { text: prow.key; color: note.theme.muted; font.pixelSize: Math.round(14 * note.theme.scale); Layout.preferredWidth: 130; Layout.alignment: Qt.AlignTop; Layout.topMargin: 4; elide: Text.ElideRight }
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
                    Text { id: chipText; anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 8; text: chip.modelData; color: prow.key === "tags" ? note.theme.tag : note.theme.foreground; font.pixelSize: Math.round(12 * note.theme.scale) }
                    Icon { anchors.right: parent.right; anchors.rightMargin: 5; anchors.verticalCenter: parent.verticalCenter; name: "close"; color: note.theme.faint; size: 11; TapHandler { onTapped: { const l = note.listOf(prow.value); l.splice(chip.index, 1); note.setProperty(prow.key, l) } } }
                    HoverHandler { id: chipHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: if (prow.key === "tags") note.openTag(chip.modelData.replace(/^#/, "")) }
                }
            }
            TextField {
                id: chipAdd
                width: Math.max(60, implicitWidth)
                height: 22
                font.pixelSize: Math.round(12 * note.theme.scale)
                placeholderText: "add"
                background: Rectangle { color: chipAdd.activeFocus ? note.theme.hover : "transparent"; radius: 10; border.color: note.theme.line; border.width: chipAdd.activeFocus ? 1 : 0 }
                // The tags row completes from the vault's tag index (TICKET-024): the list
                // follows the text with nothing picked, Down and Up pick, Enter takes the
                // pick or adds the text as typed, Tab takes the pick or the first.
                readonly property bool completes: prow.key === "tags"
                property var completions: []
                property int pick: -1
                function refresh() {
                    if (!completes) return
                    completions = note.tagCompletions(text)
                    pick = -1
                    if (activeFocus && completions.length > 0) completionPopup.open(); else completionPopup.close()
                }
                function add(v) {
                    const clean = v.trim()
                    if (clean.length === 0) return
                    const l = note.listOf(prow.value); l.push(clean)
                    text = ""; completionPopup.close()
                    note.setProperty(prow.key, l)
                }
                onTextChanged: refresh()
                onActiveFocusChanged: if (activeFocus) refresh(); else completionPopup.close()
                onAccepted: add(completionPopup.visible && pick >= 0 ? completions[pick].tag : text)
                Keys.onDownPressed: if (completionPopup.visible) pick = Math.min(pick + 1, completions.length - 1)
                Keys.onUpPressed: if (completionPopup.visible) pick = Math.max(pick - 1, 0)
                Keys.onTabPressed: (event) => { if (completionPopup.visible && completions.length > 0) { add(completions[Math.max(pick, 0)].tag); event.accepted = true } else event.accepted = false }
                Keys.onEscapePressed: (event) => { if (completionPopup.visible) { completionPopup.close(); event.accepted = true } else event.accepted = false }
                Popup {
                    id: completionPopup
                    parent: chipAdd
                    y: chipAdd.height + 2
                    width: 220
                    padding: 4
                    focus: false
                    closePolicy: Popup.CloseOnPressOutsideParent
                    background: Rectangle { color: note.theme.panel3; radius: 6; border.color: note.theme.line; border.width: 1 }
                    contentItem: ColumnLayout {
                        spacing: 0
                        Repeater {
                            model: chipAdd.completions
                            delegate: Rectangle {
                                required property int index
                                required property var modelData
                                Layout.fillWidth: true
                                height: 24
                                radius: 4
                                color: index === chipAdd.pick || cHover.hovered ? note.theme.hover : "transparent"
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    anchors.rightMargin: 8
                                    spacing: 6
                                    Text { text: "#" + modelData.tag; color: note.theme.tag; font.pixelSize: Math.round(12 * note.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                                    Text { text: modelData.count; color: note.theme.faint; font.pixelSize: Math.round(11 * note.theme.scale) }
                                }
                                HoverHandler { id: cHover; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: chipAdd.add(modelData.tag) }
                            }
                        }
                    }
                }
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
            font.pixelSize: Math.round(14 * note.theme.scale)
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
            font.pixelSize: Math.round(13 * note.theme.scale)
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
