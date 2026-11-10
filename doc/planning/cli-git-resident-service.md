# Resident commit service for the git wrapper: design and migration

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
design for slice work,
written by the commit-path performance owner.
Evidence comes from a throwaway prototype
(scratch only,
not in the repository) measured in the same harness and on the same 20 207-file clone as the earlier
remeasurement in `doc/planning/cli-git-commit-timeouts.md`.

## Owner requirements this design follows

- "Thoughts on locks:
  I believe they're an anti-pattern.
  I still think the server client architecture has potential."
- "Assume the chance different people/agents trying to commit at the same time is going to grow more and more."
- "child or resident proces" - another reason for server client architecture.
  "Some policies are meant to be ran async."
- "What is going on?
  Didn't I say locks are bad?"
  Read as:
  no flock,
  no lock files,
  no owner records,
  no polling and
  no lock-based fallback anywhere in the commit path.
- "socket activation (OS starts it) or a loud,
  clear failure - no,
  never leak internal things that could be resolved to
  the user."
  Read as:
  when a client cannot reach the service,
  either the operating system starts it and the commit
  proceeds,
  or the client stops with a plain failure that says what the user can do.
  Nothing in between.

The prototype used one `flock` to refuse a second daemon.
That is dropped here (section "One service,
no lock").

## What the prototype measured

Setup:
fixed wrapper (policies on),
a byte copy of it as control for the spread,
the design reviewer's lean serial
prototype (plain Git,
no policies) and the resident service prototype,
alternating inside every round on the same clone,
each in its own `systemd-run` unit (CPUQuota 1200%,
MemoryMax 16G,
TasksMax 6000,
IOWeight 50).
Arrival is the instant a
client process is spawned,
25 ms apart (and 0 ms apart for the burst runs).
Host load average was 25 to 50 during the
staggered run and about 18 to 39 during the burst run,
so absolute times are worse than the earlier remeasurement
(load about 10) and the spread is large:
at K=64 the wrapper took 131.8 s and its byte-identical control 40.5 s.

The prototype:
a resident Rust process,
clients that send one request over a unix socket and wait for the
acknowledgement,
FIFO by accept order,
one batch of whatever is queued at a time.
A batch is landed by one
`git fast-import` (blobs,
trees and one commit per request),
one journal write with one `fsync` of the file and one of
the directory,
one `update-ref` transaction (compare-and-swap of the branch and removal of the scratch ref),
one `reset`
of the real index for the named files,
then the acknowledgements.
Push and maintenance run afterwards on another thread.
Preparation and policies run in the clients,
before the request,
so they are outside the serialized step;
in this
harness the probe files are `.txt`,
for which the wrapper's policies are in-process and cheap,
so the prototype has no
policy stand-in and the policy cost shows up only as the wrapper's CPU (see "Where preparation and policies run").

Makespan of K simultaneous commits,
milliseconds,
staggered arrival:
- K=1:
  wrapper 647 to 1316,
  control 693 to 1034,
  lean 97 to 107,
  service 60 to 64.
- K=4:
  wrapper 1969 to 2413,
  control 1997 to 2559,
  lean 374 to 630,
  service 192 to 528.
- K=8:
  wrapper 5099 to 6517,
  control 3643 to 4967,
  lean 957 to 2112,
  service 271 to 444.
- K=16:
  wrapper 7556 and 9408,
  control 8469 and 8957,
  lean 1134 and 1135,
  service 498 and 545.
- K=32:
  wrapper 16824 and 18579,
  control 27836 and 29536,
  lean 2531 and 5810,
  service 879 and 902.
- K=64:
  wrapper 131802,
  control 40529,
  lean 4873,
  service 1771.

Per-commit latency (median / 95th percentile / maximum,
ms),
staggered arrival:
- K=16:
  wrapper 4460/8730/9301,
  lean 417/750/755,
  service 78/203/208.
- K=32:
  wrapper 9726/16820/18231,
  lean 1536/4778/5019,
  service 77/104/128.
- K=64:
  wrapper 103820/128373/130461,
  lean 1779/3006/3139,
  service 78/100/103.

The staggered run offers 40 commits per second,
which the service absorbs without queueing (latency flat from K=16 to
K=64,
batches of mean size 1.8).
To find where it saturates,
a burst run released all clients at once (arrival 0 ms
apart),
service and lean only:
- K=16:
  service 102 and 105 ms (batches of mean size 8,
  largest 15),
  lean 1035 ms.
- K=32:
  service 111 and 117 ms (mean batch 16,
  largest 31),
  lean 2097 and 2632 ms.
- K=64:
  service 166 and 178 ms (mean batch 21,
  largest 39),
  lean 4201 and 4910 ms.
- K=128:
  service 305 and 540 ms (mean batch 32,
  largest 64),
  lean 8541 and 8702 ms.
No client failed in any arm.

Serialized section:
- Service:
  49 to 68 ms per batch whatever the batch size (1 to 64),
  so 27 ms per commit when batches are 1.8 commits and
  2.6 to 6 ms per commit in the burst run.
  Throughput ceiling is the batch size over the section:
  about 400 commits per
  second at a batch of 21.
  One journal sync pair per batch (2 `fsync` calls) against about 28 syncs per wrapper commit.
- Lean:
  60 to 70 ms per commit,
  a ceiling of about 15 commits per second,
  and it is exactly what the K=128 run shows.
- Wrapper:
  the landing lock is held 130 to 175 ms for a landing that wins;
  under contention the effective serialized
  section is the reservation (replay plus landing),
  median 360 to 560 ms,
  so a ceiling of about 2 commits per second.

Fairness (commits served after a commit that arrived later):
wrapper 4 of 12 at K=4 up to 53 of 64 at K=64,
the
largest overtaking 49;
lean 0;
service 0 in every staggered run.
In the burst run the service shows 22 of 128 at K=64
and 1 of 256 at K=128.
That is the harness:
with zero stagger the spawn order of 64 processes is not the order in which
their connections reach the service,
and the service's own order is strict accept order.
The count therefore bounds
what a client could see,
and it says nothing against the queue.

CPU per commit:
wrapper 0.59 s at K=1,
0.86 s at K=32,
1.05 to 1.42 s at K=64 (replays);
service 0.130 s at K=1
(its Git children included) falling to 0.005 to 0.017 s per commit in the burst run;
lean about 0.12 s.

User-visible latency of the commit command with the after-commit push (local bare remote whose receive hook sleeps for
a chosen delay,
three alternating runs each):
- Push in the command (wrapper,
  today):
  527 to 1322 ms with no delay,
  2520 to 3564 ms with a 2 s remote.
- Push moved out (service):
  the client is acknowledged in 35 to 91 ms (one 416 ms outlier) with no delay and 41 to 81 ms
  with the 2 s remote;
  the push itself finishes 96 to 457 ms and 2092 to 2179 ms after the request.
  The 30 s push
  deadline that bounds the wrapper's command is no longer in the commit's path.

Cold start:
the process is ready 7 to 8 ms after it is spawned and the first commit through a fresh process takes 38 to
41 ms. Socket activation adds the manager's own start cost,
not measured here.

## Failure cases a resident process introduces, tested on the prototype

Each case was run for real against the prototype with a deterministic fault point (the process kills itself with
`SIGKILL` at the named point) or an external `SIGKILL`.
The client retried with its request id after the harness restarted
the service.
- Killed after the batch's objects were written,
  before the journal:
  nothing landed,
  nothing to recover,
  the client's
  retry landed the commit once;
  repository clean,
  `fsck` clean.
- Killed after the journal sync,
  before the branch moved:
  restart found an open intent whose branch was still at the old
  value and dropped it;
  the client's retry landed the commit once.
- Killed after the branch moved,
  before the real index was updated (the window in which the index shows the file as
  reversed):
  restart saw the branch at the new value,
  finished the index update (idempotent) and answered the retry "already";
  index and worktree clean,
  committed once.
- Killed after the index update,
  and killed after everything but the acknowledgement:
  restart answered the retry
  "already";
  committed once.
- Client killed right after sending:
  the commit landed once,
  index clean;
  sending the same request id again was answered
  without a second commit.
- Stale socket file after `SIGKILL`:
  the restart detected it (nothing answered),
  replaced it and served the next commit.
- Second service,
  same state directory:
  refused (exit status 3,
  by the `flock` this design drops);
  second service on the
  same socket with another state directory:
  refused because the first still answered.
- No service at all:
  the client gave up after its bounded connect wait (500 ms in the test) with exit status 75.
- Service cold start:
  see the `Cold start` measurement under `What the prototype measured`.
- Eight external `SIGKILL`s of the service while 16 clients were in flight (the kill came 6 ms to 45 ms after the burst
  started,
  plus three at later moments):
  in every run all 16 clients succeeded after the restart,
  all 16 commits were
  on the branch exactly once (no duplicate from the retry,
  no loss),
  index and worktree clean,
  `fsck` clean.
  At the
  earliest kills 0 or 1 of 16 had landed at death and the restart recovered one open batch;
  at later kills the batch had
  completed.
- A client inside a PID namespace (`unshare --user --pid`):
  the service sees the client's process id as the service's own
  namespace numbers it (4124420,
  while the client's own number inside is 3);
  the commit path works.
  A service inside a PID
  namespace with a client outside:
  the service sees process id 0 (unmapped).
  Any identity or liveness check by process id
  therefore breaks in the second arrangement,
  which is the failure behind issues 1781 and 1784;
  this design uses none.
- Linux abstract socket:
  a second bind of the same name is refused ("Address already in use") and the name is free at
  once when the first process ends (no stale name).

## Decision summary

1.  One resident,
    single-writer service per worktree.
    The wrapper is a thin client for the commit-path commands that mutate
    a repository;
    every other command is forwarded to real Git as today.
2.  The service is never started by the wrapper.
    The operating system starts it on demand through socket activation,
    or
    the client stops with a plain failure.
    No second writer,
    no lock,
    no fallback path.
3.  Preparation and policies run in the client,
    outside the serialized step;
    the service lands prepared changes in FIFO
    order in batches with one journal sync per batch.
4.  Exactly-once landing by request id,
    journal replay on restart.
5.  After-commit work (push,
    maintenance,
    standards review) is recorded as durable work items and runs after the client
    is acknowledged.
6.  The lock,
    owner record,
    recovery,
    shadow repository and replay machinery are deleted once the service is the only
    writer of the commit path (the list and line counts under `What is deleted and what is kept`).

## One service, no lock

Mutual exclusion between two services is needed only if two can run.
Both ways of starting one prevent it without a lock:
- Socket activation:
  the manager (systemd on Linux,
  launchd on macOS) owns the listening socket and runs at most one
  instance of the service unit.
  A second instance cannot be started by the wrapper because the wrapper never starts one.
- A development or test run without a manager binds a Linux abstract socket (a name starting with a NUL byte).
  The
  kernel allows one binder of a name,
  the name disappears with the process,
  and so there is no stale file,
  no lock file
  and no cleanup.
  Measured:
  the second bind is refused and the name is free the moment the first process ends.
  Portable fallback for tests on other platforms:
  bind a path inside a private directory;
  a second bind fails with
  "address in use",
  and a start that finds an existing path connects once:
  an answer means another service runs,
  no
  answer means a stale path that is removed and bound again.
  That check is a race only between two simultaneous manual
  starts,
  which the manager path does not have.

### One service per worktree

Corrected 2026-11-01 by the owner's decision ("Per worktree (Recommended)").
An earlier draft of this document,
and releases
0.5.0 to 0.5.5,
ran one service per user (per uid) with one socket for every repository of the account.
That was the design
agent's choice and never the owner's,
and it broke the document's own rule that repositories do not wait on each other:
a
wait for a foreign `index.lock` in one repository delayed the landings of every other one.

The scope key is the worktree's own Git directory (`$GIT_DIR`:
the main worktree's `.git`,
or `.git/worktrees/<id>` of a
linked worktree).
One service instance and one socket per worktree,
the way Bazel runs one server per workspace (its output
base is a hash of the workspace root).
Why the worktree and not the repository,
from the Git documentation read for the
decision:
- Linked worktrees share the objects,
  `refs/`,
  `packed-refs`,
  the configuration and the hooks,
  but each has its own `HEAD`,
  index,
  `logs/HEAD`,
  `refs/worktree`,
  `refs/bisect` and `refs/rewritten`.
- Git refuses to check out one branch in two worktrees,
  so two instances land on different branches.
- `update-ref` verifies the old value and a transaction aborts when a reference cannot be locked,
  so Git keeps concurrent
  writers of shared references safe;
  the service retries a push that such a collision refused (`PUSH_LOCK_RETRIES`).

Identity:
`service_instance.rs` hashes the real path of the Git directory with 128-bit FNV-1a,
32 hexadecimal digits.
A hash
and not `systemd-escape --path`,
because an escaped path of an ordinary home directory exceeds the 255-byte unit-name limit
(each dash costs four bytes);
32 digits keep the socket path (`$XDG_RUNTIME_DIR/cli-git/<instance>.sock`,
about 60 bytes)
under the 108-byte `sun_path` limit.
A collision of two worktrees would only make one instance serve both.
The units are
templates,
`cli-git@.socket` (`ListenStream=%t/cli-git/%i.sock`) and `cli-git@.service` (`--instance %i --journal
%S/cli-git/service/%i`),
instanced by the hash;
the wrapper starts `cli-git@<instance>.socket` through the manager and the
manager starts the service on the first connection,
as before.
The service refuses any message that names another worktree
(`Failed` with `Rejected`),
idles out per instance,
and keeps its own journal.

State that was shared moved to the worktree's own Git directory,
so that instances never share a writer:
the work records
(`<git dir>/cli-git/work`;
for the main worktree this is the path they always had),
the notices of failed pushes
(`<git dir>/cli-git/notices`,
which also shows a notice in the worktree whose push failed and in no other),
and the
temporary landing reference (`refs/worktree/cli-git-service/<batch>`,
a per-worktree namespace).
The standards-review spool
(`cli-git/review-entries/<hash of the worktree root>`) stays under the directory it is given:
its names are unique per
worktree,
so no two writers meet.

Maintenance on the shared object store needs no wrapper lock:
Git takes a lock on the repository's object database for
every `git maintenance run` and a concurrent one skips (the `git-maintenance(1)` manual,
section "Troubleshooting",
and
measured with Git 2.56.0:
with `objects/maintenance.lock` present,
`git maintenance run --task=gc` and `--auto` exit 0).

Upgrade from the per-user service (0.5.5 and earlier):
the first commit of this release finds the earlier `cli-git.socket`
and `cli-git.service` in the runtime unit directory,
asks the service on the old socket to retire (`encode_retire` in the
version it announced),
has the manager stop the socket and then the service,
removes the two unit files and the socket file,
reloads,
and starts its own worktree's instance.
Work the old service had claimed stays in the work directory and is claimed
again by a drain of the instance once its lease is over (`CLAIM_LEASE`,
120 seconds;
a claim found while the lease runs may
belong to the service that is retiring,
which renews it while it works,
so it is left alone,
the drain says when the lease is
over,
and the service looks again then;
a claim taken back costs the item one try).
The batches it left open sit in the shared journal,
the
directory above each instance's own;
each instance finishes those of its worktree (found by the worktree's index) before it
accepts a connection.
Work records of linked worktrees that the old service wrote in the common directory are drained by the
main worktree's instance,
whose Git directory is the common directory,
and also by the instance of any linked worktree on
each of its wakes (`drain_common_records`,
found through the `commondir` file),
so a repository whose worktrees are all
linked (a bare one) does not strand them.
Such an instance takes back a claim there only after the same lease,
because the
main worktree's instance may be running it.

### Records of a removed worktree (finding 8 of the 2026-11-01 review)

A worktree's work records and unshown push-failure notices live in its own Git directory,
which `git worktree remove` and
`git worktree prune` delete.
Owner requirement:
a failure is loud and clear,
so neither a failure nobody had seen nor a push
the service had not run may vanish with the worktree.
- Before either command runs,
  the wrapper copies the notices and the pending or claimed push records of every linked
  worktree that has any into a snapshot under `<common>/cli-git/removal-snapshots/<instance>` (`worktree_removal_rescue.rs`).
  The wrapper hands the process to Git,
  so it cannot look afterwards;
  the snapshot is therefore taken for every linked
  worktree with records and judged later.
- A later wrapper command in any worktree settles each snapshot by whether the worktree's Git directory still exists.
  If it
  does (Git refused,
  a dry run,
  a prune that spared it),
  the snapshot is discarded.
  If not,
  its notices move to
  `<common>/cli-git/orphaned-notices` and each unfinished push becomes a record saying the push may not have finished,
  unless
  a notice of that commit already holds Git's words.
- Any worktree's next command that is not read by a program shows the orphaned notices once,
  so the branch that is not on
  the remote is named wherever the person is.
- Not covered:
  a worktree deleted by `git gc` pruning (which Git runs itself,
  with no wrapper command to hook) or by hand
  with its registration left to a later Git.
  Records written by the service in the instant between the snapshot and the
  removal are lost.
  Both are limits of a hook that cannot see inside Git's own removal.

## Protocol and request identity

Transport:
a unix stream socket at `$XDG_RUNTIME_DIR/cli-git/<instance>.sock` (mode 0700 directory),
one per worktree
(releases before one service per worktree used `$XDG_RUNTIME_DIR/cli-git/service.sock` for every repository).
One connection per
request.
Messages are length-prefixed frames carrying UTF-8 JSON,
versioned by the first frame.

Client to service:
- `hello`:
  protocol version range and the client's build identifier.
- `commit`:
  request id,
  repository (canonical path of the worktree and of the common git directory),
  expected branch
  reference,
  expected base commit,
  the prepared change (described under `The prepared change`),
  message source resolved to text,
  author and
  committer identity,
  options that change the commit object,
  and the client's policy findings to record.
- `status` (non-mutating):
  queue depth,
  the batch in progress,
  version.
- `retire`:
  finish the queue and exit,
  used for version replacement.

Service to client:
- `welcome`:
  the service's protocol and build,
  so the client can decide on replacement.
- `position`:
  sent when the request is queued behind others and whenever the position changes:
  "number N of M".
- `landed`:
  request id,
  commit object,
  the batch it landed in,
  plus "already" when the id had landed before.
- `retry_prepare`:
  the paths named by the request changed on the branch between the expected base and the current head;
  carries the new base.
  The client does not read the worktree again:
  it merges the change it captured at invocation onto
  the new head (three-way,
  with the preparation base as merge base;
  a path whose captured bytes already hold the landed
  edit keeps the captured bytes,
  and so does a path that every landed commit changed from an earlier capture of the same
  worktree:
  the request carries a `CLOCK_MONOTONIC` capture stamp,
  the service remembers the stamp of each commit it
  landed per worktree and sends them with `retry_prepare`,
  and a commit it does not remember is unordered),
  re-runs the
  policies on the merged result,
  and sends the request again with the same id,
  keeping its place (the service queues a re-sent id ahead of requests that arrived after its first arrival).
  An
  overlap that cannot be merged fails the command with exit 1,
  the message saved,
  and the paths and the commit that
  landed first named;
  nothing is landed.
- `failed`:
  a code from a closed list that maps to a user-facing message (section "Failures the client can meet").

Request identity:
the client makes a random request id (128 bits) per invocation and keeps it across reconnects and
re-preparation.
The service records landed ids in its journal and keeps them for a bounded time (default one day,
and
at least the last 10 000).
A request whose id has landed is answered "already" with the original commit object.
That is
what makes a client killed mid-request,
a service killed mid-batch and a retry all land the commit exactly once,
which
the prototype showed in every kill case.

The prepared change:
a list of (path,
mode,
blob object) for the paths the commit names,
with the blobs already written
into the repository's object store by the client (unreferenced loose objects are safe until referenced and until
`gc.pruneExpire`),
plus deletions.
The service never reads the user's worktree.
This keeps the serialized step free of
work that scales with the repository,
and a hung policy or a slow disk in a client cannot stall the writer.

## The serialized step, group commit, FIFO and backpressure

Per batch,
in this order,
all by the one writer of the queue:
1.  Take every queued request up to a batch limit (64 in the prototype),
    in accept order,
    re-sent ids first.
2.  Read the branch head.
    For each request check that none of its paths changed between its expected base and the head
    (one tree comparison per batch for all paths).
    A request that conflicts gets `retry_prepare`;
    the rest continue.
3.  Build one commit per request,
    chained,
    with the objects written by one `fast-import` (or by in-process object
    writing once the crate can;
    the prototype shows 15 to 35 ms for this step).
4.  Write the batch intent to the journal:
    one file,
    one `fsync` of it,
    one rename into place and one `fsync` of the
    directory.
    This is the only synchronous durable write of the batch.
5.  Move the branch with one compare-and-swap (Git's own ref transaction:
    it still takes Git's own lock file for that ref,
    which is Git's,
    not ours,
    and also protects against a plain `git` process moving the branch).
6.  Update the real index for the named paths (`reset` of those paths against the new commit),
    which takes Git's own
    `index.lock`;
    a holder from another program is waited for with a bounded retry,
    then the failure is reported.
7.  Mark the intent done (a rename,
    not synced;
    a lost mark only makes recovery redo an idempotent step).
8.  Acknowledge every client of the batch,
    then append the after-commit work items.

FIFO:
the order is the order connections are accepted;
a re-sent id keeps its first position.
Group commit is the
natural result of taking everything queued:
batches grow from 1 under light load to 21 to 64 in a burst,
the section stays
49 to 68 ms,
and so the cost per commit falls with the load instead of rising.

Backpressure:
per-repository queue limit (default 256 requests) and a per-connection frame size limit.
Past the limit the
service stops accepting from that repository's clients;
clients wait in the socket backlog and receive `position`
notices once accepted.
A client stops waiting after a bounded commit wait (default 10 minutes,
configurable) and fails with
a plain message.
No polling anywhere:
clients block on the socket.

## Durability and crash recovery

The journal directory holds one file per batch.
States:
absent (nothing happened),
intent (synced),
done (renamed).
A batch's files are named by its number,
and the number is taken from the journal (2026-11-03):
the service starts counting at
a clock reading in microseconds,
then creates `<number>.claim` exclusively (`O_EXCL`) and skips a number that has a claim,
an
intent or a finished batch.
Production gives each instance its own journal (`cli-git@.service`,
`--journal %S/cli-git/service/%i`),
so one service counts there alone;
a development run and the binary controls give several services one journal,
where two
clock readings can be equal and a count that starts a few microseconds behind another's reaches the other's numbers after a few
batches.
Before the claim,
two landings with one number wrote one intent over the other and one finished record over the other
(`two_services_of_one_journal_never_number_their_batches_alike`),
so a request that had landed was forgotten,
and the shared
`marks-<number>` file could hand one landing the other's commits.
The claim file is removed when the landing ends,
and one a
killed service left is removed with the abandoned temporary files,
once older than their grace.
Recovery runs when the service starts,
before it accepts connections:
- Intent whose branch still equals the old value:
  the batch never landed;
  drop the intent.
  Clients re-send by id.
- Intent whose branch equals the new value,
  or descends from it:
  the batch landed;
  redo the idempotent index update for
  its paths,
  mark done,
  remember its ids so a re-sent id is answered "already".
- Anything else (the branch moved to an unrelated commit):
  the batch did not land and cannot now;
  drop the intent with a
  logged reason;
  clients re-send and are prepared again against the new head.
- Temporary files are removed.
Tested above for kills after the objects were written,
  after the journal,
  after the branch moved,
  after the index update
and before the acknowledgement.
  What a client sees when the service dies mid-request:
  its connection closes without a
reply;
  it reconnects (activation restarts the service,
  which replays its journal first) and re-sends the same id;
  the
answer is either a fresh landing or "already".
  If the service stays down past the client's bounded wait,
  the client
says plainly that the commit may or may not have been made and what to check (`git log -1`,
  `git status`).

Objects are written without syncing,
as the wrapper accepts today (`core.fsync` default,
loose objects),
because the
branch only moves after the journal sync and objects are content-addressed.
A power loss after the ref update on a
filesystem that reorders data and metadata can leave a ref naming a missing object;
this is the risk the owner already
accepted for the migrated pack ("Real gain real risk.
We're high rollers.")
and is unchanged.

## Where preparation and policies run

Evidence:
the serialized step is 49 to 68 ms;
the wrapper spends 0.59 s of CPU per commit in total;
the design reviewer's
figure for a policy pass is about 300 ms. If policies ran inside the step,
the ceiling would be about 1/(0.3+0.06),
less
than 3 commits per second,
and a hung policy would stall every queue;
outside,
they scale with the machine's cores.

Decision:
policies and preparation run in the clients,
in the invoking process,
before the request.
Reasons from the
evidence and the constraints:
they read the user's worktree under the user's identity and environment;
the message editor,
if one opens,
must stay in the client's terminal;
the OS schedules the parallelism with no pool to size;
and a failure
or timeout of a policy is a failure of that one client.
A worker pool inside the service is the alternative for policies
that become expensive enough to need a bounded fan-out (the existing child bounds and the per-class limits of
`child_class.rs` carry over to the client).
Policies meant to be asynchronous (the standards review) are not part of the
commit at all:
they are work items.

## After-commit work

The service writes one durable work item per landed commit and kind,
in the record agreed with the standards-review port
(owner `ad02e84404af49107`):
directory `<common git dir>/cli-git/work/{pending,claimed,done}/`,
file name `<kind>-<oid>`
which is derivable so a retry cannot duplicate it,
one JSON line `{"v":1,"kind","id","repository","gitDir","ref","oid",
"parent","created","tries"}`,
state is the directory the file is in,
a consumer claims by renaming into `claimed`,
and a
consumer that died is detected by an expired lease and never by a process id.
Kinds:
`push`,
`maintenance`,
`review`.
Items are written whole and not synced:
a lost item after a power loss is a skipped review,
and push and maintenance are
recomputable from the refs.
Consumers:
push and maintenance run in the service on a separate thread,
coalesced (several
pending pushes of one branch become one push) with the push deadline now outside the commit;
the review consumer is the
standards-review port's,
started by the same activation (a second unit) or by a user command,
never by the wrapper.
What the client sees:
the acknowledgement as soon as the branch and index are updated;
push and review outcomes are
reported later,
at the next wrapper command that prints notices (a one-line "the push of your last commit failed:
..."),
or by `status`.
The prototype wrote the work items with no extra sync.

## Failures the client can meet

The rule:
a failure the system can resolve is resolved without the user;
the rest ends the command with a plain message
that follows the repository's diagnostic rule (the affected input and the calls named plainly,
every valid remediation,
no implementation terms).
Messages live in one module as named constants,
each with the commands it quotes.

Resolved silently:
- Service not running while activation is installed:
  the connect succeeds because the manager owns the socket and starts
  the service;
  the client waits up to the start bound (10 s).
  If the wait passes one second,
  the client prints
  "Starting the commit service..." once (honest,
  bounded).
- Service killed or crashed during a request:
  the client reconnects,
  the manager restarts the service,
  recovery replays
  the journal,
  the same request id is re-sent;
  "already" or a fresh landing.
  Bounded to a handful of attempts.
- Client killed mid-request:
  the service finishes the request by id;
  the next invocation of the same commit is not
  affected (its id differs);
  nothing to do.
- Stale socket or stale record:
  with activation there is no stale socket (the manager owns it);
  stale journal intents are
  resolved by recovery;
  in an unmanaged development run a stale path is replaced as the `One service, no lock` fallback describes.
- Version skew:
  the `welcome` frame shows an older service than the client.
  The client sends `retire`;
  the service drains
  its queue and exits;
  the manager starts the new binary on the next connect;
  the client reconnects.
  The user sees at
  most "Updating the commit service..." and never a protocol message.
  If the service is newer than the client,
  the older
  client works within the protocol range both support;
  if no common version exists the client is the old one and says
  so in the failure below.
- Queue busy:
  the client prints the position ("Waiting for 3 earlier commits") and waits,
  no error.
- The branch moved under the request:
  `retry_prepare` runs inside the client without a message unless it repeats past a
  small bound,
  then the failure "The branch keeps changing" below.

Reaches the user (exact text is in the module;
each is checked by tests against the denylist below):
- Activation not installed (the socket does not exist and nothing can start the service):
  "The commit service is not installed,
  so this commit was not made.
  Install it once with `cli-git service install`
  and run the commit again."
- Service cannot start (it exits during start or the start bound passes):
  "The commit service did not start,
  so this commit was not made.
  Its log shows why:
  `journalctl --user --unit
  cli-git.service`.
  Run the commit again after fixing that.
  Nothing in the repository changed."
- Platform without activation support (Windows at first;
  BSD systems;
  Linux without a user systemd,
  which means most
  minimal containers,
  WSL 1 and distributions with another init):
  "This computer cannot start the commit service on demand,
  so this commit was not made.
  [Platform remediation.]"
  with the platform's own remediation:
  Windows and BSD,
  the supported remediation named in the install guide;
  Linux
  without a user systemd,
  "turn on user services (`loginctl enable-linger`) or use a distribution with systemd".
  Never a second writer,
  never a silent bypass.
- Service owned by another user (peer credential mismatch):
  "Another account runs the commit service on this socket,
  so this commit was not made.
  Use your own account or
  ask that account to stop its service."
- Commit may or may not have landed (service down past the bounded wait after the request was sent):
  "The commit service stopped before it confirmed this commit.
  Run `git log -1` to see whether it was made,
  and
  `git status` to see what is still staged.
  If it was not made,
  run the commit again."
- Version has no common protocol with the installed service after a replacement attempt:
  "This git is older than the commit service and cannot talk to it.
  Update the git wrapper with the same tool that
  installed it (`cli-git service install` repeats the step) and run the commit again."
- Repository disk full or read-only:
  the repository's own message,
  with the path,
  the code mapped to plain words
  ("There is no space left in <repository>,
  so the commit was not made.").
- The branch keeps changing:
  "The branch <name> changed 5 times while this commit was prepared,
  so it was not made.
  Run the commit again;
  if other people commit to the same branch at the same moment,
  it will succeed once they pause."
- Waited longer than the commit wait:
  "This commit waited <N> minutes behind other commits and was not made.
  Run it
  again later,
  or raise the limit in the settings for the wrapper."
- The git index is in use by another program (Git's own `index.lock`):
  the existing message that names the holder,
  since
  that holder is not ours.

Tests (slice 1):
every message is a named constant;
a test renders each with sample inputs and asserts the text
contains none of the denylist (case-insensitive,
after removing the quoted commands):
protocol,
journal,
request id,
daemon,
socket,
errno,
EPIPE,
ECONNREFUSED,
EAGAIN,
flock,
lock file,
peer,
frame,
handshake,
batch,
oid,
fast-import,
unix domain,
abstract,
"pid",
"uid",
"fd",
"ipc",
a raw os error such as "os error",
"No such file".
A second test
asserts each message names at least one action the user can take (a line containing a quoted command or a verb from a
closed list) and that none is empty.
A third test runs the real client against a stub service for every failure above
and compares the printed text to the constant,
so the printed text cannot drift from the constant.

## Security

- The socket lives in the user's runtime directory (mode 0700) under the user's uid;
  the service checks `SO_PEERCRED`
  on every connection and refuses a uid other than its own.
  No TCP.
- Requests are validated:
  the repository path must be owned by the uid,
  the paths of a change must stay inside the
  repository (the canonical path of each must start with the worktree root),
  frame size and field sizes are bounded,
  and
  a request names only a repository the client could read as that uid.
- Containers and PID namespaces:
  a client inside a container reaches the service by a bind mount of the runtime directory
  (`-v $XDG_RUNTIME_DIR/cli-git:/run/cli-git`).
  Identity never depends on process ids:
  not for the peer,
  not for
  liveness,
  not for the owner of anything.
  The earlier wrapper needed a namespace-aware owner identity twice (issues 1781
  and 1784) because locks recorded process ids;
  this design records none.
  Measured:
  a client in a PID namespace is served normally;
  a service in a PID namespace sees its clients as process id 0,
  so nothing may depend on it.
- The service runs with the user's rights only,
  with the manager's resource limits.

## Version skew and replacement

Every `welcome` carries the protocol version and the release (see "Upgrades heal on the first commit").
A client asks a
service of another release,
or an older protocol,
to `retire`,
which drains the queue and exits;
the manager's socket stays bound,
so connections made meanwhile
wait in the backlog and are served by the new binary.
The unit's `ExecStart` points at a stable path that the installer
replaces atomically.
The service also exits after a configurable idle time so a release is picked up without any
request.
No protocol error ever reaches the user.

## Linked worktrees, resource limits, observability

Linked worktrees:
one queue per common git directory;
the request names its worktree;
the real index and `HEAD` of that
worktree are updated in the step;
two worktrees of the same repository share the queue and so never race on refs.

Resource limits:
unit settings `MemoryMax`,
`TasksMax`,
`CPUQuota`,
`IOWeight`;
queue length,
frame size and batch size
limits in the service;
a per-request wait bound;
children of the service (Git) bounded by the existing per-class child
bounds (60 s for a quick query,
300 s for a bulk step).
Note of 2026-11-03:
the unit settings were set without a measurement and are removed;
the service unit sets no resource limits of its own
(owner decision,
recorded in `package/git-policy/cli/SPEC.md` beside the socket-activation paragraph).

Observability:
`status` request and a `cli-git service status` command;
one structured line per batch (size,
section
milliseconds,
queue wait of the oldest request) to the journal of the manager;
the client prints queue position and
start notices;
every request has its id in the service log so a user report can be traced.

## What is deleted and what is kept

Line counts in the `What is deleted and what is kept` section are of the files as of this branch,
without tests;
tests are listed after.

Deleted once the service owns the commit path (about 12 000 lines of code and 9 000 of tests):
- Our own locks:
  `owner_lock.rs`,
  `owner_lock_record.rs`,
  `live_lock.rs`,
  `commit_landing_lock.rs`,
  `capture_order.rs`,
  `replay_reservation.rs`,
  `transaction_lease.rs`,
  and `holder_budget.rs` where it only budgets lock holders:
  2 647 lines
  and 2 405 of tests.
  Gone with them:
  the 20 ms polling,
  the wait notices for locks and the lock environment variables.
- Multi-process recovery:
  the `recovery_*` files and `transaction_registry.rs`:
  4 031 lines and 2 918 of tests.
  Replaced
  by the single journal replay above.
- The shadow repository:
  `shadow_*.rs`:
  1 052 lines.
  The shadow existed so that a native `git commit` with hooks could run
  without touching the real repository;
  hooks are removed,
  the commit object is built from a prepared change,
  and the
  prototype landed without a shadow in 49 to 68 ms.
- Replay and reservation:
  `replay_*.rs` and `landing_*.rs`,
  `commit_landing_loop.rs`:
  about 6 285 lines and 3 893 of tests
  with the shadow tests.
  The merge-on-a-moved-head replay becomes `retry_prepare`,
  done by the client by merging
  the change it captured onto the new head (`service_replay.rs`),
  never by preparing again from the worktree,
  which
  would land bytes written after the command ran.
  A path-level conflict (a named path changed on the branch) is
  detected by the service and sent back;
  the client's merge lands the edits of different regions of one file,
  and an
  overlap fails plainly,
  as native Git would report two edits of the same lines.
  The deletion of the replay files
  keeps what the retry calls:
  `replay_merge.rs` (the merge),
  `replay_shared_paths.rs`,
  `replay_subsumption.rs`,
  `replay_subsumption_text.rs`,
  `replay_containment.rs`,
  `replay_reverse_apply.rs` and `replay_patch.rs` (the check that
  the captured bytes already hold a landed edit),
  with `replay_shadow.rs` and `shadow_git.rs` for the Git runner they use.

Kept and reused:
the policy engine and its checks,
the candidate and private-index machinery that reads content for
policies (in the client),
`child_class.rs`,
`child_watch.rs`,
`commit_index.rs` and `index_flags.rs` (preparation),
the
per-command forwarding of everything that is not a commit-path mutation,
and the diagnostics rules.

To decide in a later slice with measurements:
the `index_lock_*` files (1 767 lines) parse `/proc` to name the holder of
Git's own `index.lock`.
The service still has to update the real index through Git's own `index.lock`,
so a bounded retry
and a plain message stay;
the holder-identifying evidence may shrink to the message that already names the program.

## Migration plan

Each slice ships and passes the container gate.
1.  Service plus the commit path,
    end to end,
    for the explicit-path commit (the default wrapped commit):
    the service
    crate module (`service_*`),
    the client path in `commit_command.rs`,
    journal and recovery,
    group commit,
    FIFO,
    request
    id,
    activation unit files for systemd and a development entry that binds the abstract socket,
    the failure messages
    with their tests,
    the kill tests.
    The existing policies run in the client unchanged.
    The old landing code stays in the
    tree during this slice but is not reachable from the commit command (no runtime fallback).
    Index commits and `--amend`
    keep the old path until slice 2 (one release,
    then removed).
2.  The other commit modes through the service (index commit,
    amend,
    `-a`),
    the after-commit work queue with push and
    maintenance moved out of the command,
    and `retry_prepare`.
3.  Standards review as a work-item consumer (the port owner's module),
    notices of late outcomes.
4.  Delete the lock,
    recovery,
    shadow,
    replay and registry code and their tests;
    keep a one-time reader that removes the
    old registry directory of a repository.
5.  macOS:
    a launchd agent with a `Sockets` entry;
    same service,
    same tests where launchd is available.
6.  Windows:
    designed as a named-pipe service started by the service control manager;
    until built,
    the loud failure above.

## Kill-test plan

Tests live beside the service as binary tests that start the service as a child of the test (the test is a harness,
not
the wrapper) with a deterministic fault point set by an environment variable that only the test build honors:
- A kill at each fault point of the batch (after the objects,
  after the intent,
  after the branch moves,
  after the index,
  before the acknowledgement) and at every journal write,
  rename and `fsync` by injected system call failure and by
  `SIGKILL` (the existing fault harness with `strace --inject` carries over).
- Random `SIGKILL` at moments inside a burst of many clients,
  checking that every acknowledged commit is on the branch
  exactly once,
  the index is clean,
  `git fsck` is clean,
  and every client either succeeded or got the "may or may not"
  message.
- A client killed after sending,
  before reading;
  a client that never reads (slow client);
  a connection that sends half a
  frame;
  an oversized frame;
  a second service on the same socket;
  version skew with `retire`;
  a full queue;
  a request
  for a repository owned by another uid;
  a PID-namespaced client and service.
- Mutators outside the service (a plain `git` commit,
  an IDE running `git status`) racing the service:
  the compare-and-swap
  and Git's own `index.lock` must hold,
  and the service must report rather than overwrite.
Guard tests are shown failing as the repository rules require (for example,
  removing the journal sync,
  removing the
"already" answer,
  removing the recovery step,
  each must fail a named test).

## Where this differs from the coordinator's suggestions

- Agreed:
  thin client for mutating commit-path commands only;
  activation by the OS;
  no fallback writer;
  loud failure.
- Added:
  the service exits when told to `retire` and when idle,
  so replacement and release need no install-time restart
  command;
  the client may not run `systemctl`.
- Added:
  queue keyed by common git directory,
  not by worktree,
  because refs and objects are shared.
- Not decided here and listed for the owner:
  whether `cli-git service install` (a user-run command that writes the unit
  files) is acceptable under the rule that the wrapper writes no scripts;
  unit files are configuration,
  not shell text.
- The prototype's `flock` and its stale-socket probe are not carried over;
  the abstract socket and the manager replace them.

## Acceptance test from issue 1770 (scoped commit with an unrelated tracked LFS asset)

Issue 1770:
a scoped explicit-path commit exits 128 in a 2 GiB,
2 CPU consumer container at an unrelated tracked LFS
asset,
with "Error cleaning Git LFS object:
open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/<n>:
no such
file or directory";
the same commit succeeds through the host wrapper.

Checked on this branch,
with the real `git-lfs` from the toolchain:
- `git-lfs` creates `lfs/` (with `objects` and `tmp`) on demand inside an existing git directory:
  `git lfs clean` against a
  freshly created bare repository with no `lfs` entry exited 0 and left `lfs/objects` and `lfs/tmp`.
  So Git not creating
  `lfs/tmp`,
  in `git init` or in `shadow_skeleton.rs`,
  is not the cause,
  and the in-process skeleton changed nothing for
  this issue (it creates the same entries as `git init --bare --template=`,
  compared by a test).
- Reproduction attempt:
  a repository with `*.bin` tracked through the LFS filter (64 MiB asset),
  the asset made stat-dirty
  (same content,
  new modification time,
  so the clean filter has to run for it during the shadow commit),
  and a scoped
  commit of another file,
  through the fixed wrapper and the installed wrapper,
  in a unit limited to 2 GiB of memory
  (no swap) and 200% CPU.
  Both exited 0,
  the commit landed,
  the status was clean.
  Not reproduced.
- Reading:
  an `open` that fails on `<shadow>/lfs/tmp/<n>` after `git-lfs` would have created `lfs/tmp` itself means the
  shadow directory (its parent) was gone while `git-lfs` ran.
  That fits the shadow being removed under a live
  transaction by recovery in another process,
  which is the same family as issues 1781 and 1784:
  a recovery that decides a
  transaction owner is dead by process identity,
  here possibly between a container and the host sharing one worktree
  (a service in one PID namespace sees clients of another as process id 0,
  measured above).
  It is a hypothesis;
  the
  container was not available.
  The new design removes the whole class (no shadow directory,
  no owner record,
  no recovery
  by process identity).

Acceptance test for the service:
a repository with a tracked LFS asset that the commit does not name,
an explicit-path
commit of another file,
the asset stat-dirty,
run with the real `git-lfs` on the client's PATH,
inside a unit limited to
2 GiB of memory and 2 CPUs,
repeated with the client in a PID namespace (`unshare --user --pid`) and the service outside
it.
It must exit 0 with the commit landed,
without the client printing any path under a temporary directory.
The client
runs the clean filter during preparation (in its own process,
under its own limits),
never the service.

## Slice 1b status (what is wired, what is measured, what is left)

Wired:
`CLI_GIT_COMMIT_SERVICE=1` (the user's runtime directory) or `abstract:<name>` (tests) sends the plain scoped commit
(`git commit -m <message> [-m <message>] [--quiet] -- <paths>`) through the service.
The wrapper prepares it itself (a
private index seeded from the real index,
the policies,
no shadow repository,
no capture lock),
the service lands it,
and
the result is the commit native Git would have made:
same tree,
same message after Git's clean-up,
author and committer
from `git var`.
The differential test compares the two commits field by field.
Everything else (amend,
`-a`,
an editor,
`-F`,
signing,
dates,
trailers,
a merge in progress,
an unborn or detached head,
a policy that rewrites a file,
nothing to
commit) is declined before anything changes and runs through the existing transaction,
so the old locks,
shadow,
replay and
recovery still serve those command lines.
With the opt-in on and no service reachable the client prints the plain texts of
`service_messages.rs`;
there is no second writer.

Measured on the 20 207-file clone,
release binaries,
host load 70 to 90,
six alternating scoped commits of one file:
- native path:
  757 to 10 149 ms,
  median 1474 ms.
- service path:
  419 to 932 ms,
  median 572 ms. The service's own step is 50 to 70 ms;
  the rest is the wrapper's preparation
  and the policies,
  not yet optimized.

Real systemd socket activation (transient units,
`systemd-run --user --socket-property=ListenStream=...`):
the first
commit started the service on demand and landed in 95 ms end to end with debug binaries,
a warm commit took 73 ms,
and after
`SIGKILL` of the service the next commit started it again through the manager and landed (1.1 s,
the manager's restart
delay).
The unit templates are `src/native/service_units/cli-git@.socket` and `cli-git@.service`;
a test reads them against the
socket path the wrapper uses and the resource rules (IOWeight at most 50).

Tests added in this step:
a commit from inside a PID namespace lands;
a scoped commit beside a stat-dirty tracked LFS asset
(the shape of issue 1770,
real `git-lfs`) lands with no temporary path in the output,
through the service path;
a service
killed at four arbitrary moments with sixteen clients in flight loses and duplicates nothing.
A guard break (the service path
returning early) fails the differential test because no batch reaches the journal.

Left for later slices:
the install command that fills `@SERVICE_BINARY@` and `@GIT@` in the unit files and enables the socket;
making the opt-in the default;
the amend,
`-a`,
`-F`,
editor and signing command lines;
moving the after-commit push and
maintenance out of the command into work items;
per-repository parallel landing;
the 2 GiB,
2 CPU container variant of
issue 1770;
deleting the old landing path with its locks,
shadow,
replay and recovery once no command line reaches it.

## Install command, default and the uninstalled machine (superseded by "Provisioning")

`git cli-git service install` fills the two unit templates (`@SERVICE_BINARY@` is the `cli-git-service` program next to the
wrapper,
`@GIT@` is the real Git the wrapper resolves),
writes only the files that differ,
runs `daemon-reload` and
`enable --now cli-git.socket`.
It never starts the service:
the socket is the only unit started,
and the operating system
starts the service on the first connection.
A second run changes nothing and says so.
`uninstall` disables and removes
both units and is repeatable;
`status` exits 0 only when the socket is active.
All three are answered by the wrapper
without Git.
The unit files are configuration the service manager reads;
the wrapper writes no script or shell text.

The commit path is chosen by `CLI_GIT_COMMIT_SERVICE` and the files on disk,
without starting a process:
- unset:
  the service is used when the socket unit file exists in the account's `systemd/user` directory,
  or a socket
  already exists at the runtime path.
  Otherwise the commit takes the path it took before.
- `0`:
  never the service.
- `1`:
  always the service;
  without the activation installed the commit stops at once with the not-installed text that
  names `git cli-git service install`,
  and nothing in the repository changes.

Decision for the uninstalled machine:
until the old landing path is deleted (the last step of the migration),
an
uninstalled machine keeps it,
because that path is the only writer such a machine has,
and removing it before the service
covers every command line would stop commits.
This is the present writer,
not a fallback:
an installed machine never
falls back to it (an installed machine whose service does not answer gets the plain stop of the failure catalogue).
Once
the old path is deleted,
the unset case on an uninstalled machine becomes the same plain stop as `1`,
and the install
command becomes a precondition for committing.
Rejected:
installing on first use (the wrapper would write service-manager
configuration without being asked),
and a lock-based fallback writer on any machine (the owner's decision:
locks are an
anti-pattern).

Tests:
`service_commit_tests.rs` (the choice from value,
runtime directory and unit directory),
`service_install_tests.rs`
(a stand-in service manager program records every call),
`binary_service_default_tests.rs` (installed and unset lands
through the service,
`0` stays native,
uninstalled stays native,
`1` uninstalled stops plainly and quickly).

## After-commit work as built (push and upkeep)

A commit the service landed no longer pushes or runs upkeep inside the command.
After the post-commit gate passes (the gate
still runs in the wrapper,
and a blocked gate still exits 2 and leaves the push undone),
the wrapper writes two work items,
`push-<oid>` and `maintenance-<oid>`,
with `work_item.rs` (whole files,
not synced,
the same landing always yields the same
name),
and sends one `work` message naming the common Git directory.
That message starts the service through the operating
system when it is not running,
exactly like a commit does,
and the wrapper returns when the service has read it.
The
protocol version is now 2 so an older service is asked to retire first.

The service runs the work on its own thread (`service_work.rs`),
one repository at a time.
Pending pushes of one branch are
claimed together and become one push of the branch (`push_once` in `auto_push.rs`,
the same decision as the wrapper's own
auto-push but with no lock,
because the service is the only pusher of the repositories it serves);
upkeep runs once per
repository.
A claim that was held by a service that died is taken back when its lease runs out (120 seconds;
the holder renews it
three times in that span while it works,
so a push that outlasts the lease keeps its claim) and the item is tried again up to
three times.
Items of other kinds,
the standards review,
are never claimed.

A service that starts looks at its repositories without a wake (2026-11-04,
`service_claim_watch.rs`).
Before this,
the service
learned which repository to drain only from a wake,
which only a commit sends,
so a service killed while it held a claim was
started again by the next connection and then had nothing to drain until the next commit,
which might never come:
the backup
push of the last commit of the day waited without a bound ("nothing should ever wait forever or indefinitely").
The work thread
now writes the Git directory of every wake it is about to drain as one small file in `claim-watch/` of the instance's own journal
(once per directory per process,
before the first claim,
never on the landing thread),
and a starting service reads those files
and puts each directory on the same schedule that the lease wake uses,
due after the schedule's one second of slack.
That pass
runs what is pending,
takes back what has run out its lease,
and says when the first claim still held will run out,
which the
schedule waits for;
the service counts itself busy while the schedule is not empty,
so it does not exit idle before then.
A
directory with no work directory any more has its file removed at the start.
The bound after a kill is therefore the lease plus
one pass,
with no commit needed,
and the lease still decides that a holder is dead:
a claim renewed a moment ago is left alone.
In production the manager starts the service again by itself after a kill:
the instance unit sets `Restart=on-failure`
(`service_units/cli-git@.service`),
a signal ending counts as a failure,
and `RestartSec=` defaults to 100 ms (`systemd.service(5)`,
systemd 262 on the owner's host,
read with `systemctl --user show -p DefaultRestartUSec`).
The wait after a SIGKILL with nobody
committing is then that restart,
the schedule's one second of slack for the first look,
and the lease.
A service the manager
gives up on (its start limit,
five starts in ten seconds by default,
is for a crash loop) is started by the next connection,
as
before.
The service exits idle only after three hours without work (`IDLE_EXIT`) and counts itself busy while the schedule holds
a repository,
so it is not gone when the lease runs out.

The note and its limits (2026-11-04,
after the cross-vendor review of the startup look).
A note holds the bytes of the Git
directory in hexadecimal after a format line,
so a path that is not text or ends in a newline is kept as it is;
it is written whole
under a staging name and published by one rename (`write_private_file_unsynced`),
so a kill leaves a whole note or none,
and it is
not synced:
it survives the end of the process,
as the work records it points at do,
and not a power cut.
An instance is the hash
of the canonical path of its Git directory,
so the instance is the repository:
a note is used only when its path names this
instance (a path that now leads through a link to another repository,
a copied journal and a damaged note do not),
and its work
directory must be a real directory,
not a link.
Only a note that is certainly of no use is removed:
it holds no path,
names another
instance,
or its work directory is absent or not a directory.
A read that fails (the store,
a note,
the look at the work
directory) or a write that fails leaves the note where it is and is tried again after the pause of a push that is tried again,
doubled for each failure in a row and never past ten minutes,
and the service counts itself busy until it works,
so a refused read
or a full disk delays the look and never ends it;
the commit and the work in hand go on meanwhile.
A name taken by another
directory's note is skipped,
and a damaged note under the name is replaced.
The startup look is scheduled for a repository even
when a push plan names it with a later time,
because the plan is the push's own schedule.
A repository that moved is another
instance and is found by its own wake.
A journal holds one note per spelling of its instance's Git directory that clients used.
What is not covered:
a service killed while it holds a claim and while its first note could not yet be written,
which needs two
faults at once;
the claim then waits for the next wake.
A shared journal of several live services of one instance is not assumed.

How a claim is held (2026-11-01,
the service queue fix).
A claim is one file in `claimed/` named
`<id>~<generation>~<boot id>~<monotonic ms>~<wall s>`:
the generation is unique to one acquisition and the last two parts are
the holder's clock reading at its last renewal.
A renewal renames the file to the same name with a new reading,
and a
settlement renames it to `done/<id>`,
both from the exact name the holder last wrote,
so a holder whose claim was taken over
finds nothing to renew or settle and is told;
it can never touch a successor's claim of the same item.
Recovery takes a claim
by renaming the exact name it looked at into `recovering/` (the file that holds the item while it is returned,
so it is in
exactly one place),
rewrites it there with one more try,
and renames it to `pending/` or `failed/`;
a claim that was renewed or
claimed again since the look has another name and is not taken.
Within one boot a lease is judged by `CLOCK_MONOTONIC`,
which a
change of the wall clock does not move;
a claim whose boot id differs from the judge's has run out at once,
because a restart
ended its holder and the wall clock,
set back at boot,
would only strand it (2026-11-02;
until then the wall clock judged
across boots);
where either side has no boot id,
by the wall clock (`claim_time.rs`).
A claim by an earlier release (named by its id alone) is aged by its modification time,
on the wall clock,
and is given a
generation when it is renewed.

What the person sees:
the commit returns without waiting for the network.
A failed push settles its items as failed and
leaves one notice file under `<common git dir>/cli-git/notices/`.
The file holds the branch,
the whole commit name and Git's
words with every URL's credentials already masked (only the user name and password of a URL are masked,
never the line around
them);
the next wrapped command that is not machine-read prints it once and removes it,
in the worktree that has the branch
checked out (in any worktree when none has it),
naming the commit with the abbreviation Git gives it in the repository as
it then is (`landed_summary.rs`,
`object_abbreviation.rs`).
A file an earlier release wrote,
which is finished text,
is shown
as it is.
The notice says the commit exists only on this computer for now,
repeats what Git said,
and says to run
`git push` when ready.
When the push was stopped at its deadline the text gives the remedy that reaches the service (the
user manager's `systemctl --user set-environment` and a restart),
not a variable of the terminal
(`push_deadline_advice.rs`).
A handoff that could not be written or announced prints the plain `WorkNotHandedOver` text and does not
change the exit status;
the records stay pending and the next wake for the repository runs them.

Environment limit,
stated plainly:
the push runs in the service's environment,
which is the one the unit file gives it,
not
the terminal's.
A push that needs a variable only the terminal has (an ssh agent address in particular) can fail there.
The
failure notice says so.
The fix on the person's side is to make the variable part of the user manager's environment
(`systemctl --user import-environment`,
or an `environment.d` entry);
recording a terminal's environment in a work item was
rejected because the file would carry secrets.

Not yet moved:
the push of the old landing path (it still pushes inline,
for the command lines the service does not cover
and for an uninstalled machine),
and the post-commit policy gate (it stays in the wrapper because it decides whether the
push may run).

Tests:
`service_work_tests.rs` (coalescing,
kinds,
notices shown once,
refused items,
lease and tries),
`service_handoff_tests.rs` (records before the wake,
a repeat adds nothing,
notices),
protocol round trip for the new
message,
and two binary tests with a real service and a bare upstream (the push happens after the commit returned and both
records end in `done`;
a vanished upstream fails the push,
not the commit,
and the next commit prints the notice once).
The
handoff guard was shown failing by dropping the push record from the handoff.

## Command lines the service covers now, and what still goes through the transaction

Covered,
each compared field by field with the commit native `git commit` makes (tree,
whole message,
author with date,
committer,
parents) and with what is left staged,
modified and untracked:
- `-m` (repeated),
  `-F <file>` in each spelling,
  `--cleanup=whitespace|default|verbatim`,
  `--allow-empty-message`.
- `--author=Name <email>`,
  `--date=<date>` (Git's own `var` reads the date),
  `--trailer=<t>` (Git's `interpret-trailers`).
- An index commit and `commit -a` (the shipped commit-only enforcement blocks both unless `--no-enforce-only` or
  `--no-only` is given,
  so they are rare here);
  named paths as before.

Declined to the transaction,
before anything changes:
`-F -` (standard input is Git's to read),
a message from both `-m`
and `-F`,
`--author` without an email (Git searches history for it),
`-s`,
`-S` and `commit.gpgSign`,
`-e` and the editor,
`-c`,
`-C`,
`-t`,
`--cleanup=strip|scissors`,
`--amend`,
`--allow-empty`,
`--fixup` and `--squash`,
`-i`,
`-p`,
`--dry-run`,
`--pathspec-from-file`,
a conclusion of a merge,
a cherry-pick or a revert,
an unborn or detached head,
and a repository whose
configuration sets `commit.template`,
`commit.cleanup`,
`i18n.commitEncoding`,
`core.commentChar`,
`commit.signoff`,
`commit.verbose` or any `trailer.*` key.
The last group closes a gap that existed before:
the service path used to ignore
those settings silently.

What stands between this and deleting the old landing path,
shadow,
replay and locks:
the transaction still serves the
declined group above,
and several of those need real Git to build the commit (the editor,
signing,
sign-off,
amend).
The
design question is how the service lands a commit that real Git built.
The proposal,
for the owner or coordinator to accept
before the deletion:
the wrapper runs real `git commit` against a thin scratch Git directory (its own `HEAD` at the base,
the
private index,
the private object directory with the real object store as an alternate),
and ships the resulting commit's
identity to the service,
which lands it by comparing and swapping the branch when the base is still the branch value,
and by
asking the client to prepare again when it is not (a signed commit cannot be rebuilt on another parent).
That keeps Git as
the only author of the message,
trailers,
signature and amend semantics,
takes no lock,
and is a smaller piece of code than
the shadow,
replay and recovery machinery it replaces.
It does not remove the scratch Git directory;
it removes the shadow's
landing,
replay,
registry and recovery.

### Decision for the remaining command lines (coordinator, owner veto open)

A hybrid.
First,
embed every option that is pure message text or parent arithmetic,
proved field by field against native
Git (tree,
whole message,
author with date,
committer,
parents,
left-behind status),
on the owner's rule "Invoke real git as
rarely as possible."
Second,
only where Git must run a program (an editor,
signing) run real `git commit` against a thin
scratch Git directory and land the result by compare-and-swap or `retry_prepare`;
the wrapper itself never launches the
editor or the signer,
Git does,
as it does today.
Keeping the old transaction for those lines is out,
because it keeps
the locks the owner banned.

Sub-step 4b-1,
built and compared with native:
`-s` and `--signoff` (Git's trailer interpreter,
with the cases of a
trailer block before it,
the same sign-off already last,
another person's sign-off last,
a `---` divider in the body),
`--allow-empty`,
`-C <commit>` (message and authorship reused),
`--fixup=<commit>` (plain commit names only),
`--cleanup=strip` (default comment character only;
any `core.commentChar` setting declines).
`--squash` always opens an
editor natively,
so it is not embedded.

Sub-step 4b-2,
built and compared with native (`binary_service_amend_tests.rs`,
field by field:
tree,
whole message,
author
with date,
committer,
parents,
status,
staged difference and commit count):
`--amend` with `-m`,
`-F`,
`-C`,
`--fixup`,
`-s`,
`--trailer`,
`--author`,
`--date` or `--cleanup`,
or with `--no-edit` (the replaced commit's message,
cleaned as Git
cleans it),
scoped to named paths or over the whole index or `-a`;
`--reset-author` on a plain amend (no `--author`,
`--date` or reused commit);
a root commit and a merge commit (the parents are the replaced commit's parents,
whatever
their number);
`--allow-empty`.
Parents come from the replaced commit,
its author is kept unless the line replaces it,
and
a replaced commit with a header an amend would have to carry over (`encoding`,
a merge tag) stays native.
An amend that
would leave the commit equal to its first parent (a merge excepted) is declined,
because native Git refuses it.

The landing rule of an amend:
the client sends `amend` and the `tree` it wrote into the object store;
the service builds the
commit with the replaced commit's parents;
the request lands alone (never in a batch with others,
and what arrives after it
starts a new group) and only while the branch tip equals the base the client read.
Otherwise the service answers
"branch moved" (`FailureCode::BranchMoved`),
which the client prints as a plain failure;
an amend is never rebuilt on another
tip,
because it would replace a commit the person did not see.
The protocol version is 3 so an older service retires first.

### Commits Git builds (step 4c)

Every command line the service cannot build from the command line itself is built by real `git commit` in a thin scratch
repository (`service_native.rs`),
and the service lands the result unchanged.
That covers the editor (`-e`,
no message
option,
`-c`,
`-t`,
`--squash`,
`--fixup=amend:` and `reword:`),
signing (`-S`,
`--gpg-sign`,
`commit.gpgSign`),
and,
as the
general fallback while the embedded route is the optimization,
the lines whose meaning is Git's own text work:
`-F -`,
`--author` without an email,
`--cleanup=scissors`,
and a repository configuration that sets `commit.template`,
`commit.cleanup`,
an encoding,
a comment character,
`commit.signoff`,
`commit.verbose` or `trailer.*`.
Real Git is started
only after the embedded route declined,
so the common lines still start none to build the commit.

The scratch repository is made inside the commit's own scratch directory (`<git dir>/cli-git-service-<id>/shadow`),
away
from the registry directory recovery scans,
so no other process can mistake it for a dead transaction's shadow.
It reuses
`shadow_create.rs` (the config that includes the real one with every hook off,
`HEAD` and the branch at the base,
a
snapshot of the other refs so `-C v1.0` and `--fixup=<ref>` resolve,
the links that let `git-lfs` find its directory) and
`run_native_commit` (the signal guard,
the person's terminal for the editor).
The one difference is that Git writes its new
objects straight into the real object store (`GIT_OBJECT_DIRECTORY`),
because a branch cannot point at an object that is
only in the scratch store.
The commit is an unreferenced loose object until the service moves the branch to it,
like the
blobs the client already writes.

The request names the commit (`commit`,
protocol version 4),
with `amend` when it replaces the tip.
The service checks that
the name is a commit whose parents are the base (or,
for an amend,
the base's parents),
lands it alone and only while the
branch tip equals the base,
and otherwise answers "branch moved".
There is no second attempt for these commits:
an editor
text or a signature is not rebuilt on another tip,
which matches the earlier transaction (it never replayed an amend,
and
a signed commit needed the person's signing again).
A person who signs every commit and commits to a branch other people
commit to at the same moment will see that failure more often than someone who does not sign;
automatic rebuilding of a
commit with no editor is the follow-up that would remove it.

A policy that rewrote a file still declines to the transaction,
as do an unborn or detached head,
a merge,
cherry-pick or
revert conclusion,
an unmerged index,
`--dry-run`,
`-i`,
`-p`,
`--include` and `--pathspec-from-file`;
their landing is
what step 4d replaces,
their preparation stays.

Still declined after 4b-2 and before 4c,
each with its reason (now built by Git as above):
`-e`/`--edit`,
`-c`,
`-t`,
no message option (an editor opens),
`--fixup=amend:` and `--fixup=reword:` (editor),
`--squash` (editor),
`-S`,
`--gpg-sign` and `commit.gpgSign` (signer),
`--reset-author` with another source of authorship,
`--author` without an email (history search),
`-F -`
(standard input),
`-i`,
`-p`,
`-u`,
`-v`,
`--dry-run`,
`--pathspec-from-file`,
a message from two sources,
an unborn or
detached head,
a merge,
cherry-pick or revert conclusion,
a policy that rewrote a file,
an unmerged index,
a repository
configuration that sets a template,
a clean-up mode,
an encoding,
a comment character,
trailers or sign-off.

### Owner correction on provisioning (verbatim, adopted)

The owner,
asked when to run `git cli-git service install` on this machine,
answered:
"What is going on?
Bazel uses client
server architecture and Bazel doesn't do this."
Their earlier rule:
"socket activation (OS starts it) or a loud,
clear
failure - no,
never leak internal things that could be resolved to the user."
Bazel's client/server page says:
"If the
client cannot find a running server instance,
it starts a new one.";
the client installs what it needs itself;
and on a
version mismatch "the server is stopped and a new one started".
So the install step,
the installed-or-not default and the
`CLI_GIT_COMMIT_SERVICE` switch are internal things the user must never see.
Adopted:
the wrapper provisions socket
activation itself the first time a commit needs the service.

## Provisioning

Supersedes the install command,
the installed-or-not default and the `CLI_GIT_COMMIT_SERVICE` switch,
which were removed.
The owner's correction (verbatim above) is the requirement:
the first commit that needs the service makes socket activation
exist,
as Bazel's client starts its server.

What the wrapper does,
in order,
whenever the session bus socket and the wrapper's own executable (which is also the service,
see "One executable") exist (an existing socket does not end it:
see the "Upgrades heal on the first commit" section).
The first two steps and the
bus calls are skipped when the check there finds nothing to change:
1.  Remove units of an earlier release from `~/.config/systemd/user` (they outrank runtime units in the unit search path),
    recognized by content,
    plus the enabling link they left.
    A file of the same name that something else wrote is never
    touched;
    the commit stops with a message naming the file.
2.  Write the templates `cli-git@.socket` and `cli-git@.service` into `$XDG_RUNTIME_DIR/systemd/user` through a temporary file and a rename,
    only when the content differs.
    Nothing remains in the home directory,
    and the runtime directory disappears at logout,
    so
    the next first use provisions again.
    Concurrent first commits agree through idempotence alone:
    identical content,
    atomic
    renames,
    no lock.
3.  Reload the manager only when a unit file that already existed changed or went away (an upgrade moved the program).
    A new
    file needs no reload:
    measured on this machine,
    the probe unit of
    `the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit` was found and started with no reload,
    while one
    reload took over twenty seconds on the loaded host and interrupted other clients of the manager for that time.
4.  Clear failed state (`ResetFailedUnit` on both units,
    errors ignored) when a unit file was already there,
    then start
    `cli-git@<instance>.socket` through the manager.
    The manager,
    not the wrapper,
    starts the daemon on the first connection,
    so
    the daemon is in its own cgroup (`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`).

The calls go to the user session bus (`DBUS_SESSION_BUS_ADDRESS`,
else `$XDG_RUNTIME_DIR/bus`) as `org.freedesktop.systemd1`
on `/org/freedesktop/systemd1`:
`Hello` on the bus daemon,
then `StartUnit(ss) -> o` (and `Reload()` when needed),
the
signatures in `org.freedesktop.systemd1(5)`.
The manager's private socket is not used:
`systemd(1)` calls it private to
systemd.

Build versus buy for the D-Bus client.
zbus would add roughly fifty crates and an executor to a binary that starts on every
git command;
`dbus` links the C library;
the only small pure-Rust candidate,
rustbus,
is not in the offline registry cache.
The wrapper needs two methods with string arguments,
so it carries a hand-written client of a few hundred lines
(`systemd_dbus.rs`):
the writer and the reader cover the types those calls and the bus's replies use (`s`,
`o`,
`g`,
`u`),
the reader is total over arbitrary bytes (a unit control walks every prefix and single-byte change;
the `dbus_reply` fuzz
target in `cli.fuzz` holds it to that),
and the whole exchange is bounded by one deadline of twenty seconds so a stuck bus
never hangs a commit.
The layout of a call is checked against a message worked out by hand from the specification,
and the
whole client was verified once against the real user manager with a uniquely named runtime-only probe unit (written,
started,
found listening,
stopped,
removed;
the `#[ignore]` control `the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit`).

### Retiring the single service of the earlier releases (findings 1 and 2 of the 2026-11-01 review)

- The retirement stops the earlier `cli-git.socket` and `cli-git.service` wherever their unit files are:
  the runtime unit
  directory,
  and `~/.config/systemd/user`,
  where 0.5.1 wrote them (`cli-git service install`).
  A socket file of the earlier
  service with no unit file of that name anywhere also sends the stops,
  because a release that removed the files without
  stopping the service left exactly that;
  a file of that name that something else wrote blocks this.
- A stop call returns when the job is queued,
  so the wrapper then asks `GetUnit` and reads the `ActiveState` property (a
  variant holding a string) until both units are inactive,
  failed or not loaded,
  for at most ten seconds.
  A stop the manager
  refuses,
  or a service still stopping when the wait ends,
  fails the commit with a message naming the unit and the
  `systemctl --user stop` remedy;
  the unit files and the socket file stay,
  so the next commit finds the same state and tries
  again.
  Nothing is started beside a service that may still be landing a batch.

### Limit: a downgrade to a release with one service for every worktree (finding 3)

Not fixed,
by design of what a release can do.
A wrapper of an earlier release (0.5.5 and before) that runs after this one
provisions the per-user writer again (`cli-git.socket`,
`cli-git.service`) without looking for per-worktree instances,
and the
instances this release started keep running until their three idle hours pass.
Both generations can then write the same
worktree,
which breaks the rule that every claim and index lock left at start belongs to a dead predecessor.
This release
cannot change what a released wrapper does,
and no file it leaves behind can stop one:
the earlier wrapper rewrites its own
unit files by rename.
The remedy for a downgrade is to stop the worktree instances first
(`systemctl --user stop 'cli-git@*.service' 'cli-git@*.socket'`) and to commit once with the older release only after they are
gone.
A later release that adds a way for the instances to notice the single service's socket and exit would narrow the
window but not close it,
so it is not promised here.

### Upgrades heal on the first commit

Defect found 2026-11-01 in 0.5.4:
the route returned early when the manager's socket existed,
and the socket outlives an
upgrade,
so a unit written by 0.5.2 kept naming `.../0.5.2/bin/cli-git-service` after the install removed it.
The service
failed with `status=203/EXEC`,
five restarts in two seconds,
`service-start-limit-hit`,
and the client waited its ten second
start bound (`doc/troubleshooting/cli-git-service-unit-names-a-removed-program-status-203-exec.md`).

The decision now:
- `choose_route` hands back the provisioning host whenever the bus and the program exist,
  socket or not.
  It still reads only
  the file system,
  and still returns the socket as it is when it cannot provision.
- `provision` first calls `in_place`:
  both runtime units read and compared with the text the wrapper would write,
  plus one
  `stat` of the socket.
  Equal means no bus connection at all (no rewrite,
  no reload,
  no call).
  Different or missing means the numbered provisioning steps under the Provisioning heading,
  so an upgrade or a downgrade is repaired by the first commit that follows it,
  with no step for the person.
- When the manager refuses after a unit was rewritten,
  the rewrite is withdrawn,
  so the next commit does not take a file the
  manager never reloaded for an installed one.
- Cost of the fast path,
  measured 2026-11-01 with `strace -f -c` on the `in_place` check alone (unit test binary,
  runtime
  directory on the host file system,
  run 0 against 1000 iterations):
  per check 2 `openat`,
  3 `statx`,
  4 `read`,
  2 `close`,
  eleven system calls,
  and no bus connection.
  Time per check without `strace`:
  20.6 microseconds in a debug build (181
  microseconds under `strace`,
  which is the tracer's cost).
  A bus connection costs a connect,
  an authentication round trip and
  a `Hello` round trip before any call,
  so the check is the cheaper side by orders of magnitude.

A service already running from the other release is handled by the handshake,
not by the units.
The `welcome` carries the
release (the crate version,
shared by the wrapper and the service program).
A service whose release is not this wrapper's,
newer or older,
or that sends none (every release before this field),
is sent `retire`:
it finishes its batch and exits,
the manager's socket stays bound,
and the next connection starts the program the rewritten unit names.
A retiring service
still accepts connections until it exits,
so the client asks each connection again until the start wait (ten seconds by
default) passes,
then fails with a message naming `systemctl --user restart cli-git.service`.
No lock and no unbounded
loop.
`retire` is accepted whatever protocol version the sender names,
which is what lets a future protocol bump be
replaced the same way;
a service of a newer protocol still ends the command,
because it cannot be asked.

An unrunnable `ExecStart` that the wrapper did not write (a `noexec` mount,
a security policy) cannot be told from a slow
start without reading the unit's exit status,
which needs variant-typed `Properties` replies that `systemd_dbus.rs` does not
read;
it stays the loud message with the journal command.
A stale path,
which was the observed cause,
can no longer arise
because the unit text is compared on every service commit.

Failures follow the owner's rule:
no session bus or no user manager is the plain message with the remedy for the platform;
a
refused call says the service manager refused and gives its words,
without its internal error name.

Idle exit and version skew follow Bazel.
The service exits after three hours with nothing to do (`--idle-seconds`,
default
`IDLE_EXIT`;
Bazel's `--max_idle_secs` default is three hours);
an announced push or upkeep keeps it alive.
On a version
mismatch the client asks the older service to retire,
which drains its queue and exits,
and the next connection activates
the new program.

What a machine without a user manager does:
since step 4d part 2 the commit stops with the plain message,
because the earlier
landing path is deleted.
The paths that would hit that message (the gate containers,
CI release jobs,
`mise` tasks that commit)
were listed and checked first:
see "Paths that would hit the no-manager failure",
and "Step 4d,
part 2" for the result.

### Landing is replaced, preparation is kept (coordinator decision, owner veto open)

For the command lines the service still does not cover (editor,
signing,
amend and the rest of the declined list),
the
transaction's preparation stays:
policy rewrites installed into the worktree,
unborn and detached heads,
and merge,
cherry-pick and revert conclusions.
Only its landing step becomes the service's compare-and-swap,
which is the one place
the owner's ban applies ("Didn't I say locks are bad?":
the ban is on locks and polling,
not on preparation logic).
Step 4d
then deletes the landing machinery (the owner lock and its records,
replay and the reservation,
landing recovery,
the
shadow when it served landing),
not the preparation.

### Measured reload of the user manager (record, no claimed connection)

The first probe against the real user manager called `Reload` and then `StartUnit`.
Its test log was last written at
2026-11-01 09:59:48 EST,
the run took about forty seconds (two attempts of up to twenty seconds each,
both cut by the
deadline),
and the connection errors `systemd-run` printed in the next minutes were "Transport endpoint is not connected".
The machine's boot time afterwards reads 2026-11-01 10:08:37 EST (`uptime --since`).
These times are recorded so they can
be compared with the reboot's cause later;
no connection between them is claimed.

### Paths that would hit the no-manager failure (measured before 4d, nothing deleted)

The failure applies only after 4d,
to a commit that runs the wrapper with no socket and no session bus (the route reads
`XDG_RUNTIME_DIR`,
the socket under it,
the session bus socket,
and `cli-git-service` beside the wrapper).
Checked from the
repository's workflows,
tasks and harnesses:

Unaffected,
with the evidence:
- `mise.toml` line with `cli-git-native cli-git fix --all --policy mono/dependent-version-bump`:
  a management command,
  not
  a commit.
- `.github/workflows/catalog-specifiers.yml`:
  installs `cli-git-native` under that name only and runs `cli-git check`;
  the
  `git add` there is real Git.
- `.github/workflows/npm-release.yml`:
  exposes the executable "under its own name,
  never as git" (its own comment),
  so the
  changesets action commits with real Git.
- `.github/workflows/crate-release.yml` and `cargo-publish.yml`:
  no step installs the wrapper as `git`.
  The release commit is
  made by `package/crate-release/cli/src/release-steps.ts` (`git commit --message ... -- <paths>`) with the runner's real
  Git,
  and the bump reaches main through the GraphQL API.
- `mise` tasks:
  none runs a commit (the only matches are comments about the CI version commit and a formatting-commit parse
  gate that a person runs first).

Affected (each needs a service the harness starts,
or it stops with the platform message):
- The gate (`bin/test-native-container.ts`,
  `native:test:host`,
  `native:test:binary:host`) and the mutation runs
  (`bin/mutate-native-container.ts`,
  `native-planted-controls*.ts`):
  the binary tests build their environment with
  `env_clear()` and no `XDG_RUNTIME_DIR` (`binary_support.rs`),
  so every wrapped commit takes the earlier path today.
  Forty-two test files run a wrapped commit,
  one hundred fourteen calls of `wrapped_commit`/`run_commit`.
  The place to fix
  is the one constructor of the fixture environment:
  start a `cli-git-service` per fixture at a short runtime directory (as
  `start_in` already does for the service tests) and set `XDG_RUNTIME_DIR`.
- `bin/e2e-native-container.ts` and the concurrent-commit scenarios in `e2e/`:
  the image installs the wrapper as `git` and
  replays workloads in `podman run` with no bus;
  the harness must start the service in the container before the scenarios.
- `bin/interop-native-container.ts` and `native-interop/`:
  they exercise old-format transaction states (registry,
  recovery);
  they retire with that code,
  except the one-time reader.
- `bin/native-perf.ts` and `native-perf-git.ts`:
  they commit through the wrapper on the host,
  where a bus exists,
  so they
  would provision the person's own units as a side effect;
  they should start their own service instead.
- A shell with `XDG_RUNTIME_DIR` unset:
  cron (the default `crond` PAM stack has no `pam_systemd`),
  containers and toolbox
  shells without the runtime directory mounted,
  and an agent sandbox that does not mount `/run/user/<uid>` (the earlier
  sessions' note that `systemd-run` fails inside the sandbox is the sign).
  `ssh` gets the runtime directory and the bus while
  the user manager runs (a login session,
  or linger).
- Nested wrappers (`rebase --exec`,
  hooks) inherit the caller's environment and need nothing more.

### What the binary suite shows with the service as the only route (measured once, before 4d)

`CLI_GIT_TEST_SERVICE=1` gives every fixture its own service and the runtime directory (`binary_support.rs`),
so every wrapped
commit takes the person's route.
One run of the whole binary target (423 tests,
159 s,
`--no-fail-fast`):
341 passed,
82
failed.
The failures are what 4d must settle,
grouped by cause:
- Tests of the machinery 4d deletes (they assert replay,
  reservation,
  recovery or the lock,
  so they retire with it):
  24 in
  `recovery_tearing`,
  5 in `commit` (replay and reservation),
  1 `event_path_commit` (a replayed commit),
  1 `merge_isolation`
  (a commit waiting for a running merge),
  8 `wait_notice` (lock wait notices),
  17 in the `fault_*` files (kill,
  filesystem,
  space,
  limits,
  stream,
  signal and remote faults of the old landing,
  which need new service-side equivalents where the
  fault still applies),
  5 `budget` (process and sync budgets of the old transaction;
  the service route needs its own budget).
- Tests that assert the push inside the command (the push moved to a work item):
  `lfs_pre_push` 4,
  `lfs_server` 3,
  `push_environment` 3,
  `hooks_off` 1,
  `index_pack_fsync` 2,
  `lfs_hang` 2,
  `external_hooks` 2.
  They have to be restated for
  the push the service runs after the command returned,
  including the upload before the ref moves.
- Two comparison tests (`service_options`,
  `service_amend`) that start their own service:
  with a fixture service in play their
  native baseline also takes the service,
  so under the switch they must name the earlier route explicitly or drop it with it.

The switch stays opt-in until 4d,
because before it those tests assert the earlier route;
at 4d it becomes the default.

### Every lock-like mechanism and what 4d does with it (owner: "Use as few locks as possible.")

Sizes are lines of code and of tests under `src/native` at this commit.
Dispositions are proposals for the coordinator's
go-ahead;
nothing is deleted yet.

Delete,
because the single-writer service makes it unnecessary (landing moves to the service's compare-and-swap):
- The owner-lock family:
  `owner_lock.rs`,
  `owner_lock_record.rs`,
  `live_lock.rs`,
  `commit_landing_lock.rs`,
  `transaction_lease.rs`,
  `transaction_owner.rs`,
  `ownerless_directory.rs`,
  `capture_order.rs` (the capture lock and
  sequence exist only so replay can order captures),
  `replay_reservation.rs`:
  3 022 and 2 917 lines.
- `real_index_lock.rs` and `recovery_dead_owner_locks.rs`,
  `recovery_unrecorded_lock.rs`:
  the wrapper's own creation of
  Git's `index.lock` for the landing,
  and the proofs that release a dead owner's copy.
  The service updates the index with
  `git reset`,
  which takes Git's own lock.
- `index_writer_coordination.rs` (and `index_writer_forward.rs`'s use of it,
  `management.rs`'s `hold_landing_lock`):
  today
  every forwarded index writer (`add`,
  `reset`,
  `checkout`,
  `stash`,
  `rebase`...) takes the landing lock and waits for a
  foreign `index.lock` first.
  With no landing lock to protect,
  such a command competes only on Git's own `index.lock`,
  as it does without the wrapper.
- Recovery and registry:
  `recovery_*.rs`,
  `commit_recovery.rs`,
  `transaction_registry.rs`,
  `transaction_gate.rs`:
  4 440 and
  3 532 lines between them;
  replaced by the service's journal replay.
  Keep a one-time reader that removes the old
  registry directory.
- Replay and landing:
  `replay_*.rs`,
  `landing_*.rs`,
  `commit_landing_loop.rs`,
  `post_landing.rs`.
- The push lock:
  `auto_push_single_flight.rs` (the lock and its join logic;
  `run_single_flight_push_repeating` is called
  only from `auto_push.rs`,
  the inline push after a commit),
  `holder_budget.rs` (a time budget that exists so a waiter's
  limit can be derived),
  `lock_holder_facts.rs`,
  the waits for it in `bounded_push.rs`,
  and the environment variable
  `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`.
  Pushes become service work items coalesced per branch (`push_once`).
  Manual `git push`
  never took this lock,
  so nothing changes for it.
- Correction (coordinator decision,
  after the concurrent end-to-end scenarios `foreign-index-lock-holder` and
  `foreign-index-lock-unproven` failed):
  the foreign `index.lock` waiting and holder-evidence family is NOT deletable.
  It
  moves into the service landing,
  which takes `index.lock` first (see "Landing every command line").
  Kept:
  `index_lock_holders.rs`,
  `index_lock_holder_parsers.rs`,
  `index_lock_evidence.rs` (the evidence and the holder lines),
  `live_lock.rs` for `live_wait_limit` and `LIVE_WAIT_VARIABLE` only,
  and the environment variable
  `CLI_GIT_LOCK_WAIT_SECONDS` (read by the service,
  so it belongs in the unit's environment).
  Deletable from the old family:
  the polling loop of `index_lock_wait.rs` and `index_lock_wait_host.rs` (backoff,
  jitter,
  tickers,
  `WaitEffects`),
  which
  the inotify wait in `service_index_lock.rs` replaces,
  and `CLI_GIT_WAIT_NOTICE_SECONDS`.

Replace with a lock-free mechanism:
- The transaction directory of the preparation that stays (unborn and detached heads,
  conclusions,
  policy rewrites):
  its
  owner record and registry become the commit's own scratch directory,
  removed with the commit,
  as `service_commit.rs`
  already does for the private indexes and `service_native.rs` for the scratch repository.
- The `last-pushed` record of `auto_push_single_flight.rs`:
  written whole by rename (`auto_push_record.rs`),
  read by the
  service's push;
  a lost update only repeats a push.
- `worktree_copy_lock.rs` (a per-repository settlement lock for linked-worktree ignored-state copies,
  with its
  `settlement.lock` directory,
  `CLI_GIT_WORKTREE_COPY_LEASE` variable and 20 ms polling):
  independent of the commit service.
  Each copy has its own destination,
  journal and stage,
  and `worktree_copy_stage_claim.rs` already claims a stage by exclusive
  creation;
  the replacement is a per-destination claim by exclusive create and rename,
  with recovery claiming a journal the
  same way.
  It is a slice of its own,
  with its own differential tests.

Keep,
because nothing else works:
- Git's own `index.lock`,
  honoured by the service's bounded retry (five tries,
  a tenth of a second apart,
  then the plain
  `IndexInUse` message).
  That is Git's lock,
  held by another Git process;
  the bound is the smallest that rides out a
  concurrent `git add`.

Not locks (exclusive creation used as a one-time claim,
never waited on),
so they stay:
`private_storage.rs` (`create_new`
for private records),
`worktree_copy_stage.rs` (exclusive file creation in a private stage),
`direct_fix_install.rs`
(a temporary file beside the file,
then a rename),
`worktree_completion.rs` (the same pattern),
and `bounded_push.rs`,
which
holds no file creation at all (the only lock it carries is the wait limit derivation above).
No `flock` or `fcntl` lock
exists in non-test code.

### Secrets in the service's environment (open for the coordinator)

A systemd user service does not inherit the terminal's environment,
and no secret is written into a unit file.
If a handler
needs a key,
for example `SHADOW_REVIEW_LLMGATEWAY_API_KEY`,
the clean ways provisioning could use are:
the user manager's
own environment block,
set through `SetEnvironment` on the manager at provisioning time from the wrapper's environment (held
in the manager's memory,
never on disk,
and visible to every unit of the account until the manager restarts);
or a
credential the manager passes with `LoadCredential`,
which needs the key in a file the account owns.
Neither is
implemented.
The first is one more call on the same bus connection,
with the cost that other units of the account can read
the value;
that exposure is the decision.

## Landing every command line (step 4d, part 1)

Every command line lands through the service.
The preparation (`service_prepare.rs`) never declines:
a selection by
`--include`,
`--patch`,
`--interactive` or `--pathspec-from-file` is made in the private index by real Git;
a conclusion,
a
detached head,
a branch that does not exist yet,
an unmerged index,
nothing to commit,
a policy rewrite and a change list the
embedded route cannot carry are built by real Git in the scratch repository (so its words and its exit status are the
person's) and landed as Git made them.
Only a machine with no user manager keeps the earlier path,
until part 2.

Landing rules of a commit Git built (`service_land.rs`),
protocol version 5:
- The first parent is the branch tip the client read (none for a branch that does not exist yet,
  compared against the
  all-zero name);
  later parents are a merge conclusion's.
  An amend has the replaced commit's parents.
- A detached head is the reference `HEAD`,
  updated with `option no-deref`.
  Measured:
  Git's no-deref update of a symbolic
  `HEAD` replaces the symbolic reference instead of failing the compare,
  so the service checks `symbolic-ref` first.
- The index entries are the paths that differ between the old branch and the new commit,
  read as bytes and handed to
  `reset --pathspec-from-file=- --pathspec-file-nul`,
  so names that are not text,
  gitlinks and lists past 4096 need no list.
- A conclusion's state (`MERGE_HEAD`,
  `MERGE_MSG`,
  `MERGE_MODE`,
  `SQUASH_MSG`,
  `AUTO_MERGE`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `MERGE_RR`,
  `sequencer/`) is removed by the landing exactly as native Git removed it in the scratch
  repository,
  each entry only while it still holds the bytes copied at preparation (`reproduce_conclusion_cleanup`),
  and the
  cleanup is journaled with the batch.
  Any commit made while such state exists is built by Git,
  so a squash message is
  removed too.
  A failed cleanup cannot be redone after the client removed its scratch directory.
- A rewrite a policy made is installed into the worktree by the client after the landing;
  a crash in between leaves the
  person's own content in the worktree and the rewrite in the commit and the index (shown as a modification).

Index lock,
first (found by the concurrent end-to-end scenarios;
ships in the 0.5.5 hotfix):
the landing creates
`index.lock` exclusively before it reads the branch,
holds it across the compare-and-swap,
builds the new index beside the
real one and installs it by renaming the lock over `index`,
as Git's own `commit` does.
A foreign holder is waited for on
an inotify watch of the Git directory (no polling;
the only timers are the limit `CLI_GIT_LOCK_WAIT_SECONDS` and the
ten-second repetition of the holder's line),
the holder's line from `index_lock_evidence.rs` reaches the client as a
`Waiting` message,
and a holder that outlives the limit ends the commit before anything moved with a plain message naming
the process (`Holder` message).
While the service holds the lock it carries `cli-git-service <batch>`,
so a restart removes
its own stale lock and no other.
Platforms without inotify fail at once when the lock is held.
While the service waits it
serves no other repository (one serialized step);
the limit bounds that.

A lock that carries the service's marker and is found by a landing is a dead instance's (one service runs and it holds no
lock between two landings),
so the landing removes it and takes its own;
measured:
without this rule a service killed while it
held the lock made every later landing wait out the whole limit for a holder that was gone.

Not covered:
a service killed between taking the lock and writing its marker leaves an empty `index.lock`,
which is a
foreign lock to every reader,
exactly as a killed Git leaves one.

### Suite with the service as the only route, after part 1

`CLI_GIT_TEST_SERVICE=1`,
whole binary target,
one run (436 tests,
221 s):
365 passed,
71 failed.
Three of the failures
(`wait_notice` for the index lock) were then restated for the service's wait and pass singly;
the other 68 remain.
Each
remaining failure asserts machinery that part 2 deletes:
- `recovery_tearing` (24):
  damaged or lost records of the transaction directory,
  owner locks,
  shadow refs,
  phases.
- `wait_notice` (5):
  the landing lock,
  the reservation and two real wrappers queueing behind each other.
- `commit` (4):
  replay after a lost race,
  a replay conflict,
  the reservation threshold,
  and the pack-per-landing migration that
  a maintenance control counts (the service writes loose objects,
  so nothing reaches the pack limit).
- `event_path_commit` (4):
  replayed commits and recovery of a killed transaction with names that are not text.
- `fault_enospc` (6),
  `fault_filesystem` (4),
  `fault_kill` (7),
  `fault_limits` (3),
  `fault_signal` (1),
  `fork_failure` (2):
  faults injected into the wrapper at each call of the earlier landing and then a check that its transaction directory
  recovers.
  The service needs its own equivalents where a fault still applies (the journal's kill points are in
  `binary_service_tests.rs`).
- `budget` (5):
  process and sync budgets of the earlier transaction;
  the service route needs its own.
- `merge_isolation` (1):
  a commit waiting for a running merge under the landing lock.
- `index_pack_fsync` (2):
  the migration `index-pack` of the earlier landing,
  which no longer exists.
  The service writes
  objects with the person's own `core.fsync`.
  The coordinator decided the service respects that setting,
  as it leaves
  `core.fsmonitor` as configured,
  so both controls (`only_the_migration_index_pack_gets_the_fsync_override` and
  `the_fsync_override_beats_a_users_core_fsync`,
  in `binary_index_pack_fsync_tests.rs`) were retired with the 0.5.5
  integration.
  The earlier landing's override stays in the code until the machinery part 2 deletes it,
  and has no control
  of its own in the meantime;
  the process and sync counts of `binary_budget_tests.rs` still cover its default configuration.

The push tests were restated,
not retired:
the push runs after the command returned,
in the service's environment.
The
harness waits for the work records to settle,
gives the fixture service the variables a control sets for the push,
and
reads a failed push from the notice.
Variables only the terminal has do not reach the push (documented limit).
`push_once` runs the same `run_push_command` path as the wrapper's push,
so the in-process pointer scan and the `git-lfs`
upload before the ref moves run in the service (shown by `lfs_pre_push` and `lfs_server` with the service).

Two end-to-end scenarios named by the coordinator:
`lint-staged-stash-concurrent` passes with the rebuild from the saved
message;
`reservation-after-lost-races` now lands every commit (exits 0) but asserts the earlier reservation counters,
so it
belongs with the machinery part 2 deletes.

### What part 2 can delete, and what it must keep

Delete:
`owner_lock*`,
`commit_landing_lock`,
`transaction_lease`,
`transaction_owner`,
`ownerless_directory`,
`replay_*`,
`landing_*`,
`commit_landing_loop`,
`post_landing`,
`landed_disposal`,
`commit_recovery`,
`recovery_*` (but see the Keep or move list),
`transaction_registry`,
`transaction_gate`,
`commit_workspace`,
`capture_store`,
`capture_records`,
the capture lock
and sequence of `capture_order.rs`,
`real_index_lock`,
`index_writer_coordination` and `index_writer_forward`,
the lock of
`auto_push_single_flight.rs`,
`holder_budget`,
`lock_holder_facts`,
the polling loop of `index_lock_wait*`,
`run_commit_transaction`
and `conclude_commit_transaction`,
and the earlier route in `commit_command.rs` once a machine without a user manager gets the
plain stop.

Keep or move (used by the service route):
`CaptureIndexes` and `indexed_request` (`capture_order.rs`);
`Selection`,
`decide_selection`,
`TransactionResult`,
`TransactionStop`,
`failed`,
`private_checks`,
`staged_at_base`,
`base_object`,
`settled_passes` and `CommitInvocation` (`commit_transaction.rs`);
`events_with_changed_summary` and `events_with_summary`
(`commit_conclusion.rs`);
the no-change constants and `no_change_message` (`commit_no_change.rs`);
`recovery_error.rs` and
`recovery_files.rs` (used by `conclusion_cleanup.rs` and `worktree_completion.rs`);
`conclusion_copy.rs`,
`conclusion_cleanup.rs`,
`shadow_create.rs`,
`shadow_config.rs`,
`shadow_links.rs`,
`shadow_refs.rs`,
`shadow_skeleton.rs`,
`shadow_git.rs`,
`commit_preparation.rs` (`run_native_commit`,
`read_prepared_commit`),
`commit_index.rs`,
`worktree_completion.rs`,
`commit_worktree_records.rs`,
the types of `transaction_journal.rs` (`Base`,
`Conclusion`,
`SymbolicHead`,
`TransactionMode`,
`RefFormat`,
`AddedPath`),
`PushGit` and `git_command` of `auto_push_single_flight.rs`,
`index_lock_holders*`,
`index_lock_evidence.rs`,
and `live_wait_limit` with `LIVE_WAIT_VARIABLE` of `live_lock.rs`.
This
list was read from the imports of the new files,
not from a deletion build;
the deletion itself is the check.

## Service-side fixes of 0.5.6

Found by the end-to-end port;
each has a control that failed before the fix.
- A client killed without warning left `.git/cli-git-service-<id>`.
  The service now owns the directory's life while the
  client's connection is open:
  after the first preparation the client sends a `scratch` message on a connection it keeps
  (close-on-exec,
  so no program it starts holds it),
  and the service removes the directory between batches when that
  connection closes,
  however it closed.
  The connection is the signal;
  there is no process id,
  no lock,
  and no age.
  A message
  names only `cli-git-service-<id>` directly inside the worktree's own Git directory.
  Limits:
  a first-ever commit has no
  service to hand the directory to (provisioning follows the preparation),
  a client killed during its own preparation
  (before the hand-over) leaves the directory as before,
  and a client killed after sending a conclusion's request can lose the
  scratch copies a landing still reads (the same limit as the earlier note on a failed cleanup).
- A killed or retired service left its work records `claimed` for the 30-minute lease.
  The first drain of an instance now
  returns every claim it finds to `pending` at once (this thread claims nothing before that point,
  and the manager runs one
  instance of a worktree at a time),
  costing the item one try.
  Superseded on 2026-11-01:
  a restarted instance runs beside
  the one it replaces,
  so no drain takes a claim before its lease is over (120 seconds,
  renewed by the holder);
  see "How a
  claim is held" in "After-commit work as built".
- A branch switch during a held commit landed on the old branch,
  rebuilt the index of the new one,
  and pushed the new one.
  The landing now refuses a commit whose branch the worktree's `HEAD` no longer names (read from the `HEAD` file of the
  worktree's own Git directory,
  from Git for a reftable repository),
  the client does not build the commit again on the branch
  it switched to and keeps the typed message in a file,
  and the service pushes the branch its work record names,
  never the
  symbolic `HEAD` (a worktree that left the branch gets a push that names both sides).
  Native Git fails such a commit with
  "cannot lock ref 'HEAD'" when the two tips differ and lands on the new branch when they are equal;
  the service refuses in
  both cases.
- A stale socket file broke a service run on `--path` (reproduced:
  "Address already in use").
  A run on a path now connects
  once:
  a refused connection means a dead service's file,
  which is removed and bound again;
  an answer means another service
  runs and the bind fails;
  a file that is not a socket is never touched.
  Production sockets belong to the manager and never
  take this path.
- The landing checked nothing about the prepared content before it moved the branch.
  It now finds a missing blob,
  tree or
  commit in process before importing anything (so `fast-import` leaves no crash report),
  answers a pruned request as
  unavailable (the client then prepares again once and says what happened),
  and checks a commit Git built with
  `rev-list --objects` because `update-ref` accepts a commit whose blob a prune took (measured,
  Git 2.56.0).
- `reservation-after-lost-races` asserts the landing reservation and the `race-lost` marker of the transaction route
  ("Suite with the service as the only route,
  after part 1" lists them for deletion in part 2).
  The property it guarded,
  that a commit that keeps losing races is not starved,
  holds in the service by the queue's arrival order and the bound on
  `RetryPrepare` (`LANDING_ATTEMPTS`);
  the scenario is retired with the machinery,
  not restated.

## Step 4d, part 2: the earlier landing is deleted

Decision (owner,
2026-11-01,
in a question):
"After per-worktree (Recommended)".
The step ran after the per-worktree service
change merged and gated (gate 21,
3349 of 3349).
Condition,
recorded the same way:
every commit command line the transaction
route still handled works through the service's compare-and-swap,
proven field by field against native Git before the old
landing went.
Part 1 moved every declined line;
the twin tests in `binary_service_landing_selection_tests.rs`,
`binary_service_selection_tests.rs`,
`binary_service_amend_tests.rs`,
`binary_service_options_tests.rs` and
`binary_service_native_tests.rs` now compare with real Git directly (`direct`),
no longer with the earlier route,
which no
longer exists.

What the route does now.
A commit that cannot reach or start a service stops with exit 1,
the plain message of its
platform (`ClientFailure::NoActivation`:
Linux without user services names `loginctl enable-linger`) and changes nothing,
before any scratch directory is made;
a commit outside a repository ends with Git's own words and status 128.
The
controls are in `binary_service_no_manager_tests.rs`,
each shown to fail when its guard is removed.

Deleted:
the transaction's landing,
recovery and registry (`landing_*`,
`post_landing`,
`landed_disposal`,
`commit_landing_lock`,
`commit_landing_loop`,
`commit_revalidation`,
`commit_workspace`,
`commit_replay_options`,
`recovery_*` but `recovery_error` and `recovery_files`,
`transaction_registry`,
`transaction_lease`,
`transaction_gate`,
`commit_recovery`,
`ownerless_directory`,
`real_index_lock`,
`journal_record`,
`capture_store`,
`capture_records`,
`capture_order` but its two kept items),
the replay machinery the service does not use (`replay_commit`,
`replay_reservation`,
`replay_revalidation`,
`replay_capture_history`,
`replay_object`,
the dead parts of `replay_merge` and
`replay_capture_order`),
`index_writer_coordination` (the landing lock,
the waits for a foreign `index.lock`,
the lease),
the polling wait (`index_lock_wait_host` and the loop of `index_lock_wait`),
`transaction_journal_encode` and
`transaction_journal_parse`,
`content_digest`,
`filesystem_identity*`,
`pack_keep`,
the five landing events of the
JSONL schema (`landing-race-lost`,
`landing-reserved`,
`commit-replayed`,
`replay-headers-dropped`,
and the `ReplayConflict`
variant) and the engine failure `index-lock-unproven-owner`,
the transaction half of `commit_transaction`,
`commit_conclusion`,
`commit_no_change`,
`commit_preparation`,
`transaction_journal`,
`recovery_files`.

Kept,
and why.
`index_writer_forward` keeps the wrapper alive around a forwarded index writer,
without a landing lock:
it
waits for the service's hold,
completes an isolated merge (`merge_isolation*`,
a user-visible feature that no decision
retires),
and runs the writer again when the service took the index first (`service_hold_retry`).
Isolated-merge recovery
runs at the start of a command (`startup_gate`) without a lock;
a merge's intent record still names an owner process
(`transaction_owner`,
`pid_namespace`,
`process_identity`),
so those stay.
`process_use` stays for the worktree copy.
(Note,
2026-11-02:
`process_use.rs` was deleted on 2026-11-01 in `757227c6cb` because nothing called it,
so it did not stay for the worktree copy;
the
subvolume migration (`subvolume_migrate.rs`) calls its `/proc` scan to refuse moving a tree in use,
and `864eca4852` restored it for
that caller.)
The
foreign `index.lock` evidence (`index_lock_evidence`,
`index_lock_holders`,
`index_lock_holder_parsers`) and the notice
cadence (`wait_progress`) are what the service's landing uses;
its wait limit and the variable
`CLI_GIT_LOCK_WAIT_SECONDS` moved to `wait_limit.rs`.
`CLI_GIT_WAIT_NOTICE_SECONDS` stays:
`service_index_lock.rs` reads it.
The Git backoff schedule of `index_lock_wait` stays for `worktree_copy_lock`.

Not deleted in this step,
each blocked by a file another agent is editing or by work that has not merged:
- The push lock and its family (`auto_push_single_flight`'s lock,
  `holder_budget`,
  `lock_holder_facts`,
  `owner_lock`,
  `owner_lock_record`,
  `LOCK_WAIT_VARIABLE` and `CLI_GIT_PUSH_LOCK_WAIT_SECONDS` of `bounded_push`):
  `auto_push.rs` still
  imports `resolve_push_coordination_paths`,
  `run_single_flight_push_repeating`,
  `holder_facts` and `read_owner_lock_record`
  for its inline `auto_push` function,
  which nothing calls any more.
  Removing that function from `auto_push.rs` frees the
  rest.
  (Note,
  2026-11-02:
  deleted on 2026-11-01 in `69d275d92b`.
  `auto_push.rs` lost the inline `auto_push` entry,
  and with it the
  per-branch push lock and its last-pushed record,
  `owner_lock`,
  `lock_holder_facts`,
  `holder_budget` and
  `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`.
  The commit service is the only pusher:
  pushes are service work items coalesced per branch
  and run by `push_once`,
  so no waiter derives a limit from a push holder.
  `owner_lock_record` stays,
  without its unused
  reader,
  for `worktree_copy_lock`;
  a wait on a foreign `index.lock` takes its limit from `CLI_GIT_LOCK_WAIT_SECONDS`
  (`wait_limit.rs`).)
- `worktree_copy_lock` and the dead `CopyFacts.global_prefix` and `environment`:
  the settlement lock is still in this tree
  (the lock-free copy lives on an unmerged branch),
  and its inherited-lease check reads those two fields.

No retry of an old registry is made:
a command that finds a transaction directory in `.git/cli-git-transactions` names the
directory once and goes on (`startup_gate.rs`);
it removes nothing,
because a transaction of an earlier release that
was interrupted after the branch moved is evidence the person may need.

The suite with the service as the only route,
after part 2.
The fixture service is now the default (`CLI_GIT_TEST_SERVICE`
is gone).
The controls that asserted the deleted machinery are retired with it:
the recovery tearing controls,
the
landing and landing-crash controls,
the fault sweeps of the earlier landing (`strace --inject` at every call),
the
fork-failure sweeps,
the landing and reservation notices,
the replay and reservation controls of the transaction,
the
index-writer coordination waits,
the phase-marker replay controls of non-UTF-8 names (restated for the service in
`binary_service_replay_tests.rs`),
the migration `index-pack` budgets,
and the maintenance-after-landing control (the
service writes loose objects,
so nothing reaches `gc.autoPackLimit`).
The budgets of `binary_budget_tests.rs` are restated
for the wrapper's side of a commit.
The push controls read the failure from the notice the next command shows.

## One executable (2026-11-04T16:30:00-05:00)

Owner requirement,
verbatim (2026-11-04):
"Bazel doesn't need to distribute as 2 bins."
The crate declared two `[[bin]]`
targets from 2026-11-01 (`e9eaf63d14`):
`cli-git-native` and `cli-git-service`.
The release job builds and archives only the
first,
and cargo-binstall requires every `[[bin]]` of a crate to be in the archive,
so the `catalog-specifiers` workflow
(`--disable-strategies compile`) has failed since 2026-11-01T14:34:19Z with `When resolving monochromatic-git-policy-cli bin
cli-git-service is not found. This binary is not optional`.
Machines that compile from source did not see it.

What changed.
The service is a mode of `cli-git-native`,
selected by the three leading arguments `cli-git service run`
(`src/native/service/service_main.rs`,
dispatched first in `main.rs`,
before the wrapper's own setup).
The options are the
ones the standalone program took (`--journal`,
`--git`,
`--instance`,
`--abstract`,
`--path`,
`--idle-seconds`,
`--release`,
`--lock-ceiling-millis`).
The generated `cli-git@.service` unit starts `<wrapper's own executable> cli-git service run
--journal ... --git ... --instance %i` and sets `SyslogIdentifier=cli-git-service`.
The route names the wrapper's own path
(`current_exe`,
identification only) in the unit instead of a sibling file.

Choices open to the owner's veto (the coordinator's brief called the mode form a suggestion):
- The form `cli-git service run`,
  inside the wrapper's existing `cli-git` namespace and apart from `status`.
  Not `--service`
  or a name-based dispatch (the self-invocation rules ban dispatch on `argv[0]`).
- The mode words count only first.
  Behind a global option of Git (`git -C x cli-git service run`) they are an unknown
  management command,
  refused with the usage.
- The unit keeps the journal identifier `cli-git-service`:
  `journald_native.rs` sends `SYSLOG_IDENTIFIER=cli-git-service`
  natively,
  independent of the executable's name,
  and the unit's `SyslogIdentifier` covers the lines the manager captures from
  standard error.
- On-disk and wire names are unchanged because earlier releases read and write them:
  the scratch reference
  `refs/worktree/cli-git-service/<seq>`,
  the index-lock marker `cli-git-service v2 ...` (`service_index_marker.rs`),
  the
  client scratch directory `cli-git-service-<id>`,
  the log prefix `cli-git-service: `,
  and the journal identifier.
- The unit still names the versioned toolchain path.
  Naming `.../current/bin/cli-git-native` would shorten the window in which
  a pruned path can be activated,
  but it is a different design and is not done here.

Invariants,
each with a control:
- The wrapper never starts the service (the manager does).
  The process setup of the service (`service_signal::install`,
  `mark_service_process`,
  the mode words) is referenced by the service mode and the dispatch only,
  and the three setup calls
  come first in `run` (`service_main_tests.rs`).
  `hooks_off::activate` also runs in the wrapper's own start (`entry.rs`),
  so
  it is the one setup the two modes share.
- A start with no socket from the manager and no `--path` or `--abstract` is refused before anything is read or created
  (exit 3,
  no directory,
  no listener):
  `binary_service_mode_tests.rs`,
  `a_start_with_no_socket_and_no_address_is_refused_and_creates_nothing`.
- A `--git` that is the wrapper (the built file,
  a hard link,
  a symbolic link,
  a byte copy,
  or a launcher) is refused (exit 2):
  `a_git_that_is_the_wrapper_is_refused`.
  The service uses the wrapper's candidate classification,
  so the executable's own file
  is now a case,
  where the standalone program checked launchers only.
- Nothing identifies the service by its process name or executable path.
  The one process-name test in the crate is the stale
  reference-lock judge's `git` prefix on `/proc/<pid>/comm`;
  `cli-git-native` is not Git,
  as `cli-git-service` was not
  (`a_process_named_like_the_one_executable_is_not_git`).
  `open_by=[42 (cli-git-native)]` in a wait note is display.

Upgrade from a two-program release (0.5.34 and earlier).
The unit a 0.5.34 wrapper wrote names
`<toolchain>/0.5.34/bin/cli-git-service`,
a file the install of the next release removes.
The first commit that goes to the
service already compares the installed units with what the wrapper would write (`in_place`),
so the new unit text differs and
the commit rewrites the units,
reloads the manager,
clears failed state and starts the socket before it connects.
A service of
the older release that still runs is sent `retire` by the release mismatch in the welcome;
it finishes its batch and exits with
success,
and the next connection starts the program the rewritten unit names.
Controls:
`binary_service_upgrade_one_executable_tests.rs` (a manager stand-in that owns the socket,
an old service on it,
two commits at
once;
the unit is rewritten before the socket start,
the old service exits with success,
the successor is started from the
rewritten unit's own command line with the socket passed in as the manager passes it,
both commits land) and the same against
the real 0.5.34 `cli-git-service` program,
which needs a build of another version than 0.5.34 (a service of the same release
is not asked to retire).
Run once on this branch with a temporary version 0.5.35 build:
passed.
Not repairable by the wrapper:
a service of the old release that exits after the install removed its program and before the first commit of the new release
is activated by a connection that arrives first;
only a connection from the new wrapper arrives in practice,
and it
rewrites the unit first.
`git cli-git service status` connects without provisioning;
under an unrewritten unit it can trigger
the failed activation,
which the next commit's `reset-failed` clears.

Release plumbing.
The release job keeps `--bin cli-git-native`.
`source_guard/release_bins_tests.rs` (in the gate) fails when
`Cargo.toml` declares any other `[[bin]]`,
and `bin/native-release-bins.unit.test.ts` fails when the release job,
the
release list `releasable-crates.json` and the lock `toolchain.lock.json` differ from `Cargo.toml`.
