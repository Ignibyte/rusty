//! The source editor's line tokenizer: the spans a `QSyntaxHighlighter` colours for one
//! line of Obsidian-flavoured markdown, with the state the next line starts in (inside a
//! fence, the frontmatter, or a `%%` comment). The rules live here, tested without Qt;
//! `cpp/highlighter.cpp` is the thin subclass that asks for them per block.

/// Bit set when the line is inside a fenced code block.
pub const IN_FENCE: i32 = 1;
/// Bit set when the line is inside the frontmatter.
pub const IN_FRONTMATTER: i32 = 2;
/// Bit set when the line is inside a `%%` comment that spans lines.
pub const IN_COMMENT: i32 = 4;

/// Span kinds, shared with the C++ side by number.
pub mod kind {
    /// Heading levels are 1 to 6, so a heading's kind is its level.
    pub const HEADING_MAX: u8 = 6;
    /// `*emphasis*` or `_emphasis_`.
    pub const EMPHASIS: u8 = 10;
    /// `**strong**` or `__strong__`.
    pub const STRONG: u8 = 11;
    /// `` `inline code` ``.
    pub const CODE: u8 = 12;
    /// A line of a fenced code block, or the fence itself.
    pub const CODE_BLOCK: u8 = 13;
    /// `[[wikilink]]` or `[text](url)`.
    pub const LINK: u8 = 14;
    /// A bare URL.
    pub const URL: u8 = 15;
    /// `#tag`.
    pub const TAG: u8 = 16;
    /// The `>` of a quote line.
    pub const QUOTE: u8 = 17;
    /// A list marker: `-`, `*`, `+`, `1.`.
    pub const LIST: u8 = 18;
    /// A task box `[ ]` or `[x]`.
    pub const TASK: u8 = 19;
    /// The frontmatter fences and their YAML.
    pub const FRONTMATTER: u8 = 20;
    /// `%% comment %%`.
    pub const COMMENT: u8 = 21;
    /// `==highlight==`.
    pub const MARK: u8 = 22;
    /// `~~strike~~`.
    pub const STRIKE: u8 = 23;
    /// A horizontal rule.
    pub const RULE: u8 = 24;
    /// An HTML tag.
    pub const HTML: u8 = 25;
    /// `[!kind]` at the head of a callout.
    pub const CALLOUT: u8 = 26;
    /// `$math$`.
    pub const MATH: u8 = 27;
    /// A table row's pipes.
    pub const TABLE: u8 = 28;
}

#[cxx_qt::bridge(namespace = "rusty")]
mod ffi {
    /// One coloured run of a line, in UTF-16 units as Qt counts them.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Span {
        /// Start position.
        start: u32,
        /// Length.
        len: u32,
        /// See [`super::kind`].
        kind: u8,
    }

    /// The spans of one line and the state the next line starts in.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct LineSpans {
        /// The runs to colour.
        spans: Vec<Span>,
        /// The block state for the next line ([`super::IN_FENCE`] and friends).
        state: i32,
    }

    extern "Rust" {
        /// Tokenize one line. `prev_state` is the previous block's state, or -1 for the
        /// first line of the document.
        fn highlight_line(line: &str, prev_state: i32) -> LineSpans;
        /// A page as the frontmatter (or an empty string) followed by one part per
        /// section, joining back to the page byte for byte (TICKET-028).
        fn page_sections(raw: &str) -> Vec<String>;
    }
}

pub use ffi::{LineSpans, Span};

/// A page as parts: the frontmatter (or an empty string) first, then the text before
/// the first heading when there is any, then one part per heading line (`#` × 1–6 and
/// a space, or `#`s alone) outside fenced code (``` or `~~~` toggle a fence). The parts
/// joined give the page back byte for byte, which is what lets a section be edited on
/// its own and the page assembled from the rest (TICKET-028).
pub fn page_sections(raw: &str) -> Vec<String> {
    let (frontmatter, body) = match raw.strip_prefix("---\n") {
        Some(rest) => match rest.find("\n---\n") {
            Some(i) => raw.split_at(4 + i + 5),
            None => match rest.strip_suffix("\n---") {
                Some(_) => (raw, ""),
                None => ("", raw),
            },
        },
        None => ("", raw),
    };
    let mut parts = vec![frontmatter.to_string()];
    let mut current = String::new();
    let mut in_fence = false;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_end_matches('\n');
        let start = trimmed.trim_start();
        if start.starts_with("```") || start.starts_with("~~~") {
            in_fence = !in_fence;
        } else if !in_fence && is_heading_line(trimmed) && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// `#` × 1–6 then a space or nothing else: an ATX heading line.
fn is_heading_line(line: &str) -> bool {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&hashes) && line[hashes..].chars().next().is_none_or(|c| c == ' ')
}

