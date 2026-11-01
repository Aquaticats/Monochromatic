# Translation repair handover 2026-09-06: the checkpoint of 2026-10-18, the package split, the shared lint analysis and the rounds after it

The checkpoint of 2026-10-18:
the split of the package into `package/translation-repair/<name>`,
the shared lint analysis that answered why lint was slow,
and the rounds that landed after it up to the ledger round `w197`,
kept as those sessions wrote it and not as the current state.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## 2026-10-18: the package split moved, its path readers are being fixed

The owner asked on 2026-10-18 why lint is slow and to split the package into `package/translation-repair/<name>` first,
before the other queued work.

### State

- At the split's planning,
  branch head `8a6b27f976`,
  pushed.
  n171 (`0199690299`) and n173 (`ff6be38eef`,
  `43428948cb`,
  `f9169093e6`) landed;
  `94ebefa1e3` listed the three blankness tests they added.
- The pre-split controls were taken at `94ebefa1e3` in `w165`
  (`s245/control/summary.txt`):
  build and blankness scan green,
  the TianqiChen66625 decisions replay byte-identical to the n171 build's apart from its label
  (`s245/control/replay-presplit.json`).
- The lint answer and the module map are in `doc/planning/translation-repair-package-split.md`
  (`fb3c55b0de`,
  amended in `8a6b27f976`),
  with the map in `doc/audit/translation-repair-package-split-map-2026-10-18.json`:
  warm whole-package oxlint 24 s,
  a 31 to 120 file run 3.0 to 3.7 s,
  34 packages of 4,055 to 23,968 lines,
  acyclic.
- The first auto-push of `fb3c55b0de` and of `8a6b27f976` was refused with
  `GH006: Protected branch update failed` and `protected branch hook declined`;
  a plain `git push` straight after went through both times.
  The branch's rules are a Copilot review ruleset and classic protection with no required checks,
  so the cause is not known.

- Since then,
  pushed:
  `09d9805ddf` splits `corpus-run/cli-refusal.unit.test.ts` into one file per runner package;
  `03c0de897a` and `d0810bb6df` add `familyLibraryNodeConfig` to `package/config/rolldown`
  (a family member's library build keeps its siblings as imports of their built entries);
  `2e79bef413` records the pilot and the readers that find a file by path in the plan,
  and moves `build-entries.ts` to `coverage-census` in the map.
- The pilot (`markup` and `test-fixture` moved out,
  in `w165`) built and passed every case
  but those of code that resolves a family file by path;
  the plan's "Readers that find a file by path" section lists that code.
- The split's scratch scripts are in `s245/split/`:
  `own.ts` (placement and the move),
  `scaffold.ts` (package files),
  `literal-paths.ts` (file names held as text),
  `full/full-run.ts` (a whole dry run in a keyless worktree).
- Landed since,
  pushed:
  the semantic-wrap config travels with the build (`3d844e0315`,
  `76eb00d8fc`;
  gate g100 at `76eb00d8fc` green,
  suite 2484 passing,
  the suite baseline for the split);
  `familyLibraryNodeConfig` takes `extraPlugins` (`2083d7bec1`);
  the move (`4bd1aea8d5`,
  36 packages);
  the lockfile and file-enforcer outputs (`bbd121d7c1`).
- At `4bd1aea8d5` every package with production code builds;
  tests fail in 9 packages,
  all in readers that find a file by path
  (94 test files,
  `s245/split/full/failing-files.txt`).
- The old package's `node_modules/.monochromatic` (21 GB of logger files and the kept evidence
  the ledger cites) and its probe and candidate directories stay in place;
  only its `node_modules/.bin` was removed.
  Run state is unaffected:
  runs live under the worktree root's `node_modules/.monochromatic/translation-repair-runs*`.

### Next

