# Audit ledger: process mistakes, launch checks and cache versions

What went unchecked before a run launched:
the full suite,
the lint,
the cache versions,
the handover,
the log's two streams,
and an audit that read nothing without saying so.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## M19: a suite run against a stale build after a mutation was restored

Status:
caught the same hour,
2026-09-27 (E1).
The package's `test:unit` task runs the tests against `dist` without building;
only `buildAndTest` builds first.
After a mutation check the source was restored with `git checkout --`
and the suite launched through `test:unit`,
so it ran against the mutant's build and reported its two guards failing.
Prevention:
every suite run goes through `buildAndTest`,
and so does any single test file run after a source change,
through `mise run build` first.

## M25: floors and sheets changed a cached stage's decisions with no cache version moved

Status:
corrected in `66703994a`,
which moved the translate cache to 15 and the consolidation cache to 20,
with the repair cache's version 33 account naming what rode inside it,
2026-09-27.
The translate and consolidation versions landed at 06:34 UTC (`bd98bdc70`);
after them the F-series floors,
the Han residue floor,
the sheets reading the declared names and house rules,
the reachable quorum and the A17 signer floor all changed what those stages ask or accept,
and none moved a number;
the repair lane had four more such changes after its version 32 (07:34 UTC).
No harm reached a page,
and only by luck:
no run cached a slice after 04:26 UTC that day.
Each version file says the bump is enforced by nothing and was missed before.
Prevention:
a commit that changes a floor,
a sheet,
a threshold or a settlement rule in the translate,
consolidation
or repair path names in its message which cache version it moves,
or why none moves;
and before any run launches,
the versions are checked against `git log` since each one last moved.

## M28: the M25 correction checked three of six cache versions

Status:
corrected in the commit that moved the lane contest to 6 and pairing to 3,
2026-09-28.
M25's check walked the translate,
consolidation and repair versions and stopped there.
The lane contest (version 5 since 2026-08-29) kept its number through changes to its sheet,
its eligibility floor,
its windows and its quorum,
and its ballots were cached under version 5 on fifteen days of changing sheets;
pairing (version 2 since 2026-08-29) kept its number through windowed rounds and bench-sized quorums;
refine (version 5 since 07:34 UTC on 2026-09-27) carried house-rule changes with no account.
Found while accounting for K13,
whose house-rule line reaches every one of those sheets.
The page-assembly Canadian pass is outside every key,
but the prose ranges it shares with the Han-residue floor are not;
that too was nearly missed.
Prevention:
M25's pre-launch check lists every `*CACHE_VERSION` constant in `src`
(`rg 'CACHE_VERSION[A-Z_]* = ' src`),
not the ones remembered,
and for each one runs `git log` since it last moved over every file its stage's sheet or floors import,
the shared house rules and prose ranges included.

The first pre-launch check under this rule,
2026-09-28,
run by `cache-account-audit.ts` over all six constants:
translate 15 and consolidation 20 set in `66703994a` (02:56 UTC on 2026-09-28),
repair 33 in `f2cd70ece` (02:41 UTC on 2026-09-28),
refine 5 in `30e66051e` (07:34 UTC on 2026-09-27),
lane contest 6 and pairing 3 in `d614a0c1d` (04:30 UTC on 2026-09-28).
Every one was set after the newest slice-cache file under the agent runs (04:26 UTC on 2026-09-27),
and none has been written since:
`TZ=UTC find ... -newermt '2026-09-27 04:27'` gives 0,
and its control at 04:20 UTC gives 18.
So no answer cached under an earlier question can be served under any current number,
and no version moves.
Of the 153 non-test source commits since the earliest of them,
37 are named by an account and 116 by none
(the script prints each with its subject and the versions set before it);
they ride inside every version by the same fact,
and each version's TSDoc now says so,
dated.
Rerun 2026-09-28 over seven constants,
with the page title lexicon's `PAGE_TITLE_CACHE_VERSION` 1 (`ea61cbd4c`)
added to the script and the repair version now 34 (`ef20e7978`):
`TZ=UTC find ... -newermt '2026-09-27 04:20'` gives the same 18 files,
all by 04:26 UTC that day,
and none since,
so every number still holds what it names.
The page title lines and X20's heading pairs reach the slices through the identity context,
which all five slice keys hash,
and never reach a pairing sheet,
whose key does not carry them.

