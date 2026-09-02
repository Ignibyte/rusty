//! Transport-agnostic application event bus.
//!
//! The backend emits [`AppEvent`]s into a broadcast channel and every connected
//! frontend subscribes:
//!
//! - The Tauri shell runs a bridge that re-emits each event via
//!   `AppHandle::emit`, preserving the exact event names and payloads the
//!   webview has always listened for.
//! - The localhost web server forwards each event over a WebSocket as JSON.
//!
//! This lets the manager layer drive either front door without knowing which is
//! attached — emitters just call [`EventBus::emit`].

use crate::engine::agent_manager::AgentCompletedEvent;
use crate::engine::process_manager::TaskCompletedEvent;
use serde::Serialize;
use tokio::sync::broadcast;

/// An event broadcast to every connected frontend.
///
/// Serializes to `{ "event": "<name>", "payload": <data> }` for the WebSocket
/// transport. The Tauri bridge instead uses [`AppEvent::name`] and
/// [`AppEvent::payload`] to emit the bare payload under the event name, matching
/// the original Tauri event contract.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "payload", rename_all = "kebab-case")]
pub enum AppEvent {
    /// A queued Claude task finished (success or failure).
    TaskCompleted(TaskCompletedEvent),
    /// A dispatched background agent finished.
    AgentCompleted(AgentCompletedEvent),
    /// The AI requested a view switch; the payload is the view name.
    NavigateView(String),
    /// On-disk data (notes/brain) or the DB change-sentinel was modified.
    DataChanged,
    /// An interactive PTY session's rendered screen changed.
    PtyOutput {
        /// Bridge session name.
        session: String,
        /// The full rendered screen after the change.
        screen: String,
    },
    /// An interactive PTY session's child process exited.
    PtyExited {
        /// Bridge session name.
        session: String,
    },
}

impl AppEvent {
    /// The frontend event name, matching the original Tauri event identifiers.
    pub fn name(&self) -> &'static str {
        match self {
            AppEvent::TaskCompleted(_) => "task-completed",
            AppEvent::AgentCompleted(_) => "agent-completed",
            AppEvent::NavigateView(_) => "navigate-view",
            AppEvent::DataChanged => "data-changed",
            AppEvent::PtyOutput { .. } => "pty-output",
            AppEvent::PtyExited { .. } => "pty-exited",
        }
    }

    /// The bare JSON payload delivered with this event — what the webview's
    /// existing `listen(name, e => e.payload)` handlers expect.
    pub fn payload(&self) -> serde_json::Value {
        match self {
            AppEvent::TaskCompleted(e) => {
                serde_json::to_value(e).unwrap_or(serde_json::Value::Null)
            }
            AppEvent::AgentCompleted(e) => {
                serde_json::to_value(e).unwrap_or(serde_json::Value::Null)
            }
            AppEvent::NavigateView(view) => serde_json::Value::String(view.clone()),
            AppEvent::DataChanged => serde_json::Value::Null,
            AppEvent::PtyOutput { session, screen } => {
                serde_json::json!({ "session": session, "screen": screen })
            }
            AppEvent::PtyExited { session } => serde_json::json!({ "session": session }),
        }
    }
}

/// A cloneable handle for broadcasting [`AppEvent`]s to all subscribers.
///
/// Cloning shares the same underlying channel; dropping every clone closes it.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<AppEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Broadcast channel capacity. Subscribers that fall further behind than
    /// this drop the oldest events (surfaced as `RecvError::Lagged`, which the
    /// bridge ignores).
    const CAPACITY: usize = 256;

    /// Create a new, empty event bus.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(Self::CAPACITY);
        Self { tx }
    }

    /// Broadcast an event to all current subscribers.
    ///
    /// Silently drops the event when there are no subscribers — emitters never
    /// need to know whether a frontend is currently attached.
    pub fn emit(&self, event: AppEvent) {
        let _ = self.tx.send(event);
    }

    /// Subscribe to the event stream. Each receiver observes every event sent
    /// after it subscribed.
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.tx.subscribe()
    }
}
