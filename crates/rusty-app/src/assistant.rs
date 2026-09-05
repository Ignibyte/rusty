//! The `Assistant` QML type (TICKET-025): the agent beside a note as a headless
//! `claude -p` over stream-json, long-lived and bidirectional. The process is spawned
//! with the page's session resumed by id when there is one; its stdout is read line by
//! line on a thread and parsed here into the events the pane renders; the pane's text
//! goes in as `user` messages, and a permission prompt comes out as a `control_request`
//! the pane answers with a `control_response`. Nothing here talks to the Claude API: it
//! is Claude Code's own harness — skills, tools, MCP, the box's auth — without its
//! terminal. The terminal tabs stay terminals.

use core::pin::Pin;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt's string type.
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, available)]
        #[qproperty(bool, running)]
        #[qproperty(bool, busy)]
        #[qproperty(QString, status)]
        #[qproperty(QString, session_id)]
        type Assistant = super::AssistantRust;

        /// Start (or restart) the process for a page: `resume` is a session id to
        /// continue or empty for a fresh one, `system_prompt` is appended to Claude
        /// Code's own, `mcp_url` is Rusty's server.
        #[qinvokable]
        fn start(
            self: Pin<&mut Assistant>,
            cwd: &QString,
            resume: &QString,
            system_prompt: &QString,
            mcp_url: &QString,
        );
        /// Send the user's text as one message; false when nothing is running.
        #[qinvokable]
        fn send(self: Pin<&mut Assistant>, text: &QString) -> bool;
        /// Answer a permission request: allow with the input as given, or deny.
        #[qinvokable]
        fn answer(
            self: Pin<&mut Assistant>,
            request_id: &QString,
            allow: bool,
            input_json: &QString,
        ) -> bool;
        /// Ask the running turn to stop; the process stays.
        #[qinvokable]
        fn interrupt(self: Pin<&mut Assistant>) -> bool;
        /// End the process.
        #[qinvokable]
        fn stop(self: Pin<&mut Assistant>);

        /// The process announced itself with its session id.
        #[qsignal]
        fn started(self: Pin<&mut Assistant>, session_id: QString);
        /// A content block began: `text`, `tool_use` (with its name and id) or `thinking`.
        #[qsignal]
        fn block_started(self: Pin<&mut Assistant>, kind: QString, name: QString, id: QString);
        /// A piece of the assistant's text.
        #[qsignal]
        fn text_delta(self: Pin<&mut Assistant>, text: QString);
        /// The whole text of an assistant message, once it is complete.
        #[qsignal]
        fn text_final(self: Pin<&mut Assistant>, text: QString);
        /// A tool call's input, once the message that carries it is complete.
        #[qsignal]
        fn tool_input(self: Pin<&mut Assistant>, id: QString, name: QString, input_json: QString);
        /// What a tool answered.
        #[qsignal]
        fn tool_result(self: Pin<&mut Assistant>, id: QString, text: QString, is_error: bool);
        /// The agent wants to use a tool that needs a decision.
        #[qsignal]
        fn permission_asked(
            self: Pin<&mut Assistant>,
            request_id: QString,
            tool: QString,
            input_json: QString,
            description: QString,
        );
        /// A turn ended, well or not.
        #[qsignal]
        fn turn_done(
            self: Pin<&mut Assistant>,
            ok: bool,
            cost_usd: f64,
            num_turns: i32,
            text: QString,
        );
        /// Something worth a line in the conversation (a denied permission, for one).
        #[qsignal]
        fn notice(self: Pin<&mut Assistant>, text: QString);
        /// The process ended; `message` carries the tail of its stderr.
        #[qsignal]
        fn exited(self: Pin<&mut Assistant>, code: i32, message: QString);
    }

    impl cxx_qt::Threading for Assistant {}
}

/// Rusty's tools the pane's agent may call without asking: the reads. A write prompts.
pub const READ_TOOLS: &[&str] = &[
    "brain_read_page",
    "brain_search",
    "brain_list_pages",
    "brain_get_links",
    "brain_tags",
    "brain_tree",
    "brain_render",
    "brain_stats",
    "brain_due",
    "brain_get_timeline",
    "brain_page_types",
    "brain_resolve_slug",
    "brain_unresolved",
    "brain_graph",
    "brain_semantic_status",
    "brain_ask",
    "list_tasks",
    "list_task_groups",
    "list_notes",
    "read_note",
    "list_memories",
    "search_conversations",
    "skill_list",
    "skill_view",
    "script_list",
    "script_view",
    "settings_list",
    "setting_get",
];

