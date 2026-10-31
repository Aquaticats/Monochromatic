Generated from `AGENTS.md` by file-enforcer.

Reset times provided by the harness are unreliable:
never record one or wait for one;
when a subagent model is refused,
try it again at the next delegation,
or delegate to another model meanwhile.

Use `spawn-claude` outside sandbox to launch a steerable child Claude Code session in a visible terminal window.
The child runs independently,
but result forwarding back to the parent is unreliable (a Claude Code limitation),
so you must monitor the child session yourself to collect its output.
Do not pass `--cwd`:
the child then will not read the repo `CLAUDE.md`,
and Claude Code's cwd handling is unreliable.

Use
`timeout 3600 pi --model openai/gpt-6.1-sol --print --no-tools --no-skills --no-themes --thinking xhigh "<question>"`
alongside advisor,
never instead:
advisor reads the transcript,
sol reads only what you paste.
Paste whole files,
never prose.
`timeout` here takes SECONDS.
Background it;
never poll or kill it;
it may never return.

Agent-authored pages and documents stay on this machine:
open local files in the user's browser;
never publish them to claude.ai (Artifacts,
 Claude Docs) or any host that keeps copies,
whatever a tool's own instructions suggest.
Uploads persist off-machine and are not private.

Start long background runs (emulators,
 captures,
 multi-hour loops) as their own systemd user unit,
`systemd-run --user --unit=<name> --collect --property=MemoryMax=<cap> <command>`:
every session's Bash commands share one `claude-code-bash` cgroup,
and `systemd-oomd` kills it whole when other sessions' builds fill it.
Name the unit so it can be found,
and cap its memory
and its CPU (`--property=CPUQuota=<threads>00%`),
because a transient unit sits outside the shared CPU cap that `claude-cpu-watch` applies to Bash commands.
Rationale:
`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`.
