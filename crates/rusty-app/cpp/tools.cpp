#include "tools.h"

#include <QImage>

#include "rusty-app/src/markdown.cxxqt.h"

bool Tools::grabWindow(QQuickWindow* window, const QString& path) {
    if (!window) return false;
    const QImage image = window->grabWindow();
    if (image.isNull()) return false;
    return image.save(path, "PNG");
}

QStringList Tools::pageSections(const QString& raw) {
    const QByteArray utf8 = raw.toUtf8();
    const rust::Vec<rust::String> parts =
        rusty::page_sections(rust::Str(utf8.constData(), static_cast<std::size_t>(utf8.size())));
    QStringList out;
    out.reserve(static_cast<int>(parts.size()));
    for (const rust::String& part : parts) {
        out << QString::fromUtf8(part.data(), static_cast<int>(part.size()));
    }
    return out;
}
