# Slint IDE language-server write confinement

## Purpose and status

The accepted scope ([decision](../decision/slint-ide-0x-scope.md),
[implementation plan](slint-ide-implementation.md) "Language module") requires that language-server processes,
and everything they spawn,
cannot write project files directly or by delegation,
while keeping writable private tool state outside the project.
Cache redirection alone does not count as enforcement.

This document records an investigation done on 2026-10-05 on the target host with disposable fixtures.
It proposes one launch shape and ranks the alternatives.
Nothing under `package/` was changed.
The proposal is not adopted until the coordinator accepts it.

What to inspect:
"Ranking",
"Recommended launch shape",
"Open decisions for the coordinator",
and "Residual risks".
How to respond:
accept or veto the recommended shape,
and answer the network question under "Open decisions for the coordinator".

## Recommendation in short

- Wrap every language server in `/usr/bin/bwrap` (bubblewrap 0.12.0,
   already installed)
  with the whole file system read-only,
  one private state directory and a private `/tmp` writable,
  `/run` hidden,
  and,
   where the server tolerates it,
   its own process-id namespace.
- Put the wrapper in the Helix `command`,
  the recipe plus the original command in `args`,
  and the redirect variables in `environment`.
  No Helix fork and no new dependency is needed.
- rust-analyzer needs `CARGO_TARGET_DIR` and,
   on this host,
   `CARGO_BUILD_BUILD_DIR` pointed into private state.
- Both TypeScript servers measured exit by themselves inside a process-id namespace,
  because `helix-lsp` sends its own process id and the servers watch it.
  They run without `--unshare-pid`.
- Ranking:
   bubblewrap > `unshare` script > systemd transient user unit > podman > Landlock through `setpriv`.
  Reasons are under "Ranking".

## Host facts

- Bazzite 44 (Fedora Atomic),
   kernel `7.2.7-ogc1.1.fc44.x86_64`,
   x86_64,
   SELinux enforcing.
- `/usr/bin/bwrap` 0.12.0,
   not setuid.
   `user.max_user_namespaces` is 254592.
- util-linux 2.41.5 (`setpriv`,
   `unshare`,
   `mount`).
   systemd 259.9.
   podman 5.8.7.
- Landlock is in `/sys/kernel/security/lsm`;
   `landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION)`
  returns ABI 10.
- `/home` is a symlink to `var/home`.
   The home directory is on btrfs,
   `/tmp` is tmpfs.
- `~/.cargo/bin/rust-analyzer` is a rustup 1.29.1 proxy;
  the default toolchain gives rust-analyzer 1.100.0-nightly (1303417 2026-09-21) and cargo 1.100.0-nightly.
- `~/.cargo/config.toml` sets `build.build-dir = "{cargo-cache-home}/build/{workspace-path-hash}"`.
- Node 26.10.0.
   `typescript-language-server` 6.0.0 (it bundles no TypeScript).
  TypeScript 7.0.2 (`tsc --lsp --stdio`,
   identifies as `typescript-go`),
  the server adopted in [the language-intelligence plan](slint-ide-language-intelligence.md).
- The host was heavily loaded by other sessions during measurement:
  load average 45 to 53 on 16 processors.
  Readiness times in this document are therefore not comparable between runs.

## Integration constraints from `helix-lsp`

Read at the pinned revision `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`.

- Launch is `Command::new(which(cmd)).envs(environment).args(args)` with piped standard streams,
  `.current_dir(&root_path)`,
   and `.kill_on_drop(true)` (`helix-lsp/src/client.rs:228` to `:239`).
  The command type is `tokio::process::Command` (`client.rs:36`).
- There is no pre-exec hook,
   no process group,
   and no environment clearing.
  A wrapper command is therefore the shape that works without forking Helix
  and without depending on which thread spawns the server.
- Environment is additive:
   the server inherits the whole application environment plus the `environment` map.
- The working directory is the workspace root Helix computes inside `start_client`
  (`helix-lsp/src/lib.rs:900` to `:915`).
  It is not known when the configuration is built,
  so the wrapper arguments must not depend on it.
- `kill_on_drop` sends SIGKILL to the direct child only,
   which is the wrapper.
- `initialize` always carries `process_id: Some(std::process::id())` (`client.rs:579`).
- A workspace `.helix/languages.toml` is merged only when the workspace is trusted for local configuration
  (`helix-loader/src/config.rs:14` to `:24`).
  If the application ever loaded it,
   the project could replace `command` and defeat any wrapper.

Measured shapes:

- Wrapper in `command`,
   original command and arguments appended to `args`:
   works for all three servers.
- Variables in the `environment` map reach the server through bubblewrap unchanged
  (`extra-results.json`,
   `cwdAndEnvironment`).
- Bubblewrap keeps the working directory it was started in when no `--chdir` is given
  (measured,
   and `bubblewrap.c:3414` at tag `v0.12.0`),
  so no workspace root appears in the arguments.
- A relative server command (`node_modules/typescript/bin/tsc`) also works after `--`,
  but it then resolves against the server working directory instead of the application working directory.

## Method

All work used disposable copies under `~/temp/agent/ide-confine.JNN2oC`,
on the same file system as real projects so that hard-link and rename probes are meaningful.
No server was pointed at the repository.

### Fixtures

- `template/rs`:
   dependency-free Rust package with a `build.rs`,
   a path proc-macro crate `mac`,
  a committed `Cargo.lock`,
   and victim files.
  `src/main.rs` uses a constant generated by the build script and a function generated by the proc macro,
  so a correct hover on each proves that the build script and the proc-macro server ran.
- `template/rs-dep`:
   the same with the registry dependency `cfg-if = "=1.0.5"`,
  which the host cargo cache already holds downloaded and extracted.
- `template/rs-toolchain`:
   the same with a `rust-toolchain.toml` naming the uninstalled channel `1.70.0`.
- `template/ts`:
   TypeScript project with a copied TypeScript 6.0.3 in `node_modules`
  and a project-local tsserver plugin named in `tsconfig.json`.
- `template/ts7`:
   TypeScript project with a copied TypeScript 7.0.2;
  its `node_modules/typescript/bin/tsc` launcher runs the probe first,
   then the real launcher.
- Both TypeScript fixtures hold a JavaScript file with an untyped bare import,
  which invites automatic type acquisition.

### Session driver

`session.mjs` copies a fixture,
 snapshots the project tree,
 starts the server the way `helix-lsp` does
(resolved command,
 arguments,
 added environment,
 working directory set to the root,
 piped streams),
then runs `initialize`,
 `initialized`,
 `didOpen`,
 hover until the expected text appears,
one definition request,
 `shutdown`,
 and `exit`.
It then waits,
 lists surviving processes by a marker variable,
 snapshots again,
 and diffs.

The snapshot covers every path with type,
 size,
 mode,
 modification time,
 change time,
 link count,
