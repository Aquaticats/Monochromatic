# Commits that stopped after their add steps

Status:
measurement and fix,
2026-11-10.
Branch:
`fix/git-policy-cli-commits-that-stop-after-their-add-steps`,
from `main` at `e498914475`.
Measured release:
the installed wrapper, `0.6.1-alpha.8`,
which wrote every record quoted here,
and then `0.6.1-alpha.9`,
which the coordinator installed at 2026-11-11T01:08:50Z and which wrote 11 of the
records recounted below.
Paths below are relative to `package/git-policy/cli/src/native/` unless they start with `doc/`.

Purpose:
the owner counted commits that never landed and said
"Prioritize fixing the jams",
"commits vs native git should not take 5s longer",
and "Commits should never fail".
This page records what those commits stopped on,
matched to the commit service's journal and to the client's own standard error,
and what the wrapper says instead on this branch.

## What was counted

The timing records in the common Git directory,
`.git/monochromatic-git-policy-cli-commit-timing.jsonl` and its rotated file `.1`,
read at 2026-11-10T22:40:48Z:

- 395 commit commands recorded, every one of them on release `0.6.1-alpha.8`.
- 366 recorded `landed`.
- 29 recorded `stopped`: 20 with exit `2`, 9 with exit `1`.
- No other outcome appears, so those 29 are every commit of that span that did not land.

The coordinator's earlier count (20 of 245, 17 of them `stopped` with exit `2`)
was taken over a shorter window;
the two files here are the whole span the records cover,
and the second file rotates at about one mebibyte,
so older records are gone.

A second read at 2026-11-11T01:34:44Z found the span the two files then held:

- 354 commit commands: 335 recorded `landed`,
  1 recorded `ended` with exit `1`,
  and 18 recorded `stopped`, 12 of them with exit `2` and 6 with exit `1`;
- the two files then covered 2026-11-10T19:56:21.832Z to 2026-11-11T01:33:45.660Z,
  because the record rotates at about one mebibyte and the window before it is gone;
- 343 of the 354 name release `0.6.1-alpha.8`,
  and 11 name `0.6.1-alpha.9`,
  which the coordinator released at 2026-11-11T01:08:45Z (`68937a8e22`)
  and which wrote records from 2026-11-11T01:08:50.349Z on;
  every one of those 11 landed.

Of those 12 exit `2` records, nine are records of the list below and three are new,
each with the same shape and each matched to its own worktree's agent feed:

- 2026-11-10T23:22:31Z, `pi-pushes-of-main-that-stalled-today`:
  `git add doc/planning/git-policy-cli-push-jams-2026-11-10.md && git commit -- <paths> -m 'fix(git-policy-cli): a retry names the reason Git prints, not the remote it pushed to' -m '...'`
- 2026-11-10T23:59:44Z, `pi-merge-pass-fewer-wrong-merges`:
  `git add <paths> && git commit -- <paths> -m "test(git-policy-cli): two shapes a word-level merge must refuse, stored as controls ..."`
- 2026-11-11T00:11:47Z, `pi-gate-flakes-scratch-sweep-and-descriptor-limit`:
  `git commit -- <paths> -m 'fix(git-policy/cli): the scratch sweep controls state the entries a sweep leaves, not the count one call removes' -m '...'`

Each of those three feeds holds exactly one commit command whose message is written after `--`,
and each of those three worktrees has exactly one exit `2` record.
The last exit `2` record is at 2026-11-11T00:11:47Z,
written by `0.6.1-alpha.8`;
`0.6.1-alpha.9` was cut at `68937a8e22`,
before this branch's fix,
and its 11 records of this span all landed,
which is no evidence about this shape:
none of those eleven command lines wrote a message after the paths.

Every one of the 20 exit `2` records carries one shape:

- the last preparation span is `index equals base`,
  and neither `index stat check` nor any `policy ...` span follows it;
- the recorded Git steps end with the `git add` starts of the private staging,
  the last of them always the `--update` one;
- `provisionMs`, `submitMs`, `reachMs`, `connectTries` and `answerWaitMs` are `0`.

That shape says the command stopped inside the private staging,
after the wrapper copied the real index and before it refreshed the copy.

## The records, and the standard error each one kept

Each record was matched to its worktree's run folder under `${HOME}/temp/agent/pi-runs/`,
and to the event that folder's agent kept in `feed.jsonl`.
Times are the records' own `time` field, UTC.

Ten records kept their event:

- 2026-11-10T17:52:25Z, `pi-catalog-tighten-matrix-starts`:
  `git add --update -- -m fix(dependency-catalog-cli-tighten.matrix): mount the store the workspace links name ... -m Two causes stopped every scenario ...`
- 2026-11-10T17:57:32Z, `pi-ready-issues-681-708`:
  `git add --update -- -m refactor(ssg-aquati.cat): strip block comments with a linear walk instead of a regex ...`
- 2026-11-10T17:58:02Z, `pi-ready-issues-751-762`:
  `git add --update -- -m refactor(music-player-design): the fold review reads embedded payloads ... -m ...`
