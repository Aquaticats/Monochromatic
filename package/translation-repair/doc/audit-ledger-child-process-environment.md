# Audit ledger: the environment handed to child processes

What a child process inherits:
keys,
settings and git's own variables,
in production code and in tests.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B291: each test that started a child built the child's environment itself, and several passed the keys on

Scan red in `a67f453cb`,
fixed in `cebfcc122`.

Found on 2026-10-06 (UTC),
after a fixture merged with a coverage cluster was found to start a child
with the parent's environment whole (M121).
The owner's rule is that keys come from the task runner and no child of a test inherits them.
Nothing held it:
each test file that started a child built the child's environment itself,
or passed none.
`nano-spawn` merges the `env` it is given into the parent's environment
(the lead's reading of its type definitions,
stated in the agent's brief),
so leaving a variable out of that object removes nothing.
No child of any of these files calls a provider,
so no key was used or shown.

The fix:
`src/child-environment.test-fixture.ts` (moved on 2026-10-18 to
`package/translation-repair/adjudication/src/child-environment.test-fixture.ts`) is the one place a test may start a
child.
It hands the child the test process's environment with every variable whose name ends in `_API_KEY` removed,
and every variable whose name starts `TRANSLATION_REPAIR_` too,
then the variables the case names
(`environmentWithoutKeys`,
`spawnKeyless` over `nano-spawn`,
`runKeyless` over node's own `spawn`,
`runBuiltCommand` for a built command).
A removed name is stated as `undefined`,
which the merge inside `nano-spawn` and node's `spawn` both read as absent.
Its test file starts real children that print the names they see,
with a control case that hands a child a key-named variable and sees it arrive.

`src/test-children-keyless.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/test-children-keyless.unit.test.ts`) reads every test file and fixture but
that one.
It fails on an import of a spawning module
(`nano-spawn`,
`node:child_process`,
`execa`,
`cross-spawn`) under any form,
and on a call of a spawning function.
It joined the `source-scans` task,
which ran 41 files from then.
Its package case failed with 54 findings in 27 files,
an import and a call in each.
Of the 27,
five are the keyless fixtures of the runner clusters merged since the agent's base,
where the agent counted 44 findings in 22 files.

Every child of the 27 files goes through the fixture.
`git` in a throwaway repository runs through `spawnKeyless`,
node on a script or a built command through `runKeyless`,
and built commands through `runBuiltCommand`.
Three files passed more than the rule allows,
by the agent's reading:
`verify-published.unit.test.ts` spread the parent's environment into its child,
keys included;
`fidelity-reference-cli` blanked the keys it knew of and let the rest through;
and `editor-standing-read.unit.test.ts` wrote the runs directory into the parent's own environment,
for its child to inherit.
That file hands the directory to the child by name,
so its helper `runsDirPointedAt`,
the two inventory lines of `duplicate-bodies` that named it and the suite's `concurrency: 1` are gone.
The five fixtures of the runner clusters are the lead's edits:
three keep their exported names and call `runBuiltCommand`,
one calls `runKeyless`,
and one runs git through `spawnKeyless`.

Measured by the lead on the red tree:
the new scan alone fails with the 54 findings and its own fixture case passes;
the fixture's test file passes;
and the `source-scans` task prints 41 pass lines and fails on that one case alone.
Measured by the lead on the fix tree,
with the two red test files of `3c74be74a` also in it:
the whole unit suite prints 1,823 pass lines and 10 fail lines,
every one of the 10 in those two files (M125);
the `source-scans` task prints 41 pass lines and no fail line;
and lint ends with no warning and no error.
The agent could not run `bench-sample-draw.unit.test.ts` in its worktree,
where the git policy shim was not built;
it runs in the whole suite.
The whole suite ran on `040656c51` in a side checkout,
which holds the fixes of B291 to B293:
1,823 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open:
production code starts children that inherit the parent's environment whole
(`git` from `corpus-source.ts`,
`artifact-generation.ts`,
`coverage-census-commit.ts`,
`cache-account-git.ts` and `run-config.ts`;
the image readers of `image-ocr.ts`;
and the coverage census,
which runs the unit suite as a child).
None calls a provider.
A follow-up for these was not merged at `f8e99b84d`.
One name for the three fixtures that only rename `runBuiltCommand` waits for the last runner cluster.
Both closed since:
the children in `4049d9eb2` (B314),
and the wrapper fixtures folded into `runBuiltCommand` in `3d36ba7b9` (B319).

Open to the owner's veto:
removing every `TRANSLATION_REPAIR_` variable and not the keys alone;
a case's own variable winning over the removal,
so a case can hand a child a key-named variable on purpose;
`runKeyless` reporting a signalled child as code -1;
no standard input for `runBuiltCommand`;
and the node child of `bench-sample-draw` inheriting the environment less keys and settings,
with `HOME` set by the case,
where it was given `PATH` and `HOME` alone.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B314: every child production code starts was handed the run's provider keys and settings

Red in `93c30e8ea`,
fixed in `4049d9eb2`.