inode,
 SHA-256 for files,
 and the target for symbolic links.

The inherited environment is an allowlist (`PATH`,
 `HOME`,
 `USER`,
 `LOGNAME`,
 `LANG`,
 `SHELL`,
`XDG_RUNTIME_DIR`,
 `DBUS_SESSION_BUS_ADDRESS`,
 `WAYLAND_DISPLAY`,
 `DISPLAY`),
so shell-only variables such as `RUSTUP_TOOLCHAIN` did not influence the servers.

### In-tree probes

Write attempts come from inside the server's own process tree:

- the Rust `build.rs` (rust-analyzer,
   cargo,
   build script);
- the proc macro,
   which runs both in `rustc` and in `rust-analyzer-proc-macro-srv`;
- a Node probe started by the build script;
- the project-local tsserver plugin,
   inside tsserver;
- the project-supplied TypeScript 7 launcher.

The Rust probes try the project,
 private state,
 an unrelated directory,
 and `/tmp`,
 and record the raw error number.
The Node probe (`probe.cjs`) additionally tries:
append,
 truncate,
 rename,
 unlink,
 mkdir,
 chmod,
 utimes,
 and setxattr on project files;
a symbolic link from state into the project and a write through it;
a symbolic link created inside the project;
hard links in both directions;
rename of a state file over a project file and of a project file out;
the aliases `/proc/self/root/...`,
 `/proc/self/cwd/...`,
 `/proc/<application pid>/root/...`,
the other spelling of the home path,
 and reopening a read-only descriptor through `/proc/self/fd`;
a read-write remount from a nested user namespace;
`systemd-run --user` asked to touch a project file;
connections to the session bus,
 the Wayland socket,
 the system bus,
 the abstract X11 socket,
and a loopback TCP listener opened by the driver;
signal 0 to the driver process;
and a double-forked child in its own session that writes after a delay.

### Controls

- Positive control for the tree diff and for every probe:
   the unconfined runs `base-rust`,
   `base-ts`,
   and `base-ts7`.
  Every probe succeeded there and the diff reported each change,
  including the mode and time change on `victim-meta.txt`
  and the files `ESCAPED_VIA_SYSTEMD_*` and `DOUBLE_FORK_*`.
- Real tool state was protected.
  The first runs used an enumeration mode
  (bubblewrap with a discarded tmpfs overlay on the home directory,
   the project writable,
   `strace` inside)
  to learn every default write.
  The unconfined baselines then ran with a private cargo home holding a copy of the host `config.toml`,
  private `TMPDIR`,
   `XDG_CACHE_HOME`,
   and `npm_config_cache`;
  `strace` confirmed no write under the real home outside the run directory.

## What the servers write by default

Measured in enumeration mode,
 with no redirect at all.

### rust-analyzer

- Project:
   `Cargo.lock` when it is absent;
  `target/` with `CACHEDIR.TAG`,
   `debug/.cargo-lock`,
   `flycheck0/stdout`,
   and `flycheck0/stderr`;
  the modification time of the project root.
  With a committed lock file,
   `Cargo.lock` stayed byte-identical.
- Home:
   `~/.cargo/.package-cache`,
   `~/.cargo/.package-cache-mutate`,
   `~/.cargo/.global-cache`,
  and build intermediates under `~/.cargo/build/<hash>/`,
   because of the host cargo `build.build-dir` setting.
- `/tmp`:
   directories `rust-analyzer<hex>-<n>` and `proc-macro-srv<hex>-<n>`,
  and linker temporaries `cc*.res` and `cc*.cdtor.*`.
- No socket connections.

### `typescript-language-server` 6.0.0 with TypeScript 6.0.3

- Project:
   nothing.
   The tree was identical.
- Home:
   automatic type acquisition ran `npm install` into `~/.cache/typescript/6.0/`,
  wrote `~/.npm/_cacache` and `~/.npm/_logs`,
   and deleted two older npm log files.
- `/tmp`:
   `node-compile-cache` and two hash-named directories.
- Network:
   TCP 443 to the npm registry,
   and the resolver socket `/run/systemd/resolve/io.systemd.Resolve`.

### TypeScript 7.0.2 native server

- Project:
   nothing.
   The tree was identical.
- Home:
   automatic type acquisition as well:
  `~/.cache/typescript/7.0/`,
   `~/.npm/_cacache`,
   `~/.npm/_logs`,
   and one older npm log deleted.
- `/tmp`:
   `node-compile-cache`.
- Network:
   TCP 443 to the npm registry.

### Project code that runs inside the servers

These are not server defaults,
 but they decide what confinement must withstand:

- `build.rs` and proc macros run with the user's identity (measured writing everywhere in `base-rust`).
- A plugin from the project's `node_modules`,
   named in `tsconfig.json`,
   ran inside tsserver (`base-ts`).
- The TypeScript 7 launcher is itself a file in the project's `node_modules` (`base-ts7`).

## Candidates

Each candidate was driven through the same sessions and probes.
Results per probe are under "Cross-candidate results".

### Bubblewrap wrapper

One `bwrap` argument list:
 recursive read-only bind of `/`,
 private `/dev` and `/proc`,
 tmpfs on `/run`,
private state bound read-write,
 a state subdirectory bound at `/tmp`,
 new namespaces,
`--die-with-parent`,
 `--new-session`.

- Pros:
  - Already installed;
     one executable;
     no setup step.
  - Fits the Helix launch as is:
     environment and working directory pass through.
  - Every project write probe failed with `EROFS`,
     metadata included.
  - Hiding `/run` removed the session bus,
     so delegation through `systemd-run --user` failed.
  - With a process-id namespace,
     SIGKILL of the wrapper left no process.
  - Fails closed with a clear message when it cannot set up.
  - 10 ms per launch at best,
     20 to 64 ms median under load.
- Cons:
  - Needs unprivileged user namespaces.
  - A process-id namespace breaks servers that watch the client process id (both TypeScript servers).
  - `--die-with-parent` follows the spawning thread,
     not only the process (measured).
  - Reads stay unrestricted.

### Hand-rolled namespaces with `unshare`

`unshare --user --map-root-user --mount --pid --fork --kill-child` running an inline shell program
that binds state,
 remounts everything read-only with `mount --options remount,bind,ro=recursive /`,
remounts state read-write,
 mounts fresh `/proc` and a tmpfs on `/run`,
 binds the private `/tmp`,
then drops into a nested user namespace mapped back to the real user.

- Pros:
  - util-linux only.
  - Same measured protection as bubblewrap:
     tree identical,
     probes blocked,
     no survivor after SIGKILL.
  - Environment and working directory pass through.