- 2026-11-10T18:37:55Z, `pi-node-modules-subvolume-measured`:
  `git add --update -- -m docs(planning): measure whether node_modules/.monochromatic can be a btrfs subvolume ...`
- 2026-11-10T19:01:41Z, `pi-ready-issues-885-896`:
  `git add --update -- -m refactor(ide): read the font notices' copyright line with a line scan ...`
- 2026-11-10T20:36:33Z, `pi-wrapper-suite-failures-on-main-tip`:
  `git add --update -- -m test(git-policy-cli): a held notice write is judged without a wall-clock race ...`
- 2026-11-10T20:40:29Z, `pi-readme-claims-worktrees-and-locks`:
  `git add --update -- own notice line was undocumented. failed: fatal: pathspec 'own' did not match any files`
- 2026-11-10T20:53:28Z, `pi-held-push-released-by-the-retry`:
  `git add --update -- -m fix(git-policy-cli): a held push is settled by the verdict its command left ...`
- 2026-11-10T21:54:07Z, `pi-wrapper-citations-outside-decision-records`:
  `git add --update -- -m docs(planning): #1907 names three deleted citations and the note each needs ...`
- 2026-11-10T22:04:30Z, `pi-process-look-ceiling-under-load`:
  `git add --update -- -m test(git-policy-cli): controls for a command line the look could not prove ...`

Ten more records have the same shape,
but their own text is not quotable:

- 2026-11-10T22:28:49Z is the main checkout's own commit (`gitDir` `.git`), which has no agent feed;
- 2026-11-10T18:00:41Z, `pi-ready-issues-787-798`;
- 2026-11-10T18:02:14Z, `pi-ready-issues-411-650`;
- 2026-11-10T18:24:20Z, `pi-worktree-trash-hand-over-review-findings`;
- 2026-11-10T18:31:59Z, `pi-ready-issues-897-909`;
- 2026-11-10T19:43:57Z, `pi-ready-issues-1173-1300`;
- 2026-11-10T19:44:23Z, `pi-worktree-add-follow-ups-from-the-handover`;
- 2026-11-10T20:19:07Z, `pi-readme-claims-commit-and-merge`;
- 2026-11-10T22:36:48Z, `pi-worktree-add-tree-fill-cost`;
- 2026-11-10T22:37:14Z, `pi-ignored-path-and-waiting-path-review-findings`.

The nine worktree records of that list kept their event only inside a tool result the
feed truncated, and the main checkout's commit has no feed at all.

The reproduction below runs the command shape the ten quotable records name.

The same worktree feed shows the command line that produced one of them,
in `ready-issues-681-708-1/feed.jsonl`:

```
git commit -- package/ssg/aquati.cat/src/build/project-source-scanning.ts ... -m 'refactor(ssg-aquati.cat): ...'
```

The message is written after `--`,
so Git reads `-m` and every word of the message as a pathspec.
The private staging asks Git for exactly those elements,
and `git add --update` answered
`fatal: pathspec '-m' did not match any files`.

## The commit service never saw them

The service's journal names the request of every commit it answers
(`answered request="..." branch="..." outcome=landed ...`).
None of the 20 `requestId` values of the exit `2` records appears anywhere in
`journalctl --user` over the boot that holds these records,
and the record's own `provisionMs`, `submitMs`, `reachMs`,
`connectTries` and `answerWaitMs` are all `0`.

So the stop is not a wait in the service,
not a lock another commit held,
and not a slow disk:
the command refused its own command line in its own process,
and then called that refusal an `engine-failure` with code `transaction-failed`
and exit `2`.

The nine exit `1` records are a different class and are not diagnosed here.
Eight of them record the scratch repository's `git commit` as their last Git step
and end with exit `1`, which is Git's own status:
the one whose worktree feed quotes it
(2026-11-10T22:36:45Z, `pi-worktree-add-tree-fill-cost`)
shows
`error: there was a problem with the editor 'hx'`
and `Please supply the message using either -m or -F option`
for a commit line that named no message.
The ninth (2026-11-10T18:58:08Z, `pi-ready-issues-861-872`) stopped after two
`git rev-parse` starts and one `git diff-index`, before any preparation span,
and its feed does not quote the text.

## Why the wrapper said what it said

`landing/service_prepare.rs` turned every failure of the private index build into
`failed(error.0)` (`commit/commit_transaction.rs`),
and `commit/commit_command.rs` renders a `TransactionStop::Failed` as an
`engine-failure` event with exit `2` and this hint:

> if you were committing, run `git status` and `git log --oneline -3` to see whether the commit landed before you retry

A commit whose pathspec matched nothing cannot have landed,
so the hint sent the agent looking for a landing that could not exist,
and the exit status `2` (`doc/decision/cli-output-and-exit-codes.md`,
"the caller or configuration is wrong")
was the only part of the report that was right.

