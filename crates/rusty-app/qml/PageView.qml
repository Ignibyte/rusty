import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// One brain page: title and facts, the compiled truth rendered as markdown (or edited
// in place), the timeline with an append box, links in and out, and a button that
// shows the page in Obsidian. Brain and Notes both embed it. Wikilinks render as links
// and navigate inside the app.
Item {
    id: view
    required property var backend
    required property var theme

    property string slug: ""
    property var page: null
    property var links: null
    property bool editing: false
    property string notice: ""
    property var pending: ({})

    signal navigate(string slug)

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function open(s) {
        slug = s
        editing = false
        notice = ""
        if (s.length === 0) { page = null; links = null; return }
        ask("brain_read_page", { slug: s }, "page")
        ask("brain_get_links", { slug: s }, "links")
    }
    function reload() { if (slug.length > 0) open(slug) }
    function save(content) { ask("brain_update_page", { slug: slug, content: content }, "saved") }
    function append(text) {
        if (text.trim().length === 0) return
        ask("brain_add_timeline", { slug: slug, summary: text.trim() }, "appended")
    }
    function openInObsidian() { ask("obsidian_open", { path: slug, new_tab: false }, "obsidian") }
    function follow(target) {
        const t = decodeURIComponent(target)
        if (t.indexOf("/") >= 0) navigate(t)
        else ask("brain_resolve_slug", { partial: t }, "resolve")
    }

    // [[a/b|Alias]] and [[a/b#Heading]] become markdown links on a rusty: scheme.
    function markdown(text) {
        if (!text) return ""
        return text.replace(/\[\[([^\]|#]+)(#[^\]|]*)?(\|([^\]]+))?\]\]/g, function (m, target, heading, p, alias) {
            const label = alias ? alias : target.split("/").pop()
            return "[" + label + "](rusty:" + encodeURIComponent(target.trim()) + ")"
        })
    }
    function onLink(link) {
        if (link.startsWith("rusty:")) follow(link.slice(6))
        else Qt.openUrlExternally(link)
    }

    Connections {
        target: view.backend
        function onResult(id, tool, json, ok) {
            const kind = view.pending[id]
            if (kind === undefined) return
            const p = view.pending; delete p[id]; view.pending = p
            if (!ok) { view.notice = tool + ": " + json; return }
            switch (kind) {
            case "page": {
                const data = JSON.parse(json)
                view.page = data
                if (data === null) view.notice = "no page at " + view.slug
                break
            }
            case "links": view.links = JSON.parse(json); break
            case "saved": view.editing = false; view.reload(); break
            case "appended": appendField.text = ""; view.reload(); break
            case "resolve": {
                const matches = JSON.parse(json)
                if (matches.length > 0) view.navigate(matches[0])
                else view.notice = "no page matches that link"
                break
            }
            case "obsidian": break
            }
        }
        function onDataChanged() { if (!view.editing) view.reload() }
    }

    Rectangle { anchors.fill: parent; color: "transparent" }

    Text {
        anchors.centerIn: parent
        visible: view.page === null
        text: view.slug.length === 0 ? "Pick a page" : (view.notice.length > 0 ? view.notice : "loading " + view.slug)
        color: view.theme.foreground
        opacity: 0.5
        font.pixelSize: 14
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 8
        visible: view.page !== null

        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            Text { text: view.page ? view.page.title : ""; color: view.theme.foreground; font.pixelSize: 22; font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight }
            Button { text: view.editing ? "Cancel" : "Edit"; onClicked: { if (view.editing) view.editing = false; else { editor.text = view.page.compiled_truth; view.editing = true; editor.forceActiveFocus() } } }
            Button { text: "Save"; visible: view.editing; highlighted: true; onClicked: view.save(editor.text) }
            Button { text: "Open in Obsidian"; onClicked: view.openInObsidian() }
        }
        Text {
            text: view.page ? (view.page.page_type + "  ·  " + view.page.slug + (view.page.frontmatter && view.page.frontmatter.updated ? "  ·  updated " + view.page.frontmatter.updated : "")) : ""
            color: view.theme.foreground; opacity: 0.55; font.pixelSize: 12
        }

        Flickable {
            id: scroller
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentHeight: body.implicitHeight
            clip: true
            ScrollBar.vertical: ScrollBar {}
            ColumnLayout {
                id: body
                width: scroller.width
                spacing: 14

                TextArea {
                    id: editor
                    visible: view.editing
                    Layout.fillWidth: true
                    Layout.minimumHeight: 320
                    wrapMode: TextEdit.Wrap
                    font.family: view.theme.termFont
                    font.pointSize: 10.5
                    color: view.theme.foreground
                    background: Rectangle { color: Qt.darker(view.theme.background, 1.15); radius: 6; border.color: view.theme.accent; border.width: 1 }
                    Keys.onPressed: (event) => { if (event.key === Qt.Key_S && (event.modifiers & Qt.ControlModifier)) { view.save(text); event.accepted = true } else if (event.key === Qt.Key_Escape) { view.editing = false; event.accepted = true } }
                }
                Text {
                    visible: !view.editing
                    Layout.fillWidth: true
                    text: view.page && view.page.compiled_truth.length > 0 ? view.markdown(view.page.compiled_truth) : "*empty*"
                    textFormat: Text.MarkdownText
                    wrapMode: Text.WordWrap
                    color: view.theme.foreground
                    linkColor: view.theme.accent
                    font.pixelSize: 14
                    onLinkActivated: (link) => view.onLink(link)
                }

                Text { text: "Timeline"; color: view.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 8 }
                Text {
                    Layout.fillWidth: true
                    visible: view.page && view.page.timeline.length > 0
                    text: view.page ? view.markdown(view.page.timeline) : ""
                    textFormat: Text.MarkdownText
                    wrapMode: Text.WordWrap
                    color: view.theme.foreground
                    linkColor: view.theme.accent
                    font.pixelSize: 13
                    onLinkActivated: (link) => view.onLink(link)
                }
                TextField {
                    id: appendField
                    Layout.fillWidth: true
                    placeholderText: "Append to the timeline and press Enter"
                    onAccepted: view.append(text)
                }

                Text { text: "Links"; color: view.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 8; visible: view.links !== null }
                Flow {
                    Layout.fillWidth: true
                    spacing: 6
                    visible: view.links !== null
                    Repeater {
                        model: view.links ? view.links.backlinks : []
                        delegate: Rectangle {
                            required property var modelData
                            radius: 4; color: Qt.rgba(1, 1, 1, 0.06)
                            width: backText.implicitWidth + 16; height: 24
                            Text { id: backText; anchors.centerIn: parent; text: "← " + modelData.from_slug; color: view.theme.accent; font.pixelSize: 12 }
                            TapHandler { onTapped: view.navigate(modelData.from_slug) }
                        }
                    }
                    Repeater {
                        model: view.links ? view.links.outbound : []
                        delegate: Rectangle {
                            required property var modelData
                            radius: 4; color: Qt.rgba(1, 1, 1, 0.06)
                            width: outText.implicitWidth + 16; height: 24
                            Text { id: outText; anchors.centerIn: parent; text: modelData.to_slug + " →"; color: view.theme.foreground; font.pixelSize: 12 }
                            TapHandler { onTapped: view.follow(modelData.to_slug) }
                        }
                    }
                    Text { visible: view.links && view.links.backlinks.length === 0 && view.links.outbound.length === 0; text: "none"; color: view.theme.foreground; opacity: 0.5; font.pixelSize: 12 }
                }
                Text { text: view.notice; visible: view.notice.length > 0 && view.page !== null; color: view.theme.accent; font.pixelSize: 12; wrapMode: Text.WordWrap; Layout.fillWidth: true }
            }
        }
    }
}
