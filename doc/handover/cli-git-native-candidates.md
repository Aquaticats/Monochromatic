# cli-git native candidates and scanner adapter

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/ide` is now `package/ide/app`;
> `package/stub/throwing` is now `package/pnpm-override/stub-throwing`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings.fuzz` is now `package/forbidden-strings/cli.fuzz`;
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

## Purpose and how to respond

This branch adds two layers to the native wrapper under `package/git-policy/cli/src/native/`:
a candidate layer that lists and reads what Git would record
(staged index state or a committed tree),
and an adapter that scans those exact bytes through the repository's forbidden-strings scanner,
linked as a library instead of started as a process.
The accepted approach is the "Git operations and candidate content"
and "Built-in policies and scanner integration" sections of
[`cli-git-rust-implementation.md`](../planning/cli-git-rust-implementation.md);
the behavior inventory is the "Candidate facts" and "forbidden-strings" sections of
[`cli-git-rust-behavior-ledger.md`](../planning/cli-git-rust-behavior-ledger.md).

Paths starting with `src/` or `bin/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Nothing here is reachable from the executable yet.
`entry.rs`,
`management.rs`,
the engine and the built-in policies belong to another delegation;
this branch only adds library modules,
their controls,
a fuzz target,
and the build inputs the new dependency needs.

The branch also holds the pure mapping from these layers' failures to engine failure codes,
described under "Failure codes by cause",
and `main` is merged into it twice.

Inspect first:
the mapping and its three adopted cases under "Failure codes by cause",
what the engine must add before any of it reaches an event, under "What the engine must add",
the merged-tree results under "Gate results", "Mutation testing" and "Fuzzing",
the lockfile decision under "Lockfile",
the edits to files outside this delegation under "Edits outside the owned files",
and the choices under "Choices open to veto".
Respond by merging the branch into `main` and by vetoing any listed choice.

The branch's own gate and its first mutation campaigns ran on the tree of commit `d34761d35`.
The merged-tree gate,
the mapping's mutation campaign
and the merged-tree fuzz tasks ran on the tree of the merge commit `571fe1003`,
as described under "Merged tree at `571fe1003`" in each of those sections;
every later commit changes only this document.

## Branch and commits

Branch `feat/cli-git-native-candidates`,
started from `main` at `a046bc1d2`
and fast-forwarded to `main` at `2cfaf22b0` before the first change
(that brought in the file-enforcer manifest commit `ff3559d89`).
Oldest first:

- `2d2f4c7e4` link the scanner crate with a lockfile seeded from the scanner's own
- `975260fc9` copy the scanner and vendor locked dependencies into the native gate image
- `265c57562` add candidate versions and the batched object reader
- `d2d8860be` give the tester ownership of vendored sources in the gate image
- `128139f32` split candidate controls under the line limit and resolve Clippy findings
- `9e55cdd84` scan candidate bytes through the linked scanner
- `1bc4bf402` name the moved scan-pass controls by their new module
- `98d20d1ab` fuzz cat-file batch reply framing and follow the subject's scanner dependency
- `e86d6eeee` parse batch object sizes with the standard integer parser
- `a678bdfe6` accept either step noticing an exited reader process
- `835930d24` pin intent-to-add listing and document the batch_reply fuzz target
- `abaa364d7` build the rules file pathname from path names on every system
- `ab9339a50` draft of this document
- `c2690ad39` run one policy pass through the public candidate and scanner interface
- `a815718f4` pin the declared-size check against a source that grows after reporting its end
- `7483bef30` update of this document
- `d34761d35` compile the test targets while the gate image is built
- `b66b7ebe6`,
  `1519fd577`,
  `fd69aa2eb`,
  `0bbac323d` and the commits after them up to `30c854f96`:
  updates of this document only
- `27af8f580` merge `main` at `817fb1daa`,
  resolving six conflicts
- `21f93b471` map candidate and scanner failures to engine failure codes by cause
- `571fe1003` merge `main` at `fb64be854`,
  without conflicts
- the commits after `571fe1003`:
  updates of this document only

The commit message of `2d2f4c7e4` states wrong counts;
a corrective commit comment is on GitHub.
It says 72 packages differed from the scanner lockfile and 11 were new to the repository.
Measured:
70 registry package versions differed,
7 of those versions appear in no tracked lockfile,
and one crate name (`jiff-core`) appears in none.

## Merging into `main`

### Merges done

`main` is merged into the branch twice,
each time with `git merge main` and a merge commit.

- `27af8f580` merged `main` at `817fb1daa`.
  The six conflicts listed under "Conflicts of the first merge" were resolved by those steps.
  `git show --remerge-diff 27af8f580` shows each resolution against the conflicted merge:
  both sides kept,
  `main`'s entries first,
  and in `package/git-policy/cli.fuzz/Cargo.toml` two whole `[[bin]]` tables.
  Whether that merged tree was gated was not recorded;
  it was not counted as gated.
- `571fe1003` merged `main` at `fb64be854`,
  14 commits later.
  `git merge-tree --write-tree --name-only main HEAD` reported no conflict beforehand,
  and the merge had none,
  so `git rerere` (enabled in this repository) replayed nothing.
  None of the 14 commits touches `package/git-policy/`,
  the scanner,
  `package/rust-module/`,
  a Cargo manifest or lockfile,
  or `file-enforcer.config.ts`.

The tree of `571fe1003` is the one gated under "Merged tree at `571fe1003`" in "Gate results".
A later merge of this branch into `main` conflicts only where `main` has moved again.

### Conflicts of the first merge

`main` had moved since the branch base `2cfaf22b0`:
another delegation added library modules to the wrapper and the fuzz target `wrapper_controls` to the sidecar.
A trial merge of this branch into `main` at `b3c21676d`
(`git merge-tree --write-tree main HEAD`,
which changes no file)
reported conflicts in six files,
the same six that `27af8f580` resolved.
In each one both sides added an entry at the same place,
so the resolution keeps both,
`main`'s first.

- `package/git-policy/cli/src/native/lib.rs`:
  both sides appended module declarations.
  Keep `main`'s block,
  then this branch's block from `candidate_error` to `scanner_test_support`.
- `package/git-policy/cli.fuzz/src/lib.rs`:
  keep `mod control_tables;` and `pub mod controls;`,
  then `pub mod batch;`.
- `package/git-policy/cli.fuzz/Cargo.toml`:
  both sides added one `[[bin]]` table.
  The conflict covers only the `name` and `path` lines,
  because the three lines after them are the same on both sides.
  The result needs two whole tables,
  `wrapper_controls` and `batch_reply`,
  each with its own `[[bin]]` header and its own `test`, `doc` and `bench` lines.
- `package/git-policy/cli.fuzz/bin/container.mjs`:
  keep both `targets` entries.
- `package/git-policy/cli.fuzz/bin/planted-controls.mjs`:
  `main` added four planted defects and this branch two.
  The conflict ends inside the last entry of each side,
  because both end in the same lines
  (`to: 'if false {',` and the closing brackets).
  Take `main`'s four entries whole,
  closing the fourth with those lines,
  then this branch's two.
- `package/git-policy/cli.fuzz/README.md`:
  three regions.
  Keep both target sections,
  and join the two enumerations
  (what the generator controls reach,
  and which defects are planted)
  so each sentence lists both sides' items.

`package/git-policy/cli/mise.toml` merges without a conflict.
Measured between the branch base `2cfaf22b0` and `main` at `fb64be854`:
`main` changed no lockfile of the wrapper,
the sidecar or the scanner
(its one changed `Cargo.lock` is `package/desktop-app/ide`'s),
not the wrapper manifest,
not the scanner's sources,
build script,
data or manifest
(only its tests,
README,
mutation runner and task file,
none of which the gate snapshot copies),
and not the gate runner or its snapshot helper.
It changed the mutation runner
(the two excluded mutant kinds named under "Campaigns")
and one planted guard removal,
and its sidecar manifest change is the one `[[bin]]` table.
The two lockfiles of this branch therefore stand without regeneration;
`--locked` in the gate and in the sidecar tasks would have refused them otherwise,
and every run under "Merged tree at `571fe1003`" passed with it.

The campaigns recorded under "Results" in "Mutation testing" are bound to the gate image of `d34761d35` alone.

## Candidate layer

### Shape

Seven flat modules,
each with sibling controls:

- `candidate_error`:
  `CandidateError { failure, message }` and the closed `CandidateFailure` list.
  Messages never contain file content or pathnames,
  because a pathname can itself be a forbidden string;
  a listing record is named by its position.
- `candidate_object`:
  `ObjectId`,
  constructible only by `parse_object_id` from 40 or 64 lowercase hexadecimal digits,
  and `CandidateMode` with `mode_from_git`.
- `candidate_batch`:
  `read_batch_reply`,
  the pure reader of one `git cat-file --batch` reply from any buffered byte stream.
- `candidate_record`:
  `parse_raw_records`,
  the pure parser of raw NUL-delimited changed-path records.
- `candidate_reader`:
  `ObjectReader`,
  the long-lived `git cat-file --batch` process.
- `candidate_version`:
  `CandidateSource`,
  `CandidateIdentity`,
  `Candidate`,
  and the immutable `CandidateVersion`.
- `candidate_store`:
  `CandidateStore`,
  the per-invocation owner of versions,
  bytes and invalidation.

A candidate carries its identity
(the store's generation and its position in the version),
its pathname as the raw bytes Git printed,
its mode,
its change against the baseline (`Added`,
`Modified` or `Deleted`),
and its object name,
absent for a deleted path.
Pathnames are never decoded.

### Listing

One listing process names every changed path of a version.
The commands,
verified against Git 2.56.0 in the gate image before any code was written:

```sh
# doc/handover/cli-git-native-candidates.md
git diff-index --cached --raw -z --no-renames --no-abbrev <commit or empty tree> --
git diff-tree --root --no-commit-id -r -z -m --no-renames --no-abbrev <commit>
git hash-object -t tree --stdin
```

`CandidateSource::StagedAgainstHead` asks the running reader for `HEAD`.
A repository without commits answers `HEAD missing`;
the store then computes the empty tree's name with `hash-object` on empty input,
which works for SHA-1 and SHA-256 repositories,
and every index entry is an addition.
`StagedAgainstCommit` compares the index with one named commit,
such as a transaction's recorded base.
`Committed` lists one commit against each of its parents;
a merge lists one comparison per parent,
and the first record of a pathname wins,
as in `src/policy-engine/raw-diff-records.ts`.

Rename detection is off:
a renamed file is one deleted path and one added path naming the same object.
A conflicted index entry (status `U`) fails the whole listing with `UnmergedPath`.
Statuses other than `A`,
`M`,
`T` and `D`,
and modes other than `100644`,
`100755`,
`120000` and `160000`,
are refused.
Output that does not end with a NUL,
or that holds a record without its pathname,
is refused whole:
a partial list would leave paths unchecked.

The environment overlay reaches both the listing and the reader,
so `GIT_INDEX_FILE` selects a private index and `GIT_OBJECT_DIRECTORY` a private object directory.
Both are controlled.
One store,
and so one reader,
is bound to one overlay for its whole life.

### Object reader

`ObjectReader` starts `git cat-file --batch` once and alternates strictly:
one request line,
then its whole reply.
Only complete object names and the fixed word `HEAD` are ever written as requests.
A reply is framed by its declared size alone,
never by searching content,
so content that imitates a reply header cannot move the frame.
A found reply to a request by name must name that object.
Any reply failure closes both pipes,
so no later request can be answered from a stream that is out of step.
Blob bytes are remembered by object name and shared as `Rc<[u8]>`;
a name always denotes the same bytes,
so a remembered blob cannot become stale.
Dropping the reader closes the pipes and waits for the process;
a control observes that no live or zombie process entry remains.

The header length is bounded at 128 bytes.
Content is read with `take(size).read_to_end`,
which grows the buffer only as bytes arrive,
so a header that claims an enormous size ends as a truncated-reply failure and not as one allocation.

### Invalidation

`CandidateStore::version` lists a source once per generation and returns the same shared value afterwards.
`CandidateStore::invalidate` retires every version and advances the generation.
`CandidateStore::bytes` refuses a candidate of an earlier generation with `StaleCandidate`
instead of answering with bytes that may be outdated.
Blob bytes stay remembered across an invalidation.
The reader finds objects written after it started,
which a fix that stages new content requires;
this is controlled.

### Entry kinds

Controls against real Git 2.56.0 cover each kind the delegation named:

- Non-UTF-8 pathnames and non-UTF-8 content,
  including NUL bytes.
- Symbolic links:
  the bytes are the link target.
- Submodules (gitlinks):
  no blob exists in this repository,
  so the bytes are the submodule commit's name,
  as in `src/policy-engine/commit-transaction-candidates.ts`,
  and the reader is never asked.
- Deleted paths:
  no object and no bytes.
- Renamed paths:
  a deletion and an addition of the same object.
- Empty files.
- Type changes,
  mode changes,
  an unborn `HEAD`,
  a SHA-256 repository,
  a root commit and a merge commit.
- Intent-to-add entries (`git add --intent-to-add`):
  Git 2.56.0 lists one as an addition of the empty blob,
  which it stored when the intent was recorded,
  so the candidate has empty content.

Not exercised:
sparse and split indexes.

## Process-count bound

### Mechanism

`candidate_process_count_tests.rs` gives the store a counting `git` as its Git executable:
a script that appends one line to a count file and then executes `/usr/bin/git`.
The count is taken by the operating system's append,
outside the store's own bookkeeping.

### Measurement

For 1,
20 and 200 staged files of distinct content,
listing the staged version and reading every candidate's bytes twice started 2 Git processes each time:
the reader and one `diff-index`.
A new listing after `invalidate` started 1 more and no second reader.
A repository without commits started 3:
the reader,
`hash-object` and `diff-index`.

### Positive control

A per-file reader through the same counting script,
one `git cat-file blob <oid>` per candidate,
was counted 1,
20 and 200 times for the same repositories.
The mechanism therefore shows growth when there is growth.

### Limit of the mechanism

The script counts starts of the executable the store was given.
It would not see a start that bypassed that path.
Two controls close that gap from the other side:
with a nonexistent executable path,
both the listing and the reader fail with `GitNotStarted`,
so neither has another way to reach Git.
Processes Git itself might start are not counted;
`cat-file`,
`diff-index`,
`diff-tree` and `hash-object` are built-in commands.
The ledger's `strace` observation at the executable is still open,
because the executable does not call this layer yet.

## Scanner adapter

### Shape

- `scanner_adapter`:
  `CandidateScanner::load` calls the scanner's `Scanner::load` once;
  `scan` passes a candidate's pathname and bytes to `Scanner::scan`
  and returns the scanner's own `CandidateScan`;
  `cache_warnings` returns the scanner's own `CacheWarning` values.
- `scanner_selection`:
  the rules-file precedence and which candidates are scanned.
- `scanner_run`:
  `scan_version` reads each eligible candidate from the store and scans it,
  returning a result for every candidate scanned,
  clean ones included.

The adapter adds no catch of its own and filters no finding.
The scanner's fail-closed findings (`EngineError`,
`PathnameLineBreak`) come back like any other finding.
A load failure leaves no scanner at all;
its message is the scanner's redacted text.
The first candidate that cannot be read or named ends a pass with no partial result.

The scan identity is the candidate's position in its version,
so a finding is attributed without reading the display path,
which the scanner may have masked.

### Rules file and eligibility

`rules_source` restates the standalone scanner's precedence:
`FORBIDDEN_STRINGS_RULES` if set,
else `forbidden-strings.local.txt` in the repository root,
and a set variable makes a missing file an error.
The spawned scanner ran with the repository root as its working directory;
in-process the root is joined explicitly.

`is_scannable` restates `src/optional/forbidden-strings/scan-candidates.ts`
and `materialize-candidates.ts`:
deleted candidates are not scanned,
the three rule-source paths are not scanned,
and the rules file's own candidate is not scanned.
`rules_candidate_path` resolves the rules file to its repository-relative pathname by names alone,
as the TypeScript policy did.

### Where the rules path is resolved

The user decided on 2026-10-05,
relayed by the coordinating session,
that a `rulesFile` option in `cli-git.config.jsonc` will name the rules file,
with `FORBIDDEN_STRINGS_RULES` as the fallback and the default file after that.
The option belongs to the optional-policies phase;
this branch does not implement it.

Today the path is resolved in one function and nowhere else:

```rust
// package/git-policy/cli/src/native/scanner_selection.rs
pub fn rules_source(configured: Option<&OsStr>, repository_root: &Path) -> RulesSource;
```

- `rules_source` does not read the process environment.
  Its caller passes the value of the variable that the constant `RULES_VARIABLE` names
  (`FORBIDDEN_STRINGS_RULES`),
  or nothing when the variable is unset.
- A passed value,
  even an empty one,
  gives `repository_root.join(value)` with `explicit: true`,
  so an absolute value replaces the root and a missing file is a load failure.
- No value gives `repository_root.join(DEFAULT_RULES_FILE)` with `explicit: false`.
  A missing default file is then tolerated only while the built-in rules are on,
  which leaves the built-in rules alone;
  with them off it is a load failure too
  (`scanner_adapter::tests::load_failures_leave_no_scanner`).
- No production code calls `rules_source` yet,
  so nothing on this branch reads the variable from the environment;
  the controls pass values directly.
  The call that reads the variable is the policy phase's to write.

`rules_source` is unchanged by the mapping commit and by both merges,
and it stays the one place to extend for `rulesFile`:
outside the controls,
no other module under `src/native/` in the merged tree of `571fe1003` names the variable or the default file
(`scanner_test_support.rs` only removes the variable from a control's child environment).
To add `rulesFile`,
extend that one place:
pass the option's value as `configured` when it is set and the variable's value otherwise,
or give `rules_source` the option as a parameter ahead of `configured`.
A value from either is explicit.
`rules_candidate_path` takes the resolved path,
so the rules file's own candidate stays excluded whichever source named it.

### Candidate-byte isolation

`scanner_run_tests.rs`,
in one staged version:

- a file staged clean whose worktree copy then gained the planted token:
  no finding;
- a file staged with the token whose worktree copy was then cleaned:
  `Content { line: 2, rule: "0" }`;
- a file staged with the token and then removed from the worktree:
  the same finding;
- a symbolic link whose target is the token:
  `Content { line: 1, rule: "0" }`;
- an untracked file holding the token:
  not a candidate.

The reverse direction is also the positive control:
the same bytes are reported when they are what is staged.
For a committed version,
the commit's bytes are scanned after the worktree and index swapped which file holds the token,
and a pathname component equal to the token is reported as `Name { component: 1, rule: "0" }`
with the display path masked.
No result's debug text contains the token.

The planted token is assembled at run time,
so this repository's own commit policy cannot report it in the control sources.

### Consumer contracts

The scanner's README section "In-process candidate scans" puts two duties on the linking executable.

The effective profile must unwind.
`scanner_adapter.rs` holds a build-time assertion on `cfg!(panic = "unwind")`.
Planted proof:
`CARGO_PROFILE_DEV_PANIC=abort mise run //package/git-policy/cli:native:check`
fails with the assertion's message,
and the same task without the variable passes.