The wrapper already had one finding of exactly this kind:
a commit naming a path an ignore rule matches stops with
`explicit-path-commit/named-path-ignored`,
exit `1`,
naming every rejected path and both ways forward
(`commit/commit_named_ignored_files.rs`,
`landing/service_prepare_selection_refusals.rs`,
merged at `b4ed7deb73`).
Every other refusal of the staging had no finding of its own.

## The reproduction, before the fix

A disposable repository made by the probe
(`${HOME}/temp/agent/pi-commits-stop/probe.ts`),
with the wrapper built from this branch at `e498914475` reached through a
`git`-named link on the probe's own `PATH`,
its own `HOME`,
a short `XDG_RUNTIME_DIR`,
one commit service started by the probe itself,
and three commits started at once in that one repository.

At 2026-11-10T22:57:35.102Z,
`git commit -- a.txt -m 'one two'` three times at once:

- all three ended within 47 ms with exit `2`, and each printed
  `{"type":"engine-failure","code":"transaction-failed","message":"git add --update -- -m one two failed: fatal: pathspec '-m' did not match any files", ...}`
  with the `git log --oneline -3` hint;
- the base commit before them landed at 2026-11-10T22:57:33.837Z (`fa1007b`);
- the control right after them,
  `git commit --message='one two' -- a.txt`,
  landed at 2026-11-10T22:57:35.149Z (`45611fb`),
  so the repository, the index and the service were all able to land a commit
  at that moment.

## The fix

`commit/commit_named_unmatched_files.rs` asks Git,
after the private staging refused the selection,
which of the selection's own elements match no entry of the private index:
one `git ls-files --cached -z -- <element>` per element,
through the same request builder and the same pathspec modes the staging used,
so the answer is the staging's own question and never the words of a Git message.
An element that names one file of the worktree is not asked about:
the worktree is the question for it,
and a refusal there is Git's own text.
A selection whose every element matches asks nothing.

`landing/service_prepare_selection_refusals.rs` turns that answer into the finding
`explicit-path-commit/named-path-unmatched`:
exit `1`,
nothing landed,
nothing staged,
every unmatched element named,
and both ways forward:

- write the message before the paths,
  `git commit -m <message> -- <paths>`;
- name elements the worktree or the commit's index holds.

Nothing asks the question when the private staging succeeds,
so a commit that stages correctly starts no Git it did not start before.

The branch that names why a commit stopped
(`feat/git-policy-cli-timing-records-name-why-a-commit-stopped`,
merged into the integration branch at `b84807a6c0`) is part of this branch:
where such a commit ends, its timing record says outcome `ended` with exit `1`,
and the stop reason `core-finding` with core `explicit-path-commit` and code
`named-path-unmatched`
(`commit/commit_stop_reason.rs`, `commit/commit_command.rs`),
so the record no longer looks like an engine failure either.

## The reproduction, after the fix

The same probe,
run at 2026-11-11T01:42:53.192Z against the wrapper built from this branch after
`f14bf55b0c`, the merge with the integration branch,
in a fresh disposable repository:

- the base commit landed at 2026-11-11T01:42:54.012Z (`197b212`);
- three `git commit -- a.txt -m 'one two'` started at 2026-11-11T01:42:59.244Z,
  and all three ended within 109 ms with exit `1`,
  each printing one event and nothing else on standard error:
  `{"type":"core-finding","trigger":"pre-forward","coreId":"explicit-path-commit","code":"explicit-path-commit/named-path-unmatched","message":"... these elements match no file of the worktree and no entry of the commit's index, so this commit was not made and nothing was staged by it: [\"-m\",\"one two\"]. Native Git refuses the same command line with exit status 1. ..."}`
  with no `engine-failure`, no `transaction-failed`
  and no `git log --oneline -3` hint;
- the control at 2026-11-11T01:42:59.354Z,
  `git commit --message='one two' -- a.txt`,
  landed as `3a492ad`,
  and `git status --short` at 2026-11-11T01:43:00.588Z printed nothing,
  so the three refusals staged nothing;
- the whole probe run took 7.4 s on this host,
  and the three refusals took 109 ms of it.

The before run at 2026-11-10T22:57:35.102Z ended the same three commands in 47 ms
with exit `2` and one `engine-failure` each.
This run took 109 ms.
The two runs were not made under the same host load,
so their difference is no measurement of the question alone;
what the wrapper adds on a refusal is two `git ls-files` starts,
one per element that names no file of the worktree,
and a commit that stages its selection starts neither.

## What this does not change

- Elements read from a pathspec file whose bytes do not fit one command line
  (`Selected::File` and `Selected::Unreadable` in `commit/commit_selection_staging.rs`)
  keep the old ending:
  the wrapper does not read those elements,
  so it cannot name them,
  and Git's own refusal comes back as the transaction failure it was.
- A `git add` refusal of a named file for a reason other than a missing path
  (a path outside the worktree, a path inside `.git`)
  keeps the old ending for the same reason:
  no element of that selection matches nothing,
  so nothing names it.
- The nine exit `1` records are a different class and are not diagnosed here.