- Cons:
  - An eight-line shell program inside the argument list re-implements bubblewrap,
    with its own quoting and ordering hazards.
  - `/dev/shm` ended up read-only,
     and `no_new_privs` is not set.
  - Same process-id watchdog problem as bubblewrap.
  - 19 ms per launch at best,
     28 to 82 ms median under load.

### systemd transient user unit

`systemd-run --user --pipe --wait --collect --quiet --same-dir` with `ProtectSystem=strict`,
`ProtectHome=read-only`,
 `ReadWritePaths=`,
 `PrivateTmp=yes`,
 `NoNewPrivileges=yes`.

- Pros:
  - Already running;
     the user manager created the mount and user namespaces without privileges.
  - With `InaccessiblePaths=/run/user/1000`,
     `InaccessiblePaths=/run/dbus`,
     `PrivatePIDs=yes`,
    and `PrivateNetwork=yes` added,
     it matched bubblewrap on every write and reachability probe
    (`systemd-rust-best`).
- Cons:
  - The service does not inherit the caller's environment;
     each name needs `--setenv=NAME`.
    It does inherit the user manager's environment:
    a stale `RUSTUP_TOOLCHAIN=1.98.1` there made rust-analyzer fail with
    `error: infinite recursion detected` until `UnsetEnvironment=RUSTUP_TOOLCHAIN` was added.
  - With the default property set the session bus stayed reachable,
    and `systemd-run --user` from inside created `ESCAPED_VIA_SYSTEMD_*` in the project.
  - SIGKILL of the `systemd-run` client left the unit `active` with the server and all children running,
    so `kill_on_drop` does not stop the server.
  - 54 ms at best,
     75 to 350 ms median under load.
  - Depends on the user bus;
     every launch adds a transient unit to the journal.

### Podman with a read-only mount

`podman run --rm --interactive --read-only --network=host --security-opt label=disable`
with the home directory mounted read-only at its host path,
 state read-write,
 and the private `/tmp`,
using the repository's `localhost/monochromatic/ide` image.

- Pros:
  - The repository already uses it for bounded builds.
  - Tree identical;
     every write probe failed with `EROFS`;
     no bus inside.
  - Memory,
     processor,
     and process-count limits come with it.
- Cons:
  - The image must carry or be compatible with every server:
    the host Node binary failed inside with `libatomic.so.1: cannot open shared object file`
    until the host library was bound in for the measurement.
  - Paths must match the host for URIs to be valid:
    both `/home/<user>` and `/var/home/<user>` had to be mounted.
  - The working directory must be given with `--workdir`,
     which the Helix configuration does not know.
  - Environment is not inherited;
     each name needs `--env=NAME`.
  - SIGKILL of the client left the container `Up` with `conmon` and every process.
  - 176 to 204 ms at best,
     350 ms to 2 s median under load.
  - Same process-id watchdog problem.
  - Relabeling with `:z` or `:Z` would write extended attributes on the project,
    so `label=disable` is mandatory.

### Landlock through `setpriv`

`setpriv --no-new-privs --landlock-access fs:<write-class rights>`
with `--landlock-rule path-beneath:<rights>:STATE` and `path-beneath:write-file:/dev/null`,
plus `TMPDIR` pointed into state.
No new crate or helper binary is needed for this shape:
 `setpriv` is part of util-linux.

- Pros:
  - Smallest overhead (3 ms at best,
     4 to 10 ms median under load) and simplest arguments.
  - Needs no user namespace.
  - Content writes,
     creation,
     removal,
     rename,
     and links in the project failed with `EACCES`.
  - Path aliases do not matter:
     rules follow the file hierarchy.
- Cons:
  - Project metadata stayed writable:
     `chmod`,
     `utimes`,
     and `setxattr` on a project file succeeded,
    and the tree diff showed the mode and time change.
    The kernel headers (`/usr/include/linux/landlock.h`,
     kernel-headers 7.2.4) define no access right for these.
  - The session bus stayed reachable and `systemd-run --user` created a file in the project.
    The kernel has `LANDLOCK_ACCESS_FS_RESOLVE_UNIX`,
     scopes,
     and network rights,
    but `setpriv` 2.41.5 offers only the file rights from `execute` to `truncate`.
  - No process cleanup:
     children survived SIGKILL of the wrapper.
  - `/tmp` and `/dev/shm` are denied,
     so tools that ignore `TMPDIR` fail.
- Using the missing rights would need an in-process applicator:
  the `landlock` crate (a new dependency) or raw `libc` system calls
  (`libc` 0.2.190 is already in `package/ide/app/Cargo.lock` as a transitive package).
  Because `helix-lsp` has no pre-exec hook,
   that shape depends on which thread spawns the server:
  measured with `landlock-thread.py`,
   a child of the restricted thread could not write,
  while a child of another thread of the same process could.
  This thread stops here:
   it needs a dependency decision and still leaves the metadata gap.

### Bubblewrap with Landlock inside

The bubblewrap recipe wrapping `setpriv` with the Landlock rules.
Both servers worked and the tree stayed identical (`combo-rust`,
 `combo-ts`).
It adds a second argument block and blocks nothing the read-only mounts do not already block,
so it is not part of the recommendation.

### Dropped with evidence

- `flatpak-spawn --sandbox`:
   `Portal call failed: org.freedesktop.portal.Flatpak.Spawn only works in a flatpak`.
- `systemd-nspawn --directory=/ --read-only`:
   `Failed to allocate user namespace with 64K users: Timer expired`
  as an unprivileged user.
- Toolbx and Distrobox:
   their manuals describe sharing the home directory with the container
  (`toolbox-create(1)`:
   "providing access to the user's home directory,
   the Wayland and X11 sockets";
  `distrobox-create(1)`:
   "sharing of the HOME directory of the user").
  They are integration tools,
   not confinement.
- Not installed:
   `firejail`,
   `landrun`,
   `nsjail`,
   `minijail0`,
   SELinux `sandbox`.
- A separate user account,
   SELinux policy,
   or immutable attributes need root for every project and were not pursued.

## Cross-candidate results

Labels are run directories under `runs/` in the scratch directory.
Rust runs use the build-script child probe;
 TypeScript runs use the plugin or launcher probe.

### Server function and tree identity

- Unconfined (`base-rust`,
   `base-ts`,
   `base-ts7`):
   all requests correct;
   tree changed as described under "Controls".
- Bubblewrap recipe (`bwrap-rust-final`,
   `bwrap-ts7-final`,
   `bwrap-ts-nopid-nonet-noata`):
  all requests correct,
   including the build-script and proc-macro hovers;
   tree identical.
- `unshare` (`unshare-rust`):
   all requests correct;
   tree identical.
  Both TypeScript servers (`unshare-ts`,
   `unshare-ts7`) answered the hover and definition,
  then exited through their client-process watchdog;
   tree identical.
- systemd,
   default properties (`systemd-rust`,
   `systemd-ts`,
   `systemd-ts7`):
   requests correct;
  tree not identical,
   `ESCAPED_VIA_SYSTEMD_*` added.
