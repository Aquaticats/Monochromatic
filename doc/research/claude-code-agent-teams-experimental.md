# Why Claude Code Agent Teams is still experimental

Agent Teams lets one Claude Code session spawn other full Claude Code sessions as teammates,
coordinated through a shared task list and a file-backed mailbox.
It shipped as a research preview and is still gated behind an opt-in environment variable.
This note collects what the public evidence supports about why.

Three answers get blurred together and are worth keeping apart:
what "experimental" means mechanically in the shipped binary,
what Anthropic documents as limitations,
and what the release history and issue tracker show.
Only the middle one is Anthropic's own account of the feature.
Anthropic states nowhere in the sources gathered here
which limitation determines general availability,
so every causal reading in the "Assessment" section is this note's interpretation,
labeled as such.

## Verification basis

Facts in this note come from reading primary sources on 2026-08-16,
not from recall.

- Binary:
   Claude Code v2.1.233 at `/var/home/user/.local/share/claude/versions/2.1.233`,
  a Bun-compiled ELF of 324598064 bytes.
  Published to npm as `@anthropic-ai/claude-code@2.1.233` on 2026-08-14
  (`https://registry.npmjs.org/@anthropic-ai/claude-code`).
- Extraction method:
   `dd bs=1M skip=270` over the executable,
   then `strings --bytes 6`,
  producing 82219 lines and 33560290 bytes of printable text.
  The JavaScript bundle is minified,
  so identifier names are mangled and only string literals are verbatim.
- Documentation:
   [`https://code.claude.com/docs/en/agent-teams`][agent-teams-docs] and
  [`https://code.claude.com/docs/en/costs`][costs-docs],
   fetched 2026-08-16.
  The older `docs.claude.com/en/docs/claude-code/agent-teams` path now returns a 301 to the former.
- Feature-status labels:
   [the beta and research preview features page][status-labels],
  fetched 2026-08-16.
- Release history:
   `anthropics/claude-code` `CHANGELOG.md` retrieved through `gh api`.
- Issue counts and samples:
   GitHub `search/issues` through `gh api`,
   same date,
  with the exact queries given in the "What users report" section.

**Source quality note:**
the binary and the official documentation are primary.
The changelog is primary but curated by Anthropic,
so absence of an entry is not absence of a change,
and an entry saying a defect was fixed is evidence about the past,
 not the present.
Issue-tracker material is the weakest evidence here:
titles are unverified reporter allegations,
no report was reproduced,
and maintainer triage status was not checked.
No claim in this note rests on a secondary blog or aggregator.

**Reproduction limit:**
no team was run.
Every claim about the binary is static reading of minified bundle text.

## What "experimental" means mechanically

Enablement requires two independent conditions,
and the user controls only one of them.
From the extracted bundle,
the module exporting `isAgentSwarmsEnabled`:

```js
// claude-code 2.1.233, extracted bundle text
function udv(){return process.argv.includes("--agent-teams")}
function md(){
  if(!V.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS&&!udv())return!1;
  if(!rt("tengu_amber_flint",!0))return!1;
  return!0
}
```

The user-facing condition is the documented opt-in:
`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`,
 or the undocumented `--agent-teams` argument.

The other condition is a feature-flag lookup.
`rt(name, fallback)` resolves through `fkr` to `eY().getFeatureValueWithSource(name, fallback)`,
and the same bundle embeds the GrowthBook client SDK,
defaulting its API host to `https://cdn.growthbook.io`,
with a startup span named `growthbook_init`
and a refresh interval read from `tengu_gb_refresh_interval_minutes`
(default 360 minutes,
 clamped to the range 5 to 360,
 jittered).
So `tengu_amber_flint` is eligible for remote override through that evaluator,
its observed fallback is `true`,
and the CLI argument does not bypass it.
This investigation did not observe the flag's live source or value.

