# Graph driver 815591bf misses staged source changes and detached-command cancellation

## Symptom

The research graph at `815591bf307b1237d31b1c4e644463d7c13654fa`
accepts a modified and staged tracked source file.
Terminating its Node driver with `SIGTERM` leaves a detached owning command alive.
These are repository harness defects;
Git `2.39.5` and Node `26.12.0` behave according to their documented interfaces.

The [independent receipt][receipt] and [original source archive][sources]
retain actual driver execution against explicitly fake owning commands.
No application test case was executed by these fixtures.

GitHub run `39135886118` is a separate prerequisite failure:
all its graph steps were skipped after setup omitted configured `cargo-binstall`.
The [existing Mise troubleshooting record][mise-prerequisite] explains that failure.
It does not establish either a product-test failure or successful graph execution.

## Root cause

### The guard compares the working tree with the index

The original `changedSource` in `.depot/research/graph/mise.toml:9`
checks committed top-level object IDs,
then obtains uncommitted paths with this decoded driver expression:

```javascript
// .depot/research/graph/mise.toml, decoded campaign task at 815591bf.
execFileSync('git', ['diff', '--name-only'], { cwd: root, encoding: 'utf8' })
```

Staging the changed file makes its index and working bytes agree.
The committed object IDs remain unchanged.
Both checks therefore accept that changed working file.

Git source tag `v2.39.5`,
commit `cc7d11c16782041a6bb73e2fb56417b7d4c6d186`,
`Documentation/git-diff.txt:26` describes the unnamed-commit form:

```text
# git/git, Documentation/git-diff.txt:26.
This form is to view the changes you made relative to
 the index (staging area for the next commit).
```

The same document at line 57 defines the explicit-commit form:

```text
# git/git, Documentation/git-diff.txt:57.
This form is to view the changes you have in your
 working tree relative to the named <commit>. You can
 use HEAD to compare it with the latest commit.
```

The deciding dispatch in `builtin/diff.c:578` sends a request without tree operands
through `builtin_diff_files`.
A request with one tree operand uses `builtin_diff_index` at line 600:

```c
/* git/git, builtin/diff.c:578 and 599, selected dispatch branches. */
if (!ent.nr) {
    /* The zero-blob branch: */
    result = builtin_diff_files(&rev, argc, argv);
}
else if (ent.nr == 1)
    result = builtin_diff_index(&rev, argc, argv);
```

The source archive retains the complete deciding files;
the excerpt omits unrelated branches.

### Default signal handling does not run the driver cleanup protocol

The driver creates each owner with `detached: true` and redirects its output to a log file.
Its cleanup is a JavaScript `finally` block,
with no `SIGINT` or `SIGTERM` listener in the original version.

Node source tag `v26.12.0`,
commit `26e72d85a611e222e3f06025de8a3c496923bacd`,
`doc/api/child_process.md:934` documents the process boundary:

```text
# nodejs/node, doc/api/child_process.md:934.
On non-Windows platforms, if options.detached is set to true, the child
process will be made the leader of a new process group and session.
```

The Linux libuv child setup in `deps/uv/src/unix/process.c:317` performs it:

```c
/* nodejs/node, deps/uv/src/unix/process.c:317. */
if (options->flags & UV_PROCESS_DETACHED)
    setsid();
```

Node registers its default signal handler in `src/node.cc:625`:

```cpp
// nodejs/node, src/node.cc:625.
RegisterSignalHandler(SIGINT, SignalExit, true);
RegisterSignalHandler(SIGTERM, SignalExit, true);
```

`SignalExit` at `src/node.cc:190` resets terminal state and raises the signal:

```cpp
// nodejs/node, src/node.cc:190.
void SignalExit(int signo, siginfo_t* info, void* ucontext) {
  ResetStdio();
  raise(signo);
}
```

`doc/api/process.md:669` documents default exit behavior and its replacement by an installed listener.
The old driver has no listener to initiate its JavaScript cleanup.
The local control confirms the detached owner survives that default exit.
This does not prove how a provider cancels its entire sandbox.

## Verification

The reviewer read the existing exact-tag Git and Node source clones without modifying them.
All driver fixtures ran under actual Node `26.12.0`,
with cgroup values `cpu.max=200000 100000`,
`memory.max=2147483648` and `pids.max=256`.
They had no network access or host credentials.
The fake executables identify every owning invocation as fixture execution.

The original controls retain the following outcomes:

- A complete graph invokes all 222 planned fake commands and produces matching log hashes.
- A shared-producer failure runs its shared and independent Rust commands,
  then blocks dependent module and browser branches.
- A failed consumer preserves the remaining command receipts.
- Near-deadline commands are explicitly unstarted;
  a resistant command reaches the actual timeout and kill path.
- Normal completion kills an intentionally lingering descendant,
  whose PID is independently observed absent.
- An unstaged tracked edit and an existing Cargo target are rejected before owning commands.
- A staged tracked edit incorrectly succeeds with 222 fake commands.
- `SIGTERM` terminates the driver while its detached owner remains alive.
  The fixture then explicitly kills and reaps that group inside the disposable container.

