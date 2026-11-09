# Standards-review route findings on `main`

## Scope and purpose, 2026-11-09T06:55:44-05:00

Audited revision:
`baf3d03fe668432f127e024275edeb3b9a668281`,
the `main` snapshot checked out when this audit began.
Only Linux was examined.
Unmerged branches were not inspected or incorporated.
This is an audit with suggested fixes,
not authorization to implement them.
Production code is unchanged.
The temporary Rust controls are not part of the committed change.

The route is an advisory shadow measurement,
not an enforcement gate and not a patch generator.
It asks whether a standards review would name superlinear cost in a landed commit,
then appends the choice and probabilities to an untracked repository log.
A `yes` answer neither rejects nor rewrites the commit.
`standards_review_patch.rs` parses the local Git patch to locate excerpts;
it does not apply a patch from the server.

The callers and data flow are:

- `package/git-policy/cli/src/native/commit/commit_command.rs:276` hands work over after landing and the post-commit gate.
  A handoff error is printed,
  but this path still returns exit `0` at line 281.
- `package/git-policy/cli/src/native/landing/service_handoff.rs:101` leaves a review item only if the worktree has
  `doc/audit/standards-review-gate-shadow.md`.
  The tracked file is a switch and historical log,
  not the destination for new entries.
- `package/git-policy/cli/src/native/landing/service_work_drain_pending_claims.rs:204` runs reviews after the other work.
  `package/git-policy/cli/src/native/landing/service_work.rs:386` calls the review handler.
- `package/git-policy/cli/src/native/service/service_review.rs:203` reads the changed names and message,
  then calls `review_landed` at line 216.
  `package/git-policy/cli/src/native/standards_review/standards_review_source.rs:198` reads the patch and excerpt blobs.
- `package/git-policy/cli/src/native/standards_review/standards_review_input.rs:279` builds the model input.
  `package/git-policy/cli/src/native/standards_review/standards_review_request.rs:89` JSON-encodes it with fixed instructions,
  choices and model name.
  The HTTP request adds the bearer header and JSON content type.
- `package/git-policy/cli/src/native/standards_review/standards_review_run.rs:312` delivers the rendered entry.
  The default destination is
  `<common Git directory>/monochromatic-git-policy-cli/standards-review-shadow.md`.
  `SHADOW_REVIEW_LOG` can explicitly replace that path.

No real model endpoint was contacted.
No credential was read.
The controls supplied invented bearer text directly in `ReviewOptions` and used
`standards_review_mock.rs` on loopback.
All fixture files,
including the invented outside-repository file,
were inside this audit worktree.

## Requirements and existing records

The owner's requirements supplied with this audit are:

- "Commits should never fail."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "commits vs native git should not take 5s longer."

The normal conditions include a process killed at any instruction,
a closed terminal or broken pipe,
slow fsync,
unusual filesystems,
and DNS or mirror trouble.
These are operating conditions,
not reasons to dismiss a sequence.

`rg` over `doc/decision/` found no standards-review-specific decision.
The matching rewrite record,
`doc/decision/cli-git-rust-rewrite.md`,
requires preserving the existing required policy,
transaction,
concurrency,
recovery,
worktree and push behavior.
The earlier policy-platform record is superseded where that rewrite removes executable configuration.
Neither record authorizes sending unrelated files to a review endpoint.

The relevant later owner decision is recorded in
`doc/handover/cli-git-rust-implementation.md:6441`:
"Silence plus a ceiling (Recommended)" for non-Git tools,
including standards review.
The owner subsequently chose "5x (Recommended)" for the measured ceilings at line 6480.
`package/git-policy/cli/SPEC.md:2653` specifies a 60-second call ceiling,
a 10-second service quiet window,
and a 120-second wait for the whole review.
The 30-second quiet window is the script-compatible default,
not the service setting.

The handover's `Open review findings` section was read before reporting.
It does not record the disclosure,
receipt or quoted-path findings in this page.
Its later review-runner history does record the abandoned-worker limitation;
that is not presented here as a new discovery.

## Findings ranked by risk

### Filename framing can send an unchanged committed file as another file's excerpt

Status:
**Reproduced** for a line-feed filename.
Risk:
confidentiality and incorrect review input,
without a configured diff converter or a malicious server.

Locations:
`package/git-policy/cli/src/native/standards_review/standards_review_source.rs:158`,
`:204`,
`:206` and `:127`.

Sequence:

1.  Commit `a.ts` and `private.ts` as the base.
    Put the invented marker `AUDIT_UNCHANGED_COMMITTED_PRIVATE_MARKER` only in `private.ts`.
