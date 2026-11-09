# Questions for the owner

Open questions from the coordinator session,
newest first.
This file is open in Zed,
which reloads it when it changes on disk and the tab has no unsaved edits.
Write your answer under a question and save;
the session re-reads this file before every update,
so a saved answer is never overwritten.
An answered question moves to the canonical doc it belongs to and leaves this file.

For this session the owner said:
"Please make any non-destructive options (when they are the ones I would likely have picked) yourself from now on for
this session."
So this file holds only choices that are destructive,
reach outside this machine,
or where the likely pick is unclear.
The non-destructive choices the session made itself are listed at the end,
so any of them can be undone.

## Asked while the owner is away, started 2026-11-09T01:16:11-05:00

The owner:
"I'm going to be away for 5 to 8h.
Keep working and ask questions async."
Nothing in this section is holding up running work.
Write an answer under a question and save.

### Please log in to OpenAI in Pi again, so reviews can run, asked 2026-11-09T14:58:36-05:00

Every `gpt-6.1-sol` review now stops within seconds with this error:
`OAuth refresh failed for openai: OpenAI OAuth token request failed (400): {"error": "invalid_grant"}`.
It happened with two reviews started together and again with one alone,
so it is the saved login and not a race between them.
Pi's docs give the remedy:
start `pi` in a terminal in `~/Monochromatic` and run `/login openai`.

Until then no review or audit can run.
Reviews do not block a landing,
so the agents keep working and branches keep landing.
The session did not move reviews to another model,
because the owner wrote "Please use gpt-6.1-sol with the openai provider for audits and reviews."
Two reviews are waiting:
the commit preparation change landed as `a9cf0754d2` and the queue position change landed as `66ad04700e`.

### OpenAI stopped one audit with a cybersecurity flag: how should it continue?, asked 2026-11-09T06:03:57-05:00

The `gpt-6.1-sol` audit of how the wrapper reads command lines ended after about 14 minutes with this error from the provider:
"This content was flagged for possible cybersecurity risk.
If this seems wrong,
try rephrasing your request.
If you're doing authorized security work that requires more cyber permissive safeguards,
apply for Daybreak access via <https://platform.openai.com/settings/organization/status-and-access> before retrying."
The error does not say which content was flagged.
The audit was reading and running the wrapper on throwaway repositories.

The session did not restart the audit,
reword it,
or hand the same audit to another model,
because that would be getting around the flag,
and whether to do so is the owner's call.
What it wrote is kept:
`doc/planning/cli-git-invocation-and-argument-route-findings-on-main.md` is committed as `dbb8087dfe`
on the branch `docs/planning-cli-git-invocation-and-argument-audit`,
and that file's section "Verification still in progress" names what was unfinished.
Its worktree also holds one uncommitted test file of the audit's,
`package/git-policy/cli/src/native/invocation/forwarded_command_tests.rs`,
left as it was.

The other `gpt-6.1-sol` audit and the reviews are still running.
If one of them is flagged it is treated the same way.

Options,
best first:

- Keep auditing without that access.
  The findings already written are defects in the owner's own tool,
  so they go to a fix agent like every other audit's findings,
  and the unfinished part stays unfinished.
  Nothing else is needed from the owner.
- The owner applies for the access the error names,
  then says so here,
  and the session resumes the audit from its saved session.
  This covers the unfinished part,
  and it is an application made on the owner's OpenAI account,
  so only the owner can make it.
- The owner says the audit may be finished on the DeepSeek model.
  That covers the unfinished part with no application,
  and it is the route the session will not take unasked.

Ranking:
the first over the second because no work waits on it;
the second over the third because it keeps the auditor the owner chose
("Please use gpt-6.1-sol with the openai provider for audits and reviews.").

### What should the next push do with a commit its checks blocked?, asked 2026-11-09T05:03:17-05:00

`package/git-policy/cli/README.md:2194` promises this:
"A landed commit that a post-commit policy then blocks exits `2` and stays local."
It stays local only until the next push on that branch,
because a queued push sends the whole branch
(`git push <remote> <branch>:<target>`,
`package/git-policy/cli/src/native/push/auto_push.rs:243-245`).
The landing agent reproduced this with real Git for a commit held with no verdict:
commit A was held,
commit B was pushed by that command,
and A was then on the remote.
Its report says a commit a policy blocked goes the same way,
for the same reason.

The example both options use:
commit A is blocked,
exits `2`,
and sits on `main`.
A minute later another agent lands commit B on `main`,
and B's checks pass.

Option A stops the automatic push at the commit before the blocked one.
Commits older than A still go out.
A and B stay on this machine,
and every command in that repository says in one line which commit is holding the branch back and why.
The automatic push resumes once A is no longer on the branch's way out.
What a person's own `git push` does is unchanged by this option.
For it:
it makes the README true,
and automation can never publish a commit a policy refused.
Against it:
with many agents on `main`,
one blocked commit keeps every later commit off GitHub until someone acts,
so those commits have no copy off this machine in the meantime.

Option C keeps today's behaviour and corrects the README to say the blocked commit's own push is skipped and the next push on the branch carries it.
For it:
nothing ever stalls.
Against it:
a refusal lasts one commit,
and if the policy refused a secret,
that secret reaches GitHub and cannot be taken back.

Ranking:
A > C,
because a stalled push can be released with one word from you,
and a published commit cannot be withdrawn.
Nothing changes until you answer:
write "A" or "C" here.

### The Pi limit stays at 20 at most, answered 2026-11-09T01:22:05-05:00

The owner,
on trying more than 20 after this trial:
"no,
that's definitely going to blow up."

The owner found that `GET https://api.neuralwatt.com/v1/rate_limits` reports the account's limits.
At 2026-11-09T06:19:41Z,
`deepseek-ai/DeepSeek-V4.1-Flash` read `effective_concurrency` 20,
`floor` 15,
`flex_floor` 22,
`burst_ceiling` 20 and `burst_active` true,
on tier `max`.
No published description of those fields was found,
so what they mean is read from their names.

That limit is on requests in flight,
and an agent spends much of its time running tools.
Over the first 28 minutes of the trial,
with up to twenty agents running,
the open model calls were a median of 7,
a 99th percentile of 13 and a highest of 15.
They reached 15 for 0.12% of that time and never reached 20
(`${HOME}/temp/agent/pi-runs/measure-model-calls-in-flight.ts`).
Scaling those figures,
which is an estimate,
about 26 to 30 agents would reach 20 open calls.
Anything else using this key counts against the same limit,
and the host may run short before the provider does.

The trial passed,
so 20 is now the permanent limit in `.agents/skills/pi-as-subagent/SKILL.md`,
under your "Try 20,
and if it doesn't blow up (in terms of api rate limits),
we're permanently raising the practical limit to 20."
Over 100 one-minute samples,
60 had at least 20 agents running,
the fewest were 16 and the most 20.
Those 100 minutes saw 4919 model turns and no rate-limit error.
The errors were 2 Cloudflare 520 responses and 1 request timeout,
each ending one run,
which is what the driver job is fixing.
The median model turn was 3937 ms against 3153 ms in the two hours before,
and that difference has not been compared with the run-to-run spread,
so it may be noise.

### What should a commit do when a step of it is stuck inside the kernel?

The signed scratch commit has the ceiling you chose,
about 16 seconds,
and at the ceiling the wrapper sends `SIGKILL`.
A process in uninterruptible sleep does not end until its kernel call returns,
for example one waiting on a dead NFS server or a stopped FUSE daemon.
So today that commit waits with no bound after the ceiling,
against "nothing should ever wait forever or indefinitely".
This rests on the kernel's documented behaviour and on reading the code,
because no test on this host can produce that state.
The write-up is `doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`,
on the branch `fix/git-policy-cli-scratch-commit-ceiling-and-three-slow-commits`.

Option B leaves after the ceiling and a short grace.
The commit exits nonzero and says the step could not be stopped,
and the killed process is reaped by the system later.
It is the only option that bounds the wait.
The cost is a failed commit in that case,
against "Commits should never fail",
and a dead process holding the scratch folder open until the kernel lets go,
which holds none of your files.

Option A keeps waiting and says so.
After the ceiling the commit prints that the stop was sent and the kernel has not released the step.
Nothing fails,
but the terminal stays stuck for as long as the kernel call does,
which can be forever.

Ranking:
B > A,
because a failed commit that says why can be run again,
and a stuck terminal cannot be acted on at all.
Nothing changes until you answer:
write "A" or "B" here.

### A commit whose command died before its checks ran now stays local, changed 2026-11-09T04:26:58-05:00