1.  Three Sonnet agents fix the path readers,
    each in its own keyless worktree at `bbd121d7c1`,
    committing on a detached HEAD:
    `w167` the source scans and `typography-prose-mask`;
    `w168` the tests and fixtures that start a child over built files
    (adjudication,
    bench,
    corpus-pass,
    probe,
    run,
    two slice-cache tests);
    `w169` the coverage census and the cache-account audit,
    each with a before-and-after source-set control.
    Cherry-pick their commits onto the branch,
    then gate the merged head.
2.  Package READMEs landed (`704165269a` to `48ea1fc3d0`).
    `w167` landed (`2bd2d2ab4f` and the six before it):
    every scan reads every family package,
    keys are `<package>/<path under src>`,
    and `source-scans` depends on the four sibling builds its tests load.
    Left:
    `source-scan` lists `coverage-census` as a development dependency no scan imports
    (drop it with a lockfile regeneration);
    the move's import rewrite and scaffolding left oxlint findings in 30 of 36 packages
    (census at `w167`'s head,
    `s245/split/lint/census.txt`;
    every package builds and type-checks):
    526 `no-import-type-side-effects`,
    440 duplicate imports,
    40 TSDoc links to `import('@monochromatic-dev/...')` read as tags,
    12 arrow functions in rolldown configs,
    5 `index.ts` barrels over 300 lines,
    1 readonly parameter,
    and a `test-fixture` lint task that matches no file.
    A Sonnet agent in `w172` clears them in the 23 packages no other agent holds,
    with a dist byte comparison per package;
    adjudication,
    bench,
    corpus-pass,
    probe,
    run
    and slice-cache follow after `w168` lands.
    The move's dry run built and tested but never linted:
    a ledger entry is owed with the split's docs batch,
    with the other split mistakes collected in `s245/split/ledger-notes.md`.
    The per-package lint times in that census (2.5 to 37 s) were taken under four agents' load
    and are not the measurement the plan owes;
    `w169` landed (`a77972ac5c` and the twelve before it):
    the census measures every family package and reads the union of the six runner packages'
    entry lists;
    the cache-account audit reads every package's `src`;
    both source-set controls match (`w169-notes/control-output.txt`).
    It left:
    refusals and their tests still name `//package/module/translation-repair:coverage-census`
    and `...:cache-account-audit`;
    the audit's git log and pickaxe queries cover only the new paths,
    so a cache version set before the split reports the move commit as its setter
    (add the old `src` path to the pathspec);
    the census README is not updated for the family reading;
    `runBuiltCommand` and `runProbeOver` gained a `builtDirectory` option.
    `w168` finished (`32171e7931`,
    `8a55a9295a`,
    `161d80b27a`;
    its 53 test files pass) with a required `packageEntry` instead,
    and its first commit conflicts with `w169`'s.
    A Sonnet agent in `w173` cherry-picks it onto the branch head toward the required form,
    converts `w169`'s callers,
    regenerates the lockfile,
    and fixes the old task paths in refusals and the audit's pre-split history (tests first).
    `w168` added test-only workspace cycles
    (adjudication's tests start runners from bench,
    coverage-census,
    probe,
    run and slice-cache;
    run's from probe;
    probe's from bench):
    after the merge,
    decide whether those tests move to the packages whose runners they start;
    oxlint errors in `bench`,
    `probe` and `run` from the scaffolding.
    `source-scan` and `test-fixture` export `./ts` to an `src/index.ts` neither has;
    the `source-scan` README describes its readers as mid-move.
    The README agent named role wording it inferred from module names without opening the modules
    (`consistency`,
    `artifact`,
    `page-check` among them):
    a Sonnet agent in `w174` reads every claim of the 37 READMEs against the code.
    The docs move landed (`w171`,
    on the branch as eight commits ending `e6e41417d5`):
    131 docs in `package/translation-repair/doc/`,
    `pictures.md` in `slice-reading/doc/`,
    `semantic-wrap-sync-spawn.md` in `typography/doc/`,
    the old package directory and its `LICENSES/` gone from git
    (its ignored `node_modules` evidence stays);
    the docs scans read 169 files before and after,
    identical through the move table;
    22 ledger parts carry one header note instead of per-mention notes.
    Two comment conflicts with `w169` were resolved to `w169`'s text.
    It left
    (its one reported dead link,
    `doc/planning/translation-repair-toka-ls-reading-2026-09-02.md:207`,
    is an escaped bracket in quoted output that renders as text,
    not a link):
    pre-existing `monochromatic-lint` findings in 12 repository docs it did not change in substance
    (`w171-notes/lintlist-out.txt`);
    it amended one of its own unpushed commits,
    against the no-amend rule (a slip for the ledger).
3.  Integration in `w172` (detached,
    not yet on the branch):
    on the branch head `b030c7ca71` (the 23-package lint fixes landed and pushed),
    the base oxlint ignore narrowed to `**/package/test-fixture/**`
    (root lint still skips the fixture packages,
    the translation-repair `test-fixture` lints 46 files),
    `w173`'s nine commits (`w168` merged onto `w169` with a required `packageEntry`,
    refusals naming `taskPathOf` over `COMMAND_OWNERS`,
    the audit's pre-split history,
    tests first;
    three conflicts with the docs move kept the derived task paths),
    and `w174`'s four README corrections.
    A Sonnet agent continues there:
    the six `source-scans` findings in `w169`'s coverage-census code,
    the test-only workspace cycles removed by moving each runner test to the runner's owner,
    oxlint to zero in the remaining eight packages,
    the `./ts` exports of `source-scan` and `test-fixture`,
    `source-scan`'s unused `coverage-census` development dependency,
    and `run`'s `budget-sample` description.
    Then gate `w172`'s head and fast-forward the branch to it.