/// The most of stderr kept for the exit message.
const STDERR_TAIL: usize = 2000;

/// What the process said, one line of stream-json at a time.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Init {
        session_id: String,
    },
    BlockStart {
        kind: String,
        name: String,
        id: String,
    },
    TextDelta(String),
    TextFinal(String),
    ToolInput {
        id: String,
        name: String,
        input: String,
    },
    ToolResult {
        id: String,
        text: String,
        is_error: bool,
    },
    Permission {
        request_id: String,
        tool: String,
        input: String,
        description: String,
    },
    TurnDone {
        ok: bool,
        cost_usd: f64,
        num_turns: i64,
        text: String,
    },
    Notice(String),
}

/// What the reader hands back: a line of stdout, or the exit once stdout closes, with
/// the tail of stderr.
#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Line(String),
    Exit(i32, String),
}

fn str_of(value: &serde_json::Value, key: &str) -> String {
    value[key].as_str().unwrap_or("").to_string()
}

/// A tool result's content: a string, or text blocks joined.
fn content_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The events in one line of stream-json; a line that says nothing to the pane gives
/// none, and a line that is not JSON gives none too.
pub fn parse_line(line: &str) -> Vec<Event> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    match v["type"].as_str().unwrap_or("") {
        "system" => match v["subtype"].as_str().unwrap_or("") {
            "init" => out.push(Event::Init {
                session_id: str_of(&v, "session_id"),
            }),
            "permission_denied" => {
                let tool = str_of(&v, "tool_name");
                out.push(Event::Notice(if tool.is_empty() {
                    "A permission was denied.".to_string()
                } else {
                    format!("A permission for {tool} was denied.")
                }));
            }
            _ => {}
        },
        "stream_event" => {
            let event = &v["event"];
            match event["type"].as_str().unwrap_or("") {
                "content_block_start" => {
                    let block = &event["content_block"];
                    let kind = str_of(block, "type");
                    if kind == "text" || kind == "tool_use" || kind == "thinking" {
                        out.push(Event::BlockStart {
                            kind,
                            name: str_of(block, "name"),
                            id: str_of(block, "id"),
                        });
                    }
                }
                "content_block_delta" if event["delta"]["type"].as_str() == Some("text_delta") => {
                    out.push(Event::TextDelta(str_of(&event["delta"], "text")));
                }
                _ => {}
            }
        }
        "assistant" => {
            if let Some(blocks) = v["message"]["content"].as_array() {
                for block in blocks {
                    match block["type"].as_str().unwrap_or("") {
                        "tool_use" => out.push(Event::ToolInput {
                            id: str_of(block, "id"),
                            name: str_of(block, "name"),
                            input: block["input"].to_string(),
                        }),
                        "text" => out.push(Event::TextFinal(str_of(block, "text"))),
                        _ => {}
                    }
                }
            }
        }
        "user" => {
            if let Some(blocks) = v["message"]["content"].as_array() {
                for block in blocks {
                    if block["type"].as_str() == Some("tool_result") {
                        out.push(Event::ToolResult {
                            id: str_of(block, "tool_use_id"),
                            text: content_text(&block["content"]),
                            is_error: block["is_error"].as_bool().unwrap_or(false),
                        });
                    }
                }
            }
        }
        "control_request" => {
            let request = &v["request"];
            if request["subtype"].as_str() == Some("can_use_tool") {
                out.push(Event::Permission {
                    request_id: str_of(&v, "request_id"),
                    tool: str_of(request, "tool_name"),
                    input: request["input"].to_string(),
                    description: str_of(request, "description"),
                });
            }
        }
        "result" => {
            let is_error = v["is_error"].as_bool().unwrap_or(false);
            let subtype = v["subtype"].as_str().unwrap_or("");
            let ok = !is_error && subtype == "success";
            let text = if ok {
                str_of(&v, "result")
            } else {
                let given = v["result"]
                    .as_str()
                    .or_else(|| v["error"].as_str())
                    .unwrap_or("");
                if given.is_empty() {
                    format!("The turn ended with {subtype}.")
                } else {
                    given.to_string()
                }
            };
            out.push(Event::TurnDone {
                ok,
                cost_usd: v["total_cost_usd"].as_f64().unwrap_or(0.0),
                num_turns: v["num_turns"].as_i64().unwrap_or(0),
                text,
            });
        }
        _ => {}
    }
    out
}