- systemd,
   extended properties (`systemd-rust-best`,
   rust-analyzer only):
   requests correct;
   tree identical.
- Podman (`podman-rust`):
   requests correct;
   tree identical.
  Both TypeScript servers (`podman-ts`,
   `podman-ts7`) answered,
   then exited through the watchdog;
   tree identical.
- Landlock (`landlock-rust`,
   `landlock-ts`,
   `landlock-ts7`):
   requests correct;
  tree not identical,
   `ESCAPED_VIA_SYSTEMD_*` added and `victim-meta.txt` mode and time changed.
- Not every combination was run.
  The redirect,
   lock-file,
   toolchain,
   registry-dependency,
   type-acquisition,
   and real-server kill variants ran under bubblewrap only.

### Write and escape probes

Error numbers as reported inside the process tree.

- Create,
   append,
   truncate,
   rename,
   unlink,
   mkdir in the project:
  `EROFS` (30) for bubblewrap,
   `unshare`,
   systemd,
   podman;
   `EACCES` (13) for Landlock.
- `chmod`,
   `utimes`,
   `setxattr` on a project file:
  `EROFS` for bubblewrap,
   `unshare`,
   systemd,
   podman;
   succeeded under Landlock.
- Write in private state:
   succeeded everywhere.
- Write in an unrelated directory under the home directory:
   `EROFS`,
   or `EACCES` under Landlock.
- Write in `/tmp`:
   succeeded into the private directory for bubblewrap,
   `unshare`,
   podman,
   and systemd;
  `EACCES` under Landlock.
- Symbolic link from state into the project,
   then write through it:
   `EROFS`,
   or `EACCES` under Landlock.
- Symbolic link created in the project:
   `EROFS`,
   or `EACCES` under Landlock.
- Hard link from the project into state:
   `EXDEV` everywhere.
- Hard link inside the project:
   `EROFS`,
   or `EACCES` under Landlock.
- Rename of a state file over a project file,
   and of a project file out to state:
  `EXDEV`,
   or `EACCES` under Landlock.
- `/proc/self/root/...`,
   `/proc/self/cwd/...`,
   the `/home/...` spelling,
   and `/proc/self/fd` reopen:
  `EROFS`,
   or `EACCES` under Landlock.
- `/proc/<application pid>/root/...`:
  `ENOENT` with a process-id namespace;
   `EACCES` without one (bubblewrap without `--unshare-pid`,
   systemd,
   Landlock).
- Read-write remount from a nested user namespace:
   the write still failed with `EROFS` everywhere.
  At system-call level inside the bubblewrap recipe (`peel-results.json`),
   as root of a nested namespace:
  `umount2("/run")` gave `EINVAL`,
   remount of `/` and of `/var/home` gave `EPERM`,
  the bus socket stayed invisible,
   and the project write gave `EROFS`.
  The control (a self-made bind mount flipped read-only then read-write on the host side) succeeded,
  so the probe can show a successful remount.
- Double-forked child writing after a delay:
   failed everywhere.
- Mount audit inside the bubblewrap recipe (`mounts-results.json`):
  of 53 mounts,
   40 are read-only.
  The writable ones are the private state,
   `/tmp` (a state subdirectory),
   the private `/dev` tmpfs and its
  device nodes,
   `/dev/pts`,
   the fresh `/proc`,
   the `/run` tmpfs,
  and an `autofs` entry at `/proc/sys/fs/binfmt_misc` that the fresh `/proc` covers.

### Host services still reachable

- Session bus,
   Wayland socket,
   system bus (path sockets under `/run`):
  unreachable (`ENOENT`) for bubblewrap,
   `unshare`,
   and podman;
  unreachable (`EACCES`) for systemd with `InaccessiblePaths`;
  reachable for Landlock and for systemd with default properties.
- `systemd-run --user touch <project file>`:
  failed where the bus is hidden;
   succeeded under Landlock and default systemd.
- Abstract X11 socket `@/tmp/.X11-unix/X0` and a loopback TCP listener:
  reachable whenever the network namespace is shared;
  `ECONNREFUSED` with `--unshare-net` or `PrivateNetwork=yes`.
- Signal 0 to the application process:
  `ESRCH` with a process-id namespace;
   allowed without one.
- `/proc/<application pid>/environ`:
   `ENOENT` with a process-id namespace,
   `EACCES` without one.
- `~/.ssh` listing:
   readable everywhere.
   No candidate restricts reads.
- Network policy is a separate question;
   see "Open decisions for the coordinator".

### Kill behavior

`kill.mjs` starts a payload with one ordinary child and one double-forked child,
 then sends SIGKILL to the wrapper,
which is all `kill_on_drop` does.

- No wrapper:
   both children survive.
- Bubblewrap with a process-id namespace:
   nothing survives.
- Bubblewrap without one:
   the server dies,
   both children survive.
- Landlock:
   both children survive.
- `unshare`:
   nothing survives.
- systemd:
   the unit stays `active`;
   the payload and both children survive.
- Podman:
   the container stays `Up`;
   `conmon`,
   the payload,
   and both children survive.

With real servers:

- rust-analyzer under the recipe,
   SIGKILL of the wrapper:
   nothing survives (`bwrap-rust-kill`).
- TypeScript 7 without a process-id namespace and with type acquisition off:
   nothing survives (`bwrap-ts7-kill`).
- Either TypeScript server without a process-id namespace and with type acquisition on:
  `npm install types-registry@latest` survived in four of seven such runs,
  including after a graceful shutdown.

`pdeathsig-results.json` shows what `--die-with-parent` does:

- Without it,
   SIGKILL of `bwrap` leaves the sandbox running,
   so it is required for `kill_on_drop` to work.
- With it,
   the sandbox also dies when the application process dies.
- With it,
   the sandbox is killed when only the spawning thread exits while the process lives on.
  This matches `PR_SET_PDEATHSIG(2const)` ("the thread that created this process")
  and `kernel/exit.c:746` to `:755` at `v7.2`.

### Process-id namespace and server watchdogs

- `typescript-language-server` 6.0.0 polls `process.kill(processId, 0)` every 3 s and exits with status 1 when it fails
  (`lib/cli.mjs:12854` to `:12864`).
  Inside a process-id namespace it exited 3.2 s after start (`bwrap-ts`).
  With `processId: null` in `initialize` it ran normally (`bwrap-ts-nullpid`),
   which confirms the cause.
  Its command line rejects `--clientProcessId` (`error: unknown option '--clientProcessId=1'`).
- TypeScript 7.0.2 printed `Parent process <pid> has exited, shutting down.`
  and exited 5.2 s after start (`bwrap-ts7`).
