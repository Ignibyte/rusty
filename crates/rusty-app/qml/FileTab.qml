import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// A file from a folder root, read-only in part one (TICKET-016): markdown rendered like
// a page with a Source toggle, text in a monospace viewer with line numbers, an image
// fitted to the tab. The disk is read through `folders`; the renderer is the back end's.
Item {
    id: file
    required property var theme
    required property var backend
    required property var folders
    required property string path
    property string kind: ""
    property string text: ""
    property var lines: []
    property string html: ""
    property bool showSource: false
    property string notice: ""
    property var pending: ({})
    readonly property string name: folders.baseName(path)

    function style() {
        const t = JSON.parse(theme.tokens || "{}")
        return {
            text: theme.foreground, muted: theme.muted, link: theme.link, unresolved: theme.faint,
            accent: theme.accent, code: theme.code, code_bg: theme.codeBg, mono: theme.termFont,
            mark_bg: t.mark || theme.accent, line: theme.line, tag: theme.tag,
            red: t.red, green: t.green, yellow: t.yellow, blue: t.blue, magenta: t.magenta, cyan: t.cyan,
            headings: [t.h1, t.h2, t.h3, t.h4, t.h5, t.h6], size: Math.round(15 * file.theme.scale),
            bright: theme.bright, gold: theme.gold, alive: theme.alive, accent_soft: theme.accentSoft,
            panel3: theme.panel3, line_bright: theme.lineBright, marks: true, code_head: true
        }
    }
    function load() {
        kind = folders.kindOf(path)
        notice = ""
        if (kind === "image") return
        text = folders.readText(path)
        lines = text.length > 0 ? text.split("\n") : []
        if (kind === "markdown") render()
    }
    function render() {
        if (!backend.connected) { notice = backend.status; return }
        const id = backend.call("brain_render", JSON.stringify({ slug: "", markdown: text, style: style() }))
        const p = pending; p[id] = "render"; pending = p
    }
    Connections {
        target: file.backend
        function onResult(id, tool, json, ok) {
            const kind = file.pending[id]
            if (kind === undefined) return
            const p = file.pending; delete p[id]; file.pending = p
            if (!ok) { file.notice = tool + ": " + json; return }
            const r = JSON.parse(json)
            file.html = typeof r.html === "string" ? r.html : (r.blocks || []).map(function (b) { return typeof b === "string" ? b : (b.html || "") }).join("")
        }
    }
    Connections { target: file.theme; function onScaleChanged() { if (file.kind === "markdown") file.render() } }
    Component.onCompleted: load()

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        // The header: name, path, kind, and the controls.
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 12
            spacing: 10
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Text { text: file.name; color: file.theme.bright; font.pixelSize: Math.round(16 * file.theme.scale); font.bold: true; elide: Text.ElideMiddle; Layout.fillWidth: true }
                Text { text: file.path; color: file.theme.faint; font.pixelSize: Math.round(11 * file.theme.scale); font.family: file.theme.termFont; elide: Text.ElideMiddle; Layout.fillWidth: true }
            }
            Text { text: file.kind === "markdown" ? "markdown" : file.kind === "image" ? "image" : file.kind === "text" ? file.lines.length + " lines" : "file"; color: file.theme.muted; font.pixelSize: Math.round(11 * file.theme.scale) }
            Button { visible: file.kind === "markdown"; flat: true; text: file.showSource ? "Rendered" : "Source"; onClicked: file.showSource = !file.showSource }
            Button { flat: true; text: "Reload"; onClicked: file.load() }
            Button { flat: true; text: "Open outside"; onClicked: file.folders.openExternally(file.path) }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: file.theme.line }

        // Markdown, rendered by the back end the way a page is.
        Flickable {
            visible: file.kind === "markdown" && !file.showSource
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentHeight: rendered.height + 48
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            Text {
                id: rendered
                x: 24; y: 24
                width: parent.width - 48
                text: file.html
                textFormat: Text.RichText
                wrapMode: Text.Wrap
                color: file.theme.foreground
                font.pixelSize: Math.round(15 * file.theme.scale)
                onLinkActivated: (link) => Qt.openUrlExternally(link)
            }
        }

        // Text, and markdown's source: numbered monospace lines.
        ListView {
            id: source
            visible: file.kind === "text" || (file.kind === "markdown" && file.showSource)
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: file.lines
            topMargin: 8
            bottomMargin: 24
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            readonly property int gutter: Math.round((String(file.lines.length).length + 1) * 8 * file.theme.scale) + 12
            delegate: Item {
                required property int index
                required property string modelData
                width: source.width
                height: body.implicitHeight + 2
                Text { x: 0; width: source.gutter; text: index + 1; horizontalAlignment: Text.AlignRight; color: file.theme.faint; font.family: file.theme.termFont; font.pixelSize: Math.round(12 * file.theme.scale) }
                Text {
                    id: body
                    x: source.gutter + 14
                    width: source.width - x - 16
                    text: modelData.replace(/\t/g, "    ")
                    textFormat: Text.PlainText
                    wrapMode: Text.WrapAnywhere
                    color: file.theme.foreground
                    font.family: file.theme.termFont
                    font.pixelSize: Math.round(12 * file.theme.scale)
                }
            }
        }

        // An image, fitted.
        Item {
            visible: file.kind === "image"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Image {
                id: picture
                anchors.fill: parent
                anchors.margins: 24
                source: file.kind === "image" ? "file://" + file.path : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                smooth: true
            }
            Text { anchors.bottom: parent.bottom; anchors.right: parent.right; anchors.margins: 8; text: picture.sourceSize.width > 0 ? picture.sourceSize.width + " × " + picture.sourceSize.height : ""; color: file.theme.faint; font.pixelSize: Math.round(11 * file.theme.scale) }
        }

        // Anything the viewer does not read.
        Item {
            visible: file.kind === "other" || file.kind === ""
            Layout.fillWidth: true
            Layout.fillHeight: true
            Text { anchors.centerIn: parent; text: "Not a text file or an image; \"Open outside\" hands it to the desktop."; color: file.theme.muted; font.pixelSize: Math.round(13 * file.theme.scale) }
        }
        Text { visible: file.notice.length > 0; text: file.notice; color: file.theme.accent; font.pixelSize: Math.round(12 * file.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.margins: 12 }
    }
}