/// One user message, as a line.
pub fn user_message(text: &str) -> String {
    serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": text }] }
    })
    .to_string()
}

/// The answer to a `can_use_tool` request: allow with the input as given, or deny.
pub fn control_response(request_id: &str, allow: bool, input_json: &str) -> String {
    let response = if allow {
        let input = serde_json::from_str::<serde_json::Value>(input_json)
            .unwrap_or_else(|_| serde_json::json!({}));
        serde_json::json!({ "behavior": "allow", "updatedInput": input })
    } else {
        serde_json::json!({ "behavior": "deny", "message": "Declined in Rusty." })
    };
    serde_json::json!({
        "type": "control_response",
        "response": { "subtype": "success", "request_id": request_id, "response": response }
    })
    .to_string()
}

/// A request to stop the running turn.
pub fn interrupt_request(n: u64) -> String {
    serde_json::json!({
        "type": "control_request",
        "request_id": format!("rusty-interrupt-{n}"),
        "request": { "subtype": "interrupt" }
    })
    .to_string()
}

/// The MCP configuration handed to the process: Rusty's server over HTTP.
pub fn mcp_config(mcp_url: &str) -> String {
    serde_json::json!({ "mcpServers": { "rusty": { "type": "http", "url": mcp_url } } }).to_string()
}

/// The arguments of the process: print mode over stream-json both ways with partial
/// messages, permissions asked on stdout, Rusty's server alone, its reads pre-allowed,
/// the page in the system prompt, and the session resumed when there is one.
pub fn build_args(resume: Option<&str>, system_prompt: &str, mcp_url: &str) -> Vec<String> {
    let allowed: Vec<String> = READ_TOOLS
        .iter()
        .map(|t| format!("mcp__rusty__{t}"))
        .collect();
    let mut args: Vec<String> = [
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--include-partial-messages",
        "--verbose",
        "--permission-prompt-tool",
        "stdio",
        "--permission-mode",
        "default",
        "--strict-mcp-config",
        "--mcp-config",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.push(mcp_config(mcp_url));
    args.push("--allowedTools".to_string());
    args.push(allowed.join(","));
    if !system_prompt.is_empty() {
        args.push("--append-system-prompt".to_string());
        args.push(system_prompt.to_string());
    }
    if let Some(id) = resume.filter(|id| !id.trim().is_empty()) {
        args.push("--resume".to_string());
        args.push(id.trim().to_string());
    }
    args
}

/// Where `claude` is: `RUSTY_CLAUDE_BIN` when set, else the first on `PATH`, else what
/// a login shell knows (the app runs as a user service whose `PATH` may not carry the
/// shims a shell adds).
pub fn claude_binary() -> Option<PathBuf> {
    if let Some(given) = std::env::var_os("RUSTY_CLAUDE_BIN").filter(|p| !p.is_empty()) {
        let given = PathBuf::from(given);
        return given.is_file().then_some(given);
    }
    if let Some(found) = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join("claude"))
            .find(|p| p.is_file())
    }) {
        return Some(found);
    }
    let out = Command::new("bash")
        .args(["-lc", "command -v claude"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !path.is_empty()).then(|| PathBuf::from(path))
}

/// A running process: the child behind a lock shared with the reader, and its stdin.
pub struct Process {
    child: Arc<Mutex<Child>>,
    stdin: Mutex<Option<ChildStdin>>,
}

impl Process {
    /// Write one line to the process; false when stdin is gone.
    pub fn write_line(&self, line: &str) -> bool {
        let Ok(mut guard) = self.stdin.lock() else {
            return false;
        };
        let Some(stdin) = guard.as_mut() else {
            return false;
        };
        let ok = stdin
            .write_all(line.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush())
            .is_ok();
        if !ok {
            *guard = None;
        }
        ok
    }