4.  Done:
    `w172` landed (36 commits ending `8d09e8e092`,
    gate g101 green);
    the controls match at `8d09e8e092`
    (suite 2506 passing against g100's 2484,
    every name accounted for;
    census and audit source sets;
    the TianqiChen66625 decisions replay;
    the corpus measure),
    recorded in the plan's "Measured after the move".
    The lint times are in the plan's "After the move,
    measured":
    one package lints faster than the old package did
    (`corpus-pass` 4 s warm and 35 s cold,
    against 25.5 s warm and 72.6 s cold),
    but the 36 packages one after another take 333 to 352 s.
    The whole difference is the `prefer-readonly-parameter-type` plugin:
    it analyses shared modules once per package project,
    and its one repository cache (256 MiB,
    4096 entries) overflows.
    With the bounds raised (`s245/split/lint/cost-probe4b.txt`),
    the warm family lint takes 70.93 s against 312 s;
    the family needs 8269 entries and 508 MB.
    The owner chose on 2026-10-18:
    raise the bounds to 1 GiB and 16384 entries
    (landed:
    test `dcf2c4886a` red first,
    fix and docs to `96a615549e`,
    pushed);
    and investigate whether projects can share one module's analysis,
    which would fix cold lints and gates too.
    The investigation is `doc/planning/prefer-readonly-parameter-type-shared-analysis.md`:
    992 files under up to 34 projects have the same analysis in every one.
    The owner then chose to prototype it in a scratch worktree with a verification mode,
    results back before anything lands.
    The prototype is on `w177`'s detached head (`1b0cdd8653`,
    unmerged);
    its results are in the proposal's "Prototype results" (`59f6acf347`):
    cold family lint about 131 s against about 322 s,
    warm unchanged,
    0 mismatches over 6856 verified shared hits.
    The verification mode's positive control passed (a tampered shared entry was reported,
    the clean rerun was not).
    The owner chose on 2026-10-18 to harden,
    then land,
    and to profile the remaining cold time after landing.
    The hardening is done (`6cab24de4c` to `6899ef69af` in `w177`,
    recorded in the proposal);
    the comparison over every package against the baseline bundle runs in `w177-notes/fullrepo/`.
    Landing path:
    first the hardening (schema bump,
    `projectSourceText` TSDoc,
    remove-the-guard checks on each fallback,
    fixtures for `export ... from` and `/// <reference types>`,
    and lint equality over every repository package against `w177-notes/baseline-oxlint-dist`;
    84 of 219 packages were compared),
    then cherry-pick `0f2f380e59` to `6899ef69af` in order onto the branch.
    Running `mise` in `w177` moved the `tools.rust` nightly in `mise.lock` (`nightly-2026-10-13` to `2026-10-17`);
    the agent reverted it each time,
    and no landing commit may carry it.
    The docs batch of item 5 landed (`065353a34f` to `8194356f4d`:
    B629 to B655,
    D34,
    M234 to M255).
    The scratch configs in `w165` and `w175` are deleted and `w165`'s plugin edit is reverted.