**That gating is not what distinguishes the feature.**
The bundle contains 342 distinct `rt("tengu_...")` gate lookups
covering features that are generally available,
so remote flag evaluation is how Claude Code ships everything,
not a marker of preview status.
What is distinctive is the conjunction:
Agent Teams additionally requires an explicit user opt-in that generally available features do not.

The settings panel classifies the feature the same way the environment variable name does.
Its `Experimental` group is
`["precomputeCompactionEnabled","timestamps","showStatusInTerminalTab","teammateMode","teammateDefaultModel"]`,
so both team-related settings sit in that group rather than in `Advanced`.
The documentation adds that the per-session override
"is experimental and doesn't appear in `claude --help`".

## Anthropic's own status vocabulary does not cover this feature

Anthropic publishes definitions for two feature-status labels
on [the beta and research preview features page][status-labels]:

> "Research preview features are earlier in development.
> They give you a first look at something we're exploring,
> and they're more likely to change significantly before they become generally available."

> "Beta features are further along in development.
> They're stable enough for regular use,
> and we're still refining how they work based on what we learn from people using them."

Neither definition attaches a support or stability commitment beyond those characterizations,
and neither defines the word "experimental".

Agent Teams is not listed on that page at all.
The Claude Code entries there are Claude Code Desktop (beta),
Claude Code Security Center (research preview),
Claude Code web (research preview),
and Code review (research preview).

The feature is therefore described three ways across Anthropic's own surfaces:
the changelog entry that introduced it calls it a "research preview",
the documentation and the environment variable call it "experimental",
and the status-label page tracks neither.
That is a gap in the public record:
no published criterion says what would move Agent Teams out of this state.

## What Anthropic documents as limitations

The documentation page opens with a warning that the feature is
"experimental and disabled by default",
and closes with a section headed "Limitations" that begins
"Agent teams are experimental."
Those limitations,
 quoted from the [Agent teams documentation page][agent-teams-docs],
sort into classes.
Some read as unfinished work and some as deliberate design constraints;
the documentation does not say which are which.

### State that does not survive the session

- "No session resumption with in-process teammates:
   `/resume` and `/rewind` do not restore in-process
  teammates.
   After resuming a session,
   the lead may attempt to message teammates that no longer exist."
- "One team per session:
   a session has exactly one team,
   scoped to that session.
  You can't create additional named teams or share a team across sessions."
- "Lead is fixed:
   the main session is the lead for its lifetime.
  You can't promote a teammate to lead or transfer leadership."

The resume limitation is the most consequential of the three,
because it does not merely lose the teammates.
It leaves the lead holding names that no longer resolve,
and the documented remedy is for the human to notice and say so.
The other two read as scope decisions rather than defects.

### Coordination that needs a human to close the loop

- "Task status can lag:
   teammates sometimes fail to mark tasks as completed,
   which blocks dependent tasks.
  If a task appears stuck,
   check whether the work is actually done and update the task status manually
  or tell the lead to nudge the teammate."
- "Shutdown can be slow:
   teammates finish their current request or tool call before shutting down."

A blocked dependent task is not cosmetic:
task claiming is how teammates pick up work,
so a missed completion stalls the team rather than slowing it.

### Topology limits

- "No nested teams:
   teammates cannot spawn their own teammates.
   Only the lead can manage the team."
- "No background subagents from in-process teammates ... returns an error,
  because a teammate's background work can't outlive the lead's process."

Both are stated with a rationale,
 which reads as design rather than as unfinished work.

### Environment and permission limits

- "Permissions set at spawn:
   all teammates start with the lead's permission mode.
  You can change individual teammate modes after spawning,
  but you can't set per-teammate modes at spawn time."
- "Split panes require tmux or iTerm2:
   the default in-process mode works in any terminal.
  Split-pane mode isn't supported in VS Code's integrated terminal,
   Windows Terminal,
   or Ghostty."

### The delegation change that comes with the flag

Enabling Agent Teams changes ordinary delegation,
which the documentation covers in troubleshooting rather than in its Limitations section:

> "while agent teams are enabled,
>  a subagent that Claude names launches as a teammate,
> so teams can form even when you didn't ask for one."

Subagents and teammates then differ in how results come back.
The documentation states this as a two-item list followed by a separate sentence,
quoted here with the seams marked:

> "Subagents:
>  Claude receives the subagent's result when it completes."
> "Teammates:
>  the idle notification reports that the teammate stopped,
>  without its output."
> "An orchestration flow that waits on subagent results can stall."

The user opts into Agent Teams,
but not separately into each conversion of a named subagent into a teammate.
Turning the flag on therefore changes how results reach the caller
in delegation the user never framed as team work.

### Cost

The [cost management documentation page][costs-docs] provides a quantified figure:

> "Agent teams use approximately 7x more tokens than standard sessions when teammates run in plan mode,
> because each teammate maintains its own context window and runs as a separate Claude instance."

That is token usage,
 not spend;
per-token pricing varies by model,
 so the two do not convert one to one.
The figure is also scoped to plan mode,
and no source gathered here establishes how often teammates run in plan mode.

## What the shipped code shows

The binary enforces the documented topology limits rather than merely documenting them,
each with its own telemetry category.
Literals are verbatim from the extracted bundle,
including their dash characters:

```text
subagent_nested_teammate
  Teammates cannot spawn other teammates — the team roster is flat.
  To spawn a subagent instead, omit the `name` parameter.

subagent_teammate_background_denied
  In-process teammates cannot spawn background agents.
  Use run_in_background=false for synchronous subagents.

Agent tool description, teammate branch
  `name` is unavailable here — teammates cannot spawn teammates.

/rename in a teammate session
  Cannot rename: This session is a teammate. Teammate names are set by the team leader.
```

The binary also carries a limit the documentation page does not mention:

> "Background agents and teammates are not supported for this credential kind.
> Run this from the main session,
> or switch the desktop app to a profile-based or API-key credential"

Three further literals show conditions the implementation represents and handles:

```text
getTeammateModeFromSnapshot called before capture - this indicates an initialization bug

The permission request could not be delivered to the team lead (mailbox write failed).
Retry the tool call.

Couldn't open a teammate pane — running in-process instead.
```

The teammate-mode literal is a guard.
`getTeammateModeFromSnapshot` passes it to the error reporter when its snapshot is unset,
then captures the setting lazily and continues with the `"in-process"` default.
This shows the code recognizes an invalid initialization ordering and recovers from it.
It does not show that the ordering currently occurs in practice.

The permission literal is the resolution value of a failed mailbox write.
Both call sites resolve the permission promise with `behavior: "ask"` carrying that message,
so the failure is returned to the requesting teammate rather than leaving it blocked indefinitely,
and the instruction is to retry.
What a mailbox-write failure does establish is that
the lead may never receive a request the teammate believed it sent.

The pane literal accompanies a warning notification with `color: "warning"`.
Split-pane selection therefore falls back to in-process execution non-fatally and with a warning,
not silently.

The transport underneath is a per-recipient JSON file with a sibling lockfile:
`writeToMailbox` computes `${inboxPath}.lock`,
creates the inbox with `writeExclusive`,
and logs `"Failed to write to inbox for ..."` on failure.
The documentation confirms the layout as `~/.claude/teams/{team-name}/inboxes/{agent-name}.json`.

**Interpretation caveat:**
an error string proves that a condition is represented and handled.
It does not prove that the condition is frequent,
 severe,
or the reason Anthropic withholds general availability.
No extracted text states a general-availability criterion.

## How much the feature has churned

Measured against `anthropics/claude-code` `CHANGELOG.md`,
which covers 365 releases from 0.2.21 through 2.1.233:

- 36 changelog lines match `teammate` or `agent team`,
   spread across 26 releases.
- 29 of those 36 begin with `Fixed` or `Improved`.
  The remaining 7 begin with `Added`,
   `Agent teams:`,
   `Simplified`,
   or `Subagent`;
  they were classified by that prefix,
   not by reading each entry's effect.