    /// End the process and wait for it.
    pub fn kill(&self) {
        if let Ok(mut guard) = self.stdin.lock() {
            *guard = None;
        }
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Spawn `binary` with `args` in `cwd`, handing every stdout line and then the exit to
/// `on_output` from a reader thread.
pub fn spawn(
    binary: &Path,
    args: &[String],
    cwd: &Path,
    on_output: impl FnMut(Output) + Send + 'static,
) -> Result<Process, String> {
    let mut child = Command::new(binary)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", binary.display()))?;
    let stdout = child.stdout.take().ok_or("the process has no stdout")?;
    let stderr = child.stderr.take().ok_or("the process has no stderr")?;
    let stdin = child.stdin.take();
    let child = Arc::new(Mutex::new(child));
    let tail: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let tail_writer = Arc::clone(&tail);
    std::thread::Builder::new()
        .name("assistant-stderr".to_string())
        .spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if let Ok(mut t) = tail_writer.lock() {
                    t.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if t.len() > STDERR_TAIL {
                        let cut = t.len() - STDERR_TAIL;
                        let cut = t.floor_char_boundary(cut);
                        t.drain(..cut);
                    }
                }
            }
        })
        .map_err(|e| format!("stderr thread: {e}"))?;
    let waiter = Arc::clone(&child);
    std::thread::Builder::new()
        .name("assistant-reader".to_string())
        .spawn(move || {
            let mut on_output = on_output;
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(l) => on_output(Output::Line(l)),
                    Err(_) => break,
                }
            }
            let code = waiter
                .lock()
                .ok()
                .and_then(|mut c| c.wait().ok())
                .and_then(|s| s.code())
                .unwrap_or(-1);
            let message = tail
                .lock()
                .map(|t| t.trim().to_string())
                .unwrap_or_default();
            on_output(Output::Exit(code, message));
        })
        .map_err(|e| format!("reader thread: {e}"))?;
    Ok(Process {
        child,
        stdin: Mutex::new(stdin),
    })
}

/// The Rust side of [`qobject::Assistant`].
pub struct AssistantRust {
    available: bool,
    running: bool,
    busy: bool,
    status: QString,
    session_id: QString,
    process: Option<Process>,
    /// Lines from a process that was replaced are dropped by this.
    generation: u64,
    interrupts: u64,
}

impl Default for AssistantRust {
    fn default() -> Self {
        let available = claude_binary().is_some();
        Self {
            available,
            running: false,
            busy: false,
            status: QString::from(if available {
                "not started"
            } else {
                "Claude Code is not installed"
            }),
            session_id: QString::default(),
            process: None,
            generation: 0,
            interrupts: 0,
        }
    }
}

impl qobject::Assistant {
    /// See the bridge.
    pub fn start(
        mut self: Pin<&mut Self>,
        cwd: &QString,
        resume: &QString,
        system_prompt: &QString,
        mcp_url: &QString,
    ) {
        self.as_mut().stop();
        let Some(binary) = claude_binary() else {
            self.as_mut().set_available(false);
            self.as_mut()
                .set_status(QString::from("Claude Code is not installed"));
            return;
        };
        let generation = self.rust().generation + 1;
        self.as_mut().rust_mut().generation = generation;
        let resume = resume.to_string();
        let args = build_args(
            Some(resume.as_str()),
            &system_prompt.to_string(),
            &mcp_url.to_string(),
        );
        let qt = self.qt_thread();
        let spawned = spawn(&binary, &args, Path::new(&cwd.to_string()), move |out| {
            let _ = qt.queue(move |mut assistant| {
                if assistant.rust().generation == generation {
                    assistant.as_mut().handle(out);
                }
            });
        });
        match spawned {
            Ok(process) => {
                self.as_mut().rust_mut().process = Some(process);
                self.as_mut().set_running(true);
                self.as_mut().set_busy(false);
                self.as_mut().set_session_id(QString::from(&resume));
                self.as_mut().set_status(QString::from("starting"));
            }
            Err(e) => {
                self.as_mut().set_running(false);
                self.as_mut().set_status(QString::from(&e));
                self.as_mut().exited(-1, QString::from(&e));
            }
        }
    }

