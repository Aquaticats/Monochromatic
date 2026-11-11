# Commits that stopped after their add steps

Status:
measurement and fix,
measured 2026-11-10 and 2026-11-11,
the fix landed 2026-11-11.
Branch:
`fix/git-policy-cli-commits-that-stop-after-their-add-steps`,
from `main` at `e498914475`,
merged with `integrate/git-policy-cli-jam-fixes-after-alpha-9`
at `ba87cce901`.
Measured release:
the installed wrapper,
`0.6.1-alpha.8`,
which wrote every record quoted here,
and then `0.6.1-alpha.9`,
which the coordinator installed at 2026-11-11T01:08:50Z:
the second read below counted 11 of its records,
all landed,
and the third counted 72,
of which 71 landed.
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
`.git/monochromatic-git-policy-cli-commit-timing.jsonl` and its generations
(`.jsonl.1`,
`.jsonl.2`,
and so on),
which move aside at `ROTATE_AT_BYTES` = 1 MiB and are kept up to
`LOG_HISTORY_BYTES` = 32 MiB
(`foundation/log_lines_appended_without_a_lock.rs`,
`foundation/log_generations_kept_to_a_bound.rs`),
read at 2026-11-10T22:40:48Z:

- 395 commit commands recorded,
  every one of them on release `0.6.1-alpha.8`.
- 366 recorded `landed`.
- 29 recorded `stopped`:
  20 with exit `2`,
  9 with exit `1`.
- No other outcome appears,
  so those 29 are every commit of that span that did not land.

The coordinator's earlier count (20 of 245,
17 of them `stopped` with exit `2`)
was taken over a shorter window.
A read has to name the files it read,
because the span grows while the files rotate:
a read of the current file alone sees only what was written since the last rotation of a mebibyte,
and one of every generation sees the whole retained 32 MiB.

A second read at 2026-11-11T01:34:44Z found the span the two files then held:

- 354 commit commands:
  335 recorded `landed`,
  1 recorded `ended` with exit `1`,
  and 18 recorded `stopped`,
  12 of them with exit `2` and 6 with exit `1`;
- the two files then covered 2026-11-10T19:56:21.832Z to 2026-11-11T01:33:45.660Z,
  because the record moves a full file aside at about one mebibyte;
- 343 of the 354 name release `0.6.1-alpha.8`,
  and 11 name `0.6.1-alpha.9`,
  which the coordinator released at 2026-11-11T01:08:45Z (`68937a8e22`)
  and which wrote records from 2026-11-11T01:08:50.349Z on;
  every one of those 11 landed.

Of those 12 exit `2` records,
nine are records of the list below and three are new,
each with the same shape and each matched to its own worktree's agent feed:

- 2026-11-10T23:22:31Z,
  `pi-pushes-of-main-that-stalled-today`:
  `git add doc/planning/git-policy-cli-push-jams-2026-11-10.md && git commit -- <paths> -m 'fix(git-policy-cli): a retry names the reason Git prints, not the remote it pushed to' -m '...'`
- 2026-11-10T23:59:44Z,
  `pi-merge-pass-fewer-wrong-merges`:
  `git add <paths> && git commit -- <paths> -m "test(git-policy-cli): two shapes a word-level merge must refuse, stored as controls ..."`
- 2026-11-11T00:11:47Z,
  `pi-gate-flakes-scratch-sweep-and-descriptor-limit`:
  `git commit -- <paths> -m 'fix(git-policy/cli): the scratch sweep controls state the entries a sweep leaves, not the count one call removes' -m '...'`

Each of those three feeds holds exactly one commit command whose message is written after `--`,
and each of those three worktrees has exactly one exit `2` record.
The last exit `2` record of that read is at 2026-11-11T00:11:47Z,
written by `0.6.1-alpha.8`;
`0.6.1-alpha.9` was cut at `68937a8e22`,
before this branch's fix,
and its 11 records of that span all landed,
which is no evidence about this shape:
none of those eleven command lines wrote a message after the paths.