2.  Create a file whose literal name is `a.ts\nHEAD:private.ts`,
    where `\n` denotes one line-feed byte,
    and create `z.ts`.
    Commit only those new files.
3.  `read_names` correctly receives the NUL-delimited changed names.
    `read_landed_commit` then interpolates each name into a line-delimited `cat-file --batch` request.
4.  The newline turns the first path into separate object requests.
    Git returns the unchanged `a.ts`,
    then `HEAD:private.ts`,
    then the actual `z.ts`.
    `parse_batch` consumes one reply per requested filename,
    labels the private blob `z.ts`,
    and does not reject the unconsumed reply.
5.  Run the real input builder,
    request encoder and review against the local mock.
    The marker from the unchanged private file is in the HTTP request under the wrong excerpt name.

Observed output:

```text
# standards_review_route_audit_tests.rs, local mock capture
AUDIT newline: unchanged_private_blob_mapped_to_z=true unchanged_private_blob_sent=true
```

This does not read an untracked file directly:
it discloses a different committed blob that was not part of the reviewed change.
The injected `HEAD` also resolves at review time,
not necessarily to the item's landed object.

Suggested fix:
retain raw path bytes,
use unambiguous object-request framing,
and verify the reply count and identity before accepting excerpts.
Reject or explicitly skip an unrepresentable name rather than substituting another blob.
This is a suggestion,
not a selected implementation.

### A local text converter exports ignored and outside-repository text

Status:
**Reproduced**.
Risk:
confidentiality beyond the staged and committed candidate.

Location:
`package/git-policy/cli/src/native/standards_review/standards_review_source.rs:75`.
The command disables external diff drivers but does not disable text conversion.
The resulting text enters the model input at
`package/git-policy/cli/src/native/standards_review/standards_review_input.rs:297`.

Sequence:

1.  In a disposable repository only,
    configure `diff.audit.textconv` as `cat ignored.txt ../outside.txt`.
    Add `*.ts diff=audit` to `.gitattributes`.
2.  Ignore `ignored.txt` and leave it unstaged.
    Put `AUDIT_IGNORED_UNSTAGED_MARKER` in it.
    Put `AUDIT_OUTSIDE_REPOSITORY_MARKER` in the sibling `outside.txt`,
    outside the fixture repository but still inside the audit worktree.
3.  Commit `.gitattributes`,
    `.gitignore` and a new `a.ts`.
4.  Run `read_landed_commit` for that commit.
    Its `git show --patch --no-color --no-ext-diff` executes the local converter.
    `--no-ext-diff` does not prevent this conversion.
5.  Capture the real review request with the route's local mock.
    Both markers are transmitted as patch text,
    although neither file was staged or committed.

Observed output:

```text
# standards_review_route_audit_tests.rs, local mock capture
AUDIT textconv: ignored_unstaged_sent=true outside_repository_sent=true
```

This needs a configured converter;
`.gitattributes` alone does not install its command.
The audit neither installed a converter in the real repository nor opened a real secret.
The problem is exporting a local presentation command's output as if it were committed candidate bytes,
not a claim that Git's converter is sandboxed.

Suggested fix:
read a patch with text conversion explicitly disabled,
and keep the source boundary at immutable committed objects.
Test a converter that reads ignored and outside text as a negative control.

### The remote response has no byte bound before collection and parsing

Status:
**Reproduced** for a bounded oversized error response;
unbounded allocation and process-exhaustion consequences are **read from the code only**.
Risk:
availability of the service process,
not a reproduced failed commit.

Locations:
`package/git-policy/cli/src/native/standards_review/standards_review_client.rs:371` to `:374`,
and `package/git-policy/cli/src/native/standards_review/standards_review_request.rs:168` to `:169`.

Sequence:

1.  Have the local mock return status `503` with a 1,048,576-byte body.
2.  Call the production `post_decision` with invented authentication and the mock URL.
3.  The client reads and collects every body byte,
    converts the aggregate to a byte vector and then a string,
    and returns that whole string.
4.  Only `read_answer`,
    after collection,
    rejects the status and retains a 200-UTF-16-unit diagnostic excerpt.

Observed output:

```text
# standards_review_route_audit_tests.rs, local mock capture
AUDIT response: status=503 collected_bytes=1048576 retained_failure_units=200 server=Whole
```

