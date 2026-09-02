import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Brain tab: search or browse by type on the left, the page on the right, and a
// capture box that drops a line into today's daily page or the inbox.
Item {
    id: page
    required property var backend
    required property var theme

    property var types: []
    property var pagesByType: ({})
    property var expanded: ({})
    property var results: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() {
        ask("brain_page_types", {}, "types")
        for (const t in expanded) if (expanded[t]) loadType(t)
    }
    function loadType(t) { ask("brain_list_pages", { page_type: t, limit: 500 }, "pages:" + t) }
    function toggleType(t) {
        const e = expanded; e[t] = !e[t]; expanded = e
        if (e[t] && !(t in pagesByType)) loadType(t)
    }
    function search(q) {
        if (q.trim().length === 0) { results = []; return }
        ask("brain_search", { query: q.trim(), limit: 40 }, "search")
    }
    function capture(text, target) {
        if (text.trim().length === 0) return
        ask("brain_capture", { text: text.trim(), target: target }, "capture")
    }
    function focusEntry() { searchField.forceActiveFocus() }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            if (kind === "types") page.types = JSON.parse(json).filter(t => t.count > 0)
            else if (kind.startsWith("pages:")) { const m = page.pagesByType; m[kind.slice(6)] = JSON.parse(json); page.pagesByType = m }
            else if (kind === "search") page.results = JSON.parse(json)
            else if (kind === "capture") { captureField.text = ""; const r = JSON.parse(json); page.notice = "captured to " + r.slug }
        }
        function onDataChanged() { page.refresh(); if (searchField.text.length > 0) page.search(searchField.text) }
    }
    Component.onCompleted: if (backend.connected) refresh()

    Timer { id: searchDebounce; interval: 250; onTriggered: page.search(searchField.text) }

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
                spacing: 8
                TextField {
                    id: searchField
                    Layout.fillWidth: true
                    placeholderText: "Search the brain"
                    onTextChanged: searchDebounce.restart()
                    Keys.onEscapePressed: text = ""
                }
                Flickable {
                    id: tree
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    contentHeight: treeColumn.implicitHeight
                    clip: true
                    ScrollBar.vertical: ScrollBar {}
                    ColumnLayout {
                        id: treeColumn
                        width: tree.width
                        spacing: 2
                        // Search results
                        Repeater {
                            model: searchField.text.trim().length > 0 ? page.results : []
                            delegate: Rectangle {
                                required property var modelData
                                Layout.fillWidth: true
                                height: hitCol.implicitHeight + 10
                                radius: 6
                                color: pageView.slug === modelData.slug ? Qt.rgba(1, 1, 1, 0.1) : (hitHover.hovered ? Qt.rgba(1, 1, 1, 0.05) : "transparent")
                                ColumnLayout {
                                    id: hitCol
                                    anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 5
                                    spacing: 2
                                    Text { text: modelData.title; color: page.theme.foreground; font.pixelSize: 13; elide: Text.ElideRight; Layout.fillWidth: true }
                                    Text { text: modelData.page_type + " · " + modelData.slug; color: page.theme.foreground; opacity: 0.5; font.pixelSize: 11; elide: Text.ElideRight; Layout.fillWidth: true }
                                }
                                HoverHandler { id: hitHover }
                                TapHandler { onTapped: pageView.open(modelData.slug) }
                            }
                        }
                        Text { visible: searchField.text.trim().length > 0 && page.results.length === 0; text: "no matches"; color: page.theme.foreground; opacity: 0.5; font.pixelSize: 12; Layout.leftMargin: 6 }
                        // Type tree
                        Repeater {
                            model: searchField.text.trim().length > 0 ? [] : page.types
                            delegate: ColumnLayout {
                                id: typeRow
                                required property var modelData
                                Layout.fillWidth: true
                                spacing: 1
                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 28
                                    radius: 6
                                    color: typeHover.hovered ? Qt.rgba(1, 1, 1, 0.05) : "transparent"
                                    RowLayout {
                                        anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8
                                        Text { text: page.expanded[typeRow.modelData.page_type] ? "▾" : "▸"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12 }
                                        Text { text: typeRow.modelData.dir; color: page.theme.foreground; font.pixelSize: 14; Layout.fillWidth: true }
                                        Text { text: typeRow.modelData.count; color: page.theme.foreground; opacity: 0.5; font.pixelSize: 12 }
                                    }
                                    HoverHandler { id: typeHover }
                                    TapHandler { onTapped: page.toggleType(typeRow.modelData.page_type) }
                                }
                                Repeater {
                                    model: page.expanded[typeRow.modelData.page_type] ? (page.pagesByType[typeRow.modelData.page_type] || []) : []
                                    delegate: Rectangle {
                                        required property var modelData
                                        Layout.fillWidth: true
                                        Layout.leftMargin: 18
                                        height: 26
                                        radius: 6
                                        color: pageView.slug === modelData.slug ? page.theme.accent : (leafHover.hovered ? Qt.rgba(1, 1, 1, 0.05) : "transparent")
                                        Text {
                                            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 8; anchors.right: parent.right; anchors.rightMargin: 6
                                            text: modelData.title; elide: Text.ElideRight; font.pixelSize: 13
                                            color: pageView.slug === modelData.slug ? page.theme.background : page.theme.foreground
                                        }
                                        HoverHandler { id: leafHover }
                                        TapHandler { onTapped: pageView.open(modelData.slug) }
                                    }
                                }
                            }
                        }
                    }
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: page.theme.accent; opacity: 0.25 }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    TextField { id: captureField; Layout.fillWidth: true; placeholderText: "Capture a line…"; onAccepted: page.capture(text, captureTarget.currentText) }
                    ComboBox { id: captureTarget; model: ["daily", "inbox"]; Layout.preferredWidth: 90 }
                }
                Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: 11; elide: Text.ElideRight; Layout.fillWidth: true }
            }
        }
        Rectangle { width: 1; Layout.fillHeight: true; color: page.theme.accent; opacity: 0.25 }

        PageView {
            id: pageView
            Layout.fillWidth: true
            Layout.fillHeight: true
            backend: page.backend
            theme: page.theme
            onNavigate: (slug) => open(slug)
        }
    }
}