Found on 2026-10-06 (UTC),
the item B291 left open.
The git children of the corpus reads,
of the artifact generation checks,
of the cache account,
of the run configuration and of the coverage census,
the census's suite child,
and the picture program
were started with the run's whole environment,
or with nothing said about it.
A key could leave through a child's own log,
a crash dump,
a tool's debug output,
or a hook or alias that a repository's configuration has a git child run.
Measured by the lead on the red tree:
the new scan reported thirteen starts,
and all ten new cases failed,
nine of them seeing the invented key and setting arrive beside the plain variable.

The fix:
`child-process-environment.ts` builds the environment of every child,
the parent's with every name ending `_API_KEY` and every `TRANSLATION_REPAIR_` setting stated as absent,
and every start of a child in the package's source passes it;
`corpusGitEnvironment` is built on it.
The lead made the four git starts of `corpus-run/coverage-census-commit.ts`
and the suite child of `corpus-run/coverage-census-steps.ts`,
which another agent was moving at the time.
`production-children-keyless.unit.test.ts`,
among the source scans,
fails a production start whose `env` is not built by `childEnvironment`;
`production-children-environment.unit.test.ts` starts each site's real child against an invented key
and reads from the child's own side,
git's trace or a file the program writes,
that only the plain control variable arrives.
The whole suite on `4049d9eb2`,
run in a side checkout:
1,985 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 42 source scans passing.

Not measured:
a picture program reading a real picture without the removed names,
and the whole suite run as the census's child without any setting of the package;
the next whole-suite census measures the second.
Out of the scan's reach,
by its own header:
a start through an alias (`const run = spawn`),
a computed member,
and starts in other workspace packages.

Open to the owner's veto:
removed names stated as `undefined`;
the settings removed with the keys for every child,
with no switch for a site;
the sites reading the process's environment where they start the child;
the cases writing invented variables to the test process's environment,
one at a time.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B315: the corpus listing's git child kept the repository variables its guard meant to drop

Red in `93c30e8ea`,
fixed in `4049d9eb2`.

Found on 2026-10-06 (UTC) by the agent of B314.
`corpusGitEnvironment` left `GIT_DIR` and its kin out of the object it returned,
which only `execFile` reads as absent;
`nano-spawn` merges the object it is given over the process's environment,
so every name left out came back from the parent.
`listCorpusPeople` starts its git through `nano-spawn`,
so with `GIT_DIR` pointing at another repository it failed its read with `CorpusReadError`,
while `readCorpusBytes`,
which starts its git through `execFile`,
read the right bytes.
The red commit's case sets `GIT_DIR` to another repository and expects the listing of the pinned clone.

The fix:
the environment states each removed name with the value `undefined`,
which both spawn forms read as absent.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world":
an environment for a merging spawner states what it removes,
and the case reads the child's own environment.

## B319: nothing stopped a built command from running in a shell that held provider keys

Red in `668340466`,
fixed in `3d36ba7b9`;
the configuration doc's paragraph broken at its semantic points in `c8196cf6c` (M133),
and the corpus pass runbook's two direct starts given the marker in `5f3e26958`.

Found on 2026-10-06 (UTC) after M120:
an agent started a built command by hand from a shell holding the operator's real keys,
and one start bought six model calls.
The rule for agents that followed is no guard.

The fix:
`taskRunnerVerdict` (`task-runner-guard.ts`) lets a process that holds a key,
a variable whose name ends `_API_KEY` with a value that is not blank,
run a command only when `TRANSLATION_REPAIR_STARTED_BY` names that command;
a process holding no key runs as before.
`reportingRefusals` takes the process's environment as a required `env`
and asks the guard before it reads the command line,
so the refusal is the stated one every entry ends in,
at exit 6.
Each of the 42 entry files hands it `process.env`,
and each of the 42 command tasks in `mise.toml` sets the marker to its command's name.
`runBuiltCommand` of `child-environment.test-fixture.ts` is the one way a test starts a built command,
and sets the marker for the command it starts unless a case asks otherwise;
four wrapper fixtures were folded into it,
by the agent's account 83 call sites in 26 test files.
`task-runner-guard-reach.unit.test.ts`,
among the source scans,
fails a task that starts a built command without the marker,
a built command no task starts,
and an entry that hands no environment;
on the red tree its package case reported 84 findings,
the count of the 42 tasks and the 42 entry files the fix changes.

The lead's change to the agent's wording,
open to the owner's veto:
the agent wrote the refusal as "unless its own task started it",
the task as the one way out;
the corpus pass runbook starts a built file directly where nothing may rebuild while a pass runs,
so the marker is the process's own statement of the command it means to start,
which the task,
the test fixture or an operator by hand sets,
and the refusal names both ways.
The module's comment says the guard is no access control.
Measured by the lead:
started through its task from a shell holding the session's keys,
`spend-report` built the package,
passed the guard and printed its own refusal for a missing log file at exit 6;
`mise tasks info` reads the marker on the task.
The whole suite on `3d36ba7b9`,
run in a side checkout:
1,993 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 44 source scans passing.

