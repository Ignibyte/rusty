import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The graph view, as Obsidian draws it: pages as dots sized by their links, links as
// lines, laid out by forces on a canvas, with a foldable panel of filters, colour
// groups, display and force settings. With `around` set it is the local graph of one
// page. A node opens its page on a click; drag a node to pin it while held, drag the
// background to pan, wheel to zoom.
Item {
    id: view
    required property var backend
    required property var theme
    property string around: ""
    property int depth: 1
    // Settings the window remembers (a JSON object); the view writes back on change.
    property var settings: ({})
    signal openPage(string slug)
    signal searchTag(string tag)
    signal settingsEdited(var settings)

    // Filters
    property string filter: ""
    property bool showTags: false
    property bool showUnresolved: false
    // A decision's typed edges (consulted, supersedes, follows_up), drawn dashed in the accent.
    property bool showDecisions: true
    property bool showOrphans: true
    // Groups: [{query, colour}]
    property var groups: []
    // Display
    property bool arrows: false
    property real textFade: 0.5
    property real nodeSize: 1.0
    property real linkThickness: 1.0
    // Forces
    property real centerForce: 0.5
    property real repelForce: 10
    property real linkForce: 1.0
    property real linkDistance: 250

    property var nodes: []
    property var edges: []
    property var pending: ({})
    property string notice: ""
    property bool loaded: false
    property real zoom: 1
    property real panX: 0
    property real panY: 0
    property int hovered: -1
    property int dragging: -1
    property bool panning: false
    property real pressX: 0
    property real pressY: 0
    property real moved: 0
    property real alpha: 1
    property bool panelOpen: true
    // The first settled layout after a load fits the view once.
    property bool fitted: false
    readonly property var tokens: JSON.parse(theme.tokens || "{}")
    readonly property var palette: [theme.accent, tokens.red, tokens.green, tokens.yellow, tokens.magenta, tokens.cyan, tokens.blue].filter(function (c) { return c })

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function load() {
        const args = { tags: showTags, unresolved: showUnresolved }
        if (around.length > 0) { args.around = around; args.depth = depth }
        ask("brain_graph", args, "graph")
    }
    onAroundChanged: load()
    onDepthChanged: if (around.length > 0) { load(); persist() }
    onShowTagsChanged: { load(); persist() }
    onShowUnresolvedChanged: { load(); persist() }
    onShowDecisionsChanged: { load(); persist() }
    onShowOrphansChanged: { applyFilter(); persist() }
    onFilterChanged: applyFilter()
    onGroupsChanged: { colour(); canvas.requestPaint(); persist() }
    onArrowsChanged: { canvas.requestPaint(); persist() }
    onTextFadeChanged: { canvas.requestPaint(); persist() }
    onNodeSizeChanged: { canvas.requestPaint(); persist() }
    onLinkThicknessChanged: { canvas.requestPaint(); persist() }
    onCenterForceChanged: { restart(); persist() }
    onRepelForceChanged: { restart(); persist() }
    onLinkForceChanged: { restart(); persist() }
    onLinkDistanceChanged: { restart(); persist() }

    function applySettings(s) {
        if (!s) return
        if (typeof s.showTags === "boolean") showTags = s.showTags
        if (typeof s.showUnresolved === "boolean") showUnresolved = s.showUnresolved
        if (typeof s.showDecisions === "boolean") showDecisions = s.showDecisions
        if (typeof s.showOrphans === "boolean") showOrphans = s.showOrphans
        if (Array.isArray(s.groups)) groups = s.groups
        if (typeof s.arrows === "boolean") arrows = s.arrows
        for (const k of ["textFade", "nodeSize", "linkThickness", "centerForce", "repelForce", "linkForce", "linkDistance", "depth"])
            if (typeof s[k] === "number") view[k] = s[k]
    }
    function persist() {
        if (!loaded) return
        settingsEdited({ showTags: showTags, showUnresolved: showUnresolved, showDecisions: showDecisions, showOrphans: showOrphans, groups: groups, arrows: arrows,
                          textFade: textFade, nodeSize: nodeSize, linkThickness: linkThickness, centerForce: centerForce, repelForce: repelForce,
                          linkForce: linkForce, linkDistance: linkDistance, depth: depth })
    }

    Connections {
        target: view.backend
        function onResult(id, tool, json, ok) {
            const kind = view.pending[id]
            if (kind === undefined) return
            const p = view.pending; delete p[id]; view.pending = p
            if (!ok) { view.notice = tool + ": " + json; return }
            view.notice = ""
            if (kind === "graph") view.setGraph(JSON.parse(json))
        }
        function onDataChanged() { view.load() }
    }
    Component.onCompleted: { applySettings(settings); loaded = true; load() }

    // Keep positions of nodes that stay across a reload.
    function setGraph(g) {
        const old = {}
        for (const n of nodes) old[n.id] = n
        const index = {}
        const next = []
        const spread = Math.max(120, Math.sqrt(g.nodes.length) * 40)
        for (let i = 0; i < g.nodes.length; i++) {
            const n = g.nodes[i]
            const prev = old[n.id]
            const angle = i * 2.399963
            const radius = spread * Math.sqrt((i + 1) / g.nodes.length)
            next.push({ id: n.id, kind: n.kind, title: n.title, page_type: n.page_type, folder: n.folder, tags: n.tags || [],
                        x: prev ? prev.x : Math.cos(angle) * radius, y: prev ? prev.y : Math.sin(angle) * radius,
                        vx: 0, vy: 0, degree: 0, colour: "", visible: true, pinned: false })
            index[n.id] = i
        }
        const e = []
        for (const edge of g.edges) {
            const a = index[edge.from], b = index[edge.to]
            if (a === undefined || b === undefined) continue
            e.push({ a: a, b: b, kind: edge.kind || "link" })
            next[a].degree++
            next[b].degree++
        }
        nodes = next
        edges = e
        fitted = nodes.length === 0 || Object.keys(old).length > 0
        colour()
        applyFilter()
        restart()
    }

    // A group query: tag:x, path:p, type:t, or text in the title or id.
    function matches(n, query) {
        const q = query.trim().toLowerCase()
        if (q.length === 0) return false
        for (const token of q.split(/\s+/)) {
            let ok
            if (token.startsWith("tag:")) { const t = token.slice(4).replace(/^#/, ""); ok = n.tags.some(function (x) { const l = String(x).toLowerCase(); return l === t || l.startsWith(t + "/") }) }
            else if (token.startsWith("path:")) ok = n.id.toLowerCase().startsWith(token.slice(5))
            else if (token.startsWith("type:")) ok = n.page_type.toLowerCase() === token.slice(5)
            else ok = n.title.toLowerCase().indexOf(token) >= 0 || n.id.toLowerCase().indexOf(token) >= 0
            if (!ok) return false
        }
        return true
    }
    function colour() {
        for (const n of nodes) {
            n.colour = ""
            for (const g of groups) if (g.query && matches(n, g.query)) { n.colour = g.colour; break }
        }
    }
    function applyFilter() {
        const q = filter.trim()
        for (const n of nodes) n.visible = q.length === 0 || matches(n, q)
        if (!showOrphans) {
            const linked = {}
            for (const e of edges) if (nodes[e.a].visible && nodes[e.b].visible) { linked[e.a] = true; linked[e.b] = true }
            for (let i = 0; i < nodes.length; i++) if (nodes[i].visible && !linked[i] && nodes[i].kind === "page") nodes[i].visible = false
        }
        canvas.requestPaint()
    }

    // The simulation: repulsion between every pair, springs on the edges, a pull to
    // the centre, damping; it cools until the layout settles and restarts on change.
    function restart() { alpha = 1; sim.running = nodes.length > 0 }
    function tick() {
        const n = nodes
        const count = n.length
        if (count === 0) { sim.running = false; return }
        const repel = repelForce * 220
        for (let i = 0; i < count; i++) { n[i].fx = 0; n[i].fy = 0 }
        for (let i = 0; i < count; i++) {
            if (!n[i].visible) continue
            for (let j = i + 1; j < count; j++) {
                if (!n[j].visible) continue
                let dx = n[i].x - n[j].x, dy = n[i].y - n[j].y
                let d2 = dx * dx + dy * dy
                if (d2 < 1) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; d2 = 1 }
                const f = repel / d2
                const d = Math.sqrt(d2)
                const ux = dx / d, uy = dy / d
                n[i].fx += ux * f; n[i].fy += uy * f
                n[j].fx -= ux * f; n[j].fy -= uy * f
            }
        }
        for (const e of edges) {
            const a = n[e.a], b = n[e.b]
            if (!a.visible || !b.visible) continue
            const dx = b.x - a.x, dy = b.y - a.y
            const d = Math.max(1, Math.sqrt(dx * dx + dy * dy))
            const f = linkForce * (d - linkDistance * 0.6) * 0.05
            const ux = dx / d, uy = dy / d
            a.fx += ux * f; a.fy += uy * f
            b.fx -= ux * f; b.fy -= uy * f
        }
        let energy = 0
        for (let i = 0; i < count; i++) {
            const p = n[i]
            if (!p.visible) continue
            p.fx -= p.x * centerForce * 0.02
            p.fy -= p.y * centerForce * 0.02
            if (p.pinned) { p.vx = 0; p.vy = 0; continue }
            p.vx = (p.vx + p.fx * alpha) * 0.55
            p.vy = (p.vy + p.fy * alpha) * 0.55
            const speed = Math.sqrt(p.vx * p.vx + p.vy * p.vy)
            if (speed > 30) { p.vx *= 30 / speed; p.vy *= 30 / speed }
            p.x += p.vx
            p.y += p.vy
            energy += speed
        }
        alpha = Math.max(0.02, alpha * 0.985)
        if (energy / count < 0.03 && alpha < 0.3) {
            sim.running = false
            if (!fitted) { fitted = true; fit() }
        }
        canvas.requestPaint()
    }
    Timer { id: sim; interval: 33; repeat: true; running: false; onTriggered: view.tick() }

    // Node radius in graph units and screen hit-testing.
    function radiusOf(n) { return (3 + Math.sqrt(n.degree) * 1.6) * nodeSize }
    function toGraph(sx, sy) { return { x: (sx - canvas.width / 2 - panX) / zoom, y: (sy - canvas.height / 2 - panY) / zoom } }
    function nodeAt(sx, sy) {
        const p = toGraph(sx, sy)
        let best = -1, bestD = 1e9
        for (let i = 0; i < nodes.length; i++) {
            const n = nodes[i]
            if (!n.visible) continue
            const dx = n.x - p.x, dy = n.y - p.y
            const d = Math.sqrt(dx * dx + dy * dy)
            const hit = radiusOf(n) + 6 / zoom
            if (d < hit && d < bestD) { best = i; bestD = d }
        }
        return best
    }
    function neighbours(i) {
        const set = {}
        for (const e of edges) { if (e.a === i) set[e.b] = true; if (e.b === i) set[e.a] = true }
        return set
    }
    function nodeColour(n) {
        if (n.colour) return n.colour
        if (n.kind === "tag") return tokens["graph-node-tag"] || theme.tag
        if (n.kind === "unresolved") return theme.faint
        return tokens["graph-node"] || theme.accent
    }
    function fit() {
        if (nodes.length === 0) return
        let minX = 1e9, minY = 1e9, maxX = -1e9, maxY = -1e9
        for (const n of nodes) { if (!n.visible) continue; minX = Math.min(minX, n.x); maxX = Math.max(maxX, n.x); minY = Math.min(minY, n.y); maxY = Math.max(maxY, n.y) }
        const w = Math.max(50, maxX - minX), h = Math.max(50, maxY - minY)
        zoom = Math.min(1.6, Math.max(0.2, Math.min((canvas.width - 120) / w, (canvas.height - 120) / h)))
        panX = -(minX + maxX) / 2 * zoom
        panY = -(minY + maxY) / 2 * zoom
        canvas.requestPaint()
    }

    Rectangle { anchors.fill: parent; color: view.theme.background }

    Canvas {
        id: canvas
        anchors.fill: parent
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        onPaint: {
            const ctx = getContext("2d")
            ctx.reset()
            ctx.clearRect(0, 0, width, height)
            ctx.save()
            ctx.translate(width / 2 + view.panX, height / 2 + view.panY)
            ctx.scale(view.zoom, view.zoom)
            const n = view.nodes
            const hover = view.hovered
            const near = hover >= 0 ? view.neighbours(hover) : null
            const line = view.tokens["graph-line"] || view.theme.line
            // Edges
            for (const e of view.edges) {
                const a = n[e.a], b = n[e.b]
                if (!a.visible || !b.visible) continue
                const typed = e.kind !== "link"
                if (typed && !view.showDecisions) continue
                const lit = hover < 0 || e.a === hover || e.b === hover
                ctx.globalAlpha = hover < 0 ? (typed ? 0.8 : 0.55) : (lit ? 0.95 : 0.08)
                ctx.strokeStyle = typed ? view.theme.accent : (lit && hover >= 0 ? view.theme.accent : line)
                ctx.lineWidth = view.linkThickness / view.zoom * (lit && hover >= 0 ? 1.6 : 1) * (typed ? 1.3 : 1)
                ctx.setLineDash(typed ? [6 / view.zoom, 4 / view.zoom] : [])
                ctx.beginPath()
                ctx.moveTo(a.x, a.y)
                ctx.lineTo(b.x, b.y)
                ctx.stroke()
                if (view.arrows) {
                    const dx = b.x - a.x, dy = b.y - a.y
                    const d = Math.max(1, Math.sqrt(dx * dx + dy * dy))
                    const ux = dx / d, uy = dy / d
                    const r = view.radiusOf(b) + 2 / view.zoom
                    const tx = b.x - ux * r, ty = b.y - uy * r
                    const s = 5 / view.zoom
                    ctx.fillStyle = ctx.strokeStyle
                    ctx.beginPath()
                    ctx.moveTo(tx, ty)
                    ctx.lineTo(tx - ux * s * 2 - uy * s, ty - uy * s * 2 + ux * s)
                    ctx.lineTo(tx - ux * s * 2 + uy * s, ty - uy * s * 2 - ux * s)
                    ctx.closePath()
                    ctx.fill()
                }
            }
            // Nodes
            for (let i = 0; i < n.length; i++) {
                const p = n[i]
                if (!p.visible) continue
                const lit = hover < 0 || i === hover || (near && near[i])
                ctx.globalAlpha = lit ? 1 : 0.15
                ctx.fillStyle = view.nodeColour(p)
                const r = view.radiusOf(p) * (i === hover ? 1.35 : 1)
                ctx.beginPath()
                ctx.arc(p.x, p.y, r, 0, Math.PI * 2)
                ctx.fill()
                if (i === hover) {
                    ctx.strokeStyle = view.theme.foreground
                    ctx.lineWidth = 1.5 / view.zoom
                    ctx.stroke()
                }
            }
            // Labels fade in with the zoom, and always show for the hovered node and its neighbours.
            const fade = Math.max(0, Math.min(1, (view.zoom - (2.2 - view.textFade * 1.6)) / 0.6))
            ctx.font = (11 / view.zoom).toFixed(1) + "px sans-serif"
            ctx.textAlign = "center"
            for (let i = 0; i < n.length; i++) {
                const p = n[i]
                if (!p.visible) continue
                const forced = i === hover || (near && near[i])
                const a = forced ? 1 : fade * (hover < 0 ? 1 : 0.15)
                if (a <= 0.02) continue
                ctx.globalAlpha = a
                ctx.fillStyle = forced && i === hover ? view.theme.foreground : view.theme.muted
                ctx.fillText(p.title, p.x, p.y + view.radiusOf(p) + 12 / view.zoom)
            }
            ctx.restore()
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton
        cursorShape: view.hovered >= 0 ? Qt.PointingHandCursor : (view.panning ? Qt.ClosedHandCursor : Qt.ArrowCursor)
        onPositionChanged: (m) => {
            if (view.dragging >= 0) {
                const p = view.toGraph(m.x, m.y)
                const n = view.nodes[view.dragging]
                n.x = p.x; n.y = p.y; n.vx = 0; n.vy = 0
                view.moved += 1
                if (!sim.running) view.restart()
                canvas.requestPaint()
            } else if (view.panning) {
                view.panX += m.x - view.pressX
                view.panY += m.y - view.pressY
                view.pressX = m.x; view.pressY = m.y
                view.moved += 1
                canvas.requestPaint()
            } else {
                const h = view.nodeAt(m.x, m.y)
                if (h !== view.hovered) { view.hovered = h; canvas.requestPaint() }
            }
        }
        onPressed: (m) => {
            view.moved = 0
            view.pressX = m.x; view.pressY = m.y
            const h = view.nodeAt(m.x, m.y)
            if (h >= 0) { view.dragging = h; view.nodes[h].pinned = true }
            else view.panning = true
        }
        onReleased: (m) => {
            if (view.dragging >= 0) {
                const n = view.nodes[view.dragging]
                n.pinned = false
                if (view.moved < 3) {
                    if (n.kind === "page") view.openPage(n.id)
                    else if (n.kind === "tag") view.searchTag(n.id.slice(4))
                }
                view.dragging = -1
                view.restart()
            }
            view.panning = false
        }
        onExited: { if (view.hovered >= 0) { view.hovered = -1; canvas.requestPaint() } }
        onWheel: (w) => {
            const factor = w.angleDelta.y > 0 ? 1.15 : 1 / 1.15
            const next = Math.min(6, Math.max(0.15, view.zoom * factor))
            // Zoom around the cursor.
            const gx = (w.x - canvas.width / 2 - view.panX) / view.zoom
            const gy = (w.y - canvas.height / 2 - view.panY) / view.zoom
            view.zoom = next
            view.panX = w.x - canvas.width / 2 - gx * next
            view.panY = w.y - canvas.height / 2 - gy * next
            canvas.requestPaint()
        }
    }

    // The title of the hovered node, near it.
    Rectangle {
        visible: view.hovered >= 0 && view.hovered < view.nodes.length
        x: Math.min(parent.width - width - 8, Math.max(8, mouse.mouseX + 14))
        y: Math.max(8, mouse.mouseY - height - 10)
        width: hoverText.implicitWidth + 16
        height: hoverText.implicitHeight + 10
        radius: 6
        color: view.theme.surface
        border.color: view.theme.line
        Text { id: hoverText; anchors.centerIn: parent; text: view.hovered >= 0 && view.hovered < view.nodes.length ? view.nodes[view.hovered].title : ""; color: view.theme.foreground; font.pixelSize: Math.round(12 * view.theme.scale) }
    }

    // Top-left: what this is, and the count.
    Text {
        x: 12; y: 10
        text: (view.around.length > 0 ? "Local graph · " + view.around : "Graph view") + "  ·  " + view.nodes.filter(function (n) { return n.visible }).length + " nodes"
        color: view.theme.faint
        font.pixelSize: Math.round(11 * view.theme.scale)
    }
    Text { anchors.centerIn: parent; visible: view.loaded && view.nodes.length === 0; text: view.around.length > 0 ? "No links around this page yet" : "No pages yet"; color: view.theme.faint; font.pixelSize: Math.round(14 * view.theme.scale) }
    Text { x: 12; y: 28; visible: view.notice.length > 0; text: view.notice; color: view.theme.muted; font.pixelSize: Math.round(11 * view.theme.scale) }

    // The settings panel, as Obsidian's: four foldable sections at the top right.
    Rectangle {
        id: panel
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 12
        width: view.panelOpen ? 260 : 34
        height: view.panelOpen ? Math.min(parent.height - 24, panelColumn.implicitHeight + 16) : 34
        radius: 8
        color: view.theme.surface
        border.color: view.theme.line
        clip: true
        Behavior on width { NumberAnimation { duration: 120 } }
        Rectangle {
            visible: !view.panelOpen
            anchors.fill: parent
            color: "transparent"
            Icon { anchors.centerIn: parent; name: "settings"; color: view.theme.muted; size: 16 }
            HoverHandler { cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: view.panelOpen = true }
        }
        Flickable {
            visible: view.panelOpen
            anchors.fill: parent
            anchors.margins: 8
            contentHeight: panelColumn.implicitHeight
            clip: true
            ColumnLayout {
                id: panelColumn
                width: parent.width
                spacing: 4
                RowLayout {
                    Layout.fillWidth: true
                    Item { Layout.fillWidth: true }
                    PanelButton { icon: "refresh"; tip: "Restart the layout"; onClicked: { for (const n of view.nodes) { n.pinned = false }; view.restart() } }
                    PanelButton { icon: "collapse"; tip: "Fit to view"; onClicked: view.fit() }
                    PanelButton { icon: "close"; tip: "Hide the panel"; onClicked: view.panelOpen = false }
                }
                Section {
                    title: "Filters"
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4
                        TextField { Layout.fillWidth: true; placeholderText: "Search files…"; font.pixelSize: Math.round(12 * view.theme.scale); text: view.filter; onTextChanged: view.filter = text }
                        CheckBox { text: "Tags"; font.pixelSize: Math.round(12 * view.theme.scale); checked: view.showTags; onToggled: view.showTags = checked }
                        CheckBox { text: "Decision edges"; font.pixelSize: Math.round(12 * view.theme.scale); checked: view.showDecisions; onToggled: view.showDecisions = checked }
                        CheckBox { text: "Existing files only"; font.pixelSize: Math.round(12 * view.theme.scale); checked: !view.showUnresolved; onToggled: view.showUnresolved = !checked }
                        CheckBox { text: "Orphans"; font.pixelSize: Math.round(12 * view.theme.scale); checked: view.showOrphans; onToggled: view.showOrphans = checked }
                        RowLayout {
                            visible: view.around.length > 0
                            Text { text: "Depth"; color: view.theme.muted; font.pixelSize: Math.round(12 * view.theme.scale); Layout.preferredWidth: 90 }
                            Slider { Layout.fillWidth: true; from: 1; to: 4; stepSize: 1; value: view.depth; onMoved: view.depth = Math.round(value) }
                            Text { text: view.depth; color: view.theme.faint; font.pixelSize: Math.round(11 * view.theme.scale) }
                        }
                    }
                }
                Section {
                    title: "Groups"
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4
                        Repeater {
                            model: view.groups
                            delegate: RowLayout {
                                required property int index
                                required property var modelData
                                Layout.fillWidth: true
                                spacing: 6
                                Rectangle {
                                    width: 18; height: 18; radius: 9
                                    color: modelData.colour
                                    border.color: view.theme.line
                                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                                    TapHandler { onTapped: { const g = view.groups.slice(); const i = view.palette.indexOf(modelData.colour); g[index] = { query: modelData.query, colour: view.palette[(i + 1) % view.palette.length] }; view.groups = g } }
                                    ToolTip.visible: swatchHover.hovered
                                    ToolTip.text: "Next colour"
                                    HoverHandler { id: swatchHover }
                                }
                                TextField { Layout.fillWidth: true; font.pixelSize: Math.round(12 * view.theme.scale); text: modelData.query; placeholderText: "tag:x path:y type:z or text"; onEditingFinished: { if (text !== modelData.query) { const g = view.groups.slice(); g[index] = { query: text, colour: modelData.colour }; view.groups = g } } }
                                PanelButton { icon: "close"; tip: "Remove group"; onClicked: { const g = view.groups.slice(); g.splice(index, 1); view.groups = g } }
                            }
                        }
                        Text {
                            text: "+ New group"
                            color: newGroupHover.hovered ? view.theme.foreground : view.theme.muted
                            font.pixelSize: Math.round(12 * view.theme.scale)
                            HoverHandler { id: newGroupHover; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: { const g = view.groups.slice(); g.push({ query: "", colour: view.palette[(g.length + 1) % view.palette.length] }); view.groups = g } }
                        }
                    }
                }
                Section {
                    title: "Display"
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        CheckBox { text: "Arrows"; font.pixelSize: Math.round(12 * view.theme.scale); checked: view.arrows; onToggled: view.arrows = checked }
                        SettingSlider { label: "Text fade threshold"; from: 0; to: 1; value: view.textFade; onMoved: (v) => view.textFade = v }
                        SettingSlider { label: "Node size"; from: 0.3; to: 3; value: view.nodeSize; onMoved: (v) => view.nodeSize = v }
                        SettingSlider { label: "Link thickness"; from: 0.2; to: 4; value: view.linkThickness; onMoved: (v) => view.linkThickness = v }
                    }
                }
                Section {
                    title: "Forces"
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        SettingSlider { label: "Center force"; from: 0; to: 1; value: view.centerForce; onMoved: (v) => view.centerForce = v }
                        SettingSlider { label: "Repel force"; from: 0; to: 20; value: view.repelForce; onMoved: (v) => view.repelForce = v }
                        SettingSlider { label: "Link force"; from: 0; to: 1; value: view.linkForce; onMoved: (v) => view.linkForce = v }
                        SettingSlider { label: "Link distance"; from: 30; to: 500; value: view.linkDistance; onMoved: (v) => view.linkDistance = v }
                    }
                }
            }
        }
    }

    component Section: ColumnLayout {
        id: section
        property string title
        property bool open: true
        default property alias content: body.data
        Layout.fillWidth: true
        spacing: 2
        Rectangle {
            Layout.fillWidth: true
            height: 26
            radius: 4
            color: secHover.hovered ? view.theme.hover : "transparent"
            RowLayout {
                anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                Icon { name: section.open ? "chevron-down" : "chevron-right"; color: view.theme.faint; size: 14 }
                Text { text: section.title; color: view.theme.foreground; font.pixelSize: Math.round(13 * view.theme.scale); Layout.fillWidth: true }
            }
            HoverHandler { id: secHover; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: section.open = !section.open }
        }
        Item {
            id: body
            visible: section.open
            Layout.fillWidth: true
            Layout.leftMargin: 8
            implicitHeight: visible && children.length > 0 ? children[0].implicitHeight : 0
            onChildrenChanged: { for (const c of children) { c.width = Qt.binding(function () { return body.width }) } }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: view.theme.line; opacity: 0.5 }
    }

    component SettingSlider: RowLayout {
        id: ss
        property string label
        property real from: 0
        property real to: 1
        property real value: 0
        signal moved(real v)
        Layout.fillWidth: true
        spacing: 6
        Text { text: ss.label; color: view.theme.muted; font.pixelSize: Math.round(12 * view.theme.scale); Layout.preferredWidth: 110; elide: Text.ElideRight }
        Slider { Layout.fillWidth: true; from: ss.from; to: ss.to; value: ss.value; onMoved: ss.moved(value) }
    }

    component PanelButton: Rectangle {
        id: pb
        property string icon
        property string tip: ""
        signal clicked()
        width: 22; height: 22; radius: 4
        color: pbHover.hovered ? view.theme.hover : "transparent"
        Icon { anchors.centerIn: parent; name: pb.icon; color: view.theme.muted; size: 13 }
        HoverHandler { id: pbHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: pb.clicked() }
        ToolTip.visible: pbHover.hovered && pb.tip.length > 0
        ToolTip.text: pb.tip
        ToolTip.delay: 600
    }
}
