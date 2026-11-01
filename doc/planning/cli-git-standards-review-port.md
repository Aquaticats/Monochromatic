# Porting the standards-review shadow run into the Git wrapper

This page is the port plan for the post-commit hook `hook.standards-review-shadow`,
whose command starts `mise run shadow-review` (the inline TypeScript in the root `mise.toml`, variable `shadow_review`).
It records what that run reads, sends and writes,
the in-process modules that now do the same work,
the dependency choice,
the three ways to run it without a child process,
and the work-item record that carries after-commit work to a resident service.

Owner statements this follows, verbatim:

> I already said we don't allow 3p code and custom actions?

> Didn't I just say to embed everything in and port? Why are you still suggesting to execute children?

> child or resident proces" - another reason for server client architecture. Some policies are meant to be ran async.

> Avoid wasting compute on stuff that would be shortly superseded.

Status:
the pure parts, the gateway client, the bounded runner, the log writer and the work-item record are merged-ready and tested.
Nothing is wired into the commit path.
The old hook, the `shadow-review` task and `doc/audit/standards-review-gate-shadow.md` are untouched.

## What the run does today

The hook fires after each landed commit and starts a detached systemd unit running `mise run shadow-review`.
The script then does the following,
in this order.

- Resolve the commit.
  `git rev-parse --show-toplevel`,
  then `git rev-parse` of `SHADOW_REVIEW_SHA` or `HEAD`.
- Read the commit.
  The subject (`git log -1 --format=%s`, trimmed),
  the patch (`git show --format= --patch`),
  and the changed names (`git diff-tree --no-commit-id --name-only -r`).
- Decide whether to write an entry.
  No names (a root commit or a merge, because `diff-tree` prints nothing for them) or only the log file itself: no entry.
  No name ending in `.ts`, `.tsx`, `.rs`, `.js`, `.mjs` or `.cjs`: a three-line skip entry.
  Otherwise a review entry.
- Cap and excerpt.
  The patch is cut to its first 2000 lines.
  Up to 8 code files get an excerpt (stopping once 4000 excerpt lines are reached):
  the new-side file lines from 30 lines before the first changed old-side line to 30 lines after the last,
  taken from `git diff -U0` hunk headers,
  under a header `// candidate excerpt of <name>, lines <from> to <to> of <total>`.
- Build the model input.
  The 2000-line patch, a blank line, then the excerpts joined by blank lines, cut to 24000 UTF-16 code units.
- Ask the gateway.
  One POST to `https://api.llmgateway.io/v1/decisions`,
  model `openai/gpt-6-luna-decisions`,
  a single choice question `cost_verdict` with the answers `yes`, `partial`, `no`,
  `authorization: Bearer <key>`,
  a 30 s limit on the whole call.
- Append one entry to the log.

### What it reads

- The repository through Git (up to 5 + 2n Git starts for n excerpted files, 7 to 21, plus `mise`, `node` and `systemd-run`).
- The environment:
  `SHADOW_REVIEW_LOG` (log path, default `doc/audit/standards-review-gate-shadow.md` under the repository),
  `SHADOW_REVIEW_SHA` (commit to replay, default `HEAD`),
  `SHADOW_REVIEW_ALLOW` (`1` selects the full-review route),
  `SHADOW_REVIEW_LLMGATEWAY_API_KEY` and, only when that one is unset, `STOP_REMINDER_LLMGATEWAY_API_KEY`
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
- Two scratch files, `~/temp/agent/shadow-review/<sha12>.diff` and `.candidate`,
  which only the full-review route reads.
  The port does not write them.

The review entry fields,
one per line pair,
each value indented by two spaces:

- `Date`: `new Date().toISOString()` at the end of the call.
- `Subject`: the first 12 characters of the commit name, a space, and the subject in backticks.
- `Input size`: `<n> code files, <L> diff lines[, capped at 2000], luna input <N> chars` in backticks.
  `L` is the line count of the whole patch (a final newline counts as one more, empty line),
  `N` is the UTF-16 length of the capped model input.