/// A span in byte offsets, converted to UTF-16 units at the end.
struct ByteSpan {
    start: usize,
    end: usize,
    kind: u8,
}

/// Tokenize one line; the bridge function of the same name hands this to C++.
pub fn highlight_line(line: &str, prev_state: i32) -> LineSpans {
    let first = prev_state < 0;
    let mut state = prev_state.max(0);
    let mut spans: Vec<ByteSpan> = Vec::new();
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let whole = |kind: u8| ByteSpan {
        start: 0,
        end: line.len(),
        kind,
    };

    // Frontmatter: only at the top of the file.
    if first && trimmed.trim_end() == "---" {
        return finish(line, vec![whole(kind::FRONTMATTER)], IN_FRONTMATTER);
    }
    if state & IN_FRONTMATTER != 0 {
        if trimmed.trim_end() == "---" {
            state &= !IN_FRONTMATTER;
        }
        return finish(line, vec![whole(kind::FRONTMATTER)], state);
    }

    // A comment that spans lines.
    if state & IN_COMMENT != 0 {
        return match line.find("%%") {
            Some(end) => {
                spans.push(ByteSpan {
                    start: 0,
                    end: end + 2,
                    kind: kind::COMMENT,
                });
                state &= !IN_COMMENT;
                inline_spans(line, end + 2, &mut spans, &mut state);
                finish(line, spans, state)
            }
            None => finish(line, vec![whole(kind::COMMENT)], state),
        };
    }

    // Fenced code.
    let is_fence = trimmed.starts_with("```") || trimmed.starts_with("~~~");
    if state & IN_FENCE != 0 {
        if is_fence {
            state &= !IN_FENCE;
        }
        return finish(line, vec![whole(kind::CODE_BLOCK)], state);
    }
    if is_fence {
        return finish(line, vec![whole(kind::CODE_BLOCK)], state | IN_FENCE);
    }

    // Block prefixes: quotes, then a heading, list marker, rule or table on what remains.
    let mut pos = indent;
    let mut rest = trimmed;
    while let Some(after) = rest.strip_prefix('>') {
        spans.push(ByteSpan {
            start: pos,
            end: pos + 1,
            kind: kind::QUOTE,
        });
        let skipped = after.len() - after.trim_start().len();
        pos += 1 + skipped;
        rest = after.trim_start();
    }
    if let Some(after) = rest.strip_prefix("[!") {
        if let Some(close) = after.find(']') {
            spans.push(ByteSpan {
                start: pos,
                end: pos + 2 + close + 1,
                kind: kind::CALLOUT,
            });
            pos += 2 + close + 1;
            rest = &after[close + 1..];
        }
    }
    let hashes = rest.chars().take_while(|c| *c == '#').count();
    if (1..=kind::HEADING_MAX as usize).contains(&hashes) && rest[hashes..].starts_with([' ', '\t'])
    {
        spans.push(ByteSpan {
            start: pos,
            end: line.len(),
            kind: hashes as u8,
        });
        inline_spans(line, pos + hashes, &mut spans, &mut state);
        return finish(line, spans, state);
    }
    if is_rule(rest) {
        spans.push(ByteSpan {
            start: pos,
            end: line.len(),
            kind: kind::RULE,
        });
        return finish(line, spans, state);
    }
    if let Some(marker_len) = list_marker(rest) {
        spans.push(ByteSpan {
            start: pos,
            end: pos + marker_len,
            kind: kind::LIST,
        });
        let after = &rest[marker_len..];
        let skipped = after.len() - after.trim_start().len();
        let after = after.trim_start();
        let mut next = pos + marker_len + skipped;
        if after.starts_with("[ ]") || after.starts_with("[x]") || after.starts_with("[X]") {
            spans.push(ByteSpan {
                start: next,
                end: next + 3,
                kind: kind::TASK,
            });
            next += 3;
        }
        inline_spans(line, next, &mut spans, &mut state);
        return finish(line, spans, state);
    }
    if rest.starts_with('|') {
        for (i, c) in rest.char_indices() {
            if c == '|' {
                spans.push(ByteSpan {
                    start: pos + i,
                    end: pos + i + 1,
                    kind: kind::TABLE,
                });
            }
        }
    }
    inline_spans(line, pos, &mut spans, &mut state);
    finish(line, spans, state)
}