- rust-analyzer ran normally in a process-id namespace.
- The same failure appeared under `unshare` and podman,
   which also create a process-id namespace
  (`unshare-ts`,
   `podman-ts`;
   `unshare-ts7` at 5.3 s and `podman-ts7` at 5.5 s with the same message).
- Without a process-id namespace the user namespace still denied `/proc/<pid>/root` and `/proc/<pid>/environ`.

### Launch overhead

`overhead.mjs` times `<wrapper> /usr/bin/true`,
 interleaved round-robin,
 with two bare series for spread.
Two runs,
 20 and 15 rounds,
 under the load stated in "Host facts".

- Bare `/usr/bin/true`:
   minimum 1.6 to 1.8 ms;
   median 2.7 to 7.0 ms;
   maximum 20 to 43 ms.
  The two bare series differ by up to 1.9 ms in median within one run,
  and single launches reach 43 ms,
   so differences of a few milliseconds are noise.
- Landlock through `setpriv`:
   minimum 2.5 to 3.0 ms;
   median 3.7 to 9.6 ms.
- Bubblewrap recipe:
   minimum 9.7 to 13.7 ms;
   median 20 to 64 ms. With and without `--unshare-net` are within noise.
- `unshare` script:
   minimum 19 to 20 ms;
   median 28 to 82 ms.
- `systemd-run --user`:
   minimum 53 to 54 ms;
   median 75 to 349 ms.
- Podman:
   minimum 176 to 204 ms;
   median 352 to 2016 ms.

For scale:
wrapper included,
the bubblewrap sessions reached a correct hover in 0.1 to 1.1 s for TypeScript 7
and in 4 to 29 s for rust-analyzer.

### Failure when the mechanism is unavailable

From `unavailable.mjs`;
 each case has a positive control showing the inner command runs when the mechanism works.
In every case the inner command did not run.

- Wrapper executable missing:
   spawn fails with `ENOENT`.
  In Helix,
   `which` fails first with `ExecutableNotFoundError` (`client.rs:228`).
- Bubblewrap with user namespaces denied (simulated inside `bwrap --disable-userns`):
   status 1,
  `bwrap: Creating new namespace failed: nesting depth or /proc/sys/user/max_*_namespaces exceeded (ENOSPC)`.
- `unshare` in the same context:
   status 1,
   `unshare: unshare failed: No space left on device`.
- Landlock system calls answering `ENOSYS` (simulated with a systemd system-call filter):
   status 127,
  `setpriv: landlock_create_ruleset failed: Function not implemented`.
- Private state directory missing:
   bubblewrap status 1,
   `bwrap: Can't find source path .../tmp`;
  `setpriv` status 1,
   `could not open file for landlock`.
- User bus unreachable:
   `systemd-run` status 1,
   `Failed to connect to user scope bus`.
- Image missing:
   podman status 125,
   `image not known`.

## Ranking

Ranking:
 bubblewrap > `unshare` script > systemd transient user unit > podman > Landlock through `setpriv`.

- Bubblewrap over the `unshare` script:
  both gave the same measured protection,
  but bubblewrap is one audited executable with declarative arguments,
   a private `/dev`,
   and `no_new_privs`,
  while the script is a hand-written copy of it inside an argument list.
- `unshare` script over the systemd unit:
  the script keeps the Helix process model intact (inherited environment,
   kill by SIGKILL),
  while the unit ignores the caller's environment,
   picked up a stale variable that broke rust-analyzer,
  and kept running after its client was killed.
- systemd unit over podman:
  the unit runs the host's servers directly,
  while podman needs an image that carries every server and runtime,
   same-path mounts,
  and an explicit working directory,
  and launches several times slower.
   Both leak on client kill.
- Podman over Landlock through `setpriv`:
  podman kept the tree identical under every probe,
  while Landlock left project metadata writable and delegation through the session bus open,
  so it fails the requirement by itself.

## Recommended launch shape

### Private state directories

One directory per project and server,
 outside the project,
 created by the application before launch:

- `STATE`,
   for example `${XDG_CACHE_HOME:-$HOME/.cache}/monochromatic-ide/language/<project id>/<server>`;
- `STATE/tmp`,
   which becomes `/tmp` inside;
- `STATE/target` and `STATE/build` for cargo;
- `STATE/cache` and `STATE/npm-cache` for Node tools.

`STATE` and `STATE/tmp` must exist or bubblewrap refuses to start.
`STATE` must be neither inside the project nor an ancestor of it:
with the parent of the project as state,
a write into the project succeeded (`extra-results.json`,
 `stateContainsProject`).
Compare canonical paths.

### Wrapper arguments

`command` is `/usr/bin/bwrap`.
 `args` is this list,
 then the server's own arguments:

```text
# Helix language-server `args`; STATE and SERVER are absolute paths filled in by the application
--die-with-parent
--new-session
--unshare-user
--unshare-pid
--unshare-ipc
--unshare-uts
--unshare-cgroup
--unshare-net
--ro-bind / /
--dev /dev
--proc /proc
--tmpfs /run
--bind STATE/tmp /tmp
--bind STATE STATE
--
SERVER
```

Why each part is there:

- `--ro-bind / /`:
   everything is read-only,
   so no argument depends on the workspace root Helix picks,
  and aliases of the project path are covered.
- `--dev /dev`:
   with only the read-only bind,
   writing `/dev/null` fails with `Permission denied` (measured).
- `--proc /proc`:
   a fresh `/proc` for the process-id namespace.
- `--tmpfs /run`:
   hides the session bus,
   the system bus,
   and the other path sockets under `/run`.
- `--bind STATE/tmp /tmp`:
   tools that hard-code `/tmp` keep working without touching the shared `/tmp`.
- `--bind STATE STATE`:
   the only writable host directory.
   It comes after the read-only bind on purpose.
- `--unshare-pid` with `--die-with-parent`:
   SIGKILL of the wrapper removes the whole tree.
  Omit `--unshare-pid` for servers that watch the client process id.
- `--unshare-net`:
   closes abstract sockets and loopback services.
   It is the subject of the network decision.
- `--new-session`:
   no controlling terminal inside.

### Environment

In the Helix `environment` map:

- all servers:
   `XDG_CACHE_HOME=STATE/cache`,
   `npm_config_cache=STATE/npm-cache`;
- rust-analyzer:
   `CARGO_TARGET_DIR=STATE/target`,
   `CARGO_BUILD_BUILD_DIR=STATE/build`.

### Per-server settings

#### rust-analyzer

- Full argument list,
   including `--unshare-pid`.
- `CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR` as stated under "Environment".
  The rust-analyzer setting `cargo.targetDir` also works in place of `CARGO_TARGET_DIR` (`bwrap-rust-cfgtarget`).
- The host cargo home and rustup home stay read-only.
- Verified by `bwrap-rust-final`:
   all three hovers and the definition correct,
   tree identical,
  build script and both proc-macro hosts got error 30 on the project and success in state and `/tmp`,
  no survivor.