Open:
a process that imports the built library directly,
a `node --eval` or a script,
can still build a client from its environment,
since the library cannot know what started it;
a shell in a worktree still holds the keys.
Three test files start a built command through `runKeyless` with their own path
(`verify-published`,
`editor-standing-read`,
`sink-names-only`),
keyless,
so the guard passes them.

Open to the owner's veto,
the agent's:
a blank key value counts as no key;
a marker value is repeated in the refusal only when it has a command name's shape;
`env` required on `reportingRefusals`.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world" and "Guards that cannot fail".

## B347: the corpus git children spoke the caller's language

Red in `de6ff17`,
fixed in `a7eca7f`.

Found on 2026-10-06 (UTC) by the agent writing the runner observations,
whose change is not merged yet.
`corpusGitEnvironment` kept the caller's locale variables,
and git translates its messages,
while the corpus reader tells a missing page from another failure by git's English words.
By the agent's probe with git 2.55 over a throwaway repository,
reading a path absent at a held commit,
the refusal came in German under `LANGUAGE=de` with an English `LANG`
and under `LANG` or `LC_MESSAGES` set to German,
and in English once `LC_ALL=C` was added to any of them;
that the reader then classes a missing page as another failure
is the agent's inference from `classifyCorpusReadFailure`.

The fix:
the environment sets `LC_ALL` to `C` after inheritance.
`corpus-git-context.unit.test.ts` asserts the environment whole,
and runs real git under a German locale through `runKeyless`,
reading the English refusal back,
beside a control showing the same read translated without the hold;
the lead's run of it on the red tree showed `LC_ALL: 'de_DE.UTF-8'` kept
and the refusal printed as `Schwerwiegend: Pfad ... existiert nicht`.
The corpus entry barrel exports `corpusGitEnvironment` for that case.

Open to the owner's veto:
the real-git case depends on the host's git translating;
on a host without German messages its control fails,
not the guard.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B369: a clone directory that is itself a link escaped git's search ceiling

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its third finding,
and measured with git 2.55 by the agent porting it.
`GIT_CEILING_DIRECTORIES` held the parent of the clone directory as named,
while git compares the list with its real working directory,
so a clone directory that is a link to a plain directory inside a repository read that repository's page.
By the agent's run on the port build,
the corpus-source case on such a link fails,
the read returning where it should refuse;
and at the environment level,
before the interface changed,
the ceiling held the scratch directory where the case expected the link's real parent.

The fix:
`gitSearchCeiling({ cloneDir })` in `corpus-git-context.ts` returns the parent of the clone's real path,
and falls back to the parent of the path as named only for `ENOENT`,
`ENOTDIR`,
`EACCES` and `ELOOP`,
each of which ends git at its own `-C` with exit 128,
by the agent's probe;
any other failure of `realpath` is rethrown.
The keyless scan requires every spawn to build its environment inline,
which a first asynchronous version failed,
so `corpusGitEnvironment` stays synchronous and takes `{ searchCeiling }`,
which every read in `corpus-source.ts` resolves before its `try`.
A new case pins the fallback for a clone directory that does not exist,
and the TSDoc sentence saying git resolves a link in the list is corrected.

Open to the owner's veto:
The fallback covers four codes;
the alternative is the review's,
`ENOENT` alone,
where the agent measured the other three ending git at its own `-C` as well.
`unreadable-clone` covers an enclosed directory,
and now also a clone directory that is a link to one;
the report names no alternative.
A failure of `realpath` outside the four measured codes surfaces as a fault at exit 5,
not as `unreadable-clone`;
the report names no alternative.
`corpusGitEnvironment` takes `{ searchCeiling }` and `gitSearchCeiling` is exported;
the alternative is the merge plan's note to pass `{ cloneDir }`,
which this supersedes.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B370: a parent path holding a colon voided git's search ceiling

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its fourth finding.
Git splits its ceiling list at colons,
so by the agent's probe a plain directory under `a:b` read the enclosing repository,
and a page absent there read as git's words for a missing object;
a link to that directory keeps the colon in its real path,
and a real clone under `c:d` reads correctly without the ceiling.
By the agent's run on the port build,
the case reading a page,
its bytes and a listing under such a parent fails,
each read returning where it should refuse.

The fix:
`gitSearchCeiling` throws a `StatedRefusalError` before any git call
when the real parent path holds a colon,
naming the clone and the parent,
and saying to move the clone to a directory whose path holds no colon.
`doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) says so beside
`TRANSLATION_REPAIR_CORPUS_CLONE_DIR`.

Open to the owner's veto:
The refusal says to move the clone only;
the alternative is the review's remedy,
to move or link it,
where by the agent's measurement a link keeps the colon in the real path.
A clone whose real parent path holds a colon is refused as stated,
working clones there included,
by the agent's measurement;
the report names no alternative.
The paragraph in `doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) goes
beyond what the review asked and was kept,
lint clean;
the alternative is to drop it.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".