The script moved into the package on 2026-09-29 as the read-only mise task `cache-account-audit`
(`corpus-run/cache-account-audit.ts`),
which reads every constant ending in `CACHE_VERSION` out of the tracked source instead of a typed list,
finds each setting as the newest commit that added the declaration on balance,
reads diff lines with the same declaration reader so a longer name or value holding one is told apart,
and compares times in unix seconds.
On that day's history it gives the script's answer:
the same seven settings,
353 source commits since `30e66051e`,
and the same 295 hashes named by no account.
Its readers have their own unit tests,
and 12 mutants of them are caught with a comment control surviving.
Moving it in found one gap:
the page title version had no account,
and its TSDoc said a span detector change moves it,
while `d9a306602` and `ee39e2ba5` changed the detector and left it at 1.
The key hashes every detected title with its count,
so a changed detection is a changed key;
the TSDoc now says only the sheet's wording and the choice move it,
names both commits as riding inside,
and records that no slice-cache file has been written since the value was set,
where a control time finds 494.

## M37: a key component added from a flag's name

Status:
happened 2026-09-28 in X11 (`c5ac38345`);
reverted in `6d9b361b4`,
whose guard pins the key unmoved.
`checkerSelfCertificationPermitted` was folded into the repair run shape
on the reading that it "decides who may check a refinement".
Every reader hands it to `assertCheckerIndependence`,
which only refuses a roster before anything is bought:
a roster it admits is asked and weighed the same either way,
and the checkers are keyed already.
The readers also default an absent flag to refused while the fold read only an explicit `false`,
which is how the mistake surfaced.
Prevention:
before a field goes into a key,
list every reader of it (`rg` for the field)
and name the question or weighting it changes for a run that goes ahead;
a field that only admits or refuses the run stays out,
with a comment saying why.

## M57: a cache-file check whose time this host's `find` refused, with its error discarded

Status:
happened 2026-09-29 during B22,
caught by its control before any account used it.
The rides-inside check for the declared-name fix ran
`find <runs> -path '*slice-cache*' -type f -newermt '2026-09-26 20:00 UTC' 2>/dev/null`
as its control and printed a count of zero.
This host's `find` is bfs,
which takes ISO 8601 times only and refuses `… UTC` as an invalid timestamp;
with its error discarded,
the refusal read as no file.
The control's zero,
where files had to exist,
exposed it,
and the checks rerun as `-newermt '2026-09-26T20:00:00Z'` found 495 files for the control
and none newer than the consolidation entry of 04:26:57 UTC on 2026-09-27.
Prevention:
step 4 of the pre-launch list in `mistake-prevention.md` names the ISO form,
and a check's error output goes to a file that is read,
never to `/dev/null`.

It recurred on 2026-09-30 (UTC),
checking that B42 rides inside the translate version:
`-newermt '2026-09-27 04:27:00 UTC'` printed zero,
and so did its control from midnight.
The error went to a file this time,
and the control's zero sent the check to it;
the ISO rerun found no file after the cutoff and 450 since 00:00 UTC that day.
The same form found 494 on 2026-09-29,
over the agent runs and the package's own runs directory both;
this paragraph inferred that 44 files of that window had left those roots,
and the control read 494 again on 2026-09-30
(M81),
so the inference does not hold.
A written step and a read error file did not stop the slip,
so the check itself is queued for `cache-account-audit`:
report the newest slice-cache file and a control count,
so an account needs no hand-written `find`.
It reports both since 2026-10-01 (UTC) (M81).

## M81: a cache-file count taken over a root holding a throwaway checkout and a copy payload

