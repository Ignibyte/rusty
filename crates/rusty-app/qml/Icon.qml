import QtQuick

// A line icon drawn from inline SVG in the given colour. The paths are the app's own,
// 24 by 24, stroked; names match what the ribbon, panes and menus ask for.
Item {
    id: icon
    property string name
    property color color: "#888888"
    property real size: 18
    property real stroke: 1.75
    implicitWidth: size
    implicitHeight: size

    readonly property var paths: ({
        "files": '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
        "search": '<circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>',
        "bookmark": '<path d="M6 3h12v18l-6-4-6 4z"/>',
        "graph": '<circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="12" cy="18" r="3"/><path d="M8.5 7.5l2 8M15.5 7.5l-2 8M9 6h6"/>',
        "calendar": '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 3v4M16 3v4"/>',
        "terminal": '<path d="m5 7 5 5-5 5M12 17h7"/>',
        "tasks": '<path d="m3 6 2 2 4-4M3 13l2 2 4-4M3 20l2 2 4-4M13 6h8M13 13h8M13 20h8"/>',
        "memory": '<path d="M9 4a3 3 0 0 0-3 3v1a3 3 0 0 0-2 5 3 3 0 0 0 2 5 3 3 0 0 0 3 3h3V4zM15 4a3 3 0 0 1 3 3v1a3 3 0 0 1 2 5 3 3 0 0 1-2 5 3 3 0 0 1-3 3h-3V4z"/>',
        "skills": '<path d="m12 3 1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z"/>',
        "secrets": '<circle cx="8" cy="15" r="4"/><path d="m10.9 12.1 8.6-8.6M16 6l3 3M13 9l3 3"/>',
        "settings": '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1"/>',
        "new-note": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5M12 11v6M9 14h6"/>',
        "new-folder": '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M12 10v6M9 13h6"/>',
        "collapse": '<path d="m7 20 5-5 5 5M7 4l5 5 5-5"/>',
        "sort": '<path d="M4 6h16M4 12h10M4 18h6"/>',
        "edit": '<path d="M17 3a2.8 2.8 0 0 1 4 4L7 21l-4 1 1-4z"/>',
        "read": '<path d="M2 4h6a4 4 0 0 1 4 4v12a3 3 0 0 0-3-3H2zM22 4h-6a4 4 0 0 0-4 4v12a3 3 0 0 1 3-3h7z"/>',
        "more": '<circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/>',
        "close": '<path d="M18 6 6 18M6 6l12 12"/>',
        "pin": '<path d="M12 17v5M9 3h6l-1 7 3 3H7l3-3z"/>',
        "link": '<path d="M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1"/><path d="M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1"/>',
        "outgoing": '<path d="M7 17 17 7M8 7h9v9"/>',
        "outline": '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
        "chevron-right": '<path d="m9 6 6 6-6 6"/>',
        "chevron-down": '<path d="m6 9 6 6 6-6"/>',
        "panel-left": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>',
        "panel-right": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/>',
        "arrow-left": '<path d="m12 19-7-7 7-7M5 12h14"/>',
        "arrow-right": '<path d="m12 5 7 7-7 7M5 12h14"/>',
        "plus": '<path d="M12 5v14M5 12h14"/>',
        "command": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/>',
        "help": '<circle cx="12" cy="12" r="9"/><path d="M9.5 9.5a2.5 2.5 0 0 1 5 0c0 1.5-2.5 2-2.5 3.5M12 17h.01"/>',
        "vault": '<path d="m7 15 5 5 5-5M7 9l5-5 5 5"/>',
        "agent": '<rect x="4" y="8" width="16" height="12" rx="2"/><path d="M12 4v4M9 13h.01M15 13h.01M9 17h6"/>',
        "refresh": '<path d="M21 12a9 9 0 1 1-3-6.7L21 8M21 3v5h-5"/>',
        "file": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>',
        "tag": '<path d="M3 12V4h8l9 9-8 8z"/><circle cx="7.5" cy="8.5" r="1"/>',
        "check": '<path d="m5 12 5 5L20 7"/>',
        "text": '<path d="M4 6h16M4 12h16M4 18h10"/>',
        "list": '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
        "hash": '<path d="M5 9h14M5 15h14M10 3 8 21M16 3l-2 18"/>',
        "check-square": '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="m8 12 3 3 5-6"/>',
        "clock": '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
        "inbox": '<path d="M3 13h5l2 3h4l2-3h5M5 5h14l2 8v6H3v-6z"/>',
        "daily": '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 3v4M16 3v4M9 15h2v3"/>',
        "unlink": '<path d="M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1M4 4l16 16"/>'
    })

    function hex(c) {
        function ch(v) { return Math.round(v * 255).toString(16).padStart(2, "0") }
        return "#" + ch(c.r) + ch(c.g) + ch(c.b)
    }

    Image {
        anchors.fill: parent
        sourceSize: Qt.size(icon.size * 2, icon.size * 2)
        smooth: true
        mipmap: true
        fillMode: Image.PreserveAspectFit
        source: "data:image/svg+xml;utf8," + encodeURIComponent(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="' + icon.hex(icon.color)
            + '" stroke-width="' + icon.stroke + '" stroke-linecap="round" stroke-linejoin="round">'
            + (icon.paths[icon.name] || '') + '</svg>')
    }
}
