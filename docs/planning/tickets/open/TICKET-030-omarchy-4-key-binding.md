---
title: TICKET-030-omarchy-4-key-binding
status: open
ticket_number: 030
type: fix
created: 2026-09-05
intake:
pipeline_spec: TBC
---

# TICKET-030-omarchy-4-key-binding

## Summary

The key snippet Rusty ships, `omarchy/hyprland-bindings.conf`, is Omarchy 3 syntax, and
the installer's hint says to append it to `~/.config/hypr/bindings.conf`. Omarchy 4
configures Hyprland from `~/.config/hypr/*.lua` and reads no `.conf` there, so on the
desktop Rusty targets the snippet binds nothing. Ship the binding as Lua for Omarchy 4,
keep the `.conf` for Omarchy 3, and have the installer's hint name the file the box reads.

## Why

Found at TICKET-029's validation on 2026-09-05: SUPER+ALT+R had been dead on the dev box
since the Quattro upgrade the day before (`hyprctl binds` listed 228 binds, none for
Rusty; `hyprland.lua` requires `hypr.bindings` and nothing sources `bindings.conf`). The
box was fixed by hand in `bindings.lua`; the repo still tells every Omarchy 4 user to do
the dead thing.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The repo shall ship the Rusty key binding in Omarchy 4's form: a Lua snippet under `omarchy/` calling `o.bind("SUPER + ALT + R", "Rusty", …)` with `omarchy-launch-or-focus` and `rusty session start`, its comment saying where it goes. | file review; the shipped-files test still green |
| REQ-002 | WHEN the installer runs on a box where `~/.config/hypr/hyprland.lua` exists, it shall point at the Lua snippet and `~/.config/hypr/bindings.lua`; WHEN only `hyprland.conf` exists, it shall keep pointing at the `.conf` snippet. | installer run on the box (Omarchy 4); reading of the other branch |
| REQ-003 | The PKGBUILD shall install the Lua snippet under `/usr/share/rusty/` beside the `.conf` one, and `omarchy/README.md` and the wiki shall name both and say which Omarchy reads which. | `package()` review; doc review; `openwiki_finish` complete |

## Scope

- In: `omarchy/hyprland-bindings.lua` (new), `omarchy/hyprland-bindings.conf` (its comment
  names Omarchy 3), `omarchy/install.sh` (the hint), `packaging/PKGBUILD`,
  `omarchy/README.md`, the wiki page `development-and-validation.md`.
- Out: writing the binding into the user's config (the installer points, as it does for
  the compositor drop-in); any other key; Omarchy 3 beyond keeping the `.conf` file.

## Notes

- Pipeline spec: TBC
- Related docs: `omarchy/README.md` (the file table), `AAR-029-session-commands.md` (§4),
  and in omarchy-ops `docs/ops/dev-box.md` (the box's `bindings.lua` line).
- Promoted from intake: none; opened as a follow-up of TICKET-029.
- Follow-ups opened: none.
