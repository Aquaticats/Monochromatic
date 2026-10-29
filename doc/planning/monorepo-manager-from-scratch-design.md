# From-scratch monorepo manager design

## Status

- Status:
   accepted route,
   design in progress.
  On 2026-09-16 the stack narrowed to one remaining route,
   an all-Rust tool with file-enforcer rewritten in Rust,
   and the user accepted it the same day:
   [`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md).
  Three more decision records followed on 2026-09-17:
   the cache key hash,
   the HCL front end,
   and the per-user configuration.
  The build matrix,
   output format,
   command surface,
   and configuration authoring settled the same day.
  The software-development model is open:
   the user rejected treating development stages as special tasks.
  The [software model exploration](monorepo-manager-software-model.md)
   records the user's continuous-maintenance direction:
   keep building,
   correctness checks,
   and publication-related work current.
  The active discussion remains the non-task model of package maintenance.
  Publication is explicitly deferred to 1.x,
   with push to `main` recorded as its trigger boundary;
   no publication question blocks 0.x.
  Configuration schema and ordering remain open,
   rather than following the withdrawn task-name questionnaire.
- Implementation plan:
   [`monorepo-manager-implementation-plan.md`](monorepo-manager-implementation-plan.md).
  The user accepted resource-aware package maintenance and requested continued grilling.
  Session deliverable:
   a plan concrete enough to implement 0.x.
  No code exists and none is authorized yet.
- Session state,
   commits,
   and next action:
   [`doc/handover/monorepo-manager.md`](../handover/monorepo-manager.md).
- Route comparison:
   [`monorepo-manager-build-routes.md`](monorepo-manager-build-routes.md).
- Requirements checklist:
   [`tech-monorepo-manager-vet-2026-09-16.md`](../audit/tech-monorepo-manager-vet-2026-09-16.md),
   sections "Requirement checklist" and "Frozen hard constraints".
- Scope:
   design only;
   the route is chosen,
   but implementation has not been requested.
- Version scope,
   stated by the user on 2026-09-17:
   "Every decision we made and are going to make in this session applies to v0.x
   and won't necessarily apply to v1.x and so on."
  Every requirement,
   decision,
   and answer in this document and in the decision records it links is scoped to meow 0.x;
   1.x and later may revisit any of them.
  Later explicit exception:
   the user deferred the entire publishing subsystem to 1.x during the non-task-model discussion.
  Publication statements record that future direction,
   not 0.x requirements.

## User-stated design requirements

Stated by the user on 2026-09-16.

### Process model

- The user runs one long-running process in its own terminal.
- Clients control that process over RPC.
- For now the process prints logs and reacts only to Ctrl+C;
   a TUI comes later.

### Default behavior

- By default the process watches the entire repository.
- On every change it builds everything affected
   and runs the default set of tests for everything affected.
- Caching is on by default,
   and at this stage it cannot be turned off.
- On filesystems that support it,
   the process uses filesystem features to speed up change comparison.
  For 0.x,
   btrfs is the only supported filesystem for this,
   per the user's correction on 2026-09-16 that dropped ZFS.
- The process may create btrfs subvolumes and snapshots whenever it wishes.

### Automatic publication

Deferred to 1.x by the user:
"The publishing subsystem won't exist until 1.x".
The agent's publication-policy detour did not resolve the active non-task-model question.
This section records future direction only;
publication adds no 0.x implementation work or blocker.

Accepted during the software-model clarification:
"Publication is automatic too.
On version bump + target registry already has it".
The user confirmed that "it" means the package,
not the newly bumped version:

- A package already published to the target registry receives automatic publication of a newly bumped version.
- If that version is already present,
  there is nothing to publish.
- A package never published to that registry does not receive automatic first publication.
- No separate release request is required for an eligible update.

The user chose push to `main` as the publication trigger boundary.
Further publication design waits for 1.x;
do not pursue its checks,
registry protocol,
or release machinery during the current model discussion.
These answers do not authorize publishing anything during this session.

### Doctor command

- The tool provides a `doctor` command.
- `doctor` tells the user which commands to run or which files to edit,
   and why,
   to enable capabilities the environment currently blocks.
- The user's example:
   a btrfs mount lacking `user_subvol_rm_allowed`,
   which `doctor` reports with the exact change and its reason.

### Scheduling and control

- Every task given to the process has a priority,
   defaulting to 0 when unspecified.
- Concurrency defaults to the machine's available parallelism,
   with an environment variable override.
- A client can change a task's priority,
   pause a task,
   end a task,
   and perform similar task controls.

### Distribution

- Hard requirement:
   the tool ships as a single file that a user runs directly,
   as in `./meow`
   (`meow` is a placeholder name).
- Unpacking at run time is acceptable,
   as AppImage does.
- A single file that carries Node is too big,
   per the user's decision recorded in "Single-file shipping and a file-enforcer rewrite".
- `./meow` is meant for repositories other than Monochromatic too
   (user answer,
   2026-09-16).

### Platforms and builds

Stated by the user on 2026-09-16:

- Supported in 0.x:
   Linux on x86_64 and aarch64
   ("Sorry we need to support ARM too").
- Each supported target is built both glibc-linked and as a static musl binary:
   "build both glibc and static musl.
   Musl has known performance problems."
- "Not supporting other archs/OSs doesn't mean to never build paths for them;
   it only means an issue in these paths don't block publishing."
  So "Linux only" names the release-blocking tier,
   not a ban on code paths or CI for other systems.
- "Meow takes over only when it supports the full matrix.
   Meow 0.x doesn't need to guarantee good support for platforms other than Linux."
   (user,
   2026-09-17).
  Until then Mise keeps running the macOS and Windows jobs in `readonly-semantic-bridge.yml`;
   the user noted these two statements do not conflict.
- "We're planning to ship as many builds as possible.
   Shipping x86-64-v3 and x86-64-v4 as separate builds is fine."
   (user,
   2026-09-17).
  A build raised above its target's baseline needs the startup check in "Missing CPU capabilities",
   because feature detection macros evaluate to `true` at compile time for enabled target features.
- Asked which x86-64 levels block publishing,
   the user answered "v4 block only" on 2026-09-17:
   among x86-64 builds only `x86-64-v4` is release-blocking,
   while the baseline,
   `x86-64-v2`,
   and `x86-64-v3` builds still ship without blocking.
  aarch64 stays release-blocking;
   the question covered x86-64 levels only.

### Output format

Stated by the user on 2026-09-17,
answering which diagnostic renderer meow should use:
"There is no need.
 Emit every line as a json,
 error or warning or not."

- Every line meow writes to standard output or standard error is a JSON object:
   logs,
   progress,
   task output framing,
   warnings,
   and errors alike.
- meow ships no human-facing renderer and takes no diagnostic-rendering dependency,
   so `codespan-reporting`,
   `annotate-snippets`,
   `ariadne`,
   and `miette` are all out.
- Rules `DGT`,
   `DNL`,
   and `WRN` still govern what a diagnostic says:
   the JSON object carries the affected input,
   the remediation paths,
   and neutral wording,
   with source spans as fields rather than as drawn carets.
- Clients,
   including the later TUI,
   render for humans from these objects.
- Exception,
   stated by the user on 2026-09-17 ("User interface"):
   `meow run` forwards a task's own standard output and standard error as-is,
   byte for byte,
   to the terminal that asked for the task.
  That path carries the task's bytes,
   not meow's messages,
   so it is not JSON and is not framed.

### User interface

Stated by the user on 2026-09-17,
after four proposed output models were all rejected:
the surfaces are split by audience,
and neither one renders the other's content.

#### The watch terminal

```sh
$ meow watch
{...}
{...}
```

- The user runs `meow watch` in a second terminal and keeps it open but minimized.
- Its output does not need to be human-readable:
   it is the JSON stream from "Output format",
   and nobody is expected to read it while working.
- This is the daemon:
   the watcher,
   the scheduler,
   and the cache all live behind it.

#### The working terminal

```sh
meow run //package/cow:test
```

- The user works in their own terminal and asks for a task by target.
- Not cached:
   meow queues the task at interactive priority,
   holds the terminal until the task has run,
   and forwards the task's own standard output and standard error as-is.
  While it waits for a slot,
   it shows the queue position and what is ahead,
   updated as that changes (user,
   2026-09-17),
   so a wait is never mistaken for a hang.
- Failure adds nothing:
   meow exits with the task's own exit code and prints none of its own text
   (user,
   2026-09-17).
  The task has already explained itself,
   and its exit code is what a script reads.
- For a foreground request attached to automatic maintenance,
   Ctrl+C detaches that request and returns the terminal;
   the maintenance and other waiters continue (Q7).
  This supersedes the earlier kill-on-Ctrl+C contract for maintained work.
  Attachment does not turn automatic maintenance into a one-off operation.
  The earlier process-tree kill contract remains for explicit non-automatic executions.
- Already running as background work:
   `meow run` attaches to that run rather than starting a second one (user,
   2026-09-17).
  It raises that task's priority,
   replays the output already produced from the cache's stored log,
   streams the rest,
   and exits with its code.
- Unbuilt dependencies do not arise.
  Asked what `meow run` does when a dependency is not built,
   the user answered that this should never happen:
   `meow watch` is expected to always keep everything up to date,
   so dependency work has already run,
   is running,
   or is queued.
  `meow run` therefore waits on that work rather than discovering it,
   and raises its priority along with the target's,
   since the newest task a person waits on holds the highest priority.
- Cached:
   meow does not execute the task,
   and prints the result of the last run instead,
   which the cache already stores for replay ("Cache").
  One line marks the replay,
   naming that the result is cached and when the original ran,
   so an instant return is never unexplained (user,
   2026-09-17).
- "Interactive" is the user's stand-in word,
   borrowed from Windows Task Manager;
   the implementation uses an integer priority like every other task (user,
   2026-09-17).
- The newest task a person is waiting on always holds the highest priority,
   so a later `meow run` outranks an earlier one,
   and other tasks may be paused to give it the machine.

#### When the daemon is not running

- If `meow run` finds no live `meow watch`,
   it prompts,
   and waits for the user to choose (user,
   2026-09-17):
  - spawn a terminal running `meow watch`,
     or
  - start a daemon detached in the background.
- Ctrl+C at that prompt leaves nothing started.
- `meow run` never starts a daemon without being told to at that prompt.

#### The commands

Answered by the user on 2026-09-17:
inspection is "A `meow status` command",
task controls are "CLI commands now",
and "We don't need specifically a socket or a separate client,
 ever."

- `meow watch`:
   the daemon,
   in its own minimized terminal.
- `meow run <target>`:
   the working terminal's blocking run.
  "Follow Bazel" (user,
   2026-09-17),
   read against Bazel's user manual on the same day:
   `bazel run` "is used to build and run a single target",
   while `bazel build` and `bazel test` take lists.
  So `meow run` takes exactly one target,
   for the same reason:
   it forwards one task's standard output,
   standard error,
   and exit code.
  Arguments after `--` go to the task,
   as Bazel does it.
  Target syntax follows Bazel's patterns:
   `//package/cow:test`,
   `//package/cow` for the same-named target,
   `//package/...` for a tree,
   `:all` and `:*`,
   relative forms,
   and `-` to negate.
  Asked whether 0.x needs a list-taking command,
   the user answered "No list command,
   but we accept glob patterns" (2026-09-17).
  So `meow run` stays the only way to ask for work,
   and its single argument may be a glob that matches more than one target.
  Answered the same day:
  - Syntax is shell-style globs,
     `*` and `**` as they behave elsewhere in this repository,
     not Bazel's `...` and `:all`.
  - meow warns properly about quoting,
     because an unquoted glob is expanded by the shell before meow sees it.
    The tell is more than one argument arriving where one target belongs,
     or an argument naming an existing path;
     the diagnostic names the quoting fix.
  - Every matched task runs concurrently,
     and each forwarded line carries its target as a prefix.
    Byte-exact forwarding therefore holds for a single target,
     which is the common case,
     and not for a multi-match run.
  - `meow run` waits for every matched task,
     then exits with the code of the one that failed first.
- `meow status`:
   shows only what needs attention in 0.x.
  Asked to choose a full status layout on 2026-09-17,
   the user declined:
   "I haven't decided yet.
   I think there is an extremely good opportunity for good UI/UX here.
   Your 'grouped by state' proposal is a step in the right direction but it's not good enough.
   I think we might want to use a TUI here.
   But since TUI is deferred,
   let it just only show what needs attention for now."
  So the full inspection view is the TUI's design problem,
   not a command layout to settle now,
   and 0.x prints the attention list instead.
  Its content,
   given by the user the same day:
   failed,
   paused,
   blocked,
   flaky retries,
   in queue,
   and running.
  `meow status` never shows successes (user,
   2026-09-17),
   so finished-and-passed work is absent from that list,
   and the layout stays deliberately unchosen.
- `meow pause`,
   `meow resume`,
   `meow end`,
   and `meow priority`:
   the task controls,
   reaching the daemon the same way `meow run` does.
  `meow end` is later restricted to non-automatic work in 0.x:
   the user forbids ending automatic maintenance rather than choosing a restart policy.
  They name a task by its target label (user,
   2026-09-17),
   the same label `meow run` takes,
   so there is one naming scheme.
  Earlier wording called the same target running twice a bug.
  The user's Q10 correction rejects interpreting that as a mutex on the target label:
   distinct invocations enter the ordinary priority queue and may run concurrently,
   be paused,
   or remain queued under that scheduler.
  Equivalent reusable computations still share existing work;
   a common label alone does not establish equivalence.
  Control selection when a label has distinct invocations remains to be specified.
  `meow priority` takes either an absolute integer or a signed bump (user,
   2026-09-17);
   the parser must keep `-1` from reading as a flag.
- `meow doctor`:
   prints problems only (user,
   2026-09-17),
   each naming what is wrong,
   the exact change,
   and why it matters.
  A healthy machine prints nothing,
   so any output is worth reading.
- `meow stop`:
   stops the daemon from any terminal,
   alongside Ctrl+C in the watch terminal (user,
   2026-09-17),
   because the watch terminal is the one a person keeps minimized.
  Either way,
   running tasks are ended through their cgroups before the daemon exits,
   which is what Ctrl+C already means under `meow run`,
   and leaves no task outliving the daemon that started it.
- `meow lsp` and the trust commands from "Per-user configuration".
  Trust state reads through `meow trust status`,
   keeping bare `meow status` about tasks.
- meow ships no separate client application,
   and the transport behind these commands is private ("RPC").
- The TUI,
   when it arrives,
   is "just `meow watch` displayed differently while adding some interactivity"
   (user,
   2026-09-17):
   the same command and the same process,
   rendering its own stream for a person and accepting control there,
   rather than a second program.
  The JSONL sink keeps recording either way.

#### Trusting a repository

Answered by the user on 2026-09-17:
"Prompt inline with the full contents of the .hcl file.
 Like how AUR helpers do it."

- Running a command that loads an untrusted configuration shows that configuration in full,
   then asks whether to trust it,
   in the same terminal.
- The model is an AUR helper's `PKGBUILD` review:
   the person reads what would run before anything runs.
- Declining,
   or Ctrl+C,
   leaves the configuration untrusted and runs nothing.
- Once trusted,
   the cli-git rules from "Per-user configuration" take over:
   exact bytes,
   a snapshot evaluated in place of the live file,
   and a later byte change blocking until re-trust.
- The review covers every file that would be evaluated (user,
   2026-09-17),
   each under its path,
   not only the root `meow.hcl`,
   because a one-line import would otherwise bring in code nobody read.
- The review does not page (user,
   2026-09-17):
   it prints every file and then the prompt,
   composing with whatever the output is piped into,
   rather than handing a security decision to an arbitrary `$PAGER`.

#### Where the JSON stream goes

Asked where a detached daemon's stream should go,
the user pointed at the repository's own logger:
"In ts,
we use our module/logger which logs to every available sink simultaneously."
Recommended,
not yet accepted:
meow writes a small Rust equivalent rather than choosing one destination.

- Read from `@monochromatic-dev/module-logger` on 2026-09-17:
   each sink carries a `verify` that probes its backend,
   the file sink writing a record and reading it back before it counts as available
   (`package/module/logger/src/sink/file.ts:219-227`);
   the logger keeps every sink that verifies and fans each record out to all of them
   (`src/default-sinks.node.ts`);
   and a logger whose sinks all fail verification raises "No logging backends available"
   rather than discarding silently.
- meow mirrors that shape:
   verify,
   fan out,
   and fail loudly when nothing verifies.
- Sinks for 0.x:
   the terminal stream that `meow watch` prints,
   and a timestamped JSONL file in meow's state directory,
   one per daemon start,
   as the TypeScript file sink already does under `node_modules/.monochromatic/`.
- This answers the detached case without a separate rule:
   a detached daemon has no terminal,
   the file sink still verifies,
   so the stream is recorded either way.
- Rule `RCI`:
   no Rust package in this repository owns logging yet,
   so meow owns it,
   and the tag discipline rule `TLG` states for TypeScript carries over.

#### Which terminal speaks which language

Derived from the answers above,
not separately stated:

- The `meow watch` stream is JSON,
   as "Output format" requires.
- The working terminal is for a person:
   the replay marker,
   the daemon prompt,
   and `meow run`'s own errors are human-readable there,
   and the task's bytes pass through untouched.

#### What this settles

- The JSON rule covers meow's own messages,
   not a task's bytes:
   `meow run` is a pass-through for those.
- meow needs no human-readable renderer for the watch stream,
   which is why no diagnostic renderer crate was taken.
- The later TUI is another client of the same socket,
   not a replacement for either terminal.

### Modularity

Stated by the user on 2026-09-17,
answering whether meow should mirror `@monochromatic-dev/module-logger` inside its own binary
or port it as a package:
"Everything that could possibly be modular should be reusable package/library.
 This includes not just the ported logger."

- meow is composed of reusable repository packages,
   not one crate with private modules.
  Anything that could stand alone becomes its own package:
   the logger,
   the HCL front end and evaluator,
   the hasher wrapper,
   the cache,
   the watcher,
   the scheduler,
   the RPC layer,
   the structured editors,
   and the file-enforcement rules.
- The single-file requirement is unaffected:
   `./meow` is one binary that composes those packages
   ("Distribution").
- This supersedes the implementation plan's line that package layout is undecided,
   and it makes the plan's milestones deliver packages rather than modules.
- Open tension,
   flagged for the user:
   on 2026-09-17 the same day,
   answering the vm-builder question,
   the user chose "No shared owner" for TypeScript process execution,
   which points the other way for that case.

### Configuration

Stated by the user on 2026-09-16,
answering whether configuration edits may need the Rust toolchain:
"What are we doing here,
 now that we're determined to take on this huge project,
that implies we have no objections of writing much more code
therefore eliminate the turing-complete requirement of the config language.
File-enforcer was written under the constraints of time."

- The configuration language does not need to be Turing-complete.
- Logic in today's `file-enforcer.config.ts` moves into code;
   the TypeScript configuration's shape reflects time constraints,
   not requirements.
- The user later pointed to OpenTofu as a precedent:
   "I think opentofu does config files properly."
- After the format brief
   (ranking and pros and cons from "Format options and ranking"),
   the user answered "Yes to HCL" on 2026-09-17.

### Hashing

Stated by the user on 2026-09-16:

- "We don't need a crypto hash.
   A non-crypto hash would do."
- Clarified by the user on 2026-09-17:
   "non-crypto isn't a requirement.
   I only said we don't necessarily need a crypto hash."
  Cryptographic hashes are eligible for cache keys.
- Asked how much crafted-collision resistance weighs against throughput,
   the user chose "Speed first" on 2026-09-17:
   cryptographic hashes compete under the vet's existing weights,
   with whole-file throughput at weight 5.
  The reasoning presented was that the cache is local and rebuildable,
   a repository able to craft collisions already runs its own tasks,
   and changing the hash later costs one cache rebuild.
- "We already chose a non-crypto hash fn in music-player":
   `gxhash`,
   which "benched the best";
   the benchmark was not recorded at the time.
- "We don't need to support non-modern CPUs,
   but we do need to properly warn users when they try to use this w/o the required CPU capacities."
- Asked how aarch64 tests should avoid the `gxhash` debug-build panic (issue #111):
   "Dirty room (crediting ogxd) reimplement gxhash ourselves.
   Or forking it.
   Also,
   there are many optimization opportunities the original seemingly didn't have time to take,
   so we are doing it."
  `gxhash` 3.5.0 is MIT-licensed,
   by Olivier Giniaux,
   with 979 lines under `src/` and 58 `unsafe` occurrences
   (measured in the cached crate).

Design detail is in "Cache".

### Managed file edits

Answered by the user on 2026-09-16:

- Generated files may change once in a reviewed commit when the Rust tool takes over;
   today's exact bytes are not required.
- "but we must still handle comment-preserving jsonc editing and toml editing properly."
- "XML comments must also be properly preserved."
- A malformed managed XML file fails with a diagnostic instead of a best-effort splice.

### Installs for probes

Authorized by the user on 2026-09-16:
"You may install anything and everything except invoking rpm-ostree install."

## Measured environment

Measured 2026-09-16 on the development machine:

- The repository is on btrfs:
   `findmnt` reports subvolume `/home` mounted `rw,relatime,seclabel,ssd,discard=async,space_cache=v2`,
   without `user_subvol_rm_allowed`.
- The repository checkout itself is not a subvolume:
   `btrfs subvolume show /var/home/user/Monochromatic` reports "Not a Btrfs subvolume".
- Node `v26.8.2` exposes `os.availableParallelism()` and `navigator.hardwareConcurrency`;
   both return 16.
  Node has no API named `availableConcurrency`,
   the term the user used.
  With the all-Rust route,
   the concurrency source is `std::thread::available_parallelism`,
   which returned `Ok(16)` under rustc `1.100.0-nightly` (0fc141305 2026-09-11)
   (`parallelism-probe/main.rs` in the session scratchpad).
- 104 packages define `test:unit` and 23 define `test`;
   rarer test tasks include `test:container`,
   `test:wayland`,
   `test:mutation`,
   `test:integration`,
   `test:conformance`,
   network resolver tests,
   and instrumented device tests.

### Answers on 2026-09-16

- Default test set:
   every test suite except heavy suites.
- Pausing:
   holds queued tasks and freezes running tasks.
- Platforms:
   only Linux support is required for 0.x.
- First-version controls besides changing priority,
   pausing,
   and ending:
   resuming a paused task,
   listing tasks and their state,
   and queueing or rerunning a task by hand.
  Changing concurrency at runtime is not in the first version.

### Further answers on 2026-09-16

- Platforms:
   everything in 0.x is Linux only.
- Sandboxing is a must,
   using at least cgroups.
  The kernel cgroup v2 documentation lists CPU,
   memory,
   IO,
   PID,
   and cpuset controllers and a `cgroup.freeze` interface that stops every process in a cgroup and its descendants
   (<https://docs.kernel.org/admin-guide/cgroup-v2.html>);
   it provides no filesystem access control.
  Cgroups therefore cover resource limits and pausing,
   while restricting undeclared file reads needs a separate mechanism or the lint-level enforcement recorded in
   [`monorepo-manager-build-routes.md`](monorepo-manager-build-routes.md).
- Heavy suites are test files whose names match `*.expensive.*.test.*`.
  Four files match today:
  `package/dev-script/task-util/src/tsc-filter.expensive.unit.test.ts`,
  `package/module/image-diff/src/client.expensive.unit.test.ts`,
  `package/vmsync/cli/src/lifecycle.expensive.unit.test.ts`,
  and `package/mvm/cli/src/backend/hetzner/provision.expensive.unit.test.ts`.
  Suites that are heavy today only by task name,
   such as `test:container` or `test:wayland`,
   are not excluded by this rule unless their files adopt the naming.
  The user considers expensive tests without the naming a user error:
   the daemon runs them by default and does not add a fallback classification.
- The user delegated the concurrency override name.
  Chosen:
   `MONOCHROMATIC_JOBS`.
  The prefix follows the repository's existing environment variables,
   such as `MONOCHROMATIC_VERBOSE` and `MONOCHROMATIC_WARN`;
   `JOBS` follows Bazel's `--jobs` option
   (`src/main/java/com/google/devtools/build/lib/buildtool/BuildRequestOptions.java` in the Bazel source)
   and the `--jobs` convention of Make-style build tools.
  If the tool receives its own name,
   the prefix follows that name;
   since `./meow` also serves other repositories,
   a repository-named prefix no longer fits.

### Sandboxing scope answer

For 0.x,
sandboxing is cgroups only,
without restricting file reads,
per the user on 2026-09-16.
Undeclared reads stay covered by lint-level enforcement;
read restriction is left for a later version.

## Design

Research with citations and verified or unverified labels:
[`monorepo-manager-route-research/from-scratch-inputs.md`](monorepo-manager-route-research/from-scratch-inputs.md).

### Tech stack options

#### Current stack direction

As of 2026-09-16,
the only remaining route is an all-Rust tool with file-enforcer rewritten in Rust,
shipped as one binary.
Every other option is out:

- TypeScript on Node and every shape that ships Node inside the single file,
   including a TypeScript daemon with a Rust native addon
   and the Rust core with TypeScript file-enforcer children:
   Node single executables are too big,
   per the user's decision in "Single-file shipping and a file-enforcer rewrite".
  The Rust core with TypeScript children is out by inference from that reason,
   open to the user's veto.
- Kotlin JVM core,
   all Kotlin on the JVM,
   Kotlin/Native,
   Bun,
   Deno,
   Python,
   OCaml,
   and .NET:
   the user judged them not worth further research.
- Go and Zig:
   excluded by the user before design.

The all-Rust design,
its configuration-hosting ranking,
and the questions it leaves for the user are in "All-Rust tool".

The subsections from "Findings that apply to every stack" through "Running tasks under a pseudo terminal"
record how the stack narrowed,
including rankings later superseded.

#### Selection history

Every option in "Designed options and their worst problems" was designed against the same requirements
until its disqualifying problems surfaced,
per rule `YKZ`;
full designs,
probes,
and citations live in the `stack-*.md` appendices of
[`monorepo-manager-route-research/`](monorepo-manager-route-research/).
An earlier surface-level ranking was withdrawn after the user's correction on 2026-09-16.
Go and Zig are excluded by the user.
Approval of a language for this scope is separate,
per `doc/planning/load-bearing-code-languages.md`.

#### Findings that apply to every stack

- glibc 2.39 and later provide `pidfd_spawn` with `posix_spawnattr_setcgroup_np`,
   which starts a child directly inside a cgroup;
   a probe from Python `ctypes` succeeded against the probe's own cgroup.
  Its only documentation is the glibc NEWS file and a header.
- The documented fallback is re-executing a launcher that joins the cgroup and then `exec`s the task,
   or `systemd-run --user --scope --expand-environment=no` per task.
- Writing `cgroup.procs` after start measured a 9.7-millisecond median on this host,
   whose cgroup mount lacks `favordynmods`.

#### Designed options and their worst problems

- TypeScript on Node
   (`stack-typescript.md`):
   cgroup placement before start and watch-overflow detection use `node:ffi`,
   documented as "Stability:
   1 - Experimental" since v26.1.0;
   built-in `fs.watch` silently drops events after an inotify queue overflow;
   recursive `fs.watch` watched 41,205 paths with 662 milliseconds of blocking setup;
   Node's documentation has contradictions on the `fs.watch` and FFI pages the design uses.
  A stable fallback exists:
   `systemd-run --scope` per task and per-directory watching with periodic rescans.
- Rust core with TypeScript file-enforcer children
   (`stack-rust.md`):
   task spawning into cgroups is repository-written `unsafe` code,
   because Rust's standard library lacks `clone3` and `libc` declares `CLONE_INTO_CGROUP` as a `c_int` that overflows;
   `notify`,
   `jsonrpsee`,
   and `cgroups-rs` fail the documentation rule or the requirements,
   so the watcher and JSON-RPC framing are hand-written;
   each file-enforcer rerun pays about 147 milliseconds of Node startup;
   Rust and TypeScript event types need one shared schema.
- All Rust
   (`stack-rust.md`):
   every Rust core problem plus a rewrite of 79 file-enforcer modules
   and no workable host for the 2,330-line TypeScript configuration.
  The Rust core problems were later corrected by the crate-level research in "Library-level results",
   and configuration hosting is being redesigned as recorded in "Current stack direction".
- Kotlin JVM core with TypeScript file-enforcer children
   (`stack-kotlin.md`):
   Kotlin documentation pages every Kotlin option needs contradict themselves or the library source;
   `ProcessBuilder` cannot place a child in a cgroup,
   so spawning needs JDK 25's foreign-function API beside the pinned JDK 21;
   the JDK watcher is not recursive and can stop silently.
- All Kotlin on the JVM
   (`stack-kotlin.md`):
   the Kotlin JVM core problems plus a port of about 13,774 file-enforcer lines;
   GraalVM Native Image is discontinued for Java SE customers.
- Kotlin/Native
   (`stack-kotlin.md`):
   the linker toolchain is frozen at glibc 2.19 and kernel 4.9 headers with no fix date,
   leaving `pidfd_spawn` and 11 other needed functions unbound;
   Ktor's native sockets fail at file descriptors 1024 and above.
- TypeScript on Bun
   (`stack-typescript.md`):
   the pinned 1.3.14 silently ignores the `cgroup` spawn option,
   so the sandbox is off without an error;
   fixes exist only in the 1.4 rewrite that `mise.toml` distrusts;
   the design conflicts with the repository's Bun-islands policy.
- TypeScript on Deno
   (`stack-typescript.md`):
   recursive `Deno.watchFs` blocked for 131 seconds and still watched `node_modules`;
   pending async FFI calls stalled file reads for 1.9 seconds.
- Python
   (`stack-others.md`):
   named unapproved in `doc/planning/load-bearing-code-languages.md`;
   the GIL makes threaded hashing slower than serial;
   all kernel work goes through untyped `ctypes`.
- OCaml
   (`stack-others.md`):
   Eio documents that forked-child code must be C;
   mise provides no OCaml compiler.
- C# on .NET NativeAOT
   (`stack-others.md`):
   Microsoft's own interop pages disagree on whether `DllImport` works under Native AOT,
   and every kernel call in the design is a P/Invoke.

Considered and excluded before design,
with reasons in `stack-others.md`:
Swift,
Java without Kotlin,
C++,
Haskell,
Nim,
Crystal,
and Elixir or Gleam.

#### Ranking

The ranking depends on a preference the user has not stated:
whether the documentation confusion rule applies to language runtime and library documentation
at the same strictness as it applied to monorepo manager candidates.
The research agents applied it unevenly:
the Kotlin research culled on contradictions,
the TypeScript research recorded Node contradictions without culling,
and the Rust research excluded typos.

If runtime documentation problems are recorded but do not cull:
Node > Rust core with TypeScript children > Kotlin JVM core with TypeScript children >
Python > Bun > Deno > OCaml > .NET >
all Kotlin on the JVM > all Rust > Kotlin/Native.

- Node over Rust core:
   Node has a stable fallback for each experimental dependency and stays in one language,
   while the Rust core hand-writes unsafe spawning,
   the watcher,
   RPC framing,
   and a cross-language schema.
- Rust core over Kotlin JVM core:
   both hand-write process control,
   but Kotlin adds a second JDK version and a watcher that can stop silently.
- Kotlin JVM core over Python:
   Python is recorded as unapproved.
- Python over Bun:
   Bun's pinned version turns the sandbox off silently and conflicts with the Bun-islands policy.
- Bun over Deno:
   Deno's watcher and FFI stalls block the core loop,
   while Bun's defects have fixes in a newer release.
- Deno over OCaml:
   OCaml requires C for cgroup placement and has no mise toolchain.
- OCaml over .NET:
   the .NET contradiction is on its own interop pages that every kernel call depends on.
- .NET over all Kotlin on the JVM,
   all Rust,
   and Kotlin/Native:
   each of those three adds a file-enforcer rewrite or port,
   and Kotlin/Native also lacks the needed glibc bindings.

If runtime documentation problems cull as strictly as for tool candidates,
Node,
every Kotlin option,
Bun,
Deno,
and .NET exit,
leaving Rust core with TypeScript children > Python > OCaml > all Rust,
for the same adjacent reasons.

#### Resolved preferences on 2026-09-16

- The documentation confusion rule records runtime and library documentation problems for building the tool
   but does not cull on them.
  The first ranking applies,
   so TypeScript on Node leads.
- The design may depend on experimental `node:ffi` for placing tasks into cgroups before they start
   and for detecting inotify queue overflow.
  Node's FFI documentation says the module can be disabled with `--no-experimental-ffi`,
   and the TypeScript research reports it enabled by default from v26.9.0;
   the repository currently resolves Node v26.8.2,
   so the minimum Node version or flag is an implementation detail to pin.

Node hashing measurement,
2026-09-16,
Node v26.8.2,
sequential `readFile` plus one hash per git-tracked file,
8,087 files and 144.5 MiB,
five runs each
(`node-hash-bench.ts` in the session scratchpad):

- SHA-256:
   minimum 417 milliseconds,
   median 437,
   maximum 843,
   the maximum from the first run.
- BLAKE2b-512:
   minimum 486 milliseconds,
   median 494,
   maximum 502.
- Resident memory after both series:
   224.0 MiB,
   which includes read buffers and is not a daemon footprint measurement.

A full sequential rehash stays under one second on this machine,
so hashing throughput does not decide the stack.

Recommended stack at that point:
TypeScript on Node,
following from these answers.
Superseded by "Library-level results" and then by the single-file decision.

#### Research gap found on 2026-09-16

The user challenged the ranking's basis:
the Node research evaluated built-in `fs.watch` although the repository already uses `chokidar`
in `package/dev-script/file-enforcer` and `package/dev-script/watch-restart`,
and the Rust research concluded that cgroup placement,
the watcher,
and JSON-RPC must be hand-written and `unsafe`
without surveying crate alternatives or the repository's existing `zbus` 5,
`tokio`,
and `ignore` dependencies.
The repository also already uses `@homebridge/dbus-native` in `package/kwin/key-helper`.
Library-level research for both leading options was redone:
`stack-node-libraries.md` and `stack-rust-crates.md`.

#### Library-level results

Revised Node design:
`chokidar` for watching with daemon-side re-hashing,
`systemd-run --user --scope` per task,
`@homebridge/dbus-native` for `FreezeUnit`,
`ThawUnit`,
and `KillUnit`,
`json-rpc-2.0` with `readLines` from `@monochromatic-dev/mcp-stdio`,
plain `child_process`,
and `node:crypto`.
Remaining Node problems:

- `chokidar` watches every file at repository scale:
   41,209 watches,
   1.74 to 1.78 seconds to ready,
   and 403 MiB resident memory (measured by the research).
  Its open issue `paulmillr/chokidar#1455`,
   "Event throttling discards updates",
   drops a second change within 50 milliseconds.
- No Node watcher library reports inotify queue overflow;
   a worker-thread canary detected losses on a fixture,
   unproven at repository scale.
- `@parcel/watcher`,
   the alternative,
   misses directories created or moved in after start without any event.
- Node's `'pipe'` stdio gives children a socket,
   so a task that writes `> /dev/stdout` fails:
   `sh -c 'echo probe > /dev/stdout'` spawned with piped stdio exited 1 with
   "/dev/stdout:
   No such device or address"
   (measured 2026-09-16).
- Pure-JavaScript D-Bus libraries cannot pass file descriptors,
   and systemd marks `FreezeUnit` and `ThawUnit` as not documented.
- The systemd scope model and a daemon-owned delegated cgroup need different freeze,
   kill,
   and doctor logic.

Revised Rust core design,
with no `unsafe` in repository code
(probes compiled under `#![forbid(unsafe_code)]`):
a launcher that writes its own PID into the task cgroup and calls std's `CommandExt::exec`,
direct cgroup file writes under a delegated cgroup,
the `inotify` crate with a directory walk filtered by the repository's `ignore` dependency,
`tokio-util` `LinesCodec` with `serde_json` for JSON-RPC,
`tokio::process` and `tokio::signal`,
`rustix` `ioctl_ficlone`,
and `btrfs-uapi` 0.13.0 for subvolumes and snapshots.
The development machine's systemd 259.8 user manager exposes `StartTransientUnit`,
`FreezeUnit`,
`ThawUnit`,
and `KillUnit` over D-Bus
(`busctl --user introspect`,
measured).
Remaining Rust problems:

- Moving a task into a new cgroup,
   freezing it,
   and killing it are unexercised.
- Watch correctness,
   JSON-RPC subscriptions,
   and backpressure are repository code by choice after the crate survey.
- Rust and TypeScript event types can drift;
   `yerpc` generates TypeScript types but is unchecked.
- The launcher's re-exec after a rebuild replaces the daemon binary is untested.
- `btrfs-uapi` needs `libclang` at build time.
- Rust's piped stdio does not share Node's socket problem:
   the same `/dev/stdout` probe spawned through `std::process::Command` with `Stdio::piped()`
   exited 0 and captured the output
   (measured 2026-09-16).

Revised ranking of the two leading options:
Rust core with TypeScript file-enforcer children over TypeScript on Node.
Node's remaining problems sit in two core requirements with no library fix found:
repository-scale watching that is heavy,
lossy,
and silent about overflow,
and task stdio that breaks common shell redirections.
The Rust core's remaining problems are repository-written code chosen after a crate survey
and a cross-language schema with a generator candidate.
Node would regain the lead if single-language maintenance outweighs both of those Node problems.

The lower-ranked stacks have not had the same library-level pass.
On 2026-09-16 the user judged none of them worth further research,
so Kotlin JVM core,
all Kotlin,
Kotlin/Native,
Bun,
Deno,
Python,
OCaml,
and .NET are out of consideration.
The stack choice was then between the Rust core with TypeScript file-enforcer children and TypeScript on Node,
with the Rust core recommended,
until the single-file decision removed both.

#### Single-file shipping and a file-enforcer rewrite

Raised by the user on 2026-09-16:
the TypeScript daemon with a Rust addon may not ship as a single file,
and the user is willing to rewrite file-enforcer in Rust.

- Node single executable applications are "Stability:
   1.1 - Active development",
   with built-in generation through `--build-sea` since v25.5.0.
  Native addons ship as `assets` and load by writing the asset to a temporary file and calling `process.dlopen()`
   (<https://nodejs.org/api/single-executable-applications.html>).
  One file on disk is possible,
   but the addon is extracted at run time.
- The Rust core with TypeScript file-enforcer children also needs Node and the TypeScript sources at run time,
   so it is not a single file either.
- An all-Rust tool with file-enforcer rewritten in Rust is back under consideration;
   its configuration-hosting options are being designed.

Hard requirement,
stated by the user on 2026-09-16:
the tool ships as a single file that a user runs directly,
as in `./meow`
(`meow` is a placeholder name).
Unpacking at run time is acceptable,
as AppImage does.
So a Node single executable that extracts its addon is acceptable,
while the Rust core with TypeScript file-enforcer children fails
unless Node and the TypeScript sources travel inside that one file.

Top-level await is also a hard requirement for any Node single executable,
per the user on 2026-09-16.
The Node single executable documentation offers `"mainFormat": "module"` for an ECMAScript module entry point,
and its ESM entry point example uses `await import(...)` at top level.
Probe on 2026-09-16 with Node v26.8.2:
a `main.mjs` running `await new Promise(...)` and `await import("node:os")` at top level,
built with `node --build-sea` and `"mainFormat": "module"`,
ran as `./meow` and printed `top-level await ok on linux`;
the executable was 144 MiB
(`~/temp/agent/sea-tla-probe-2026-09-16`).
ECMAScript module entry points landed in `nodejs/node#61813`,
"sea:
 support ESM entry point in SEA",
merged 2026-02-18 with a `backport-open-v24.x` label;
before it,
single executable entry scripts were CommonJS,
which has no top-level await.
The Node version the tool builds with must include that change.

Decision on 2026-09-16:
the user killed the TypeScript route because Node single executables are too big;
the minimal probe measured 144 MiB.
By the same reason,
every shape that ships Node inside the single file is out,
including the Rust core with TypeScript file-enforcer children,
unless the user says otherwise.
The remaining route is an all-Rust tool with file-enforcer rewritten in Rust;
its configuration-hosting variants are being designed.

#### How TypeScript monorepo tools meet the same problems

Checked 2026-09-16 after the user noted that many monorepo tools are written in TypeScript.

- The `/dev/stdout` failure comes from libuv:
   `UV_CREATE_PIPE` creates child stdio with `uv_socketpair`
   (`deps/uv/src/unix/process.c:202-207` in the Node v26.8.2 source clone).
  A pseudo terminal or a real pipe avoids it.
- Nx keeps its orchestration in TypeScript but implements these parts in its Rust native addon:
   `packages/nx/Cargo.toml` in `nrwl/nx` depends on `notify = "=9.0.0-rc.5"` for watching,
   `portable-pty` for running tasks in a pseudo terminal,
   `ignore = '0.4'` for walking,
   and `xxhash-rust` for hashing,
   and `packages/nx/src/native/` contains `watch/`,
   `pseudo_terminal/`,
   and `hasher.rs`.
- So a TypeScript daemon with a Rust native addon for watching,
   task spawning,
   and hashing is a third shape that neither stack deep dive designed.
  Its design research started on 2026-09-16 and was stopped unfinished when the user killed the TypeScript route;
   Nx's addon also runs tasks under a pseudo terminal,
   which "Running tasks under a pseudo terminal" records as disqualifying.

#### Limits no stack removes

- inotify is per directory and not recursive,
   and a full queue drops events and generates `IN_Q_OVERFLOW`
   (<https://man7.org/linux/man-pages/man7/inotify.7.html>).
  The development machine's `max_queued_events` is 16,384
   (measured).
  Per the user on 2026-09-16,
   the queue limit is not a con against any stack:
   `doctor` reads `/proc/sys/fs/inotify/max_queued_events` and `max_user_watches` without root
   and tells the user how to raise them persistently,
   for example with a drop-in under `/etc/sysctl.d/`.
- Cgroup delegation,
   a user systemd session on CI,
   frozen tasks holding locks and timers,
   daemons that escape task cgroups,
   watcher correctness logic,
   and JSON-RPC subscription logic are the same work in every stack.

#### Running tasks under a pseudo terminal

Probe on 2026-09-16:
Python `pty.openpty` with the slave as a child's stdout and stderr,
running `sh` that printed to stdout,
to stderr,
tested `test -t 1`,
and wrote `> /dev/stdout`.
The master read returned `b'out\r\nerr\r\nstdout-is-tty\r\nredirect\r\n'`,
then `EIO` instead of end of file.

- Stdout and stderr arrive as one stream,
   so the daemon cannot tell them apart.
- Output newlines become `\r\n`:
   termios `ONLCR` "Map NL to CR-NL on output"
   (<https://man7.org/linux/man-pages/man3/termios.3.html>).
- Tasks see a terminal,
   so tools switch to colors,
   progress bars,
   pagers,
   or interactive prompts,
   and logs differ from non-terminal CI runs.
- Terminal control characters generate signals under `ISIG`,
   and the task's input side needs an explicit policy.
- After the child exits,
   reading the master fails with `EIO` rather than returning end of file.
- Ptys are a bounded resource:
   `/proc/sys/kernel/pty/max` is 4,096 with 15 in use
   (measured).
- `> /dev/stdout` works under a pty,
   which is the problem a pty was proposed to solve.

The user classified these pseudo-terminal costs as disqualifying problems on 2026-09-16:
tasks do not run under a pseudo terminal by default,
and any design that does,
including Nx's `portable-pty` task runner,
carries those problems.
Accepted by the user on 2026-09-16:
0.x offers no pseudo-terminal opt-in.
A task needs a terminal only when a person must interact with it,
such as an editor or a terminal password prompt;
the repository's interactive `mise run --raw secrets:edit` contract
(`doc/planning/mise-removal-coverage.md`)
is that kind of command,
and it runs in the user's own terminal,
outside the unattended daemon.

A real pipe solves the same problem without those costs,
in Node as well:
a Node child spawned with a FIFO write descriptor as stdout
(`real-pipe-stdio-probe.ts` in the session scratchpad)
passed `test -p /dev/stdout`,
wrote `> /dev/stdout` successfully,
and produced `stdout-is-pipe\nredirect\n` with plain newlines and exit code 0
(measured 2026-09-16).
The `/dev/stdout` failure is therefore specific to libuv's socketpair for `'pipe'` stdio,
not to Node,
and it does not decide the stack.

### All-Rust tool

Research:
[`stack-all-rust-rewrite.md`](monorepo-manager-route-research/stack-all-rust-rewrite.md),
2026-09-16.
Its Cargo builds ran offline because the research agent could not reach `index.crates.io`,
so engines missing from the local crate cache were judged from documentation,
source,
and release assets.

#### Rewrite scope

- Production code:
   77 modules,
   13,396 lines,
   6,469 code lines,
   excluding tests,
   a fuzz budget,
   a regression fixture,
   a container test,
   and the generated package index
   (re-running the research's `allrust/fe-count.ts` reproduced these totals).
- Leaves the port:
   FE18 and FE19 go to Meta Package Manager
   (10 modules,
   943 code lines,
   plus the generated index),
   and FE21 to FE23 watch mode is replaced by the daemon watcher
   (9 modules,
   1,027 code lines).
- Plugins compiled into the tool:
   FE14 Cargo
   (4 modules,
   241 code lines)
   and FE15 JetBrains
   (5 modules,
   711 code lines).
- Core to port:
   49 modules and 3,547 code lines,
   including 20 staleness modules with 1,420 code lines,
   plus the `module-toml-edit` formatting and placement rules file-enforcer reaches
   and 56 test files with 11,126 lines.
- The root `file-enforcer.config.ts` is 2,329 lines.
- Consumers outside the package:
   `package/dev-script/vm-builder` imports `exec` from the `/ts` subpath in two files;
   `package/test-fixture/file-enforcer-perf` benchmarks the TypeScript implementation;
   a `prefer-readonly-parameter-type` unit test reads `cargo/apply-plan.ts` as a fixture;
   and the `sync:files` and `watch:sync:files` Mise tasks run the TypeScript CLI.
- Recorded decision in conflict:
   `package/dev-script/file-enforcer/DECISION.rust-migration.md`,
   "Decision:
   no Rust migration for file-enforcer",
   whose reasons do not address single-file shipping;
   it is superseded only after the user accepts a variant.

#### Byte-identical output

- Crate defaults change today's bytes.
  `toml_edit` 0.25.13 `Table::insert` resets an existing key's formatting through `entry.key_mut().fmt()`
   (`src/table.rs:429-443` in the cached crate,
   read 2026-09-16),
   which deleted the comment above a key in the probe;
   arrays,
   inline-table trailing commas,
   and new top-level key placement also differ.
- `serde_json` differs from `JSON.stringify` in key order for array-index keys and in number formatting.
- Repository-written emulation layers matched every probed TOML,
   JSON,
   glob-mirror,
   and well-formed XML case.
- Malformed XML still differs:
   `quick-xml` and `roxmltree` fail where `@lezer/xml` recovers and today's code splices anyway.
- A mismatch rewrites managed files,
   including `CLAUDE.md`,
   so a differential harness in a throwaway worktree gates the switch.

#### Single binary

- A no-engine daemon skeleton built stripped with LTO for glibc is 2,055,536 bytes,
   or 3,134,664 bytes with `zbus`.
- When the research ran,
   this host had only the `x86_64-unknown-linux-gnu` target and no static glibc.
  The user then authorized installing the musl target,
   and `rustup target add x86_64-unknown-linux-musl` installed it for the active nightly-2026-09-12 toolchain.
- Static probe,
   2026-09-16:
   the same skeleton with BLAKE3 and SHA-256 replaced by `xxhash-rust` 0.8.15 XXH3-128
   (`allrust/skeleton-musl` in the session scratchpad,
   `cargo build --offline --release --jobs 4`)
   measured 1,983,424 bytes for glibc and 2,098,120 bytes for `x86_64-unknown-linux-musl`,
   which `file` reports as "static-pie linked".
  Run as `skeleton-musl run <throwaway directory>`,
   the static binary added inotify watches,
   walked,
   matched a glob,
   hashed,
   parsed TOML,
   JSON,
   and XML,
   read the btrfs `statfs` magic `2435016766`,
   bound a Unix socket,
   spawned a child,
   and exited 0.
  Binding the socket under the long session scratchpad path failed with "path must be shorter than SUN_LEN",
   so the daemon's socket path needs a length check.
- C sources in QuickJS-ng,
   vendored Lua,
   and Wasmtime's helper need a musl-targeting C compiler for a static build;
   the `blake3` crate also compiles assembly through `cc` unless its `pure` feature is set
   (`build.rs:228-229` in `blake3` 1.8.7).
- The stated requirement is one file that runs directly,
   with AppImage-style unpacking acceptable;
   it does not say static linking,
   and a glibc-linked binary is also one file,
   tied to the host glibc.

#### Configuration-hosting variants

These variants were designed on the premise that the configuration keeps today's logic;
the user's statement in "Configuration" removed that premise.
Worst problems per variant:

- A1,
   TypeScript on `rquickjs` (QuickJS-ng):
   no disqualifying problem found.
  Needs repository-written shims for the Node modules the configuration imports
   and a `.d.ts` kept in step with them;
   type stripping needs an unmeasured `oxc` dependency or JavaScript with JSDoc types;
   C sources;
   single-threaded evaluation.
  Most of the 2,329 configuration lines stay as they are.
- A2,
   TypeScript on Boa:
   A1's shim and stripping work,
   plus a size bounded only by the 33.8 MB Boa CLI;
   pure Rust.
- A3,
   TypeScript on `deno_core` (V8):
   V8's compressed static library is 39,784,686 bytes and the host `deno` binary is 95,600,728 bytes.
- B,
   Starlark:
   `async`,
   `try`,
   `except`,
   `while`,
   and `class` are reserved,
   so the configuration is a full rewrite;
   no released language-server binary.
- C,
   Rhai:
   no async per its maintainer;
   language server last pushed 2023-03-17;
   full rewrite.
- D,
   Lua through `mlua`:
   async support and a small engine;
   full rewrite;
   vendored C sources.
- E,
   Rune:
   one release in the past year,
   low adoption,
   language server only as a 2023 nightly asset;
   full rewrite.
- F,
   Rust compiled into the tool:
   every configuration edit needs `cargo` and restarts the daemon,
   untracked `std::fs` reads are blocked only by lint,
   and the binary carries one repository's configuration.
- G,
   Rust configuration crate run as a child binary:
   `cargo` on every fresh clone and version skew between the tool and the configuration library.
- H,
   WebAssembly guest on `wasmtime`:
   `cargo` and a WebAssembly target for edits,
   or a committed `.wasm`;
   71 `wasmtime` releases in the past year;
   component-model size unmeasured.
- I,
   declarative TOML or KDL with built-in Rust generators:
   logic edits behave like F,
   and each bespoke generator needs its own schema.
- J,
   Nickel or Jsonnet:
   Nickel has no effects and a 19,498,430-byte static library;
   Jsonnet's Rust evaluator has had no stable release since 2021.
- Excluded before design:
   Pkl needs its 101,977,360-byte binary on `PATH`,
   CUE embedding needs Go,
   Dhall has no host effects,
   and dynamically loaded Rust plugins cannot load from a static musl binary.

Shared by every variant:
the byte-identical output work,
the unexercised cgroup,
watcher,
and RPC risks from `stack-rust-crates.md`,
and `browserslist-rs`,
whose data follows crate releases instead of the pnpm lock and whose output is unprobed.

#### Ranking from the research

Withdrawn as a recommendation by "Correction on 2026-09-16";
kept as evidence.

A1 > A2 > D > B > C > E > H > I > F > G > Jsonnet > Nickel > A3.

- A1 over A2:
   both keep TypeScript,
   but QuickJS-ng has size evidence under 2.6 MB while Boa's only bound is 33.8 MB.
- A2 over D:
   A2 keeps most of the configuration and rule `AD2`'s TypeScript wording;
   D rewrites everything into a language no repository rule names.
- D over B:
   Lua keeps async and error handling;
   Starlark has neither.
- B over C:
   Starlark has a maintained language-server library;
   Rhai's language server is stale and neither has async.
- C over E:
   Rhai releases and is adopted more.
- E over H:
   Rune needs no compiler for edits.
- H over I:
   WebAssembly hot-loads and sandboxes;
   I restarts the daemon and invents schemas.
- I over F:
   I's data-only edits need no compiler or restart.
- F over G:
   an unedited F configuration runs without `cargo`.
- G over Jsonnet:
   G's toolchain is maintained.
- Jsonnet over Nickel:
   Jsonnet can read through native callbacks and its CLI is 3.3 MB.
- Nickel over A3:
   Nickel's size is bounded at 19.5 MB,
   while V8 sits closer to the size that killed Node.

#### Correction on 2026-09-16

- The question put to the user,
   whether configuration edits may need the Rust toolchain,
   assumed the configuration must stay Turing-complete to host today's logic.
  The user removed that assumption
   ("Configuration").
- Retracted:
   the A1 recommendation,
   and counting "most of the 2,329 lines stay as they are" as a benefit.
  Keeping the time-constrained configuration's shape is not a goal.
- Moot:
   whether F,
   G,
   H,
   and I fail the single-file requirement,
   and A3's size threshold.
- New direction:
   a declarative configuration,
   with today's configuration logic moved into general built-in features of the tool
   or into ordinary repository tasks with declared inputs and outputs,
   since `./meow` also serves other repositories.
  Research designing the format options and placing every unit of today's logic is running,
   with OpenTofu as the lead precedent.
- Process gap:
   the configuration-hosting research carried the incumbent's Turing-complete shape as a requirement
   instead of asking whether it survives the new project scope.

#### Adopted from settled requirements

Open to the user's veto:

- File-enforcement work runs as a child that re-executes the single file,
   such as `/proc/self/exe` with an internal subcommand,
   inside a task cgroup,
   not in the daemon process.
  Cgroup sandboxing is a must,
   and pause and end act through `cgroup.freeze` and `cgroup.kill`,
   which cannot target in-process work.
  Event types stay shared inside one binary.
- A declarative configuration stays config-as-data under rule `AD2`,
   so `AD2` needs no change for the tool's own configuration.

#### Next measurements

- Done 2026-09-16:
   a static musl daemon skeleton
   ("Single binary").
- Run the differential output harness in a throwaway worktree.
- Measure the chosen configuration format's parser in the static build.

### Declarative configuration

Research:
[`stack-declarative-config.md`](monorepo-manager-route-research/stack-declarative-config.md),
2026-09-16.
Cargo reached crates.io this time,
so its sizes come from static musl builds.
Spot-checked on 2026-09-17:
the size table matches `declarative/sizes/out/summary.txt` in the session scratchpad,
and the tests of `starlark` 0.14.2 expect recursion to run until "Starlark call stack overflow"
(`src/tests/call.rs:63-65` in the cached crate).

#### Correction to the research premise

The research treated Turing-completeness as forbidden,
because the decision record and this design said the configuration is "not Turing-complete".
The user said the Turing-complete requirement is eliminated,
not that Turing-completeness is forbidden;
both documents were corrected on 2026-09-17,
and the question is being asked.
Options G (Starlark) and H (KCL) are disqualified only under the forbidding reading.

#### Logic inventory

Today's `file-enforcer.config.ts` splits into 27 units:

- General built-in features:
   13,
   such as the forbidden `CONTEXT.md` check,
   `LICENSE`,
   `CLAUDE.md`,
   git-policy mirrors,
   license texts,
   Cargo manifest keys,
   JetBrains settings,
   the skill mirror,
   scheduling,
   events,
   and input tracking.
- Repository tasks:
   3,
   the forbidden-strings rule compilation,
   the pnpr configuration generator,
   and resolved Browserslist targets.
- Plain data:
   5.
- Retired:
   6,
   including `mise.toml` generation once Mise leaves CI.

Awkward placements:
per-manifest Cargo derivations,
writes outside the repository (JetBrains settings and the scanner cache),
license pruning broader than today's,
and `mise.toml` while Mise stays on the macOS and Windows runners.

#### Format options and ranking

Static musl size added over a 385,656-byte baseline:
`jsonc-parser` 180,224 bytes,
`toml` 208,896,
`toml_edit` 225,280,
`kdl` 303,104,
`hcl-edit` 344,064,
`serde-saphyr` 942,216,
`hcl-rs` with its evaluator 1,052,776,
`serde_dhall` 1,785,992,
`cel` 2,662,824,
and `regorus` 7,038,952.

Ranking from the research:
A (OpenTofu-shaped HCL) > B (TOML) > F (TOML with CEL) > E (JSONC) > D (YAML)
> C (KDL) > I (Dhall) > J (Rego) > G (Starlark) > H (KCL).

- A over B:
   native expressions keep per-file derivations in one general language,
   following the user's OpenTofu pointer and the repository's OpenTofu incumbent;
   the gaps in A are repository code the user accepted writing.
- Worst problems of A:
   `hcl-rs` has no built-in functions (issue #484 open)
   and drops source locations on evaluation errors,
   `hcl-edit` has no formatter and warns "Expect breaking changes at any time",
   and its round trip joined a four-line `&&` condition onto one line.
- OpenTofu 1.12.6 expressed `CLAUDE.md`,
   the skill mirror map,
   SPDX text mapping,
   and pnpr entry-point selection with built-in functions only.
- The remaining adjacent reasons are in "8.
   Ranking" of the appendix.

#### Settled without asking

Each follows from recorded decisions:

- Retracted on 2026-09-17.
  This entry read "Ordering comes from declared reads,
   writes,
   and explicit `depends_on`,
   which keeps author control over sequencing where it matters (FE01)",
   recorded as following from earlier decisions.
  The user said:
   "I never approved 'depends' or 'depends on'.
   Let's align on that.
   Grill me."
  Nothing about task ordering is decided;
   it is being asked instead.
  Asked what creates ordering,
   the user reframed the question:
   "We need to settle on 'what is a task' first too.
   In my opinion 'build' and 'test' should be builtin to `meow` and not mixed in with lesser tasks."
  Clarified immediately after:
   "`meow build` and `meow test` are not dedicated commands.
   They still run with the `run` syntax.
   Let's say some tasks are more than others."
  The earlier conclusion that `build` and `test` are special task names is withdrawn.
  The user subsequently rejected Claude's task framing:
   development stages need not be modeled as build tasks and similar commands.
  Keeping `meow run //package/cow:test` syntax does not decide what its label refers to.
  The [software model exploration](monorepo-manager-software-model.md)
   starts from software state and lifecycle meaning before choosing execution or ordering mechanisms.
- Settled the same day:
   cross-package edges are read from the native manifests,
   the pnpm workspace links,
   Cargo path dependencies,
   and Gradle project references that already exist and are maintained by the tools that need them.
- Settled the same day:
   freshness comes from hashing a task's declared reads,
   so unchanged hashes mean a cache hit and no execution,
   with undeclared reads remaining the recorded staleness risk.
- Skill-mirror ownership moves into the tool's state;
   both mirror roots are gitignored.
- The pnpr configuration and Browserslist targets become TypeScript tasks in their own packages,
   which already hold that code.
- The research's byte-identity question was already answered:
   a one-time reviewed change.

#### Answers on 2026-09-17

- Expression power:
   "Full language allowed".
  Turing-completeness is permitted,
   so G (Starlark) and H (KCL) lose their only disqualifier.
  A stays first:
   the user endorsed HCL,
   while `starlark` 0.14.2 fails to build on the repository nightly and pulls a C compiler,
   and KCL is not published on crates.io.
  Author-defined functions and recursion are allowed within the chosen syntax.
- Syntax:
   "HCL syntax is fine.
   This is only a light endorsement."
- Rules that write outside the repository,
   such as the JetBrains settings,
   live in a per-user `meow` configuration.
- Forbidden-strings rule compilation uses the published scanner,
   not a repository build:
   the user noted "We already publish it to crates.io and GitHub releases and cargo-binstall should discover it fine."
  `forbidden-strings` 0.4.1 is on crates.io (API read 2026-09-17).
  The research carried over today's local path,
   `package/forbidden-strings/cli/target/release/forbidden-strings` (`file-enforcer.config.ts:182-184`),
   and asked whether to build it first.
  A released scanner older than the repository cannot corrupt the compiled cache:
   the cache has a magic header (`package/forbidden-strings/cli/src/runtime_cache/envelope.rs:20`)
   and the scanner reports "compile-from-text" recovery on a mismatch (`src/runtime_cache/warning.rs:78`).

#### Root `mise.toml` and where meow is built

Answered by the user on 2026-09-17:
"Hand-maintained file,
 and we're obviously going to build meow in a new worktree."

- Clarified by the user the same day:
   "mise.toml:
   hand-maintained now,
   once meow takes over,
   mise is out."
  Corrected on 2026-09-17,
   after the vm-builder research found that `file-enforcer.config.ts:718-721` still writes the file
   and `mise.toml:1` still carries its generated header:
   asked which record stands,
   the user chose "Correct the record",
   so file-enforcer keeps generating the root `mise.toml` until Mise is removed,
   and the hand-maintained wording applied to `mise.no-env.toml`,
   the source it is generated from.
  meow gets no Mise-specific rule,
   and Mise is removed entirely when meow takes over,
   which happens only once meow supports the full platform matrix
   ("Platforms and builds").
- "`meow` doesn't need to own the MPM lifecycle":
   installing and updating Meta Package Manager stays outside meow.
- meow is built in a separate git worktree,
   so the TypeScript file-enforcer and meow never enforce the same tree during development,
   and the lock interoperability concern for coexisting enforcers does not arise.

### Configuration authoring

The software-model correction in "Open questions" limits how this section is read:
these answers remain the record of configuration preferences,
not proof that every development concept must be represented by a `task` block.
The built-in schema awaits that discussion.

Answered by the user on 2026-09-17,
after the research's per-package proposal was put to them.

- One root `meow.hcl` holds everything:
   "One root file for everything,
   with tags per pkg and inherince."
  There is no per-package configuration file,
   which is a deliberate break from today's 178 Mise files
   and from the research's `task_template` plus per-package `project` block sketch.
- Packages carry tags,
   and tasks reach packages through those tags rather than through file placement.
- Inheritance composes task definitions,
   so a task written once applies to every package its tags select.
- A task's command is an argv list,
   spawned directly with no shell:
   `command = ["node", "--test", "src"]`.
  This matches how file-enforcer's `exec` and the repository's other spawners already run programs,
   and keeps quoting and word splitting out of the language.
- Block shape is OpenTofu's one-label form,
   as `variable`,
   `module`,
   and `output` use,
   without the `resource` keyword,
   which in OpenTofu exists to namespace provider types meow does not have:
   `task "test" { }` (user,
   2026-09-17).
- The working directory scopes every command (user,
   2026-09-17):
   "Infer everywhere",
   so `meow run test` inside a package means that package's task,
   and listings narrow the way standing in a directory narrows them today
   (56 tasks against 1,807).
- The logic that today lives inline in TOML strings,
   for built-versus-source selection,
   test discovery,
   and platform branching,
   becomes meow built-ins (user,
   2026-09-17),
   not repository bins and not filesystem probing from HCL expressions.
- An agent in a disposable worktree gets past the trust prompt with `meow trust --yes` (user,
   2026-09-17),
   the non-interactive form cli-git already defines,
   rather than a new flag.
- Tags are inferred and then adjusted:
   meow derives the obvious ones from manifests and layout,
   and `package` blocks add or remove the rest.
  A `package` block's selector accepts globs,
   so one block can tag a subtree (user,
   2026-09-17).
- Specialization is by tag:
   several `task` blocks may share a name with different tag sets,
   and the most specific matching set wins.
- Overriding is explicit on both sides (user,
   2026-09-17):
   the specialization is marked as overriding,
   and the task it overrides is marked as overridden,
   in the spirit of Kotlin's `override` and `open`.
  A specialization without a marked base,
   or a marked base nothing overrides,
   is an error rather than a silent win.
  The markers are meta-argument attributes (user,
   2026-09-17),
   the form OpenTofu already uses for `count`,
   `for_each`,
   and `depends_on`:
   `overridden = true` on the base,
   `override = true` on the specialization,
   validated as a pair.
  Rule `SYB` ruled out a comment-based marker,
   because an attribute carries the relation directly.

```hcl
task "test" {
  tags       = ["pnpm"]
  overridden = true
  command    = ["node", "--test", "src"]
}

task "test" {
  tags     = ["pnpm", "serial"]
  override = true
  command  = ["node", "--test", "--concurrency=1", "src"]
}
```

#### What using Mise for tasks is actually like

Study:
[`mise-task-usage.md`](monorepo-manager-route-research/mise-task-usage.md),
2026-09-17,
requested by the user as "the good and the bad and the ugly".
It supplies what the design was missing:
evidence from the incumbent rather than from what meow should be.

Worth keeping:

- One vocabulary across 179 packages:
   `mise run //package/<path>:lint:types` works for all 145 packages that declare it,
   from one definition at `mise.toml:606-612`,
   which is what makes the rule in `AGENTS.md` workable.
- Bazel labels are already universal:
   1,836 of 1,836 tasks are namespaced.
- Standing inside a package narrows a listing to 56 tasks against 1,807,
   and 174 distinct working directories come from file placement,
   with only 35 explicit `dir` lines.

What the decided meow design already fixes:

- Tag selection removes 2,632 lines of header-plus-`extends`,
   since 1,341 of 1,898 task blocks (70.7%) are a bare `extends`.
- Explicit `overridden` and `override` markers address 96 blocks that silently shadow a template name
   with different behavior,
   plus 21 partial overrides.
- Argv-list commands remove 112 `&&` and 26 `;` entries
   and the hand-written shell-word parser at `mise.toml:426-451`.
- A real expression language replaces textual interpolation:
   1,159 of 1,836 tasks (63.1%) have a `run` containing `{{vars.`,
   and `mise task info` prints the template unexpanded,
   so a task's real behavior is invisible from both its definition and the inspection command.
- The daemon-held cache answers the freshness gap:
   no task declares `sources` or `outputs`,
   so nothing is ever skipped as fresh.

Costs the study found in decisions already made:

- One root file survives better than expected on content:
   package configuration files contain only `[tasks.*]`,
   five `[vars]`,
   and three `[env]`,
   and none declares `[tools]`,
   `[settings]`,
   or `[hooks]`,
   so nothing has to be relocated.
- It costs size and listing:
   removing bare `extends` still leaves 5,831 substantive package lines plus 1,240 root source lines,
   so one `meow.hcl` lands near 7,000 lines,
   which is where `hcl-edit`'s missing formatter starts to matter;
   and the design says nothing about what narrows a listing when a person stands in a directory,
   which today is 56 tasks against 1,807.
- Argv-only commands leave the non-fan-out half of 1,155 inline TypeScript lines without an owner:
   built-versus-source selection,
   test discovery,
   and platform branching.
- Tag inference must reproduce a graph currently encoded in task names,
   whose only written-down exceptions are two comments.
- Three incumbent behaviors have no counterpart in the design:
   Windows run variants,
   interactive tasks such as `secrets:edit` needing `--raw`,
   and worktree trust,
   which bites hardest because rule `IWT` sends agents into a fresh worktree
   and an agent cannot answer an interactive trust prompt.

Defects in today's repository,
verified before merging and filed for repair:

- `mise run test` at the root does not run 15 packages' tests.
  `mise.toml:681-698` spawns only the root `test:unit`,
   `test:browser`,
   and `test:e2e`;
   root `test:unit` globs `**/*.unit.test.ts`,
   so the 15 package `test` tasks that run `cargo nextest` or `./gradlew` are unreachable.
  `//:lint` does fan out,
   so lint reaches those packages and test does not,
   and `//:validate` and `//:buildAndTest` inherit the gap.
- Three active tasks still use the `mise watch ... -- node ...` form
   that `doc/troubleshooting/mise-watch-runs-tasks-not-commands.md` documented:
   `package/kwin/key-helper/mise.toml:40`,
   `package/desktop-daemon/hall-monitor/mise.toml:11`,
   and `package/webapp-productivity/rss/mise.toml:21`.
- `mise.toml:308-312` justifies 85 lines of hand-written orchestration by citing an observation
   that `doc/handover/lint-fix-2026-06.md` does not contain;
   the claim is repeated in `doc/research/bazel-migration-dx.md:47` with no primary evidence,
   while `doc/decision/desktop-app-podman-build.md:84-87` still tells readers to use the native glob.

#### Consequences of one root file

- Target labels cannot come from a file's directory,
   because there is one file;
   they come from the package a tag selected,
   as in `//package/fy/cli:test`.
- A change to the root file re-evaluates everything,
   so the read set and the cache key for configuration evaluation cover one file rather than many
   ("Per-user configuration").
- The language server works in one large file,
   which makes its outline and go-to-definition more load-bearing than they would be per package.
- The file is large by construction:
   today's root `mise.toml` is generated and already long,
   and every package's tasks now live beside it.

### HCL tooling

Research:
[`hcl-tooling.md`](monorepo-manager-route-research/hcl-tooling.md),
2026-09-17,
with two vet reports:
[`tech-meow-hcl-front-end-vet-2026-09-17.md`](../audit/tech-meow-hcl-front-end-vet-2026-09-17.md)
and
[`tech-meow-language-server-framework-vet-2026-09-17.md`](../audit/tech-meow-language-server-framework-vet-2026-09-17.md).
Nothing here is adopted:
the brief went to the user on 2026-09-17 and rule `DRR` requires acceptance first.

#### Answers on 2026-09-17

- Front end accepted:
   "Patched `hcl-edit`".
  meow depends on `hcl-edit` 0.9.7 with the two prototype patches carried,
   under its own evaluator over that tree.
- Functions with unpredictable results:
   "Allow,
   mark evaluations uncacheable for stuff that used them,
   and warn user with detailed messages."
  `timestamp`,
   `uuid`,
   and `bcrypt` exist with OpenTofu's names.
  An evaluation that calls one is recorded as uncacheable,
   and meow warns with a diagnostic naming the function,
   the block it was called from,
   and what caching it disabled (rule `DGT`).
- Upstream defects:
   "All three".
  Reminder issues track the user filing the two `hcl-edit` write-back defects
   and the `hcl-rs` evaluator divergences personally.
- Naming:
   "Namespaced `meow::tomldecode`".
  meow-only functions carry the `meow::` namespace,
   so an author can see which names are portable to OpenTofu;
   OpenTofu's own `provider::` namespace is the precedent.
- Formatter:
   "Ship no formatter".
  meow has no `fmt` command and no formatting rules of its own,
   so managed HCL keeps whatever style its author wrote.
  The `hclwrite` conformance corpus stays in the research as evidence,
   unused.
- Language server:
   "`lsp-server`".
  `lsp-server` 0.10.0 with `gen-lsp-types` 0.11.0,
   behind a hidden `meow lsp` stdio subcommand.
- Diagnostic renderer:
   dissolved.
  Every line meow emits is JSON ("Output format"),
   so no renderer crate is taken.

#### Recommended shape

- Front end:
   `hcl-edit` 0.9.7 carried with two prototype patches,
   under meow's own evaluator over its tree.
- Functions:
   a curated set with OpenTofu's names,
   plus `tomldecode`,
   `filetype`,
   indented `jsonencode`,
   and a `glob` shape;
   impure and Terraform-only families dropped.
  I/O functions take a read sink recording `File`,
   `Absent`,
   `Directory`,
   `Glob`,
   and `Env` entries,
   and that read set becomes the daemon's cache key and watch list.
  `chmod`,
   `mkdir`,
   and `spawn` stay block effects rather than functions.
- Formatter,
   rejected by the user on 2026-09-17 in favor of shipping none:
   meow's own over the same tree,
   with byte equality against `hclwrite` output on every valid corpus file as the conformance target.
- Language server:
   `lsp-server` 0.10.0 with `gen-lsp-types` 0.11.0,
   behind a hidden `meow lsp` stdio subcommand.

#### Deciding evidence

Corpus:
2,246 files,
1,317,750 bytes,
drawn from `opentofu`,
`hcl`,
`hcl-rs`,
`tofu-ls`,
and `hcl-lang`,
plus one synthetic meow inventory.
The Go reference accepts 2,185 and rejects 61.

- `hcl-edit` accepted and rejected exactly what the reference did on all 2,246 files.
  `babbel_hcl` failed 199,
   the `SamuelMarks` crate failed 85 and panicked on one,
   and `tree-sitter-hcl` reported no error node on 5 invalid files.
- `hcl-edit` round-tripped 2,153 of 2,185 valid files byte for byte.
  The 32 differences reduce to two defects,
   both root-caused,
   prototype-fixed,
   and green on the upstream suite (243 passed,
   0 failed):
   comments lost inside binary operations
   ([`hcl-edit-binary-operator-decor.md`](../troubleshooting/hcl-edit-binary-operator-decor.md))
   and `<<-` heredoc introducer and body indentation rewritten
   ([`hcl-edit-heredoc-dedent.md`](../troubleshooting/hcl-edit-heredoc-dedent.md)).
- `hcl-rs` 0.19.8's evaluator diverged from `hclsyntax` with `go-cty` on 12 of 21 cases:
   wrapping arithmetic,
   `f64::MAX` where the reference gives infinity,
   insertion-order object iteration where the specification says lexicographic,
   a panic on `5 % 0`,
   and `-9223372036854775809` evaluating to `9223372036854775807`;
   four of the twelve panic in a build with overflow checks
   ([`hcl-rs-eval-arithmetic-and-iteration-order.md`](../troubleshooting/hcl-rs-eval-arithmetic-and-iteration-order.md)).
  Its evaluation errors carry no spans,
   while `hcl-edit` exposes byte spans on every node,
   which rule `DGT` needs.
- Formatter reference:
   `hclwrite` changes 676 of the 2,185 valid files and is idempotent on all of them;
   `tofu fmt` is `hclwrite` plus three Terraform-only rewrites.
  `hcl-rs`'s `hcl::format` deletes every comment and is not idempotent,
   and the `SamuelMarks` formatter is not idempotent on 1,444 files.
- Language server frameworks:
   `tower-lsp` does not compile at HEAD in any feature combination,
   `async-lsp` has 4 tests and 1 adopter against `lsp-server`'s 44 and 4,
   and `tower-lsp-server` adds 1,499,280 bytes against 274,544 for `lsp-server` with a types crate.
- Static musl sizes over a 455,424-byte baseline:
   `hcl-edit` 344,064,
   `hcl-rs` 495,648,
   `tree-sitter-hcl` 184,384,
   `codespan-reporting` 57,344,
   `ariadne` 77,824,
   `annotate-snippets` 176,128,
   `miette` with `fancy` 307,232.
- Scores:
   front end `hcl-edit` 79.5 over `tree-sitter-hcl` 60.6 and `hcl-rs` 56.8,
   winner stable in all 63 sensitivity tests;
   language server `lsp-server` 87.5 over `tower-lsp-server` 73.2 and `async-lsp` 58.0,
   order stable in all 44.
- Speed decides nothing:
   a 12,618-byte configuration parses in 371 microseconds median,
   with a 30.5% run-to-run band.

#### Consequences and risks

- The evaluator carries a base directory per configuration file,
   because `file`,
   `fileset`,
   and `templatefile` resolve relative to it,
   and the cache key includes both the repository and per-user configuration digests.
- `hcl-edit` aborts the process at nesting depth 5,000 with no knob,
   so meow pre-scans depth before parsing;
   the Go reference survives to 100,000 with a 1 GB stack.
- A patched dependency must be re-applied and re-measured on every upstream release;
   the round-trip probe over the corpus is the regression test and ran in 0.255 seconds.
- `hcl-edit` has no error recovery,
   which limits the language server on a file in mid-edit;
   that is the most likely reason a later session would own the parser too.
- Owning HCL semantics means owning its corner cases,
   mitigated by the Go oracle harness,
   which makes meow's conformance suite differential rather than hand-written.
- `lsp-types` upstream last committed on 2024-06-04,
   so protocol types stay behind meow's own thin layer.
- The corpus is Terraform-shaped,
   since no meow configuration exists yet,
   so conformance is proven against the language rather than meow's idioms.
- A C toolchain in the build is an open user question,
   not a settled ban;
   the `tree-sitter-hcl` exit stands on its 5 accepted invalid files.

### Per-user configuration

Research:
[`per-user-config.md`](monorepo-manager-route-research/per-user-config.md),
2026-09-17.
The brief went to the user the same day;
rule `DRR` requires acceptance before any of it becomes a decision.

#### Answers on 2026-09-17

- Outside-the-repository writes:
   "Repo proposes,
   you accept".
  A repository declares content and no outside destination;
   the per-user file supplies destination and consent,
   matchable by path prefix,
   and an unmatched proposal is reported.
- Per-user tasks:
   "Yes,
   in a separate non-shadowing kind".
  The per-user file may define tasks that run in any repository,
   in a kind that cannot collide with a repository task name
   and does not join a repository's task graph or its cache keys.
- Automated runs:
   "Switch,
   default on everywhere".
  `--no-user-config` and `MEOW_NO_USER_CONFIG` exist,
   the per-user file is read unless one of them is given,
   and no CI detection changes that.
- Trust:
   "Go with the logic in our cli-git",
   which replaces the restricted-mode design proposed by the research.

#### Trust, following cli-git

Read from `doc/decision/cli-git-policies-platform.md` and
`package/git-policy/cli/src/allowed-worktree-dirs.ts` on 2026-09-17.

- First evaluation of a repository configuration always requires explicit trust.
  meow gets `trust`,
   `trust --yes`,
   `untrust`,
   and `status` subcommands,
   and they inspect or recover trust without evaluating the configuration first.
- An untrusted or changed configuration blocks every command that loads configuration,
   and meow does not fall back to a built-in subset.
  Commands that need no configuration keep working.
  This is stricter than the restricted mode the research recommended:
   listing a task graph loads configuration,
   so it blocks until the configuration is trusted.
- The blocking diagnostic names the affected path and both recovery commands,
   the interactive one and the noninteractive `--yes` form,
   as a JSON object like every other line meow writes.
- Trust identity is the complete pair of filesystem ID and canonical configuration path,
   and the registry path encodes that identity reversibly rather than hashing it.
- Trust compares exact bytes,
   never a content hash,
   and meow evaluates the stored snapshot rather than the live file,
   which closes the compare-then-swap window between checking and evaluating.
  This is the one place meow does not use its cache key hash.
- A later byte change blocks until re-trust.
- The trust registry lives under the operating-system account home,
   not under a path derived from `HOME`,
   `XDG_STATE_HOME`,
   or `APPDATA`,
   because those are environment values a repository can influence.
  Per-user configuration discovery still uses the XDG variables,
   since that file is the user's own and is not a trust record.
  This supersedes the research's `$XDG_STATE_HOME/meow` proposal.
- Registry replacement is atomic,
   rejects symlinks in its ancestry,
   requires current-account ownership,
   and uses private modes.
- Trusted configuration is not sandboxed:
   it evaluates with meow's full authority,
   which is why trust is explicit.
- A baked-in allowlist exempts third-party tool caches,
   resolved through `realpath` with segment-aware containment
   so `/a/b` never matches `/a/bc`,
   and entries missing from the machine drop out of the check.
  meow reuses that shape for the prefix acceptances in the per-user file.
- `--no-user-config` skips the per-user file only.
  It is not a kill switch for repository configuration discovery,
   which cli-git deliberately does not have.

#### Recommended shape

- Discovery,
   in order:
   `--user-config <path>`,
   then `MEOW_CONFIG`,
   then `$XDG_CONFIG_HOME/meow/meow.hcl` when that value is absolute,
   then `$HOME/.config/meow/meow.hcl`.
  `--no-user-config` and `MEOW_NO_USER_CONFIG` skip all four and record an `Absent` read-set entry.
  No `XDG_CONFIG_DIRS` search list.
- Resolution is hand-written over `std::env::var_os`,
   following the policy already in `package/forbidden-strings/cli/src/runtime_cache/path.rs:116-140`,
   so no dependency is added (rule `RCI`).
- Layering:
   every schema attribute declares its scope as `user_only`,
   `repository_only`,
   or `both` with a stated winner.
  A value in a layer that may not set it is ignored,
   with a JSON diagnostic naming the file,
   the byte span,
   the attribute,
   and the reason.
- Outside-the-repository writes:
   a repository declares content and names no outside destination,
   while the per-user file supplies the destination and the consent,
   optionally by path prefix.
  An unmatched proposal is reported rather than silently skipped.
  For the JetBrains Harper case this keeps the rule names in the repository
   and the options directory in the per-user file.
- Trust:
   an unseen repository starts in a restricted mode where meow parses,
   reports its task graph,
   and answers queries,
   but refuses to run tasks,
   write outside the tree,
   or evaluate impure functions.
  Trust is keyed by canonical repository root and recorded under `$XDG_STATE_HOME/meow`,
   never written back into the user's HCL.
- Cache and watching:
   the whole per-user file digest keys configuration evaluation,
   while task entries carry consumed-subset fingerprints naming block address,
   attribute,
   and value digest.
  The per-user configuration's containing directory is watched,
   never the file inode.
- Daemons:
   one per canonical repository root,
   with a socket under `$XDG_RUNTIME_DIR` named by a digest of that root;
   reload carries a configuration generation number and pins in-flight tasks;
   a malformed per-user file keeps the last good snapshot and emits a span diagnostic.

#### Measured evidence

- This machine:
   `XDG_CONFIG_HOME` is unset,
   `XDG_CONFIG_DIRS` holds three entries of which two are root-owned,
   `/home` is a symlink to `var/home`,
   and `.config` holds 159 entries against three home dotfiles.
  Verified again from this session before merging.
- OpenTofu 1.12.6 never reaches `$HOME/.config/opentofu/tofurc` when `XDG_CONFIG_HOME` is unset,
   which is this machine's state,
   so copying its dotfile-first shape would make the obvious location a silent no-op.
- dprint 0.57.4 finds both `$XDG_CONFIG_HOME` and the `$HOME/.config` fallback,
   the opposite behavior,
   and treats its global configuration as a fallback rather than a merge layer.
- mise merges environment values per key with the repository winning a shared key,
   lists and runs per-user tasks inside a repository,
   and refuses `trusted_config_paths` from a non-global configuration.
- A malformed per-user file gives git exit 128 and blocks even `git init`,
   gives mise exit 1 with a span,
   and is never read by dprint.
- Watching the file inode dies after one same-directory temp-plus-rename save,
   which is this repository's own write pattern,
   while watching the directory survives through `MOVED_TO`;
   an in-place append was the positive control that reached both.
- The repository owns no `dirs`,
   `etcetera`,
   or `directories` crate in any manifest,
   and its only home-directory use in file-enforcer is a `??` fallback with no absolute or empty check
   (`package/dev-script/file-enforcer/src/jetbrains/options-dir.ts:334-338`).

#### Correction this research forces

"Cache key hash after the collision findings" and "HCL tooling" record that the cache key includes both the
repository and per-user configuration digests.
Under the recommended two-level shape that holds for the configuration evaluation key,
while a task entry's key carries only the per-user values that task consumed.
The wording is corrected here rather than in those sections,
which describe the coarser shape the research replaced.

### vm-builder migration

Research:
[`vm-builder-exec.md`](monorepo-manager-route-research/vm-builder-exec.md),
2026-09-17,
covering the four migration items the all-Rust decision record lists as consequences.

#### Answers on 2026-09-17

- Shared process owner:
   "No shared owner".
  vm-builder owns its runner,
   direct spawning stays the repository's pattern,
   and no `module-process-run` package is created.
- Timing:
   "With the rewrite".
  The `exec` replacement lands as part of file-enforcer's deletion rather than ahead of it,
   so the rewrite's parity surface keeps that export until then.
- Speed gate:
   "No speed gate".
  The rewrite keeps its byte-identical output gate only,
   and the `file-enforcer-perf` fixture retires with the TypeScript implementation;
   no Rust benchmark is written for it.
  Recorded consequence:
   the rewrite ships without speed evidence,
   so any claim that it is faster stays unmeasured.
- Root `mise.toml`:
   "Correct the record",
   so file-enforcer keeps generating it until Mise is removed.
- The `prefer-readonly-parameter-type` fixture:
   "New fixture package".
  A new `package/test-fixture/prefer-readonly-parameter-type` holds a frozen copy of `apply-plan.ts`
   and really depends on `@monochromatic-dev/module-toml-edit`,
   which keeps a genuine cross-package chain for the analyzer
   and keeps that module a dependency target after file-enforcer leaves.
  It also removes a fragility that exists today,
   since `workspace-source-effect.unit.test.ts:67-70` finds functions by searching source text.
- Nothing from this research is left open.

#### Recommended shape

- `exec`:
   vm-builder owns a private `package/dev-script/vm-builder/src/process.ts`
   exporting `runInherited`,
   which is the inherit-stdio helper it already duplicates three times,
   and `runCaptured` over `nano-spawn`.
  Its manifest drops `@monochromatic-dev/dev-script-file-enforcer` and gains `nano-spawn` from the catalog,
   so the dependency count is unchanged
   and 72 file-enforcer source files leave vm-builder's type-check program.
- `nano-spawn` rather than `node:util.promisify(execFile)`,
   because it reproduces today's thrown message including exit code and stderr,
   which four probe `catch` blocks render to the user.
- `file-enforcer-perf`:
   keep the corpus generator and write a Rust benchmark in meow's worktree,
   following `package/forbidden-strings/core.bench`.
- The `prefer-readonly-parameter-type` fixture read:
   vendor `apply-plan.ts` into a new `package/test-fixture/prefer-readonly-parameter-type`
   that really depends on `@monochromatic-dev/module-toml-edit`,
   because after file-enforcer is deleted that module has no other non-fuzz TypeScript consumer.
- `sync:files`:
   both tasks disappear with `mise.toml` when Mise is removed,
   and watch mode is already assigned to the daemon watcher.

#### Deciding evidence

- The platform-aware `exec` form has no callers:
   every call site passes `{ cmd, args }`,
   and `platformCommands` appears only in `exec.ts`,
   its unit test,
   and the README (verified again on merging).
  So `exec.ts` and `platform/evaluate-predicate.ts` are mostly retired behavior rather than behavior to port.
- `exec` has no successor inside meow:
   its two internal callers leave the port to Meta Package Manager and to the daemon watcher,
   so vm-builder is the only consumer needing a replacement.
- vm-builder already owns three identical `run()` helpers
   (`sign-and-push.ts:63`,
   `build-and-import.ts:137`,
   `import.ts:98`),
   which is why rule `RCI` puts the owner there rather than in a new shared package.
- Direct spawning is the incumbent pattern:
   123 TypeScript files import `nano-spawn` directly,
   63 of them outside tests,
   and the root `file-enforcer.config.ts` spawns directly rather than through `exec`.
- Of 14 `exec` calls,
   one consumes stdout and four are probes whose only signal is the thrown error.
- vm-builder has no tests and no CI job,
   so the parity tests in the ledger are new work rather than existing coverage.

#### Ranking

`vm-builder owns it` > a new shared process module > relocating `exec.ts` verbatim >
attaching it to `task-util` > shelling out to meow's CLI > meow's RPC.

- Owning it over a shared module:
   rule `RCI` says extend a present boundary,
   and a new published package plus a workspace edge for one consumer is the larger change.
- A shared module over relocating `exec.ts`:
   both add a package,
   but the relocation carries 358 lines of platform dispatch no call site uses.
- Relocating over `task-util`:
   `task-util`'s declared scope is Mise orchestration,
   and its `command.ts` runs an argument parser at module scope.
- `task-util` over meow's CLI:
   shelling out makes a Node dev script depend on a Rust binary,
   its trust registry,
   and JSON line framing.
- meow's CLI over meow's RPC:
   RPC also needs the daemon running and has no method for an ad-hoc command.
  Both are disqualified rather than merely last.

#### Defects this research found

- `package/dev-script/file-enforcer/README.md:134-197` documents `exec([...])` and
   `exec('mise', ['use', 'git'])`,
   neither of which the implementation accepts,
   and `README.md:290` imports a subpath the manifest does not export.
- `workspace-source-effect.unit.test.ts:67-70` locates functions by searching source text for
   `function <name>`,
   so an unrelated rename breaks it silently,
   today,
   before any migration.
- The root `mise.toml` was recorded as hand-maintained in the all-Rust decision record,
   while `file-enforcer.config.ts:718-721` still generates it and `mise.toml:1` carries the generated header.
  Verified on merging and asked rather than corrected unilaterally;
   the user answered "Correct the record" on 2026-09-17,
   so generation stands until Mise is removed.

#### Transition

- The `exec` replacement and the fixture read depend on nothing from meow and can land at any time;
   landing the `exec` one early removes a public export from the rewrite's parity surface.
- The performance fixture keeps running against the TypeScript implementation,
   because it is the only speed baseline that exists.
- `sync:files` stays untouched until Mise goes.
- No coexistence protocol is needed,
   because meow is built in its own worktree.

### Platform probes

Research:
[`probe-platforms.md`](monorepo-manager-route-research/probe-platforms.md),
2026-09-17,
using the daemon skeleton with `gxhash` 3.5.0 on nightly-2026-09-12.
Spot-checked the same day:
`file` output for every built binary,
the aarch64 musl target spec,
the installed `qemu-user-static-aarch64` 10.2.2 package,
and the `rustix` auxv source.

#### Build matrix

- `x86_64-unknown-linux-gnu`:
   1,983,504 bytes on the host,
   1,980,536 bytes in a Debian bookworm container,
   PIE.
- `x86_64-unknown-linux-musl`:
   2,098,120 bytes,
   static-pie.
- `aarch64-unknown-linux-gnu`:
   1,709,080 bytes,
   PIE,
   built in Debian bookworm with `aarch64-linux-gnu-gcc` and the `libc6-dev-arm64-cross` 2.36 sysroot.
- `aarch64-unknown-linux-musl`:
   1,744,728 bytes,
   statically linked but not position-independent,
   because the built-in target spec lacks `static-position-independent-executables`
   (checked with `rustc -Z unstable-options --print target-spec-json`),
   so the binary gets no ASLR.
  A custom target spec copying that target with `static-position-independent-executables` set,
   built with `-Z build-std=std,panic_abort -Z json-target-spec` and `rust-lld`,
   produced a static-pie binary (`readelf` type `DYN`)
   that passed the CPU check and hashed under QEMU;
   it needed musl's self-contained startup objects linked under the custom target name in the rustup sysroot.
- Both glibc builds need glibc 2.34:
   they ran on UBI 9 (2.34) and failed on Fedora 34 (2.33) with "version `GLIBC_2.34' not found".
  The host-built binary also carries weak `GLIBC_2.39` references (`pidfd_spawnp`, `pidfd_getpid`)
   and prints "weak version `GLIBC_2.39' not found" on glibc 2.34 and 2.35;
   the bookworm-built binaries print nothing.
- Every aarch64 skeleton exited 0 under QEMU with the same `gxhash128` output as x86_64.

#### Missing CPU capabilities

- Without the check,
   the skeleton exits 132 (SIGILL) on x86_64 `Nehalem` and `qemu64`
   and on a `cortex-a72` patched to drop the crypto extension,
   for musl and glibc builds.
- The prototype check,
   compiled under `#![forbid(unsafe_code)]`,
   prints a diagnostic naming the missing capability and stating there is no fallback,
   then exits 3;
   with AES present it hashes and exits 0.
- `is_x86_feature_detected!("aes")` and `is_aarch64_feature_detected!("aes")` printed `true` on CPUs without AES,
   confirming they cannot perform the check.

#### gxhash issue #111

- Not reproduced on aarch64 Linux under QEMU:
   no panic for `gxhash64`,
   `gxhash128`,
   or `GxHasher` over input lengths 0 to 67,
   or the issue's reproducer,
   in debug-assertion builds on nightly and Rust 1.84.0,
   while a deliberate overlapping `copy_nonoverlapping` did panic with the issue's message.
- `gxhash` still reads past the end of short inputs.

#### Settled from the probes

- The aarch64 check also requires `HWCAP_PMULL`,
   because Rust's aarch64 `aes` target feature implies PMULL,
   so a `+aes` build may use it
   (`std_detect/src/detect/arch/aarch64.rs:117-118`).
- `rustix` is built with `use-libc-auxv`:
   its default auxv reader unwraps and aborts when both `PR_GET_AUXV` and `/proc/self/auxv` fail
   (`src/backend/linux_raw/param/auxv.rs:267-316` in `rustix` 1.1.4),
   while `use-libc-auxv` reads libc's `getauxval`.
- Probe containers that run QEMU pass `--init`:
   without it a SIGILL run hung because QEMU ran as PID 1.

### Managed file editing

Research:
[`rust-structured-edits.md`](monorepo-manager-route-research/rust-structured-edits.md),
2026-09-17.
Scores come from `structured-edits/probe/scores.md` in the session scratchpad;
the `toml_edit` wrapper's 21 passes were recounted there.

#### Chosen editors

- TOML:
   `toml_edit` behind a repository wrapper passed all 21 cases,
   while `toml_edit` as-is passed 12:
   setting a value dropped its same-line comment,
   removing an array element moved a comment onto the wrong element,
   and deleting a key deleted a header comment separated by a blank line.
  `taplo` 0.14.0 passed 6 and rejects TOML 1.1 forms the current editor already writes.
- JSONC:
   the `jsonc-parser` 0.33.2 CST behind a wrapper passed all 16 cases,
   with no dependencies,
   adding 134,536 bytes.
- XML:
   `roxmltree` with byte-range splicing passed all 9 cases,
   fails malformed files with line and column,
   and adds 77,072 bytes;
   `xot`,
   `xmltree`,
   and `xml-rs` wrote raw tabs or newlines into attribute values.
- The repository's TypeScript editors fall short:
   `module-toml-edit` passed 15 of 21,
   `module-jsonc-edit` 7 of 16,
   and file-enforcer's XML splicing 4 of 9.

#### Requirements after the user's answers

- Comments must survive edits and never move to a different node
   ("Managed file edits").
- "'untouched bytes identical' rule and preserve CRLF per file - no need."
  (user,
   2026-09-17):
   formatting outside edited nodes may change,
   and line endings may be normalized.
- Settled by those answers:
  - JSONC and TOML files may be reformatted canonically on write,
     so files `toml_edit` cannot round-trip byte for byte are rewritten,
     not rejected.
  - Comments directly above a deleted node leave with it;
     leaving them behind would attach them to a different node.
  - Owned XML options are edited at attribute level,
     so comments inside an entry survive.
  - JSONC editing is a general capability,
     since today's file-enforcer edits no JSONC and meow serves other repositories.

#### Research notes

- Cargo builds hit the `/tmp` quota,
   so that research built under `~/temp/agent/structured-edits-target-2026-09-16/`,
   removed on 2026-09-17.
- The tool quirks the research found are documented in `doc/troubleshooting/`,
   indexed under "Comment-preserving structured edits in Rust":
   `toml-edit-comment-loss.md`,
   `taplo-toml-1-1-inline-table-trailing-comma.md`,
   `jsonc-parser-json5-defaults.md`,
   `json-five-unterminated-block-comment.md`,
   `biome-json-crates-exact-pins.md`,
   and `xml-attribute-whitespace-serialization.md`.
- Those docs corrected research readings:
  - `jsonc-parser`'s default options are a documented loose set,
     not JSON5:
     they reject `Infinity`,
     `NaN`,
     `.5`,
     and `5.`,
     and accept missing commas and names like `a-b`.
  - The `1.0` rewrite on an equal-value set came from the probe's `serde_json` equality guard;
     `set_value` always replaces.
  - `json-five` ends every block comment span on the closing `/`,
     so a plain parse and print drops that `/`.
  - The Biome JSON crates build with three exact pins
     (`biome_rowan`,
     `biome_parser`,
     and `biome_unicode_table` at `=0.5.7`),
     not seven.

#### JSONC wrapper rules from the troubleshooting docs

Adopted because the "Managed file edits" requirements admit one answer;
the evidence and tradeoffs are in `doc/troubleshooting/jsonc-parser-json5-defaults.md`,
"Verified workarounds".

- Every managed JSONC file is parsed through one function holding JSONC-only options,
   rejecting scalar and empty roots,
   since any direct `ParseOptions::default()` reintroduces the loose set.
- The equal-value guard compares integers exactly;
   the research wrapper compared every number as `f64`
   and skipped a real change between integers above 2^53.
- Strict JSON files are validated with `parse_to_value` before building the CST,
   because `CstRootNode::parse` ignores `allow_comments: false` in `jsonc-parser` 0.33.2
   (upstream filing tracked in #549).
- Container writes that change replace the whole subtree through `set_value`,
   dropping comments inside it,
   so the wrapper sets leaf paths.

#### Build and format answers on 2026-09-17

- aarch64 musl ships from the custom static-pie target,
   so all four binaries get ASLR.
- Local glibc builds use the host (Fedora 44):
   "we don't directly ship the build outputs on local dev machines to GitHub Releases."
  Release binaries follow the repository's existing pipeline,
   `.github/workflows/cargo-publish.yml`,
   which builds natively on `ubuntu-latest` and `ubuntu-24.04-arm`
   and installs `musl-tools` for musl targets;
   the minimum glibc of binaries built there is unmeasured.
- New TOML inline tables use the TOML 1.1 style `{ a = 1, }`,
   an answer given on a false premise:
   the question said this matches today's Cargo manifests,
   but no tracked `Cargo.toml` contains an inline-table trailing comma
   (`git ls-files -z '*Cargo.toml' | xargs --null rg --multiline --count ',\s*\}'` matched 0 files on 2026-09-17);
   the form comes from `module-toml-edit` (`package/module/toml-edit/src/toml-set.unit.test.ts:484`),
   and taplo 0.14 rejects it.
  Asked again with the corrected premise on 2026-09-17,
   the user kept the TOML 1.1 style.
- file-enforcer keeps generating the root `mise.toml` for now.
  Read together with "hand-maintained now",
   meow never generates `mise.toml`,
   and Mise and its file leave when meow takes over.

### Repository-owned gxhash

Research:
[`gxhash-owned.md`](monorepo-manager-route-research/gxhash-owned.md),
2026-09-17,
with a prototype reimplementation,
differential test,
benchmarks,
and collision reproducer under `gxhash/lab/` in the session scratchpad.

#### Findings

- A prototype reimplementation written from the algorithm notes,
   with no raw pointer reads,
   matched `gxhash` 3.5.0 on 97,545 input and seed pairs on x86_64,
   on aarch64 under QEMU,
   and against the upstream `hybrid` build.
- Undefined behavior in 3.5.0:
   Miri stops on inputs of 1 to 16 bytes with
   "attempting to access 16 bytes,
   but got alloc311 which is only 8 bytes from the end of the allocation".
  The music player always hashes at least 24 bytes;
   89 git-tracked files are under 16 bytes,
   so meow would hit it.
  The prototype runs clean under Miri.
- One-byte collisions in `gxhash128`,
   reproduced independently on 2026-09-17 with the lab's `onebyte` binary
   (`BASE=random SEEDS=0,1,987654321 onebyte 4096 gxhash128 xxh3_128`):
   among all 1,044,480 single-byte variants of one random 4096-byte input,
   `gxhash128` produced 1 full 128-bit collision
   (bytes 354 and 877,
   both in block 6 of a lane group),
   equal under all three seeds,
   while XXH3-128 produced 0.
  For a content cache,
   two different one-byte edits of the same file can share a key.
- Issue #111 did not reproduce on aarch64 Linux in either investigation.
- Upstream reasons against runtime detection and stable VAES are outdated:
   safe `#[target_feature]` functions are stable since Rust 1.86 and VAES intrinsics since 1.89.
- Upstream has had no commits since 2025-05-18 and no release since 2025-03-12;
   #118,
   the inline-assembly read fix,
   is merged but unreleased.
- Benchmarks,
   x86_64 in `podman --memory=2g --cpus=2`,
   with run-to-run bands of 19.3% on fingerprint material and 23.6% on files:
   on music-player fingerprint material `gxhash128` was 31% to 37% faster than XXH3-128,
   beyond the band;
   on the 8,093 git-tracked files `gxhash` reached 45 to 47 GiB/s,
   where memory access rather than the hash limits throughput;
   `gxhash64` and `gxhash128` showed no measurable difference on file contents.

#### Settled from the findings

- Ownership shape:
   dirty-room reimplementation crediting ogxd,
   not a fork:
   a fork starts from 56 `unsafe` uses and undocumented items that rules `MXR` and `RDC` would force rewriting anyway,
   and upstream is idle.
- License:
   `LGPL-3.0-or-later`,
   the repository's license for Rust crates
   (`package/forbidden-strings/cli/Cargo.toml:21`),
   plus the upstream MIT copyright notice,
   "Copyright (c) 2023 Olivier Giniaux",
   for the derived code.
- API:
   one-shot hashing,
   later corrected on 2026-09-17 (see "Cache key hash after the collision findings"):
   meow hashes multi-GB outputs,
   so it needs streaming hashing;
   `Hasher` and `HashMap` support are left out because nothing in the repository uses them.
- The undefined behavior and the one-byte collisions go through the `troubleshooting-doc` skill,
   whose upstream filing audit decides whether anything is offered upstream.

#### Upstream reports of the collisions

Read 2026-09-17 with `gh issue view`:

- #83,
   opened 2024-06-02 and open,
   "Hash has arbitrary seed-independent multicollisions,
   is not DoS resistant".
  ogxd replied "Let's see if we can improve DoS resistance without compromising performance",
   later clarified the README security section (2024-11-05),
   and after a comment that "`compress_all` is completely independent from the seed"
   wrote on 2024-12-10:
   "Mixing the seed at the start instead of the end should significantly improve DoS resistance.
   I'm currently (slowly) exploring this path."
- #124,
   opened 2025-11-14 and open,
   "seems to be failing some tests on smhasher3";
   ogxd replied "I fail to see why gxhash would have to comply to all benchmarks in the www",
   and the reporter noted the README claims every SMHasher test passes.
- The README still states "GxHash passes all [SMHasher](https://github.com/rurban/smhasher) tests"
   (`README.md:19` in the 2026-09-16 clone),
   which names the original SMHasher rather than SMHasher3.

#### Answers on 2026-09-17

- CPU capabilities:
   `+aes` target-feature builds with the startup check,
   not a runtime detection token.
- The owned hash crate is published on crates.io;
   the name `gxhash` is taken,
   so naming research comes before publishing.
- aarch64 benchmarks run on `ssh m1`,
   once the user powers it on;
   the user asked to be told to turn it on only when it is strongly needed.
- The cache key output question was answered with questions about upstream reports,
   answered in "Upstream reports of the collisions",
   and is asked again.

#### Cache key hash after the collision findings

Answered by the user on 2026-09-17,
to the repeated cache key output question:
"Pick some other hash that has hardware acceleration on x86 and ARM instead."

- meow's cache keys stop using `gxhash`;
   the user delegated choosing a replacement with hardware acceleration on x86 and ARM.
- The selection runs through the choosing-technology workflow,
   carrying over the 128-bit key width,
   stable output for persisted keys,
   static musl builds on both architectures,
   and a clear warning on CPUs lacking required capabilities.
- Workload correction,
   raised by the user and measured on 2026-09-17:
   meow hashes huge inputs,
   because every declared output gets a content hash
   and dependency output hashes feed downstream keys.
  `package/dev-script/vm-builder/output/qcow2/disk.qcow2` is 3,966,238,720 bytes
   and music-player debug binaries are 520 to 663 MB each,
   while the largest tracked source file is 22.9 MB.
  Tracked file sizes on 2026-09-17:
   p50 4,043 bytes,
   p90 19,244,
   p99 238,057;
   files under 1 KiB are 22.0% of files but 0.39% of bytes,
   and files of 64 KiB and above hold 66.15% of bytes.
  The user added that even 20 KiB files are not small inputs:
   fingerprint-sized strings under 1 KiB are not the workload that decides the hash.
  The hash therefore needs streaming with chunking-independent output and bounded memory,
   and its throughput and multi-core scaling on multi-GB inputs matter;
   the running vet was told.
- Superseded for meow:
   the `gxhash128` key choice and the `+aes`-specific build and startup check,
   whose target features and check follow the chosen hash.
- Music player:
   the user decided on 2026-09-17 that it must switch away from `gxhash`,
   tracked in issue #545;
   this design work does not touch the music player.

#### Cache key hash vet result

Vet:
[`doc/audit/tech-meow-cache-key-hash-vet-2026-09-17.md`](../audit/tech-meow-cache-key-hash-vet-2026-09-17.md),
finished 2026-09-17.
Recorded as adopted the same day after the user confirmed the SIMD reading ("SIMD counts"),
then withdrawn:
the user had not been shown the brief,
and their answers to it changed two premises of the vet.

- "non-crypto isn't a requirement":
   the vet scoped out cryptographic hashes,
   measuring BLAKE3,
   SHA-256,
   and AES-CMAC only as controls.
- "Shipping x86-64-v3 and x86-64-v4 as separate builds is fine":
   the vet decided `twox-hash` over `xxhash-rust` mainly on the default-build speed and run-time kernel selection,
   while in an `x86-64-v3` build the two were within band,
   and no `x86-64-v4` build was measured.

The selection reopens with both changes;
the measurements below stay valid evidence for it.
The user then chose speed-first weights and made `x86-64-v4` the only release-blocking x86-64 build
("Hashing" and "Platforms and builds"),
and the re-run vet finished the same day ("Cache key hash re-run vet result").

- Recommended:
   `twox-hash` 2.1.4,
   `XxHash3_128` (XXH3-128).
- The user's phrase "hardware acceleration" has two readings,
   and the recommendation relies on the second:
  - R1,
     dedicated instructions such as AES-NI,
     SHA extensions,
     or the Arm Cryptography Extension:
     no non-cryptographic library passes the hard gates.
    `rotohash-rs` meets R1 but has no streaming API and no stability statement,
     and the R1 controls are slow on file contents:
     SHA-256 1.86 GiB/s,
     AES-CMAC 1.43 GiB/s.
  - R2,
     designed SIMD kernels
     (AVX2 or SSE2 on x86_64 and NEON on aarch64):
     four finalists.
- Ranking under R2:
   `twox-hash` (68.5 of 76) > `xxhash-rust` 0.8.18 (58) > `hashcrew` 0.3.0 (49) > `highway` 1.3.0 (41);
   `twox-hash` led all 94 one-at-a-time sensitivity tests.
- Deciding evidence,
   medians of five container runs on one Zen 4 host:
  - One call per tracked file in a default build:
     `twox-hash` 49.3 GiB/s,
     `xxhash-rust` 29.1,
     `hashcrew` 16.0,
     `highway` 12.4.
    `twox-hash` picks AVX2 at run time (`src/xxhash3/large.rs:105-116` at `v2.1.4`),
     while `xxhash-rust` picks its kernel with compile-time `cfg(target_feature)` (`src/xxh3.rs:16-28`),
     so it runs SSE2 unless meow raises the build target.
  - In an `x86-64-v3` build the three XXH3 crates are within 5% of each other.
  - Cold multi-GB reads are I/O-limited for every finalist;
     warm multi-GB streaming is limited by the page-cache copy (13.7 to 15.3 GiB/s),
     where `twox-hash` streams at 11.2 to 11.5 GiB/s.
  - Peak memory stayed near 3.3 MiB for the 3.97 GB image;
     splitting one file across two threads was slower than one thread.
  - Fingerprint-sized inputs decided no adjacent pair.
- Correctness checks:
   0 one-byte collisions for every finalist at 8 to 4,096 bytes under three seeds,
   with `gxhash` colliding as the positive control;
   0 mismatches against the C reference implementations on x86_64 and on aarch64 under QEMU;
   0 streaming against one-shot mismatches over 8,548 chunkings;
   each check's control failed as expected.
- An `ssh m1` run could not change the ranking:
   aarch64 speed carries weight 1,
   and the order holds at both ends of its range and at weight 5.
  Per the user's instruction to recommend powering on the m1 only when strongly needed,
   it is not recommended for this choice.
- Raising quality evidence beyond the collision gate to weight 5 only ties third place,
   so it does not bear on the choice.

Usage rules the vet derived for `twox-hash`,
kept for the reopened selection:

- Dependency:
   `twox-hash = { version = "=2.1.4", default-features = false, features = ["std", "xxhash3_128"] }`,
   pinned because persisted keys need identical output across upgrades;
   `std` enables run-time AVX2 and NEON detection,
   and the default features add `rand`.
- Golden output vectors in meow's own tests,
   such as the empty input `99aa06d3014798d86001c324468d497f`,
   so an output change fails a test.
- Default seed only,
   with salts and domain tags in the hashed bytes,
   because SMHasher3 finds full-width XXH3 collisions when seeds vary.
- A fresh `XxHash3_128::new()` per file,
   reads of 64 KiB to 1 MiB,
   files hashed in parallel rather than one file split across threads,
   and the `u128` serialized in one fixed byte order.
- The default x86_64 and aarch64 targets already guarantee SSE2 and NEON,
   so the default build needs no startup CPU check;
   a raised target such as `x86-64-v3` would bring back the check from "Missing CPU capabilities".
- Known risks:
   XXH3-128 fails 36 SMHasher3 tests at partial width or with varying seeds,
   none a full-width collision at a fixed seed,
   and `twox-hash` has one maintainer.

#### Cache key hash re-run vet result

Vet:
[`tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md`](../audit/tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md),
finished 2026-09-17 under the changed premises,
with a screening appendix listing every scanned crate.
Answering the brief,
the user said "I just powered on my m1 mac.
Please measure.",
so the aarch64 numbers that decide `twox-hash` against `rscrypto` were measured there,
and the user then answered "Accept twox-hash":
[`doc/decision/monorepo-manager-cache-key-hash.md`](../decision/monorepo-manager-cache-key-hash.md).

- Recommended:
   `twox-hash` 2.1.4 XXH3-128,
   one-shot for in-memory files and its streaming state for larger inputs.
- Ranking (scores out of 92):
   `twox-hash` 85 > `rscrypto` 83.5 > `xxhash-rust` 82 > `hashcrew` 74 > `highway` 46.
  `rscrypto` is a new finalist that the widened scope brought in.
- Cryptographic and AES-based candidates lost the weight-5 criteria by factors of 8 to 15 on both architectures,
   so the function is settled by measurement and only the crate is close.
- Whole tracked-file corpus on an `x86-64-v4` build,
   medians of five container runs in GiB/s:
   `hashcrew` 47.74,
   `rscrypto` 47.45,
   `twox-hash` 47.42,
   `xxhash-rust` 47.03,
   `highway` 15.28,
   AES-PMAC 5.89,
   BLAKE3 with its C kernels 4.64,
   pure-Rust BLAKE3 4.33,
   SHA-256 with SHA-NI 2.24 to 2.27,
   AES-CMAC 1.78,
   KangarooTwelve 0.92.
  Run-to-run band on the unchanged build:
   median 1.9%,
   p90 5.4%.
- aarch64 was first rated from `rscrypto`'s public CI artifacts on an AWS `c9g.2xlarge`
   (XXH3-128 24.53 to 24.59 GiB/s,
   SHA-256 1.687 to 1.957,
   single-threaded BLAKE3 1.613),
   then measured on the user's m1 on 2026-09-17,
   which supersedes those borrowed numbers.
  Measured on a MacBook Air M1,
   `-Ctarget-cpu=generic` so only NEON is used,
   medians of five runs on the same corpus bytes,
   one call per tracked file,
   in GiB/s:
   `hashcrew` 30.910,
   `twox-hash` 30.728,
   `xxhash-rust` 29.859,
   `rscrypto` 29.736,
   `highway` 7.172,
   with SHA-256 at 2.142 and BLAKE3 at 1.515.
  Streaming per file separates them further:
   `hashcrew` 28.220,
   `twox-hash` 27.508,
   `rscrypto` 23.755,
   `xxhash-rust` 21.273.
  Bands were tighter than on the x86 host:
   median 0.3% and p90 1.9% on that build,
   with `pmset` reporting no thermal or speed limit across all 100 probes.
- After the measurement:
   `twox-hash` 85 of 92 > `rscrypto` 83.5 > `hashcrew` 79 > `xxhash-rust` 77 > `highway` 36.
  `hashcrew` and `xxhash-rust` swapped third and fourth,
   because `xxhash-rust` is fastest of the five when streaming on x86 and slowest of the four XXH3 crates
   when streaming on the M1.
  Of 96 sensitivity tests none changes the winner now,
   where before the measurement one did;
   the closest margin for the top pair is 0.50 points.
- Remaining proxy:
   the measurement is Apple Firestorm on Darwin,
   while the release-blocking aarch64 targets are Linux.
  A full rating step of transfer error would hand first place to `rscrypto`,
   which needs `twox-hash`'s true ratio on a Linux aarch64 core to fall below 0.90.
- Digests match across machines:
   60 of 60 accumulated digest lines are byte identical between the `x86-64-v4` and m1 builds,
   with a control that matched none.
  Correctness on the m1 repeated the x86 results:
   82,200 C-reference checks with 0 mismatches against a control that failed all of them,
   0 collisions in 60 one-byte keyset results,
   and 0 streaming failures against a control with 196,282.
- Multi-GB inputs:
   warm reads resolve the finalists (XXH3 13.1 to 15.4 GiB/s,
   BLAKE3 5.4,
   SHA-256 2.0),
   cold reads are storage-bound at 0.33 to 1.59 for everything,
   and peak memory stayed near 3.5 MB on the 3.97 GB input.
- Adjacent pairs:
  - `twox-hash` over `rscrypto`:
     tied on all three weight-5 criteria,
     decided by audit surface
     (1,533 lines on the used path against 7,496 inside a 177,154-line crate)
     and maintenance (11 years and 53.4M recent downloads against 4 months and about 1,100).
    Margin 1.5 points.
  - `rscrypto` over `xxhash-rust`:
     run-time dispatch wins the non-blocking builds,
     against `xxhash-rust`'s smaller audit surface and longer history.
    Margin 1.5 points.
  - `xxhash-rust` over `hashcrew`:
     a measured aarch64 number,
     1.8 times the short-input speed,
     and better non-blocking-build behavior.
  - `hashcrew` over `highway`:
     `highway` reaches 0.28 of XXH3 on whole files;
     its clean SMHasher3 pass is worth one point.
    Margin 28 points.
- Correctness:
   zero one-byte collisions at every width for every finalist,
   with `gxhash` colliding as the positive control;
   23 of 24 functions had zero streaming failures against 204,816 in the control run;
   4,261 reference-equality checks passed with a control that failed all of them.
  `rscrypto` received equal-depth validation,
   including Miri with strict provenance and a 600-second differential fuzz run
   (46,844,896 executions,
   no finding).
- Discovery saturated 36 crates.io queries,
   scanned 5,092 crates,
   and reviewed 118 by hand.
  Exits include `blake3` (its aarch64 build needs a C cross compiler,
   and its `pure` build has no aarch64 vector path),
   `graviola` SHA-512 (no aarch64 kernel),
   and `xoodyak` (per-message absorb,
   so 5,155 of 8,548 chunkings differ).
- The C toolchain branch closed with a measured bound instead of a user question:
   BLAKE3's C kernels buy 18% to 22%,
   pure-Rust BLAKE3 matches them here,
   and BLAKE3 still sits at about a tenth of XXH3 on whole files.
- Sensitivity:
   97 one-at-a-time tests,
   of which exactly one changes the winner:
   lowering `twox-hash`'s aarch64 rating by one step puts `rscrypto` first.
  That rating rests on upstream's M1 Max table rather than a local measurement.

Usage rules and risks if XXH3-128 is adopted:

- No collision resistance once the seed is known,
   accepted because the cache is local
   and a repository that can craft inputs already runs meow's tasks.
- 36 of 250 SMHasher3 tests fail,
   none showing a full-width collision at a tested seed;
   the practical exposure is the birthday bound at 2 to the 64th.
- The seed is fixed as part of the key format,
   the full 128 bits are kept,
   and changing either costs one cache rebuild.
- Every build raised above its target baseline runs meow's startup capability check before hashing.
- A key that must resist an adversary who knows the seed is a different decision,
   costing 8 to 15 times the throughput.

### Process model

- The user starts the daemon in its own terminal under a delegated cgroup,
   such as `systemd-run --user --scope -p Delegate=yes`,
   which keeps the terminal and Ctrl+C.
- The daemon moves itself into a leaf cgroup,
   then creates one cgroup per task below its delegated root.
- The development machine's user manager delegates `cpu`,
   `io`,
   `memory`,
   `pids`,
   and `dmem`
   (`user@1000.service/cgroup.controllers`,
   measured).
- 0.x logs to the terminal and handles only Ctrl+C.
  The TUI that comes later is that same `meow watch` process rendering itself differently,
   with interactivity added,
   not a second program ("User interface").
- That terminal is the one running `meow watch`,
   which the user keeps open but minimized;
   the working terminal runs `meow run` against the same daemon
   ("User interface").

### RPC

Reframed on 2026-09-17 by the user:
"We don't need specifically a socket or a separate client,
 ever."
and the socket is "Private plumbing for now".

- The user interface is meow's own commands ("User interface");
   the transport below is how they reach the daemon,
   not a product surface.
- Nothing about it is documented or promised to outside callers in 0.x,
   and no separate client application ships.
- It may be replaced wholesale as long as the commands keep working.
- Transport:
   a filesystem Unix socket under `$XDG_RUNTIME_DIR` with owner-only permissions,
   not an abstract socket,
   because abstract sockets carry no file permissions.
- Framing:
   newline-delimited JSON-RPC 2.0.
- Methods:
   list tasks,
   get one task,
   queue or rerun a task,
   end a task,
   pause,
   resume,
   and set priority.
- Notifications:
   task started,
   progress,
   and finished events modeled on Build Server Protocol task notifications,
   plus a per-task state snapshot modeled on Tilt's `UIResource`.
- A subscriber resumes from a sequence number,
   modeled on Watchman clocks;
   in 0.x the only subscribers are meow's own commands.
- Anyone who can connect to the socket can run tasks as the user,
   so socket permissions are the security boundary.

### Scheduler

- Every task has a priority,
   default 0;
   higher priority runs first,
   and priority changes apply in place.
- Work a person is waiting on,
   which today means anything `meow run` queues,
   takes an integer priority above background work,
   and the newest such task holds the highest priority of all
   ("User interface",
   user,
   2026-09-17).
  Giving it the machine may pause other tasks,
   including an earlier `meow run`,
   through `cgroup.freeze`,
   which carries the frozen-task risks already recorded under "Risks".
- Concurrency is `MONOCHROMATIC_JOBS` when set,
   otherwise `std::thread::available_parallelism`.
- Pause freezes a running task through `cgroup.freeze` and holds a queued task.
- For non-automatic work,
   end kills its running process tree through `cgroup.kill`,
   which also catches children that left the process group.
  `meow end` rejects automatic work in 0.x,
   without killing or changing it.
  Internal cancellation of superseded maintenance remains allowed.

### Maintenance and foreground lifecycle

Accepted in the first implementation-plan interview:

- Q1:
  finish the remaining independent source checks after one fails.
  Downstream production stays blocked;
  unrelated maintenance continues.
- Q2:
  an explicit foreground build request honors the failed check gate.
  It reports the blocker rather than implicitly bypassing it.
  Q8 settles the blocked outcome:
   report the current blocker and return nonzero,
   rather than wait for an edit to fix it.
- Q3:
  cancel superseded ordinary background builds and read-only checks,
  then pursue the latest relevant inputs.
  Cancellation is not a failing assertion result.
- Q4:
  a waiting foreground maintenance request follows the latest relevant inputs,
  rather than promising a historical input state.
- Q5:
  "Let's forbid `meow end` for automatic work for 0.x."
  The user rejected choosing a lifetime for an ended automatic concern.
  Foreground attachment does not change the maintained concern's class.
  Q7 settles Ctrl+C for an attached maintenance request:
   detach the requester while automatic work continues.
  Daemon shutdown still ends all of its children as already required.
- Q6:
  ordinary-source lint autofixing is explicit only.
  Automatic checking and managed-file enforcement retain their respective responsibilities.

No foreground bypass feature is introduced by these answers.

Further answers:

- Q9:
  after rejecting the replay-versus-effect contrast,
  the user also rejected a proposed per-definition reuse toggle:
  "Every execution has a defined set of inputs and outputs.
  If something must always re-execute,
  they can add a input of currentDateTime to the set."
  Execution caching remains universal.
  An intentionally varying declared input produces a different computation identity;
  there is no per-definition cache-off policy.
  Do not make every explicit operation unconditionally fresh or silently inject an undeclared input.
- Q10:
  "Follow priority and queue everything.
  The active invocation might be ran simutanously,
  paused,
  or whatever."
  The proposed target-wide serialization,
  forced replacement,
  and rejection policies are withdrawn.
  The ordinary priority scheduler owns distinct invocations.
  Actual conflicting writes still need correctness protection,
  not a blanket scheduling restriction based on label equality.

Q9 leaves the earlier always-on cache requirement intact.
Every execution has defined inputs and outputs;
its resolved inputs determine computation identity.
Volatile-input evaluation still needs a concrete binding and sampling contract in the implementation plan,
not an execution-cache bypass.

### Watching and affected work

- The daemon watches the entire repository.
- inotify watches are per directory:
   99,416 directories exist when only `.git` is excluded,
   against 5,480 when dependency and output directories are also excluded,
   with `max_user_watches` at 524,288
   (measured).
  The watcher excludes dependency and output directories.
- Change bursts,
   such as `pnpm install` or a branch switch,
   are coalesced before the daemon schedules affected work.
- For each change,
   the daemon builds everything affected and runs every test suite except files matching `*.expensive.*.test.*`.

### Cache

Q9 confirms rather than weakens the always-on requirement:
every execution declares inputs and outputs.
An operation intended to re-execute can include an intentionally varying input,
such as `currentDateTime`.
The resolved value participates in the normal key;
there is no cache-off flag or per-definition result-reuse toggle.

- Caching is always on.
- Entry contents,
   answered by the user on 2026-09-16:
  - Every run is cached,
     failures included:
     "Cache everything,
     because a flaky task is a user error and users should know better.
     We also provide a retry mechanism for unfixably flaky tasks."
  - Captured stdout and stderr are stored and replayed on a hit.
  - Outputs are recorded only as pointers to their locations with content hashes;
     a changed or missing output means the task runs again.
    The user first answered "We only record pointers and reflinks",
     then on 2026-09-17 dropped reflinks:
     "There's no need to restore an earlier build because builds are by definition ephermal.
     In addition,
     we consider switching git branches in place a user error."
  - Eviction uses a size cap plus a maximum age.
  - The size cap counts bytes only the cache pins (user,
     2026-09-16).
    With pointers only,
     the cache holds no output data,
     so those bytes are the plain size of its records and logs;
     the pinned-bytes research started for reflinks was stopped unfinished on 2026-09-17.
  - The default maximum age is 30 days since an entry was last used,
     matching Cargo's one-month threshold for regenerable global-cache files
     (`doc/book/src/reference/config.md`,
     "Global caches",
     in `rust-lang/cargo`).
  - Retries are declared for unfixably flaky tasks;
     a pass after a failed attempt is recorded as a pass marked flaky,
     with every attempt's logs kept.
- Measuring cache size with reflinks,
   kept as evidence for the dropped reflink design,
   probe on 2026-09-16 in a throwaway directory on the development machine's btrfs:
   an 8 MiB random file plus a `cp --reflink=always` clone reported
   `Total 16.00MiB`,
   `Exclusive 0.00B`,
   `Set shared 8.00MiB` from unprivileged `btrfs filesystem du --summarize`.
  After overwriting the original's first 1 MiB in place,
   per-file output was `clone.bin` exclusive `0.00B` shared `8.00MiB`
   and `original.bin` exclusive `1.00MiB` shared `7.00MiB`.
  So apparent size double-counts reflinked data,
   and per-file exclusive bytes miss the old extent the clone alone still pins.
- The cache key covers the task definition,
   argument vector,
   input file content hashes,
   declared environment,
   tool versions,
   the package's lockfile slice,
   dependency outputs,
   platform,
   and a salt.
- File-enforcer's staleness manifest is not reused as the task cache,
   because it checks size and modification time for sources.
- Content hashing is the source of truth.
  btrfs features only accelerate it.
- Hash:
   superseded on 2026-09-17;
   `gxhash` left after the collision findings,
   and the replacement is reopened in "Cache key hash vet result".
  Originally `gxhash` 3,
   the repository incumbent in `package/music-player/desktop-app` and `android-app/rust`,
   with `gxhash128` for cache keys
   (`src/gxhash/mod.rs:49` in `gxhash` 3.5.0;
   the music player uses `gxhash64`).
  The user chose the 128-bit width on 2026-09-16;
   a false hit needs a new key equal to a stored one,
   so its odds scale with stored entries times lookups divided by 2 to the key width.
- Cryptographic hashes were first excluded,
   reasoning that the cache is local only
   and that file-enforcer's SHA-256 uses,
   the staleness manifest (`package/dev-script/file-enforcer/src/io/staleness-hash.ts:20`)
   and skill-mirror ownership digests (`file-enforcer.config.ts:1103`),
   detect change and ownership rather than defend against crafted input.
  The user made them eligible on 2026-09-17 ("non-crypto isn't a requirement").
  A shared or remote cache would raise the weight of crafted-input resistance.
- `gxhash` constraints (historical after 2026-09-17),
   from `doc/troubleshooting/gxhash-aes-target-feature.md`:
   the build needs `-C target-feature=+aes,+sse2`,
   output is stable only within a major version,
   aarch64 debug builds can panic (upstream issue #111),
   and a CPU without AES-NI crashes with SIGILL because there is no software fallback.
- CPU check,
   written for `gxhash`'s AES requirement;
   after 2026-09-17 the same mechanism applies to every target feature a build is compiled with,
   such as AVX2 in an `x86-64-v3` build:
   the tool checks for AES and SSE2 at startup,
   before any hashing,
   and exits with a diagnostic naming the missing capability,
   explaining that the tool requires it and has no fallback;
   `doctor` reports the same check.
  `is_x86_feature_detected!("aes")` cannot perform this check in a `+aes` build:
   its macro evaluates `cfg!(target_feature = ...)` before runtime detection
   (`library/std_detect/src/detect/macros.rs:9-10` in the nightly-2026-09-12 sources,
   with `aes` declared without the cfg-check opt-out at `std_detect/src/detect/arch/x86.rs:124`),
   so it is `true` at compile time.
  The check calls `core::arch::x86_64::__cpuid(1)` directly,
   a safe function in that toolchain (`stdarch/crates/core_arch/src/x86/cpuid.rs:107`),
   and reads ECX bit 25 for AES and EDX bit 26 for SSE2,
   the bits std uses (`std_detect/src/detect/os/x86.rs:108`,
   `:118`).
  On aarch64 the same short-circuit applies,
   because `aes` is declared without the cfg-check opt-out (`std_detect/src/detect/arch/aarch64.rs:123`);
   the check reads `AT_HWCAP` bit 3,
   the bit std uses on Linux (`std_detect/src/detect/os/linux/aarch64.rs:147`),
   through the safe `rustix::param::linux_hwcap` (`src/param/auxv.rs:66` in `rustix` 1.1.4),
   and `gxhash` on aarch64 also needs `neon`.
  Exercised on 2026-09-17 under QEMU user mode
   ("Platform probes");
   an earlier note here that QEMU user mode was absent came from checking only the x86_64 binary names,
   while `qemu-user-static-aarch64` 10.2.2 was installed.
- Static probe,
   2026-09-16:
   the daemon skeleton with `gxhash128`,
   built for `x86_64-unknown-linux-musl` with `RUSTFLAGS='-C target-feature=+aes,+sse2'`
   (`allrust/skeleton-gxhash` in the session scratchpad),
   measured 2,098,120 bytes and ran its probe to exit 0;
   the XXH3-128 variant measured the same size.

### btrfs acceleration

- Reflink copies work without root,
   but the cache does not use them:
   outputs are pointers only
   ("Cache").
- `btrfs subvolume find-new` needs privileges:
   run without root on the `/var/home` subvolume,
   it fails with "Operation not permitted"
   (measured),
   so the daemon does not rely on it.
- Snapshots need the checkout to be its own subvolume.
  The checkout is not one today,
   so converting it is a one-time copy that replaces every inode
   and can disturb editors,
   git worktrees,
   and running watchers.
- Deleting subvolumes without root needs the `user_subvol_rm_allowed` mount option.
  On this bootc host,
   research found that `/etc/fstab` btrfs options may not reach the live mount;
   the remediation path is unverified.

### Doctor

- Output follows per-capability status,
   numbered problems and warnings,
   a `--json` form,
   and a non-zero exit when problems exist,
   drawing on `flutter doctor`,
   `mise doctor`,
   and `brew doctor`.
- Checks include cgroup v2 delegation and controllers,
   the btrfs mount and `user_subvol_rm_allowed`,
   whether the checkout is a subvolume,
   and inotify limits,
   each with detection,
   the exact command or file edit,
   and the reason.

### File enforcement

- File-enforcer is rewritten in Rust and ships inside the single binary.
- The configuration language no longer needs to be Turing-complete;
   its format and the placement of today's configuration logic are being designed
   ("Correction on 2026-09-16").
- File-enforcement work runs in a re-executed child of the single file inside a task cgroup,
   as adopted in "All-Rust tool".
- Reads come from the tool's own code and declared task inputs,
   so undeclared reads are limited to repository tasks,
   which keep the cache risk recorded under "Risks".
- Superseded with the TypeScript route:
   running each TypeScript configuration evaluation in a child process instead of a cache-busting re-import.
- Carried over:
   file-enforcer emits typed events,
   because today's log records carry no structured fields for the activity feed.

### Migration from Mise

- 25 TypeScript files reference `MISE_MONOREPO`,
   and 41 source files reference Mise environment variables or invoke `mise`
   (research,
   verified).
- Tool provisioning,
   environment,
   and secrets stay separate owners per `mise-removal-coverage.md`.

## Risks

- Scope:
   the task graph,
   cache,
   watcher,
   scheduler,
   cgroup sandbox,
   RPC,
   and documentation are all repository-built.
- CI:
   whether GitHub runners provide a systemd user session for delegated cgroups is unverified.
- Frozen tasks keep holding locks while timers run:
   file-enforcer's manifest lock times out after 5 seconds for other writers,
   and tests may time out after resume.
- Shared daemons:
   ending one task's cgroup can kill a Gradle daemon another task reuses.
- Undeclared inputs:
   lint-level enforcement leaves stale cache hits possible.
- The vet's HC5 lists macOS and Windows CI runners;
   the user keeps Mise on those runners for now,
   and issues on unsupported systems do not block publishing.
- Rewrite:
   49 core modules,
   the Cargo and JetBrains plugins,
   `module-toml-edit` formatting behavior,
   and their tests move to Rust,
   and the 2,329-line root configuration moves to a new host.
- Output:
   every candidate crate changes today's bytes by default,
   and a missed formatting rule rewrites managed files including `CLAUDE.md`.
- Build:
   `btrfs-uapi` needs `libclang` at build time,
   `gxhash` needs the `+aes,+sse2` target features,
   and the size of the full binary is unmeasured beyond the static skeleton.
- CPU:
   the startup AES check is unexercised on a CPU without AES-NI.

## Decisions on 2026-09-16

- Tasks run Gradle with its daemon disabled,
   so ending or freezing a task cannot kill or stall shared Gradle state.
  Gradle's `--no-daemon` does not stop the Kotlin compile daemon:
   `/tmp` held 43 `kotlin-daemon` logs on the development machine
   (measured 2026-09-16).
  Gradle projects also need `kotlin.compiler.execution.strategy=in-process`
   (`monorepo-manager-route-research/stack-kotlin.md`).
- Per-task spawning through `systemd-run --scope` must pass `--expand-environment=no`;
   by default,
   systemd-run expands `${VARIABLE}` in command arguments itself when `--scope` is used
   (`man systemd-run`).
- A paused running task releases its concurrency slot.
- Affected work comes from native manifests:
   pnpm workspace dependencies,
   Cargo path dependencies,
   and Gradle projects,
   with extra rules for relationships between ecosystems.

## Open questions

User choices research raises are asked as they arise (rule `QGR`).

The pending task-name,
override,
and namespace questionnaire is withdrawn.
It narrowed the user's statement that build and test are built in to an unaccepted conclusion:
that they are privileged instances of a generic task.
Its duplication counts describe Mise configuration,
not the right domain model for meow.

The user resumed with:
"Do not accept Claude's framing of what a 'task' even is.
I believe better UX can be had by not thinking of stages that software must go through
in development as 'build task' and so on."

Current proposal,
not adopted:
[software model exploration](monorepo-manager-software-model.md).
It distinguishes products,
evidence,
readiness for a use,
and execution,
using the repository's source-import and built-bundle-test paths as a concrete case.
The user found the software-state-versus-stages comparison difficult to follow,
tentatively preferred software state,
and emphasized the coherent model of automatically keeping everything up to date across build,
correctness,
and publication.
The comparison is withdrawn as a decision the user needs to make.
The user then answered:
"Publication is automatic too.
On version bump + target registry already has it".
The user confirmed the package reading,
recorded in "Automatic publication".
The user then deferred the entire publication subsystem to 1.x,
chose push to `main`,
and instructed the agent to return to the model of non-tasks.
No publication question remains in the active frontier.

The current working proposal is package maintenance:
meow intrinsically understands affected build outputs,
current check results,
and managed-file requirements.
Asked where linting fits,
the model explicitly places lint,
type checking,
and tests together as maintained checks.
The word correctness includes repository policy and style here,
not only functional correctness.
Lint autofixing is a source-changing operation,
not an implied permission granted by automatic checking.
The user then asked whether a package should build after lint fails,
from a resource-saving standpoint.
The user accepted gated automatic progression:
applicable blocking source checks precede downstream production,
and a current failure blocks that work.
Actual build prerequisites of a check must still be available.
Independent result states do not justify eager concurrent execution.
Acceptance was followed by a request to continue grilling until the 0.x plan is concrete enough to implement.
Executions are how meow maintains these responsibilities,
not their common definition.
Q11 A additionally settles package-wide scope:
a maintained concern covers all applicable native parts owned by the package,
not only the native project at its root.
Narrower addresses remain possible;
address syntax is still open.
Q12 A requires an explicit connection when native declarations do not establish nested membership.
The user then clarified that both example groups should use separate packages:
the Android Gradle application and Rust engine,
and the fixture cases.
Do not absorb independent packages merely because they are nested,
or preserve those incumbent groupings as requirements.
A cross-package connection is distinct from ownership.
The [design workpad](monorepo-manager-design-workpad.md)
now examines whether an upstream source-check failure blocks a source-consuming package's production.

The conceptual model is aligned enough to resolve its concrete 0.x contracts.
Do not reopen the task-name or state-versus-stages questionnaires.
The [software model exploration](monorepo-manager-software-model.md)
records the repository example and the unresolved conceptual boundary with explicit one-off operations.
It does not ask which task names get special treatment.

Ordering remains open.
The unapproved `depends_on` entry stays retracted
("Settled without asking" in "Declarative configuration").
Native-manifest relationships and content-hash freshness remain accepted,
but neither settles how the new domain concepts relate.
No implementation is authorized.

Answered on 2026-09-17 before re-running the hash selection:
speed first,
and only `x86-64-v4` blocks publishing among x86-64 builds.

Closed on 2026-09-17:

- meow's cache key hash:
   XXH3-128 from `twox-hash` 2.1.4,
   accepted after the re-run vet and the m1 measurement
   ("Cache key hash re-run vet result").

- Repository-owned `gxhash` for meow,
   superseded when cache keys left `gxhash`;
   the music player's switch is issue #545.
- Comment-preserving JSONC,
   TOML,
   and XML editing,
   settled in "Managed file editing".
- aarch64 builds and the CPU capability warning under QEMU,
   settled in "Platform probes".

Design work finished on 2026-09-17:

- How `vm-builder` replaces its `exec` import from file-enforcer's `/ts` subpath
   ("vm-builder migration"),
   answered by the user on 2026-09-17
   and recorded there and in the all-Rust decision record.
- The HCL evaluator,
   function library,
   formatter,
   and language server ("HCL tooling"),
   accepted in
   [`doc/decision/monorepo-manager-hcl-front-end.md`](../decision/monorepo-manager-hcl-front-end.md).
- The per-user configuration's discovery and precedence ("Per-user configuration"),
   accepted in
   [`doc/decision/monorepo-manager-per-user-config.md`](../decision/monorepo-manager-per-user-config.md).