A third read,
of copies of every generation taken at 2026-11-11T03:02:40.602Z,
found 415 records over 2026-11-10T19:56:21.832Z to 2026-11-11T03:02:38.791Z:

- on `0.6.1-alpha.8`,
  324 `landed`,
  1 `ended` at exit `1`,
  6 `stopped` at exit `1`
  and 12 `stopped` at exit `2`;
- on `0.6.1-alpha.9`,
  71 `landed` and 1 `stopped` at exit `1`,
  whose record carries no `stop` field at all,
  because the release predates the stop reason (`b84807a6c0`);
  that record landed again on its branch 21 s later.

The 12 exit `2` records of that read are the nine of the list below and the three later
ones the second read quotes,
so the class is closed at 2026-11-11T00:11:47Z,
and no `0.6.1-alpha.9` record stopped with exit `2` at all.
No read of the records can say why the `0.6.1-alpha.9` stop ended at exit `1`:
that release writes no stop reason.

Every one of the 20 exit `2` records carries one shape:

- the last preparation span is `index equals base`,
  and neither `index stat check` nor any `policy ...` span follows it;
- the recorded Git steps end with the `git add` starts of the private staging,
  the last of them always the `--update` one;
- `provisionMs`,
  `submitMs`,
  `reachMs`,
  `connectTries` and `answerWaitMs` are `0`.

That shape says the command stopped inside the private staging,
after the wrapper copied the real index and before it refreshed the copy.

## The records, and the standard error each one kept

Each record was matched to its worktree's run folder under `${HOME}/temp/agent/pi-runs/`,
and to the event that folder's agent kept in `feed.jsonl`.
Times are the records' own `time` field,
UTC.

Ten records kept their event:

- 2026-11-10T17:52:25Z,
  `pi-catalog-tighten-matrix-starts`:
  `git add --update -- -m fix(dependency-catalog-cli-tighten.matrix): mount the store the workspace links name ... -m Two causes stopped every scenario ...`
- 2026-11-10T17:57:32Z,
  `pi-ready-issues-681-708`:
  `git add --update -- -m refactor(ssg-aquati.cat): strip block comments with a linear walk instead of a regex ...`
- 2026-11-10T17:58:02Z,
  `pi-ready-issues-751-762`:
  `git add --update -- -m refactor(music-player-design): the fold review reads embedded payloads ... -m ...`
- 2026-11-10T18:37:55Z,
  `pi-node-modules-subvolume-measured`:
  `git add --update -- -m docs(planning): measure whether node_modules/.monochromatic can be a btrfs subvolume ...`
- 2026-11-10T19:01:41Z,
  `pi-ready-issues-885-896`:
  `git add --update -- -m refactor(ide): read the font notices' copyright line with a line scan ...`
- 2026-11-10T20:36:33Z,
  `pi-wrapper-suite-failures-on-main-tip`:
  `git add --update -- -m test(git-policy-cli): a held notice write is judged without a wall-clock race ...`
- 2026-11-10T20:40:29Z,
  `pi-readme-claims-worktrees-and-locks`:
  `git add --update -- own notice line was undocumented. failed: fatal: pathspec 'own' did not match any files`
- 2026-11-10T20:53:28Z,
  `pi-held-push-released-by-the-retry`:
  `git add --update -- -m fix(git-policy-cli): a held push is settled by the verdict its command left ...`
- 2026-11-10T21:54:07Z,
  `pi-wrapper-citations-outside-decision-records`:
  `git add --update -- -m docs(planning): #1907 names three deleted citations and the note each needs ...`
- 2026-11-10T22:04:30Z,
  `pi-process-look-ceiling-under-load`:
  `git add --update -- -m test(git-policy-cli): controls for a command line the look could not prove ...`

Ten more records have the same shape,
but their own text is not quotable:

- 2026-11-10T22:28:49Z is the main checkout's own commit (`gitDir` `.git`),
  which has no agent feed;