#### TypeScript 7 native server

- Argument list without `--unshare-pid`.
- `SERVER` is the absolute path of the project's `node_modules/typescript/bin/tsc`,
   followed by `--lsp --stdio`.
- With no network:
   turn automatic type acquisition off.
  Each of these settings stopped it in the driver,
   which answers `workspace/configuration` per dotted section
  as `helix-lsp` does (the server asks for `js/ts`,
   `typescript`,
   `javascript`,
   `editor`):
  `typescript.tsserver.automaticTypeAcquisition.enabled = false`;
  `"js/ts".tsserver.automaticTypeAcquisition.enabled = false`;
  `typescript.disableAutomaticTypeAcquisition = true` together with the same key under `javascript`.
  Controls with no setting and with an unrelated key both ran `npm`.
- Verified by `bwrap-ts7-final`:
   hover and definition correct,
   tree identical,
   launcher probe fully blocked,
  nothing written to `STATE/cache`,
   `STATE/npm-cache`,
   or `STATE/tmp`,
   no survivor.

#### `typescript-language-server` 6.0.0

Measured as a second Node server;
 the language-intelligence plan does not use it.

- Argument list without `--unshare-pid`.
- `disableAutomaticTypingAcquisition: true` in the initialization options when there is no network.
- Verified by `bwrap-ts-nopid-nonet-noata`:
   hover and definition correct,
   tree identical,
   plugin probe fully blocked,
  no survivor.

### Construction rules for the application

- Build the `LanguageServerConfiguration` values in code from Helix's built-in defaults.
  Never merge a workspace `.helix/languages.toml`,
   and do not merge the user's Helix `languages.toml` either.
- Resolve the server executable to an absolute path before placing it after `--`.
- Start servers from a thread that lives as long as the server
  (for example the dedicated language worker thread of the language-intelligence plan),
  never from a short-lived or pooled blocking thread,
   because of `--die-with-parent`.
- If the wrapper cannot start,
   report the start failure.
   Never fall back to an unconfined launch.
- Clear `STATE/tmp` before each launch.
- Nothing outside the confinement may execute or trust files from `STATE`.

## Writable needs and redirects

Each line states what the tool wanted and what the confined run showed.

- Cargo target directory:
   `CARGO_TARGET_DIR` or `cargo.targetDir`.
   Both verified.
  With neither,
   rust-analyzer logged
  `FetchBuildDataError: error: Read-only file system (os error 30) at path ".../project/target<random>"`;
  the plain hover still worked,
   the build-script and proc-macro hovers did not (`bwrap-rust-noredirect`).
- Cargo build directory:
   `CARGO_BUILD_BUILD_DIR`.
  Needed on this host because the cargo configuration places it under the cargo home;
  without it cargo failed with ``failed to create directory `/home/<user>/.cargo/build/94` ``
  (`bwrap-rust-nobuilddir`).
- Cargo home locks:
   `.package-cache`,
   `.package-cache-mutate`,
   and `.global-cache` all failed with `EROFS`
  and cargo carried on.
  A project with a registry dependency that is already downloaded and extracted
  worked with the read-only host cargo home and no network (`bwrap-rust-dep`).
- Private cargo home (`CARGO_HOME=STATE/cargo-home` with a copy of `config.toml`):
  with network and the resolver directory bound,
   cargo downloaded into state and everything worked
  (`bwrap-rust-dep-private-net`);
  without network the build-script and proc-macro hovers failed (`bwrap-rust-dep-private-nonet`).
- `Cargo.lock` absent:
   rust-analyzer still answered all three hovers,
  but its `cargo check` failed with `failed to write .../Cargo.lock ... Read-only file system`,
  so `rustc` diagnostics are missing for such projects (`bwrap-rust-nolock`).
   No setting redirects this.
- rustup:
   read-only is enough when the toolchain is installed.
  A project `rust-toolchain.toml` naming an uninstalled toolchain made the proxy try to install it and exit:
  `error: could not create temp file /home/<user>/.rustup/tmp/...: Read-only file system` (`bwrap-rust-toolchain`).
- rust-analyzer temporary directories and linker temporaries:
   the private `/tmp`.
- Type-acquisition cache:
   `XDG_CACHE_HOME`;
   both TypeScript servers wrote `STATE/cache/typescript/<version>`.
- npm cache and logs:
   `npm_config_cache`;
   logs landed in `STATE/npm-cache/_logs`.
- `node-compile-cache`:
   the private `/tmp`.
- tsserver logs:
   none were written by default in any run.
- `XDG_RUNTIME_DIR` points at a path that does not exist inside.
   Neither server needed it.

## Open decisions for the coordinator

### Network access for confined servers

This is a policy question;
 the measurements only show the consequences.

- Option A:
   no network (`--unshare-net`).
  - Pros:
     closes the abstract X11 socket and loopback services,
    the two delegation channels still measured as reachable;
    nothing readable can be sent out;
     works for both servers when dependencies are already present.
  - Cons:
     cargo cannot fetch missing crates,
     and type acquisition cannot run,
     so it should be switched off.
- Option B:
   host network,
   with `--ro-bind /run/systemd/resolve /run/systemd/resolve` after `--tmpfs /run`
  so name resolution works.
  - Pros:
     type acquisition worked into private state (`bwrap-ts-net`);
    with a private cargo home,
     cargo fetched missing crates (`bwrap-rust-dep-private-net`).
  - Cons:
     the abstract X11 socket and every loopback service stay reachable,
     so delegated writes are not excluded;
    readable secrets can leave the machine;
     `npm` can outlive the server without a process-id namespace.
- Ranking:
   A > B,
   because the scope makes "no delegated writes" a hard requirement
  and treats richer analysis as best effort.

### Process-id namespace default

- Option A:
   on by default,
   off for a measured list (currently both TypeScript servers).
  - Pros:
     clean kill and no signals to host processes for every server that tolerates it.
  - Cons:
     each new server needs the liveness test before it is trusted.
- Option B:
   off for every server.
  - Pros:
     one argument list;
     no watchdog surprises.
  - Cons:
     children survive SIGKILL of the wrapper (measured),
     which matters most for cargo trees;
    confined code can signal the user's other processes.
- Ranking:
   A > B,
   because rust-analyzer is the server with heavy child trees and it works with the namespace.

### Adopted on 2026-10-05

The coordinating session adopted the bubblewrap launch shape with the first-ranked option for both questions.
All three choices were reported to the user as open to veto.

- Mechanism:
  the bubblewrap wrapper from "Recommended launch shape".
  `/usr/bin/bwrap` is already on the host,
  so nothing new is installed.
  No formal technology-vetting run was made,
  following the user's proportionality direction of 2026-10-05;
  the five-candidate measurement in this document is the evidence.
