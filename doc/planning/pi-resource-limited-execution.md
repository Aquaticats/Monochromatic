# Plan: resource-limited Pi command execution

Status:
 confirmed implementation plan from grill-me review.
 Not built.
 Authored 2026-08-29.

Tracking issue:
 [#470](https://github.com/Aquaticat/Monochromatic/issues/470).

## Goal

Run Pi and every subprocess it starts under fixed Linux resource controls.
The design protects the interactive host from accidental or buggy resource exhaustion.
It is not a security sandbox and does not claim to contain hostile same-user code.

The policy covers:

- Pi's model-callable `bash` tool;
- interactive `!` and `!!` commands;
- every `process start` command from `@aliou/pi-processes`;
- subprocesses launched through `pi.exec()`;
- subprocesses launched directly by extensions;
- Pi's own Node runtime.

Every Pi session is independent.
Concurrent sessions do not share a parent resource ceiling.

The implementation must degrade visibly when systemd enforcement is unavailable.
It warns and executes through the least-limited available parent path rather than making Pi unusable.
If the outer session scope cannot start,
Pi runs without this plan's resource controls.

## Settled policy

The user confirmed these decisions during the grilling session:

- Protect against accidental and buggy consumption,
  not deliberate escape attempts.
- Cover every Pi-triggered subprocess surface.
- Install globally for this user while keeping source and tests in this repository.
- Warn and continue without the unavailable control rather than fail closed.
- Place the complete Pi process tree inside a per-session systemd slice.
- Do not aggregate limits across concurrent Pi sessions.
- Give each `process start` invocation its own child profile.
- Use separate fixed profiles for ordinary commands and long-running processes.
- Let the implementation choose the numeric budgets.
- Add no default Bash timeout.
- Keep total storage consumption and network bandwidth outside this plan.

No user preference questions remain open.

## Verified source facts

### Pi extension seams

Installed Pi is `@earendil-works/pi-coding-agent` `0.84.3`.
The inspected workspace link under
`package/pi-plugin/advisor/node_modules/@earendil-works/pi-coding-agent/`
resolves to the same pnpm store directory as the package read during investigation.
The compared extension document and Bash implementation hashes are identical.
Its extension documentation states:

- `tool_call` inputs are mutable before execution at `docs/extensions.md:768-783`.
- Extensions can intercept interactive Bash through `user_bash` at `docs/extensions.md:859-887`.
- `dist/modes/interactive/interactive-mode.js:2506-2517` routes both `!` and `!!` to `handleBashCommand`;
  lines 5414-5422 emit `user_bash` for that shared path.
- Extensions can override built-in tools by registering the same name at `docs/extensions.md:2060-2084`.
- The Bash tool exposes `BashOperations` and local-operation helpers at `docs/extensions.md:2122-2126`.

The installed Bash implementation spawns a detached process group and kills that group on abort or timeout at
`dist/core/tools/bash.js:48-89`.
The override must preserve those cancellation and process-tree semantics.

Pi does not add a timeout when a Bash call omits `timeout`:

- `bash.js:18-20` maps an absent value to `undefined`.
- `bash.js:30-33` describes the field as optional with no default.
- `bash.js:74-80` creates a timer only when the value is defined.

The extension must preserve a caller-supplied timeout but must not invent one.

Pi exports `main(args)` from its package interface.
The repository-owned launcher can invoke that function directly after entering the limited session,
without locating another `pi` executable or recursively invoking its own shim.

### `pi-processes` execution seam

Installed `@aliou/pi-processes` is version `0.12.0`.
Its `execution.shellPath` setting is an absolute shell executable at
`~/.pi/agent/npm/node_modules/@aliou/pi-processes/extensions/processes/config/types.ts:8-13`.

`~/.pi/agent/npm/node_modules/@aliou/pi-processes/src/utils/command-executor.ts:44-59`
resolves that setting and spawns the selected executable with `['-lc', command]`.
The process manager keeps the original command separately,
so a shell adapter can add resource control without replacing command metadata,
logs,
stdin,
notifications,
or stop behavior.

The global process setting belongs at:

```text
~/.pi/agent/extensions/processes.json
```

`@aliou/pi-utils-settings` derives that path from the extension name at
`~/.pi/agent/npm/node_modules/@aliou/pi-utils-settings/src/config-loader.ts:355-361`.
Its `merge()` implementation gives local configuration precedence over global configuration
and memory configuration precedence over both at lines 605-619.

The process extension exposes its resolved configuration synchronously through the
`processes:request:config` event-bus request at
`~/.pi/agent/npm/node_modules/@aliou/pi-processes/extensions/processes/client.ts:76-90`
and `extensions/processes/handlers/requests.ts:51-54`.
The resource-limit extension can therefore detect a local or memory `shellPath` override at runtime.

### Host and systemd behavior

The measured host has:

- systemd `259`;
- a reachable user manager;
- cgroup v2 controllers `cpu`,
  `io`,
  `memory`,
  and `pids`;
- 16 logical CPUs;
- 62.4 GiB physical memory;
- 16 GiB swap;
- `~/.pi/agent/bin` as the first `PATH` entry;
- no existing file in `~/.pi/agent/bin`.

A disposable `systemd-run --user --scope` probe applied:

```text
MemoryHigh=64M
MemoryMax=128M
MemorySwapMax=0
CPUQuota=50%
TasksMax=32
```

The Node parent and its Bash child reported the same cgroup path.
The cgroup files reported:

```text
memory.high = 67108864
memory.max = 134217728
memory.swap.max = 0
cpu.max = 50000 100000
pids.max = 32
```

This proves that ordinary direct and detached descendants inherit the selected scope.
It also proves that the existing `pi-processes` child-spawn mechanism remains bounded by the outer session slice.

A second disposable probe started an automatically synthesized user slice,
applied runtime cgroup properties with `systemctl --user set-property`,
and read those values back successfully.
The same interface rejected `StopWhenUnneeded` with
`Cannot set property StopWhenUnneeded, or unknown property`.
The implementation can create a unique limited slice before starting Pi without a persistent user unit file,
but crash cleanup needs an explicit package-owned stale-unit reaper.

A shell-free 50-run `hyperfine` measurement established the pre-implementation process-start band on this host:

```text
/usr/bin/true
mean 0.608 ms, standard deviation 0.064 ms, range 0.508 ms to 0.897 ms

systemd-run with memory, CPU, and task properties around /usr/bin/true
mean 7.6 ms, standard deviation 0.3 ms, range 6.9 ms to 8.3 ms
```

The built command adapter must repeat this measurement with its acknowledgement helper.
Its 95th-percentile added startup time must remain at or under 25 ms on this host.

The repository already has a narrower systemd precedent at
`package/nested-wayland-session/cli/src/systemd.rs:137-196`.
It wraps a hosted process with
`systemd-run --user --scope --collect --quiet`,
applies `CPUQuota` and `CPUWeight`,
and degrades to a direct command with a warning.
The Pi package should share the policy shape,
but its availability check must create or inspect a real user unit rather than treating `systemd-run --version`
as proof that the user manager is reachable.

## Resource profiles

Profiles are fixed implementation policy.
The model and individual tool calls cannot raise or disable them.

### Session profile

The session slice contains Pi,
its ordinary descendants,
and the command and process child scopes.
Its cgroup properties are:

```text
MemoryHigh=8GiB
MemoryMax=12GiB
MemorySwapMax=0
CPUQuota=400%
TasksMax=2048
```

The outer helper applies `RLIMIT_NOFILE=16384` to Pi before invoking `main(args)`.
The descriptor ceiling is inherited per process;
it is not a slice property or an aggregate descriptor count.

`CPUQuota=400%` means four logical CPUs of aggregate runtime.
`TasksMax` counts processes and threads in the session slice.

### Command profile

Model `bash`,
`!`,
and `!!` use:

```text
MemoryHigh=2GiB
MemoryMax=4GiB
MemorySwapMax=0
CPUQuota=200%
TasksMax=512
```

The command helper applies `RLIMIT_NOFILE=4096` before invoking the shell.

### Long-running process profile

Every `process start` uses:

```text
MemoryHigh=4GiB
MemoryMax=8GiB
MemorySwapMax=0
CPUQuota=300%
TasksMax=1024
```

The process helper applies `RLIMIT_NOFILE=8192` before invoking the login shell.

The process profile has no policy wall-clock limit.
Servers and watchers remain under CPU,
memory,
task,
swap,
and descriptor ceilings until the process manager stops them.

### Effective hierarchy

The session slice is the aggregate parent.
A child scope's effective resource availability is the stricter of its own profile and the session profile.

Subprocesses that do not use a child adapter remain in Pi's scope and inherit the session profile.
This includes direct extension subprocesses and `pi.exec()`.

## Systemd hierarchy

Create a cryptographically random lowercase hexadecimal session identifier.
Use it in a unit name that passes systemd unit-name escaping and length validation.

The intended hierarchy is:

```text
pi.slice
└── pi-resource.slice
    └── pi-resource-<session>.slice
        ├── pi-resource-<session>-host.scope
        ├── pi-resource-<session>-command-<call>.scope
        └── pi-resource-<session>-process-<call>.scope
```

The leading `pi.slice` level is synthesized by systemd's dash-delimited slice hierarchy.
`pi-resource.slice` is also synthesized and has no aggregate limits.
This preserves the decision that concurrent Pi sessions do not share a ceiling.

Startup order is strict:

1. Start the unique session slice.
2. Apply every session cgroup property with one runtime `systemctl --user set-property` invocation.
3. Read back and validate every effective property.
4. Publish a private ownership marker containing the unit,
   nonce,
   launcher PID,
   and process start identity.
5. Launch Pi's inner entrypoint in the host scope under that slice.
6. Apply the session descriptor limit before invoking Pi's exported `main(args)`.

If any step before Pi starts fails,
clean up the partial unit state,
print a warning naming the failed operation,
and invoke `main(args)` directly.
Do not describe that fallback as limited.

On normal exit or signal-driven shutdown:

- let Pi perform its extension and process-manager cleanup;
- stop the unique session slice;
- revert its runtime properties;
- remove its ownership marker and generated transient state;
- preserve Pi's actual exit status.

A launcher killed before cleanup can leave an active empty slice and runtime property drop-ins.
Every outer launcher startup reaps stale package-owned markers whose recorded process identity no longer exists.
It verifies that the matching unit has no processes or populated descendant cgroups,
stops that unit,
and reverts its runtime properties.
It must not touch a live or populated unit or a unit without a valid package-owned marker.

Cleanup errors are warning diagnostics and must not replace a more informative Pi failure.

## Package shape

Create:

```text
package/pi-plugin/resource-limit/
```

The package name is:

```text
@monochromatic-dev/pi-resource-limit
```

The package is one deep module.
Its interface is the fixed profile names plus launch operations.
Systemd unit naming,
argument construction,
startup acknowledgement,
process-group termination,
fallback,
and cleanup remain implementation details behind that interface.

Expected package files include:

- `README.md`;
- `package.json`;
- `mise.toml`;
- `rolldown.node.config.ts`;
- `tsconfig.json`;
- `src/index.ts`;
- `src/profiles.ts`;
- `src/resource-runner.ts`;
- `src/systemd-controller.ts`;
- `src/direct-controller.ts`;
- `src/session-launcher.ts`;
- `src/process-shell.ts`;
- `src/bash-operations.ts`;
- `src/diagnostic.ts`;
- `src/mise.verify-extension.ts`;
- focused colocated unit tests;
- systemd integration verification using disposable unit names.

The package should expose executable artifacts for:

- the outer `pi` launcher;
- the `pi-processes` shell adapter.

The launcher resolves Pi through the package's host peer and invokes the exported `main(args)` from that same module.
Build verification must compare the launcher and extension Pi module realpaths and exported versions.
A mismatch blocks activation.
Pi and this package are upgraded and rebuilt together rather than allowing the launcher to retain an older private Pi.

The Pi extension artifact registers the built-in Bash override and the `user_bash` handler.
It must not expose model-configurable resource knobs.

Use tagged loggers for extension runtime logs.
Use raw terminal output only for launcher diagnostics that must be visible before Pi initializes logging.

## Launcher interface

Install an atomic symbolic link:

```text
~/.pi/agent/bin/pi
```

The link targets the built repository-owned launcher.
The directory is already first in `PATH`,
so a new shell resolves the limited launcher without editing shell startup files.

The launcher has two internal modes:

- outer mode creates and validates the session slice;
- inner mode invokes Pi's exported `main(process.argv.slice(2))` after acknowledgement.

An unforgeable per-launch nonce and inherited environment marker distinguish the modes.
Do not use command-string reparsing to find or invoke the real Pi binary.

A nested `pi` invocation that inherits a valid session marker stays in its parent's session slice.
It invokes `main(args)` without creating a sibling session slice,
so a Pi-started child Pi cannot escape the parent's aggregate profile accidentally.
A separately launched terminal session has no marker and receives its own independent slice.

Only the installed `~/.pi/agent/bin/pi` entrypoint can establish the outer session profile.
Direct execution of another package's absolute `pi` path or bundle bypasses the launcher and is documented as such.

The launcher must preserve:

- all Pi command-line arguments;
- stdin,
  stdout,
  stderr,
  and TTY ownership;
- signals and terminal resize behavior;
- exit status;
- the current working directory;
- Pi's existing environment.

## Startup acknowledgement

Warning fallback must never execute a command twice.
A nonzero `systemd-run` exit is ambiguous because it can represent either setup failure or the wrapped command's exit.

Use an explicit acknowledgement channel:

1. The outer controller creates a private per-launch acknowledgement endpoint under `XDG_RUNTIME_DIR`.
2. The endpoint name binds one nonce,
   one expected unit,
   and one expected profile to one launch.
3. A monitor helper acknowledges after systemd places it in the expected cgroup and before it starts user work.
4. The helper starts the requested child,
   waits for it,
   and remains in the scope while it snapshots `memory.events` and final unit evidence.
5. The helper sends structured exit and memory evidence to the outer controller before leaving the scope.
6. Once acknowledged,
   every later exit belongs to the wrapped command and must never trigger fallback.
7. If the wrapper exits or five seconds elapse before acknowledgement,
   the controller warns and runs the direct adapter once.
8. Remove the acknowledgement endpoint on every terminal path.

The endpoint must be owned by the current user and inaccessible to group or other users.
The acknowledgement includes the nonce and observed cgroup path.
Parallel tool calls always receive distinct endpoints,
nonces,
and unit names.

If `XDG_RUNTIME_DIR` is absent or fails its ownership and permission checks,
warn and select the direct adapter before attempting the command scope.
Do not place acknowledgement endpoints in a shared temporary directory.

Use the same protocol for session,
command,
and long-running process launches.

## Bash tool and interactive Bash

Override built-in `bash` with a tool created from Pi's exported Bash definition and resource-limited operations.
Preserve the built-in schema,
prompt metadata,
renderers,
output truncation,
full-output spool behavior,
and result details.

The operations adapter launches the monitor helper and resolved shell through:

```text
systemd-run --user --scope --collect --quiet --same-dir
```

It preserves Pi's configured shell and command transport,
including non-login `bash -c` on this host.
It passes arguments as an argv array.
Never interpolate the model's command into another shell command string.
The original command remains one argument to the resolved shell's command transport.
This is the syntax-encoding seam.

Before the shell starts,
apply `RLIMIT_NOFILE` with `prlimit` or an equivalent argv-based executable boundary.

Preserve Pi's behavior for:

- streamed stdout and stderr;
- abort signals;
- caller-supplied timeout;
- process-tree termination;
- nonzero exit status;
- missing working directory;
- shell resolution;
- session `PI_*` environment variables;
- output truncation and spool files.

Register the same command-profile operations through `user_bash` for `!` and `!!`.

At `session_start`,
inspect `pi.getAllTools()` and verify that the effective active `bash` source is this package's override.
If another extension wins the tool slot,
warn that the command profile is unavailable and name whether the outer session profile remains active.
Package-list activation places this extension after known Bash overrides,
but runtime verification remains authoritative.

A quiet background descendant may outlive the foreground shell.
It remains in the command scope and session slice until it exits,
and the transient scope is collected only when empty.
Session shutdown stops the session slice and its remaining descendants.
This package does not add a new background-command parser or silently redirect background work.

When a child scope cannot start before acknowledgement,
log and stream a warning,
then execute through Pi's ordinary local Bash operations.
That fallback remains under the session profile when the outer launcher succeeded.

## `pi-processes` adapter

Implement an executable that accepts the shell interface used by `pi-processes`:

```text
pi-resource-process-shell -lc <command>
```

Reject unsupported argument shapes with a direct diagnostic.
Do not silently reinterpret extra shell flags.

The adapter creates a process-profile scope beneath the current session slice,
then invokes the configured real Bash as a login shell with `-lc` and the original command as one argv element.
It passes the caller's working directory explicitly through `systemd-run --same-dir` and verifies it in the helper.
It applies the process descriptor limit before Bash starts.

Configure the existing extension globally:

```json
{
  "execution": {
    "shellPath": "/absolute/path/to/pi-resource-process-shell"
  }
}
```

Merge that key into any future existing `processes.json` object.
Do not replace unrelated user settings.
The installer validates the absolute target before publishing the config.

At `session_start` and before every `process start` tool call,
request the resolved process configuration through `processes:request:config`.
When `execution.shellPath` differs from the installed adapter,
warn that the process child profile is unavailable and that only the outer session profile applies.
Track that call and prepend the same warning to its tool result.
This check detects project-local and in-memory overrides that take precedence over the global file.

The existing `pi-processes` package remains authoritative for:

- process identity;
- listing and sorting;
- logs;
- stdin;
- watches and notifications;
- stop and kill behavior;
- cleanup.

Do not fork or replace it unless end-user verification proves the shell adapter cannot preserve those behaviors.
If that contingency occurs,
stop implementation,
record the failing call and output,
and add a replacement design to this plan before writing one.

## Configuration and activation

Add the repository package path to global `~/.pi/agent/settings.json`.
Do not modify project-local `.pi/settings.json`.

Activation occurs only after build and verification artifacts exist.
It must update these user-owned resources transactionally:

- `~/.pi/agent/settings.json` package list;
- `~/.pi/agent/extensions/processes.json` shell path;
- `~/.pi/agent/bin/pi` launcher link.

Before mutation:

- resolve the current home directory at runtime;
- validate every source artifact;
- inspect existing target ownership;
- preserve unrelated JSON properties;
- prepare rollback copies with user-only permissions.

Publish temporary files with atomic rename.
Create or replace the launcher link only when its current target is absent or owned by this package.
An unrelated existing target is a blocker,
not something to overwrite.

Rollback removes only package-owned entries and restores prior values.

## Diagnostics

Every degraded launch must name:

- affected surface,
  either session,
  Bash,
  interactive Bash,
  or managed process;
- failed control or operation;
- whether the session parent still limits the command;
- whether execution is fully unbounded by this package;
- exact remediation paths:
  user-manager availability,
  missing executable,
  rejected systemd property,
  or invalid installation target.

Do not report an OOM-killed command as a generic nonzero exit.
The monitor helper remains a member of the child scope after the requested child exits,
reads `memory.events` while the cgroup still exists,
and sends that evidence before it exits and permits collection.
Append a plain diagnostic identifying the profile and `MemoryMax` value only when the captured counters support it.
If the helper is also killed and no evidence survives,
report the observed signal or status without inferring OOM.

CPU throttling is normal enforcement,
not an error.
Expose applied profile and unit name in debug logs without adding noise to normal tool output.

## Explicit non-goals

This plan does not provide:

- a hostile-code sandbox;
- filesystem read or write policy;
- a total disk-usage quota;
- block-I/O bandwidth limits;
- network bandwidth limits;
- network destination policy;
- GPU memory or execution limits;
- a shared cap across concurrent Pi sessions;
- model-selected or per-call resource budgets;
- a new default wall-clock timeout;
- a replacement for `@aliou/pi-processes` unless its adapter verification fails;
- interception of Pi launched through an absolute alternate executable or bundle path;
- a new parser or policy for shell background syntax.

A same-user command may deliberately ask the user manager to create a sibling unit outside its current hierarchy.
That is accepted under the accidental-containment threat model.

## Tests

### Profile and argument tests

Cover:

- exact systemd properties for every profile;
- exact `prlimit` values;
- unit-name validation and escaping;
- parent-slice propagation;
- argv preservation for spaces,
  quotes,
  newlines,
  shell separators,
  leading hyphens,
  and Unicode;
- environment and working-directory preservation;
- no command-string interpolation;
- acknowledgement success,
  rejection,
  timeout,
  stale nonce,
  parallel-call uniqueness,
  and missing `XDG_RUNTIME_DIR`;
- direct fallback before acknowledgement;
- no fallback after acknowledgement;
- monitor-held cgroup evidence before collection;
- signal and exit-status propagation;
- cleanup after success,
  command failure,
  cancellation,
  setup failure,
  and launcher `SIGKILL`;
- stale marker and inactive-slice reaping without touching live or unowned units;
- nested Pi remaining in its inherited session slice;
- launcher and extension Pi module realpath and version equality.

### Pi extension tests

Cover:

- exactly one Bash override registration;
- effective active Bash source verification with another override loaded after it;
- built-in Bash input and result shapes;
- command profile selection;
- interactive `user_bash` selection;
- caller-supplied timeout preservation;
- absent timeout remaining absent;
- PI session environment preservation;
- warning content for command-profile fallback.

### `pi-processes` adapter tests

Cover:

- only `-lc <command>` is accepted;
- original command reaches login Bash unchanged;
- process profile properties are visible from the child;
- working directory and login-shell environment are preserved;
- stdout and stderr remain distinct;
- stdin reaches the child;
- process-group stop terminates the complete scope;
- the stop guard is shown to fail when scope-aware descendant termination is removed,
  then restored;
- a nonzero child status is preserved;
- process setup fallback starts the command once;
- stored process metadata retains the original command;
- global adapter configuration is recognized;
- project-local and memory `shellPath` overrides produce session and tool-result warnings;
- restoring the adapter path removes the warning on the next start.

### Installation tests

Use disposable homes and settings fixtures.
Cover:

- absent and existing settings files;
- preservation of unrelated settings;
- package-list de-duplication;
- absent and existing process config;
- refusal to replace an unrelated launcher;
- atomic activation failure rollback;
- idempotent activation;
- rollback after activation.

## Verification plan

Run package-scoped Mise tasks only.
The package must provide tasks for:

```text
build
lint:types
lint:oxlint
test:unit
verify:extension
verify:systemd
verify:pi-runtime
```

`verify:systemd` uses unique disposable user units and restores runtime properties after every case.
It must include a positive control that proves the probe can distinguish a limited child from a direct child.

Verify cgroup values by reading the child's own `/proc/self/cgroup` and controller files.
Do not infer enforcement from command construction.

Use reduced fixture limits for destructive guard tests:

- memory allocation crosses a disposable fixture's `MemoryMax` without pressuring the host;
- task creation reaches a disposable fixture's `TasksMax` without approaching the host limit;
- descriptor opening reaches a disposable fixture's `RLIMIT_NOFILE`;
- CPU verification reads `cpu.max` and observes throttling counters without a host-wide stress run.

Measure direct and integrated command startup in shell-free 50-run bands.
Report mean,
standard deviation,
range,
median,
and 95th percentile.
The integrated adapter's added 95th-percentile startup time must remain at or under 25 ms on this host.

At the end-user boundary:

1. Activate the built package against a disposable Pi home and launcher directory.
2. Start Pi through the installed launcher.
3. Confirm Pi itself is inside the session slice with every session cgroup property and descriptor limit.
4. Start a second top-level Pi and confirm it receives a different independent session slice.
5. Start a nested Pi from limited Bash and confirm it remains in the inherited session slice.
6. Invoke model `bash` and inspect command-profile cgroup values.
7. Invoke `!` and `!!` through a pseudo-terminal and inspect the same values.
8. Start a managed process that reports its cgroup,
   current directory,
   login-shell environment,
   and descriptor limit;
   accepts stdin;
   emits watched output;
   and remains running.
9. Confirm process output,
   notification,
   stdin,
   stop,
   and cleanup behavior through the actual `process` tool.
10. Start a quiet background descendant from Bash,
    confirm its scope remains bounded after foreground-shell exit,
    and confirm session shutdown removes it.
11. Trigger each pre-acknowledgement fallback with a fixture controller and confirm one warning plus one execution.
12. Confirm a post-acknowledgement command failure never reruns directly.
13. Kill a fixture launcher without cleanup and confirm the next startup safely reaps only its inactive owned state.
14. Activate against the real global Pi home only after disposable verification passes.
15. Start a fresh real Pi session and repeat session,
    Bash,
    interactive Bash,
    and managed-process probes.

The currently running Pi process cannot be retrofitted into its future session slice.
Real activation verification therefore occurs in a newly launched Pi process.

## Completion criteria

Implementation is complete only when:

- the package has a README,
  zero lint diagnostics,
  and passing branch-complete tests;
- every top-level Pi session launched through `~/.pi/agent/bin/pi` enters its own validated session slice;
- nested Pi stays inside its inherited session slice;
- launcher and extension resolve the same Pi module realpath and version;
- stale package-owned slice state is reaped after crash without touching live,
  populated,
  or unowned units;
- every model and interactive Bash call uses the command profile or emits an explicit fallback warning;
- the effective active Bash tool source is the repository-owned override or a visible warning names the conflict;
- every `process start` uses the process profile or emits an explicit fallback warning,
  including when local or memory process settings override the global adapter path;
- direct extension subprocesses remain inside the session slice;
- caller-supplied Bash timeouts still work and absent timeouts remain absent;
- no command is run twice during fallback;
- Pi's TTY,
  signal,
  working-directory,
  environment,
  and exit behavior are preserved;
- `pi-processes` start,
  login-shell environment,
  working directory,
  output,
  stdin,
  notification,
  stop,
  and cleanup pass through the installed adapter;
- each acknowledgement endpoint is unique to one unit and call;
- missing or insecure `XDG_RUNTIME_DIR` degrades once without duplicate execution;
- OOM attribution uses monitor-captured cgroup evidence before collection;
- integrated command startup satisfies the measured 25 ms 95th-percentile bound on this host;
- activation and rollback preserve unrelated global settings;
- storage,
  network,
  GPU,
  hostile-code,
  and concurrent-session limitations are documented in the package README;
- the README states that absolute alternate Pi executables bypass the launcher;
- user-boundary verification passes from a newly launched real Pi session.

## Rejected alternatives

### Bash-only extension

Pros:

- Uses Pi's direct Bash override seam.
- Does not constrain Pi's own runtime.

Cons:

- Leaves `pi.exec()`,
  direct extension spawns,
  and `pi-processes` outside the policy.
- Contradicts the confirmed all-surface requirement.

### Known-path interception without an outer session slice

Pros:

- Can add limits to model Bash,
  interactive Bash,
  and known process tools.

Cons:

- Future and direct extension subprocesses bypass it.
- Does not provide one aggregate ceiling for a Pi session.

### Shared aggregate across Pi sessions

Pros:

- Concurrent Pi sessions cannot multiply resource use beyond one host budget.

Cons:

- One session can throttle or trigger memory pressure in another.
- The user explicitly chose independent session ceilings.

### Replace `pi-processes` immediately

Pros:

- Full ownership of process spawning and resource profiles.

Cons:

- Reimplements process identity,
  logs,
  stdin,
  notifications,
  watches,
  stop,
  UI,
  and cleanup before evidence requires it.
- The installed package already exposes an absolute shell seam that fits the design.

### Default Bash wall-clock timeout

Pros:

- Bounds forgotten command lifetime.

Cons:

- Changes existing Pi behavior.
- The user explicitly rejected adding a timeout.

### Fail-closed enforcement

Pros:

- Never executes outside the promised profile.

Cons:

- Makes Pi command execution unavailable when the user manager or controller is unavailable.
- The user explicitly selected visible degradation and continued execution.

Ranking:
 whole-session hierarchy with fixed child profiles > known-path interception > Bash-only extension.
The hierarchy ranks first because it covers both current and future subprocess surfaces.
Known-path interception ranks above Bash-only because it covers more of the confirmed scope,
but both leave direct extension spawns unbounded.