- `Reviewer route`: `openai/gpt-6-luna-decisions via the Decisions API` in backticks.
- `Wall-clock cost`: the call's seconds with `toFixed(1)`, then ` s`.
- `Named superlinear cost`: `yes` or `no` (the model's top choice),
  a two-line `partial (cost gestured as unverified,` / `not named)`,
  or `unjudged` when the call failed or the answer has no choice.
- `Findings kept`: a five-backtick `text` fence holding one line,
  `cost_verdict: <choice> (<value> <probability> ...)`,
  or `Luna failed: <message>`.
- A blank line after the entry.

The skip entry is `- Skipped:`, then `<sha12> (no code files):`, then the subject in backticks.

### What it needs from the network

Outbound TCP and TLS to `api.llmgateway.io` port 443 and a name lookup.
The previous run needed the same from `node`.
No inbound connection,
no proxy (the port ignores proxy variables on purpose),
no other host.

### The full-review route is out of scope

`SHADOW_REVIEW_ALLOW=1` starts `pi` as a child and sends the whole `AGENTS.md`, the diff file and the excerpt file to
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
  the zero-context old-side ranges of one file,
  derived from the patch the review already holds.
- `standards_review_input.rs`:
  the plan (no entry, skip, review) from the changed names,
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
  the whole run, the options from the environment, and `run_within`, the bounded runner.
- `work_item.rs`:
  the durable work-item record.

Shared files touched:
`Cargo.toml` and `Cargo.lock` of the package (the `ureq` line and its locked crates),
`rust-catalog.toml` at the root (`ureq = ">=3.5.0"`),
and `src/native/lib.rs` (module declarations at the end).

### Git reads: two starts, none for most landings

The landing knows the commit name, its message and the changed names.
So the subject comes from the message (`commit_subject`),
and the plan comes from the names,
with no Git start.
A log-only commit or a commit with no code files therefore costs no Git start.
A commit with code files costs two:
one `git show --format= --patch --no-color --no-ext-diff <sha>` for the patch,
and one `git cat-file --batch` for the new-side text of the excerpted files.
The per-file `git diff -U0` start is replaced by reading the change runs out of the patch,
which `standards_review_patch_tests.rs` checks against real `git diff -U0` for replacements, insertions, deletions,
adjacent edits inside one three-line-context hunk,
new files,
renames and paths with spaces.

Two starts are still real Git.
The patch needs Git's diff engine;
reproducing its hunking and rename detection byte for byte in-process was not attempted,
because the wrapper links no object-database or diff library and adding one is a larger decision than this port.
The file texts could come from the wrapper if the post-landing step kept the blobs it already reads for added paths,
but that step reads only added paths, not modified ones.
Both are candidate removals for the resident service,
which can keep a repository open and read objects without a start per commit.

### HTTP client choice

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
  (`cargo tree` shows `ring` as the provider, `webpki-roots` as the roots, no async runtime):
  chosen.
  It adds `ureq`, `ureq-proto`, `http`, `httparse`, `rustls`, `rustls-webpki`, `rustls-pki-types`, `ring`, `webpki-roots`,
  `subtle`, `untrusted`, `zeroize`, `base64`, `bytes`, `itoa`, `log`, `percent-encoding`, `utf8-zero`, `cc`, `shlex`,
  `once_cell`, `getrandom` 0.2 and the Windows target crates the lock lists:
  31 new lock entries in total
  (`Cargo.lock` grew by 277 lines).
  Not in the local registry cache; fetched once from crates.io (read only).
- `rustls` plus `webpki-roots` plus a hand-written HTTP/1.1 exchange:
  about 12 crates against ureq's 22 on Linux,
  but it makes this repository own response framing (chunked bodies, content lengths, early close).
  Rejected: an existing maintained client passes every gate, so a custom one is not eligible.
- `reqwest` blocking or `hyper` with `hyper-rustls`:
  rejected for the runtime and threads they bring.
- `native-tls` and `openssl`:
  rejected, they link a system library.
- `aws-lc-rs` as the provider:
  rejected, it needs a C and CMake build; `ring` is what ureq's feature selects.

Scope of this vet:
candidates came only from memory, the repository's lock and registry cache, and a `cargo tree` of two probe crates,
which is the no-check-in subset of the repository's technology-choice skill.
No full saturation ledger or source audit of `ureq` was done;
that would be the next step if the dependency is kept.

Measured:
- The package's release build, baseline (no `ureq`): 8,593,400 bytes.
- With `ureq` in the dependency list and the new code unwired: 8,651,632 bytes (+58,232, +0.7 percent),
  because the linker drops code nothing reaches.
- A probe binary that does only a `ureq` HTTPS POST, same release profile: 2,552,184 bytes.
  So the first wired build should be expected to grow the wrapper by roughly 2 MiB,
  about 24 percent,
  which `Cargo.toml` says matters for the first start after eviction.
  This is the strongest reason to link the client into the resident service and not into every one-shot wrapper start,
  but both are the same binary today.
- Release build time of this package from an empty target directory:
  59 s baseline (with `--timings`, cold) and 21 s after.
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

Option (a), bounded synchronous wait in the wrapper:

- Cost:
  the landing waits for the call.
  The gateway figures on record are 0.3, 0.5, 0.4 and 1.4 s in `doc/audit/standards-review-gate-shadow.md`
  and 1.2 s in `doc/runbook/standards-review-gate.md`.
  That is five points, too few for a distribution,
  and all of them include a warm connection from the earlier `node` process or not,
  which the records do not say.
  Add a TLS handshake and two Git starts.
  The post-commit step runs under the hook lock,
  so the wait would hold that lock too.
- Bound:
  `run_within` gives a hard limit over reads, call and write.
  The loopback mock tests show the call ends at the deadline
  (`a_silent_gateway_costs_at_most_the_deadline`).
  A deadline above the largest recorded figure (1.4 s) would be needed to avoid logging timeouts for ordinary calls,
  so a few seconds,
  paid on every landing that has code.
- Failure: no landing fails (every error is an entry or an outcome), but every landing with code is slower.

Option (b), a durable work item for a resident consumer:

- Cost to the landing:
  one small file write in the Git directory (`write_private_file_unsynced`: a staged write and a link, no sync),
  no network, no Git start.
- Failure: if no consumer runs, items wait and the review does not happen.
  That is visible (see the notice below).
- Needs a resident process,
  which the owner has decided on (below).

Option (c), skip:

- Cost zero, coverage zero.
  The runbook says an empty log must mean the trigger did not run,
  so skipping makes the shadow log mean nothing.

Recommendation:
(b),
with (a) as the fallback for a one-shot environment only if the owner wants review without a resident process,
and (c) rejected.
The owner has since decided the resident single-writer service, so (b) is the contract, not a fallback.

## The work-item record

The record is generic so auto-push and maintenance can use the same queue.
It is the after-commit queue contract of the resident service,
agreed with the service's designer (`a6e4566588e9ec087`) in one exchange.

### Location, schema, writes

- Directory:
  `<common git dir>/cli-git/work/` with the state directories `pending`, `claimed`, `done` and `failed`.
  The state of an item is the directory that holds its file,
  so there is no state field to keep in step.
- File name, which is also the item id:
  `<kind>-<landed oid>` with kind `review`, `push` or `maintenance`.
  Derived from the kind and the object, so the same landing always gives the same id.
- Content:
  one JSON line with `v` (1), `kind`, `id`, `repository` (worktree root), `gitDir`, `ref` (branch reference),
  `oid`, `parent` (empty for a root commit), `created` (ISO time) and `tries`.
  A reader refuses another version, an unknown kind, a missing field and an id not derived from kind and object.
- Write:
  the existing whole-or-absent publish of `private_storage` (a staged file, then a link),
  so a kill at any instruction leaves the record complete or missing, never cut.
  A link that finds the name taken means the item exists and the enqueue is a no-op.
  The enqueue also checks `claimed`, `done` and `failed`, so a replayed landing adds nothing.
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
  then make its result durable (for `review`, append the log entry),
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
It never waits for a consumer, reads no result and starts nothing.
When a consumer is not running the queue grows;
the wrapper can notice this without any process identity:
`oldest_pending_age` returns how long the oldest pending item has waited,
and the proposed notice is one line on standard error when it exceeds a bound,
for example `cli-git: N after-commit items have waited T; no resident service is draining them`.
The bound and the wording are open for the owner,
and nothing prints this yet.

### Failure cases a resident process introduces

Which the record design avoids, and which remain:

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
  The script runs `git show <sha>:<name>` for it, which fails and ends the script with no entry at all.
- A root commit and a merge write no entry in both, because `diff-tree` prints no names for them.
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
  and the script then fails on it; the port takes the landing's exact names.
- A patch over 64 MiB is refused, as the script's `maxBuffer` refuses it, but with a named message.

Kept from the script on purpose, and worth the owner's attention:

- The excerpt window applies the old-side line numbers of each hunk to the new-side file.
  After an insertion earlier in the file the window is shifted.
  The port reproduces this so the two logs agree;
  a fix would use the new-side `+start,length` instead.
- A failure message that quotes the response body is written to the log unescaped, so a body with line breaks
  could forge entry lines.
  The gateway is a fixed TLS endpoint and the quote is 200 units,
  but replacing line breaks in the quoted body would remove the case.

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
- The wiring: nothing calls `review_landed`, `enqueue` or `run_within`.
  The wiring is one decision for the coordinator and the resident-service owner:
  the landing enqueues a `review` item and the service drains it with `review_landed`,
  with `read` bound to `standards_review_source::read_landed_commit`.
- Removing the old hook,
  the `shadow-review` task and the runbook's trigger text.
  That is a separate step after the service drains reviews.