- 9 of the 36 mention a crash,
   leak,
   hang,
   memory problem,
   or loop.

The first entry sets the baseline:

> 2.1.32:
>  "Added research preview agent teams feature for multi-agent collaboration
> (token-intensive feature,
>  requires setting `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`)"

npm published 2.1.32 on 2026-02-05 and 2.1.233 on 2026-08-14,
 which is 190 days.
Counting `## ` headings in the changelog,
 160 releases shipped after 2.1.32.

**These entries describe defects Anthropic reports as fixed,
in releases preceding the v2.1.233 binary examined here.
They are evidence of sustained stabilization work,
 not of current behavior.**
Grouped by what broke:

- Memory retention:
   2.1.50 (completed teammate tasks never garbage collected),
  2.1.63 (long-running teammates retained all messages after compaction),
  2.1.69 (parent's full conversation history pinned for the teammate's lifetime).
- Crashes and loops:
   2.1.34 (crash when the setting changed between renders),
  2.1.114 (crash in the permission dialog on a teammate request),
  2.1.207 (crash loop from one malformed mailbox message,
   recurring every second
  until the file was deleted by hand).
- Whole-provider breakage:
   2.1.41 and 2.1.45 (teammates failing on Bedrock,
   Vertex,
   and Foundry).
- Message delivery:
   2.1.224 fixed `SendMessage` "reporting `Message sent` when the write to a teammate's
  inbox had actually failed".
- Enforcement gaps:
   2.1.69 fixed "teammates accidentally spawning nested teammates",
  2.1.98 fixed team members not inheriting the leader's permission mode
  under `--dangerously-skip-permissions`.
- Non-interactive hangs:
   2.1.71 fixed `--print` "hanging forever when team agents are configured".
- Encoding:
   2.1.145 fixed teammates with non-ASCII names "failing every API call
  due to invalid header encoding".

Several of these are attributed to Agent Teams by the changelog wording
but plausibly belong to shared machinery that teams exercise:
provider environment propagation,
 permission dialogs,
 compaction,
 and non-interactive exit handling
are not team-specific subsystems.
The team-specific entries are the roster,
 mailbox,
 and nested-spawn ones.

The interface also changed under existing users mid-preview.
Release 2.1.178,
 published 2026-06-15,
 removed the `TeamCreate` and `TeamDelete` tools entirely,
replaced explicit teams with one implicit team per session,
and left `team_name` "still accepted but ignored".
The documentation still carries a compatibility note about that change.
That release is 62 days before this note.

**No baseline was measured.**
Nothing here compares this churn rate against another preview feature
or against a generally available one,
so the counts describe activity without establishing that the activity is unusual.

## What users report

Counts from GitHub `search/issues` through `gh api` on 2026-08-16,
with the exact query strings:

- `repo:anthropics/claude-code is:issue "agent teams"`:
   928,
   of which `is:open` gives 205.
- `repo:anthropics/claude-code is:issue teammate`:
   969,
   of which `is:open` gives 249.
- `repo:anthropics/claude-code is:issue in:title "agent team"`: 410.
- `repo:anthropics/claude-code is:issue is:open in:title teammate`: 63.

**Data quality note:**
 the unrestricted queries match comment text as well as titles,
overlap each other heavily,
and have no denominator,
so they cannot establish defect density or unusual instability.
High counts also track usage.
The title-scoped counts are the tighter measure.

The sample that follows is the 30 most recently created open issues matching
`repo:anthropics/claude-code is:issue is:open in:title teammate OR "agent team"`
with `sort=created&order=desc&per_page=30`,
classified by title text alone,
each issue counted once,
with no duplicate detection applied.