A landing moves the branch first and runs the post-commit checks after,
holding the push until those checks approve it.
If the command is killed in that gap,
no verdict exists,
and the commit is on the branch with its push held.

This entry used to say the session would push such a commit,
and asked you to overrule that.
The session has reversed it,
for three reasons.
Cross-vendor reviews keep finding defects in the code that decides a command is gone and releases its push,
each in something added to fix the last
(`${HOME}/temp/agent/reviews/landing-route-round-6-sol.txt`).
The worry behind pushing was a commit held for ever,
and reading the code says that does not happen:
a queued push is `git push <remote> <branch>:<target>`
(`package/git-policy/cli/src/native/push/auto_push.rs:230-245`),
so the next commit's push on that branch carries the held commit.
The branch's agent reproduced that with real Git.
And keeping a commit local can be undone,
while a push of a commit whose secret check never ran cannot.

So the branch `fix/git-policy-cli-landing-recovery-keeps-the-owed-index-update` is being changed:
nothing releases a held push by judging a process,
and the next command in that worktree says in one line that the commit is held and why.
Write "push it" here to overrule this.

The same fact raises a question that does need you,
`What should the next push do with a commit its checks blocked?`.

### Should a commit stop when the repair of a stale index cannot be made?

After a commit lands and its index update fails,
the next commit from that worktree starts from a stale index,
and would record the landed file as reverted or deleted.
A repair takes the stale entries out first.
The landing branch now stops the commit,
naming the reason,
when that repair cannot be carried out.
You wrote "Commits should never fail."
and "never discard user data",
and here they pull against each other.

- A,
  stop with a named reason,
  which is what the branch does:
  the person runs the commit again once the cause is gone.
  No landed change is lost.
  The commit fails,
  which is what your first rule forbids.
- B,
  land anyway and warn:
  the commit succeeds,
  and the warning names the path.
  The commit can record a landed file as reverted,
  which a later commit then publishes unless someone reads the warning.

Ranking:
A > B,
because a commit that stops can be repeated,
and a silently reverted file is the failure the repair exists to prevent.
That is the session's judgment.
The moved-worktree case the review found is being fixed either way,
so this is about the causes that remain:
an index the wrapper cannot read or write.

### Should forced checkouts and resets refuse when they would replace an ignored file?

The discarding-commands branch hands a `switch` over an ignored file to Git,
which refuses.
Force overrides that:
the agent measured that `git checkout -f --no-overwrite-ignore other` still replaced `.env`,
and `git reset --hard` has no such option.
In a clean linked worktree those forms can still replace an ignored file the target tree carries.

The session will build option A unless you say otherwise,
because you wrote "never discard user data":
read the target tree and the ignored paths,
refuse naming the paths they share.
The agent estimates that adds 40 to 120 ms,
only for these forms and only in a linked worktree.
Option B is to accept it and document it.
Option C,
refusing whenever any ignored file exists,
would refuse every worktree with `node_modules`.
Ranking:
A > B > C,
because A protects the file at a small cost on a rare form,
B leaves the loss in place,
and C blocks routine work.

### `git clean -fdx` is now refused in a linked worktree that has ignored files

This is the same branch,
and agents will notice it.
`git clean -fdx` deletes ignored files,
so the guard now counts them,
and a worktree with `node_modules` is refused.
`git clean -nx` shows what would go.
It follows from "never discard user data",
and it changes a common way of getting a fresh build.
Write "allow it in linked worktrees" here if you want the old behaviour back.

### Should a commit add the manifests of dependents it bumps?

Your decision of 2026-09-15 and `SPEC.md` say opposite things.
`doc/decision/private-npm-registry.md:158-160` says:
"A cli-git commit policy bumps dependents by patch,
transitively,
in the same commit as a hand bump."
It rejects "a read-only check plus a `version:bump` command (two steps when forgotten)".
`doc/decision/cli-git-policies-platform.md:751-754` says the same.
`package/git-policy/cli/SPEC.md:207-208` says:
"A commit adds no paths of its own:
only the paths it selected reach the landing."

The code follows `SPEC.md`,
and an audit reproduced the result
(`doc/planning/cli-git-config-and-dependent-version-route-findings-on-main.md`,
finding 2).
A hand bump of `a` commits only `a`'s manifest,
and nothing says the dependents were left behind.
After that commit,
`check` and `fix` are silent too,
because the raise now equals `HEAD`.
So a published package can depend on an older version of a package it was meant to follow.

Recommended:
build what the decision says,
so a commit adds the dependents' manifests and updates them in the worktree when that copy still matches `HEAD`.
It is the only option where a forgotten step cannot leave a stale dependent,
at the price of a commit touching paths the person did not name.
The alternative keeps `SPEC.md`,
changes the decision,
and makes a commit warn loudly that dependents were left behind.
That is much less code,
and the person has one more step to remember.

### Should `add-explicit` judge a pathspec by its spelling or by what it selects?

The session picked the audit's recommendation,
extending the fixed list,
because it starts no process and is non-destructive.
A fix agent is queued for it.
Write "ask Git" here to change that.

The policy exists so `git add` names the paths it stages
(`package/git-policy/cli/src/native/command/rule_add_explicit.rs:1-20`).
It reads the arguments alone and compares each positional pathspec with four
byte strings,
`.`,
`./`,
`*` and `:/`
(`package/git-policy/cli/src/native/command/command_add.rs:113`).
Five spellings that Git resolves to the whole tree pass that list and staged
every file in a reproduction:
`:/*`,
`:/**`,
`:(top)`,
`**` and
`:(glob)**`
(`doc/planning/cli-git-policy-and-candidate-route-findings-on-main.md`,
finding 2).
`:(top)` is the one a person writes expecting a scope rather than a sweep.
A sixth,
`:(top)/*`,
fails inside Git 2.56.0 (`fatal: oops in prep_exclude`) and is reported as
`content-unavailable`,
which is Git's own failure and not this choice.

Recommended:
extend the list with those five and name them in the rejection text,
because the decision stays argument-only and starts no process before Git
does,
and the accidents this policy is about are the common spellings.
The alternative asks Git what the pathspec selects,
which catches every form including ones added to Git later,
at the cost of one Git start per `git add`
and a decision that no longer follows from the arguments.

### May a moved-path note be added to records in `doc/decision/`?

Rule `WR6` says a historical record keeps a moved identifier's original text and gains an in-place note with the current value and the move date.
Agents may not edit `doc/decision/` without your approval,
so these open issues are stuck on that alone:
4365,
5929,
6201,
3279,
5926,
5788,
3191,
3275,
5765,
5766,
6101 and 4357.
The note would add a line beside each stale path and change no decision.

Recommended:
yes,
for notes of that one kind only,
because the original words stay and the records stop pointing nowhere.
The alternative is to leave those issues open,
and each stays a dead reference.

### Which build profile should the uncovered crates follow? (issue 1641)

22 tracked `Cargo.toml` files declare a build profile,
and the rules that keep those profiles the same name 11 folders.
The other crates are uncovered,
and several say "Enforced by file-enforcer" in their own comments,
which is not true of them.
They are `cargo-shadow/cli`,
`codex-plugin/bash-limit`,
`git-policy/cli`,
`git-policy/cli.fuzz`,
`ide/app`,
`jsonc-edit/core-rust.fuzz`,
`linter/monochromatic-lint.fuzz`,
`mpm-shadow/cli`,
`pnpm-shadow/cli`,
`search-shadow/cli`,
`wg/cli-quicker-exempt` and `xml-edit/core-rust.fuzz`.

Recommended:
add one rule for the shape the shadow crates share (`panic = "abort"`) and cover the rest with the existing rules,
so the comments become true.
The alternative is to delete the claims and leave these crates free to drift,
which is less work and protects nothing.

### Should upstream forks keep their own MIT text? (issue 1645)

The rule that writes license files knows no MIT text,
so the MIT files of the packages under `upstream-fork/` are neither written nor removed by it.
All carry the same copyright line,
naming Sindre Sorhus.

Recommended:
keep one text per package and teach the rule to seed it,
because a shared text would put that copyright on any other MIT project added later.
The alternative,
one shared root `LICENSES/MIT.txt`,
is simpler and is only correct while every MIT fork has the same author.

### Should 30 packages declare a license? (issue 2994)

30 `package/*/*/package.json` files have no `license` field,
so no license text is due for them and none ships.
This is a legal choice about your code,
and nothing about it can be measured.
Name a license,
or say these packages stay undeclared.

### Who owns the repository links in `package.json`? (issue 1124)