- 2026-11-10T18:00:41Z,
  `pi-ready-issues-787-798`;
- 2026-11-10T18:02:14Z,
  `pi-ready-issues-411-650`;
- 2026-11-10T18:24:20Z,
  `pi-worktree-trash-hand-over-review-findings`;
- 2026-11-10T18:31:59Z,
  `pi-ready-issues-897-909`;
- 2026-11-10T19:43:57Z,
  `pi-ready-issues-1173-1300`;
- 2026-11-10T19:44:23Z,
  `pi-worktree-add-follow-ups-from-the-handover`;
- 2026-11-10T20:19:07Z,
  `pi-readme-claims-commit-and-merge`;
- 2026-11-10T22:36:48Z,
  `pi-worktree-add-tree-fill-cost`;
- 2026-11-10T22:37:14Z,
  `pi-ignored-path-and-waiting-path-review-findings`.

The nine worktree records of that list kept their event only inside a tool result the
feed truncated,
and the main checkout's commit has no feed at all.

The reproduction below runs the command shape the ten quotable records name.

The same worktree feed shows the command line that produced one of them,
in `ready-issues-681-708-1/feed.jsonl`:

```text
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
and the record's own `provisionMs`,
`submitMs`,
`reachMs`,
`connectTries` and `answerWaitMs` are all `0`.

So the stop is not a wait in the service,
not a lock another commit held,
and not a slow disk:
the command refused its own command line in its own process,
and then called that refusal an `engine-failure` with code `transaction-failed`
and exit `2`.

The nine exit `1` records are a different class and are not diagnosed here.
Eight of them record the scratch repository's `git commit` as their last Git step
and end with exit `1`,
which is Git's own status:
the one whose worktree feed quotes it
(2026-11-10T22:36:45Z,
`pi-worktree-add-tree-fill-cost`)
shows
`error: there was a problem with the editor 'hx'`
and `Please supply the message using either -m or -F option`
for a commit line that named no message.
The ninth (2026-11-10T18:58:08Z,
`pi-ready-issues-861-872`) stopped after two
`git rev-parse` starts and one `git diff-index`,
before any preparation span,
and its feed does not quote the text.

## Why the wrapper said what it said

`landing/service_prepare.rs` turned every failure of the private index build into
`failed(error.0)` (`commit/commit_transaction.rs`),
and `commit/commit_command.rs` renders a `TransactionStop::Failed` as an
`engine-failure` event with exit `2` and this hint:

> if you were committing,
> run `git status` and `git log --oneline -3` to see whether the commit landed before you retry

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

- all three ended within 47 ms with exit `2`,
  and each printed
  `{"type":"engine-failure","code":"transaction-failed","message":"git add --update -- -m one two failed: fatal: pathspec '-m' did not match any files", ...}`
  with the `git log --oneline -3` hint;
- the base commit before them landed at 2026-11-10T22:57:33.837Z (`fa1007b`);
- the control right after them,
  `git commit --message='one two' -- a.txt`,
  landed at 2026-11-10T22:57:35.149Z (`45611fb`),
  so the repository,
  the index and the service were all able to land a commit
  at that moment.

## The fix

Two branches answered this refusal,
and the merge kept one implementation.
The integrated one
(`fix/git-policy-cli-commit-pathspec-matching-nothing-is-an-input-refusal`,
merged into the integration branch at `b0ee8414e1`,
fixing issue 6283)
classifies the staging's own failure.
This branch's own answer was dropped there for the three reasons
"How the two answers met" records;
the behaviour both aimed at is the one below.

`commit/commit_selection_staging.rs` gives the private staging's failure a type,
`SelectionStagingFailure`,
with two arms:
`Git(GitFailure)` for every failure that was always an engine failure,
and `PathspecMatchesNothing(Vec<Vec<u8>>)` for tokens the caller wrote and Git refused.
Only the start that stages the directory and pattern elements asks the question,
and only after that start failed,
so a commit that stages its selection starts no Git it did not start before,
and every other start of the staging keeps the class and the wording it had.

`commit/commit_pathspec_matching_nothing.rs` reads the refusal from the words of that
`git add`:

- a token is read only between Git's own `pathspec '` and one of the two wordings Git
  closes that sentence with (`' did not match any files` for `git add`,
  `' did not match any file(s) known to git` for `git commit`),
  scanning forward from one reading to the next,
  so no other sentence of Git's can be mistaken for a refusal;