The 200-unit constant is explicitly a display cap,
not an existing response-memory contract.
The defect is the absence of any separate response-byte boundary,
not failure to treat 200 as that boundary.
A server can supply an arbitrarily larger finite body before the call ceiling,
with allocation proportional to its bytes;
the time ceiling is not a memory bound.
Success bodies are also collected whole and then parsed into a JSON value.
An allocation failure is not a normal returned transport error.
No exhaustion experiment was run,
and this page does not claim an observed service death or commit failure.

This finding touches "Commits should never fail."
and "nothing should ever wait forever or indefinitely":
the remote call is advisory but uses the service's process and runtime resources.

Suggested fix:
bound bytes while streaming,
including chunked and error responses,
before aggregation and JSON parsing.
Derive the bound from the actual decision-answer workload or an owner decision,
not from the display excerpt constant.

### Arbitrary log text and an interrupted prefix count as completed reviews

Status:
**Reproduced** through the remote-answer seam and a seeded interrupted-write state.
No process was killed during a real filesystem write.
Risk:
a review is silently lost or a fragment is permanently accepted as its completed record.

Locations:
`package/git-policy/cli/src/native/standards_review/standards_review_append.rs:53` to `:61`,
`:95` to `:96` and `:117` to `:123`;
`package/git-policy/cli/src/native/standards_review/standards_review_run.rs:289` to `:290`;
`package/git-policy/cli/src/native/service/service_review.rs:294`.

The completion test is a substring search for the first 12 commit-name characters anywhere in the whole log.
It does not require that commit's entry,
a complete entry,
a successful verdict,
or the full object name.

Remote-answer sequence:

1.  Review commit A against the local mock.
    Return a valid `choice: no` answer whose probability `value` names the first 12 characters of commit B.
2.  The answer reader accepts that arbitrary value and the entry renderer puts it in A's outcome text.
3.  Review B.
    `names_commit` finds B's prefix inside A's outcome and returns `AlreadyLogged` before reading B or calling the gateway.
4.  The service maps `AlreadyLogged` to `Verdict::Done`,
    although no B entry exists.

Interrupted-write sequence:

1.  Obtain a complete entry from a successful loopback review as a positive control.
2.  Seed the log with precisely that entry's prefix ending after the commit token,
    before the subject and verdict.
    This models a short append followed by termination before `write_all` continues.
3.  Retry the same review.
    The fragment's token is enough for `AlreadyLogged`;
    the reader is never called and the fragment is left unchanged.

Observed output:

```text
# standards_review_route_audit_tests.rs, local mock and interrupted-state seam
AUDIT token: next=AlreadyLogged target_entry_present=false
AUDIT fragment: outcome=AlreadyLogged fragment_unchanged=true
```

The two sequences share the same unstructured receipt check and are one finding.
Commit subjects containing another object's prefix can supply the same false token without a remote answer.
A logged gateway failure also names its own commit,
so the same check does not distinguish that failure from a usable verdict on a later attempt.

This touches the owner's "never discard user data"
and normal operation with "a process killed at any instruction":
there is no proof of complete delivery before recovery suppresses the remaining review text.
`O_APPEND` protects a write's file position;
it does not make `write_all` and a process lifetime an indivisible entry transaction.
Queue recovery may requeue an expired claim,
but this log check then suppresses the retried review.
Nothing in this route repairs the fragment.

Suggested fix:
use a full-object-name completion receipt installed only after a complete entry is durably available.
Keep incomplete text recoverable instead of treating its token as success or deleting it.
Do not let subjects,
probabilities or failure-message text create receipts.
A lock-free per-entry publication scheme is a suggestion consistent with the existing append-only design,
not a decision to add a repository-wide lock.

### Quoted Git paths silently receive the head excerpt instead of changed-line context

Status:
**Reproduced**.
Risk:
incorrect review evidence presented without a source-read failure.

Locations:
`package/git-policy/cli/src/native/standards_review/standards_review_patch.rs:101` to `:103`,
and `package/git-policy/cli/src/native/standards_review/standards_review_input.rs:256` to `:258`.

Sequence:

1.  Commit `quote".rs` with 122 unchanged lines.
2.  Append `AUDIT_LATE_CHANGED_LINE` as line 123 and commit the file.
3.  Read the patch and new-side blob through the production source reader.
    Git quotes and escapes the filename in the patch header.
4.  `section_is_for` compares that header with an unquoted string containing the raw name.
    It finds no section and returns no spans.
5.  The builder silently treats this like a legitimately hunkless file and sends lines 1 to 61 as the candidate excerpt.

Observed output:

```text
# standards_review_route_audit_tests.rs, local mock capture
AUDIT quoted_path: actual_changed_line=123 parsed_spans=[] head_fallback_sent=true
```