5.  The queue the split went ahead of,
    oldest first:
    the delegated items:
    the finalist extension (not started before 2026-10-18;
    the lead's veto-open rule that a lone leader without a majority is run off with the second-count labels,
    fixing the 2-1-1 regression the `TianqiChen66625` replay showed in n171)
    is done in `w180` (`c3d881805b`,
    `7123ffb2f8`,
    `114c8dbc52`,
    ledger B656;
    not yet on the branch):
    the artifact reader's recount now refuses a recorded group with a lone leader and no run-off,
    so whether any stored artifact is refused is being measured (`w180-notes/reader-check/`)
    before it lands,
    and the guard-failure demonstrations
    (missing for n171 and n168,
    unmapped for n173's `s244/guards/`)
    run in `w181` (notes `w181-notes/report.md`);
    the docs batch's leftovers (the batch itself landed in `065353a34f` to `8194356f4d`;
    `1939670f87` brought `design-commitments.md` up to the B619 landing and the B641 carry run-off;
    gate g102 on `1939670f87` is green,
    output `s245/split/gate-g102/`:
    145 steps exit 0,
    suite 2506 pass and 0 fail,
    the same as g101):
    n171 and n173 left no hand-back to source their breaches,
    and Z4 has no recorded fix;
    a `pi` sol review of the five rounds' landed production diff
    (`s245/sol-review/question.md` and `production.diff`;
    blocked at first:
    `pi` reported `No API key found for openai-codex`;
    the owner moved the review model to `openai/gpt-6.1-sol` (`CLAUDE.md`,
    `923e70ccf2`,
    2026-10-18),
    and that run answered with eight findings (`s245/sol-review/answer-gpt61.md`):
    span agreement across an apostrophe joint,
    mixed-pair quotation masking,
    a link labelled with a mark,
    a label ending in a space before an apostrophe,
    inline code padded with line endings,
    the identity-keyed label cache,
    a run-off on an unasked carry,
    and span-agreement TSDoc;
    each is reproduced red first and fixed in `w187` (on `bc9b2db70d`),
    ledger text to `s245/w187-notes.md`,
    then gated,
    landed and filed with the docs batch;
    the guard demonstrations for n168,
    n169,
    n171 and n173 finished (`w181-notes/report-tables.md`):
    of 287 mutations 64 still pass,
    three of them real behaviour changes no test sees,
    and a test round pinning every reachable survivor,
    each shown to fail under its mutation,
    runs in `w188`;
    an independent Sonnet review of the same diff found R1 to R7,
    report `s245/sol-review/sonnet/review.md`;
    R1 is confirmed in the code:
    the carry run-off sheet is byte-identical to the first sheet when every candidate
    is a finalist (`runOff` only renames the stage,
    `pass-consistency-carry-ask.ts:328`),
    and `promptUniqueClient` replays the first reply for a repeated digest,
    so the run-off never decides;
    the finalist extension makes that case commoner (2-1-1 over three candidates),
    so one fix round for R1 to R7,
    each red first,
    runs in `w184` on top of the extension's head `114c8dbc52`;
    the first stored-artifact check searched only the pooled runs directory (164 legacy artifacts,
    none with a
    consistency record);
    re-run over the five consistency-era TianqiChen666 run directories,
    the new reader refuses `corpus-pass-TianqiChen66624` group 0 (generation 17,
    2-1-1 of 4,
    no run-off),
    which the old reader accepts (`w180-notes/reader-check/results-throwaway-*.json`),
    so `w184` also bumps the artifact generation and recounts generation 17 and earlier under the tied-only rule;
    done in `w184` (`9684e3e00f`,
    `c351d52644`,
    `98d9089318`,
    ledger B657 to B664,
    generation 18,
    all five stored TianqiChen666 artifacts accepted,
    `w184-notes/reader-check.txt`);
    a follow-up triple there records the page definitions in generation 18 so the recount reads them (R5),
    gates the carry reader's "no run-off read as written" tolerance to generation 17 and earlier,
    and corrects a refusal string,
    done (`f5dc1482dc`,
    `7410c9948f`,
    `6939a15e33`,
    ledger B665 to B667,
    still 5 of 5 stored artifacts accepted,
    `w184-notes/reader-check-2.txt`);
    gate g103 on `6939a15e33` is green (145 steps exit 0,
    suites 2513 pass and 0 fail,
    `s245/split/g103/`);
    landed as `16ab4bf9c9` to `3eb276709b` (package tree identical to the gated head) and pushed;
    the cache-account audit at `3eb276709b` cannot attribute commits since the split moved every version constant
    (787 of 823 source commits named by none,
    `s245/split/audit-3eb2.txt`),
    so the cache account since `29d606470` (n171's code,
    where seven versions were last set) is read from the code
    in `w185` (`w185-notes/cache-account.md`),
    n173's fold change being the one with no recorded cache decision;
    it found no version that must move (every stored record is a judge reply keyed by its exact sheet,
    or a derived result whose producing packages changed only in formatting,
    checked by syntax-tree comparison with a positive control);
    the owner kept the finalist extension on 2026-10-18 (`design-commitments.md`);
    `TianqiChen66626` launched on 2026-10-18 at 20:59 UTC at `7c21a56fc2`
    (`--only TianqiChen666`,
    runs directory `/var/home/user/temp/agent/corpus-pass-TianqiChen66626`,
    log beside it,
    unit `corpus-pass-corpus-pass-TianqiChen66626`,
    streams truncate and inherit,
    plan ok,
    meters:
    Synthetic unreadable and treated as spendable,
    Bedrock,
    Hyper and OpenRouter wet);
    her page has no reference or footnote definitions (the pattern finds 158 in 67 other pages),
    so the open emphasis-definitions gap cannot reach this pass;
    it settled at 23:52 UTC the same day,
    with 24 slices in a page of 8,525 characters and no `DEFECTS` line
    (`verify-published`:
    every wording present,
    `read-at-generation=18`),
    and the read is in
    [the pass log for 2026-10-18](../planning/translation-repair-openrouter-2026-09-03-2026-10-18.md):
    no high-severity defect,
    the refrain and the repeat both carried (4 of 4 places and 2 of 2),
    her last message in her own present tense,
    and the most consistent of the pages of TianqiChen66624 to 66626,
    with a medium fault in the Uno sentence at the close (source line 87);
    no consistency run-off happened,
    since every first ask and carry had a strict majority,
    so the finalist extension the owner kept was not exercised
    (group 1 had exactly quorum,
    3 usable ballots of a bench of 4);
    both Bedrock Gemma seats were dead all run on an expired security token (HTTP 401)
    while every meter read `wet`;
    the Synthetic meter was unreadable in 126 of 126 readings,
    so 651 subscription calls ran with no allowance reading;
    and 58 Qwen3.8-27B replies were cut at the straggler deadline without showing in the seat census;
    the fix round `w191` landed after gate g106 (145 steps exit 0,
    suite 2532 pass lines and 0 fail),
    ending at `e9d720b816`:
    a seat whose credential gets HTTP 401 now says so once as `SEAT CREDENTIAL REFUSED`,
    the JUDGE SEATS line prints `unreadable-treated-as-spendable` for a meter that said nothing,
    and the SEAT line carries an `abandoned` column for replies cut at the straggler deadline.
    Quorum over the full bench of 8 follows the recorded reachable-quorum rule and is unchanged.
    The Bedrock token itself is the owner's to renew;
    nothing in the code can.
7.  Under way in `w186` at `7c21a56fc2`:
    emphasis read with the page's definitions in the three calls that pass none
    (`consistency-loose-marks.ts:340`,
    and the fallbacks in `consistency-marks-past-markup.ts:73`
    and `consistency-joined-marks.ts:76`);
    the cache-version TSDoc made consistent (a sheet change moves the key,
    not the version);
    the pairing version's range `d614a0c1d..29d606470`;
    the Synthetic meter reader for the provider's changed `/quotas` body.
    Done there (`074599761e` to `dcf55b5eae`,
    ledger B668 to B671):
    emphasis in the proof and the edge readers takes
    the page's definitions;
    the version TSDoc is consistent;
    pairing need not move (the pipeline digest retires its
    records);
    no public source gives Synthetic's new body,
    so only a note.
    A second round there (`54e693c569` to `67c9b1accb`,
    ledger B672 and B673) corrected a local time the agent
    labelled UTC (20:58 to 21:00 UTC,
    not the host's 16:58 to 17:00 EDT) and made the Synthetic warning name the received keys
    and JSON types,
    never values.
    An attempt at generation 19 that recorded the definitions moving the words of a place was built and withdrawn,
    never landed,
    so generation 19 later went to the stricter recount of `w189`:
    a definition cannot supply a delimiter outside the span,
    so on `Take *[*]a*’ now.` the words read with the
    recorded definition call the edge mark loose while the page calls it an apostrophe of its word,
    and a probe of 205,740 short strings found no case where reading with definitions is the faithful one.
    Decided 2026-10-18:
    the words of a place stay read alone (B673's first option),
    which planner,
    apply and
    reader already share;
    `withProvenApostrophes` takes definitions for the locator only.
    Gate g104 on `67c9b1accb` was green (every step exit 0,
    suite 2528 pass and 0 fail,
    scans pass),
    and both `w186` rounds landed on 2026-10-18 by cherry-pick,
    pushed as far as `ce7be13112`;
    the first round had not landed before,
    which the second round's conflicts showed.
    The sol findings round in `w187` reproduced five findings (1,
    2,
    5,
    6, 7) and fixed them test first,
    pinned 3 and 4 as regression cases (they do not reproduce),
    corrected the span-agreement docs (8),
    and moved the translate slice,
    consolidate and lane-contest cache versions for the tense candidates.
    Its stricter recount would refuse a generation-18 artifact that was right when written,
    so `w189` (on `e87411447d`) ports the round and gates that recount by a new generation 19.
    The `w188` test round pinned every reachable survivor (15 commits,
    tests only,
    on `95a57f628d`;
    notes `s245/w188-notes.md`);
    queued in order after `w189` reports:
    port `w188` onto `w189`;
    turn the four unreachable markup-range branches that silently return `[]`
    (`linkResourcesOf` past end and unplaced,
    a link with no offset,
    a node with no start)
    into named invariant throws,
    test first;
    one gate;
    land;
    post a corrective comment on the landed copy of `77823411c4`,
    whose body says six guards for the nine mutations it pins.
    All of it is done:
    gate g105 on `2445ccc7b0` was green (145 steps exit 0,
    suite 2532 pass and 0 fail),
    the round landed as far as `5c1d1aef1b` with the tree identical to the gated one,
    the corpus check found no page that reaches the new throws (184 texts),
    and the correction is
    commit comment 205562500 on `5a0fe85281`.
    The docs round filing `s245/w187-notes.md`,
    `s245/w188-notes.md` and the `w191` notes
    into the package ledger from B675 and M265 runs in `w193`;
    the `w190` notes follow in a later round.
    `w190` finished the per-snapshot request memo of the readonly plugin
    (cold lint median over the 36 packages 191.5 s to 159.2 s in an alternating measurement,
    199 of 199 packages identical in exit,
    summary and diagnostic headers cold and warm).
    Its four memo commits are replayed as `a349c1a26d` to `abab8e541a` and gated as g107
    (the plugin and config tasks,
    then the family gate);
    g107 was green and they landed ending at `a47fa17b02`.
    The ledger round `w193` landed as B675 to B687 and M265 to M272 (`15b26d5048`).
    The spread-bridge redo `w194` is replayed as `729de4182d` and gated as g108
    (the six plugin packages that use the fixture runner,
    the config bundle,
    then the family gate).
    Its root cause:
    oxlint reserves about 192 GiB of address space for 28 workers,
    so a fork after the workers exist fails with `ENOMEM`;
    the plugin now starts its bridge at load.
    g108 was green apart from the existing `lint:oxlint` findings of `tsdoc` and `stylistic`,
    which the branch without the redo reports the same;
    it landed ending at `249c3d2d0b`.
    `w195` closed the fix-mode fixture runs that still passed through a plugin crash:
    `runOxlintFixture` gained `fix` and `environment` options and every fix-mode spawn of the plugin tests uses it.
    Gate g109 on `483e74e462` was green apart from the same `tsdoc` and `stylistic` findings
    and one `repair` `lint:oxlint` abort inside a host-wide thread-creation failure window
    (recorded in `doc/troubleshooting/fedora-44-fold-emulator-user-task-ceiling.md` on `main`,
    `174bddfeac`);
    that step reran green at the same commit,
    and `w195` landed ending at `20ab2696b9`.
    `w196` (from `483e74e462`) added `OxlintOutputParseError` with the raw oxlint output,
    documented the runner's options in the `test-support` README,
    and dropped the unused `nano-spawn` from `tsdoc`;
    gate g110 on `29b3da16dc` was green apart from the same two packages' findings,
    and it landed ending at `5e2fae86ca`.
    The user slice's `TasksMax` was raised from 12288 to 32768 on the owner's instruction (2026-10-19);
    the matching PAM `nproc` file still reads 12288 until the owner rewrites it.
    Issues filed 2026-10-19 for what is not fixed:
    1777 (`tsdoc` asterisk alignment),
    1778 (`stylistic` findings),
    1779 (Synthetic quota body shape,
    needs one keyed reading),
    1780 (more read-only requests for the memo);
    the feature-branch `GH006` rejection is commented on 1032.
    The ledger round `w197` filed the `w190`,
    `w194`,
    `w195` and `w196` notes and the tail of `s245/ledger-notes-after-split.md`
    as B688 to B690 and M273 to M276;
    gate g111 was green and it landed as `8a475ed139`.
    The next entries start at B691 and M277.
    Running processes keep the old `nproc` limit of 12288 until the owner logs in again.
    Its spread-bridge pair `50ccee0440` and `5dce27bb59` is held:
    two valid-fixture cases passed only because `runOxlintFixture` hides a plugin's throw,
    one ambiguous-spread case newly failed,
    and the commit body counts the failures without naming them.
    `w194` redoes it on `abab8e541a`:
    the harness gap,
    the seven spread-rule cases that already fail on a
    28-thread host,
    and the bridge-start failure.
    The catch-all that warns `semantic evidence unavailable` and skips a file stays as it is:
    `doc/troubleshooting/typescript-go-tuple-type-panic.md` records it as the deliberate handling
    of the upstream tuple panic,
    and narrowing it would fail the `consistency` lint on that known panic.
    In `w177`:
    the hardening's follow-up is done (`83e43a4604` to `f931308964`:
    the sibling origin fix,
    external omissions persisted under schema 10,
    the 75 TS2307 packages built and re-compared identical),
    and the whole repository is compared once more under that bundle before `0f2f380e59` to `f931308964` land.
6.  Done 2026-10-18:
    the owner authorized issue comments and added a permission rule;
    status comments posted on 527 and 541 (`cbedea3574` removed the code on the branch)
    and 542 (fixed by ledger X15),
    each saying the issue stays open until the branch merges to `main`.
