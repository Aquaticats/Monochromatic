# Resident commit service for the git wrapper: design and migration

Status: design for slice work, written by the commit-path performance owner. Evidence comes from a throwaway prototype
(scratch only, not in the repository) measured in the same harness and on the same 20 207-file clone as the earlier
remeasurement in `doc/planning/cli-git-commit-timeouts.md`.

## Owner requirements this design follows

- "Thoughts on locks: I believe they're an anti-pattern. I still think the server client architecture has potential."
- "Assume the chance different people/agents trying to commit at the same time is going to grow more and more."
- "child or resident proces" - another reason for server client architecture. "Some policies are meant to be ran async."
- "What is going on? Didn't I say locks are bad?" Read as: no flock, no lock files, no owner records, no polling and
  no lock-based fallback anywhere in the commit path.
- "socket activation (OS starts it) or a loud, clear failure - no, never leak internal things that could be resolved to
  the user." Read as: when a client cannot reach the service, either the operating system starts it and the commit
  proceeds, or the client stops with a plain failure that says what the user can do. Nothing in between.

The prototype used one `flock` to refuse a second daemon. That is dropped here (section "One service, no lock").

## What the prototype measured

Setup: fixed wrapper (policies on), a byte copy of it as control for the spread, the design reviewer's lean serial
prototype (plain Git, no policies) and the resident service prototype, alternating inside every round on the same clone,
each in its own `systemd-run` unit (CPUQuota 1200%, MemoryMax 16G, TasksMax 6000, IOWeight 50). Arrival is the instant a
client process is spawned, 25 ms apart (and 0 ms apart for the burst runs). Host load average was 25 to 50 during the
staggered run and about 18 to 39 during the burst run, so absolute times are worse than the earlier remeasurement
(load about 10) and the spread is large: at K=64 the wrapper took 131.8 s and its byte-identical control 40.5 s.

The prototype: a resident Rust process, clients that send one request over a unix socket and wait for the
acknowledgement, FIFO by accept order, one batch of whatever is queued at a time. A batch is landed by one
`git fast-import` (blobs, trees and one commit per request), one journal write with one `fsync` of the file and one of
the directory, one `update-ref` transaction (compare-and-swap of the branch and removal of the scratch ref), one `reset`
of the real index for the named files, then the acknowledgements. Push and maintenance run afterwards on another thread.
Preparation and policies run in the clients, before the request, so they are outside the serialized step; in this
harness the probe files are `.txt`, for which the wrapper's policies are in-process and cheap, so the prototype has no
policy stand-in and the policy cost shows up only as the wrapper's CPU (see "Where preparation and policies run").

Makespan of K simultaneous commits, milliseconds, staggered arrival:
- K=1: wrapper 647 to 1316, control 693 to 1034, lean 97 to 107, service 60 to 64.
- K=4: wrapper 1969 to 2413, control 1997 to 2559, lean 374 to 630, service 192 to 528.
- K=8: wrapper 5099 to 6517, control 3643 to 4967, lean 957 to 2112, service 271 to 444.
- K=16: wrapper 7556 and 9408, control 8469 and 8957, lean 1134 and 1135, service 498 and 545.
- K=32: wrapper 16824 and 18579, control 27836 and 29536, lean 2531 and 5810, service 879 and 902.
- K=64: wrapper 131802, control 40529, lean 4873, service 1771.

Per-commit latency (median / 95th percentile / maximum, ms), staggered arrival:
- K=16: wrapper 4460/8730/9301, lean 417/750/755, service 78/203/208.
- K=32: wrapper 9726/16820/18231, lean 1536/4778/5019, service 77/104/128.
- K=64: wrapper 103820/128373/130461, lean 1779/3006/3139, service 78/100/103.

The staggered run offers 40 commits per second, which the service absorbs without queueing (latency flat from K=16 to
K=64, batches of mean size 1.8). To find where it saturates, a burst run released all clients at once (arrival 0 ms
apart), service and lean only:
- K=16: service 102 and 105 ms (batches of mean size 8, largest 15), lean 1035 ms.
- K=32: service 111 and 117 ms (mean batch 16, largest 31), lean 2097 and 2632 ms.
- K=64: service 166 and 178 ms (mean batch 21, largest 39), lean 4201 and 4910 ms.
- K=128: service 305 and 540 ms (mean batch 32, largest 64), lean 8541 and 8702 ms.
No client failed in any arm.

Serialized section:
- Service: 49 to 68 ms per batch whatever the batch size (1 to 64), so 27 ms per commit when batches are 1.8 commits and
  2.6 to 6 ms per commit in the burst run. Throughput ceiling is the batch size over the section: about 400 commits per
  second at a batch of 21. One journal sync pair per batch (2 `fsync` calls) against about 28 syncs per wrapper commit.
- Lean: 60 to 70 ms per commit, a ceiling of about 15 commits per second, and it is exactly what the K=128 run shows.
- Wrapper: the landing lock is held 130 to 175 ms for a landing that wins; under contention the effective serialized
  section is the reservation (replay plus landing), median 360 to 560 ms, so a ceiling of about 2 commits per second.

Fairness (commits served after a commit that arrived later): wrapper 4 of 12 at K=4 up to 53 of 64 at K=64, the
largest overtaking 49; lean 0; service 0 in every staggered run. In the burst run the service shows 22 of 128 at K=64
and 1 of 256 at K=128. That is the harness: with zero stagger the spawn order of 64 processes is not the order in which
their connections reach the service, and the service's own order is strict accept order. The count therefore bounds
what a client could see, and it says nothing against the queue.

CPU per commit: wrapper 0.59 s at K=1, 0.86 s at K=32, 1.05 to 1.42 s at K=64 (replays); service 0.130 s at K=1
(its Git children included) falling to 0.005 to 0.017 s per commit in the burst run; lean about 0.12 s.

User-visible latency of the commit command with the after-commit push (local bare remote whose receive hook sleeps for
a chosen delay, three alternating runs each):
- Push in the command (wrapper, today): 527 to 1322 ms with no delay, 2520 to 3564 ms with a 2 s remote.
- Push moved out (service): the client is acknowledged in 35 to 91 ms (one 416 ms outlier) with no delay and 41 to 81 ms
  with the 2 s remote; the push itself finishes 96 to 457 ms and 2092 to 2179 ms after the request. The 30 s push
  deadline that bounds the wrapper's command is no longer in the commit's path.

Cold start: the process is ready 7 to 8 ms after it is spawned and the first commit through a fresh process takes 38 to
41 ms. Socket activation adds the manager's own start cost, not measured here.

## Failure cases a resident process introduces, tested on the prototype

Each case was run for real against the prototype with a deterministic fault point (the process kills itself with
`SIGKILL` at the named point) or an external `SIGKILL`. The client retried with its request id after the harness restarted
the service.
- Killed after the batch's objects were written, before the journal: nothing landed, nothing to recover, the client's
  retry landed the commit once; repository clean, `fsck` clean.
- Killed after the journal sync, before the branch moved: restart found an open intent whose branch was still at the old
  value and dropped it; the client's retry landed the commit once.
- Killed after the branch moved, before the real index was updated (the window in which the index shows the file as
  reversed): restart saw the branch at the new value, finished the index update (idempotent) and answered the retry "already";
  index and worktree clean, committed once.
- Killed after the index update, and killed after everything but the acknowledgement: restart answered the retry
  "already"; committed once.
- Client killed right after sending: the commit landed once, index clean; sending the same request id again was answered
  without a second commit.
