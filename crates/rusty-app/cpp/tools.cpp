#include "tools.h"

#include <QImage>

bool Tools::grabWindow(QQuickWindow* window, const QString& path) {
    if (!window) return false;
    const QImage image = window->grabWindow();
    if (image.isNull()) return false;
    return image.save(path, "PNG");
}
