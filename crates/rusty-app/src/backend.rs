//! The `Backend` QML type: the app's MCP client. It keeps one Streamable HTTP session
//! with `rusty-mcp` on a tokio runtime in the background, reconnects when the service
//! goes away, forwards tool calls from QML and hands the JSON replies back through the
//! `result` signal, and turns the server's `resources/list_changed` notifications into
//! the `dataChanged` signal so every tab can refresh on real changes.
//!
//! The app holds no store of its own; everything it shows came through here.

use core::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientInfo};
use rmcp::service::{NotificationContext, Peer};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::{ClientHandler, RoleClient, ServiceExt};

/// Where the app looks for the back end unless `RUSTY_MCP_URL` says otherwise.
pub const DEFAULT_URL: &str = "http://127.0.0.1:4174/mcp";
const RETRY_EVERY: Duration = Duration::from_secs(3);

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
        #[qproperty(bool, connected)]
        #[qproperty(QString, status)]
        #[qproperty(QString, url)]
        type Backend = super::BackendRust;

        /// Connect to `rusty-mcp` and keep reconnecting. Call once.
        #[qinvokable]
        fn start(self: Pin<&mut Backend>);

        /// Call a tool with JSON arguments; the answer arrives through `result` with the
        /// returned id.
        #[qinvokable]
        fn call(self: Pin<&mut Backend>, tool: &QString, args_json: &QString) -> i32;

        /// A tool answered: the JSON text it returned (or the error text when `ok` is false).
        #[qsignal]
        fn result(self: Pin<&mut Backend>, id: i32, tool: QString, json: QString, ok: bool);

        /// The server said its data changed (a tool mutated something, or a file moved on
        /// disk). Tabs refresh what they show.
        #[qsignal]
        fn data_changed(self: Pin<&mut Backend>);
    }

    impl cxx_qt::Threading for Backend {}
}

type PeerSlot = Arc<Mutex<Option<Peer<RoleClient>>>>;

/// The Rust side of [`qobject::Backend`].
pub struct BackendRust {
    connected: bool,
    status: QString,
    url: QString,
    next_id: i32,
    runtime: Option<tokio::runtime::Runtime>,
    peer: PeerSlot,
}

impl Default for BackendRust {
    fn default() -> Self {
        let url = std::env::var("RUSTY_MCP_URL")
            .ok()
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        Self {
            connected: false,
            status: QString::from("not connected"),
            url: QString::from(&url),
            next_id: 1,
            runtime: None,
            peer: Arc::new(Mutex::new(None)),
        }
    }
}

/// The client side of the MCP session: forwards change notifications to the Qt thread.
struct Handler {
    qt: cxx_qt::CxxQtThread<qobject::Backend>,
}

impl ClientHandler for Handler {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }

    fn on_resource_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let qt = self.qt.clone();
        async move {
            let _ = qt.queue(|mut backend| backend.as_mut().data_changed());
        }
    }
}

/// The text content of a tool result, concatenated.
pub fn text_of(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect()
}

/// Parse the JSON arguments QML sends; anything that is not an object means no arguments.
pub fn arguments_from(json: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .filter(|m| !m.is_empty())
}

impl qobject::Backend {
    /// Start the runtime and the connection loop.
    pub fn start(mut self: Pin<&mut Self>) {
        if self.rust().runtime.is_some() {
            return;
        }
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                self.as_mut()
                    .set_status(QString::from(&format!("no async runtime: {e}")));
                return;
            }
        };
        let qt = self.qt_thread();
        let slot = Arc::clone(&self.rust().peer);
        let url = self.rust().url.to_string();
        runtime.spawn(async move {
            loop {
                let transport = StreamableHttpClientTransport::from_uri(url.as_str());
                let handler = Handler { qt: qt.clone() };
                match handler.serve(transport).await {
                    Ok(service) => {
                        if let Ok(mut peer) = slot.lock() {
                            *peer = Some(service.peer().clone());
                        }
                        let _ = qt.queue(|mut backend| {
                            backend.as_mut().set_connected(true);
                            backend.as_mut().set_status(QString::from("connected"));
                            backend.as_mut().data_changed();
                        });
                        let _ = service.waiting().await;
                        if let Ok(mut peer) = slot.lock() {
                            *peer = None;
                        }
                        let _ = qt.queue(|mut backend| {
                            backend.as_mut().set_connected(false);
                            backend
                                .as_mut()
                                .set_status(QString::from("connection lost; retrying"));
                        });
                    }
                    Err(e) => {
                        let message = format!("rusty-mcp not reachable at {url}: {e}");
                        let _ = qt.queue(move |mut backend| {
                            backend.as_mut().set_connected(false);
                            backend.as_mut().set_status(QString::from(&message));
                        });
                    }
                }
                tokio::time::sleep(RETRY_EVERY).await;
            }
        });
        self.as_mut().rust_mut().runtime = Some(runtime);
    }

    /// Queue a tool call; the reply comes back through `result`.
    pub fn call(mut self: Pin<&mut Self>, tool: &QString, args_json: &QString) -> i32 {
        let id = self.rust().next_id;
        self.as_mut().rust_mut().next_id = id.wrapping_add(1).max(1);
        let tool_name = tool.to_string();
        let peer = self.rust().peer.lock().ok().and_then(|p| p.clone());
        let handle = self.rust().runtime.as_ref().map(|rt| rt.handle().clone());
        let (Some(peer), Some(handle)) = (peer, handle) else {
            self.as_mut().result(
                id,
                tool.clone(),
                QString::from("\"rusty-mcp is not connected\""),
                false,
            );
            return id;
        };
        let arguments = arguments_from(&args_json.to_string());
        let qt = self.qt_thread();
        handle.spawn(async move {
            let mut params = CallToolRequestParams::new(tool_name.clone());
            if let Some(arguments) = arguments {
                params = params.with_arguments(arguments);
            }
            let (json, ok) = match peer.call_tool(params).await {
                Ok(result) => {
                    let failed = result.is_error.unwrap_or(false);
                    (text_of(&result), !failed)
                }
                Err(e) => (format!("{e}"), false),
            };
            let _ = qt.queue(move |mut backend| {
                backend
                    .as_mut()
                    .result(id, QString::from(&tool_name), QString::from(&json), ok);
            });
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_parse_only_objects() {
        assert!(arguments_from("{}").is_none());
        assert!(arguments_from("[1]").is_none());
        assert!(arguments_from("nonsense").is_none());
        let args = arguments_from(r#"{"group_id": 5, "title": "x"}"#).unwrap();
        assert_eq!(args["group_id"], 5);
        assert_eq!(args["title"], "x");
    }
}