- Network:
  option A,
  no network,
  with automatic type acquisition switched off.
- Process-id namespace:
  option A,
  on by default and off for the measured TypeScript servers.
- The user confirmed all three on 2026-10-05:
  bubblewrap as measured without a formal vetting run,
  and the process-id namespace except for the TypeScript servers.
  On network access the user answered:
  "Why would language servers have network access?
  That (giving language servers network access) would be extremely bad and against any kind of intuition."
  No network is therefore a requirement,
  not a preference;
  option B is rejected.
- Follow-up for the implementation leg:
  test clearing the inherited environment (`--clearenv` plus an allowlist),
  because servers otherwise inherit credential variables from the application;
  keep it only if both real servers still pass their checks.

## Residual risks

- Reads are unrestricted.
   `~/.ssh` was listable in every run,
  and the servers inherit the application environment,
  which on this host holds credential variables (for example `MISE_GITHUB_TOKEN` and API key variables).
  Bubblewrap has `--clearenv` and `--unsetenv`;
   they were not tested here.
- Project-controlled code execution is intrinsic to these servers:
  `build.rs`,
   proc macros,
   `.cargo/config.toml`,
   `rust-toolchain.toml`,
   tsserver plugins,
  and the TypeScript 7 launcher itself.
   Confinement bounds writes,
   not execution.
- Private state is writable and reused,
   so a hostile project can poison its own build outputs there.
  That stays inside the confinement only while nothing outside runs or trusts those files.
- With the network namespace shared,
   abstract sockets and loopback services are reachable.
- Without a process-id namespace,
   confined code can signal the user's processes,
   and children can outlive the server.
- The project path must be on a mount that existed at launch.
  Not measured,
   because host mounts need root:
  bubblewrap marks its mounts `MS_SLAVE | MS_REC` (`bubblewrap.c:3240`),
  so a file system mounted on the host after launch may appear inside without the read-only flag.
- `--die-with-parent` kills the server when the spawning thread ends.
- User namespaces are required.
   If the host disables them,
   servers fail to start (closed,
   with a clear message).
- The application process itself,
   and its `rg` search subprocess,
   are outside this confinement.
- Functional limits of a read-only,
   offline run:
   no `cargo check` diagnostics without a committed `Cargo.lock`;
  no start when the project pins an uninstalled toolchain;
   no fetching of missing dependencies.
- The servers answered correctly on tiny fixtures only.
   Large workspaces were not exercised.

## Acceptance tests the application must carry

All on disposable fixtures and a disposable home,
 through the application's real start path.