- Stale socket file after `SIGKILL`: the restart detected it (nothing answered), replaced it and served the next commit.
- Second service, same state directory: refused (exit status 3, by the `flock` this design drops); second service on the
  same socket with another state directory: refused because the first still answered.
- No service at all: the client gave up after its bounded connect wait (500 ms in the test) with exit status 75.
- Service cold start: see above.
- Eight external `SIGKILL`s of the service while 16 clients were in flight (the kill came 6 ms to 45 ms after the burst
  started, plus three at later moments): in every run all 16 clients succeeded after the restart, all 16 commits were
  on the branch exactly once (no duplicate from the retry, no loss), index and worktree clean, `fsck` clean. At the
  earliest kills 0 or 1 of 16 had landed at death and the restart recovered one open batch; at later kills the batch had
  completed.
- A client inside a PID namespace (`unshare --user --pid`): the service sees the client's process id as the service's own
  namespace numbers it (4124420, while the client's own number inside is 3); the commit path works. A service inside a PID
  namespace with a client outside: the service sees process id 0 (unmapped). Any identity or liveness check by process id
  therefore breaks in the second arrangement, which is the failure behind issues 1781 and 1784; this design uses none.
- Linux abstract socket: a second bind of the same name is refused ("Address already in use") and the name is free at
  once when the first process ends (no stale name).

## Decision summary

1. One resident, single-writer service per user. The wrapper is a thin client for the commit-path commands that mutate
   a repository; every other command is forwarded to real Git as today.
2. The service is never started by the wrapper. The operating system starts it on demand through socket activation, or
   the client stops with a plain failure. No second writer, no lock, no fallback path.
3. Preparation and policies run in the client, outside the serialized step; the service lands prepared changes in FIFO
   order in batches with one journal sync per batch.
4. Exactly-once landing by request id, journal replay on restart.
5. After-commit work (push, maintenance, standards review) is recorded as durable work items and runs after the client
   is acknowledged.
6. The lock, owner record, recovery, shadow repository and replay machinery are deleted once the service is the only
   writer of the commit path (list and line counts below).

## One service, no lock

Mutual exclusion between two services is needed only if two can run. Both ways of starting one prevent it without a lock:
- Socket activation: the manager (systemd on Linux, launchd on macOS) owns the listening socket and runs at most one
  instance of the service unit. A second instance cannot be started by the wrapper because the wrapper never starts one.
- A development or test run without a manager binds a Linux abstract socket (a name starting with a NUL byte). The
  kernel allows one binder of a name, the name disappears with the process, and so there is no stale file, no lock file
  and no cleanup. Measured: the second bind is refused and the name is free the moment the first process ends.
  Portable fallback for tests on other platforms: bind a path inside a private directory; a second bind fails with
  "address in use", and a start that finds an existing path connects once: an answer means another service runs, no
  answer means a stale path that is removed and bound again. That check is a race only between two simultaneous manual
  starts, which the manager path does not have.

One service per user (per uid), not per repository: one activation unit and one socket for the user, with an independent
queue and serialized step per repository. Key of a queue: the repository's common git directory, because objects,
refs and the packed refs are shared by all linked worktrees; the per-worktree index and `HEAD` are updated inside the
queue's step. Repositories do not wait on each other.

## Protocol and request identity

Transport: a unix stream socket at `$XDG_RUNTIME_DIR/cli-git/service.sock` (mode 0700 directory). One connection per
request. Messages are length-prefixed frames carrying UTF-8 JSON, versioned by the first frame.

Client to service:
- `hello`: protocol version range and the client's build identifier.
- `commit`: request id, repository (canonical path of the worktree and of the common git directory), expected branch
  reference, expected base commit, the prepared change (see below), message source resolved to text, author and
  committer identity, options that change the commit object, and the client's policy findings to record.
- `status` (non-mutating): queue depth, the batch in progress, version.
- `retire`: finish the queue and exit, used for version replacement.

Service to client:
- `welcome`: the service's protocol and build, so the client can decide on replacement.
- `position`: sent when the request is queued behind others and whenever the position changes: "number N of M".
- `landed`: request id, commit object, the batch it landed in, plus "already" when the id had landed before.
- `retry_prepare`: the paths named by the request changed on the branch between the expected base and the current head;
  carries the new base. The client re-runs preparation at the new base and sends the request again with the same id,
  keeping its place (the service queues a re-sent id ahead of requests that arrived after its first arrival).
- `failed`: a code from a closed list that maps to a user-facing message (section "Failures the client can meet").

Request identity: the client makes a random request id (128 bits) per invocation and keeps it across reconnects and
re-preparation. The service records landed ids in its journal and keeps them for a bounded time (default one day, and
at least the last 10 000). A request whose id has landed is answered "already" with the original commit object. That is
what makes a client killed mid-request, a service killed mid-batch and a retry all land the commit exactly once, which
the prototype showed in every kill case.

The prepared change: a list of (path, mode, blob object) for the paths the commit names, with the blobs already written
into the repository's object store by the client (unreferenced loose objects are safe until referenced and until
`gc.pruneExpire`), plus deletions. The service never reads the user's worktree. This keeps the serialized step free of
work that scales with the repository, and a hung policy or a slow disk in a client cannot stall the writer.

## The serialized step, group commit, FIFO and backpressure

Per batch, in this order, all by the one writer of the queue:
1. Take every queued request up to a batch limit (64 in the prototype), in accept order, re-sent ids first.
2. Read the branch head. For each request check that none of its paths changed between its expected base and the head
   (one tree comparison per batch for all paths). A request that conflicts gets `retry_prepare`; the rest continue.
3. Build one commit per request, chained, with the objects written by one `fast-import` (or by in-process object
   writing once the crate can; the prototype shows 15 to 35 ms for this step).
4. Write the batch intent to the journal: one file, one `fsync` of it, one rename into place and one `fsync` of the
   directory. This is the only synchronous durable write of the batch.