221 `package.json` files still link to `github.com/Aquaticat/Monochromatic`,
and 3 link to the current organization.
No rule owns those fields since the TypeScript tool went away,
so they drift.

Recommended:
add a file-enforcer rule for `repository` and `homepage`,
matching what it already does for `Cargo.toml`,
so one run corrects them all and they stay correct.
The alternative is a single rewrite with nothing to keep it true.

### Are the three findings left on the stat-data landing worth another round?

The landing that keeps stat data went through six rounds of cross-vendor review and is on main.
The sixth review (`${HOME}/temp/agent/reviews/landing-stat-data-round-6-sol.txt`) named three more cases,
and the coordinator landed it with them open because each needs something unusual:

- Another process swaps two directories by rename in the instant between opening one and reading its `.gitattributes`.
- Another process holds `.gitattributes` in a shared memory mapping and changes it during the read,
  which moves no timestamp.
- A FUSE filesystem leaves a read of a regular file unanswered.
  That one is the open question about a stalled regular-file read in this file,
  which also covers every other read of a regular file.

In the first two,
a file can be recorded as clean when its contents differ.
Each is reachable only by another process acting in a window of microseconds.
Say if any of them should get another round.

### Conversion: is a late change of mode or owner a defect or accepted risk?

An audit (`doc/planning/cli-git-subvolume-and-conversion-route-findings-on-main.md`) found that a `chmod` made in a conversion's last window is lost,
because the comparison that decides the old tree can be deleted does not look at mode,
owner or extended attributes.
The coordinator read this as a defect and queued a fix,
since that comparison exists to catch a late change.
Say if you would rather accept the risk than pay for the wider comparison.

## Open: decisions a read-only audit of all recorded work found, recorded 2026-11-08T16:05:49-05:00

None of these has been asked yet,
and none is holding up running work.
Each record is cited so the context is one read away.

- Does the 2026-11-04 pause still cover the commit protocol that makes commits unable to fail by construction?
  Your words then:
  "aside from layout + linting fixes,
  pause everything else - but let work already in flight run to their natural
  conclusions".
  Other wrapper work has resumed since,
  but `feat/git-policy-cli-commit-protocol-models` and `fix/git-policy-cli-commits-never-fail` have not moved.
  This matters because an agent's scratch commit stopped at its ceiling under load on 2026-11-08,
  and an identical retry succeeded,
  against your "Commits should never fail."
- Should the Claude Code guardrail be registered in `~/.claude/settings.json`?
  A real-Git commit from the coordinator's session went through on 2026-11-08,
  because `~/.claude/settings.json` does not name the guardrail;
  fed directly,
  the hook refuses it.
  Pi agents are covered.
  Details:
  `doc/handover/pi-agents-blocked-from-real-git.md`.
- Pi agents started through the driver no longer inherit a credential-named variable (`9ae27d221e`).
  Three choices follow from it.
  Should interactive Pi sessions get the same rule as a plugin?
  Should mise get its own GitHub token home (`github.credential_command` or `mise token github`),
  since a subagent's `mise` now runs anonymously at 60 requests an hour instead of 4,875?
  Should `SSH_AUTH_SOCK` be allowed through for `ssh m1` probes?
  Nothing has proved it necessary.
- `doc/decision/cli-git-concurrent-commits.md` (line 158 and lines 190 to 205) still says an `index.lock` owner proven alive gets an unbounded wait.
  The shipped code bounds that wait at one minute for a merge and at the service hold ceiling for the service,
  following the owner's later "nothing should ever wait forever or indefinitely".
  May the decision record be amended to say so?
- `LOCK_CEILING` (600 s) no longer bounds any wait;
  it only feeds `FOREIGN_OWNER_PATIENCE` (1,200 s).
  The longest lock wait in 4,386 landings that ended by release was 11.49 s.
  Both values are kept as they were;
  lowering the patience changes when another namespace's record counts as abandoned,
  so that stays open.
- Should `AGENTS.md` gain this rule?
  "A control's verdict comes from the order of events:
  release its fixture from the wait's own line,
  never from a sleep whose passing only the host's speed decides."
  The same defect was fixed twice in the wrapper's index-lock tests (`2b33c7bd32` and `35cd5ae7e5`).
  The code the proposing agent suggested,
  `TSX`,
  reads as the file extension,
  so the rule needs a different one.
- May `cargo-shadow lean` remove a run record that a process of another user could be holding,
  when that process cannot be inspected?
  Refusing stops the sweep on this host.
  Do the `container-`,
  `windows-` and `test-` record prefixes belong in its list?
  And what should happen to the eighteen logs redirected by hand,
  and the one renamed 17.9 MB record,
  in the linter's verification folder?
- Fuzzing the wrapper:
  is a cheap native control over the checked-in seeds wanted,
  and may the artifact reference stand without a landed evidence file?
- The unmerged copy-in-service branch:
  the audit in `doc/planning/cli-git-copy-route-findings-on-main.md` ranks B over C over A.
  The coordinator picked option B,
  with the veto open:
  port three narrow fixes into `main` and leave the branch as it is,
  neither merged nor deleted.
  The reason is that a kill during a worktree add on the copy route silently loses that worktree's ignored files,
  and recovery can fill a registration the command did not create.
  Two agents are building it.
  Is a repository shared by two hosts supported?
  If it is,
  a foreign host's process reads as dead to the copy,
  which the audit could not settle alone.
  A new `git worktree add` from a worktree still being filled now gets no ignored-state copy and one line saying so.
  The only remedy today is to remove that worktree and add it again,
  because Git refuses an existing path and the wrapper has no way to copy ignored state on demand.
  Should the wrapper gain a command that fills an existing worktree?
  Two gaps stay open:
  a copy still staging records no destination,
  so it is not detected until the record written before Git is on `main`;
  and the snapshot and clone routes can still carry a half-filled set from a busy source.
- The `search-shadow` refusal of `rg` with a piped standard input and a named path is on `main` and not yet released.
  It refuses on the kind of standard input alone,
  so a program that starts `rg` through `PATH` with a pipe it never writes to is refused too,
  and `SEARCH_SHADOW_ALLOW_WIDE=1` does not lift it.
  Pi's `grep` tool and this repository's own callers are not affected:
  they give `rg` an ignored or null standard input,
  or pass `--files`.
  Should that override apply to this refusal before the release?
- Issue 3474 also asked whether an unbounded `rg` over a large tree that is not a repository,
  such as `~/temp/agent`,
  should be limited too:
  by a cap on the shadow's own output,
  or by treating known scratch roots like a repository root.
  Neither is built.
  Is either wanted?
- `package/hall-monitor/daemon` fails its own `lint:oxlint` on `test-import(require-eventual-artifact)`,
  two findings before and four now that it has the tests issue 1297 asked for.
  The rule needs an import from a built artifact,
  and that package has none.
  Should it gain a `dist` build,
  or an exemption?
- The btrfs host run (`native:test:host:btrfs`) runs only when someone starts it.
  Should a systemd timer run it,
  and should the gate count the `BTRFS CONTROL NOT RUN` lines so a skipped control fails?
- `mpm-shadow` launcher records named a folder carrying the version,
  so any newer install left them dangling (299 of 1051 records at 2026-11-08T18:17:00-05:00).
  The fix on `fix/mpm-shadow-launcher-records-and-lock-warning` records a path that names no version.
  The records already dangling heal on the launcher's next run,
  which is the coordinator's pick with the veto open;
  the alternative is an `mpm install` sweep over every worktree after the release.
  The newest-installed launcher pick has no `doc/decision/` record yet.
- The rule codes `MXR`,
  `RDC`,
  `GCW` and `WXG` are retired but not reserved.
  Rule `CRN` puts them in `forbidden-strings.append.local.txt`,
  which is gitignored and is the owner's file;
  that reservation is the last unmet line of issue 1720.
- The umbrella list (`225a3b6318`) has three open points left:
  which of its disagreeing maintenance mutation records is right,
  whether to relink the 16,177 dangling links it counted in 172 of 239 readable worktrees,
  and whether its release-artifact gate also meant a check of the archive's own contents.
  Its Windows and release-gate entries are split in
  `doc/handover/cli-git-rust-implementation.md`,
  where the gate's latency half is answered:
  `native:perf`,
  run against the installed binary,
  measures what mpm installs from the release tarball,
  its limits are its own task description,
  and the release records of `doc/planning/one-release-each-batch-2026-11.md` hold its figures.
  A search of `doc/` and `.github/` on 2026-11-08 for `release-artifact` and `release performance`
  found only items forwarding to that list.
- The questions that end `doc/planning/cli-git-shipped-limits-audit.md`,
  which ask what each shipped limit should cite.