Corrected source `234ccc92dabb81e998e0a0bb5e838713924dd83f`
passes twelve controls.
It rejects both staged and unstaged changed working bytes before owning commands.
Both `SIGINT` and `SIGTERM` controls use a resistant leader and resistant descendant.
They retain a completed cancellation manifest,
mark the active owner cancelled and subsequent commands unstarted,
and independently verify both PIDs absent.
Installed driver signal listeners return to zero.

Every retained `graph.json` is parsed directly with `JSON.parse`.
No escape repair is applied.
The final-source control changes ordinary tracked files `LICENSE` and `README.md`;
actual Git output with newline separators produces two distinct manifest entries.
Both corrected workflow setup strings contain an actual newline between install commands.

The corrected fixture retains the exact commit and all tree objects with all 13863 index paths.
Omitted application files carry `skip-worktree` flags;
`README.md` and `LICENSE` remain ordinary tracked files.
This source-bound control exercises driver guards without pretending to compile or test absent application files.

The source archive retains fixture code,
complete driver and bootstrap bytes,
plans,
raw results and 1878 log byte-length/SHA256 joins across original and corrected controls.
The corrected decoded driver SHA256 is
`28eda050606b50a28d5590db1d4dd2af1f2951ed22d0ae8abd476c1882c82521`.

A separate anonymous container with fresh Mise,
Rustup,
Cargo,
home and cache state verifies the actual selected-tool repair:

```bash
# Fresh isolated repository configuration, Mise 2026.10.9, 2 CPU and 2 GiB container.
mise install --locked node pnpm rust cargo-binstall
mise install --locked cargo:cargo-nextest
```

Both commands pass.
The actual installed versions are Node `26.12.0`,
pnpm `12.14.0`,
Rust nightly `5d8fec066` and Nextest `0.9.148` at `cd1d6d546`.
This qualifies selected-tool installation only.
Provider actions,
real package builds,
browsers,
Cargo tests,
provider cancellation and billing release remain unexercised by this local preflight.

## Verified workaround

The coordinator changed the actual driver,
without application-source changes:

```javascript
// .depot/research/graph/mise.toml, decoded campaign task at 234ccc92.
execFileSync('git', ['diff', '--name-only', 'HEAD', '--'], { cwd: root, encoding: 'utf8' })
```

It also installs `SIGINT` and `SIGTERM` listeners that abort an `AbortController`,
signal owned process groups,
escalate after three seconds,
prevent new owning invocations and preserve final cancellation state.
The `finally` block removes installed handlers.
The source archive retains the complete corrected code and executable controls.

The tradeoff is a graceful cleanup interval before signal-resistant owners are killed.
An uncatchable `SIGKILL` cannot run this protocol.
Comparing working bytes to `HEAD` does not inventory untracked files;
the graph separately declares its generated helper and build-output boundary.

## Rejected readings and attempts

The reviewer initially claimed browser image preparation was absent.
The coordinator supplied the complete frozen plan entry:
browser command index 2 invokes `prepare:matched-image`.
That claim is retracted.
Complete enumeration confirms 42 shared,
83 module-A,
87 module-B,
8 browser and 2 Rust commands.

The initial fixture setup used unsupported `git hash-object --write`.
It stopped before driver execution.
Installed Git help supplies `-w`,
which the fixture uses thereafter.
This was a local harness invocation error.

Normal completion and internal timeout cleanup did not prove external-signal cleanup.
The original source passed those controls while failing the independent external-termination control.
A process-group ownership set becoming empty is also insufficient by itself;
the resistant-process controls check actual PID absence.

## Repository guideline proposal

A possible tightening of `AGENTS.md`'s existing `GFP` rule is:

```text
# AGENTS.md, proposed GFP replacement; not applied.
GFP:
 Guard tests cover each claimed state.
 Commit, disable the guard, rebuild, run, restore.
 Prove changed acceptance; restoration discards uncommitted file changes.
```

The concrete driver defects are fixed and independently exercised.
The guideline change remains a proposal.

## Upstream filing decision

`.out-of-scope/` was enumerated;
no matching Git or Node exemption is present.
Anonymous GitHub searches for Node detached/SIGTERM/parent behavior
and Git diff/staged/HEAD behavior are retained in the source archive.
No empty-search claim is used to infer an upstream defect.

1.  Upstream fault:
    no.
    The repository selected the wrong comparison and omitted its signal protocol.
2.  Upstream fixability:
    existing documented Git and Node interfaces already support the required behavior.
3.  Supported use case:
    the cited documentation covers commit comparisons,
    detached process groups and signal listeners.
4.  Contribution welcome:
    not assessed because no upstream change is proposed.
5.  Fix likelihood:
    no upstream fix is needed.
6.  Prototype:
    the consumer driver repair is implemented and independently verified;
    no upstream source modification is appropriate.

Nothing is filed or drafted upstream.

[receipt]: ../audit/depot-expanded-graph-preflight-2026-10-18.evidence.json
[sources]: ../audit/depot-expanded-graph-preflight-2026-10-18.sources.json.gz
[mise-prerequisite]: mise-cargo-backend-needs-rust-installed.md
