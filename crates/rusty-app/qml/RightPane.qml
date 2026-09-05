import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The right sidebar: backlinks with their context lines, outgoing links (unresolved
// ones create the page), the outline, and an agent terminal beside the note.
Item {
    id: pane
    required property var backend
    required property var theme
    // The agent beside the note (TICKET-025): a headless Claude Code the window owns,
    // and the session id per page it keeps.
    required property var assistant
    property string sessions: "{}"
    property string agentSlug: ""
    // Whether the running process announced itself; an exit before that with a session
    // to resume means the session is gone, and it is forgotten so the next message
    // starts afresh instead of failing the same way.
    property bool sawInit: false
    property var note: null
    property var titles: ({})
    // [{tag, count}] from brain_tags, sorted; rows carry the depth and the last segment.
    property var tags: []
    readonly property var tagRows: tags.map(function (t) { return { tag: t.tag, count: t.count, depth: (t.tag.match(/\//g) || []).length, name: t.tag.split("/").pop() } })
    property string current: "backlinks"
    property bool windowActive: true
    signal openPage(string slug)
    signal createPage(string name)
    signal paneChanged(string name)
    signal searchTag(string tag)
    signal tagPage(string tag)
    signal bookmarkHeading(string text)
    signal sessionStarted(string slug, string sessionId)
    signal forgetSession(string slug)

    function titleOf(slug) { return titles[slug] || slug.slice(slug.lastIndexOf("/") + 1) }
    function focusAgent() { if (current === "agent") chatInput.forceActiveFocus() }

    // The conversation: one item per thing said or done. `extra` is a tool_use id on a
    // tool call and its result, a request id on a permission; `input` the JSON handed
    // back on Allow.
    ListModel { id: chat }
    function push(kind, name, text, extra, input) { chat.append({ kind: kind, name: name, text: text, extra: extra, input: input, answered: "" }) }
    function lastOf(kind) { for (let i = chat.count - 1; i >= 0; i--) if (chat.get(i).kind === kind) return i; return -1 }
    function brief(s) { return s.length > 600 ? s.slice(0, 600) + "…" : s }
    function sessionFor(slug) { try { const m = JSON.parse(sessions || "{}"); return typeof m[slug] === "string" ? m[slug] : "" } catch (e) { return "" } }
    function systemPromptFor(n) {
        return "You are the assistant beside a page in Rusty, a local-first knowledge workspace on this machine. The open page is `" + n.slug + "` (\"" + n.title + "\"). "
            + "Rusty's MCP tools (mcp__rusty__*) are the way to read and change pages, tasks, notes and memories; brain_read_page reads the open page by slug. Answer briefly."
    }
    // One process per page: the same page keeps its process, another page stops it,
    // clears the list and starts again with that page's session when there is one.
    function openAgent() {
        if (!assistant.available) return
        const slug = note ? note.slug : ""
        if (slug === agentSlug && assistant.running) return
        agentSlug = slug
        chat.clear()
        if (slug.length === 0) { assistant.stop(); return }
        const resume = sessionFor(slug)
        if (resume.length > 0) push("notice", "", "Continuing this page's conversation.", "", "")
        sawInit = false
        assistant.start(theme.homeDir, resume, systemPromptFor(note), backend.url)
    }
    function newConversation() { if (!note) return; forgetSession(note.slug); agentSlug = ""; assistant.stop(); openAgent() }
    function sendMessage() {
        const text = chatInput.text.trim()
        if (text.length === 0 || !note) return
        if (!assistant.running) { agentSlug = ""; openAgent() }
        if (!assistant.send(text)) { push("notice", "", "The assistant is not running; press New to start it.", "", ""); return }
        push("user", "", text, "", "")
        chatInput.text = ""
    }
    function askAgent(text) { chatInput.text = text; sendMessage() }
    function answerPermission(index, allow) {
        const item = chat.get(index)
        if (!item || item.answered.length > 0) return
        if (assistant.answer(item.extra, allow, item.input)) chat.setProperty(index, "answered", allow ? "Allowed" : "Denied")
    }
    onCurrentChanged: if (current === "agent") openAgent()
    onNoteChanged: if (current === "agent") openAgent()
    Connections {
        target: pane.assistant
        function onStarted(sessionId) { pane.sawInit = true; if (pane.agentSlug.length > 0) pane.sessionStarted(pane.agentSlug, sessionId) }
        function onBlockStarted(kind, name, id) {
            if (kind === "text") pane.push("text", "", "", "", "")
            else if (kind === "tool_use") pane.push("tool", name, "", id, "")
        }
        function onTextDelta(text) { const i = pane.lastOf("text"); if (i >= 0) chat.setProperty(i, "text", chat.get(i).text + text); else pane.push("text", "", text, "", "") }
        function onTextFinal(text) { const i = pane.lastOf("text"); if (i >= 0) chat.setProperty(i, "text", text); else pane.push("text", "", text, "", "") }
        function onToolInput(id, name, input) {
            for (let i = chat.count - 1; i >= 0; i--) { const it = chat.get(i); if (it.kind === "tool" && it.extra === id) { chat.setProperty(i, "text", pane.brief(input)); return } }
            pane.push("tool", name, pane.brief(input), id, "")
        }
        function onToolResult(id, text, isError) { pane.push("result", isError ? "error" : "", pane.brief(text), id, "") }
        function onPermissionAsked(requestId, tool, input, description) { pane.push("permission", tool, description.length > 0 ? description : pane.brief(input), requestId, input) }
        function onTurnDone(ok, cost, turns, text) { if (!ok && text.length > 0) pane.push("notice", "", text, "", "") }
        function onNotice(text) { pane.push("notice", "", text, "", "") }
        function onExited(code, message) {
            const stale = !pane.sawInit && pane.agentSlug.length > 0 && pane.sessionFor(pane.agentSlug).length > 0
            if (stale) pane.forgetSession(pane.agentSlug)
            pane.push("notice", "", "The assistant stopped" + (code !== 0 ? " (exit " + code + ")" : "") + (message.length > 0 ? ": " + message : "")
                + (stale ? ". The earlier session could not be resumed; the next message starts a new one." : ". Send a message to start it again."), "", "")
        }
    }
    function focusTags() { if (current === "tags") tagList.forceActiveFocus() }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // The mock's pane head: the pane's name, and the assistant's sigil and state.
        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 42
            Layout.leftMargin: 12
            Layout.rightMargin: 40
            spacing: 8
            Text { visible: pane.current === "agent"; text: "✦"; color: pane.theme.accent; font.pixelSize: Math.round(15 * pane.theme.scale) }
            Text { text: pane.current === "agent" ? "Rusty / Assistant" : pane.current === "backlinks" ? "Backlinks" : pane.current === "outgoing" ? "Outgoing links" : pane.current === "outline" ? "Outline" : "Tags"; color: pane.theme.bright; font.pixelSize: Math.round(10 * pane.theme.scale); font.letterSpacing: 1.2; font.capitalization: Font.AllUppercase }
            Item { Layout.fillWidth: true }
            Text { visible: pane.current === "agent"; text: "● " + (pane.assistant.running ? (pane.assistant.busy ? "Working" : "Ready") : "Idle"); color: pane.assistant.busy ? pane.theme.accent : pane.theme.alive; font.pixelSize: Math.round(9 * pane.theme.scale); font.letterSpacing: 1; font.capitalization: Font.AllUppercase }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: pane.theme.line }
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            Layout.topMargin: 4
            spacing: 2
            PaneTab { icon: "link"; name: "backlinks"; tip: "Backlinks" }
            PaneTab { icon: "outgoing"; name: "outgoing"; tip: "Outgoing links" }
            PaneTab { icon: "outline"; name: "outline"; tip: "Outline" }
            PaneTab { icon: "tag"; name: "tags"; tip: "Tags" }
            PaneTab { icon: "agent"; name: "agent"; tip: "Agent beside the note" }
            Item { Layout.fillWidth: true }
        }
        Rectangle { Layout.fillWidth: true; height: 1; color: pane.theme.line; opacity: 0.6; Layout.topMargin: 4 }

        // Backlinks
        ColumnLayout {
            visible: pane.current === "backlinks"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 4
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
            RowLayout {
                visible: pane.note !== null
                Text { text: "Linked mentions"; color: pane.theme.foreground; font.pixelSize: Math.round(13 * pane.theme.scale) }
                Rectangle { radius: 8; color: pane.theme.hover; width: countText.implicitWidth + 12; height: 18; Text { id: countText; anchors.centerIn: parent; text: pane.note ? pane.note.backlinkCount : 0; color: pane.theme.muted; font.pixelSize: Math.round(11 * pane.theme.scale) } }
            }
            ListView {
                id: backlinkList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 4
                model: pane.note && pane.note.links ? pane.note.links.backlinks : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property var modelData
                    width: backlinkList.width
                    height: bcol.implicitHeight + 12
                    radius: 4
                    color: bHover.hovered ? pane.theme.hover : "transparent"
                    ColumnLayout {
                        id: bcol
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 6
                        spacing: 2
                        Text { text: pane.titleOf(modelData.from_slug); color: pane.theme.foreground; font.pixelSize: Math.round(13 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                        Text { text: modelData.context; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); wrapMode: Text.Wrap; maximumLineCount: 3; elide: Text.ElideRight; Layout.fillWidth: true }
                    }
                    HoverHandler { id: bHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: pane.openPage(modelData.from_slug) }
                }
            }
            Text { visible: pane.note !== null && pane.note.backlinkCount === 0; text: "No backlinks found."; color: pane.theme.faint; font.pixelSize: Math.round(12 * pane.theme.scale) }
        }

        // Outgoing links
        ColumnLayout {
            visible: pane.current === "outgoing"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 4
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
            Text { visible: pane.note !== null; text: "Links"; color: pane.theme.foreground; font.pixelSize: Math.round(13 * pane.theme.scale) }
            ListView {
                id: outList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                model: pane.note && pane.note.links ? pane.note.links.outbound : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property var modelData
                    width: outList.width
                    height: 28
                    radius: 4
                    color: oHover.hovered ? pane.theme.hover : "transparent"
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 6; anchors.rightMargin: 6
                        spacing: 6
                        Icon { name: modelData.resolved ? "link" : "unlink"; color: modelData.resolved ? pane.theme.link : pane.theme.faint; size: 14 }
                        Text { text: modelData.resolved ? pane.titleOf(modelData.to_slug) : modelData.to_slug; color: modelData.resolved ? pane.theme.foreground : pane.theme.faint; font.pixelSize: Math.round(13 * pane.theme.scale); elide: Text.ElideMiddle; Layout.fillWidth: true }
                        Text { visible: !modelData.resolved; text: "create"; color: pane.theme.link; font.pixelSize: Math.round(11 * pane.theme.scale) }
                    }
                    HoverHandler { id: oHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: modelData.resolved ? pane.openPage(modelData.to_slug) : pane.createPage(modelData.to_slug) }
                }
            }
            Text { visible: pane.note !== null && pane.note.links !== null && pane.note.links.outbound.length === 0; text: "No links."; color: pane.theme.faint; font.pixelSize: Math.round(12 * pane.theme.scale) }
        }

        // Outline
        ColumnLayout {
            visible: pane.current === "outline"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 2
            Text { text: pane.note ? pane.note.title : "No file is open"; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
            ListView {
                id: outlineList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: pane.note ? pane.note.outline : []
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Rectangle {
                    required property int index
                    required property var modelData
                    width: outlineList.width
                    height: 26
                    radius: 4
                    color: hHover.hovered ? pane.theme.hover : "transparent"
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 8 + (modelData.level - 1) * 14
                        anchors.right: parent.right
                        text: modelData.text
                        color: modelData.level === 1 ? pane.theme.foreground : pane.theme.muted
                        font.pixelSize: Math.round(13 * pane.theme.scale)
                        elide: Text.ElideRight
                    }
                    HoverHandler { id: hHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: if (pane.note) pane.note.scrollToHeading(index) }
                    TapHandler { acceptedButtons: Qt.RightButton; onTapped: { headingMenu.heading = modelData.text; headingMenu.popup() } }
                }
            }
            Text { visible: pane.note !== null && pane.note.outline.length === 0; text: "No headings."; color: pane.theme.faint; font.pixelSize: Math.round(12 * pane.theme.scale) }
            Menu {
                id: headingMenu
                property string heading: ""
                MenuItem { text: "Bookmark heading"; onTriggered: pane.bookmarkHeading(headingMenu.heading) }
            }
        }

        // Tags: every tag in the vault as a tree with counts; a click searches by it.
        ColumnLayout {
            visible: pane.current === "tags"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: 8
            spacing: 2
            RowLayout {
                spacing: 6
                Text { text: "Tags"; color: pane.theme.foreground; font.pixelSize: Math.round(13 * pane.theme.scale) }
                Rectangle { radius: 8; color: pane.theme.hover; width: tagTotal.implicitWidth + 12; height: 18; Text { id: tagTotal; anchors.centerIn: parent; text: pane.tags.length; color: pane.theme.muted; font.pixelSize: Math.round(11 * pane.theme.scale) } }
            }
            ListView {
                id: tagList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: pane.tagRows
                currentIndex: -1
                keyNavigationEnabled: true
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                // The keyboard: Enter searches the current tag, T tags the open page with it
                // (TICKET-024).
                Keys.onReturnPressed: if (currentIndex >= 0) pane.searchTag(pane.tagRows[currentIndex].tag)
                Keys.onPressed: (event) => { if (event.key === Qt.Key_T && currentIndex >= 0 && pane.note) { pane.tagPage(pane.tagRows[currentIndex].tag); event.accepted = true } }
                delegate: Rectangle {
                    required property int index
                    required property var modelData
                    width: tagList.width
                    height: 26
                    radius: 4
                    color: tHover.hovered || tagList.currentIndex === index ? pane.theme.hover : "transparent"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 8 + modelData.depth * 14
                        anchors.rightMargin: 8
                        spacing: 6
                        Text { text: "#" + modelData.name; color: pane.theme.tag; font.pixelSize: Math.round(13 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                        // Tag the open page with this tag; the count follows the change.
                        Rectangle {
                            visible: tHover.hovered && pane.note !== null
                            width: 18; height: 18; radius: 4
                            color: plusHover.hovered ? pane.theme.active : "transparent"
                            Text { anchors.centerIn: parent; text: "+"; color: pane.theme.foreground; font.pixelSize: Math.round(14 * pane.theme.scale); font.bold: true }
                            HoverHandler { id: plusHover; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: pane.tagPage(modelData.tag) }
                            ToolTip.visible: plusHover.hovered
                            ToolTip.text: "Tag " + (pane.note ? pane.note.title : "the open page") + " with #" + modelData.tag
                            ToolTip.delay: 600
                        }
                        Text { text: modelData.count; color: pane.theme.faint; font.pixelSize: Math.round(11 * pane.theme.scale) }
                    }
                    HoverHandler { id: tHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { acceptedButtons: Qt.LeftButton; onTapped: { tagList.currentIndex = index; tagList.forceActiveFocus(); pane.searchTag(modelData.tag) } }
                    TapHandler { acceptedButtons: Qt.RightButton; onTapped: { tagList.currentIndex = index; tagList.forceActiveFocus(); tagMenu.tag = modelData.tag; tagMenu.popup() } }
                    ToolTip.visible: tHover.hovered && !plusHover.hovered && modelData.depth > 0
                    ToolTip.text: "#" + modelData.tag
                    ToolTip.delay: 600
                }
                Menu {
                    id: tagMenu
                    property string tag: ""
                    MenuItem { text: "Tag the open page"; enabled: pane.note !== null; onTriggered: pane.tagPage(tagMenu.tag) }
                    MenuItem { text: "Search #" + tagMenu.tag; onTriggered: pane.searchTag(tagMenu.tag) }
                }
            }
            Text { visible: pane.tags.length === 0; text: "No tags yet."; color: pane.theme.faint; font.pixelSize: Math.round(12 * pane.theme.scale) }
        }

        // The agent pane (TICKET-025): a conversation with a headless `claude -p` beside
        // the note, rendered here — text, tool calls, results and permission prompts as
        // their own items — while the terminal tabs stay real terminals.
        ColumnLayout {
            visible: pane.current === "agent"
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: 8
                spacing: 6
                Text { text: pane.note ? pane.note.title : "No page open"; color: pane.theme.foreground; font.pixelSize: Math.round(12 * pane.theme.scale); elide: Text.ElideRight; Layout.fillWidth: true }
                Text { text: pane.assistant.status; color: pane.assistant.busy ? pane.theme.accent : pane.theme.faint; font.pixelSize: Math.round(10 * pane.theme.scale); elide: Text.ElideRight; Layout.maximumWidth: 150 }
                Button { flat: true; text: "New"; visible: pane.assistant.available && pane.note !== null; ToolTip.text: "Start a new conversation for this page"; ToolTip.visible: hovered; ToolTip.delay: 600; onClicked: pane.newConversation() }
            }
            Rectangle { Layout.fillWidth: true; height: 1; color: pane.theme.line }
            ListView {
                id: chatList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: chat
                spacing: 6
                topMargin: 8
                bottomMargin: 8
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                // Follow the conversation as items arrive and as streamed text grows.
                onCountChanged: Qt.callLater(positionViewAtEnd)
                onContentHeightChanged: Qt.callLater(positionViewAtEnd)
                delegate: Item {
                    id: bubble
                    required property int index
                    required property string kind
                    required property string name
                    required property string text
                    required property string extra
                    required property string input
                    required property string answered
                    readonly property bool mono: kind === "tool" || kind === "result"
                    width: chatList.width
                    height: box.implicitHeight + 2
                    Rectangle {
                        id: box
                        x: bubble.kind === "user" ? 36 : 8
                        width: parent.width - 44
                        implicitHeight: col.implicitHeight + 16
                        radius: 6
                        color: bubble.kind === "user" ? pane.theme.active : bubble.kind === "permission" ? pane.theme.panel3 : bubble.kind === "notice" ? "transparent" : pane.theme.hover
                        border.width: bubble.kind === "permission" ? 1 : 0
                        border.color: pane.theme.accent
                        ColumnLayout {
                            id: col
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 8
                            spacing: 4
                            Text { Layout.fillWidth: true; elide: Text.ElideMiddle; visible: bubble.kind === "tool" || bubble.kind === "result" || bubble.kind === "permission"; text: bubble.kind === "tool" ? "⚙ " + bubble.name : bubble.kind === "result" ? (bubble.name === "error" ? "↳ error" : "↳ result") : "Permission: " + bubble.name; color: bubble.kind === "permission" ? pane.theme.accent : pane.theme.muted; font.pixelSize: Math.round(10 * pane.theme.scale); font.letterSpacing: 0.5 }
                            TextEdit {
                                Layout.fillWidth: true
                                visible: bubble.text.length > 0
                                text: bubble.text
                                readOnly: true
                                selectByMouse: true
                                wrapMode: TextEdit.Wrap
                                textFormat: TextEdit.PlainText
                                color: bubble.kind === "notice" ? pane.theme.faint : pane.theme.foreground
                                selectionColor: pane.theme.accent
                                font.family: bubble.mono ? pane.theme.termFont : Qt.application.font.family
                                font.pixelSize: Math.round((bubble.mono ? 11 : 13) * pane.theme.scale)
                            }
                            RowLayout {
                                visible: bubble.kind === "permission"
                                spacing: 6
                                Button { text: "Allow"; enabled: bubble.answered.length === 0; onClicked: pane.answerPermission(bubble.index, true) }
                                Button { text: "Deny"; flat: true; enabled: bubble.answered.length === 0; onClicked: pane.answerPermission(bubble.index, false) }
                                Text { visible: bubble.answered.length > 0; text: bubble.answered; color: pane.theme.faint; font.pixelSize: Math.round(11 * pane.theme.scale) }
                            }
                        }
                    }
                }
            }
            Text { visible: chat.count === 0 && pane.assistant.available; text: pane.note ? "Ask about " + pane.note.title + ". Claude Code answers here with Rusty's tools; a write asks first." : "Open a page to talk about it."; color: pane.theme.faint; font.pixelSize: Math.round(12 * pane.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.margins: 12 }
            Text { visible: !pane.assistant.available; text: "Claude Code is not installed: the pane needs `claude` on PATH. The terminal tabs still run any agent on the machine."; color: pane.theme.muted; font.pixelSize: Math.round(12 * pane.theme.scale); wrapMode: Text.Wrap; Layout.fillWidth: true; Layout.margins: 12 }
            Rectangle { Layout.fillWidth: true; height: 1; color: pane.theme.line }
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: 8
                spacing: 6
                visible: pane.assistant.available
                ScrollView {
                    Layout.fillWidth: true
                    Layout.maximumHeight: 120
                    TextArea {
                        id: chatInput
                        placeholderText: pane.note ? "Message Claude about this page — Enter sends, Shift+Enter breaks a line" : "Open a page first"
                        enabled: pane.note !== null
                        wrapMode: TextEdit.Wrap
                        font.pixelSize: Math.round(13 * pane.theme.scale)
                        Keys.onPressed: (event) => {
                            if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && !(event.modifiers & Qt.ShiftModifier)) { pane.sendMessage(); event.accepted = true }
                        }
                    }
                }
                Button { text: pane.assistant.busy ? "Stop" : "Send"; enabled: pane.note !== null && (pane.assistant.busy || chatInput.text.trim().length > 0); onClicked: pane.assistant.busy ? pane.assistant.interrupt() : pane.sendMessage() }
            }
        }
    }

    component PaneTab: Rectangle {
        id: pt
        property string icon
        property string name
        property string tip: ""
        width: 28
        height: 26
        radius: pane.theme.radius
        color: pane.current === name ? pane.theme.active : (ptHover.hovered ? pane.theme.hover : "transparent")
        Icon { anchors.centerIn: parent; name: pt.icon; color: pane.current === pt.name ? pane.theme.foreground : pane.theme.muted; size: 16 }
        HoverHandler { id: ptHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: { pane.current = pt.name; pane.paneChanged(pt.name) } }
        ToolTip.visible: ptHover.hovered && pt.tip.length > 0
        ToolTip.text: pt.tip
        ToolTip.delay: 600
    }
}