- Does "never discard user data" cover the audit history?
  The timing logs and the push trigger log keep one older generation,
  so a rotation drops the one before it.
- Should the unmerged branch `worktree-agent-ab3dd6294f9d91284` be kept or dropped,
  and should the lock-based copy modules main still ships be reviewed?
  The branch holds open review findings and fixes that exist nowhere else.
- Should `package-paused/dev-script/inference-canary`,
  `package-paused/webapp-edu/paper2vn` and the wrapper's own test runners get the container limits?
- Should the mount unit's stop block guard its `luksClose`?
  A stop with no mapper now fails where the old stop hid the failure
  (`doc/handover/repository-move-dangling-link-items.md`).
- May the stale detached worktrees under `.claude/worktrees/` be removed?
- May the dead `cli-git@` units of the old name and the 0.5.34 install folder be removed?
- Should there be an operator override that forces the native Git route when the commit service misbehaves?
- Rotate `MISE_GITHUB_TOKEN`?
  A second exposure on 2026-11-08:
  the agent `tokio-step-t12` read `/proc/<pid>/environ` of a cargo process to find its build directory,
  and that printed the same variable into its session,
  named by `sessionStats.sessionFile` in `${HOME}/temp/agent/pi-runs/tokio-step-t12-1/summary.json`.
  That run started at 2026-11-08T15:03:15-05:00,
  before the driver began removing credential-named variables (`9ae27d221e`,
  2026-11-08T18:09:11-05:00).
  A process that another session started still carries them,
  so the rule against reading another process's environment stays in force.
  A Pi agent ran `env | rg` on 2026-11-08 and its output held the token's value.
  The value is in that agent's session file,
  named by `sessionStats.sessionFile` in `${HOME}/temp/agent/pi-runs/wrapper-small-robustness-items-1/summary.json`,
  and tool output goes to the model provider.
  The count of matches was read without printing the value.
  Every running agent has since been told never to print the environment,
  and `pi-as-subagent` now requires that sentence in every prompt.
- May the root `vm-builder` run replace the existing VM?
  `mise run //package/vm-builder/cli:run` overwrites `/var/lib/libvirt/images/monochromatic-dev.qcow2`,
  dated 2026-03-31,
  and force-stops and undefines any domain of that name.
  Your authorization of `sudo` did not name that,
  so only the build half (`image:build`) was run.
- How long should `FOREIGN_OWNER_PATIENCE` be?
  It ships at twenty minutes,
  stated as twice the longest a merge waits,
  but a merge has waited at most one minute since 2026-11-07,
  so that basis would give two
  (`doc/planning/cli-git-shipped-limits-audit.md`).

Checked and not asked,
because you have already answered them:
deleting the merged branches with nondescript names (done with your approval),
and bumping and publishing ("I'm authorizing you to bump and publish any updates this session.").
The two library layout questions stay at your "Not yet.".

## Answered in the question tool, recorded 2026-11-09T01:14:41-05:00

An audit (`doc/planning/cli-git-discarding-commands-route-findings-on-main.md`) reproduced two discarding commands outside the wrapper's recorded rule.

- `git worktree remove`:
  "Log only",
  with the note
  "This one is pretty intentional because agents are just as likely to run a rm rf,
  so preventing git worktree remove buys us nothing."
  Git runs as it does today,
  and the wrapper adds an audit line naming the worktree and what the removal took.
- `git branch -D` from the main worktree:
  "Allow,
  add audit line (Recommended)".
  Git runs as it does today,
  and the wrapper records the branch and its commit so the branch can be restored.

## Answered in a plain message, recorded 2026-11-09T00:52:14-05:00

- How many Pi agents run at once:
  "I said 16 is practical,
  based on a rough estimation.
  Try 20,
  and if it doesn't blow up (in terms of api rate limits),
  we're permanently raising the practical limit to 20."
  The trial is running.
  A provider error reaches a run's `feed.jsonl` as an `assistant` record with `"stop":"error"`,
  and Pi's own retry as an `auto_retry_start` record.
  Before the trial,
  at up to sixteen agents,
  158 feeds held 25,631 model turns,
  no retry and no error shaped like a rate limit.
  The eight errors they did hold (five aborted operations,
  three unconfigured providers) show the count can see an error.
  Coordinator's pick,
  veto open:
  the trial passes after sixty minutes with twenty running and no error shaped like a rate limit.
  The first such error ends it.
  A provider that throttles by answering slowly reports no error,
  so the watcher also prints the median model turn before and during the trial.
  The count is `${HOME}/temp/agent/pi-runs/watch-trial-of-twenty-pi-agents.ts`.
  On a pass,
  `.agents/skills/pi-as-subagent/SKILL.md` gets these words in place of "In practice it's 16."
  and `file-enforcer write` regenerates its copies.

## Answered in a plain message, recorded 2026-11-09T00:07:24-05:00

- Kopia and this repository:
  "Make kopia exclude the entire Monochromatic dir."
  Applied as `kopia policy set user@bazzite:/home/user/Monochromatic --manual '--add-ignore=*'`.
  The hourly `kopia snapshot create --all` skips a source set to manual,
  and an explicit snapshot of it stores nothing,
  dot-folders included;
  both were tried on a throwaway repository with an unchanged source as the control.
  Existing snapshots stay.
  The last full one is `kf33ae88159bbdf201f5735e37d21d1c8`,
  taken 2026-11-08T23:57:49-05:00.
  To undo:
  `kopia policy set user@bazzite:/home/user/Monochromatic --no-manual '--remove-ignore=*'`.

## Answered in plain messages, recorded 2026-11-08T23:47:13-05:00

- Stale folders and the handover:
  "Delete dirs that shouldn't be there anymore and update handover now."
- Cleanup from now on:
  "Cleanup whenever it makes sense on your own."
  and then,
  about the detached worktrees the handover held for the owner's word:
  "It covers deleting stale detached worktrees."
  The coordinator now removes worktrees and leftovers without asking when nothing would be lost:
  a worktree goes only if its commits are on a branch and it has no uncommitted work.
  Branches stay,
  because branch deletion was not part of these messages,
  and the old tree kept for the 860 rollback stays for the owner.

## Answered in the question tool, recorded 2026-11-08T23:24:03-05:00

- The ceiling of the signed scratch `git commit` step,
  asked again after 886 samples showed 12 steps past the 3 s ceiling (1.35%),
  10 of those commits not landing:
  the owner chose "About 16 seconds"
  (three times the slowest ordinary step,
  5,444 ms)
  over the recommended "Stop only when idle",
  and added:
  "About 16s,
  and investigate what happened for those 3 commits.
  If we can't figure it out,
  then we have a defect and should log more."
  The three are the steps of 26,256,
  36,651 and 171,383 ms.
  This replaces the 2026-11-08 answer "About 3 s now".
- Where the overdue-agent check lives:
  "Move into the driver (Recommended)",
  so it goes into `package/pi-shared/subagent-driver`.

## Answered in a plain message, recorded 2026-11-08T23:10:25-05:00

- How to watch running subagents:
  "Regularly,
  check in on subagents that should have finished their jobs (based on time estimations 90%+ percentile) but haven't returned."
  The coordinator now runs a scratch watcher,
  `report-overdue-pi-runs.ts` under `${HOME}/temp/agent/pi-runs`,
  which ends as soon as a running Pi agent has passed the 90th percentile of finished runs of its kind
  (85 minutes over all runs,
  55 for audits,
  233 for tokio steps,
  recomputed on each look),
  and the coordinator then reads what that agent is running.
  Its first look found agents working normally past that time,
  and two agents blocked for most of an hour in their own probes,
  which waited without a deadline.
  Those two probes were ended by process number and container ID,
  and each agent was told why.

## Answered in plain messages during the 860 cut-over, recorded 2026-11-08T12:43:36-05:00

- How subagents are used from now on,
  sent 2026-11-08T07:27:21-05:00:
  "When you reach a point where you'd use subagents,
  do not continue.
  Pause there and I'll restart Claude Code again and we'll discuss how to use subagents from then on."
  The coordinator starts no subagent until that discussion has happened.
  The move needs none;
  the first likely stopping point is the Git wrapper work queued after the move.
- The hourly backup during the cut-over:
  "Sorry I forgot to quit the 2nd Claude Code session.
  You can temporarily disable the kopia job if you need to."
  The coordinator stopped `kopia-snapshot-all.timer` at 2026-11-08T07:02-05:00
  and started it again at 2026-11-08T12:43:36-05:00.
