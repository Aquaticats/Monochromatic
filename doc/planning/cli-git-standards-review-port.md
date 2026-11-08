# Porting the standards-review shadow run into the Git wrapper

This page is the port plan for the post-commit hook `hook.standards-review-shadow`,
whose command starts `mise run shadow-review` (the inline TypeScript in the root `mise.toml`,
variable `shadow_review`).
It records what that run reads,
sends and writes,
the in-process modules that now do the same work,
the dependency choice,
the three ways to run it without a child process,
and the work-item record that carries after-commit work to a resident service.

Owner statements this follows,
verbatim:

> I already said we don't allow 3p code and custom actions?

> Didn't I just say to embed everything in and port?
> Why are you still suggesting to execute children?

> child or resident proces" - another reason for server client architecture.
> Some policies are meant to be ran async.

> Avoid wasting compute on stuff that would be shortly superseded.

Status:
the pure parts,
the gateway client,
the bounded runner,
the log writer and the work-item record are merged-ready and tested.
The wrapper now leaves a `review` work item and the service runs it (heading `Wiring`);
the old hook,
the `shadow-review` task and the runbook trigger are removed in the same change.
`doc/audit/standards-review-gate-shadow.md` itself is never written by this work except by the service's appends.
(Note,
2026-11-01:
the service's appends now go to the untracked `<common dir>/cli-git/standards-review-shadow.md`;
heading `Appending to the shared log`,
dated note.)

## What the run does today

The hook fires after each landed commit and starts a detached systemd unit running `mise run shadow-review`.
The script then does the following,
in this order.

- Resolve the commit.
  `git rev-parse --show-toplevel`,
  then `git rev-parse` of `SHADOW_REVIEW_SHA` or `HEAD`.
- Read the commit.
  The subject (`git log -1 --format=%s`,
  trimmed),
  the patch (`git show --format= --patch`),
  and the changed names (`git diff-tree --no-commit-id --name-only -r`).
- Decide whether to write an entry.
  No names (a root commit or a merge,
  because `diff-tree` prints nothing for them) or only the log file itself:
  no entry.
  No name ending in `.ts`,
  `.tsx`,
  `.rs`,
  `.js`,
  `.mjs` or `.cjs`:
  a three-line skip entry.
  Otherwise a review entry.
- Cap and excerpt.
  The patch is cut to its first 2000 lines.
  Up to 8 code files get an excerpt (stopping once 4000 excerpt lines are reached):
  the new-side file lines from 30 lines before the first changed old-side line to 30 lines after the last,
  taken from `git diff -U0` hunk headers,
  under a header `// candidate excerpt of <name>, lines <from> to <to> of <total>`.
- Build the model input.
  The 2000-line patch,
  a blank line,
  then the excerpts joined by blank lines,
  cut to 24000 UTF-16 code units.
- Ask the gateway.
  One POST to `https://api.llmgateway.io/v1/decisions`,
  model `openai/gpt-6-luna-decisions`,
  a single choice question `cost_verdict` with the answers `yes`,
  `partial`,
  `no`,
  `authorization: Bearer <key>`,
  a 30 s limit on the whole call.
- Append one entry to the log.

### What it reads

- The repository through Git (up to 5 + 2n Git starts for n excerpted files,
  7 to 21,
  plus `mise`,
  `node` and `systemd-run`).
