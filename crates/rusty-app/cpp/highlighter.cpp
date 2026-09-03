#include "highlighter.h"

#include <QFont>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTextDocument>

#include "rusty-app/src/markdown.cxxqt.h"

MarkdownHighlighter::MarkdownHighlighter(QObject* parent) : QSyntaxHighlighter(parent) {}

void MarkdownHighlighter::setTarget(QQuickTextDocument* target) {
    if (m_target == target) return;
    m_target = target;
    setDocument(target ? target->textDocument() : nullptr);
    emit targetChanged();
}

void MarkdownHighlighter::setTokens(const QString& tokens) {
    if (m_tokens == tokens) return;
    m_tokens = tokens;
    m_colours.clear();
    const QJsonObject object = QJsonDocument::fromJson(tokens.toUtf8()).object();
    for (auto it = object.begin(); it != object.end(); ++it) {
        const QColor c(it.value().toString());
        if (c.isValid()) m_colours.insert(it.key(), c);
    }
    emit tokensChanged();
    rehighlight();
}

void MarkdownHighlighter::setMonoFamily(const QString& family) {
    if (m_mono == family) return;
    m_mono = family;
    emit monoFamilyChanged();
    rehighlight();
}

QColor MarkdownHighlighter::colour(const char* key, const QColor& fallback) const {
    const auto it = m_colours.constFind(QString::fromLatin1(key));
    return it == m_colours.constEnd() ? fallback : *it;
}

QTextCharFormat MarkdownHighlighter::formatFor(quint8 kind) const {
    QTextCharFormat f;
    const QColor text = colour("text", QColor(0xa9, 0xb1, 0xd6));
    const QColor muted = colour("muted", QColor(0x78, 0x7c, 0x99));
    const QColor accent = colour("accent", QColor(0x7a, 0xa2, 0xf7));
    if (kind >= 1 && kind <= 6) {
        static const char* keys[] = {"h1", "h2", "h3", "h4", "h5", "h6"};
        static const double scale[] = {1.6, 1.4, 1.25, 1.15, 1.05, 1.0};
        f.setForeground(colour(keys[kind - 1], accent));
        f.setFontWeight(QFont::DemiBold);
        const qreal base = document() ? document()->defaultFont().pointSizeF() : 11.0;
        f.setFontPointSize((base > 0 ? base : 11.0) * scale[kind - 1]);
        return f;
    }
    switch (kind) {
    case 10: f.setFontItalic(true); break;
    case 11: f.setFontWeight(QFont::Bold); break;
    case 12: f.setFontFamilies({m_mono}); f.setForeground(colour("code", colour("cyan", accent))); break;
    case 13: f.setFontFamilies({m_mono}); f.setForeground(colour("code", colour("cyan", accent))); break;
    case 14: f.setForeground(colour("link", accent)); break;
    case 15: f.setForeground(colour("link", accent)); f.setFontUnderline(true); break;
    case 16: f.setForeground(colour("tag", colour("cyan", accent))); break;
    case 17: f.setForeground(accent); f.setFontWeight(QFont::Bold); break;
    case 18: f.setForeground(accent); break;
    case 19: f.setForeground(accent); break;
    case 20: f.setForeground(muted); break;
    case 21: f.setForeground(muted); f.setFontItalic(true); break;
    case 22: f.setBackground(colour("mark", colour("yellow", accent).darker(180))); break;
    case 23: f.setFontStrikeOut(true); f.setForeground(muted); break;
    case 24: f.setForeground(muted); break;
    case 25: f.setForeground(muted); break;
    case 26: f.setForeground(colour("yellow", accent)); f.setFontWeight(QFont::Bold); break;
    case 27: f.setFontFamilies({m_mono}); f.setForeground(colour("magenta", accent)); break;
    case 28: f.setForeground(muted); break;
    default: f.setForeground(text); break;
    }
    return f;
}

void MarkdownHighlighter::highlightBlock(const QString& text) {
    const QByteArray utf8 = text.toUtf8();
    const rusty::LineSpans result = rusty::highlight_line(
        rust::Str(utf8.constData(), static_cast<std::size_t>(utf8.size())),
        previousBlockState());
    setCurrentBlockState(result.state);
    for (const rusty::Span& span : result.spans) {
        setFormat(static_cast<int>(span.start), static_cast<int>(span.len), formatFor(span.kind));
    }
}