5. Move the branch with one compare-and-swap (Git's own ref transaction: it still takes Git's own lock file for that ref,
   which is Git's, not ours, and also protects against a plain `git` process moving the branch).
6. Update the real index for the named paths (`reset` of those paths against the new commit), which takes Git's own
   `index.lock`; a holder from another program is waited for with a bounded retry, then the failure is reported.
7. Mark the intent done (a rename, not synced; a lost mark only makes recovery redo an idempotent step).
8. Acknowledge every client of the batch, then append the after-commit work items.

FIFO: the order is the order connections are accepted; a re-sent id keeps its first position. Group commit is the
natural result of taking everything queued: batches grow from 1 under light load to 21 to 64 in a burst, the section stays
49 to 68 ms, and so the cost per commit falls with the load instead of rising.

Backpressure: per-repository queue limit (default 256 requests) and a per-connection frame size limit. Past the limit the
service stops accepting from that repository's clients; clients wait in the socket backlog and receive `position`
notices once accepted. A client stops waiting after a bounded commit wait (default 10 minutes, configurable) and fails with
a plain message. No polling anywhere: clients block on the socket.

## Durability and crash recovery

The journal directory holds one file per batch. States: absent (nothing happened), intent (synced), done (renamed).
Recovery runs when the service starts, before it accepts connections:
- Intent whose branch still equals the old value: the batch never landed; drop the intent. Clients re-send by id.
- Intent whose branch equals the new value, or descends from it: the batch landed; redo the idempotent index update for
  its paths, mark done, remember its ids so a re-sent id is answered "already".
- Anything else (the branch moved to an unrelated commit): the batch did not land and cannot now; drop the intent with a
  logged reason; clients re-send and are prepared again against the new head.
- Temporary files are removed.
Tested above for kills after the objects were written, after the journal, after the branch moved, after the index update
and before the acknowledgement. What a client sees when the service dies mid-request: its connection closes without a
reply; it reconnects (activation restarts the service, which replays its journal first) and re-sends the same id; the
answer is either a fresh landing or "already". If the service stays down past the client's bounded wait, the client
says plainly that the commit may or may not have been made and what to check (`git log -1`, `git status`).

Objects are written without syncing, as the wrapper accepts today (`core.fsync` default, loose objects), because the
branch only moves after the journal sync and objects are content-addressed. A power loss after the ref update on a
filesystem that reorders data and metadata can leave a ref naming a missing object; this is the risk the owner already
accepted for the migrated pack ("Real gain real risk. We're high rollers.") and is unchanged.

## Where preparation and policies run

Evidence: the serialized step is 49 to 68 ms; the wrapper spends 0.59 s of CPU per commit in total; the design reviewer's
figure for a policy pass is about 300 ms. If policies ran inside the step, the ceiling would be about 1/(0.3+0.06), less
than 3 commits per second, and a hung policy would stall every queue; outside, they scale with the machine's cores.

Decision: policies and preparation run in the clients, in the invoking process, before the request. Reasons from the
evidence and the constraints: they read the user's worktree under the user's identity and environment; the message editor,
if one opens, must stay in the client's terminal; the OS schedules the parallelism with no pool to size; and a failure
or timeout of a policy is a failure of that one client. A worker pool inside the service is the alternative for policies
that become expensive enough to need a bounded fan-out (the existing child bounds and the per-class limits of
`child_class.rs` carry over to the client). Policies meant to be asynchronous (the standards review) are not part of the
commit at all: they are work items.

## After-commit work

The service writes one durable work item per landed commit and kind, in the record agreed with the standards-review port
(owner `ad02e84404af49107`): directory `<common git dir>/cli-git/work/{pending,claimed,done}/`, file name `<kind>-<oid>`
which is derivable so a retry cannot duplicate it, one JSON line `{"v":1,"kind","id","repository","gitDir","ref","oid",
"parent","created","tries"}`, state is the directory the file is in, a consumer claims by renaming into `claimed`, and a
consumer that died is detected by an expired lease and never by a process id. Kinds: `push`, `maintenance`, `review`.
Items are written whole and not synced: a lost item after a power loss is a skipped review, and push and maintenance are
recomputable from the refs. Consumers: push and maintenance run in the service on a separate thread, coalesced (several
pending pushes of one branch become one push) with the push deadline now outside the commit; the review consumer is the
standards-review port's, started by the same activation (a second unit) or by a user command, never by the wrapper.
What the client sees: the acknowledgement as soon as the branch and index are updated; push and review outcomes are
reported later, at the next wrapper command that prints notices (a one-line "the push of your last commit failed: ..."),
or by `status`. The prototype wrote the work items with no extra sync.

## Failures the client can meet

The rule: a failure the system can resolve is resolved without the user; the rest ends the command with a plain message
that follows the repository's diagnostic rule (the affected input and the calls named plainly, every valid remediation,
no implementation terms). Messages live in one module as named constants, each with the commands it quotes.

Resolved silently:
- Service not running while activation is installed: the connect succeeds because the manager owns the socket and starts
  the service; the client waits up to the start bound (10 s). If the wait passes one second, the client prints
  "Starting the commit service..." once (honest, bounded).
- Service killed or crashed during a request: the client reconnects, the manager restarts the service, recovery replays
  the journal, the same request id is re-sent; "already" or a fresh landing. Bounded to a handful of attempts.
- Client killed mid-request: the service finishes the request by id; the next invocation of the same commit is not
  affected (its id differs); nothing to do.
- Stale socket or stale record: with activation there is no stale socket (the manager owns it); stale journal intents are
  resolved by recovery; in an unmanaged development run a stale path is replaced as described above.
- Version skew: the `welcome` frame shows an older service than the client. The client sends `retire`; the service drains
  its queue and exits; the manager starts the new binary on the next connect; the client reconnects. The user sees at
  most "Updating the commit service..." and never a protocol message. If the service is newer than the client, the older
  client works within the protocol range both support; if no common version exists the client is the old one and says
  so in the failure below.
- Queue busy: the client prints the position ("Waiting for 3 earlier commits") and waits, no error.
- The branch moved under the request: `retry_prepare` runs inside the client without a message unless it repeats past a
  small bound, then the failure "The branch keeps changing" below.

Reaches the user (exact text is in the module; each is checked by tests against the denylist below):
- Activation not installed (the socket does not exist and nothing can start the service):
  "The commit service is not installed, so this commit was not made. Install it once with `cli-git service install`
  and run the commit again."