- The environment:
  `SHADOW_REVIEW_LOG` (log path,
  default `doc/audit/standards-review-gate-shadow.md` under the repository;
  since 2026-11-01 the port's default is `<common dir>/cli-git/standards-review-shadow.md`),
  `SHADOW_REVIEW_SHA` (commit to replay,
  default `HEAD`),
  `SHADOW_REVIEW_ALLOW` (`1` selects the full-review route),
  `SHADOW_REVIEW_LLMGATEWAY_API_KEY` and,
  only when that one is unset,
  `STOP_REMINDER_LLMGATEWAY_API_KEY`
  (an empty first key does not fall back to the second),
  and `HOME` (scratch directory).

### What it sends

One HTTPS request body of at most about 24000 UTF-16 units of repository text
(the capped patch and the code excerpts),
the fixed instruction and the three answer descriptions,
and the bearer key in the `authorization` header.
Nothing else leaves the machine.

### What it writes

- One log entry (fields below) appended to the log.
- Two scratch files,
  `~/temp/agent/shadow-review/<sha12>.diff` and `.candidate`,
  which only the full-review route reads.
  The port does not write them.

The review entry fields,
one per line pair,
each value indented by two spaces:

- `Date`:
  `new Date().toISOString()` at the end of the call.
- `Subject`:
  the first 12 characters of the commit name,
  a space,
  and the subject in backticks.
- `Input size`:
  `<n> code files, <L> diff lines[, capped at 2000], luna input <N> chars` in backticks.
  `L` is the line count of the whole patch (a final newline counts as one more,
  empty line),
  `N` is the UTF-16 length of the capped model input.
- `Reviewer route`:
  `openai/gpt-6-luna-decisions via the Decisions API` in backticks.
- `Wall-clock cost`:
  the call's seconds with `toFixed(1)`,
  then ` s`.
- `Named superlinear cost`:
  `yes` or `no` (the model's top choice),
  a two-line `partial (cost gestured as unverified,` / `not named)`,
  or `unjudged` when the call failed or the answer has no choice.
- `Findings kept`:
  a five-backtick `text` fence holding one line,
  `cost_verdict: <choice> (<value> <probability> ...)`,
  or `Luna failed: <message>`.
- A blank line after the entry.

The skip entry is `- Skipped:`,
then `<sha12> (no code files):`,
then the subject in backticks.

### What it needs from the network

Outbound TCP and TLS to `api.llmgateway.io` port 443 and a name lookup.
The previous run needed the same from `node`.
No inbound connection,
no proxy (the port ignores proxy variables on purpose),
no other host.

### The full-review route is out of scope

`SHADOW_REVIEW_ALLOW=1` starts `pi` as a child and sends the whole `AGENTS.md`,
the diff file and the excerpt file to
`openai/gpt-6.1-sol`.
The port does not do this.
`full_review_requested` reports that the variable is set so a caller can say the route is not available.
The owner can ask for it separately;
it would need an embedded client for that provider instead of a child.

## Design of the port

### Modules

All under `package/git-policy/cli/src/native`,
each with a sidecar `_tests.rs`.

- `standards_review_text.rs`:
  UTF-16 length and prefix,
  ECMAScript number text,
  `toFixed(1)` of milliseconds,
  Git's `%s` subject from a commit message.
- `standards_review_patch.rs`:
  the new-side spans of the changed lines of one file,
  derived from the patch the review already holds.
- `standards_review_input.rs`:
  the plan (no entry,
  skip,
  review) from the changed names,
  and the capped model input.
- `standards_review_request.rs`:
  the request body in the script's key order and the reading of the answer.
- `standards_review_entry.rs`:
  the entry text and its single-write append.
- `standards_review_client.rs`:
  the gateway call.
- `standards_review_source.rs`:
  the Git reads.
- `standards_review_run.rs`:
  the whole run,
  the options from the environment,
  and `run_within`,
  the bounded runner.
- `work_item.rs`:
  the durable work-item record.

Shared files touched:
`Cargo.toml` and `Cargo.lock` of the package (the `ureq` line and its locked crates),
`rust-catalog.toml` at the root (`ureq = ">=3.5.0"`),
and `src/native/lib.rs` (module declarations at the end).

### Git reads: two starts, none for most landings

The landing knows the commit name,
its message and the changed names.
So the subject comes from the message (`commit_subject`),
and the plan comes from the names,
with no Git start.
A log-only commit or a commit with no code files therefore costs no Git start.
A commit with code files costs two:
one `git show --format= --patch --no-color --no-ext-diff <sha>` for the patch,
and one `git cat-file --batch` for the new-side text of the excerpted files.
The per-file `git diff -U0` start is replaced by reading the change runs out of the patch,
which `standards_review_patch_tests.rs` checks against real `git diff -U0` for replacements,
insertions,
deletions,
adjacent edits inside one three-line-context hunk,
new files,
renames and paths with spaces.

Two starts are still real Git.
The patch needs Git's diff engine;
reproducing its hunking and rename detection byte for byte in-process was not attempted,
because the wrapper links no object-database or diff library and adding one is a larger decision than this port.
The file texts could come from the wrapper if the post-landing step kept the blobs it already reads for added paths,
but that step reads only added paths,
not modified ones.
Both are candidate removals for the resident service,
which can keep a repository open and read objects without a start per commit.

### HTTP client choice

Note 2026-11-07T04:07:54-05:00:
superseded on the HTTP client by the owner's ruling of 2026-11-07
("Adopt tokio,
do not adopt ureq,
use the `hyper` set of https libs.").
The client is now `hyper` 1.x with `hyper-util`,
`hyper-rustls` (rustls,
embedded roots,
nothing read from the system)
and `http-body-util`,
on `tokio` as the wrapper's async runtime.
The hard constraint "blocking and in-process (the wrapper has no async runtime ...)" and the rejection of `hyper` with
`hyper-rustls` in the Ledger under the HTTP client choice heading were written before that ruling and are kept as written.
The prototype and its measurements are in `doc/research/wrapper-git-free-jobs-and-published-crates.md`,
section "Tokio as the runtime".

Hard constraints:
blocking and in-process (the wrapper has no async runtime and must start no program),
TLS with roots that do not depend on the host,
no system library (the binary is static-friendly and built offline),
a total deadline,
no redirect following and no proxy variable (the key and repository text go to one named endpoint).

Candidates already in `Cargo.lock` before the change:
none.
The lock held no HTTP or TLS crate.
The local registry cache held:
`hyper` with `hyper-rustls` and `tokio` (an async runtime),
`reqwest` (blocking mode still starts a runtime and a thread),
`rustls` with `ring` or `aws-lc-rs`,
`native-tls` and `openssl` (link the system OpenSSL),
`webpki-roots`,
`httparse`.

Ledger:

- `ureq` 3.5.0 with its `rustls` feature
  (`cargo tree` shows `ring` as the provider,
  `webpki-roots` as the roots,
  no async runtime):
  chosen.
  It adds `ureq`,
  `ureq-proto`,
  `http`,
  `httparse`,
  `rustls`,
  `rustls-webpki`,
  `rustls-pki-types`,
  `ring`,
  `webpki-roots`,
  `subtle`,
  `untrusted`,
  `zeroize`,
  `base64`,
  `bytes`,
  `itoa`,
  `log`,
  `percent-encoding`,
  `utf8-zero`,
  `cc`,
  `shlex`,
  `once_cell`,
  `getrandom` 0.2 and the Windows target crates the lock lists:
  31 new lock entries in total
  (`Cargo.lock` grew by 277 lines).
  Not in the local registry cache;
  fetched once from crates.io (read only).
- `rustls` plus `webpki-roots` plus a hand-written HTTP/1.1 exchange:
  about 12 crates against ureq's 22 on Linux,
  but it makes this repository own response framing (chunked bodies,
  content lengths,
  early close).
  Rejected:
  an existing maintained client passes every gate,
  so a custom one is not eligible.
- `reqwest` blocking or `hyper` with `hyper-rustls`:
  rejected for the runtime and threads they bring.
- `native-tls` and `openssl`:
  rejected,
  they link a system library.
- `aws-lc-rs` as the provider:
  rejected,
  it needs a C and CMake build;
  `ring` is what ureq's feature selects.

Scope of this vet:
candidates came only from memory,
the repository's lock and registry cache,
and a `cargo tree` of two probe crates,
which is the no-check-in subset of the repository's technology-choice skill.
No full saturation ledger or source audit of `ureq` was done;
that would be the next step if the dependency is kept.

Measured:
- The package's release build,
  baseline (no `ureq`):
  8,593,400 bytes.
- With `ureq` in the dependency list and the new code unwired:
  8,651,632 bytes (+58,232,
  +0.7 percent),
  because the linker drops code nothing reaches.
- A probe binary that does only a `ureq` HTTPS POST,
  same release profile:
  2,552,184 bytes.
  So the first wired build should be expected to grow the wrapper by roughly 2 MiB,
  about 24 percent,
  which `Cargo.toml` says matters for the first start after eviction.
  This is the strongest reason to link the client into the resident service and not into every one-shot wrapper start,
  but both are the same binary today.
- Release build time of this package from an empty target directory:
  59 s baseline (with `--timings`,
  cold) and 21 s after.
  These are single unalternated runs on a shared host and show the build time is not dominated by the new crates;
  they are not a measured difference.
- Forwarded-command latency before and after was not measured:
  nothing is wired,
  so the harness would time code that does not run,
  and the owner asked not to spend compute on work an architecture decision may replace.
  It should be measured once the client is linked on a path that executes.

### The hard constraint: a wrapper exits when its command ends

The review must never delay or fail a landing,
but a wrapper process ends with its command.
Without starting a child there are three options.

Option (a),
bounded synchronous wait in the wrapper:

- Cost:
  the landing waits for the call.
  The gateway figures on record are 0.3,
  0.5,
  0.4 and 1.4 s in `doc/audit/standards-review-gate-shadow.md`
  and 1.2 s in `doc/runbook/standards-review-gate.md`.
  That is five points,
  too few for a distribution,
  and all of them include a warm connection from the earlier `node` process or not,
  which the records do not say.
  Add a TLS handshake and two Git starts.
  The post-commit step runs under the hook lock,
  so the wait would hold that lock too.
- Bound:
  `run_within` gives a hard limit over reads,
  call and write.
  The loopback mock tests show the call ends at the deadline
  (`a_silent_gateway_costs_at_most_the_deadline`).
  A deadline above the largest recorded figure (1.4 s) would be needed to avoid logging timeouts for ordinary calls,
  so a few seconds,
  paid on every landing that has code.
- Failure:
  no landing fails (every error is an entry or an outcome),
  but every landing with code is slower.

Option (b),
a durable work item for a resident consumer:

- Cost to the landing:
  one small file write in the Git directory (`write_private_file_unsynced`:
  a staged write and a link,
  no sync),
  no network,
  no Git start.
- Failure:
  if no consumer runs,
  items wait and the review does not happen.
  That is visible (see the notice under "What the wrapper does and prints").
- Needs a resident process,
  which the owner has decided on (below).

Option (c),
skip:

- Cost zero,
  coverage zero.
  The runbook says an empty log must mean the trigger did not run,
  so skipping makes the shadow log mean nothing.

Recommendation:
(b),
with (a) as the fallback for a one-shot environment only if the owner wants review without a resident process,
and (c) rejected.
The owner has since decided the resident single-writer service,
so (b) is the contract,
not a fallback.

## The work-item record

The record is generic so auto-push and maintenance can use the same queue.
It is the after-commit queue contract of the resident service,
agreed with the service's designer (`a6e4566588e9ec087`) in one exchange.

### Location, schema, writes

- Directory:
  `<common git dir>/cli-git/work/` with the state directories `pending`,
  `claimed`,
  `done` and `failed`.
  The state of an item is the directory that holds its file,
  so there is no state field to keep in step.
- File name,
  which is also the item id:
  `<kind>-<landed oid>` with kind `review`,
  `push` or `maintenance`.
  Derived from the kind and the object,
  so the same landing always gives the same id.
- Content:
  one JSON line with `v` (1),
  `kind`,
  `id`,
  `repository` (worktree root),
  `gitDir`,
  `ref` (branch reference),
  `oid`,
  `parent` (empty for a root commit),
  `created` (ISO time) and `tries`.
  A reader refuses another version,
  an unknown kind,
  a missing field and an id not derived from kind and object.
- Write:
  the existing whole-or-absent publish of `private_storage` (a staged file,
  then a link),
  so a kill at any instruction leaves the record complete or missing,
  never cut.
  A link that finds the name taken means the item exists and the enqueue is a no-op.
  The enqueue also checks `claimed`,
  `done` and `failed`,
  so a replayed landing adds nothing.
- The consumer needs only the object name and the places to find the repository.
  It reads the patch itself,
  so the record carries no diff.

### What the consumer does

- Claim:
  rename the oldest `pending` file into `claimed` (atomic;
  of two claimants the loser finds the file gone and takes the next),
  then stamp the claim time as the file's modification time.
  That time is the lease start.
- Do the work for the kind,
  then make its result durable (for `review`,
  append the log entry),
  then rename the file into `done`.
- Finish a retry:
  a claim whose lease is older than the lease length goes back to `pending` with `tries` plus one,
  or to `failed` at the try limit.
  A record that does not decode goes to `failed` at once so it cannot block the queue.
- Delivery is at least once.
  A crash between the result and the move to `done` repeats the work after the lease,
  so a handler must tolerate a repeat.
  For `review` a repeat is a second log entry for one commit,
  which the entry's commit prefix makes detectable;
  the exactly-once variant (an exclusive result file written before the log append) was not built.
- Where the result is logged:
  the review appends to the log named by `SHADOW_REVIEW_LOG` or the default,
  in one write,
  as `standards_review_entry::append_entry` does.

### What the wrapper does and prints

The wrapper only enqueues.
It never waits for a consumer,
reads no result and starts nothing.
When a consumer is not running the queue grows;
the wrapper can notice this without any process identity:
`oldest_pending_age` returns how long the oldest pending item has waited,
and the proposed notice is one line on standard error when it exceeds a bound,
for example `cli-git: N after-commit items have waited T; no resident service is draining them`.
The bound and the wording are open for the owner,
and nothing prints this yet.

### Failure cases a resident process introduces

Which the record design avoids,
and which remain:

- The consumer is killed at any instruction.
  Avoided for the data:
  the item stays in `claimed` or `pending` and is retried after its lease.
  Remaining:
  a repeat of the work (at-least-once).
- A stale socket.
  Avoided:
  the record needs no socket;
  the producer writes a file,
  and the consumer finds work by listing the directory (or is told by a hint that may be lost).
  A resident service that also serves a socket still has this problem for its own clients,
  not for after-commit work.
- A stale record.
  Handled:
  leases expire,
  undecodable records go to `failed`,
  and `done` and `failed` are plain files a person can read and remove.
  Remaining:
  nothing prunes `done` yet.
- Process-id namespaces.
  Avoided:
  liveness is a lease on a file's modification time,
  never a process id,
  so a consumer in another container or namespace is treated the same.
  Remaining:
  clocks.
  The lease compares the consumer's clock with the file system's timestamps,
  so a shared directory across hosts with skewed clocks can expire a live lease early.
- A hung consumer.
  Handled by the lease only if it stops holding the claim;
  a hung consumer that keeps working past the lease may overlap with a retry.
  The lease must be longer than the longest handler,
  and a handler needs its own deadline (the review has the call deadline).
- Two consumers.
  Safe:
  rename is the single point of claim.
  Remaining:
  the requeue writes a temporary `.retry` file inside `claimed`,
  which a concurrent requeue could list;
  the consequence is at worst a repeated requeue attempt,
  but it was not tested concurrently.
- Power loss.
  The enqueue does not sync,
  so a lost `pending` file after power loss means a skipped review,
  which is acceptable for a shadow log and not for a push.
  A `push` or `maintenance` item must be recomputable from the refs,
  or be written with the journal's sync.

## Differences from the TypeScript script

Intentional,
each with its reason.

- One write per entry instead of one append per line,
  so concurrent runs cannot interleave entries.
- The scratch `.diff` and `.candidate` files are not written,
  because only the full-review route reads them.
- A deleted code file gets no excerpt.
  The script runs `git show <sha>:<name>` for it,
  which fails and ends the script with no entry at all.
- A root commit and a merge write no entry in both,
  because `diff-tree` prints no names for them.
- Failure wording:
  a timeout is `The operation was aborted due to timeout` as in Node;
  other transport failures are `fetch failed: <cause>` where Node prints only `fetch failed`;
  a body that is not JSON uses this port's wording where V8 prints its own parser message.
- A redirect is a failure (`HTTP 3xx ...`) where `fetch` follows up to 20.
- A cut that falls inside a surrogate pair drops the half pair,
  where the script sends a lone surrogate.
  The reported length is still the cut length.
- Names are taken as UTF-8 text.
  Git quotes a path with unusual characters in `diff-tree --name-only`,
  and the script then fails on it;
  the port takes the landing's exact names.
- A patch over 64 MiB is refused,
  as the script's `maxBuffer` refuses it,
  but with a named message.

Two defects of the script are fixed in the port,
on the coordinator's instruction that the script is going away and divergence is fine
("Fix even pre-existing issues of the pkgs you own").
Each changes the log or the model input,
so entries from the port and entries from the script are not comparable byte for byte for these cases.

- The excerpt window used the old-side line numbers of each hunk to cut the new-side file.
  After an insertion earlier in the file the window sat in the wrong place,
  and the upper end was one line too far.
  The port cuts the new-side file by the new-side first and last changed line of each change run
  (`standards_review_patch::new_spans`),
  checked against real `git diff -U0` new-side headers,
  including a case where an early insertion makes the old and new numbers differ.
  A file with no changed line to anchor on (a binary file,
  a mode change,
  a pure rename) got a negative window in the
  script (from the last 30 lines to line 31);
  it now gets the head of the file,
  `HEAD_WINDOW` lines.
  A renamed file with edits is now excerpted around its edits;
  the script read it as an addition and excerpted its first 31 lines.
- Text written into the log can no longer add lines.
  The script wrote a failure message that quoted the response body,
  and the gateway's answer text,
  unchanged,
  so a body with line breaks could start a forged entry.
  The port writes the subject and the `Findings kept` line through `standards_review_text::on_one_line`,
  which turns every control character and Unicode line or paragraph separator into a visible escape
  (`\n`,
  `\r`,
  `\t`,
  `\u{hex}`).
  The response quote is still cut at 200 UTF-16 units first.

## Where the client may be linked

Decided by the coordinator after the first report:
the review client lives only in the resident service.
No forwarded-command path and no commit-client path may link `ureq`,
because the probe showed it adds about 2 MiB to a binary that every `git` start loads.

The boundary is structural and asserted by `standards_review_boundary_tests.rs`,
which reads the crate's own source:

- Only `standards_review_client.rs` (and its sidecar) may name the HTTP library.
- Only `lib.rs` (the module declarations),
  `standards_review_client*` and `standards_review_run*` may name the client
  or the run module.
  Everything the service shares with other code (`_input`,
  `_request`,
  `_entry`,
  `_source`,
  `_patch`,
  `_text`,
  `work_item`) is free of the network client,
  so the wrapper can enqueue a work item without linking it.
- The test has a positive control on the real tree (it must see the library inside the client file and must find the
  client-path files `main.rs`,
  `entry.rs`,
  `forwarding.rs` and `commit_command.rs`),
  and a planted `use` of the client module and a planted `ureq::` call must each be reported.

What this does not prove:
a build of the executable without the service still compiles the library crate that contains the client,
and the linker drops it only because nothing reaches it.
When the service is a separate entry,
the next step is a cargo feature that removes the client from the client build,
and a symbol check on that build.
Nothing in the repository measures the release binary for symbols today.

## Wiring

Done in the change that removed the old trigger.

- The wrapper (`service_handoff.rs`,
  `hand_over`) leaves a `review` work item for the landed commit,
  after the push and maintenance items,
  only when the worktree holds `doc/audit/standards-review-gate-shadow.md`.
  The tracked file is the repository's own switch;
  no configuration names a program,
  and other repositories the service serves get no review.
  The check is `standards_review_input::has_shadow_log`,
  a stat of one path,
  with no network code behind it.
- The service (`service_work.rs`,
  `drain`) claims `review` items with the push and maintenance items,
  checks them with `is_acceptable`,
  and runs each one after the pushes and upkeep of the same round,
  one review per commit,
  never coalesced.
  `GitHandlers::review` calls `service_review::review_item`.
- `service_review.rs` reads the names (`git diff-tree -z`) and the message (`git show --no-patch --format=%B`) of the
  commit,
  then calls `review_landed`:
  for a commit with code,
  two more starts (the patch and one `cat-file --batch`).
  A service review is therefore two Git starts for a commit without code and four for one with code.
  The old script made seven to twenty-one,
  plus `mise`,
  `node` and `systemd-run`.

### A live failure and its fix

Seen on the owner's machine with 0.5.4:
every review failed with
`starting git cat-file failed: cli-git has no repository scratch directory for the input of this Git command`.
The service's own Git context names no repository (one service serves many),
and a Git command that takes input writes
that input to a file in the repository's Git directory.
The landing sets the directory per request;
the review handler did not.
`service_review::review_item` now sets `common_dir` to the work item's Git directory,
which `is_acceptable` has already
checked against the wake's directory.

`cat-file` stays.
The patch holds only the hunks,
the wrapper links no object reader,
and the worktree may have moved on since the commit,
so the new-side file texts come from the object store;
the other way to avoid input is one `git show` start per file.
The test that found it,
`a_review_runs_through_the_real_handlers_in_a_service_context`,
runs a pending review item
through `run_round` and the real `GitHandlers` with a context that names no repository,
a real fixture repository and
the loopback gateway;
before the fix it failed with the message above.
The unit tests had missed it because the shared test context names a repository.
Other service handlers were checked for the same pattern:
push uses only the Git path,
the configuration read and the
maintenance run start Git without input,
and the landing sets the directory per request.

### Bounds

The review shares the service's work thread with pushes,
so every step is bounded.

- Gateway call:
  `CALL_DEADLINE`,
  10 s,
  covering connect,
  TLS,
  send and read.
  The largest figure on record is 1.4 s;
  the script allowed 30 s.
- Whole review:
  `BUDGET`,
  20 s,
  covering the Git reads,
  the call and the write.
  `run_within` abandons a review that runs past it,
  and its item is settled as `failed` naming the budget.
  An abandoned review that finishes later may still append its entry once;
  the one-entry-per-commit check keeps a repeat from adding a second.
- No retry inside a review.
  A gateway failure is an entry in the log (`Luna failed: ...`) and the item is `done`.
  An unreadable commit,
  an unwritable log or a spent budget is a `failed` item,
  kept as a plain file,
  with one line on the service's standard error,
  and no notice for the person.
  A claim whose service died is taken back by its lease (30 min) and run again,
  up to three tries,
  as for the other kinds.
- A repeated item (a crash after the append,
  before the settling) asks nothing and adds nothing.

### Appending to the shared log

The log is a tracked file in the worktree that people and agents edit while the service appends to it,
and the live copy in the main checkout may carry another session's uncommitted edits.
No lock may be taken.
`standards_review_append.rs`:

- Only append.
  One `O_APPEND` write of the whole entry,
  never a read-modify-write,
  a truncation or a rename,
  so existing content is never touched and two appends cannot interleave.
- Never create.
  A missing log is reported and left missing,
  because creating it would race the session about to write it.
- Once per commit.
  The log is searched for the first 12 characters of the commit name before anything is asked or written;
  those characters survive the markdown reflow that already rewrites entries in this file.
- Repair.
  What an append cannot prevent is another writer that read the file before the append and writes the whole file back
  after it,
  which loses the entry.
  So each entry is also kept as a plain file under `<git dir>/cli-git/review-entries/<worktree hash>/`,
  and every delivery checks the latest 16 kept entries against the log and appends again the ones it no longer names.
  The kept entries are trimmed to 64.
  The spool is per worktree because a worktree on another branch lacks the commits a sibling made.
  The repair is bounded and best effort:
  it can run only when another review is delivered,
  and an entry that someone deliberately removed from the log within the window is put back once more.

What is not covered:
an editor that holds the whole file in memory and saves it long after a later append is repaired only at the next delivery,
and on a quiet day nothing is delivered.
The kept entry is still there to read.

Note,
2026-11-01 (owner request:
the tracked log "shouldn't be updated almost every commit by automations"):
this design is superseded for where entries go.
The service now appends to the untracked `<common dir>/cli-git/standards-review-shadow.md`,
one log per repository shared by every worktree's service,
created (with its directory) when missing.
The tracked `doc/audit/standards-review-gate-shadow.md` stays the switch,
as `standards_review_input::has_shadow_log` reads it,
and a commit that changes only that file still gets no entry;
the service never writes it.
The text under "Appending to the shared log" describes the removed behavior and is kept as the record:
the spool (`<git dir>/cli-git/review-entries/`),
the repair,
and the never-create rule were dropped
because only a tracked file that sessions edit needed them;
an old spool directory may remain on disk and nothing reads it.
Kept:
only append,
one `O_APPEND` write of the whole entry (a cut-short write is continued with the rest of the same entry),
and once per commit by the 12-character test.
Concurrent appends from several instances are covered by `concurrent_appends_land_whole`,
which failed when the entry
was written line by line.
`SHADOW_REVIEW_LOG` still overrides the path;
the entry format is unchanged.

### Guard evidence

Each guard was removed with the test that watches it run,
and the test failed;
the file was then restored.

- Review budget (`service_review.rs`):
  `the_budget_bounds_the_whole_review` failed after 4.0 s with the budget removed.
- Shadow-log gate in the handler:
  `a_repository_without_a_shadow_log_is_left_alone` failed.
- Shadow-log gate in the wrapper:
  `a_review_record_is_left_only_where_the_repository_keeps_a_shadow_log` failed.
- Never create:
  removing only the open mode left `a_missing_log_is_not_created` passing,
  because the read of the log
  before the append is a second guard for the same case;
  removing both made it fail.
- One entry per commit:
  `a_commit_is_added_once_even_after_a_reflow` failed.
- Repair:
  `an_overwritten_entry_is_put_back` failed.
- A failed review is settled as failed:
  `a_failed_review_is_a_plain_record_not_a_stuck_item` failed.
- Gateway deadline:
  `a_silent_server_ends_at_the_deadline` failed after 1.6 s.
- Log text on one line:
  `hostile_text_cannot_add_lines` failed.
- Client boundary:
  a planted import of the client module in `service_handoff.rs` made
  `the_network_client_is_reachable_only_from_the_service_run` fail.

### The review credential

Decided by the coordinator,
owner veto open:
the wrapper passes the key from its own environment to the service
in the in-memory `work` message over the user-only socket.
Never in the on-disk work item,
a unit file or the manager's environment.

- The wrapper (`service_handoff::hand_over`) reads `SHADOW_REVIEW_LLMGATEWAY_API_KEY`,
  else
  `STOP_REMINDER_LLMGATEWAY_API_KEY` (an empty first value is no key,
  as in the script) and sends it as the optional
  `reviewKey` field of the `work` message (`service_client::wake`).
  The records written before the message hold no key field.
- `ReviewKey` (`service_review_key.rs`) prints as redacted in every `Debug` output,
  so a log line or a failed assertion
  cannot show it;
  only `expose` returns the text,
  and only the request header and the message encoder call it.
- The service holds the key in `GitHandlers` per repository for the round that brought it (`service_work::run_round`),
  and forgets it when that repository has been drained.
  `review_variables` answers the two credential variable names from that key alone,
  and never from the service's own
  environment;
  every other variable still comes from the environment.
- A review that has no key records the plain `Luna failed: no ... API_KEY` outcome and contacts no one.
- The audit file is the only place a review writes,
  and a test checks that the log never holds the key.

Guard evidence,
each guard removed with the test that watches it,
which then failed:
the key used by the review (`the_key_from_the_message_reaches_the_gateway`),
no key invented when none came (`a_review_without_a_key_records_no_key`),
the environment not read for the second credential name (`the_service_environment_never_supplies_a_key`),
the on-disk records free of the key,
scanned byte by byte over the worktree and the Git directory
(`the_on_disk_work_items_never_hold_the_key`),
redacted printing (`a_key_never_prints_its_text`),
the key sent in the message (`the_wake_message_carries_the_key_to_the_service`),
the key read from the message (`a_work_message_carries_the_review_key_and_hides_it_when_printed`),
and forgotten after the round (`a_round_forgets_the_credentials_it_was_given`).

Not covered:
a real service socket and unit,
and a request to the real gateway;
the key reaches the review in tests
through the loopback mock only.

### Removed with the wiring

The local Git config keys `hook.standards-review-shadow.event` and `.command` in the shared Git config
(read from `file:/var/home/user/Monochromatic/.git/config`;
installed by commit `17241cbad5`,
this repository's own work,
so removed),
the `shadow-review` task and the `shadow_review` variable in the root `mise.toml`,
and the trigger text of `doc/runbook/standards-review-gate.md`.
The runbook's full-review route (`SHADOW_REVIEW_ALLOW=1`,
`pi`) went with the task.

## Not done

- The differential run of five real commits against the TypeScript script was not done.
  The owner asked not to spend compute on a script that is going away.
  What replaces it:
  every pure function is tested against its stated relation,
  the patch ranges are tested against real `git diff -U0`,
  and the request body is tested by parsing.
  Not exercised:
  byte equality of the whole entry and request body against the script for real commits,
  a TLS handshake (no local TLS server and no real call),
  and the 24000-unit cut on a real repository text with a split surrogate pair.
- Forwarded-command latency before and after (above).
- The gateway key travels in the work message (heading `The review credential`);
  a service restarted before the review
  runs has none,
  and that one review records `Luna failed: no ... API_KEY`.
- No end-to-end run through a real service socket and a real systemd unit:
  the handler,
  the drain and the handoff are tested separately against the loopback mock.