- Message delivery and result reporting:
   8 issues (#87009,
   #86090,
   #86070,
   #85963,
   #85949,
  #84527,
   #84494,
   #83788).
- Feature requests rather than defects:
   4 (#86716,
   #86666,
   #83602,
   #82203).
- Failures reported while using teams but attributed to shared infrastructure by their own titles:
  4 (#86129 auto-updater,
   #84905 Remote Control,
   #82627 macOS provenance,
   #83366 tmux on Windows).
- Live and persisted state diverging:
   3 (#86518,
   #86174,
   #85955).
- Configuration not inherited:
   3 (#86006,
   #83533,
   #81852).
- Hook coverage:
   3 (#86285,
   #82665,
   #82418).
- User interface:
   2 (#86079,
   #83512).
- Lifecycle,
   isolation,
   and prompt caching:
   1 each (#85047,
   #84493,
   #85954).

Message delivery and result reporting is the largest cluster in this sample,
 at 8 of 30.
Reporters in that cluster allege,
 in their own words,
that an in-process teammate's final text "is discarded on idle"
leaving the coordinator a "payload-less `idle_notification`" (#84527),
that `SendMessage` to `team-lead` "false-succeeds into an orphaned inbox" (#85949),
and that deliveries are "batched 40-min" (#84494).
The payload-less idle notification is documented behavior rather than a transport failure:
the documentation states the notification "doesn't carry the teammate's output".
The reports allege that the documented alternative,
a teammate messaging the lead directly,
does not reliably arrive either.

One report deserves naming because it bears on an assumption a reader is likely to make.
Teammates are described as independent Claude Code sessions,
which invites the assumption that they isolate like separate sessions do.
Reporter #84493 alleges that worktree binding is session-scoped,
so "any in-process teammate's `EnterWorktree`/`ExitWorktree` silently repoints every other agent
in the session".
This was not reproduced,
and no source gathered here documents the intended working-directory isolation model,
which remains an open question about the feature.

## Local corroboration, and its limits

This repository records a finding about adjacent delegation mechanisms.
`doc/decision/general-purpose-subagent-ban.md` states that
"Programmatic steering via `SendMessage` is unreliable;
 in the last test it did not work
even though it is documented to",
and that `spawn-claude` result forwarding "is unreliable and needs manual monitoring".
The generated `CLAUDE.md` preamble repeated the `SendMessage` finding to every session until 2026-10-29,
when the preamble paragraph was removed because Claude Code updates fixed most of the underlying problems.

That document is about general-purpose subagents and `spawn-claude` child sessions,
not about Agent Teams,
and it must not be cited as evidence about teams.
Manual `SendMessage` steering and automatic final-result forwarding are also different channels,
which the repository document treats separately.
What it supports is narrower:
this repository has independently observed analogous coordination failures
in other delegation mechanisms.
That similarity does not establish a shared implementation or an Agent Teams defect.

## What this evidence does not establish

Stated with the same confidence as the findings.

- No fetched source says which limitation blocks general availability.
  The documentation enumerates limitations without ranking them or naming exit criteria,
  the status-label page does not list the feature,
  and Anthropic publishes no general-availability date.
- The live value and source of `tengu_amber_flint` are unknown.
  Only the local fallback of `true` and the presence of the GrowthBook evaluator were observed.
- Error strings prove handling,
   not incidence.
  No failure rate,
   no latency figure,
   and no scale limit appears in the extracted text or the docs.
- Team-size ceilings,
   mailbox queue bounds,
   and lock-contention behavior were not located.
  The documentation says "There's no hard limit on the number of teammates" and recommends 3 to 5,
  which is guidance,
   not a measured limit.
- The working-directory and edit-isolation model for teammates was not established
  from either the documentation or the binary.
- No team was run,
  so no claim here is a reproduction.
  Issue contents are reporter allegations classified by title.
- No churn or gating baseline was measured against other Claude Code features,
  beyond the count of 342 `rt("tengu_...")` gates showing that remote gating is universal.

## Assessment

This section is this note's interpretation of the evidence,
not Anthropic's stated reasoning.
Anthropic has not publicly identified a blocker or a general-availability criterion for Agent Teams.

Reading the material as a whole,
 "experimental" here appears to cover three separable things.

The first is a set of documented behaviors that require human intervention.
Task status lags and blocks dependents,
resume leaves the lead addressing teammates that no longer exist,
and the documented remedies are for the operator to notice and correct by hand.
These are current,
 vendor-stated behaviors in the documentation for this version,
which is the strongest evidence in this note.

The second is delegation semantics.
Enabling the flag converts named subagents into teammates,
and the documentation states plainly that an orchestration flow waiting on subagent results
can stall as a result.
A flag that changes how unrelated workflows return results is a reason for opt-in
independent of any defect.

The third is cost.
Roughly sevenfold token usage in plan mode is a design consequence of giving each teammate
its own context window,
and it argues against enabling by default regardless of stability.

Separating the evidence by tense matters for the reliability question,
because it is easy to overstate.
Currently documented:
 task-status lag,
 resume gaps,
 output-less idle notifications.
Reported fixed before this version:
 false `Message sent` on inbox write failure,
the mailbox crash loop,
 duplicate idle notifications,
 memory retention.
Currently alleged but unreproduced:
 batched deliveries,
 orphaned inboxes,
discarded final text,
 registry entries that outlive their agents.
Deliberate semantics rather than failure:
 the idle notification carrying no output.

Those four categories all touch the mailbox and lifecycle layer,
which is the narrowest description of the feature's weak point that the evidence supports.
It is not a claim that the layer currently fails at any measured rate,
and several changelog entries and issue reports grouped under Agent Teams
plausibly belong to shared subsystems that teams merely exercise.
What can be said is that message delivery and lifecycle state
are where the documented limitations,
 the fix history,
 and the current reports converge,
and that no public source shows that convergence resolved.

## What this means for this repository

Recorded as context,
 not as a decision.

Measured local state on 2026-08-16,
so the points that follow are conditional on a change nobody has made:

- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` is set in no session environment variable
  and in none of `~/.claude/settings.json`,
  `~/.claude/settings.local.json`,
  `.claude/settings.json`,
  or `.claude/settings.local.json`.
  Agent Teams is off here.
- `~/.claude/settings.json` nonetheless carries `"teammateMode": "auto"`.
  That setting is inert while the feature is off.
  Were the feature enabled,
   `"auto"` selects split panes inside tmux or iTerm2
  rather than the `"in-process"` default that shipped at v2.1.179.
- `.claude/settings.local.json` wires both `TeammateIdle` and `TaskCompleted` to `cctt`.
  Per [the hooks documentation](https://code.claude.com/docs/en/hooks),
  `TeammateIdle` fires only "when an agent team teammate is about to go idle",
  so that entry cannot fire while the feature is off.
  `TaskCompleted` fires "when a task is being marked as completed"
  and is not tied to teams,
  so that entry is live in ordinary sessions.

The `TeammateIdle` entry is dead configuration rather than a fault:
it costs nothing while unreachable,
and it would take effect if the feature were ever enabled.

- Enabling `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` here would change existing behavior,
  not just add a capability.
  The `CLAUDE.md` preamble currently tells agents that in-process subagents
  "forward their results back to you reliably".
  With the flag on,
   a subagent Claude names launches as a teammate,
  and per the documentation the lead then gets an idle notification without the output.
  That preamble would become wrong.
- The repository's existing preference for in-process subagents over `spawn-claude`
  rests on reliable result forwarding.
  Agent Teams sits on the far side of that tradeoff:
  richer coordination,
   weaker forwarding.
- The largest cluster of current reports against teams,
  a completed agent whose output does not reach the coordinator,
  resembles the failure this repository already documented for `spawn-claude`.
  The resemblance is in user-visible shape only;
  no shared implementation was established.

[agent-teams-docs]: https://code.claude.com/docs/en/agent-teams
[costs-docs]: https://code.claude.com/docs/en/costs
[status-labels]: https://support.claude.com/en/articles/14503520-available-beta-and-research-preview-features