    /// See the bridge.
    pub fn send(mut self: Pin<&mut Self>, text: &QString) -> bool {
        let line = user_message(&text.to_string());
        let Some(process) = self.rust().process.as_ref() else {
            return false;
        };
        if !process.write_line(&line) {
            return false;
        }
        self.as_mut().set_busy(true);
        self.as_mut().set_status(QString::from("working"));
        true
    }

    /// See the bridge.
    pub fn answer(
        mut self: Pin<&mut Self>,
        request_id: &QString,
        allow: bool,
        input_json: &QString,
    ) -> bool {
        let line = control_response(&request_id.to_string(), allow, &input_json.to_string());
        let Some(process) = self.rust().process.as_ref() else {
            return false;
        };
        let ok = process.write_line(&line);
        if ok {
            self.as_mut().set_status(QString::from("working"));
        }
        ok
    }

    /// See the bridge.
    pub fn interrupt(mut self: Pin<&mut Self>) -> bool {
        let n = self.rust().interrupts + 1;
        self.as_mut().rust_mut().interrupts = n;
        let line = interrupt_request(n);
        self.rust()
            .process
            .as_ref()
            .is_some_and(|p| p.write_line(&line))
    }

    /// See the bridge.
    pub fn stop(mut self: Pin<&mut Self>) {
        let generation = self.rust().generation + 1;
        self.as_mut().rust_mut().generation = generation;
        if let Some(process) = self.as_mut().rust_mut().process.take() {
            process.kill();
        }
        self.as_mut().set_running(false);
        self.as_mut().set_busy(false);
        self.as_mut().set_status(QString::from("stopped"));
    }

    /// A line or the exit, on the Qt thread.
    fn handle(mut self: Pin<&mut Self>, out: Output) {
        match out {
            Output::Line(line) => {
                for event in parse_line(&line) {
                    self.as_mut().emit(event);
                }
            }
            Output::Exit(code, message) => {
                self.as_mut().rust_mut().process = None;
                self.as_mut().set_running(false);
                self.as_mut().set_busy(false);
                self.as_mut().set_status(QString::from("stopped"));
                self.as_mut().exited(code, QString::from(&message));
            }
        }
    }

