Generated from `AGENTS.md` by file-enforcer.

Claude Code's question tool is unreliable:
a question can come back looking declined,
or the user can interrupt right after answering it.
Read either as unanswered,
never as a refusal or a decision;
the user then clarifies in plain text,
and that plain-text reply is the answer.

Reset times provided by the harness are unreliable:
never record one or wait for one;
when a Claude Code native subagent model is refused,
try it again at the next delegation,
or delegate to another model meanwhile.

MWK:
 Monitors,
 wakeups,
 and waiting Claude Code native subagents rarely wake main agent:
 wait in one command that ends at a terminal state;
 emit only lines you'd act on,
 never routine progress.

WUC:
 When a wakeup,
 cron,
 or loop's work finishes,
 cancel it that turn (`ScheduleWakeup` stop,
 `CronDelete`);
 a stale fire is a harness slip to stop,
 not a task to redo.

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
Attach a diff as an `@path` argument,
not as prose.
It is the cross-vendor reviewer:
a Claude subagent is a second pass.
`timeout` here takes SECONDS.
Background it;
never poll or kill it;
it may never return.
If you ask GPT to find issues,
it will find issues.
Be sure to dismiss them via your own judgement at some point.

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
and cap its memory,
its CPU (`--property=CPUQuota=<threads>00%`),
its process count (`--property=TasksMax=<count>`)
and its disk weight (`--property=IOWeight=<1-10>`,
never above 10,
lower for disk-heavy work),
each sized to that command,
because a transient unit sits outside the shared CPU cap that `claude-cpu-watch` applies to Bash commands.
Rationale:
`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`.

A command run in its own `systemd-run --user --scope` leaves the shared `claude-code-bash` cgroup,
and with it the CPU cap `claude-cpu-watch` keeps there;
that is fine.
