// Small things QML cannot do on its own: grab the window into a PNG (for screenshots
// of the record, taken offscreen against a scratch vault).
#pragma once

#include <QObject>
#include <QQuickWindow>
#include <QString>
#include <QtQml/qqmlregistration.h>

class Tools : public QObject {
    Q_OBJECT
    QML_ELEMENT

public:
    explicit Tools(QObject* parent = nullptr) : QObject(parent) {}

    // Render the window (exposed or not) and save it; returns whether the file was written.
    Q_INVOKABLE bool grabWindow(QQuickWindow* window, const QString& path);
};