- Service cannot start (it exits during start or the start bound passes):
  "The commit service did not start, so this commit was not made. Its log shows why: `journalctl --user --unit
  cli-git.service`. Run the commit again after fixing that. Nothing in the repository changed."
- Platform without activation support (Windows at first; BSD systems; Linux without a user systemd, which means most
  minimal containers, WSL 1 and distributions with another init):
  "This computer cannot start the commit service on demand, so this commit was not made. [Platform remediation.]"
  with the platform's own remediation: Windows and BSD, the supported remediation named in the install guide; Linux
  without a user systemd, "turn on user services (`loginctl enable-linger`) or use a distribution with systemd".
  Never a second writer, never a silent bypass.
- Service owned by another user (peer credential mismatch):
  "Another account runs the commit service on this socket, so this commit was not made. Use your own account or
  ask that account to stop its service."
- Commit may or may not have landed (service down past the bounded wait after the request was sent):
  "The commit service stopped before it confirmed this commit. Run `git log -1` to see whether it was made, and
  `git status` to see what is still staged. If it was not made, run the commit again."
- Version has no common protocol with the installed service after a replacement attempt:
  "This git is older than the commit service and cannot talk to it. Update the git wrapper with the same tool that
  installed it (`cli-git service install` repeats the step) and run the commit again."
- Repository disk full or read-only: the repository's own message, with the path, the code mapped to plain words
  ("There is no space left in <repository>, so the commit was not made.").
- The branch keeps changing: "The branch <name> changed 5 times while this commit was prepared, so it was not made.
  Run the commit again; if other people commit to the same branch at the same moment, it will succeed once they pause."
- Waited longer than the commit wait: "This commit waited <N> minutes behind other commits and was not made. Run it
  again later, or raise the limit in the settings for the wrapper."
- The git index is in use by another program (Git's own `index.lock`): the existing message that names the holder, since
  that holder is not ours.

Tests (slice 1): every message is a named constant; a test renders each with sample inputs and asserts the text
contains none of the denylist (case-insensitive, after removing the quoted commands): protocol, journal, request id,
daemon, socket, errno, EPIPE, ECONNREFUSED, EAGAIN, flock, lock file, peer, frame, handshake, batch, oid, fast-import,
unix domain, abstract, "pid", "uid", "fd", "ipc", a raw os error such as "os error", "No such file". A second test
asserts each message names at least one action the user can take (a line containing a quoted command or a verb from a
closed list) and that none is empty. A third test runs the real client against a stub service for every failure above
and compares the printed text to the constant, so the printed text cannot drift from the constant.

## Security

- The socket lives in the user's runtime directory (mode 0700) under the user's uid; the service checks `SO_PEERCRED`
  on every connection and refuses a uid other than its own. No TCP.
- Requests are validated: the repository path must be owned by the uid, the paths of a change must stay inside the
  repository (the canonical path of each must start with the worktree root), frame size and field sizes are bounded, and
  a request names only a repository the client could read as that uid.
- Containers and PID namespaces: a client inside a container reaches the service by a bind mount of the runtime directory
  (`-v $XDG_RUNTIME_DIR/cli-git:/run/cli-git`). Identity never depends on process ids: not for the peer, not for
  liveness, not for the owner of anything. The earlier wrapper needed a namespace-aware owner identity twice (issues 1781
  and 1784) because locks recorded process ids; this design records none. Measured: a client in a PID namespace is served normally; a service in a PID namespace sees its clients as process id 0, so nothing may depend on it.
- The service runs with the user's rights only, with the manager's resource limits.

## Version skew and replacement

Every `welcome` and `hello` carries a protocol version range and a build identifier. A newer client asks an older
service to `retire`, which drains the queue and exits; the manager's socket stays bound, so connections made meanwhile
wait in the backlog and are served by the new binary. The unit's `ExecStart` points at a stable path that the installer
replaces atomically. The service also exits after a configurable idle time so a release is picked up without any
request. No protocol error ever reaches the user.

## Linked worktrees, resource limits, observability

Linked worktrees: one queue per common git directory; the request names its worktree; the real index and `HEAD` of that
worktree are updated in the step; two worktrees of the same repository share the queue and so never race on refs.

Resource limits: unit settings `MemoryMax`, `TasksMax`, `CPUQuota`, `IOWeight`; queue length, frame size and batch size
limits in the service; a per-request wait bound; children of the service (Git) bounded by the existing per-class child
bounds (60 s for a quick query, 300 s for a bulk step).

Observability: `status` request and a `cli-git service status` command; one structured line per batch (size, section
milliseconds, queue wait of the oldest request) to the journal of the manager; the client prints queue position and
start notices; every request has its id in the service log so a user report can be traced.

## What is deleted and what is kept

Line counts below are of the files as of this branch, without tests; tests are listed after.

Deleted once the service owns the commit path (about 12 000 lines of code and 9 000 of tests):
- Our own locks: `owner_lock.rs`, `owner_lock_record.rs`, `live_lock.rs`, `commit_landing_lock.rs`, `capture_order.rs`,
  `replay_reservation.rs`, `transaction_lease.rs`, and `holder_budget.rs` where it only budgets lock holders: 2 647 lines
  and 2 405 of tests. Gone with them: the 20 ms polling, the wait notices for locks and the lock environment variables.
- Multi-process recovery: the `recovery_*` files and `transaction_registry.rs`: 4 031 lines and 2 918 of tests. Replaced
  by the single journal replay above.
- The shadow repository: `shadow_*.rs`: 1 052 lines. The shadow existed so that a native `git commit` with hooks could run
  without touching the real repository; hooks are removed, the commit object is built from a prepared change, and the
  prototype landed without a shadow in 49 to 68 ms.
- Replay and reservation: `replay_*.rs` and `landing_*.rs`, `commit_landing_loop.rs`: about 6 285 lines and 3 893 of tests
  with the shadow tests. The merge-on-a-moved-head replay becomes `retry_prepare`, done by the client by preparing again.
  A path-level conflict (a named path changed on the branch) is detected by the service; a content merge of the same
  file edited by two people is no longer attempted, the second commit is prepared again on the new head and the user
  sees the normal outcome of committing a file that changed.

Kept and reused: the policy engine and its checks, the candidate and private-index machinery that reads content for
policies (in the client), `child_class.rs`, `child_watch.rs`, `commit_index.rs` and `index_flags.rs` (preparation), the
per-command forwarding of everything that is not a commit-path mutation, and the diagnostics rules.

To decide in a later slice with measurements: the `index_lock_*` files (1 767 lines) parse `/proc` to name the holder of
Git's own `index.lock`. The service still has to update the real index through Git's own `index.lock`, so a bounded retry
and a plain message stay; the holder-identifying evidence may shrink to the message that already names the program.

## Migration plan

Each slice ships and passes the container gate.
1. Service plus the commit path, end to end, for the explicit-path commit (the default wrapped commit): the service
   crate module (`service_*`), the client path in `commit_command.rs`, journal and recovery, group commit, FIFO, request
   id, activation unit files for systemd and a development entry that binds the abstract socket, the failure messages
   with their tests, the kill tests. The existing policies run in the client unchanged. The old landing code stays in the
   tree during this slice but is not reachable from the commit command (no runtime fallback). Index commits and `--amend`
   keep the old path until slice 2 (one release, then removed).
2. The other commit modes through the service (index commit, amend, `-a`), the after-commit work queue with push and
   maintenance moved out of the command, and `retry_prepare`.
3. Standards review as a work-item consumer (the port owner's module), notices of late outcomes.
4. Delete the lock, recovery, shadow, replay and registry code and their tests; keep a one-time reader that removes the
   old registry directory of a repository.
5. macOS: a launchd agent with a `Sockets` entry; same service, same tests where launchd is available.
6. Windows: designed as a named-pipe service started by the service control manager; until built, the loud failure above.

## Kill-test plan

Tests live beside the service as binary tests that start the service as a child of the test (the test is a harness, not
the wrapper) with a deterministic fault point set by an environment variable that only the test build honors:
- A kill at each fault point of the batch (after the objects, after the intent, after the branch moves, after the index,
  before the acknowledgement) and at every journal write, rename and `fsync` by injected system call failure and by
  `SIGKILL` (the existing fault harness with `strace --inject` carries over).
- Random `SIGKILL` at moments inside a burst of many clients, checking that every acknowledged commit is on the branch
  exactly once, the index is clean, `git fsck` is clean, and every client either succeeded or got the "may or may not"
  message.
- A client killed after sending, before reading; a client that never reads (slow client); a connection that sends half a
  frame; an oversized frame; a second service on the same socket; version skew with `retire`; a full queue; a request
  for a repository owned by another uid; a PID-namespaced client and service.
- Mutators outside the service (a plain `git` commit, an IDE running `git status`) racing the service: the compare-and-swap
  and Git's own `index.lock` must hold, and the service must report rather than overwrite.
Guard tests are shown failing as the repository rules require (for example, removing the journal sync, removing the
"already" answer, removing the recovery step, each must fail a named test).

## Where this differs from the coordinator's suggestions

- Agreed: thin client for mutating commit-path commands only; activation by the OS; no fallback writer; loud failure.
- Added: the service exits when told to `retire` and when idle, so replacement and release need no install-time restart
  command; the client may not run `systemctl`.
- Added: queue keyed by common git directory, not by worktree, because refs and objects are shared.
- Not decided here and listed for the owner: whether `cli-git service install` (a user-run command that writes the unit
  files) is acceptable under the rule that the wrapper writes no scripts; unit files are configuration, not shell text.
- The prototype's `flock` and its stale-socket probe are not carried over; the abstract socket and the manager replace them.

## Acceptance test from issue 1770 (scoped commit with an unrelated tracked LFS asset)

Issue 1770: a scoped explicit-path commit exits 128 in a 2 GiB, 2 CPU consumer container at an unrelated tracked LFS
asset, with "Error cleaning Git LFS object: open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/<n>: no such
file or directory"; the same commit succeeds through the host wrapper.

Checked on this branch, with the real `git-lfs` from the toolchain:
- `git-lfs` creates `lfs/` (with `objects` and `tmp`) on demand inside an existing git directory: `git lfs clean` against a
  freshly created bare repository with no `lfs` entry exited 0 and left `lfs/objects` and `lfs/tmp`. So Git not creating
  `lfs/tmp`, in `git init` or in `shadow_skeleton.rs`, is not the cause, and the in-process skeleton changed nothing for
  this issue (it creates the same entries as `git init --bare --template=`, compared by a test).
- Reproduction attempt: a repository with `*.bin` tracked through the LFS filter (64 MiB asset), the asset made stat-dirty
  (same content, new modification time, so the clean filter has to run for it during the shadow commit), and a scoped
  commit of another file, through the fixed wrapper and the installed wrapper, in a unit limited to 2 GiB of memory
  (no swap) and 200% CPU. Both exited 0, the commit landed, the status was clean. Not reproduced.
- Reading: an `open` that fails on `<shadow>/lfs/tmp/<n>` after `git-lfs` would have created `lfs/tmp` itself means the
  shadow directory (its parent) was gone while `git-lfs` ran. That fits the shadow being removed under a live
  transaction by recovery in another process, which is the same family as issues 1781 and 1784: a recovery that decides a
  transaction owner is dead by process identity, here possibly between a container and the host sharing one worktree
  (a service in one PID namespace sees clients of another as process id 0, measured above). It is a hypothesis; the
  container was not available. The new design removes the whole class (no shadow directory, no owner record, no recovery
  by process identity).

Acceptance test for the service: a repository with a tracked LFS asset that the commit does not name, an explicit-path
commit of another file, the asset stat-dirty, run with the real `git-lfs` on the client's PATH, inside a unit limited to
2 GiB of memory and 2 CPUs, repeated with the client in a PID namespace (`unshare --user --pid`) and the service outside
it. It must exit 0 with the commit landed, without the client printing any path under a temporary directory. The client
runs the clean filter during preparation (in its own process, under its own limits), never the service.

## Slice 1b status (what is wired, what is measured, what is left)

Wired: `CLI_GIT_COMMIT_SERVICE=1` (the user's runtime directory) or `abstract:<name>` (tests) sends the plain scoped commit
(`git commit -m <message> [-m <message>] [--quiet] -- <paths>`) through the service. The wrapper prepares it itself (a
private index seeded from the real index, the policies, no shadow repository, no capture lock), the service lands it, and
the result is the commit native Git would have made: same tree, same message after Git's clean-up, author and committer
from `git var`. The differential test compares the two commits field by field. Everything else (amend, `-a`, an editor,
`-F`, signing, dates, trailers, a merge in progress, an unborn or detached head, a policy that rewrites a file, nothing to
commit) is declined before anything changes and runs through the existing transaction, so the old locks, shadow, replay and
recovery still serve those command lines. With the opt-in on and no service reachable the client prints the plain texts of
`service_messages.rs`; there is no second writer.

Measured on the 20 207-file clone, release binaries, host load 70 to 90, six alternating scoped commits of one file:
- native path: 757 to 10 149 ms, median 1474 ms.
- service path: 419 to 932 ms, median 572 ms. The service's own step is 50 to 70 ms; the rest is the wrapper's preparation
  and the policies, not yet optimized.

Real systemd socket activation (transient units, `systemd-run --user --socket-property=ListenStream=...`): the first
commit started the service on demand and landed in 95 ms end to end with debug binaries, a warm commit took 73 ms, and after
`SIGKILL` of the service the next commit started it again through the manager and landed (1.1 s, the manager's restart
delay). The unit templates are `src/native/service_units/cli-git.socket` and `cli-git.service`; a test reads them against the
socket path the wrapper uses and the resource rules (IOWeight at most 50).

Tests added in this step: a commit from inside a PID namespace lands; a scoped commit beside a stat-dirty tracked LFS asset
(the shape of issue 1770, real `git-lfs`) lands with no temporary path in the output, through the service path; a service
killed at four arbitrary moments with sixteen clients in flight loses and duplicates nothing. A guard break (the service path
returning early) fails the differential test because no batch reaches the journal.

Left for later slices: the install command that fills `@SERVICE_BINARY@` and `@GIT@` in the unit files and enables the socket;
making the opt-in the default; the amend, `-a`, `-F`, editor and signing command lines; moving the after-commit push and
maintenance out of the command into work items; per-repository parallel landing; the 2 GiB, 2 CPU container variant of
issue 1770; deleting the old landing path with its locks, shadow, replay and recovery once no command line reaches it.

## Install command, default and the uninstalled machine (superseded by "Provisioning")

`git cli-git service install` fills the two unit templates (`@SERVICE_BINARY@` is the `cli-git-service` program next to the
wrapper, `@GIT@` is the real Git the wrapper resolves), writes only the files that differ, runs `daemon-reload` and
`enable --now cli-git.socket`. It never starts the service: the socket is the only unit started, and the operating system
starts the service on the first connection. A second run changes nothing and says so. `uninstall` disables and removes
both units and is repeatable; `status` exits 0 only when the socket is active. All three are answered by the wrapper
without Git. The unit files are configuration the service manager reads; the wrapper writes no script or shell text.

The commit path is chosen by `CLI_GIT_COMMIT_SERVICE` and the files on disk, without starting a process:
- unset: the service is used when the socket unit file exists in the account's `systemd/user` directory, or a socket
  already exists at the runtime path. Otherwise the commit takes the path it took before.
- `0`: never the service.
- `1`: always the service; without the activation installed the commit stops at once with the not-installed text that
  names `git cli-git service install`, and nothing in the repository changes.

Decision for the uninstalled machine: until the old landing path is deleted (the last step of the migration), an
uninstalled machine keeps it, because that path is the only writer such a machine has, and removing it before the service
covers every command line would stop commits. This is the present writer, not a fallback: an installed machine never
falls back to it (an installed machine whose service does not answer gets the plain stop of the failure catalogue). Once
the old path is deleted, the unset case on an uninstalled machine becomes the same plain stop as `1`, and the install
command becomes a precondition for committing. Rejected: installing on first use (the wrapper would write service-manager
configuration without being asked), and a lock-based fallback writer on any machine (the owner's decision: locks are an
anti-pattern).

Tests: `service_commit_tests.rs` (the choice from value, runtime directory and unit directory), `service_install_tests.rs`
(a stand-in service manager program records every call), `binary_service_default_tests.rs` (installed and unset lands
through the service, `0` stays native, uninstalled stays native, `1` uninstalled stops plainly and quickly).

## After-commit work as built (push and upkeep)

A commit the service landed no longer pushes or runs upkeep inside the command. After the post-commit gate passes (the gate
still runs in the wrapper, and a blocked gate still exits 2 and leaves the push undone), the wrapper writes two work items,
`push-<oid>` and `maintenance-<oid>`, with `work_item.rs` (whole files, not synced, the same landing always yields the same
name), and sends one `work` message naming the common Git directory. That message starts the service through the operating
system when it is not running, exactly like a commit does, and the wrapper returns when the service has read it. The
protocol version is now 2 so an older service is asked to retire first.

The service runs the work on its own thread (`service_work.rs`), one repository at a time. Pending pushes of one branch are
claimed together and become one push of the branch (`push_once` in `auto_push.rs`, the same decision as the wrapper's own
auto-push but with no lock, because the service is the only pusher of the repositories it serves); upkeep runs once per
repository. A claim that was held by a service that died is taken back by lease (thirty minutes, longer than the push
deadline) and the item is tried again up to three times. Items of other kinds, the standards review, are never claimed.

What the person sees: the commit returns without waiting for the network. A failed push settles its items as failed and
leaves one notice file under `<common git dir>/cli-git/notices/`; the next commit that goes through the service prints it
once and removes it. The notice says the commit exists only on this computer for now, repeats what Git said, and says to run
`git push` when ready. A handoff that could not be written or announced prints the plain `WorkNotHandedOver` text and does not
change the exit status; the records stay pending and the next wake for the repository runs them.

Environment limit, stated plainly: the push runs in the service's environment, which is the one the unit file gives it, not
the terminal's. A push that needs a variable only the terminal has (an ssh agent address in particular) can fail there. The
failure notice says so. The fix on the person's side is to make the variable part of the user manager's environment
(`systemctl --user import-environment`, or an `environment.d` entry); recording a terminal's environment in a work item was
rejected because the file would carry secrets.

Not yet moved: the push of the old landing path (it still pushes inline, for the command lines the service does not cover
and for an uninstalled machine), and the post-commit policy gate (it stays in the wrapper because it decides whether the
push may run).

Tests: `service_work_tests.rs` (coalescing, kinds, notices shown once, refused items, lease and tries),
`service_handoff_tests.rs` (records before the wake, a repeat adds nothing, notices), protocol round trip for the new
message, and two binary tests with a real service and a bare upstream (the push happens after the commit returned and both
records end in `done`; a vanished upstream fails the push, not the commit, and the next commit prints the notice once). The
handoff guard was shown failing by dropping the push record from the handoff.

## Command lines the service covers now, and what still goes through the transaction

Covered, each compared field by field with the commit native `git commit` makes (tree, whole message, author with date,
committer, parents) and with what is left staged, modified and untracked:
- `-m` (repeated), `-F <file>` in each spelling, `--cleanup=whitespace|default|verbatim`, `--allow-empty-message`.
- `--author=Name <email>`, `--date=<date>` (Git's own `var` reads the date), `--trailer=<t>` (Git's `interpret-trailers`).
- An index commit and `commit -a` (the shipped commit-only enforcement blocks both unless `--no-enforce-only` or
  `--no-only` is given, so they are rare here); named paths as before.

Declined to the transaction, before anything changes: `-F -` (standard input is Git's to read), a message from both `-m`
and `-F`, `--author` without an email (Git searches history for it), `-s`, `-S` and `commit.gpgSign`, `-e` and the editor,
`-c`, `-C`, `-t`, `--cleanup=strip|scissors`, `--amend`, `--allow-empty`, `--fixup` and `--squash`, `-i`, `-p`, `--dry-run`,
`--pathspec-from-file`, a conclusion of a merge, a cherry-pick or a revert, an unborn or detached head, and a repository whose
configuration sets `commit.template`, `commit.cleanup`, `i18n.commitEncoding`, `core.commentChar`, `commit.signoff`,
`commit.verbose` or any `trailer.*` key. The last group closes a gap that existed before: the service path used to ignore
those settings silently.

What stands between this and deleting the old landing path, shadow, replay and locks: the transaction still serves the
declined group above, and several of those need real Git to build the commit (the editor, signing, sign-off, amend). The
design question is how the service lands a commit that real Git built. The proposal, for the owner or coordinator to accept
before the deletion: the wrapper runs real `git commit` against a thin scratch Git directory (its own `HEAD` at the base, the
private index, the private object directory with the real object store as an alternate), and ships the resulting commit's
identity to the service, which lands it by comparing and swapping the branch when the base is still the branch value, and by
asking the client to prepare again when it is not (a signed commit cannot be rebuilt on another parent). That keeps Git as
the only author of the message, trailers, signature and amend semantics, takes no lock, and is a smaller piece of code than
the shadow, replay and recovery machinery it replaces. It does not remove the scratch Git directory; it removes the shadow's
landing, replay, registry and recovery.

### Decision for the remaining command lines (coordinator, owner veto open)

A hybrid. First, embed every option that is pure message text or parent arithmetic, proved field by field against native
Git (tree, whole message, author with date, committer, parents, left-behind status), on the owner's rule "Invoke real git as
rarely as possible." Second, only where Git must run a program (an editor, signing) run real `git commit` against a thin
scratch Git directory and land the result by compare-and-swap or `retry_prepare`; the wrapper itself never launches the
editor or the signer, Git does, as it does today. Keeping the old transaction for those lines is out, because it keeps
the locks the owner banned.

Sub-step 4b-1, built and compared with native: `-s` and `--signoff` (Git's trailer interpreter, with the cases of a
trailer block before it, the same sign-off already last, another person's sign-off last, a `---` divider in the body),
`--allow-empty`, `-C <commit>` (message and authorship reused), `--fixup=<commit>` (plain commit names only),
`--cleanup=strip` (default comment character only; any `core.commentChar` setting declines). `--squash` always opens an
editor natively, so it is not embedded.

Sub-step 4b-2, built and compared with native (`binary_service_amend_tests.rs`, field by field: tree, whole message, author
with date, committer, parents, status, staged difference and commit count): `--amend` with `-m`, `-F`, `-C`, `--fixup`,
`-s`, `--trailer`, `--author`, `--date` or `--cleanup`, or with `--no-edit` (the replaced commit's message, cleaned as Git
cleans it), scoped to named paths or over the whole index or `-a`; `--reset-author` on a plain amend (no `--author`,
`--date` or reused commit); a root commit and a merge commit (the parents are the replaced commit's parents, whatever
their number); `--allow-empty`. Parents come from the replaced commit, its author is kept unless the line replaces it, and
a replaced commit with a header an amend would have to carry over (`encoding`, a merge tag) stays native. An amend that
would leave the commit equal to its first parent (a merge excepted) is declined, because native Git refuses it.

The landing rule of an amend: the client sends `amend` and the `tree` it wrote into the object store; the service builds the
commit with the replaced commit's parents; the request lands alone (never in a batch with others, and what arrives after it
starts a new group) and only while the branch tip equals the base the client read. Otherwise the service answers
"branch moved" (`FailureCode::BranchMoved`), which the client prints as a plain failure; an amend is never rebuilt on another
tip, because it would replace a commit the person did not see. The protocol version is 3 so an older service retires first.

### Commits Git builds (step 4c)

Every command line the service cannot build from the command line itself is built by real `git commit` in a thin scratch
repository (`service_native.rs`), and the service lands the result unchanged. That covers the editor (`-e`, no message
option, `-c`, `-t`, `--squash`, `--fixup=amend:` and `reword:`), signing (`-S`, `--gpg-sign`, `commit.gpgSign`), and, as the
general fallback while the embedded route is the optimization, the lines whose meaning is Git's own text work: `-F -`,
`--author` without an email, `--cleanup=scissors`, and a repository configuration that sets `commit.template`,
`commit.cleanup`, an encoding, a comment character, `commit.signoff`, `commit.verbose` or `trailer.*`. Real Git is started
only after the embedded route declined, so the common lines still start none to build the commit.

The scratch repository is made inside the commit's own scratch directory (`<git dir>/cli-git-service-<id>/shadow`), away
from the registry directory recovery scans, so no other process can mistake it for a dead transaction's shadow. It reuses
`shadow_create.rs` (the config that includes the real one with every hook off, `HEAD` and the branch at the base, a
snapshot of the other refs so `-C v1.0` and `--fixup=<ref>` resolve, the links that let `git-lfs` find its directory) and
`run_native_commit` (the signal guard, the person's terminal for the editor). The one difference is that Git writes its new
objects straight into the real object store (`GIT_OBJECT_DIRECTORY`), because a branch cannot point at an object that is
only in the scratch store. The commit is an unreferenced loose object until the service moves the branch to it, like the
blobs the client already writes.

The request names the commit (`commit`, protocol version 4), with `amend` when it replaces the tip. The service checks that
the name is a commit whose parents are the base (or, for an amend, the base's parents), lands it alone and only while the
branch tip equals the base, and otherwise answers "branch moved". There is no second attempt for these commits: an editor
text or a signature is not rebuilt on another tip, which matches the earlier transaction (it never replayed an amend, and
a signed commit needed the person's signing again). A person who signs every commit and commits to a branch other people
commit to at the same moment will see that failure more often than someone who does not sign; automatic rebuilding of a
commit with no editor is the follow-up that would remove it.

A policy that rewrote a file still declines to the transaction, as do an unborn or detached head, a merge, cherry-pick or
revert conclusion, an unmerged index, `--dry-run`, `-i`, `-p`, `--include` and `--pathspec-from-file`; their landing is
what step 4d replaces, their preparation stays.

Still declined after 4b-2 and before 4c, each with its reason (now built by Git as above): `-e`/`--edit`, `-c`,
`-t`, no message option (an editor opens), `--fixup=amend:` and `--fixup=reword:` (editor), `--squash` (editor), `-S`,
`--gpg-sign` and `commit.gpgSign` (signer), `--reset-author` with another source of authorship, `--author` without an email (history search), `-F -`
(standard input), `-i`, `-p`, `-u`, `-v`, `--dry-run`, `--pathspec-from-file`, a message from two sources, an unborn or
detached head, a merge, cherry-pick or revert conclusion, a policy that rewrote a file, an unmerged index, a repository
configuration that sets a template, a clean-up mode, an encoding, a comment character, trailers or sign-off.

### Owner correction on provisioning (verbatim, adopted)

The owner, asked when to run `git cli-git service install` on this machine, answered: "What is going on? Bazel uses client
server architecture and Bazel doesn't do this." Their earlier rule: "socket activation (OS starts it) or a loud, clear
failure - no, never leak internal things that could be resolved to the user." Bazel's client/server page says: "If the
client cannot find a running server instance, it starts a new one."; the client installs what it needs itself; and on a
version mismatch "the server is stopped and a new one started". So the install step, the installed-or-not default and the
`CLI_GIT_COMMIT_SERVICE` switch are internal things the user must never see. Adopted: the wrapper provisions socket
activation itself the first time a commit needs the service.

## Provisioning

Supersedes the install command, the installed-or-not default and the `CLI_GIT_COMMIT_SERVICE` switch, which were removed.
The owner's correction (verbatim above) is the requirement: the first commit that needs the service makes socket activation
exist, as Bazel's client starts its server.

What the wrapper does, in order, only when the socket `$XDG_RUNTIME_DIR/cli-git/service.sock` does not already exist and the
session bus socket and the `cli-git-service` program beside the wrapper do:
1. Remove units of an earlier release from `~/.config/systemd/user` (they outrank runtime units in the unit search path),
   recognized by content, plus the enabling link they left. A file of the same name that something else wrote is never
   touched; the commit stops with a message naming the file.
2. Write `cli-git.socket` and `cli-git.service` into `$XDG_RUNTIME_DIR/systemd/user` through a temporary file and a rename,
   only when the content differs. Nothing remains in the home directory, and the runtime directory disappears at logout, so
   the next first use provisions again. Concurrent first commits agree through idempotence alone: identical content, atomic
   renames, no lock.
3. Reload the manager only when a unit file that already existed changed or went away (an upgrade moved the program). A new
   file needs no reload: measured on this machine, the probe unit below was found and started with no reload, while one
   reload took over twenty seconds on the loaded host and interrupted other clients of the manager for that time.
4. Start `cli-git.socket` through the manager. The manager, not the wrapper, starts the daemon on the first connection, so
   the daemon is in its own cgroup (`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`).

The calls go to the user session bus (`DBUS_SESSION_BUS_ADDRESS`, else `$XDG_RUNTIME_DIR/bus`) as `org.freedesktop.systemd1`
on `/org/freedesktop/systemd1`: `Hello` on the bus daemon, then `StartUnit(ss) -> o` (and `Reload()` when needed), the
signatures in `org.freedesktop.systemd1(5)`. The manager's private socket is not used: `systemd(1)` calls it private to
systemd.

Build versus buy for the D-Bus client. zbus would add roughly fifty crates and an executor to a binary that starts on every
git command; `dbus` links the C library; the only small pure-Rust candidate, rustbus, is not in the offline registry cache.
The wrapper needs two methods with string arguments, so it carries a hand-written client of a few hundred lines
(`systemd_dbus.rs`): the writer and the reader cover the types those calls and the bus's replies use (`s`, `o`, `g`, `u`),
the reader is total over arbitrary bytes (a unit control walks every prefix and single-byte change; the `dbus_reply` fuzz
target in `cli.fuzz` holds it to that), and the whole exchange is bounded by one deadline of twenty seconds so a stuck bus
never hangs a commit. The layout of a call is checked against a message worked out by hand from the specification, and the
whole client was verified once against the real user manager with a uniquely named runtime-only probe unit (written,
started, found listening, stopped, removed; the `#[ignore]` control `the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit`).