A panic hook that prints no payload must be installed before the first scan.
That is `main.rs`,
which this delegation does not own.
The hook the scanner's own executable installs is:

```rust
// package/cli/forbidden-strings/src/process_boundary.rs
pub(super) fn omit_panic_payload(_information: &std::panic::PanicHookInfo<'_>) {}
```

Until the wrapper's `main.rs` installs an equivalent hook,
a caught matcher panic would still print Rust's default panic message,
which can include bytes from the file being scanned.

### Controls and their isolation

Loading runtime rules reads the home directory and writes the per-user rule cache.
Each control that loads rules re-runs itself in a child process
whose `HOME` and `FORBIDDEN_STRINGS_CACHE_DIR` are a disposable fixture
(`scanner_test_support.rs`),
as the scanner's own `tests/embedding.rs` does.
The parent requires that exactly one test ran and passed.
That requirement caught a real mistake during this work:
two controls still named their old module after a file split,
and the driver reported that zero tests had run.

"Loaded once" is observed,
not assumed:
after one load the rules file and the cache directory are deleted,
and later scans still report with the loaded rules.

### Public-interface consumer

`candidate_consumer_tests.rs` is a separate test target (`native_candidates`),
so it can reach only public items,
as the policy engine will from other modules.
It runs one whole pass:
select the rules file with `rules_source`,
load once,
list the staged version,
`scan_version`,
attribute the finding to its candidate by identity,
then stage a fix,
call `invalidate`,
observe that the earlier version is refused,
and pass again on the new version with no finding.
It is the closest control to the ledger's consumer-level test that exists before the executable calls these layers.