Status:
happened 2026-09-30 (UTC) during B43,
caught by listing the matches before any account used the count.
The rides-inside check for B43 ran over the agent runs,
where a throwaway worktree for B44's before-measurement had just been checked out
and the worktree-copy hook,
which ran out of memory,
had left a payload of copied run directories
(`~/temp/agent/.cli-git-worktree-copy-kwEhzq/payload/`).
`find -path '*slice-cache*' -newermt '2026-09-27T04:27:00Z'` printed 332 where every check since 2026-09-28 printed 0:
9 were the checkout's own source files named for the slice cache,
the rest were copies of cached records with fresh times,
and outside both the only match was a scratch output.
The same check had also named the package's runs as `package/module/translation-repair/runs` (split on 2026-10-18 into
the packages under `package/translation-repair`),
which does not exist
(bfs said so in the error file);
they live under the worktree's `node_modules/.monochromatic/translation-repair-runs*`,
where nothing was written after the cutoff or since midnight that day.
The control from midnight read 494 once the confounds were set aside,
the count of 2026-09-29,
so the 450 of the B42 check was not files leaving those roots,
as M57's recurrence paragraph inferred;
what the B42 check counted differently is not recorded.
Prevention:
a count over a root lists its matches before it is used,
names each root by where the runs are written (`runs-layout.ts`),
and leaves out checkouts and copy payloads;
`cache-account-audit` is to report the newest cache file and the control itself,
so an account needs no hand-written `find`.
It does since 2026-10-01 (UTC):
after the version accounts it prints how many slice-cache records it read under how many runs directories,
naming where it looked
(the default runs directory's parent,
the configured runs directory,
and each `--runs-under` directory,
searched four levels down outside hidden directories and dependency trees,
so a copy payload is never read),
then each directory that search could not list with the filesystem's reason
(its first run over the agents' scratch root stopped at an `EACCES`,
so a locked directory is now named as left out of the count instead of ending the audit),
then the newest record by its full path and time,
and how many cache versions were set after it.
The copy that ran out of memory and left its payload is issue #580,
filed against `cli-git`,
which owns the hook.

## M147: a launch whose stdout overwrote every line its stderr wrote