- Configuration shape:
   for each server,
   the generated `command`,
   `args`,
   and `environment` equal the expected lists;
  no workspace-derived token appears except `STATE`,
  the read-only project bind-back
  (`--ro-bind PROJECT PROJECT`,
  plus Helix's `PWD` spelling when it lies below a replaced directory),
  which the user required on 2026-10-05 so projects below `/tmp` and `/run` work,
  and the project's own TypeScript 7 launcher after `--`,
  which the adopted TypeScript server requires;
   the server path is absolute.
- Rust write denial:
   a fixture whose `build.rs` and proc macro try to write the project,
   state,
   and `/tmp`.
  Expect error 30 for the project and success for state;
  hovers on the build-script constant and the proc-macro function succeed;
  the project tree snapshot (paths,
   sizes,
   hashes,
   modes,
   times) is identical after `initialize` to `exit`.
- TypeScript write denial:
   the same with a probe in the project-supplied launcher.
- Escape set from inside the tree:
   symbolic link in both directions,
   hard link in both directions,
  rename over and out,
   `/proc/self/root`,
   `/proc/<application pid>/root`,
   the other home spelling,
  `chmod`,
   `utimes`,
   `setxattr`,
   nested-namespace remount.
   All must fail.
- Delegation:
   `systemd-run --user touch <project file>` from inside fails;
   the bus sockets are absent.
  With the no-network choice,
   the abstract X11 socket and a loopback listener refuse.
- Guard control:
   the same fixtures with the wrapper removed must change the tree,
  otherwise the tests prove nothing.
- Kill:
   after the client is dropped,
   no process carrying the session marker remains.
- Liveness:
   every configured server still answers a request 10 s after `initialize`
  (this catches a client-process watchdog),
  including when the thread that requested the start has since finished.
- Fail closed:
   with `STATE` missing,
   and with user namespaces denied
  (`bwrap --unshare-user --disable-userns` around the launch),
  the server is reported as failed to start and no unconfined process appears.
- State containment:
   a state path inside the project,
   and one containing the project,
   are both rejected;
  a sibling path is accepted.
- Mount audit:
   `/proc/self/mountinfo` inside the recipe has no read-write mount outside the expected list.
- No real-state pollution:
   the disposable home's cargo,
   cache,
   and npm directories are unchanged after a session.
- Type acquisition:
   with the no-network choice,
   `STATE/cache` holds no `typescript` directory after a session
  that opened a JavaScript file with an untyped import.

## Evidence and reproduction

Scratch directory:
 `~/temp/agent/ide-confine.JNN2oC` (it can vanish;
 the key excerpts are inlined here).

### Files

- `make-fixtures.mjs`,
   `make-variants.mjs`,
   `make-ts7.mjs`:
   fixture templates.
- `lib.mjs`:
   snapshot and diff,
   wrapper builders,
   LSP client.
   `session.mjs`:
   one session.
   `batch.mjs`:
   several.
- `probe.cjs`:
   the Node probe.
   `kill-tree.cjs`,
   `kill.mjs`:
   kill behavior.
- `overhead.mjs`,
   `unavailable.mjs`,
   `mounts.mjs`,
   `extra.mjs`,
   `peel.py`,
   `pdeathsig-thread.py`,
   `landlock-thread.py`.
- `show.mjs`,
   `show-probe.mjs`,
   `strace-summary.mjs`,
   `matrix.mjs`:
   digests.
- Results:
   `runs/<label>/result.json`,
   `runs/<label>/full-diff.json`,
   `runs/<label>/state/report/*`,
  `runs/<label>/strace-summary.json`,
   `kill-results.json`,
   `overhead-results-run1.json`,
   `overhead-results-run2.json`,
  `unavailable-results.json`,
   `mounts-results.json`,
   `extra-results.json`,
   `peel-results.json`,
  `pdeathsig-results.json`,
   `matrix.txt`,
  `systemd-rust-stale-toolchain.result.json`,
   `podman-rust-no-libatomic.result.json`.
- Sources fetched for reading:
   `bubblewrap-0.12.0.c`,
   `linux-7.2-exit.c`.

### Commands

```text
# run from ~/temp/agent/ide-confine.JNN2oC
node make-fixtures.mjs
node make-variants.mjs
node make-ts7.mjs /var/home/user/Monochromatic
node session.mjs enum-rust-nolock enumerate rust '{"removeLock":true,"redirects":false,"probes":false,"strace":true}'
node session.mjs enum-ts enumerate ts '{"redirects":false,"probes":false,"strace":true}'
node session.mjs enum-ts7 enumerate ts7 '{"redirects":false,"probes":false,"strace":true,"settleMs":10000}'
node session.mjs base-rust none rust '{"cargoHome":"private","tmpdir":true,"strace":true,"delegate":true}'
node session.mjs base-ts none ts '{"tmpdir":true,"strace":true,"delegate":true}'
node session.mjs bwrap-rust-final bwrap rust '{"envTargetDir":true,"buildDir":true,"delegate":true,"mech":{"unshareNet":true}}'
node session.mjs landlock-rust landlock rust '{"envTargetDir":true,"buildDir":true,"delegate":true}'
node session.mjs unshare-rust unshare rust '{"envTargetDir":true,"buildDir":true,"delegate":true}'
node session.mjs systemd-rust systemd rust '{"envTargetDir":true,"buildDir":true,"delegate":true,"mech":{"properties":["--property=UnsetEnvironment=RUSTUP_TOOLCHAIN"]}}'
node session.mjs bwrap-ts7-final bwrap ts7 '{"delegate":true,"mech":{"unshareNet":true,"noPid":true},"settleMs":8000,"configExtra":{"typescript":{"tsserver":{"automaticTypeAcquisition":{"enabled":false}}}}}'
node session.mjs bwrap-ts7 bwrap ts7 '{"delegate":true,"mech":{"unshareNet":true},"settleMs":10000}'
node kill.mjs
node overhead.mjs 20
node unavailable.mjs
node mounts.mjs
node extra.mjs
python3 pdeathsig-thread.py
node show.mjs <label>
node show-probe.mjs <label>
node strace-summary.mjs <label>
node matrix.mjs base-rust bwrap-rust-final bwrap-rust landlock-rust systemd-rust systemd-rust-best unshare-rust podman-rust
```

`make-fixtures.mjs` recreates `template/`,
 so the variant scripts run after it.
`template/rs-dep/Cargo.lock` was copied from the enumeration run `enum-rust-dep`.
The podman run needs `RUSTUP_HOME` and `CARGO_HOME` passed through `extraEnv`.
The runs `landlock-ts7`,
`systemd-ts7`,
`unshare-ts7`,
and `podman-ts7` use the options of `bwrap-ts7-final` without its `mech` entry.

### Script excerpts

The wrapper builder that produced every bubblewrap result:

```js
// ~/temp/agent/ide-confine.JNN2oC/lib.mjs
bwrap: (ctx, command, args, options = {}) => ({
  command: '/usr/bin/bwrap',
  args: [
    '--die-with-parent', '--new-session',
    '--unshare-user', ...(options.noPid ? [] : ['--unshare-pid']), '--unshare-ipc', '--unshare-uts', '--unshare-cgroup',
    ...(options.unshareNet ? ['--unshare-net'] : []),
    '--ro-bind', '/', '/',
    '--dev', '/dev',
    '--proc', '/proc',
    '--tmpfs', '/run',
    '--bind', join(ctx.state, 'tmp'), '/tmp',
    '--bind', ctx.state, ctx.state,
    ...(options.extra ?? []),
    '--', command, ...args,
  ],
  env: {},
}),
```

The tree snapshot behind every "tree identical" statement:

```js
// ~/temp/agent/ide-confine.JNN2oC/lib.mjs
const stat = lstatSync(absolute, { bigint: true });
const entry = {
  type: stat.isDirectory() ? 'dir' : stat.isSymbolicLink() ? 'symlink' : stat.isFile() ? 'file' : 'other',
  size: stat.isDirectory() ? null : String(stat.size),
  mode: (Number(stat.mode) & 0o7777).toString(8),
  mtimeNs: String(stat.mtimeNs),
  ctimeNs: String(stat.ctimeNs),
  nlink: stat.isDirectory() ? null : String(stat.nlink),
  ino: String(stat.ino),
};
if (entry.type === 'file') entry.sha256 = createHash('sha256').update(readFileSync(absolute)).digest('hex');
if (entry.type === 'symlink') entry.target = readlinkSync(absolute);
```

The Rust probe shared by `build.rs` and the proc macro:

```rust
// ~/temp/agent/ide-confine.JNN2oC/template/rs/probe_shared.rs
for (label, var) in [
    ("project", "IDE_PROBE_PROJECT"),
    ("state", "IDE_PROBE_STATE"),
    ("outside", "IDE_PROBE_OUTSIDE"),
    ("tmp", "IDE_PROBE_TMP"),
] {
    let Ok(dir) = std::env::var(var) else { continue };
    let path = format!("{dir}/PROBE_{who}_{label}");
    let outcome = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(b"x"));
    let text = match outcome {
        Ok(()) => "ok".to_string(),
        Err(error) => format!("errno={:?} kind={:?}", error.raw_os_error(), error.kind()),
    };
    // one JSON line per target is appended to $IDE_PROBE_REPORT_DIR/rust-probe.jsonl
}
```

Part of the Node probe:

```js
// ~/temp/agent/ide-confine.JNN2oC/probe.cjs
push(attempt('project.chmod', () => fs.chmodSync(victim('victim-meta.txt'), 0o600)));
push(attempt('project.utimes', () => fs.utimesSync(victim('victim-meta.txt'), 1, 1)));
push(attempt('escape.symlink-into-project.write', () => {
  const link = path.join(state, `PROBE_${tag}_symlink`);
  fs.symlinkSync(victim('victim-link.txt'), link);
  fs.appendFileSync(link, 'x');
}));
push(attempt('alias.proc-self-root', () => fs.appendFileSync(path.join('/proc/self/root', victim('victim-append.txt')), 'x')));
push(run('delegate.systemd-run-user', 'systemd-run', ['--user', '--wait', '--collect', '--quiet', '--',
  '/usr/bin/touch', path.join(project, `ESCAPED_VIA_SYSTEMD_${who}`)]));
push(await connectProbe('reach.unix.abstract-x11', { path: '\0/tmp/.X11-unix/X0' }));
```

The Landlock arguments that were measured:

```text
# argv built by the `landlock` entry in ~/temp/agent/ide-confine.JNN2oC/lib.mjs
/usr/bin/setpriv --no-new-privs
  --landlock-access fs:write-file,remove-dir,remove-file,make-char,make-dir,make-reg,make-sock,make-fifo,make-block,make-sym,refer,truncate
  --landlock-rule path-beneath:write-file,remove-dir,remove-file,make-dir,make-reg,make-sock,make-fifo,make-sym,refer,truncate:STATE
  --landlock-rule path-beneath:write-file:/dev/null
  -- SERVER
```