### Differences from the TypeScript policy

- A candidate pathname containing a line break was an engine failure
  (`repositoryCandidateName` threw).
  Here the scanner reports its fail-closed `PathnameLineBreak` finding,
  which `finding_failure_code` maps to `policy-incomplete`,
  so it stays an engine failure;
  see "Failure codes by cause".
- A `FORBIDDEN_STRINGS_RULES` value that is not UTF-8 is used as a path.
  The standalone scanner reads the variable as UTF-8 and treats such a value as unset.
- The `executable` option is gone,
  as the ledger's "forbidden-strings" section records.

### Failure codes by cause

The user decided on 2026-10-05,
relayed by the coordinating session,
that an engine failure code names its cause:
`content-unavailable` when a candidate's bytes or a repository fact could not be read,
`policy-incomplete` when the policy's own machinery failed
(for the scanner:
its rules cannot be loaded,
or an internal error).
`plugin-threw` is gone.

`EngineFailureCode` in `src/native/diagnostics.rs` has both codes in the merged tree:
`ContentUnavailable`,
and `PolicyIncomplete`,
which the engine delegation added on `main` in `090c147e7`.
This branch does not edit the enum.

#### The mapping

`src/native/scanner_failure_code.rs` (commit `21f93b471`) holds the mapping as four pure functions:

```rust
// package/git-policy/cli/src/native/scanner_failure_code.rs
pub fn candidate_failure_code(failure: CandidateFailure) -> EngineFailureCode;
pub fn scanner_failure_code(failure: ScannerFailure) -> EngineFailureCode;
pub fn scan_run_failure_code(error: &ScanRunError) -> EngineFailureCode;
pub fn finding_failure_code(finding: &ScanFinding) -> Option<EngineFailureCode>;
```

Each `match` names every variant and has no catch-all arm,
so a cause added later does not compile until someone chooses its code.
`scan_run_failure_code` returns the code of the failure it wraps.
`finding_failure_code` returns no code for a violation.

A previous delegate wrote the module and stopped before committing it.
It was reviewed against the decision and the variant lists before the commit,
kept as written apart from one sentence of its documentation
(the line-break finding's reason said only the pathname was read;
both fail-closed findings arrive after the whole candidate was read),
and checked on the host with `native:check`,
`native:clippy`,
`native:lint:rust` and `native:format`
before the merged-tree gate ran it.

#### Which code each failure carries

`content-unavailable`:
every `CandidateFailure` except `StaleCandidate`.
Each reaches the policy as a `CandidateError` from `CandidateStore::version` or `CandidateStore::bytes`,
or wrapped as `ScanRunError::Candidate` from `scan_version`.

- `GitNotStarted`:
  a listing command or the object reader could not be started.
- `GitFailed`:
  a listing command exited unsuccessfully.
- `ListingMalformed`,
  `UnsupportedMode`,
  `UnsupportedStatus`:
  the listing could not be read as changed-path records.
- `UnmergedPath`:
  the index holds a conflicted entry,
  so the path has no single staged content.
- `ReaderEnded`,
  `ReplyMalformed`,
  `ReplyTruncated`,
  `ReplyMismatched`:
  the object reader's stream ended or could no longer be trusted.
- `ObjectMissing`,
  `ObjectKindUnexpected`:
  Git has no such object,
  the object is not a blob,
  or `HEAD` does not name a commit.

`policy-incomplete`,
as the decision names them:

- `ScannerFailure::RulesNotLoaded`,
  as `ScannerError` from `CandidateScanner::load`:
  the rules could not be loaded.
  A missing explicit rules file,
  an invalid rule and a cache configuration error all arrive as this one failure with the scanner's text,
  for the reason under "Gaps in the scanner's embedding API";
  under this decision all of them are `policy-incomplete`,
  so that gap does not block the choice of code.
- `ScanFinding::EngineError` inside a returned `CandidateScan`:
  a matcher failed or panicked inside the scanner's catch boundary.
  It is not an `Err` of this branch;
  it arrives among the findings of that candidate and must not be read as a clean scan.

`policy-incomplete`,
adopted by the coordinating session and open to the owner's veto,
because none of them is a failure to read repository content:

- `ScannerFailure::PathnameUnrepresentable`,
  as `ScannerError` from `CandidateScanner::scan` or wrapped as `ScanRunError::Scanner`:
  the candidate's pathname bytes are not a native path on this platform.
  On Unix only an empty pathname is refused,
  and the listing parser never yields one;
  on other platforms a pathname that is not UTF-8 is refused.
  The pathname was read;
  what failed is handing it to the scanner.