- Where a rule that only concerns Claude Code native agents lives:
  "Holdup.
  Is MWK really about Claude Code native agents?
  If not,
  it shouldn't have been changed.
  If so,
  it should only live in CLAUDE.md preamble."
  `MWK` concerns Claude Code's `Monitor` tool,
  its wakeups and its subagent notifications
  (commits `80588d6aa6` and `8873aae967`),
  so it moved to `.config/file-enforcer/claude-preamble.md`.
  `pi --help` names no wakeup,
  monitor,
  cron or schedule option.
  Then,
  on `WUC` (`ScheduleWakeup`,
  `CronDelete`) and `OPA` (`AskUserQuestion`):
  "Move WUC and keep OPA."
  `WUC` moved to the same preamble and `OPA` stays in `AGENTS.md`.
- How Pi subagents commit,
  answered after the coordinator explained the read-only `git` shim:
  "We have to be happy with Pi committing directly on main.
  We are not building another git wrapper or shim when we're already so busy with one.
  If it turns out this blows up we'll consider enhancing our own git wrapper."
- Where Pi's driver lives:
  "And please promote the driver into a proper repo pkg or task or something."
- How far to trust a model by its price:
  "Expensive models are also not necessarily more trustworthy than cheap models."
- Running Pi for real work without the driver:
  "Forget about --print on a real coding task."
- Which subagents this session uses,
  answering the coordinator's open question on how to use subagents:
  "Drive pi as subagents exclusively from now on for this session."
- How many Pi subagents can run at once:
  "Pi doesn't have a limit to number of concurrent subagents.
  In practice it's 16."
- Where Pi subagents work,
  correcting the coordinator's reading that they had to work in the main checkout:
  "Feel free to use worktrees.
  I said "be okay with pi committing directly to main",
  not that they have to."
- How to stop the formatter re-wrapping the vendored Slint license,
  whose bytes must match a published checksum
  (options:
  rename it to `.txt`,
  or turn the rewrap rule off for `package/music-player/desktop-app/notices/**`):
  "Rule exception for notices/".

## Answered in plain messages and the question tool, recorded 2026-11-08T06:45:53-05:00

- Who checks the IDE at the cut-over stage `c28-ide-through-link`:
  "I believe you can do the IDE check yourself?",
  then "The IDE I use is obviously our own IDE."
  and "And using the nested compositor is fine."
  The coordinator runs `package/ide/app` in a nested compositor against the repository through the link,
  confirms the file tree and one search,
  then passes the assertion.
- A restart of Claude Code:
  "Once you finish the move,
  prepare for a restart of Claude Code,
  and tell me whenever it's a good time to restart.
  I need to restart Claude Code."
