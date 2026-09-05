// Small things QML cannot do on its own: grab the window into a PNG (for screenshots
// of the record, taken offscreen against a scratch vault), and split a page into its
// sections through the Rust rule (TICKET-028).
#pragma once

#include <QObject>
#include <QQuickWindow>
#include <QString>
#include <QStringList>
#include <QtQml/qqmlregistration.h>

class Tools : public QObject {
    Q_OBJECT
    QML_ELEMENT

public:
    explicit Tools(QObject* parent = nullptr) : QObject(parent) {}

    // Render the window (exposed or not) and save it; returns whether the file was written.
    Q_INVOKABLE bool grabWindow(QQuickWindow* window, const QString& path);

    // The page as the frontmatter (or an empty string) and one part per section, from
    // the Rust rule; the parts joined are the page again.
    Q_INVOKABLE QStringList pageSections(const QString& raw);
};