fn is_rule(rest: &str) -> bool {
    let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    compact.len() >= 3
        && (compact.chars().all(|c| c == '-')
            || compact.chars().all(|c| c == '*')
            || compact.chars().all(|c| c == '_'))
}

/// The byte length of a list marker at the start of `rest`, when there is one.
fn list_marker(rest: &str) -> Option<usize> {
    if (rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ "))
        || rest == "-"
        || rest == "*"
        || rest == "+"
    {
        return Some(1);
    }
    let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && digits <= 9 {
        let after = &rest[digits..];
        if (after.starts_with(". ") || after.starts_with(") ")) || after == "." || after == ")" {
            return Some(digits + 1);
        }
    }
    None
}

/// Inline runs from `from` to the end of the line.
fn inline_spans(line: &str, from: usize, spans: &mut Vec<ByteSpan>, state: &mut i32) {
    let bytes = line.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        let rest = &line[i..];
        // Inline code.
        if bytes[i] == b'`' {
            let ticks = rest.chars().take_while(|c| *c == '`').count();
            let fence = &rest[..ticks];
            if let Some(end) = rest[ticks..].find(fence) {
                let len = ticks + end + ticks;
                spans.push(ByteSpan {
                    start: i,
                    end: i + len,
                    kind: kind::CODE,
                });
                i += len;
                continue;
            }
            i += ticks;
            continue;
        }
        // Comments.
        if let Some(after) = rest.strip_prefix("%%") {
            match after.find("%%") {
                Some(end) => {
                    spans.push(ByteSpan {
                        start: i,
                        end: i + 2 + end + 2,
                        kind: kind::COMMENT,
                    });
                    i += 2 + end + 2;
                }
                None => {
                    spans.push(ByteSpan {
                        start: i,
                        end: line.len(),
                        kind: kind::COMMENT,
                    });
                    *state |= IN_COMMENT;
                    return;
                }
            }
            continue;
        }
        // Wikilinks and embeds.
        if let Some(after) = rest.strip_prefix("[[") {
            if let Some(end) = after.find("]]") {
                let start = if i > 0 && bytes[i - 1] == b'!' {
                    i - 1
                } else {
                    i
                };
                spans.push(ByteSpan {
                    start,
                    end: i + 2 + end + 2,
                    kind: kind::LINK,
                });
                i += 2 + end + 2;
                continue;
            }
        }
        // Markdown links and images: [text](url).
        if bytes[i] == b'[' {
            if let Some(close) = rest.find("](") {
                if let Some(end) = rest[close + 2..].find(')') {
                    let start = if i > 0 && bytes[i - 1] == b'!' {
                        i - 1
                    } else {
                        i
                    };
                    spans.push(ByteSpan {
                        start,
                        end: i + close + 2 + end + 1,
                        kind: kind::LINK,
                    });
                    i += close + 2 + end + 1;
                    continue;
                }
            }
        }
        // Bare URLs.
        if rest.starts_with("http://") || rest.starts_with("https://") {
            let len = rest
                .find(|c: char| c.is_whitespace() || c == '>' || c == ')')
                .unwrap_or(rest.len());
            spans.push(ByteSpan {
                start: i,
                end: i + len,
                kind: kind::URL,
            });
            i += len;
            continue;
        }
        // HTML tags.
        if bytes[i] == b'<' {
            if let Some(end) = rest.find('>') {
                let inner = &rest[1..end];
                if inner
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!')
                    && !inner.contains(' ')
                    || inner.starts_with("a ")
                    || inner.starts_with("img ")
                    || inner.starts_with("span ")
                    || inner.starts_with("div ")
                    || inner.starts_with("!--")
                {
                    spans.push(ByteSpan {
                        start: i,
                        end: i + end + 1,
                        kind: kind::HTML,
                    });
                    i += end + 1;
                    continue;
                }
            }
        }
        // Paired inline markers.
        let paired: &[(&str, u8)] = &[
            ("**", kind::STRONG),
            ("__", kind::STRONG),
            ("~~", kind::STRIKE),
            ("==", kind::MARK),
            ("$$", kind::MATH),
        ];
        let mut matched = false;
        for (marker, k) in paired {
            if let Some(after) = rest.strip_prefix(marker) {
                if let Some(end) = after.find(marker) {
                    if end > 0 {
                        let len = marker.len() + end + marker.len();
                        spans.push(ByteSpan {
                            start: i,
                            end: i + len,
                            kind: *k,
                        });
                        i += len;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if matched {
            continue;
        }
        // Single-character emphasis and math, with a word boundary before.
        if bytes[i] == b'*' || bytes[i] == b'_' || bytes[i] == b'$' {
            let marker = bytes[i] as char;
            let boundary_before =
                i == 0 || !line[..i].chars().last().is_some_and(char::is_alphanumeric);
            let after = &rest[1..];
            if boundary_before && after.chars().next().is_some_and(|c| !c.is_whitespace()) {
                if let Some(end) = after.find(marker) {
                    if end > 0 && !after[..end].ends_with(char::is_whitespace) {
                        let len = 1 + end + 1;
                        let k = if marker == '$' {
                            kind::MATH
                        } else {
                            kind::EMPHASIS
                        };
                        spans.push(ByteSpan {
                            start: i,
                            end: i + len,
                            kind: k,
                        });
                        i += len;
                        continue;
                    }
                }
            }
        }
        // Tags.
        if bytes[i] == b'#' {
            let boundary_before = i == 0
                || line[..i]
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_whitespace() || c == '(' || c == ',');
            let after = &rest[1..];
            let len = after
                .char_indices()
                .find(|(_, c)| !(c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '/'))
                .map(|(i, _)| i)
                .unwrap_or(after.len());
            if boundary_before && len > 0 && after[..len].chars().any(char::is_alphabetic) {
                spans.push(ByteSpan {
                    start: i,
                    end: i + 1 + len,
                    kind: kind::TAG,
                });
                i += 1 + len;
                continue;
            }
        }
        i += line[i..].chars().next().map_or(1, char::len_utf8);
    }
}

/// Convert byte spans to UTF-16 spans and wrap up.
fn finish(line: &str, spans: Vec<ByteSpan>, state: i32) -> LineSpans {
    let mut spans: Vec<ByteSpan> = spans.into_iter().filter(|s| s.end > s.start).collect();
    spans.sort_by_key(|s| (s.start, s.end));
    let spans = spans
        .into_iter()
        .map(|s| {
            let start = line[..s.start].encode_utf16().count() as u32;
            let len = line[s.start..s.end].encode_utf16().count() as u32;
            Span {
                start,
                len,
                kind: s.kind,
            }
        })
        .collect();
    LineSpans { spans, state }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_sections_split_at_headings_outside_fences() {
        let raw = "---\ntitle: T\n---\n\nPreamble line.\n\n# One\n\ntext\n```\n# not a heading\n```\n## Two\n###### Six\n#notatag\n#\nend";
        let parts = page_sections(raw);
        assert_eq!(parts[0], "---\ntitle: T\n---\n");
        assert_eq!(parts[1], "\nPreamble line.\n\n");
        assert_eq!(parts[2], "# One\n\ntext\n```\n# not a heading\n```\n");
        assert_eq!(parts[3], "## Two\n");
        assert_eq!(parts[4], "###### Six\n#notatag\n");
        assert_eq!(parts[5], "#\nend");
        assert_eq!(parts.concat(), raw, "the parts give the page back");
        assert_eq!(
            page_sections("no frontmatter\n# H\n"),
            vec!["", "no frontmatter\n", "# H\n"]
        );
        assert_eq!(page_sections("# H\nbody"), vec!["", "# H\nbody"]);
        assert_eq!(page_sections(""), vec![""]);
        assert_eq!(
            page_sections("---\nopen frontmatter"),
            vec!["", "---\nopen frontmatter"]
        );
        assert_eq!(page_sections("---\na: 1\n---"), vec!["---\na: 1\n---"]);
    }

    fn kinds(line: &str, state: i32) -> (Vec<(u32, u32, u8)>, i32) {
        let out = highlight_line(line, state);
        (
            out.spans.iter().map(|s| (s.start, s.len, s.kind)).collect(),
            out.state,
        )
    }

    #[test]
    fn frontmatter_only_at_the_top() {
        assert_eq!(
            kinds("---", -1),
            (vec![(0, 3, kind::FRONTMATTER)], IN_FRONTMATTER)
        );
        assert_eq!(
            kinds("title: X", IN_FRONTMATTER),
            (vec![(0, 8, kind::FRONTMATTER)], IN_FRONTMATTER)
        );
        assert_eq!(
            kinds("---", IN_FRONTMATTER),
            (vec![(0, 3, kind::FRONTMATTER)], 0)
        );
        assert_eq!(kinds("---", 0), (vec![(0, 3, kind::RULE)], 0));
    }

    #[test]
    fn fences_and_comments_carry_state() {
        assert_eq!(
            kinds("```rust", 0),
            (vec![(0, 7, kind::CODE_BLOCK)], IN_FENCE)
        );
        assert_eq!(
            kinds("let x = [[a]];", IN_FENCE),
            (vec![(0, 14, kind::CODE_BLOCK)], IN_FENCE)
        );
        assert_eq!(kinds("```", IN_FENCE), (vec![(0, 3, kind::CODE_BLOCK)], 0));
        assert_eq!(
            kinds("a %% open", 0),
            (vec![(2, 7, kind::COMMENT)], IN_COMMENT)
        );
        assert_eq!(
            kinds("still %% after", IN_COMMENT),
            (vec![(0, 8, kind::COMMENT)], 0)
        );
    }

    #[test]
    fn headings_lists_quotes_and_callouts() {
        assert_eq!(
            kinds("## Two *em*", 0).0,
            vec![(0, 11, 2), (7, 4, kind::EMPHASIS)]
        );
        assert_eq!(kinds("#nospace", 0).0, vec![(0, 8, kind::TAG)]);
        assert_eq!(
            kinds("- [x] done #t", 0).0,
            vec![(0, 1, kind::LIST), (2, 3, kind::TASK), (11, 2, kind::TAG)]
        );
        assert_eq!(kinds("12. item", 0).0, vec![(0, 3, kind::LIST)]);
        assert_eq!(
            kinds("> [!note] Title", 0).0,
            vec![(0, 1, kind::QUOTE), (2, 7, kind::CALLOUT)]
        );
        assert_eq!(
            kinds("> > deep", 0).0,
            vec![(0, 1, kind::QUOTE), (2, 1, kind::QUOTE)]
        );
        assert_eq!(kinds("| a | b |", 0).0.len(), 3);
    }

    #[test]
    fn inline_runs() {
        let (spans, _) = kinds(
            "see [[a/b|x]] and ![[i.png]] `co[[de]]` **b** ~~s~~ ==m== $x$ <br> https://x.y/z #tag a_b_c",
            0,
        );
        assert_eq!(
            spans,
            vec![
                (4, 9, kind::LINK),
                (18, 10, kind::LINK),
                (29, 10, kind::CODE),
                (40, 5, kind::STRONG),
                (46, 5, kind::STRIKE),
                (52, 5, kind::MARK),
                (58, 3, kind::MATH),
                (62, 4, kind::HTML),
                (67, 13, kind::URL),
                (81, 4, kind::TAG),
            ]
        );
        assert_eq!(
            kinds("[t](u) *e* _e_", 0).0,
            vec![
                (0, 6, kind::LINK),
                (7, 3, kind::EMPHASIS),
                (11, 3, kind::EMPHASIS)
            ]
        );
    }

    #[test]
    fn positions_are_utf16() {
        let (spans, _) = kinds("é😀 **b**", 0);
        assert_eq!(spans, vec![(4, 5, kind::STRONG)]);
    }
}