Status:
happened on 2026-10-11 (UTC),
by the lead's launch of TianqiChen66622;
the lines it lost cannot be recovered.
The pass ran as a systemd user unit given `StandardOutput=truncate:` and `StandardError=append:` on one file:
two descriptors,
stdout's writing at its own offset,
so every line stderr appended was overwritten.
The log kept 9,942 `info` lines and 2 `warn` lines
(the last two,
written after stdout's final write),
against TianqiChen66621's 101 `warn` lines written through a shell redirect;
every retry,
lost voice and failure message went with the rest,
among them whatever Bedrock answered the two Gemma seats on all 536 of their calls.
A read-only agent then read the silence as a pipeline defect ("no log line names the thrown message").
A control unit writing three lines to stdout and two to stderr kept only the three that way,
and all five in order with stderr left to inherit stdout's file.

Prevention:
`mistake-prevention.md`,
"Before a run launches",
step 8:
one descriptor for both streams,
read back from systemd right after the launch
(`StandardError=inherit`,
where the faulty form shows `StandardError=append`;
both shown on throwaway units the same day).
The runbook's launch step now gives the unit form.
The lead's named-test units drop their `StandardError=append:` part the same way.

## M278: a rotation launch handed its unit the session shell's whole PATH, and its build ran the main checkout's bundler

Status:
happened on 2026-10-19 (UTC),
at the launch of `CuspariaKLSY16`,
the second entry of the readiness rotation;
stopped at its `START` line.

The lead launched the unit with `--setenv=PATH="${PATH}"`,
a flag the runbook's unit form does not carry.
The session's shell was started in the main checkout,
so its `PATH` led with that checkout's `node_modules/.bin`,
which held `rolldown v1.2.14`;
the worktree's lockfile installs `1.2.13`,
which built `zheermao14`.
The build ran `1.2.14`,
whose bundle wraps CommonJS modules with a third argument
(`cb((mod={exports:{}}).exports,mod,mod.exports)`,
and `acorn`'s UMD wrapper with it),
so `corpus-pass.mjs` moved from `9a4ffb21…` to `dcb2f0cd…`
and `START` printed `pipeline=sha256-tree-v1:475fbdbb…`
against the rotation's `a92725968adf…`,
with no family source changed between the two launches.
The lead read the digest off `START`,
stopped the unit after five `SPEND` lines,
diffed the two bundles,
and found the version on the build lines (`✔ rolldown v1.2.14`).
`CuspariaKLSY17` relaunched the entry with the main-checkout entries removed from `PATH`
and the worktree's `node_modules/.bin` first:
`rolldown v1.2.13`,
and `START` read `a92725968adf…`.

Prevention:
`mistake-prevention.md`,
"Before a run launches",
step 9;
the runbook's launch step names the hazard and the two lines to read back.
The pipeline digest did its job:
it is over the built files,
so a bundler that changes the output changes the generation,
and the run was refused as evidence before its page existed.

Guard,
landed the same day:
red `2640c1c4e1`,
fix `0fa3120174`.
`checkoutBundlerPlugin` (`package/config/rolldown/src/checkout-bundler.ts`)
fails a build in `buildStart`
when the running bundler's `this.meta.rolldownVersion`
differs from the `VERSION` the config resolves from `rolldown`,
which Node finds from the config package's place in the building checkout.
`familyLibraryNodeConfig` carries it,
every family config lists that build first,
and the bundler's command line stops a list at its first failed build,
so the runner builds stop too.
A build in a scratch worktree with the main checkout's `node_modules/.bin` first on `PATH`
refused with `This build is running rolldown 1.2.14, but this checkout installs rolldown 1.2.13.`;
after that worktree was reinstalled at its own commit,
the branch worktree's `1.2.13` first on `PATH` was refused the same way,
and the whole family built under the worktree's own `PATH`
(31 builds,
each `rolldown v1.2.14`).
The guard's refusal case was shown to fail with the comparison disabled.
The branch worktree keeps its frozen build until the rotation ends,
so the rotation's launches still rest on `launch-rotation.ts` and the `START` digest.
What the guard cannot see:
an install that is itself stale against the checkout's lockfile (M280).

## M280: a scratch worktree tested a commit on the install of another

Status:
happened on 2026-10-19 (UTC),
while the bundler guard for M278 was built;
caught before anything landed.

The lead's recipe for a scratch worktree ran `setup-worktrees.ts`,
which creates the worktree at the branch worktree's `HEAD` and installs there,
and then checked the worktree out to the commit under test.
`w206` was installed at `2eeef91fac`,
whose lockfile pins `rolldown 1.2.13`,
and checked out to `37e5397b41`,
whose lockfile pins `1.2.14` (moved on `main` by `f74a7268f4`).
The guard's tests and the first end-to-end builds ran on the stale `1.2.13`,
and the refusal the lead first read,
running `1.2.14` against an installed `1.2.13`,
was the main checkout's current bundler against a stale install,
not against the commit's own pin.
The lead found it while checking which version each lockfile pins,
reinstalled `w206`,
and repeated the tests and both end-to-end builds on `1.2.14`.

Prevention:
`mistake-prevention.md`,
"Before a run launches",
step 10;
`setup-worktrees.ts` now takes `<name>=<commit>`,
installs after the checkout,
and prints the installed `rolldown` version.

## M284: the cache version commit left seven test cases red, because the check after it ran the declaring packages only

Status:
happened on 2026-10-20 (UTC),
in the main session,
at the bump of the sheet-backed cache versions for the fixes of XingZ6015.
`28f4b8fc75` (01:34 UTC) moved the versions;
`5db53c0056` (01:43 UTC) moved five pinned keys,
and `a169580740` (01:54 UTC) moved two version pins in corpus-pass.
A corrective commit comment on `28f4b8fc755c0b9fcc8bd00b7b9f211d946c67b3` is owed,
since commits are never amended.

The commit moved seven constants in six packages
(`TRANSLATE_SLICE_CACHE_VERSION`,
`PAGE_TITLE_CACHE_VERSION`,
`CONSISTENCY_CACHE_VERSION`,
`LANE_CONTEST_CACHE_VERSION`,
`REFINE_CACHE_VERSION`,
`SLICE_CACHE_VERSION` and `CONSOLIDATE_CACHE_VERSION`).
Each version is key input,
so a case that pins a key or the number itself fails at once.
Its message does not say it leaves any case red.
Seven were red:

- five pinned keys in the declaring packages,
  moved in `5db53c0056`:
  translate-lane,
  lane-contest,
  two in repair (the refine key and the repair slice case),
  and artifact;
- two version pins in corpus-pass,
  the consistency key case (version 5,
  now 6) and the page title version case (3,
  now 4),
  moved in `a169580740`.

The check after the bump ran the packages that declare the versions,
and corpus-pass is not one of them.
Each of the five declaring-package cases was read red from its own failure after the bump,
and `5db53c0056` moved them in a commit of their own.
The two corpus-pass cases name `CONSISTENCY_CACHE_VERSION` and `PAGE_TITLE_CACHE_VERSION` in a package that declares
neither,
so that check never ran them;
`a169580740` moved them,
in the commit that also gave the contest seam its `fanOut` (B701).
The message of `5db53c0056` says its cases were left red by the version commit before it;
the corrective comment says seven,
and names both commits that moved them.

Prevention:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Cached decisions":
after a version moves,
every package whose tests name the constant runs too.
The search is `rg --files-with-matches '<NAME>' package/translation-repair --glob '*.test.ts'`,
and on `a169580740` it finds the two corpus-pass files for `CONSISTENCY_CACHE_VERSION` and the one for
`PAGE_TITLE_CACHE_VERSION`.

## M285: the branch worktree moved to a new rotation build without an install, and the pass could not load a new package

Status:
happened on 2026-10-20 (UTC),
in the main session,
at the first launch of `zheermao17` on `5ae2173d23` (18:58 UTC);
no provider was called,
and the relaunch at 19:16 UTC started on the gated build.

The branch worktree was fast-forwarded from `a5dc23c5bb` to `5ae2173d23` with `git merge --ff-only`,
and nothing installed there.
The handle fix of B707 added `pinyin-pro` to rendering-audit,
so the worktree had no copy of it,
and the bundler,
unable to resolve it,
left it as an import in `corpus-pass.mjs`.
The pass stopped at load with `ERR_MODULE_NOT_FOUND` for `pinyin-pro`,
before its `START` line,
and the launch script reported no `START` line after its fifteen minutes.
The gated build in `w241`,
installed at its own commit,
bundles the package and holds no such import.
After `mise run prepare:pnpm:install` in the branch worktree
(lockfile unchanged,
no tracked file changed)
and a forced build of corpus-pass,
the branch build's digest was the gated build's
(`sha256-tree-v1:ee83b531c14ff55286558920e0b21a0d5c43fda1325ffc4ede74c1bec7d6e2ef`),
and the relaunch's `START` line carried it.
The failed attempt's log is kept as `corpus-pass-zheermao17-nostart.log` in the agent scratch directory.

Recurrence:
of M280,
whose rule names every checkout;
the branch worktree was read as outside it because it is moved by a fast-forward,
not checked out.

Prevention:
`mistake-prevention.md`,
"Before a run launches",
step 10 now names the branch worktree,
the install after its fast-forward,
and the digest of its build against the gated build's before the launch.

## M286: a new workspace package landed without a file-enforcer run, and the registry config left it out

Status:
found on 2026-10-22 (UTC),
in the main session,
when `mise run file-enforcer` ran in the scratch worktree `tr-w275` for the WUC rule in `AGENTS.md`;
fixed in `1c10d0b176`,
landed in `65e8325303`.
No run read the file,
so no pass was touched.

The katex shim (`package/shim/katex`,
added in `47b4634eeb` and landed in `c0d0325899`)
is a new workspace package,
and file-enforcer derives the private registry's config,
`package/config/pnpr/config.yaml`,
from the workspace's packages.
Nothing ran file-enforcer before that landing,
and the gate runs no step that would,
so the config kept its published count at 163 and left `@monochromatic-dev/shim-katex` out of its package list.
The run for WUC rewrote it to 164 with the shim listed.
The same run also rewrote the `_.path` list in the root `mise.toml`,
dropping the `node_modules/.bin` folders of packages the scratch worktree had not installed;
that rewrite follows the worktree's installs,
not the workspace,
so it was saved as a patch in the agent scratch directory and discarded,
not committed.

Recurrence:
none recorded in this package;
the repository's rule names file-enforcer's outputs,
and nothing in the landing steps ran it.

Prevention:
`mistake-prevention-tasks-builds-and-bulk-output.md`,
in its one section:
a change that adds,
removes or renames a workspace package runs `mise run file-enforcer` in its worktree before it lands
and commits what that run writes to files derived from the workspace,
reading the `mise.toml` `_.path` rewrite against the worktree's installs first.

## B841: the cache-account audit printed an account of no record without a word, over roots that hold no launch's runs

Status:
fixed on 2026-10-23 (UTC),
red `f4519c3d29` (its cases read the log through the level-keeping logger in `a420726b63`),
fix `c49da916d4`,
checklist and README `4be243b6b9`.
The reading is the lead's and is open to the owner's veto.

What was found:
the lead's run of the audit on 2026-10-23 at 10:31 UTC read 0 records with its default roots,
and 13,711 records in 419 runs directories with `--runs-under` the runbook's runs root under the home directory.
The newest record there was written at 2026-10-22T19:14Z,
and none of the 8 cache versions was set after it.
Step 4 of the checklist in `mistake-prevention.md` (M25 and M28) ran the audit without `--runs-under`,
while a launch that follows `doc/runbook/translation-repair-corpus-pass.md` writes its runs to
`${HOME}/temp/agent/corpus-pass-<entry>`.
The account of no record printed `newest slice-cache record: none` and nothing else,
so a reader took an empty account for an empty cache.

What changed:
- `reportSliceCaches` (`slice-cache/src/corpus-run/cache-account-slice-report.ts`) warns through the package logger
  when it read no slice-cache record,
  and prints a `WARNING:` line with the same sentence on stdout.
  The sentence says that the newest-record and versions-set-after lines then say nothing of the caches a launch would
  resume,
  names every root searched and the runbook's runs root,
  and gives the `--runs-under` argument that reads it.
  The logger and the home directory are optional arguments,
  so the cases inject both.
- Step 4 passes `-- --runs-under "${HOME}/temp/agent"`,
  requires a count above zero on the `slice-cache records under N runs directories` line,
  and reads the audit's own `newest slice-cache record` and `cache versions set after it` lines in place of the
  hand-written `find` (the audit has printed both since 2026-10-01).
  The README of `slice-cache` says the same and documents the warning.

Cases:
three in `cache-account-slice-report.unit.test.ts`,
which fail at the parent for lack of the warning.
Two scratch mutations,
one dropping the log call and one dropping the stdout line,
each fail two of them.

The rule for next time:
a census that reads nothing says so in words,
and a check that passes on zero input requires a count above zero.

Cache account:
the audit reads versions and writes none,
so no cache version moved.

Later,
the same day:
the warning fired on a count of exactly zero,
and a count above zero is no proof that the roots searched hold the runs a launch writes.
The default roots read the worktree's own runs directory and any `translation-repair-runs*` beside it,
so one stale record there gives a non-zero count that says nothing of `${HOME}/temp/agent/corpus-pass-<entry>`,
the audit stayed silent,
and step 4 of the checklist passed on "count above zero".
`reportSliceCaches` now also warns,
in the log and on stdout,
whenever the runbook's runs root is neither among nor under the roots searched,
whatever the count,
and names every root searched and the runbook's root;
the sentence of the empty case is unchanged and is the only one a zero count prints.
A root searched that lies below the runbook's root covers its own runs and not the rest of it,
so it does not silence the warning.
`auditCacheAccounts` takes the home directory as an optional argument,
the process's own by default,
so its cases state one.
Red `b3ad54afb3` (two cases fail at the parent;
the case with the root among or under the searched roots and the case of the empty count with the root searched pass
there as controls),
fix `f9c6ddde3f`.
Step 4 now puts the whole command,
`--runs-under` included,
in one code span,
and says what a `WARNING:` line beside a non-zero count means.
The audit reads versions and writes none,
so no cache version moved.

Later,
fix round 4:
the warning compared spellings and ignored the depth of the walk.
The home directory as the process reports it and its real path are two spellings of one directory
on a host where the home directory is reached through a link,
and the comparison read a link as an uncovered root,
which fails safe.
The converse did not fail safe:
a searched directory more than four levels up from the runs,
where the walk of runs directories stops,
read as covering them.
`cache-account-search-reach.ts` now decides it.
Both paths are resolved through their real path,
an absent path through its nearest existing ancestor,
and a root reaches a directory only when the directory is the root or lies below it,
no name between them is one the search never enters (hidden,
`node_modules`),
and the runs two levels below the directory sit within the four levels the walk reads.
The warning says so in its words ("is not reached by the roots searched").
Red `b93b20ef45` (two cases fail at the parent:
the link case warns where it must not,
and the deep case does not warn where it must),
fix `6d4f2cd78b`.
The audit reads versions and writes none,
so no cache version moved.

Later,
fix round 5:
the reach check ended the audit for a root the filesystem refuses to resolve.
`searchReaches` raised the filesystem's error for a root behind a file (`ENOTDIR`),
a loop of links (`ELOOP`) or a directory nobody may enter (`EACCES`),
while the walk of runs directories reports such a root as one it could not list and goes on,
so the audit stopped with a stack trace before it printed the newest record and the versions set after it,
the two lines it exists to print.
`reportSliceCaches` now checks each root on its own:
a refusal is logged as one warning with the root and the filesystem's code (never its message,
which quotes the path),
the root counts as not reaching the runbook's runs root,
and the warning on stdout ends by naming the roots it could not resolve.
A root that does reach still settles the answer,
so a refusal beside it leaves a line in the log and no warning on stdout.
`searchReaches` itself still raises for these roots and says so in its `@throws`.
Two edges went with it.
The path is no longer folded by name before it is resolved,
so `base/lnk/../d` goes to the parent of the link's target,
as the kernel and the walk read it,
where it read `base/d`.
A root equal to the directory now obeys the depth rule,
so runs five levels below it are not reached,
as the walk stops at four.
The branch for a relative path that is absolute was removed;
the module states that it takes one filesystem root.
Red `3050fca34f` (six cases fail at the parent:
four in the report and two in the reach module;
two cases of the reach module pass there as guards that the refusals still raise),
fix `93990e5e0d`.
The audit reads versions and writes none,
so no cache version moved.

Later,
fix round 6:
the reach check changed as follows.
The log line of a root the filesystem refuses to resolve now ends with the failure's text,
where fix round 5 kept the code alone on the ground that the message quotes the path:
the line already prints the root and the runbook's runs root,
the text of a failed real-path lookup names the path the lookup stopped at,
which tells a reader which file or link on the way is to blame,
and the logger takes one string,
so the caught value is rendered into the line.
The printed warning still names the code alone.
Red `a51fe1ee88` (four cases fail at the parent),
fix `f9529cb472`.
A runbook runs root that cannot be resolved is named once.
It used to fail every comparison,
so the audit logged one line per searched root,
four with the default roots,
and the warning listed each as a root it could not resolve.
The runbook's root is now resolved before any comparison;
a failure logs one line with the root and the failure's text,
the warning says the roots searched could not be compared with it,
and the newest record and the versions set after it are still printed.
Red `8410bf72fb`,
fix `23bfbdf076`.
The readings moved from the report module into `cache-account-search-reach.ts`,
beside the walk,
because the report module passed the line limit.
The ternary of `realPathOf` that joined a relative path onto the working directory by name was dropped (`578429910d`):
`realpath` resolves a relative path against the working directory itself,
and a probe over ten relative paths in a disposable directory answered the same as the joined path in every case,
the successes and `ENOENT`,
`ENOTDIR` and `ELOOP` alike,
so no case can tell the two apart and the commit has no red.
`RootReading` is now named members and a one-line union (`b90f416665`).
The audit reads versions and writes none,
so no cache version moved.
