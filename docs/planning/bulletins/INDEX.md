# Bulletins

Standing notices for anyone starting work. A bulletin marked **critical** blocks work until
it has been read and its instruction followed.

| Date | Level | Bulletin |
|---|---|---|
| 2026-09-02 | notice | The workflow in `CONSTITUTION.md` and `.claude/skills/rusty-workflow/` is new. Run `scripts/check-pipeline.sh` at the start of a session; run `scripts/setup-pipeline-tools.sh` once per clone for CodeGraph. |
| 2026-09-02 | notice | Chad uses this box while agents work on it. Do not drive the app with synthetic keystrokes or switch his workspace for screenshots; verify from logs with `RUSTY_DEBUG=1` and throwaway data. |
| 2026-09-03 | notice | In a session where Claude Code's hooks do not fire (seen twice on 2026-09-03 in the app's terminal), `openwiki_finish` returning `complete` writes no receipt; feed the genuine result to `.claude/hooks/record-pipeline-tool-use.sh` on stdin as `{"tool_name":"mcp__openwiki__openwiki_finish","tool_response":{"content":[{"type":"text","text":"{\\"status\\":\\"complete\\"}"}]}}` with the spec still under `active/`, then `bin/gate.sh --verify`. Never write the receipt itself. |
