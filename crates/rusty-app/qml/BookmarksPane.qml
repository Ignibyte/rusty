import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import dev.ignibyte.rusty

// The Bookmarks tab of the left sidebar: the files, folders, searches and headings the
// user keeps, in the order they were added. A click opens the target; the row menu
// retitles or removes the bookmark. The list itself lives in the window's state.
Item {
    id: pane
    required property var theme
    property var bookmarks: []
    signal openBookmark(var bookmark)
    signal removeBookmark(int index)
    signal retitleBookmark(int index, string title)

    function iconFor(kind) {
        return kind === "folder" ? "folder" : kind === "search" ? "search" : kind === "heading" ? "hash" : "file"
    }
    function detailOf(b) {
        return b.kind === "search" ? b.query : b.kind === "heading" ? b.path + " › " + b.heading : b.path
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6
        Text {
            text: pane.bookmarks.length === 0 ? "No bookmarks" : pane.bookmarks.length + (pane.bookmarks.length === 1 ? " bookmark" : " bookmarks")
            color: pane.theme.faint
            font.pixelSize: 11
        }
        Text {
            visible: pane.bookmarks.length === 0
            text: "Bookmark a page from its menu, a folder from the file explorer, a search from the search pane, a heading from the outline."
            color: pane.theme.faint
            font.pixelSize: 12
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: pane.bookmarks
            spacing: 1
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                id: row
                required property int index
                required property var modelData
                width: list.width
                height: 30
                radius: 4
                color: rowHover.hovered ? pane.theme.hover : "transparent"
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 6
                    spacing: 8
                    Icon { name: pane.iconFor(row.modelData.kind); color: pane.theme.muted; size: 14 }
                    Text { text: row.modelData.title; color: pane.theme.foreground; font.pixelSize: 13; elide: Text.ElideRight; Layout.fillWidth: true }
                    Text { text: pane.detailOf(row.modelData); color: pane.theme.faint; font.pixelSize: 11; elide: Text.ElideMiddle; Layout.maximumWidth: row.width * 0.45 }
                }
                HoverHandler { id: rowHover; cursorShape: Qt.PointingHandCursor }
                TapHandler { acceptedButtons: Qt.LeftButton; onTapped: pane.openBookmark(row.modelData) }
                TapHandler { acceptedButtons: Qt.RightButton; onTapped: { rowMenu.index = row.index; rowMenu.popup() } }
            }
        }
    }

    Menu {
        id: rowMenu
        property int index: -1
        MenuItem { text: "Edit title…"; onTriggered: titleDialog.openFor(rowMenu.index) }
        MenuItem { text: "Remove bookmark"; onTriggered: pane.removeBookmark(rowMenu.index) }
    }

    Dialog {
        id: titleDialog
        title: "Bookmark title"
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok | Dialog.Cancel
        property int index: -1
        function openFor(i) {
            index = i
            titleField.text = pane.bookmarks[i].title
            open()
            titleField.forceActiveFocus()
            titleField.selectAll()
        }
        onAccepted: if (titleField.text.trim().length > 0) pane.retitleBookmark(index, titleField.text.trim())
        TextField { id: titleField; width: 320; onAccepted: titleDialog.accept() }
    }
}