Failures follow the owner's rule: no session bus or no user manager is the plain message with the remedy for the platform; a
refused call says the service manager refused and gives its words, without its internal error name.

Idle exit and version skew follow Bazel. The service exits after three hours with nothing to do (`--idle-seconds`, default
`IDLE_EXIT`; Bazel's `--max_idle_secs` default is three hours); an announced push or upkeep keeps it alive. On a version
mismatch the client asks the older service to retire, which drains its queue and exits, and the next connection activates
the new program.

What a machine without a user manager does: until the earlier landing path is deleted (4d), it keeps that path, because it is
the only writer such a machine has. After 4d the same commit stops with the plain message. Before 4d runs, the paths that
would hit that message (the gate containers, CI release jobs, `mise` tasks that commit) must be listed and reported.

### Landing is replaced, preparation is kept (coordinator decision, owner veto open)

For the command lines the service still does not cover (editor, signing, amend and the rest of the declined list), the
transaction's preparation stays: policy rewrites installed into the worktree, unborn and detached heads, and merge,
cherry-pick and revert conclusions. Only its landing step becomes the service's compare-and-swap, which is the one place
the owner's ban applies ("Didn't I say locks are bad?": the ban is on locks and polling, not on preparation logic). Step 4d
then deletes the landing machinery (the owner lock and its records, replay and the reservation, landing recovery, the
shadow when it served landing), not the preparation.

### Measured reload of the user manager (record, no claimed connection)

The first probe against the real user manager called `Reload` and then `StartUnit`. Its test log was last written at
2026-11-01 09:59:48 EST, the run took about forty seconds (two attempts of up to twenty seconds each, both cut by the
deadline), and the connection errors `systemd-run` printed in the next minutes were "Transport endpoint is not connected".
The machine's boot time afterwards reads 2026-11-01 10:08:37 EST (`uptime --since`). These times are recorded so they can
be compared with the reboot's cause later; no connection between them is claimed.

### Paths that would hit the no-manager failure (measured before 4d, nothing deleted)

The failure applies only after 4d, to a commit that runs the wrapper with no socket and no session bus (the route reads
`XDG_RUNTIME_DIR`, the socket under it, the session bus socket, and `cli-git-service` beside the wrapper). Checked from the
repository's workflows, tasks and harnesses:

Unaffected, with the evidence:
- `mise.toml` line with `cli-git-native cli-git fix --all --policy mono/dependent-version-bump`: a management command, not
  a commit.
- `.github/workflows/catalog-specifiers.yml`: installs `cli-git-native` under that name only and runs `cli-git check`; the
  `git add` there is real Git.
- `.github/workflows/npm-release.yml`: exposes the executable "under its own name, never as git" (its own comment), so the
  changesets action commits with real Git.
- `.github/workflows/crate-release.yml` and `cargo-publish.yml`: no step installs the wrapper as `git`. The release commit is
  made by `package/crate-release/cli/src/release-steps.ts` (`git commit --message ... -- <paths>`) with the runner's real
  Git, and the bump reaches main through the GraphQL API.
- `mise` tasks: none runs a commit (the only matches are comments about the CI version commit and a formatting-commit parse
  gate that a person runs first).

Affected (each needs a service the harness starts, or it stops with the platform message):
- The gate (`bin/test-native-container.ts`, `native:test:host`, `native:test:binary:host`) and the mutation runs
  (`bin/mutate-native-container.ts`, `native-planted-controls*.ts`): the binary tests build their environment with
  `env_clear()` and no `XDG_RUNTIME_DIR` (`binary_support.rs`), so every wrapped commit takes the earlier path today.
  Forty-two test files run a wrapped commit, one hundred fourteen calls of `wrapped_commit`/`run_commit`. The place to fix
  is the one constructor of the fixture environment: start a `cli-git-service` per fixture at a short runtime directory (as
  `start_in` already does for the service tests) and set `XDG_RUNTIME_DIR`.
- `bin/e2e-native-container.ts` and the concurrent-commit scenarios in `e2e/`: the image installs the wrapper as `git` and
  replays workloads in `podman run` with no bus; the harness must start the service in the container before the scenarios.
- `bin/interop-native-container.ts` and `native-interop/`: they exercise old-format transaction states (registry,
  recovery); they retire with that code, except the one-time reader.
- `bin/native-perf.ts` and `native-perf-git.ts`: they commit through the wrapper on the host, where a bus exists, so they
  would provision the person's own units as a side effect; they should start their own service instead.
- A shell with `XDG_RUNTIME_DIR` unset: cron (the default `crond` PAM stack has no `pam_systemd`), containers and toolbox
  shells without the runtime directory mounted, and an agent sandbox that does not mount `/run/user/<uid>` (the earlier
  sessions' note that `systemd-run` fails inside the sandbox is the sign). `ssh` gets the runtime directory and the bus while
  the user manager runs (a login session, or linger).
- Nested wrappers (`rebase --exec`, hooks) inherit the caller's environment and need nothing more.

### Every lock-like mechanism and what 4d does with it (owner: "Use as few locks as possible.")

Sizes are lines of code and of tests under `src/native` at this commit. Dispositions are proposals for the coordinator's
go-ahead; nothing is deleted yet.

Delete, because the single-writer service makes it unnecessary (landing moves to the service's compare-and-swap):
- The owner-lock family: `owner_lock.rs`, `owner_lock_record.rs`, `live_lock.rs`, `commit_landing_lock.rs`,
  `transaction_lease.rs`, `transaction_owner.rs`, `ownerless_directory.rs`, `capture_order.rs` (the capture lock and
  sequence exist only so replay can order captures), `replay_reservation.rs`: 3 022 and 2 917 lines.
- `real_index_lock.rs` and `recovery_dead_owner_locks.rs`, `recovery_unrecorded_lock.rs`: the wrapper's own creation of
  Git's `index.lock` for the landing, and the proofs that release a dead owner's copy. The service updates the index with
  `git reset`, which takes Git's own lock.
- `index_writer_coordination.rs` (and `index_writer_forward.rs`'s use of it, `management.rs`'s `hold_landing_lock`): today
  every forwarded index writer (`add`, `reset`, `checkout`, `stash`, `rebase`...) takes the landing lock and waits for a
  foreign `index.lock` first. With no landing lock to protect, such a command competes only on Git's own `index.lock`,
  as it does without the wrapper.
- Recovery and registry: `recovery_*.rs`, `commit_recovery.rs`, `transaction_registry.rs`, `transaction_gate.rs`: 4 440 and
  3 532 lines with the files above; replaced by the service's journal replay. Keep a one-time reader that removes the old
  registry directory.
- Replay and landing: `replay_*.rs`, `landing_*.rs`, `commit_landing_loop.rs`, `post_landing.rs`.
- The push lock: `auto_push_single_flight.rs` (the lock and its join logic; `run_single_flight_push_repeating` is called
  only from `auto_push.rs`, the inline push after a commit), `holder_budget.rs` (a time budget that exists so a waiter's
  limit can be derived), `lock_holder_facts.rs`, the waits for it in `bounded_push.rs`, and the environment variable
  `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`. Pushes become service work items coalesced per branch (`push_once`). Manual `git push`
  never took this lock, so nothing changes for it.
- The index-lock holder evidence: `index_lock_wait.rs`, `index_lock_wait_host.rs`, `index_lock_holders.rs`,
  `index_lock_holder_parsers.rs`, `index_lock_evidence.rs`: 2 926 and 2 081 lines with the coordination above. They exist
  to decide how long to wait for a foreign `index.lock`; the service's own bounded retry replaces them. The environment
  variables `CLI_GIT_LOCK_WAIT_SECONDS` and `CLI_GIT_WAIT_NOTICE_SECONDS` go with them.

Replace with a lock-free mechanism:
- The transaction directory of the preparation that stays (unborn and detached heads, conclusions, policy rewrites): its
  owner record and registry become the commit's own scratch directory, removed with the commit, as `service_commit.rs`
  already does for the private indexes and `service_native.rs` for the scratch repository.
- The `last-pushed` record of `auto_push_single_flight.rs`: written whole by rename (`auto_push_record.rs`), read by the
  service's push; a lost update only repeats a push.
- `worktree_copy_lock.rs` (a per-repository settlement lock for linked-worktree ignored-state copies, with its
  `settlement.lock` directory, `CLI_GIT_WORKTREE_COPY_LEASE` variable and 20 ms polling): independent of the commit service.
  Each copy has its own destination, journal and stage, and `worktree_copy_stage_claim.rs` already claims a stage by exclusive
  creation; the replacement is a per-destination claim by exclusive create and rename, with recovery claiming a journal the
  same way. It is a slice of its own, with its own differential tests.

Keep, because nothing else works:
- Git's own `index.lock`, honoured by the service's bounded retry (five tries, a tenth of a second apart, then the plain
  `IndexInUse` message). That is Git's lock, held by another Git process; the bound is the smallest that rides out a
  concurrent `git add`.

Not locks (exclusive creation used as a one-time claim, never waited on), so they stay: `private_storage.rs` (`create_new`
for private records), `worktree_copy_stage.rs` (exclusive file creation in a private stage), `direct_fix_install.rs`
(a temporary file beside the file, then a rename), `worktree_completion.rs` (the same pattern), and `bounded_push.rs`, which
holds no file creation at all (the only lock it carries is the wait limit derivation above). No `flock` or `fcntl` lock
exists in non-test code.

### Secrets in the service's environment (open for the coordinator)

A systemd user service does not inherit the terminal's environment, and no secret is written into a unit file. If a handler
needs a key, for example `SHADOW_REVIEW_LLMGATEWAY_API_KEY`, the clean ways provisioning could use are: the user manager's
own environment block, set through `SetEnvironment` on the manager at provisioning time from the wrapper's environment (held
in the manager's memory, never on disk, and visible to every unit of the account until the manager restarts); or a
credential the manager passes with `LoadCredential`, which needs the key in a file the account owns. Neither is
implemented. The first is one more call on the same bus connection, with the cost that other units of the account can read
the value; that exposure is the decision.