- When to restart,
  after the agent found that the cut-over's holder scan counts this session
  (its working folder is the repository):
  "Restart now,
  outside repo (Recommended)".
  The owner starts the session again from `/var/home/user` with `--resume`,
  `--add-dir /var/home/user/Monochromatic` and `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`
  (tested on a throwaway folder:
  the added folder's `CLAUDE.md` loads with the variable and not without it).
- The other processes whose working folder was in the repository
  (terminal shells,
  Zed,
  a Codex session with three Codex app-server daemons,
  a DeepSeek `dsh` session with its search-fetch server,
  the `adb` server):
  "Stop agents and adb for me".
  The agent sent TERM to those seven processes by PID after re-reading each command line,
  and all seven exited;
  the owner closes Zed and the terminals.

## Answered in the question tool, recorded 2026-11-07T22:19:43-05:00

- Which `index.lock` files the commit service may remove,
  after the agent measured that a native `git commit -a`,
  `--only` or `git commit <path>` keeps a lock with the whole index in it while its editor is open,
  with no open descriptor and no PID file:
  "Empty locks only (Recommended)":
  only an empty lock that no readable process holds and no PID file names is removed;
  a lock with content is waited for to the ceiling,
  then refused with a message naming it
  (built in `373ea2efdc`).
- The ceiling of the signed scratch `git commit` step when the message came with the command:
  "About 3 s now"
  (three times the slowest of 14 per-step records,
  1,002 ms),
  chosen over the recommended 21 minutes with the con stated:
  on 2026-11-07's data it would have failed commits whose build took 11 to 406 s.
  The editor case keeps no ceiling.

- Whether a stop of the commit service may cut a running branch write
  (the 2026-11-04 answer "Never cut a branch write" against the 2026-11-07 answer "C:
  shutdown stops its Gits"):
  "B:
  let a branch write finish (Recommended)":
  a stop ends every other Git at once,
  but waits for a running branch write up to the stop limit;
  systemd signals only the service (`KillMode=mixed`) and kills what is left only at that limit.
  Measured with the question:
  the live unit had `KillMode=control-group` and `TimeoutStopUSec=15s`
  (`/etc/systemd/user.conf.d/timeout.conf`),
  below T11's `SETTLE_CEILING` (24.096 s) and `LAST_RESORT` (29.096 s).
  Coordinator pick,
  veto open:
  the unit the wrapper writes sets its own `TimeoutStopSec`,
  derived in code above `LAST_RESORT`.

- When the repository move to the 860 runs:
  "Back to back,
  after agents (Recommended)":
  once the agents running at 2026-11-07T23:30-05:00 finish,
  the coordinator starts no new agents,
  then runs the first copy and the cutover in one go,
  and tells the owner when it reaches the IDE step (`c28-ide-through-link`).
  State at the answer:
  prepare passed through the print pass (`repository-move-prepare-13`:
  inventory agrees,
  270 nested subvolumes;
  272 snapshots and 272 sends planned);
  the space rule passed (860 925 GiB free,
  NVMe 284 GiB);
  the inventory is redone (`--redo=r0-inventory --redo=r0-reconcile`) right before the first copy,
  because agents add and remove worktrees.

- How the repository linter stays at zero
  (`doc/planning/repository-linter-stays-at-zero.md`):
  "None for now.
  Focus on the git wrapper.
  Open a gh issue about this instead."
  Nothing is built;
  the issue carries the options.
- The linter's own fuzz seeds:
  "Ignore the seed folder (Recommended)":
  one ignore entry for exactly `package/linter/monochromatic-lint.fuzz/seed/`.
- Lean build folders (`doc/planning/lean-build-dirs.md`):
  "A to F,
  B allowed (Recommended)":
  all six mechanisms in order,
  in cargo-shadow;
  the decision record `doc/decision/cargo-build-dir-host-cache.md` is amended so the sweep (B) may read Cargo's fingerprint files,
  for that sweep only;
  the tools' first run under their own locks cleans what exists today;
  grace periods and size caps are measured and recorded as coordinator picks with the owner's veto open.
- What else the cleanup covers:
  "Rust toolchains (Recommended)" and "Podman images,
  volumes (Recommended)":
  nightlies no checkout's lock pins,
  and images and volumes no container uses and no current image tag of the repository names,
  each after the grace period.
- Not asked yet:
  where a blank at the end of a row goes in the IDE.
  On reading the tests,
  `a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row` fails at its own guard
  ("no fixture produced an empty tail,
  so the test proves nothing about it"),
  because Parley 0.12 no longer adds the empty row the test was written to rule out;
  the failing assertion of `the_fixtures_wrap_as_described` is not recorded.
  The question is to be re-framed from a run of both tests before it is asked.

- A closure-check skip in the repository's own lint:
  "Fetch then warn."
  (the owner's own words,
  not one of the offered options):
  the repository's lint tasks fetch Rust dependencies first,
  and a skip that remains is reported as a warning naming the package and the command that fixes it.
- Fenced Rust examples inside Markdown:
  "B:
  exempt only \"Bad\" examples (Recommended)":
  a marker on those fences exempts them from `rust/require-rustdoc`,
  and the other 176 are documented by hand.
- The decision records' line breaks:
  "Allow whitespace-only reflow (Recommended)":
  the fixer runs over `doc/decision/` once,
  with a word-level diff proving only whitespace changed.
- Container disk weight and limits for the remaining users:
  "All of them (Recommended)":
  every container start gets the lowest disk weight and limits sized to its job,
  through one shared constant,
  including `vm-builder`'s rootful run.
  Done on `feat/container-limits-everywhere`;
  the inventory,
  the evidence and the open items (the rootful run needs one root try,
  the workflow flags are unproven on
  a hosted runner) are in `doc/handover/container-limits-everywhere.md`.

- Porting the wrapper package's 145 TypeScript files to Rust
  (`doc/planning/git-policy-cli-typescript-to-rust.md`):
  "After the layout move (Recommended)":
  the pause lifts once the layout move's steps 2 to 6 land,
  with the plan's recommended option for each of its eleven choices.
- Answering a pull's Git queries in process
  (128 ms against native's 12 ms on a fast-forwarding pull):
  "Not now (Recommended)".

- How long a commit's signed `git commit` step may run:
  "A:
  limit unless editor (Recommended)":
  when the message came with the command (`-m`,
  `-F`,
  `-C`),
  a ceiling of three times the slowest measured signing time;
  with an editor open,
  no ceiling,
  only the waiting notice.
- Timing a signed commit with the owner's real signing setup:
  "Allow the probe"
  (throwaway commits in a throwaway repository,
  durations printed only,
  native `git commit` as the control).
  Result (coordinator,
  native `/usr/bin/git`,
  display and askpass variables removed):
  signed 16,
  13,
  13 ms;
  unsigned 9,
  9,
  8 ms;
  signed with no agent socket 27,
  10,
  11 ms.
  Signing costs about 5 ms and needs no agent,
  passphrase or display,
  so the 20 to 406 s build steps were something else in the scratch `git commit` step;
  the per-step timing records of release 0.6.1-alpha.2 name it on the next slow commit.
  Two wider probes (the key's passphrase state and the commit service's environment) were refused by the auto-mode check as credential exploration and were not retried.
- An `index.lock` no process holds:
  "Service removes it":
  after a complete scan of every readable process finds no holder and there is no Git PID file,
  the service removes the lock and lands the commit.
  The limit stated with the question stands:
  processes in other user namespaces and a native `git commit --only` in its hook phase cannot be seen.
- What a stop of the commit service does to a commit's Git
  (tokio plan,
  open choice 12):
  "C:
  shutdown stops its Gits (Recommended)":
  the stop handler only records the stop,
  and the service's own shutdown signals the Git processes it is waiting on,
  through a registry.

## Answered in the question tool, recorded 2026-11-07T21:28:26-05:00

- Old agent scratch on the 860:
  "Delete old scratch (Recommended)",
  option A in the record kept below.
- The refused `monochromatic-lint` 0.5.2-alpha.6 release:
  "Allow release tasks (Recommended)",
  so agents run this repository's release tasks without asking.
  Done as an auto-mode allow entry in `~/.claude/settings.json`
  after the owner wrote "I authorize you to edit Claude Code Settings and especially auto mode settings.";
  the release was dispatched as run 42438318626.
- The repository move's W6 and W7 edits to `~/.claude/settings.json`,
  `~/.claude.json` and `~/.codex/config.toml`
  (values in `doc/handover/repository-move-dangling-link-items.md`,
  section "W6 and W7 edits to apply"):
  "I apply them at cutover (Recommended)",
  so the coordinator makes them right before the cutover.
- The coordinator's picks made while the owner was away
  (tokio worker cap,
  the strict memory test,
  the full repository link check):
  "Keep all (Recommended)".
- In plain messages,
  the owner asked:
  "Modify the auto mode config in Claude Code global settings to reduce false denials."

### Record of the scratch question as asked

The move's space rule refused on 2026-11-07,
as designed
(nothing copied):
estimate 389 GiB
(nested checkout 303 GiB with sharing counted once,
external worktrees 74 GiB,
clone-limit allowance 12 GiB),
487 GiB with the plan's 25 percent margin,
against 493 GiB free on the 860,
which would leave 6 GiB where the rule requires 150 GiB.
The 860 had 662G free when the plan was measured.
Agents' scratch folder `~/temp/agent` is a link to `/var/mnt/encrypted/temp/agent`,
which is on the 860,
and holds 22,242 entries;
the 25 largest alone measure about 1,113 GiB with `du`
(which counts reflinked data once per file,
so the space freed may be less),
for example `agent-notify` 252 GiB (2026-10-11),
`pi-mimo-preparation-failure-20261018` 102 GiB,
`tr-fan` 89 GiB,
and about twenty `ide-*` and review build folders of 19 to 50 GiB from October and July.
- A (coordinator's recommendation):
  delete scratch entries last changed before 2026-10-31
  that no tracked file in the repository names,
  listing every entry and its size first
  (the list kept as a file),
  then deleting exactly those;
  rerun the space rule.
- B:
  you review that list before anything is deleted.
- C:
  delete nothing;
  shrink the move instead
  (the plan's scope four:
  main plus a chosen subset of worktrees,
  or leave the external worktrees out),
  or lower the 150 GiB floor or the margin.
Separately,
scratch on the 860 means agents write to the drive the repository moves onto;
after the move both live there.

## Told, not a question: a wrapper mutation campaign ran against a recorded pause (2026-11-07)

Mutation campaigns over the wrapper were paused on 2026-11-04 after you asked
"We're not close to doing mutation runs though,
right?
Don't we still have a lot of things to fix before doing them?",
to resume after the commit protocol is built
(`doc/handover/cli-git-rust-implementation.md` lines 1725 to 1747).
The coordinator missed that record and ran one over the 21 source files the tokio merges changed
(label `tokio-062`,
main at `40ec469247`).
It was let finish rather than discarded;
its results are recorded but not acted on,
and nothing waits on it,
alpha.2 included.
No further wrapper campaign starts until you lift the pause.

## Coordinator picks made 2026-11-07 with your veto open (not asked yet)

Each is built and in main;
saying nothing keeps it.
- How the 860 move copies each subvolume:
  every first-pass copy is sent against one empty read-only snapshot,
  with every earlier copy as a clone source,
  and only the final pass sends a subvolume as an increment on its own first-pass snapshot.
  The plan it replaced sent a worktree as an increment on a snapshot of `main`,
  which the kernel's send code can pair with the wrong file
  (it treats two inodes as one when their number and creating transaction match,
  `fs/btrfs/send.c` `changed_inode`);
  on the throwaway fixture that plan failed,
  and the new one copied every file exactly and kept sharing.
  Record:
  `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`
  and `doc/planning/repository-on-the-860-drive.md`.
- `monochromatic-lint --fix` now applies only whitespace fixes,
  and content-changing fixes need `--fix-content`
  (0.5.2-alpha.5);
  the repository's `format:monochromatic-lint` task therefore changes no word.
- When the fixer breaks a line inside a blockquote,
  the new line carries no `>`
  (a lazy continuation that renders the same under CommonMark 0.31.2,
  section 5.1,
  rule 2),
  so the fixer never adds a character to your text.
- After the repository move,
  the wrapper checks on every Git command that the repository link still leads to the 860
  (W2,
  merged `b842b34fff`),
  so a dangling link gives a plain message instead of creating things in the wrong place.
  Measured cost:
  about 1.0 ms per forwarded command (median,
  `status`;
  0.8 ms for `rev-parse`),
  beyond the same-binary spread;
  nothing before the move.
  Cheaper forms (skip the check for read-only commands,
  or trust a cached answer across commands) would let a dangling link go unnoticed,
  so the full check stays.
- Tokio step T10
  (the commit service's connections,
  claim renewal and syncs become tasks;
  gate 5313 of 5314,
  not merged yet)
  raised one idle service from 4 threads to 27,
  because the runtime starts one worker per CPU.
  Many services run at once,
  so the coordinator picked a cap on workers,
  measured at 2,
  4 and 8,
  taking the smallest within the measurement spread and never below 2
  (your ruling "Full on tokio.
  Multi-thread.").
- T10 also made the service-memory fault test find no limit at which the service starts but a commit cannot land
  (the service now reserves room for a commit before it starts).
  The agent recommended accepting that,
  which would leave the test testing nothing;
  the coordinator picked instead lowering the running service's address-space limit after it starts,
  so the test still reaches exhaustion in the middle of a landing and checks that the repository recovers.
- The claim-renewal stop still waits without a limit
  (as before T10),
  against your rule that nothing waits forever;
  the coordinator picked a measured ceiling,
  with the claim expiring by its lease if that ceiling passes.
- An equivalent mutant that no test can kill
  (`workspace_count -> 0` in `package/linter/monochromatic-lint/src/rust_file_engine_syntax.rs`,
  a function that already returns 0)
  stays recorded as equivalent,
  so the linter's `--syntax-only` mutation campaign reports one miss every run
  until `mutation:coverage` learns to skip recorded equivalent mutants
  (record:
  `doc/handover/monochromatic-lint-layout-and-workflow-mutants.md`).
- The wrapper's own `markdown/autofix` commit policy keeps applying its one content fix
  (the LFS image link rewrite),
  since that rewrite is the policy's whole purpose.

## Open: lean build folders, after the first run (not asked yet)

Built and run 2026-11-07 to 2026-11-08 (cargo-shadow 0.1.5-alpha.7;
details and per-removal proof in `doc/handover/cargo-shadow-lean-build-folders.md`).
The first run freed 128.1 GiB on the 860,
retired 12 of 17 dated nightlies (52.95 GB to 22.47 GB)
and removed 251 Podman images and 27 volumes;
the daily timer runs at 04:30 and never at login.
- Question:
  should images held only by exited containers be removed too?
  Today a container in any state keeps its image.
- Coordinator picks,
  veto open:
  - an old build tree of a live worktree
    (the folder Cargo used before the toolchain-keyed folders existed)
    is retired after the 7-day idle period once a keyed folder exists for the same workspace;
    without this nothing ever removes those trees,
    and most of the 6,884 trees the first run kept are these;
  - the size caps (E) read the same fingerprint fields as the sweep (B);
    your amendment said "for that sweep only",
    and the agent extended the reader to the caps because they evict by the same units;
  - grace 7 days,
    unidentified trees 21 days,
    run records 14 days and 2 GiB per folder,
    caps 12 GiB in a checkout and 24 GiB in the shared build folder,
    each measured (`doc/planning/lean-build-dirs.md`).
- The timer's pass order and Podman's three refusals are fixed in the sources (merge `a5b31ab7de`);
  the facts are in `doc/handover/cargo-shadow-lean-build-folders.md`,
  and they reach the timer with the next `cargo-shadow` release.

## Open: where a blank at the end of a row goes in the IDE (not asked yet; the owner asked to focus on the Git wrapper)

The text-layout library the IDE uses (Parley) moved to 0.12 in the daily update,
and it now follows the CSS Text 4 rule for blanks at a line end:
an overflowing blank "hangs" past the edge instead of staying on the row and pushing what follows down.
Two IDE tests describe the old behavior and now fail
(`ide::wrap_keys a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row`,
`ide::wrap_keys the_fixtures_wrap_as_described`);
their fixture text says "an overflowing blank stays on the upper row,
so a tab right after it starts the next row".
Measured at width 240,
scale 1:
a continuation row now starts with a tab (rows `0..28`,
`28..38`),
and no fixture produced an empty tail.
Source:
`parley-0.12.0/src/layout/whitespace.rs` (header doc),
merge `d2598b2842`.
The choice:
keep the IDE's old rule (our code moves the blank back)
or adopt the CSS rule (the tests change).

## Answered 2026-11-07T13:52:06-05:00 to 13:57:11-05:00 (question tool, then plain messages)

- Enforcing "agents never run sudo" after an agent ran `sudo -n true`
  (options:
  guardrail hook plus AGENTS.md rule,
  AGENTS.md rule only,
  leave as is):
  "Leave as is".
- Rule SCR (no bash scripts) against the wizard skill's bash root scripts:
  "Wizard writes TypeScript,
  and you just do the move yourself,
  no need to delegate to me."
  The 860 move is run by the coordinator,
  not handed to the owner;
  its runner becomes TypeScript (sent to the preparation agent).
  The coordinator read that as a request to convert the wizard skill to TypeScript and queued the conversion.
  The queued item is withdrawn,
  on the owner's later words:
  "No,
  why would there be a wizard skill at all?"
  Nothing was built.
  The skill was third-party
  (`mattpocock/skills`,
  in `~/.agents/skills/wizard` since 2026-08-18,
  outside this repository).
  The owner then:
  "Remove and Blocklist the wizard skill."
  and
  "But do keep matt's other skills."
  Done:
  `skills remove wizard --global` took out the install and its lock entry,
  leaving the pack's other 31 lock entries;
  the dead link in `~/.pi/agent/skills` and the local copy under `~/.claude/skills/synced` are deleted;
  Claude Code's user setting `skillOverrides` holds `wizard: off`
  (the `Skill` tool then refuses it);
  Pi's user setting `skills` holds `!**/wizard/**`
  (real Pi offers 27 skills before and after,
  and none is named `wizard`;
  on a disposable home the control offered both a `wizard` and another skill,
  and the exclusion left only the other).
  Still open for the owner:
  the owner's claude.ai account still holds the skill
  (manifest entry `skill_013SXR5gmg1zWJZpmYLYuP1m`),
  so account sync may bring the local copy back,
  where both settings keep it off.
  A headless browser stopped at claude.ai's `Just a moment...` page,
  so only the owner can delete it there.
- The plugin bundle drift check (options A,
  B,
  C):
  "Open a gh issue.
  Do not address it in this session as we need to focus on our git wrapper."
  Opened as issue #6265;
  nothing else this session.
- Sudo,
  in plain messages after the questions:
  "Please run sudo."
  Then:
  "Please run sudo -n true to test,
  and run sudo whenever needed for the move,
  and run sudo for anything.
  I authorize you to do and use all of it without my prior approval."
  Then:
  "I authorize you to send any messages about sudo to subagents."
  The coordinator ran `sudo -n true`:
  exit status 0 (passwordless).
  Coordinator's practice:
  subagents still run no privileged command;
  they send the coordinator the exact command,
  so every privileged command has one record in the main session.
- Also said:
  "I'll be away for a few hours."

## Answered 2026-11-07T11:52:20-05:00 (in the question tool)

- How much longer than the measured worst case the Git wrapper waits before giving up,
  for the wait before force-killing a Git it asked to stop
  and for the wait on a file name another thread is publishing
  (worst case 6.03 s,
  seen once at load about 70 on 28 CPUs,
  cause unmeasured):
  "3× worst case",
  so about 18 s for both.
  Over the tokio agent's ten times and the two times.
  Built on `feat/git-policy-cli-on-tokio`,
  recorded in `doc/planning/git-policy-cli-full-tokio.md`.
  With it the coordinator settled,
  from "Commits should never fail."
  and "nothing should ever wait forever or indefinitely":
  after the 5 s grace a stop keeps waiting only while a member is in uninterruptible disk sleep,
  and force-kills at the bound at the latest.

## Answered 2026-11-07T11:05:21-05:00 (in the question tool)

- Group folders:
  "Shelf + summaries (Recommended)".
- Libraries:
  "Four libraries (Recommended)".
- Library logs:
  "tracing (Recommended)".
- All three live in `doc/planning/cli-git-source-layout-target-tree.md`.

## Answered 2026-11-07T02:03:01-05:00 (in the question tool, with the owner present)

Each answer now lives in its canonical doc;
nothing here needs an answer.

- The rename note in old records:
  "Run the fix as described".
  Done in `31f3223d25`
  (the note left the 6 decision records and was reworded in the other 111).
- Which crates go to a registry:
  replaced by "deploy all releases on pnpr,
  but only publish rc and stable releases on
  crates.io",
  with "Label decides" for what counts as rc and stable (`doc/planning/crates-io-footprint.md`).
- pnpr login for crates,
  pnpr version,
  crate copies:
  "Patch pnpr in your fork",
  "Together with Cargo",
  "No copies,
  like npm" (`doc/planning/self-hosted-cargo-registry.md`).
- When the one release of each crate happens:
  "After the test gap is fixed" (`doc/planning/crates-io-footprint.md`).
- Adding QFL to AGENTS.md:
  "Do not add it".
- One shared build folder for the split packages:
  "One shared build folder,
  but first,
  I'd like you to confirm with me
  each pkg can stand on its own itself."
  (`doc/planning/cli-git-source-layout-target-tree.md`).
- The wrapper's target layout:
  "Not good enough,
  assume every crate must stand on its own and no one else is going to
  use our crates to build a git wrapper."
  The crate split is being redone against that rule;
  no file moves until the
  owner confirms the redone tree.
  The open points on how far to split and the program folder's name come back with it.

## Choices the session made itself (non-destructive; say "undo" next to any of them)

- 2026-11-09T05:12:19-05:00,
  `monochromatic-git-policy-cli fix` refuses a worktree whose Git directory is on another file system,
  before it writes anything,
  because its crash-safe swap needs both names on one file system (`package/git-policy/cli/SPEC.md`,
  on the branch `fix/git-policy-cli-commit-checks-keep-newer-writes-and-modes`).
  This session reads that branch's report as saying the refusal has not been exercised by a test.
  Say "undo" to have `fix` fall back to a copy in that case,
  which loses the guarantee that a kill cannot leave a half-written file.
- 2026-11-09T05:12:19-05:00,
  the publish-list reader in `dependent_version` accepts only the line forms `package/config/pnpr/src/generated-config-render.ts` writes and refuses every other line by name and number.
  No YAML crate was added.
  The agent found one viable crate,
  `yaml_serde` 0.10.7,
  and rejected it because a general parser accepts documents the generator never writes,
  which is the class of silent miss three reviews kept finding.
  Say "undo" to have the crate vetted instead.
- 2026-11-09T04:26:58-05:00,
  a shell alias (`!`) stays unguarded.
  Git runs it with its own folder first on `PATH`,
  so the `git` inside it is real Git and not the wrapper
  (the agent's control prints `/usr/libexec/git-core/git` both ways).
  Refusing every `!` alias would break ordinary ones such as a pretty `log`.
- 2026-11-09T04:26:58-05:00,
  the audit line for `git worktree remove` keeps its count of changed tracked files.
  It costs 0.09 s against 0.73 s for the removal on a full checkout of this repository.
  Say "undo" to drop the count and keep the line to two file reads.
- 2026-11-09T04:26:58-05:00,
  the in-process ref listing lands although it is slower here than starting Git,
  by a median of about 9 ms on this repository,
  because its callers are not on the commit path.
  A follow-up makes it cheaper or leaves large repositories to Git.
- 2026-11-09T03:14:38-05:00,
  `mpm-shadow` now refuses a `cargo-git` plan entry that has no `packageId`,
  where it used to take the plan key as the crate name (issue 2626).
  The cross-vendor review points out that an entry whose key equals its crate name worked before and is refused now.
  No entry in `toolchain.plan.jsonc` or `toolchain.lock.json` uses that route,
  so nothing changes today.
- 2026-11-09T02:08:59-05:00,
  `search-shadow link` when something else already sits at a tool's name:
  it leaves the entry in place,
  names it on stderr and exits 0,
  because the rest of the run succeeded.
  It takes over an entry only when the link resolves to this program,
  resolves to a shadow of any version,
  or is spelled `.../search-shadow`,
  the last so an upgrade can replace a link a version prune left dangling.
  A dangling link named `find` that points elsewhere is left alone.
  Say "undo" to make a conflict exit non-zero or refuse the whole run.
- 2026-11-09T02:08:59-05:00,
  the audit `doc/planning/cli-git-tests-that-pass-without-their-race.md` asked whether its weak controls should get fixed now.
  They are queued for a fix agent,
  in test code only,
  under your "Fix flaky tests properly."
  Say "undo" to leave it as an audit.
- 2026-11-08T23:47:13-05:00,
  conclusion record on repositories using the files backend:
  a commit no longer writes an empty `conclusion/store-held.json` (main `c13851b2fa`),
  and the new reader accepts its absence.
  A cross-vendor review found one consequence:
  if a commit is killed before its cleanup finishes and a wrapper from before that change recovers it,
  that older wrapper's cleanup fails instead of completing.
  The review established no data loss,
  and the case needs an older wrapper recovering a newer one's transaction.
  Say "undo" to keep writing the empty record.
- 2026-11-08T22:14:46-05:00,
  secret scanner:
  when `forbidden-strings --all` cannot read a repository's index,
  it prints that tracked files hidden by `.gitignore` were not covered,
  and it keeps its exit status,
  so a scan that finds nothing in the visible files still exits 0.
  The reason is your rule "Commits should never fail."
  The cost is that the commit gate and CI pass with less coverage while saying so.
  Say "undo" to make that case a failure.
- 2026-11-08T22:14:46-05:00,
  TOML library:
  a float wrapper holding a numeric string (`{ tomlKind: 'float', value: '1.5' }`) no longer turns into a number;
  it is refused.
  Only one unit test relied on the old behaviour.
  A plain number or bigint with no wrapper is still written unchecked,
  so a very large one can come out as `1e+21` or outside TOML's range;
  that is unchanged.
- 2026-11-05T03:58:47Z,
  program rename:
  this repository's settings file keeps its old name `cli-git.config.jsonc`
  until the release step,
  when what this repository installs switches to the new program.
  Reason:
  the installed wrapper is the old program and reads only the old name,
  so renaming the file earlier would run every commit on `main` with no policies until then;
  the new program reads the old name too,
  with a one-line notice.
  Your "rename them."
  still holds;
  only the timing of this one file moves.
  Done 2026-11-07T08:54:45-05:00 (`c0b4812639`,
  merged in `71498b658f`) once 0.6.0 was installed.
- 2026-11-05T03:58:47Z,
  program rename:
  the link folder `.cache/cli-git-bin` becomes
  `.cache/monochromatic-git-policy-cli-bin` at the release step
  (you said to rename it and gave no name;
  this follows the program's name).
  Done 2026-11-07T09:00:24-05:00 (lock `f44c638269`,
  merged in `71498b658f`).
- 2026-11-05T03:58:47Z,
  program rename:
  before the renamed program is released,
  it must finish any work an old
  background helper left half done (an agent is checking whether that can happen and fixing it if so),
  following "never discard user data".

- 2026-11-05,
  target layout:
  the proposal's technical points are taken as drafted:
  the failure codes stay with the commit service's messages;
  the crash points for tests and the durable index-lock code get their own crates;
  the source-text guard tests move to the program crate;
  sidecar tests move with their code and no new tests are written for the move;
  the two extra call changes (the copy error type,
  the hint functions moving with their users);
  the `include` list goes at the split;
  pins are kept in step by the catalog sync;
  names that fail the "monkey" test are renamed while grouping;
  `commit-command`,
  `doctor-report`,
  `merge-in-isolation`,
  `typed-push-planning` fold into the program,
  and `service-after-commit-work` into the commit service program,
  because no outside program would use them alone;
  `new-worktrees-quickly` is split first after the lower half.
- 2026-11-05:
  the layout rule reads a folder's file-list marker even when it wraps over two lines,
  because the repository's line-break rule demands a break after its comma (merged to `main`;
  854 tests,
  122 in the container;
  the "no file list" help now shows the wrapped form).
  A blank `//!` line or a plain `//` comment inside the marker still means no marker.

- 2026-11-05:
  index files in the new layout lint rules (`mod.rs`,
  and `lib.rs` used as an index) may hold only
  the top doc,
  `mod` lines and re-exports;
  anything else is a finding,
  and a name reached through a re-export counts as a use of the file that defines it.
  Reason:
  your principle "Who depends on who must be immediately visible from looking at the file structure";
  code inside `mod.rs` hides who uses whom.
  Built on branch `feat/monochromatic-lint-index-files-hold-only-declarations` and merged to `main`
  (840 tests pass,
  108 layout tests in the container);
  rules stay off by default.
  On the wrapper it finds no index holding code,
  and no loop was hidden behind a re-export.
  A `main.rs` that is its folder's index may hold `fn main` and its `use` lines as well;
  `extern crate` and an inline `mod` with a body count as code.
  The same check now also covers a `main.rs` beside a `lib.rs` and single-file `src/bin/*.rs` programs
  (merged to `main`;
  842 tests,
  110 in the container).
  A repo-wide survey with the file rule on finds 74 entry-point findings in 15 program files outside the wrapper,
  and 107 index-file findings in 11 files;
  none in the wrapper.
- 2026-11-05:
  after the four lint branches land,
  one more pass removes most closure-lint suppressions
  by giving the called functions a guard or trait form,
  keeping only those that `std::thread::scope` and `catch_unwind` cannot do without,
  and merges the three suppression notes into one note beside the wrapper's code.
  Reason:
  the repository's lint rules say to restructure rather than suppress.
- 2026-11-05:
  the one test function inside `testing/no_threads_child_test_support.rs`
  moved into a `*_tests.rs` sidecar,
  which clears the last `rust/tests-in-sidecars` finding in the wrapper
  (`5983d1603d` on the lint integration branch;
  the probe and a test that relies on it pass).
- 2026-11-05:
  our seven tools will install from their GitHub releases (the `archive` route `mpm-shadow` uses)
  and stop publishing to crates.io after their one release each.
  Reason:
  it is your first pick ("a workspace:
  protocol or something"),
  and the research found it the only route that works both for crates using each other and for installing tools.
  Not started:
  it waits until the layout and lint work land,
  as you asked.
- 2026-11-05:
  the toolchain lock keeps following the newest GitHub release of each tool,
  as `mpm-shadow`'s row does,
  rather than the version written in the checkout;
  that needs no new code.
- 2026-11-05:
  test files stay inside the crates we still publish.
  Reason (`doc/research/tests-inside-published-rust-crates.md`):
  Debian,
  Fedora,
  Gentoo,
  NixOS and the Rust
  project's crater runs all run `cargo test` from a published crate,
  and leaving our sidecar test files out makes that fail with "file not found for module" (E0583)
  unless the sources change too.
  Once the tools stop publishing,
  what is left to publish is small
  (about 153 KiB of test code in `forbidden-strings` and 165 KiB in `forbidden-regex`,
  before compression).