    fn emit(mut self: Pin<&mut Self>, event: Event) {
        match event {
            Event::Init { session_id } => {
                self.as_mut().set_session_id(QString::from(&session_id));
                self.as_mut().set_status(QString::from("ready"));
                self.as_mut().started(QString::from(&session_id));
            }
            Event::BlockStart { kind, name, id } => {
                if kind == "thinking" {
                    self.as_mut().set_status(QString::from("thinking"));
                }
                self.as_mut().block_started(
                    QString::from(&kind),
                    QString::from(&name),
                    QString::from(&id),
                );
            }
            Event::TextDelta(text) => self.as_mut().text_delta(QString::from(&text)),
            Event::TextFinal(text) => self.as_mut().text_final(QString::from(&text)),
            Event::ToolInput { id, name, input } => {
                self.as_mut()
                    .set_status(QString::from(&format!("running {name}")));
                self.as_mut().tool_input(
                    QString::from(&id),
                    QString::from(&name),
                    QString::from(&input),
                );
            }
            Event::ToolResult { id, text, is_error } => {
                self.as_mut()
                    .tool_result(QString::from(&id), QString::from(&text), is_error);
            }
            Event::Permission {
                request_id,
                tool,
                input,
                description,
            } => {
                self.as_mut()
                    .set_status(QString::from(&format!("asking to use {tool}")));
                self.as_mut().permission_asked(
                    QString::from(&request_id),
                    QString::from(&tool),
                    QString::from(&input),
                    QString::from(&description),
                );
            }
            Event::TurnDone {
                ok,
                cost_usd,
                num_turns,
                text,
            } => {
                self.as_mut().set_busy(false);
                self.as_mut().set_status(QString::from(if ok {
                    "ready"
                } else {
                    "the turn failed"
                }));
                self.as_mut()
                    .turn_done(ok, cost_usd, num_turns as i32, QString::from(&text));
            }
            Event::Notice(text) => self.as_mut().notice(QString::from(&text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_reads_the_probe() {
        // Lines as `claude` 2.1.260 wrote them on this box, trimmed to what matters.
        assert_eq!(
            parse_line(
                r#"{"type":"system","subtype":"init","cwd":"/x","session_id":"62faa927","tools":["Bash"]}"#
            ),
            vec![Event::Init {
                session_id: "62faa927".into()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"stream_event","event":{"type":"content_block_start","index":1,"content_block":{"type":"text","text":""}},"session_id":"s"}"#
            ),
            vec![Event::BlockStart {
                kind: "text".into(),
                name: String::new(),
                id: String::new()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"Write","input":{}}}}"#
            ),
            vec![Event::BlockStart {
                kind: "tool_use".into(),
                name: "Write".into(),
                id: "toolu_1".into()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"stream_event","event":{"type":"content_block_delta","index":1,"delta":{"type":"text_delta","text":" from the probe"}}}"#
            ),
            vec![Event::TextDelta(" from the probe".into())]
        );
        assert!(parse_line(r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":""}}}"#).is_empty());
        assert_eq!(
            parse_line(
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":""},{"type":"tool_use","id":"toolu_1","name":"Write","input":{"file_path":"probe.txt","content":"ok"}}]}}"#
            ),
            vec![Event::ToolInput {
                id: "toolu_1".into(),
                name: "Write".into(),
                input: r#"{"content":"ok","file_path":"probe.txt"}"#.into()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hello from the probe"}]}}"#
            ),
            vec![Event::TextFinal("hello from the probe".into())]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"probe-ok","is_error":false}]}}"#
            ),
            vec![Event::ToolResult {
                id: "toolu_1".into(),
                text: "probe-ok".into(),
                is_error: false
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t","content":[{"type":"text","text":"a"},{"type":"text","text":"b"}],"is_error":true}]}}"#
            ),
            vec![Event::ToolResult {
                id: "t".into(),
                text: "a\nb".into(),
                is_error: true
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"control_request","request_id":"8366049a","request":{"subtype":"can_use_tool","tool_name":"Write","display_name":"Write","input":{"file_path":"probe.txt","content":"ok"},"description":"probe.txt","tool_use_id":"toolu_1"}}"#
            ),
            vec![Event::Permission {
                request_id: "8366049a".into(),
                tool: "Write".into(),
                input: r#"{"content":"ok","file_path":"probe.txt"}"#.into(),
                description: "probe.txt".into()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"result","subtype":"success","is_error":false,"num_turns":2,"total_cost_usd":0.026,"result":"Done.","session_id":"s"}"#
            ),
            vec![Event::TurnDone {
                ok: true,
                cost_usd: 0.026,
                num_turns: 2,
                text: "Done.".into()
            }]
        );
        assert_eq!(
            parse_line(
                r#"{"type":"result","subtype":"error_max_turns","is_error":true,"num_turns":25}"#
            ),
            vec![Event::TurnDone {
                ok: false,
                cost_usd: 0.0,
                num_turns: 25,
                text: "The turn ended with error_max_turns.".into()
            }]
        );
        assert_eq!(
            parse_line(r#"{"type":"system","subtype":"permission_denied","tool_name":"Write"}"#),
            vec![Event::Notice("A permission for Write was denied.".into())]
        );
        assert!(parse_line(r#"{"type":"rate_limit_event","rate_limit_info":{}}"#).is_empty());
        assert!(parse_line("not json").is_empty());
    }

    #[test]
    fn messages_out_match_the_probe() {
        let user: serde_json::Value = serde_json::from_str(&user_message("hi there")).unwrap();
        assert_eq!(user["type"], "user");
        assert_eq!(user["message"]["role"], "user");
        assert_eq!(user["message"]["content"][0]["text"], "hi there");
        let allow: serde_json::Value =
            serde_json::from_str(&control_response("r-1", true, r#"{"file_path":"a"}"#)).unwrap();
        assert_eq!(allow["type"], "control_response");
        assert_eq!(allow["response"]["subtype"], "success");
        assert_eq!(allow["response"]["request_id"], "r-1");
        assert_eq!(allow["response"]["response"]["behavior"], "allow");
        assert_eq!(
            allow["response"]["response"]["updatedInput"]["file_path"],
            "a"
        );
        let deny: serde_json::Value =
            serde_json::from_str(&control_response("r-2", false, "nonsense")).unwrap();
        assert_eq!(deny["response"]["response"]["behavior"], "deny");
        assert!(deny["response"]["response"]["message"].as_str().is_some());
        let stop: serde_json::Value = serde_json::from_str(&interrupt_request(4)).unwrap();
        assert_eq!(stop["type"], "control_request");
        assert_eq!(stop["request"]["subtype"], "interrupt");
        assert_eq!(stop["request_id"], "rusty-interrupt-4");
    }

    #[test]
    fn build_args_carry_the_wire() {
        let args = build_args(None, "The page is x.", "http://127.0.0.1:4174/mcp");
        let joined = args.join(" ");
        for flag in [
            "-p",
            "--input-format stream-json",
            "--output-format stream-json",
            "--include-partial-messages",
            "--permission-prompt-tool stdio",
            "--permission-mode default",
            "--strict-mcp-config",
            "--append-system-prompt The page is x.",
        ] {
            assert!(joined.contains(flag), "{flag} in {joined}");
        }
        assert!(!joined.contains("--resume"));
        let config = args
            .iter()
            .position(|a| a == "--mcp-config")
            .map(|i| args[i + 1].clone())
            .unwrap();
        let config: serde_json::Value = serde_json::from_str(&config).unwrap();
        assert_eq!(config["mcpServers"]["rusty"]["type"], "http");
        assert_eq!(
            config["mcpServers"]["rusty"]["url"],
            "http://127.0.0.1:4174/mcp"
        );
        let allowed = args
            .iter()
            .position(|a| a == "--allowedTools")
            .map(|i| args[i + 1].clone())
            .unwrap();
        assert!(allowed.contains("mcp__rusty__brain_read_page"));
        assert!(!allowed.contains("secret_reveal") && !allowed.contains("brain_delete"));
        let resumed = build_args(Some(" abc-123 "), "", "http://x");
        let joined = resumed.join(" ");
        assert!(joined.ends_with("--resume abc-123"), "{joined}");
        assert!(!joined.contains("--append-system-prompt"));
        assert!(!build_args(Some("   "), "", "http://x")
            .join(" ")
            .contains("--resume"));
    }

    fn fake(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("claude-fake");
        std::fs::write(&path, format!("#!/usr/bin/env bash\n{body}")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn spawn_reports_lines_then_the_exit() {
        let dir = std::env::temp_dir().join(format!("rusty_assistant_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let script = fake(
            &dir,
            "echo '{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"s-1\"}'\nread -r line\ncase \"$line\" in *hello*) echo '{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"num_turns\":1,\"total_cost_usd\":0.01,\"result\":\"hi\"}';; esac\necho oops >&2\nexit 3\n",
        );
        let (tx, rx) = std::sync::mpsc::channel();
        let process = spawn(&script, &[], &dir, move |out| {
            let _ = tx.send(out);
        })
        .unwrap();
        assert!(process.write_line(&user_message("hello")));
        let mut got = Vec::new();
        for _ in 0..3 {
            got.push(
                rx.recv_timeout(std::time::Duration::from_secs(10))
                    .expect("an output"),
            );
        }
        let events: Vec<Vec<Event>> = got
            .iter()
            .filter_map(|o| match o {
                Output::Line(l) => Some(parse_line(l)),
                Output::Exit(..) => None,
            })
            .collect();
        assert_eq!(
            events[0],
            vec![Event::Init {
                session_id: "s-1".into()
            }]
        );
        assert!(matches!(events[1][0], Event::TurnDone { ok: true, .. }));
        assert_eq!(got[2], Output::Exit(3, "oops".into()));
        assert!(!process.write_line("late"), "stdin is gone after the exit");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn spawn_refuses_a_missing_binary() {
        let err = spawn(
            Path::new("/nonexistent/claude"),
            &[],
            Path::new("/"),
            |_| {},
        )
        .err()
        .unwrap();
        assert!(err.contains("/nonexistent/claude"), "{err}");
    }

    #[test]
    fn kill_ends_a_waiting_process() {
        let dir = std::env::temp_dir().join(format!("rusty_assistant_kill_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let script = fake(&dir, "while read -r line; do :; done\n");
        let (tx, rx) = std::sync::mpsc::channel();
        let process = spawn(&script, &[], &dir, move |out| {
            let _ = tx.send(out);
        })
        .unwrap();
        process.kill();
        let out = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the exit");
        assert!(matches!(out, Output::Exit(..)), "{out:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
