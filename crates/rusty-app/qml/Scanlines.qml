import QtQuick

// The mock's CRT overlay: a faint scanline every four pixels and a vignette towards
// the corners, painted once per size and laid over the window without taking input.
Canvas {
    id: crt
    property real strength: 0.16
    opacity: strength
    enabled: false
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onPaint: {
        const ctx = getContext("2d")
        ctx.clearRect(0, 0, width, height)
        ctx.fillStyle = "rgba(255,255,255,0.11)"
        for (let y = 0; y < height; y += 4) ctx.fillRect(0, y, width, 2)
        const g = ctx.createRadialGradient(width / 2, height / 2, Math.min(width, height) * 0.35, width / 2, height / 2, Math.max(width, height) * 0.75)
        g.addColorStop(0, "rgba(0,0,0,0)")
        g.addColorStop(1, "rgba(0,0,0,0.9)")
        ctx.fillStyle = g
        ctx.fillRect(0, 0, width, height)
    }
}
