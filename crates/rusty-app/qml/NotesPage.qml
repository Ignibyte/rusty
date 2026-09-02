import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The Notes tab: daily pages, which live in the brain vault so Obsidian sees them too.
// "Today" opens or creates today's page; older days are listed newest first.
Item {
    id: page
    required property var backend
    required property var theme

    property var days: []
    property string notice: ""
    property var pending: ({})

    function ask(tool, args, kind) {
        const id = backend.call(tool, JSON.stringify(args))
        const p = pending; p[id] = kind; pending = p
    }
    function refresh() { ask("brain_list_pages", { page_type: "daily", limit: 400 }, "days") }
    function today() { ask("brain_daily_note", {}, "today") }
    function focusEntry() { if (pageView.slug.length === 0) todayButton.forceActiveFocus() }

    Connections {
        target: page.backend
        function onResult(id, tool, json, ok) {
            const kind = page.pending[id]
            if (kind === undefined) return
            const p = page.pending; delete p[id]; page.pending = p
            if (!ok) { page.notice = tool + ": " + json; return }
            page.notice = ""
            if (kind === "days") {
                const list = JSON.parse(json)
                list.sort((a, b) => a.slug < b.slug ? 1 : -1)
                page.days = list
                if (pageView.slug.length === 0 && list.length > 0) pageView.open(list[0].slug)
            } else if (kind === "today") {
                const data = JSON.parse(json)
                pageView.open(data.slug)
                page.refresh()
            }
        }
        function onDataChanged() { page.refresh() }
    }
    Component.onCompleted: if (backend.connected) refresh()

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 220
            Layout.fillHeight: true
            color: Qt.darker(page.theme.background, 1.08)
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 6
                Button { id: todayButton; text: "Today"; highlighted: true; Layout.fillWidth: true; onClicked: page.today() }
                Text { text: "Daily pages"; color: page.theme.foreground; opacity: 0.6; font.pixelSize: 12; font.bold: true; Layout.topMargin: 6 }
                ListView {
                    id: dayList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: page.days
                    spacing: 2
                    delegate: Rectangle {
                        required property var modelData
                        width: dayList.width
                        height: 30
                        radius: 6
                        color: pageView.slug === modelData.slug ? page.theme.accent : (dayHover.hovered ? Qt.rgba(1, 1, 1, 0.06) : "transparent")
                        Text {
                            anchors.verticalCenter: parent.verticalCenter; anchors.left: parent.left; anchors.leftMargin: 10
                            text: modelData.title
                            font.pixelSize: 14
                            color: pageView.slug === modelData.slug ? page.theme.background : page.theme.foreground
                        }
                        HoverHandler { id: dayHover }
                        TapHandler { onTapped: pageView.open(modelData.slug) }
                    }
                }
                Text { text: page.notice; visible: page.notice.length > 0; color: page.theme.accent; font.pixelSize: 11; wrapMode: Text.WordWrap; Layout.fillWidth: true }
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