The patch still contains the changed line in this fixture.
The wrong result is the excerpt selection and lost changed-line context,
not a claim that the changed line was absent from the complete HTTP request.
The same raw-header comparison is used for quoted rename headers.

Suggested fix:
parse Git's quoted path representation,
or use a byte-preserving structured diff source.
Distinguish a section that is truly hunkless from failure to match a section known to contain the changed path.

## Verification state

The temporary controls ran through the package's nextest task,
not a rewritten implementation.
The initial run reported:

```text
# mise run //package/git-policy/cli:native:test:host -- route_audit --no-fail-fast --failure-output=final --success-output=final
Starting 6 tests across 1 binary (5093 tests skipped)
Summary [   0.118s] 6 tests run: 0 passed, 6 failed, 5093 skipped
```

Those failures were the intended boundary assertions,
with the captured wrong results quoted in each finding.
The command exited `100`.
The first broader baseline attempt ended at its 240-second command deadline during dependency compilation,
with exit `124`,
not a failed route assertion.
The resumed filtered build completed and exercised the named controls.
A replacement launch was stopped before testing so its fixture root could be explicitly pinned inside this worktree.

Audit controls used `CPUQuota=400%`,
`MemoryMax=8G`,
`TasksMax=512` and `IOWeight=10`,
with an outer 900-second build-and-test deadline.
The first baseline used `CPUQuota=200%`,
`MemoryMax=4G` and `TasksMax=256`.
No synthetic host load,
mutation campaign or full crate test run was used.

The native-byte path control,
existing route regression checks,
Markdown fixing,
linting and rendered inspection are still being completed.
Their final results will replace this verification-state paragraph.

## Limits of this audit

- The interrupted append was reproduced by seeding its exact prefix state,
  not by racing a signal against an actual short filesystem write.
  Power-loss durability,
  NFS append behavior and actual allocation failure were not measured.
- No native-Git versus wrapper latency comparison was run.
  The remote HTTP call is reachable from the service handler,
  not the commit-client path;
  that establishes call placement,
  not a measured guarantee that total wrapper overhead stays under 5 seconds.
- Local log opens,
  reads and writes have no individual cancellation bound.
  `run_contained_in` bounds the service's wait,
  but leaves a stalled worker outstanding and refuses further reviews until it ends.
  This behavior is already specified in `package/git-policy/cli/SPEC.md:2665`,
  not a newly discovered infinite wait in the HTTP client.
- The ceiling diagnostic suggests `SHADOW_REVIEW_SHA=<commit> mise run shadow-review`.
  The root task is absent in this snapshot,
  and `doc/runbook/standards-review-gate.md:117` says nobody can start it by hand.
  A working manual retry path was not established.
- Root commits and merges yield no changed names through this route and therefore no entry.
  The source reader documents that behavior;
  this audit does not reinterpret it as a successful cost verdict.
- Binary content and non-UTF-8 blob bytes are decoded lossily,
  not rejected as binary.
  The input caps limit transmitted text,
  not every source allocation.
  No large-blob or patch-limit exhaustion probe was run.
- Windows,
  macOS,
  network filesystems and worktrees whose repository-root path is non-UTF-8 were not exercised.

## What was checked and found sound

The following are source-verified boundaries;
regression execution results are recorded in `Verification state`.

- The request body is JSON-encoded rather than concatenating unescaped source text into JSON syntax.
  The bearer credential is a separate HTTP header.
  The production endpoint is fixed,
  redirects are not followed,
  and the client does not read proxy variables.
- Ordinary excerpts come from `cat-file` objects,
  not direct reads of unstaged worktree files.
  The filename and text-converter findings are concrete exceptions to claiming that only candidate text can leave.
- The patch module only computes line spans.
  Remote answers reach the log renderer,
  not filesystem path selection,
  the index,
  branch updates or patch application.
  No route was found for an answer to rewrite a source file or choose an outside destination.
- The final log component is opened with `O_NOFOLLOW` on Unix,
  append mode is used,
  and newly created files use mode `0600`.
  This is not a claim of no-follow validation for every ancestor directory,
  or that a user-supplied `SHADOW_REVIEW_LOG` is constrained to the repository.
- Call silence and the overall HTTP ceiling are separate checks.
  The total ceiling wraps the exchange,
  including resolution and connection setup.
  The whole-review wait also has a ceiling and a bounded handoff wait.
- Gateway errors become explicit `Luna failed: ...` entries,
  not commit-policy failures.
  Unreadable commits and unwritable logs become failed work items.
  Reviews run after the pending pushes in a work round.