- `CandidateFailure::StaleCandidate`:
  the caller asked for the bytes of a candidate whose version `invalidate` retired.
  The content is readable through a fresh version,
  so this is a sequencing defect of the calling pass,
  not unreadable content.
- `ScanFinding::PathnameLineBreak`:
  the scanner could not inspect a pathname that contains a line break.
  The pathname was read and the scan could not be completed.
  The TypeScript policy made it an engine failure,
  and this mapping keeps it one.
  The alternative the owner may choose instead is to report it as a violation of the policy,
  since the committer can remove it by renaming the file;
  that would change `finding_failure_code` to return no code for it,
  and the policy would then need a finding code of its own for it.

Not failures:
`ScanFinding::Content` and `ScanFinding::Name` are violations,
for which `finding_failure_code` returns no code,
and a `CacheWarning` is a warning after which the scan still runs.

#### Controls by variant

`src/native/scanner_failure_code_tests.rs` holds five unit tests in `scanner_failure_code::tests`.
Every variant of every mapped type is asserted by name:

- `every_candidate_failure_has_its_code`:
  each of the 12 `CandidateFailure` variants listed for `content-unavailable`
  (the test's `UNREADABLE` array),
  and `StaleCandidate`;
  13 of 13 variants.
- `every_scanner_failure_is_policy_incomplete`:
  `RulesNotLoaded` and `PathnameUnrepresentable`;
  2 of 2.
- `scan_run_errors_carry_the_code_of_the_failure_they_wrap`:
  `ScanRunError::Candidate` with each of the 13 candidate causes,
  and `ScanRunError::Scanner` with each of the 2 scanner causes;
  2 of 2 variants.
- `only_findings_that_are_not_matches_carry_a_code`:
  `Content`,
  `Name`,
  `EngineError` and `PathnameLineBreak`;
  4 of 4.
- `the_two_codes_have_the_decided_names`:
  `ContentUnavailable` prints as `content-unavailable`
  and `PolicyIncomplete` as `policy-incomplete`.

The test lists are written out by hand,
so a variant added later is caught by the exhaustive `match` in the mapping,
not by these tests.

#### What the engine must add

No event carries `policy-incomplete` from these layers yet,
and none can without engine changes,
which this branch does not make:

- `PolicyOutcome::Failed(String)`
  (`src/native/policy_engine.rs:80`)
  carries a message and no code.
  `run_policy_stage` reports every `Failed` as `EngineFailureCode::ContentUnavailable`
  (`src/native/policy_engine.rs:255` to `256`).
  The outcome needs a code,
  for example `Failed { code: EngineFailureCode, message: String }`,
  and `run_policy_stage` must put that code into the `EngineFailure` event.
- Every place that builds `PolicyOutcome::Failed` then names its code:
  `src/native/policy_checks.rs:150`,
  `189` and `217`
  (repository facts that could not be read,
  so `content-unavailable`),
  and the controls at `src/native/policy_engine_tests.rs:469`
  and `src/native/policy_checks_tests.rs:141`,
  `224` and `320`.
- The content policies still answer `PolicyOutcome::Unavailable` through
  `policy_checks::check_content`,
  whose `policy_checks::CandidateSource` has only `None` and `NotPorted`.
  The forbidden-strings check must call this branch's layers instead:
  map each `Err` with `candidate_failure_code`,
  `scanner_failure_code` or `scan_run_failure_code`,
  and run `finding_failure_code` over every finding of every returned `CandidateScan`
  before rendering any of them as a violation,
  because `EngineError` and `PathnameLineBreak` arrive inside an `Ok` result.
  A failing finding ends the policy with its code;
  it is never printed as a rule violation.
- Two public types are named `CandidateSource`:
  `policy_checks::CandidateSource`
  (the engine's "what this lifecycle offers")
  and `candidate_version::CandidateSource`
  (this branch's "which version to list").
  Neither module re-exports the other,
  so the merged tree compiles,
  but the module that wires them together must import one under another name or rename one.
- `PolicyEvent::EngineFailure` has a `path: Option<String>` field.
  A candidate pathname is raw bytes that may be a forbidden string,
  so a failure about one candidate should carry no path,
  or the scanner's masked display path,
  never the raw pathname.

## Manifest and lockfile

### file-enforcer

`file-enforcer.config.ts` manages Cargo manifests by guarded key enforcement
(`buildCargoManifestPlan`),
not by generating them.
The scanner dependency is not in `CARGO_SHARED_DEPENDENCIES`
and `package/git-policy/cli` is not in `CARGO_PROFILE_BY_DIR`.
The dependency and profile lines were therefore added by hand.
Observed,
not inferred:
`mise run file-enforcer` in this worktree left `package/git-policy/cli/Cargo.toml` byte-identical.
It did rewrite the root `mise.toml` and `package/config/pnpr/config.yaml`,
which is unrelated drift also present in the main checkout;
those two changes were discarded here and are not part of this branch.

After the merge `571fe1003`,
`mise run file-enforcer` ran again in this worktree with exit status 0.
Both manifests the branch touches,
`package/git-policy/cli/Cargo.toml` and `package/git-policy/cli.fuzz/Cargo.toml`,
match the glob `package/*/*/Cargo.toml` that `buildCargoManifestPlan` enforces,
and both were left byte-identical,
so there was no output of its own to commit.
It again rewrote only the root `mise.toml` and `package/config/pnpr/config.yaml`.
The `pnpr` change was the same diff as the main checkout's uncommitted one
(three newly published packages);
the `mise.toml` change added the same two `node_modules/.bin` entries as the main checkout's
and also removed entries whose `node_modules/.bin` directory this worktree lacks
(checked for `package/stub/throwing` and `package/git-policy/cli`),
so it depends on what is installed where it runs.
Both were discarded again.

The manifest gains `[profile.dev.build-override]` and `[profile.release.build-override]` with `opt-level = 3`.
Cargo reads profiles only from the root manifest,
so the scanner's own override does not reach this crate,
and its build script would otherwise compile the embedded baseline unoptimized.
`package/cli/forbidden-strings.fuzz/Cargo.toml` is the precedent.

### Lockfile

`mise run //package/git-policy/cli:native:lock` (`cargo generate-lockfile --offline`)
resolves every dependency to the newest version in the local Cargo cache.
Its output was inspected and not committed:
70 registry package versions differed from the scanner's own lockfile,
among them the matcher's dependencies
(`aho-corasick` 1.1.5 for 1.1.4,
`regex-automata` 0.4.18 for 0.4.14,
`memchr` 2.8.3 for 2.8.0),
7 versions that no tracked lockfile holds,
and one crate (`jiff-core`) that no tracked lockfile holds.
That would have linked the scanner against versions its own verification never ran on,
and added third-party code beyond the one dependency this delegation allows.

The committed lockfile is the scanner's lockfile plus this crate and `monochromatic-jsonc-edit`:
155 packages,
153 of them identical to `package/cli/forbidden-strings/Cargo.lock`.
It is produced by a new task:

```toml
# package/git-policy/cli/mise.toml
[tasks."native:lock:scanner"]
run = ["cp ../../cli/forbidden-strings/Cargo.lock Cargo.lock", "cargo update --workspace --offline"]
```

`native:lock` is unchanged and would undo this.
Whether it should be redefined is a choice for the owner of that task.
The lockfile was not edited by hand.

### Gate snapshot and vendoring

The gate container has no network.
`bin/native-scanner-snapshot.mjs` copies the scanner crate
(`Cargo.toml`,
`Cargo.lock`,
`build.rs`,
`src`,
`data`)
and `package/rust-module/forbidden-regex`,
and runs `cargo vendor --offline --locked --versioned-dirs` on the host.
Vendoring succeeded from the existing Cargo cache
(151 packages,
113 MiB),
so no `native:dependencies:fetch` task was added.
It succeeded because this host had already downloaded every archive the scanner's lockfile names.
A host that has not needs `cargo fetch --locked` in the wrapper package once before the gate,
and has no task for it;
that is the case the delegation's `native:dependencies:fetch` task was meant for.
The image copies the vendored sources with `--chown=1000:1000`:
`fnv-1.0.7/.travis.yml` is mode `0640` in its archive,
and the first gate run stopped because the tester could not read it.

The memory bound was not raised and compiler jobs were not lowered;
the build was never killed.
The container build of the test targets took 125 seconds with the scanner in one run,
between 183 and 301 seconds in runs that shared the host with other sessions' builds,
and 3.51 seconds for the wrapper alone in the baseline run.

Every container started from the gate image used to compile from nothing:
the gate's test run,
each of the mutation runner's five planted controls,
and the mutation baseline,
which the runner bounds at 300 seconds per build.
Commit `d34761d35` adds one build step to the image
(`cargo test --offline --locked --all-targets --no-run`),
so those containers rebuild only what changed.
The Clippy run uses the mounted host toolchain and still compiles for itself.

## Gate results

The gate is `GIT_POLICY_NATIVE_IMAGE_TAG=<tag> mise run //package/git-policy/cli:native:test:container`,
with the tag `candidates` for the branch alone and `candidates-merge` for the merged tree.
Every run is listed,
failed ones included.

### Branch alone, up to `d34761d35`

- Baseline on the merged tree (`2cfaf22b0`):
  322 unit tests,
  21 binary-level tests,
  Clippy passed.
  Evidence `package/git-policy/cli/target/verification/native-Lr0W4j`.
- First run with the dependency (sources of `265c57562`):
  stopped before compiling on the unreadable vendored file.
  Evidence `native-CgkwVM`.
- Candidate layer (sources of `265c57562` with the ownership fix):
  382 unit tests and 21 binary-level tests passed;
  Clippy failed on `collapsible_if` and twice on `chunks_exact_to_as_chunks`.
  Evidence `native-IsacfX`.
- Candidates and scanner adapter (`9e55cdd84`):
  393 of 395 unit tests passed;
  the two failures were the mistyped control names described under "Controls and their isolation".
  Clippy did not run.
  Evidence `native-bP1bMZ`.
- `1bc4bf402`:
  394 of 395 unit tests passed.
  `request_to_an_exited_process_is_a_reader_failure` failed:
  the write to the exited stand-in succeeded and the read reported the ended stream.
  A child another test thread is starting can hold an inherited copy of the pipe until it executes,
  so which step notices is not fixed.
  The control now accepts either;
  the failure kind and the closed reader are what it pins.
  Evidence `native-zGeY5k`.
- `a678bdfe6`:
  395 unit tests,
  21 binary-level tests,
  Clippy passed.
  Test image `1206b4c7bfccc5b9107e06566ff27e2b4fa5ac8f932fcac466611d4d97100f88`;
  evidence `native-QBnkSC`.
- `abaa364d7`:
  396 unit tests and 21 binary-level tests passed,
  then `podman run` of the test container returned status 127 without a message,
  and Clippy did not run.
  Other Podman commands failed with `database is locked` in the same minutes,
  while other sessions were running containers on the host;
  the cause of the 127 was not established.
  This run is not counted as a pass.
  Evidence `native-1cOzuy`.
- `c2690ad39`:
  396 unit tests,
  21 binary-level tests,
  1 public-interface consumer test,
  Clippy passed.
  The host's load average was above 70 with other sessions' containers,
  and the unit tests took 147.55 seconds where an unloaded run took 6.03.
  Evidence `native-81pAf1`.
- `a815718f4`,
  before the image build step existed:
  stopped by hand while the image was being built,
  because the runner was about to change.
  It produced no result.
- Final run,
  on the sources of `a815718f4` with the runner of `d34761d35`:
  398 unit tests in the library,
  0 unit tests in the executable,
  21 binary-level tests (`native_binary`),
  1 public-interface consumer test (`native_candidates`),
  Clippy passed,
  exit status 0.
  The image build compiled the test targets in 4 minutes 16 seconds;
  the test container then found them built in 10.22 seconds.
  The unit tests took 63.82 seconds with the host's load average between 70 and 106.
  Test image `9ffa5c481579a9f6501944626169fbb599602ce8458e0aa19f052c840ee65721`;
  evidence `native-fm9zle`.
  The commits after `a815718f4` change only the runner and this document.

### Merged tree at `571fe1003`

One run,
`GIT_POLICY_NATIVE_IMAGE_TAG=candidates-merge`,
on the tree of the merge commit `571fe1003`
(this branch with the mapping commit `21f93b471`,
and `main` at `fb64be854`).
The snapshot was copied while the worktree's `package` directory matched that commit;
only this document was being edited.
It passed with exit status 0:

- 523 unit tests in the library,
  all passed:
  the engine's tests from `main`,
  the branch's candidate and scanner controls,
  and the 5 mapping controls.
  Before the merge,
  the host's run of the mapping controls through `native:test:host` counted the same 523
  (5 run,
  518 filtered out);
  the merge changed no file under `package/git-policy/`.
- 0 unit tests in the executable.
- 37 binary-level tests (`native_binary`),
  all passed;
  the 16 more than the branch alone had come from `main`,
  since the branch changes no binary-level test file.
- 1 public-interface consumer test (`native_candidates`),
  passed.
- Clippy passed with warnings denied.
- The image's test build,
  the test run and Clippy all pass `--locked`,
  and none refused the lockfile,
  so it needed no regeneration.

The image build compiled the test targets in 2 minutes 24 seconds;
the test container found them built in 0.18 seconds,
and the library tests took 6.16 seconds.
The host's load average,
sampled four times during the run,
was between 23 and 42.
Test image `8ff0051a5b9a132ffa5ef1b01347039511d7ad3098a79831116d30ff0f1919e4`,
built from base `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`;
evidence `package/git-policy/cli/target/verification/native-mQEfwT`
(`manifest.json`,
and `passed.json` with tests and Clippy both true).

## Mutation testing

### Campaigns

`native:mutation:scoped` binds to the last gate image by tag,
plants its five guard removals,
then runs cargo-mutants 27.1.0 with a 300-second build bound and a 90-second test bound.
All campaigns here ran against the final gate image
`9ffa5c481579a9f6501944626169fbb599602ce8458e0aa19f052c840ee65721`.

The ten new source files were split over three campaigns that ran at the same time,
because one mutant took minutes on the shared host.
The runner names its campaign image by the tag,
so campaigns that run together need different tags.
The second and third campaign therefore used the tags `candidates-b` and `candidates-c`,
which `podman tag` made further names of the same gate image;
each campaign's `manifest.json` records that image identity as `baseImage`.
This departs from "always `candidates`" in the delegation;
the tags are this worktree's own.

- Tag `candidates`:
  `candidate_batch.rs`,
  `candidate_record.rs`,
  `candidate_object.rs`,
  `candidate_error.rs`.
- Tag `candidates-b`:
  `candidate_reader.rs`,
  `candidate_store.rs`,
  `candidate_version.rs`.
- Tag `candidates-c`:
  `scanner_adapter.rs`,
  `scanner_run.rs`,
  `scanner_selection.rs`.

### Results

Across the three campaigns cargo-mutants generated 147 mutants:
114 were caught,
33 did not compile,
none was missed and none timed out.
There was no survivor to disposition,
no mutant was excluded,
and no source or test changed after the campaigns.
Every campaign exited with status 0 and noticed all five planted guard removals first.

- Tag `candidates`,
  evidence `package/git-policy/cli/target/verification/native-mutation-oRwTOn`:
  84 mutants,
  73 caught,
  11 did not compile.
  `candidate_batch.rs` 36 caught and 5 not compiled,
  `candidate_record.rs` 21 and 4,
  `candidate_object.rs` 15 and 2,
  `candidate_error.rs` 1 and 0.
  Unmutated baseline:
  2.2 seconds to build,
  62.3 seconds to test.
- Tag `candidates-b`,
  evidence `native-mutation-qkwQPV`:
  39 mutants,
  22 caught,
  17 did not compile.
  `candidate_reader.rs` 4 caught and 7 not compiled,
  `candidate_store.rs` 16 and 7,
  `candidate_version.rs` 2 and 3.
  Unmutated baseline:
  0.2 seconds to build,
  67.5 seconds to test.
- Tag `candidates-c`,
  evidence `native-mutation-2k2Nnt`:
  24 mutants,
  19 caught,
  5 did not compile.
  `scanner_adapter.rs` 2 caught and 3 not compiled,
  `scanner_run.rs` 3 and 1,
  `scanner_selection.rs` 14 and 1.
  Unmutated baseline:
  0.2 seconds to build,
  67.2 seconds to test.

### Margin to the test bound

The host was shared,
so the unmutated suite took 62 to 68 seconds against the 90-second bound.
The slowest test run of any mutant took 81.3 seconds and ended in a failing test,
so it was caught,
not timed out.
A mutant that no test notices has to pass the whole suite,
which on this host took about as long as the baseline;
the margin was about 22 seconds.
`timeout.txt` and `missed.txt` are empty in all three campaigns,
so nothing here rests on a timeout.
A rerun on a busier host can turn caught mutants into timeouts;
compare each with the `Unmutated baseline` line of the same run before reading it as a hang.

### Mutants that did not compile

cargo-mutants replaces a function body with a default value of its return type.
For 33 mutants that replacement did not compile,
because the type has no default
(the layer's error,
object name,
reply,
record,
candidate,
version and reader types,
its enums,
`RulesSource`,
and the scanner's result types)
or because the generated expression for `Rc<[u8]>` does not type-check.
The tool calls these unviable and does not count them as caught or missed;
`unviable.txt` in each evidence directory lists them.
They are not exclusions,
but for those functions the campaigns say nothing about "returns a fixed value".

As extra evidence,
not as a disposition the delegation required,
30 fixed-value defects were planted by hand,
each in its own bounded container started from the same gate image
with the planted file copied in and
`cargo test --offline --locked --all-targets --no-fail-fast -- --test-threads=2` as the command,
which is how `bin/native-planted-controls.mjs` plants its guard removals.
A defect counts as noticed when the planted crate compiled,
the container exited with a failure status,
and at least one test line ended in `FAILED`.
All 30 were noticed.
The number after each defect is how many distinct controls failed.

- `candidate_store.rs`:
  `bytes` returns empty bytes (16),
  one zero byte (17),
  one byte of value one (17);
  `version` returns an empty version (23);
  `listing_failure` returns one fixed failure (2);
  `empty_tree` returns one fixed name (4).
- `candidate_reader.rs`:
  `blob` returns empty bytes (25),
  one zero byte (26),
  one byte of value one (26);
  `request` returns a missing notice without asking (32);
  `head_commit` returns one fixed name (22);
  `reader_ended` returns one fixed failure (4).
- `candidate_version.rs`:
  `build_version` ignores its records (22);
  `candidates` returns one fixed candidate (19);
  `candidate_at_path` returns one fixed candidate (18).
- `candidate_batch.rs`:
  `read_batch_reply` returns a missing notice without reading (44);
  `parse_found_header` returns a fixed header of size 0 (39) and of size 1 (40);
  `object_kind` returns blob for any word (22);
  `reply_failure` returns one fixed failure (12).
- `candidate_record.rs`:
  `parse_raw_records` returns one fixed record (30);
  `parse_record` returns one fixed record (29);
  `change_of` returns added for any status (4);
  `record_failure` returns one fixed failure (6).
- `candidate_object.rs`:
  `parse_object_id` returns one fixed name (35);
  `mode_from_git` returns regular for any text (7).
- `scanner_adapter.rs`:
  `scan` returns one fixed empty scan (8);
  `scan` scans no bytes (5).
- `scanner_run.rs`:
  `scan_version` returns one fixed empty scan (5).
- `scanner_selection.rs`:
  `rules_source` reports a configured rules file as not explicit (2).

That gives 29 of the 33 uncompiled mutants a hand-planted counterpart,
plus one further defect (`scan` scans no bytes).
Four have none,
because the fixed value cannot be written:
`start_object_reader` and `CandidateStore::reader` would need a running process as the value,
`CandidateScanner::load` a loaded scanner,
and `cache_warnings` a warning,
whose fields are private to the scanner.
For `cache_warnings` the tool's other mutant,
an empty list,
was caught.
For the other three no mutant of any kind ran;
they hold no operator the tool mutates,
and every control that reads a blob or scans a candidate passes through them.

Evidence:
`package/git-policy/cli/target/verification/planted-unviable-20261005`,
`planted-unviable-20261005-second` and `planted-unviable-20261005-third`,
each with `planted-controls.json`,
the planted sources and the container logs.
The script that drove the containers is a scratch file outside the repository.
Like every evidence directory named in this document,
these are under an ignored `target` directory and exist only in this worktree on this host.

### Merged tree at `571fe1003`

One campaign:

```sh
# doc/handover/cli-git-native-candidates.md
GIT_POLICY_NATIVE_IMAGE_TAG=candidates-merge mise run //package/git-policy/cli:native:mutation:scoped -- \
  --file src/native/scanner_failure_code.rs --file src/native/lib.rs
```

It ran against the merged-tree gate image `8ff0051a5b9a132ffa5ef1b01347039511d7ad3098a79831116d30ff0f1919e4`,
started right after that gate with no file edited in between except this document.
The scope is the new mapping module
and the one wrapper code file whose conflict was resolved in a merge
(`lib.rs`, in `27af8f580`;
the merge `571fe1003` had no conflict).
The other conflicted code files,
`src/lib.rs`,
`bin/container.mjs` and `bin/planted-controls.mjs` under `package/git-policy/cli.fuzz/`,
belong to the sidecar,
which this runner does not mutate;
the sidecar tasks under "Merged tree at `571fe1003`" in "Fuzzing" run all three.
The runner of the merged tree excludes the two mutant kinds `main` added
(`replace += with *=` and `replace -= with /=`);
no other exclusion was added.

- All five planted guard removals were noticed first.
- cargo-mutants 27.1.0 found 5 mutants,
  all in `scanner_failure_code.rs`;
  `lib.rs` holds only module declarations and yields none.
- 1 caught:
  `finding_failure_code` replaced with `None`.
- 4 unviable:
  `candidate_failure_code`,
  `scanner_failure_code` and `scan_run_failure_code` replaced with `Default::default()`,
  and `finding_failure_code` replaced with `Some(Default::default())`;
  `EngineFailureCode` has no default value.
- 0 missed,
  0 timed out;
  exit status 0.
- Unmutated baseline:
  0 seconds to build,
  15 seconds to test,
  against the 90-second test bound.

Evidence `package/git-policy/cli/target/verification/native-mutation-ZMVkNL`;
its `manifest.json` records `baseImage` as the gate image above,
and the campaign image is `9b89dbf1ac48eba26bb4c55d1a77fa57d28091c44ac980192fa745b01f4060e2`.

Four of the five mutants say nothing about "returns a fixed code",
so,
as extra evidence and after every container run had finished,
8 defects were planted by hand into the committed `scanner_failure_code.rs`,
one at a time,
each followed by `mise run //package/git-policy/cli:native:test:host -- scanner_failure_code` on the host
(these controls start no Git)
and by restoring the committed file;
`git diff` showed the file equal to `HEAD` afterwards.
A defect counts as noticed when the planted crate compiled,
the task failed,
and at least one control was reported `FAILED`.
All 8 were noticed;
the number after each is how many controls failed.

- `StaleCandidate` maps to `content-unavailable` (2).
- The twelve unreadable-content causes map to `policy-incomplete` (2).
- Both scanner causes map to `content-unavailable` (2).
- `scan_run_failure_code` returns one fixed code for every scanner failure (1)
  and another for every candidate failure (1).
- `PathnameLineBreak` returns no code,
  as if the candidate were clean (1).
- A `Content` or `Name` violation returns a code (1).
- `EngineError` and `PathnameLineBreak` return `content-unavailable` (1).

Evidence `package/git-policy/cli/target/verification/planted-mapping-20261006`
(`planted-controls.json` and one log per defect).
The script that planted them is a scratch file outside the repository.

## Fuzzing

### Target

`package/git-policy/cli.fuzz` gains `batch_reply` over `read_batch_reply`.
Each input is used twice:
split at its first line feed into a request and a raw stream,
and mapped by `generated_reply` to one of 14 built shapes with the outcome it must have
(canonical replies for each object kind and both hash formats,
missing notices,
a reply for another object,
content shorter and longer than declared,
a stream ending in the header or before the closing line feed,
an overlong header,
headers Git never prints,
a missing notice for another request,
and an empty stream).

`check_batch_reply` asserts,
for any bytes:
reading is repeatable;
an accepted reply is byte for byte Git's canonical rendering of the returned value;
a reply to a request by name names that object;
bytes after the reply change nothing;
no sampled proper prefix of an accepted reply is accepted;
and a refusal is one of the four reply failures,
"ended" only for an empty stream.

### Controls

`mise run //package/git-policy/cli.fuzz:test` passes 11 generator controls,
4 of them new.
They count that the generator reaches every object kind,
both hash formats,
a missing notice and all four failures.
`mise run //package/git-policy/cli.fuzz:test:planted` plants 7 defects,
2 of them new,
and all were noticed:
"object content past its declared size is accepted"
and "an object reply for another object is accepted"
were each noticed by `batch::tests::fixed_hard_cases_hold` and `batch::tests::generated_replies_reach_every_outcome`.
Both tasks were run again on the final tree,
with the same result:
11 controls passed and 7 of 7 planted defects were noticed.
Evidence `package/git-policy/cli.fuzz/target/verification/planted-JCLo1z`;
the earlier run is `planted-2bQ4Q5`.

A first choice of planted defect,
removing the check that fewer content bytes arrived than declared,
was dropped before it ran:
a stream that ends early also fails the read of the closing line feed,
so on a pipe that removal changes only the message.
The check stays,
because the reply reader accepts any buffered source:
`candidate_batch_resume_tests.rs` gives it a source that reports its end and then yields a line feed,
as a file still being written can,
and the check is what refuses the shortened content there.

### Smoke campaign

`GIT_POLICY_NATIVE_IMAGE_TAG=candidates mise run //package/git-policy/cli.fuzz:smoke`
builds the four targets with AddressSanitizer in a bounded container and fuzzes each for 30 seconds
with no host mounts,
no network,
2 GiB,
2 CPUs,
128 PIDs
and a 4,096-byte input limit.

The first run did not reach the new target.
`global_arguments` reported 401,541 executions and `config_loading` 142,750,
each with exit status 0;
`config_schema` ran,
and then a Podman command failed with `database is locked`
and the runner's cleanup failed the same way,
while other sessions were running containers on the host.
Three leftover containers of this worktree's run image were removed by hand.
Evidence `package/git-policy/cli.fuzz/target/verification/campaign-xNWDLy`.

The second run started at 15:59 local time on 2026-10-05,
after commit `c2690ad39` and before commit `a815718f4` was recorded;
whether the files of `a815718f4` were already in the working tree it copied was not recorded.
That does not change what was fuzzed:
`a815718f4` adds one `#[cfg(test)]` module and its file,
and `d34761d35` changes only the gate runner,
so the code a fuzz target compiles is the same at all three commits.
The run exited with status 0:

- `global_arguments`: 971,930 executions, exit status 0.
- `config_loading`: 363,596 executions, exit status 0.
- `config_schema`: 19,909 executions, exit status 0.
- `batch_reply`: 89,225 executions in 31 seconds, exit status 0,
  684 inputs added to the corpus,
  no crash artifact,
  peak resident memory 483 MiB.

Evidence `package/git-policy/cli.fuzz/target/verification/campaign-zPxrE8`;
its manifest records base image `62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884`.
The execution counts of the first three targets differ from the first run by more than a factor of two;
both runs shared the host with other sessions,
so the counts say the targets ran,
not how fast they are.

### Merged tree at `571fe1003`

The three sidecar tasks ran one after another on the tree of `571fe1003`,
after the merged-tree gate and mutation campaign,
with `GIT_POLICY_NATIVE_IMAGE_TAG=candidates-merge`,
and each exited with status 0.
No file under `package/` was edited while they ran.

- `mise run //package/git-policy/cli.fuzz:test`:
  17 generator controls passed,
  the 11 of the branch alone plus the 6 `controls::tests` controls of `main`'s `wrapper_controls` target.
  `Cargo.lock` was left unchanged.
- `mise run //package/git-policy/cli.fuzz:test:planted`:
  the 17 unplanted controls passed,
  then 11 of 11 planted defects were noticed,
  `main`'s four and this branch's two among them.
  "object content past its declared size is accepted"
  and "an object reply for another object is accepted"
  were each noticed by `batch::tests::fixed_hard_cases_hold` and `batch::tests::generated_replies_reach_every_outcome`,
  as before the merge.
  Evidence `package/git-policy/cli.fuzz/target/verification/planted-q2BzED`.
- `mise run //package/git-policy/cli.fuzz:smoke`:
  the 17 generator controls and Clippy passed in the build container,
  then each of the five targets ran for 30 seconds with AddressSanitizer:
  `global_arguments` 1,144,393 executions,
  `config_loading` 223,948,
  `config_schema` 43,206,
  `wrapper_controls` 48,949
  and `batch_reply` 159,306,
  each with exit status 0.
  `batch_reply` added 831 inputs to its corpus,
  left no crash artifact
  and peaked at 485 MiB resident memory.
  The manifest records base image `62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884`,
  the same as the branch's own second run,
  and `rustc 1.100.0-nightly (1303417c4 2026-09-21)`.
  Evidence `package/git-policy/cli.fuzz/target/verification/campaign-OVOU54`.

## Public API for the optional-policy phase

All under `git_policy_cli::`.

```rust
// package/git-policy/cli/src/native/candidate_store.rs
impl CandidateStore {
    pub fn new(real_git: &Path, global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> CandidateStore;
    pub fn version(&mut self, source: &CandidateSource) -> Result<Rc<CandidateVersion>, CandidateError>;
    pub fn bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, CandidateError>;
    pub fn invalidate(&mut self);
}

// package/git-policy/cli/src/native/candidate_version.rs
pub enum CandidateSource { StagedAgainstHead, StagedAgainstCommit(ObjectId), Committed(ObjectId) }
pub struct CandidateIdentity { pub generation: u64, pub index: usize }
pub struct Candidate {
    pub identity: CandidateIdentity,
    pub path: Vec<u8>,
    pub mode: CandidateMode,
    pub change: CandidateChange,
    pub object: Option<ObjectId>,
}
impl CandidateVersion {
    pub fn candidates(&self) -> &[Candidate];
    pub fn candidate_at_path(&self, path: &[u8]) -> Option<&Candidate>;
}

// package/git-policy/cli/src/native/scanner_adapter.rs
pub struct RulesSource { pub path: PathBuf, pub explicit: bool }
impl CandidateScanner {
    pub fn load(rules: &RulesSource, builtin_rules: bool) -> Result<CandidateScanner, ScannerError>;
    pub fn cache_warnings(&self) -> &[forbidden_strings::CacheWarning];
    pub fn scan(&self, candidate: &Candidate, bytes: &[u8]) -> Result<forbidden_strings::CandidateScan, ScannerError>;
}

// package/git-policy/cli/src/native/scanner_selection.rs
pub fn rules_source(configured: Option<&OsStr>, repository_root: &Path) -> RulesSource;
pub fn rules_candidate_path(rules_path: &Path, repository_root: &Path) -> Option<Vec<u8>>;
pub fn is_scannable(candidate: &Candidate, rules_path: Option<&[u8]>) -> bool;

// package/git-policy/cli/src/native/scanner_run.rs
pub fn scan_version(
    scanner: &CandidateScanner,
    store: &mut CandidateStore,
    version: &CandidateVersion,
    rules_path: Option<&[u8]>,
) -> Result<Vec<forbidden_strings::CandidateScan>, ScanRunError>;

// package/git-policy/cli/src/native/scanner_failure_code.rs
pub fn candidate_failure_code(failure: CandidateFailure) -> EngineFailureCode;
pub fn scanner_failure_code(failure: ScannerFailure) -> EngineFailureCode;
pub fn scan_run_failure_code(error: &ScanRunError) -> EngineFailureCode;
pub fn finding_failure_code(finding: &forbidden_strings::ScanFinding) -> Option<EngineFailureCode>;
```

Also public:
`candidate_object::{ObjectId, parse_object_id, CandidateMode}`,
`candidate_record::CandidateChange`,
`candidate_error::{CandidateError, CandidateFailure}`,
`scanner_adapter::{ScannerError, ScannerFailure}`,
`scanner_run::ScanRunError`,
and the pure parsers `candidate_batch::read_batch_reply` and `candidate_record::parse_raw_records`,
which the fuzz sidecar uses.

A policy pass is:
create one `CandidateStore` per invocation with the caller's global prefix and child environment overlay;
take `version(source)`;
load one `CandidateScanner` if the policy is enabled;
call `scan_version`;
map each `CandidateScan` by `identity` back to `version.candidates()[identity]`;
end the policy with the code `finding_failure_code` gives for any finding that has one,
and end it with the code the matching function in `scanner_failure_code` gives for any `Err`.
After a fix or replay,
call `invalidate` and take the version again.

## Gaps in the scanner's embedding API

None blocks this work.
Three are worth the scanner owner's attention:

- `Scanner::load` returns `anyhow::Result`.
  The wrapper has no `anyhow` dependency and may not add one,
  so the adapter can only render the failure as text;
  it cannot branch on a missing file versus an invalid rule versus a cache configuration error.
  A typed load error would let the policy phase choose engine-failure codes.
- A consumer cannot provoke a loader or matcher panic,
  so the wrapper cannot observe the scanner's catch boundary end to end.
  The build-time unwind assertion is the only consumer-side control.
  The scanner's own guard tasks cover the boundary inside the scanner.
- The payload-free panic hook (`omit_panic_payload`) is private to the scanner's executable.
  Every embedding host has to restate it.
  Exporting it would give hosts one definition.

The rules-file precedence and the self-match exclusions live in the scanner's executable and in the TypeScript policy,
not in the library.
They are restated in `scanner_selection.rs`.
That is duplication by position,
not a gap in what the library can do.

## Edits outside the owned files

The delegation owned new `candidate_*.rs` and `scanner_*.rs` files,
`lib.rs` module declarations,
the wrapper manifest and lockfile,
one fuzz target and its registration,
and this document.
Linking the scanner made these further edits necessary;
each is additive.

- `package/git-policy/cli/bin/test-native-container.mjs`:
  one import,
  `...scannerSnapshotEntries({ context })` in the copy list,
  one `await vendorLockedDependencies({ context: context.path })`,
  two `COPY --chown=1000:1000` lines for `vendor` and `cargo-config`,
  and one `RUN ["cargo", "test", "--offline", "--locked", "--all-targets", "--no-run"]` line
  after `WORKDIR`,
  described under "Gate snapshot and vendoring".
  Commits `975260fc9`,
  `d2d8860be` and `d34761d35`.
- `package/git-policy/cli/bin/native-scanner-snapshot.mjs`:
  new,
  holding those two helpers so the runner stays under the 300-line limit of its Oxlint configuration.
  Both files pass `mise run //package/git-policy/cli:lint:oxlint:paths`.
- `package/git-policy/cli/mise.toml`:
  the `native:lock:scanner` task appended.
- `package/git-policy/cli.fuzz/Cargo.toml`:
  the `batch_reply` target and the two build-override profiles.
- `package/git-policy/cli.fuzz/Cargo.lock`:
  regenerated as the wrapper lockfile plus the sidecar's 8 own packages.
  The 7 packages it previously shared with the wrapper's dependency tree moved to the scanner lockfile's versions;
  the sidecar-only packages kept their versions.
- `package/git-policy/cli.fuzz/mise.toml`:
  a `lock:subject` task appended,
  the sidecar's counterpart of `native:lock:scanner`.
- `package/git-policy/cli.fuzz/bin/container.mjs`:
  one `targets` entry,
  and `copyInputs` also copies the scanner and its engine.
- `package/git-policy/cli.fuzz/bin/planted-controls.mjs`:
  the scanner and engine inputs,
  and 2 planted defects.
- `package/git-policy/cli.fuzz/src/lib.rs`,
  `src/batch.rs`,
  `src/batch_tests.rs`,
  `seed/batch_reply`,
  `dictionary/batch_reply.dict`:
  the target's generator,
  invariants,
  controls and inputs.

Without the copies in the two sidecar runners,
the existing `smoke` and `test:planted` tasks would no longer build,
because the subject they compile now depends on the scanner.

No `.ts` file and nothing under `package/cli/forbidden-strings` was edited.

## Choices open to veto

- The lockfile is seeded from the scanner's lockfile instead of taken from `native:lock`.
  Reason under "Lockfile".
- `native:lock:scanner` and `lock:subject` are new tasks in files this delegation only partly owned.
  They exist so the committed lockfiles are reproducible through `mise run`.
- The build-override profiles in the wrapper and sidecar manifests.
- The reader's standard error is discarded.
  Nothing reads it while replies are awaited,
  and a full unread pipe would block Git,
  for example on the progress output of a lazy fetch in a partial clone.
  A reader failure therefore reports the protocol state and names the command to rerun,
  not Git's own text.
  The alternative that keeps Git's text is a thread that drains the pipe.
- `invalidate` retires every version,
  committed ones included,
  although a commit's delta cannot change.
  Taking a committed version again costs one process.
- A stale candidate is refused instead of answered from its still-valid object name.
- Three failures carry `policy-incomplete`,
  adopted by the coordinating session and not named by the owner's decision:
  `ScannerFailure::PathnameUnrepresentable`,
  `CandidateFailure::StaleCandidate`
  and the scanner's `ScanFinding::PathnameLineBreak` finding.
  The reasons are under "Which code each failure carries";
  a veto changes one arm of `src/native/scanner_failure_code.rs` and its control.
- Gitlink bytes are the submodule commit's name and deleted bytes are empty,
  as in the TypeScript wrapper,
  instead of a separate "no content" variant.
- An intent-to-add entry is a candidate,
  although a commit does not record it.
  `git diff-index --cached --ita-invisible-in-index` hides it
  (probed in the gate image);
  Git's default was kept.
  Whether the TypeScript wrapper treated such an entry as a candidate was not checked.
- All blob bytes read in an invocation stay in memory until the store is dropped,
  as `loadBlobBatch` kept them.
- Candidates are scanned one after another.
  The standalone scanner scans files in parallel.
  No timing was measured,
  so this document makes no speed claim in either direction.

## Failed or skipped

- Of the gate runs,
  four failed,
  one ended with a Podman status 127 whose cause was not established,
  and one was stopped by hand;
  each is listed under "Gate results".
  The first fuzz smoke run failed on a Podman error before it reached the new target.
  The merged-tree gate,
  campaign and sidecar tasks each passed on their one run.
- The tree of the first merge,
  `27af8f580`,
  was not gated on its own;
  the gated tree of `571fe1003` contains it,
  and differs from it only by the mapping commit
  and by `main`'s later documents and packages outside `package/git-policy/`.
- `native:clippy:windows` was not run for the mapping module,
  which holds no platform-specific code.
- The control for a request written to an exited process could not be made to observe one fixed step,
  for the reason given there.
  The write-failure branch of `ObjectReader::request` is reached only when the pipe has no other holder.
- No consumer-level control exists:
  the executable does not call these layers,
  so the ledger's "stage clean bytes, dirty the worktree, commit" observation and its `strace` count are open.
- Nothing here was compiled or run on Windows or macOS.
  The new modules hold no platform-specific variant;
  the adapter calls the foundation's `path_from_git_bytes`,
  whose non-Unix variant the Linux gate does not compile.
- Sparse and split indexes were not exercised.
- No timing was measured.
  The motivation for the rewrite was commit latency;
  this branch removes per-file processes and temporary files and proves the process count,
  and makes no claim about elapsed time.
- `file-enforcer` output for unrelated files was discarded,
  not committed.
- The Oxlint task was run for the two wrapper runner files only;
  the fuzz sidecar has no Oxlint task.

## What remains

- Merge the branch into `main`.
  `main` at `fb64be854` is already merged into the branch and that tree is gated;
  a merge into `main` conflicts only where `main` has moved since.
  See "Merging into `main`".
- The owner may veto any of the three adopted `policy-incomplete` cases under "Which code each failure carries".
- The engine changes under "What the engine must add",
  without which no event carries `policy-incomplete` from these layers.
- Images this delegation left on the host,
  by the names the runners' logs report:
  `localhost/git-policy-native-test` with the tags `candidates`,
  `candidates-b` and `candidates-c`
  (one image),
  `localhost/git-policy-native-mutation` with the same three tags
  (three images layered on it),
  `localhost/git-policy-cli-fuzz-build:candidates`
  and `localhost/git-policy-cli-fuzz-run:candidates`;
  and from the merged-tree runs,
  the same four names with the tag `candidates-merge`.
  The evidence directories name images by identity,
  so removing them loses no record.
- Call these layers from the policy engine:
  build the `forbidden-strings` policy over `scan_version`,
  map `ScanFinding` values and cache warnings to JSONL events,
  report failures with the codes the functions in `src/native/scanner_failure_code.rs` give,
  and read the `rulesFile` option and the variable at the one place under "Where the rules path is resolved",
  `scanner_selection::rules_source`.
- Install a payload-free panic hook in `main.rs` before the first scan.
- Candidate sources this branch does not provide:
  the `git add` staged delta between two index states
  (`src/policy-engine/add-staged-delta.ts`),
  tracked files that are not candidates
  (`commit-transaction-tracked-files.ts`),
  many commits in one listing for manual push
  (`manual-push-candidates.ts`),
  and worktree bytes for direct `check`
  (`direct-check-facts.ts`).
- Policy read sets and input fingerprints,
  which decide when a recorded result may be reused after a replay.
- The consumer-level controls named under "Failed or skipped".