- a token is kept only when it is byte-equal to an element the caller wrote
  (`WrittenPathspecs::Elements` on the start that carries the caller's own elements,
  `FromTheCallersFile` when Git read the caller's pathspec file itself),
  so a pathspec the wrapper wrote of its own can never be reported as the caller's mistake;
- the finding names each refused token as Git named it,
  and a token that begins with a hyphen also says that options belong before `--`
  (`git commit --message <text> -- <paths>`),
  which is the shape issue 6283 reported and the shape every record above carries;
  a token without a hyphen is told to correct its spelling or to
  `git add -- <path>` what it names.

`commit/commit_index.rs` returns that type from `initialize_commit_index`,
and `landing/service_prepare.rs` reports the two arms apart:
`SelectionStagingFailure::Git` is still `failed(...)`,
an `engine-failure` with exit `2` and the hint about `git status` and `git log`,
and `SelectionStagingFailure::PathspecMatchesNothing` ends the preparation as
`Prepared::Ended(pathspec_matching_nothing_ending(...))`:
one `core-finding` for `explicit-path-commit/pathspec-matches-nothing`,
exit `1`,
nothing landed,
nothing staged,
and no hint that asks whether a commit which never ran had landed
(`landing/service_prepare_selection_refusals.rs`).

Where such a commit ends,
its timing record says why
(`feat/git-policy-cli-timing-records-name-why-a-commit-stopped`,
merged at `b84807a6c0`):
`reason_of_events` takes the first core finding of the events and
`commit_command.rs` writes outcome `ended` for a result that landed nothing,
so the record now reads exit `1` and stop reason kind `core-finding`,
core `explicit-path-commit`,
code `pathspec-matches-nothing`,
instead of `engine-failure` / `transaction-failed` (`commit/commit_stop_reason.rs`).

## How the two answers met

This branch had built its own answer before it merged the integration branch:
`commit/commit_named_unmatched_files.rs` asked Git,
after the private staging refused,
which of the selection's own elements match no entry of the private index
(one `git ls-files --cached -z -- <element>` per element that names no file of the worktree),
and reported `explicit-path-commit/named-path-unmatched` with exit `1`
(red first at `166973dbd6`,
green at `ee7a4f6089`).

Both answers mean one thing:
Git refused the person's own command line,
nothing landed,
nothing was staged,
and the exit status is `1` rather than `2`.
Two implementations of one refusal would leave which finding appears to the order of
branches inside the wrapper,
so the merge kept the integrated one,
on three grounds:

- it adds no Git start to a refusal.
  The per-element question started one `git ls-files` for every element that named no
  file,
  and the probe of this branch's own answer at 2026-11-11T01:42:53.192Z took 109 ms for
  three refusals where the before-fix run of 2026-11-10T22:57:35.102Z took 47 ms.
  The two runs were not made under the same host load,
  so that difference measures nothing by itself;
  the starts are still a cost that reading the refusal already in hand does not pay.
- it asks only the start whose pathspecs are the caller's own elements,
  so a file that disappears between the wrapper's check and the staging stays that
  start's Git failure,
  rather than being reported as a spelling.
- it tells the hyphen shape where the option belongs,
  which is the shape issue 6283 reported and the shape of every stopped record above.

What the per-element answer named that this one does not:
an element of a message written after `--` that Git itself never named.
Git stops at the first refused pathspec,
so `git commit -- a.txt -m 'one two'` is reported with `-m` named,
and the sentence about `--` covers the words of the message.
A control for that exact recorded shape stayed in the binary suite
(`a_message_written_after_the_paths_is_refused_as_the_pathspec_it_became`,
`binary/binary_commit_tests.rs`),
and the branch's red-first commit stays in its history.

## The reproduction, after the fix

The same probe,
run at 2026-11-11T03:04:57.777Z against the wrapper built from this branch after
`ba87cce901`,
the merge with the integration branch,
in a fresh disposable repository:

- the service was listening at 2026-11-11T03:04:58.536Z,
  and the base commit of 201 files landed at 2026-11-11T03:05:09.795Z (`04072e9`),
  11,145 ms after it started (`totalMs` in its own record);
- three `git commit -- a.txt -m 'one two'` started at 2026-11-11T03:05:09.811Z,
  and all three ended within 2.372 s with exit `1`,
  each printing one event and nothing else on standard error:
  `{"schemaVersion":1,"sequence":0,"type":"core-finding","trigger":"pre-forward","coreId":"explicit-path-commit","code":"explicit-path-commit/pathspec-matches-nothing","message":"monochromatic-git-policy-cli: git commit with explicit paths selects worktree content, and Git refuses a pathspec that matches nothing, so this commit was not made and nothing was staged by it: pathspec '-m' did not match any files. An option written after -- is a pathspec to Git: options belong before --, as in git commit --message <text> -- <paths>."}`
  with no `engine-failure`,
  no `transaction-failed`
  and no `git log --oneline -3` hint;
- each refusal's own record names why it ended:
  written at 2026-11-11T03:05:11.150Z,
  03:05:11.975Z and 03:05:12.180Z,
  outcome `ended`,
  exit `1`,
  stop `{"kind":"core-finding","core":"explicit-path-commit","code":"pathspec-matches-nothing"}`,
  `prepareMs` 1303,
  807 and 196,
  and inside that,
  the one `git add` of the private staging is 551.955 ms,
  501.957 ms and
  179.702 ms;
- the control at 2026-11-11T03:05:12.184Z,
  `git commit --message='one two' -- a.txt`,
  landed as `9115c63`,
  and `git status --short` at 2026-11-11T03:05:23.368Z printed nothing,
  so the three refusals staged nothing;
- the whole probe run took 25.6 s,
  of which the two landings are 22.3 s:
  this wrapper is a development build and the host carries dozens of agents,
  and what a landing costs is another branch's question
  (`perf/git-policy-cli-commit-tail-on-the-installed-wrapper`).
  A refusal costs one `git add --update` over the private index,
  which Git answers with the refusal,
  and nothing else of the commit runs.

The before run at 2026-11-10T22:57:35.102Z ended the same three commands in 47 ms
with exit `2` and one `engine-failure` each,
and the run of 2026-11-11T01:42:53.192Z,
against a build that still carried this branch's own per-element answer,
ended them in 109 ms.
Those three runs were not made under the same host load,
so the differences between them measure nothing on their own.
What is not a timing claim:
the same command line starts the same Git,
a refusal adds no start of its own,
and the counting above says the one `git add` is where a refusal's time goes.

## What this does not change

- Elements read from a pathspec file whose bytes do not fit one command line
  (`Selected::File`) and elements of one this reader cannot take
  (`Selected::Unreadable`)
  keep the old ending:
  those starts carry the caller's file rather than the caller's own elements,
  so nothing asks the question,
  and Git's own refusal comes back as the engine failure it was.
- A `git add` refusal of a named file for a reason other than a missing path
  (a path outside the worktree,
  a path inside `.git`,
  content Git cannot read)
  keeps the old ending for the same reason:
  no pathspec of that start matches nothing,
  so nothing names it.
  `a_staging_failure_that_is_not_a_pathspec_matching_nothing_stays_an_engine_failure`
  is the control for the unreadable-content case.
- The nine exit `1` records are a different class and are not diagnosed here.
