// The source editor's syntax highlighter: a QSyntaxHighlighter the QML editor attaches to
// its text document. Every block is tokenized by the Rust side (src/markdown.rs); this
// class only maps span kinds to formats built from the theme's tokens.
#pragma once

#include <QColor>
#include <QHash>
#include <QObject>
#include <QQuickTextDocument>
#include <QString>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QtQml/qqmlregistration.h>

class MarkdownHighlighter : public QSyntaxHighlighter {
    Q_OBJECT
    QML_ELEMENT
    // The QML TextArea's textDocument.
    Q_PROPERTY(QQuickTextDocument* target READ target WRITE setTarget NOTIFY targetChanged)
    // JSON object of colour tokens: text, muted, accent, link, code, tag, red, green,
    // yellow, blue, magenta, cyan, h1 to h6. Unknown keys are ignored.
    Q_PROPERTY(QString tokens READ tokens WRITE setTokens NOTIFY tokensChanged)
    // The monospace family used for code spans.
    Q_PROPERTY(QString monoFamily READ monoFamily WRITE setMonoFamily NOTIFY monoFamilyChanged)

public:
    explicit MarkdownHighlighter(QObject* parent = nullptr);

    QQuickTextDocument* target() const { return m_target; }
    void setTarget(QQuickTextDocument* target);
    QString tokens() const { return m_tokens; }
    void setTokens(const QString& tokens);
    QString monoFamily() const { return m_mono; }
    void setMonoFamily(const QString& family);

signals:
    void targetChanged();
    void tokensChanged();
    void monoFamilyChanged();

protected:
    void highlightBlock(const QString& text) override;

private:
    QTextCharFormat formatFor(quint8 kind) const;
    QColor colour(const char* key, const QColor& fallback) const;

    QQuickTextDocument* m_target = nullptr;
    QString m_tokens;
    QString m_mono = QStringLiteral("monospace");
    QHash<QString, QColor> m_colours;
};
