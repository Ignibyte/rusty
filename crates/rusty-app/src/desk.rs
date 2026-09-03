//! The `Desk` QML type: what the top bar reads off the machine. Memory in use, the
//! CPU's share since the last reading, the clock, and Hyprland's workspaces with the
//! active one, read through `hyprctl -j` when the app runs under Hyprland. Offscreen
//! or on another compositor the strip shows a static one-to-four with the first lit,
//! which is what the mock shows.

use core::pin::Pin;
use std::process::Command;

use cxx_qt::CxxQtType;
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
        #[qproperty(QString, memory)]
        #[qproperty(QString, cpu)]
        #[qproperty(QString, clock)]
        #[qproperty(QString, workspaces)]
        #[qproperty(i32, active_workspace)]
        #[qproperty(bool, hyprland)]
        #[qproperty(QString, user)]
        type Desk = super::DeskRust;

        /// Take every reading again; the shell calls this on a timer.
        #[qinvokable]
        fn refresh(self: Pin<&mut Desk>);

        /// Ask Hyprland to show a workspace.
        #[qinvokable]
        fn switch_workspace(self: &Desk, id: i32);
    }
}

/// The Rust side of [`qobject::Desk`].
pub struct DeskRust {
    memory: QString,
    cpu: QString,
    clock: QString,
    /// Workspace ids as a JSON array, ascending.
    workspaces: QString,
    active_workspace: i32,
    hyprland: bool,
    /// The login name, for the rail's avatar.
    user: QString,
    /// The last `/proc/stat` totals, for the CPU share.
    last_cpu: Option<(u64, u64)>,
}

impl Default for DeskRust {
    fn default() -> Self {
        let mut desk = Self {
            memory: QString::default(),
            cpu: QString::default(),
            clock: QString::default(),
            workspaces: QString::from("[1,2,3,4]"),
            active_workspace: 1,
            hyprland: std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")
                .is_some_and(|v| !v.is_empty()),
            user: QString::from(&std::env::var("USER").unwrap_or_default()),
            last_cpu: None,
        };
        desk.take_readings();
        desk
    }
}

impl DeskRust {
    fn take_readings(&mut self) {
        self.memory = QString::from(&memory_used().unwrap_or_default());
        let (share, totals) = cpu_share(self.last_cpu);
        self.last_cpu = totals;
        self.cpu = QString::from(&share.map(|s| format!("{s:02}%")).unwrap_or_default());
        self.clock = QString::from(&clock());
        if self.hyprland {
            if let Some((ids, active)) = hyprland_workspaces() {
                self.workspaces = QString::from(&format!(
                    "[{}]",
                    ids.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                self.active_workspace = active;
            }
        }
    }
}

impl qobject::Desk {
    /// See [`DeskRust::take_readings`]; every property is set through its setter so
    /// bindings update.
    pub fn refresh(mut self: Pin<&mut Self>) {
        let mut inner = DeskRust {
            memory: QString::default(),
            cpu: QString::default(),
            clock: QString::default(),
            workspaces: self.workspaces().clone(),
            active_workspace: *self.active_workspace(),
            hyprland: *self.hyprland(),
            user: self.user().clone(),
            last_cpu: self.rust().last_cpu,
        };
        inner.take_readings();
        self.as_mut().rust_mut().last_cpu = inner.last_cpu;
        self.as_mut().set_memory(inner.memory);
        self.as_mut().set_cpu(inner.cpu);
        self.as_mut().set_clock(inner.clock);
        self.as_mut().set_workspaces(inner.workspaces);
        self.as_mut().set_active_workspace(inner.active_workspace);
    }

    /// `hyprctl dispatch workspace <id>`; nothing happens off Hyprland.
    pub fn switch_workspace(&self, id: i32) {
        if !*self.hyprland() {
            return;
        }
        let _ = Command::new("hyprctl")
            .args(["dispatch", "workspace", &id.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Memory in use as Waybar shows it: total minus available, in gigabytes to one place.
fn memory_used() -> Option<String> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let field = |name: &str| -> Option<u64> {
        text.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
    };
    let total = field("MemTotal:")?;
    let available = field("MemAvailable:")?;
    let used = total.saturating_sub(available) as f64 / (1024.0 * 1024.0);
    Some(format!("{used:.1}G"))
}

/// The CPU's busy share since the last totals, and the totals for next time.
fn cpu_share(last: Option<(u64, u64)>) -> (Option<u64>, Option<(u64, u64)>) {
    let Some(text) = std::fs::read_to_string("/proc/stat").ok() else {
        return (None, last);
    };
    let Some(line) = text.lines().next() else {
        return (None, last);
    };
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    if values.len() < 4 {
        return (None, last);
    }
    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    let total: u64 = values.iter().sum();
    let share = last.and_then(|(prev_total, prev_idle)| {
        let dt = total.saturating_sub(prev_total);
        let di = idle.saturating_sub(prev_idle);
        (dt > 0).then(|| ((dt - di.min(dt)) * 100 / dt).min(100))
    });
    (share, Some((total, idle)))
}

/// The local time as `HH:MM`.
fn clock() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

/// Hyprland's workspace ids, ascending, and the active one.
fn hyprland_workspaces() -> Option<(Vec<i32>, i32)> {
    let out = Command::new("hyprctl")
        .args(["-j", "workspaces"])
        .output()
        .ok()?;
    let list: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).ok()?;
    let mut ids: Vec<i32> = list
        .iter()
        .filter_map(|w| w.get("id")?.as_i64())
        .map(|i| i as i32)
        .filter(|i| *i > 0)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let out = Command::new("hyprctl")
        .args(["-j", "activeworkspace"])
        .output()
        .ok()?;
    let active: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let active = active.get("id")?.as_i64()? as i32;
    if !ids.contains(&active) {
        ids.push(active);
        ids.sort_unstable();
    }
    Some((ids, active))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings_have_the_shape_the_top_bar_shows() {
        let memory = memory_used().unwrap();
        assert!(memory.ends_with('G') && memory.contains('.'), "{memory}");
        let (first, totals) = cpu_share(None);
        assert!(first.is_none() && totals.is_some());
        let (total, idle) = totals.unwrap();
        let (second, _) = cpu_share(Some((total.saturating_sub(200), idle.saturating_sub(100))));
        assert!(second.is_some_and(|s| (45..=55).contains(&s)), "{second:?}");
        assert_eq!(clock().len(), 5);
    }
}
