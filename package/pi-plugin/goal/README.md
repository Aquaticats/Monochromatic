# @monochromatic-dev/pi-goal

Private repository-owned Pi extension that keeps one explicit objective active
until private independent review accepts finalized work.
It is a stop hook,
not a task list,
background worker,
interruption manager,
or unrelated-tool policy.

## Command contract

Start or replace active objective:

```text
/goal <objective>
```

Clear current goal record:

```text
/goal clear
```

Bare `/goal` and removed `status`,
`edit`,
`pause`,
`resume`,
and `--tokens` forms return usage diagnostics.
Starting another objective immediately supersedes active or terminal state without confirmation.
Objectives are trimmed and limited to 4,000 characters.
Put longer instructions in file and reference its path from objective.

Goal completion is not a primary-model tool.
Primary model receives only exact user objective and actionable task-level remaining work.
It does not receive goal generation identifiers,
reviewer identity,
verdicts,
or stop-hook protocol.

## Persistence and settlement

Goal events are branch-local Pi custom entries outside model context.
Extension reconstructs state from selected active branch only.
Restoring or navigating into active state rotates private generation identifier
but does not automatically start work.

Task kickoff and continuation are extension-authored custom messages,
not human messages.
Their model-visible content contains only objective or direct remaining work.

After Pi fully settles,
extension starts at most one private review when exact generation remains active,
Pi is idle,
no human steering or follow-up is pending,
no live `@aliou/pi-processes` process is observed,
and run was not aborted.
Pi-owned retries and overflow compaction finish before this decision.
A live process leaves goal active but suppresses goal-owned review.
Process completion alerts retain their configured turn behavior,
so goal does not poll or override process notifications.

Human decisions use Pi's `ask_user_question` tool inside primary run.
That tool blocks until user answers or cancels,
so settlement review does not need separate blocked-on-user state.

## Private independent review

Primary model cannot approve its own finalized output.
Extension selects highest expected-cost authenticated model in effective Pi scope
after excluding active primary model.
Complete reviewer attempt uses forced structured output,
bounded direct-JSON retries,
and ten-second timeout.
When candidate fails transport or contract validation,
up to two distinct fallback candidates run concurrently and first valid verdict wins.
A valid denial does not trigger another reviewer.

Review sends exact objective and active-branch session evidence recorded after current goal started
to another configured model provider.
This may incur provider cost and may disclose post-goal prompts,
assistant text,
and finalized tool results to that provider.
Extension does not send pre-goal history,
custom goal state entries,
or abandoned-branch history.
Oversized evidence is deterministically truncated and disclosed to reviewer.

Private verdict is binary:

- approval records terminal completion without another primary-model response
- denial injects only contracted task-level `remaining_work` and starts one guarded continuation

There is no arbitrary denial cap.
Goal continues until approval,
user abort,
or `/goal clear`.

Approval appends durable TUI-only `Goal complete` entry excluded from model context.
Expanded entry shows reviewer identity,
approval rationale,
attempted reviewer identities,
and evidence-truncation status.

When every configured reviewer attempt fails:

- TUI mode opens one combined `Accept` or `Reject [optional reason]` dialog.
  Escape does not settle it.
- RPC,
  JSON,
  and print modes terminate goal as `review_unavailable` without another turn.

## Local installation

Build and verify before changing global Pi settings:

```bash
mise run //package/pi-plugin/goal:build
mise run //package/pi-plugin/goal:lint:types
mise run //package/pi-plugin/goal:lint:oxlint
mise run //package/pi-plugin/goal:test:unit
mise run //package/pi-plugin/goal:verify:extension
mise run //package/pi-plugin/goal:verify:pi-runtime
```

Inspect global package settings before migration:

```bash
pi list
```

Retired source was already absent from observed global settings on 2026-07-17.
For that confirmed state,
skip removal and install only repository package.
Unreferenced package directory can remain under Pi global npm directory without being active.
Do not delete installed-package directories by hand.

If `npm:@narumitw/pi-goal` is listed on another machine,
close running Pi processes and back up `~/.pi/agent/settings.json` before replacement.
Remove only that source while Pi is stopped:

```bash
pi remove npm:@narumitw/pi-goal
```

Install globally from this checkout:

```bash
pi install /var/home/user/Monochromatic/package/pi-plugin/goal
```

Run `pi list` again.
Repository path must appear once,
`npm:@narumitw/pi-goal` must be absent,
and every unrelated package entry must remain unchanged.
Restart Pi after package settings change so no previous extension runtime remains loaded.

Do not use `pi install -l` or edit project-local `.pi/settings.json`.
All state-mutating verification uses disposable agent directories and session files.
See
[`doc/troubleshooting/pi-goal-pending-completion-message-omission.md`][goal-troubleshooting]
for retired completion-tool failure and source trace.

## Rollback

Remove only repository package:

```bash
pi remove /var/home/user/Monochromatic/package/pi-plugin/goal
pi list
```

Restart Pi after removal.
For confirmed pre-migration state,
this restores package-free goal configuration while preserving unrelated global entries.
It does not restore retired command behavior.

Reinstalling `npm:@narumitw/pi-goal` restores package with goal-owned `tool_call` blocker
and is not supported rollback.
For isolated diagnosis only,
install it explicitly,
reproduce in disposable session,
then remove it again:

```bash
pi install npm:@narumitw/pi-goal
pi remove npm:@narumitw/pi-goal
```

[goal-troubleshooting]: ../../../doc/troubleshooting/pi-goal-pending-completion-message-omission.md
