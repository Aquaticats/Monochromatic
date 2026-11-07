# search-shadow

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

Shadows the tools that walk many files:

- `rg`.
- The grep family:
  `grep` / `egrep` / `fgrep` / `ugrep` / `ug` / `ag` / `ack`.
- The finders:
  `find` / `fd` / `fdfind` / `bfs` / `tree`.
- The counters:
  `du` / `ncdu` / `tokei` / `cloc`.

Started under one of those names,
the binary lowers its own disk priority to best-effort level 7
(the same as `ionice -c2 -n7`,
set directly with the `ioprio_set` system call)
and replaces itself with the real tool,
which keeps the priority.
So it adds no process.
Exit codes,
signals and terminal behavior are kept.
Started as `search-shadow`,
it offers the `link` command and the `--help` and `--version` flags.
The last two say plainly that it is a shadow and list the tools it shadows.
When a shadowed tool is asked for its version or help,
the shadow says so (see "Version and help" below).

## Why

A broad search reads hundreds of thousands of files.
On this host's DRAM-less NVMe,
concurrent cold readers raise everyone's read wait
from about 1 ms to about 8 ms (median) with 100 to 200 ms tails,
which stalls the desktop.
The measurements are in `doc/planning/agent-search-io-throttle.md`.

## What it does

- Runs the real tool at best-effort level 7.
  The class is honored under the BFQ and mq-deadline schedulers,
  which `/etc/udev/rules.d/61-nvme-bfq.rules` sets on `nvme0n1`.
  Under kyber it is accepted and ignored.
  If the kernel refuses the call,
  the tool runs at normal priority.
  This is the crate's only `unsafe` code,
  in `src/priority.rs`.
- Refuses the short flag `-r` for `rg` anywhere with exit code 2.
  In ripgrep `-r` is `--replace`,
  not grep's "recursive":
  `rg -rl PATTERN` replaces every match with the text `l` and lists no files.
  ripgrep already recurses,
  so drop `-r`.
  Write `--replace TEXT` for real replacement (still allowed).
  There is no escape hatch.
- Refuses `rg` with `--hidden` / `--no-ignore*` / `-u` / `-.`,
  no search path and no piped input,
  when run from a git repository root.
  The exit code is 2,
  and the message names the command and the directory,
  then gives two ways forward:
  name a narrower path,
  or set `SEARCH_SHADOW_ALLOW_WIDE=1` for one deliberate full walk.
- Refuses a recursive `grep` / `egrep` / `fgrep`
  (`-r` / `-R` / `--recursive` / `--dereference-recursive` / `-d recurse`)
  that names no path or names a wide one.
  Also refuses `find` or `bfs` that start at a wide path without `-maxdepth` of 3 or less.
  A wide path is a git repository root,
  the home directory or the filesystem root.
  The message suggests a narrower directory,
  `rg PATTERN PATH` or `rg --files PATH`,
  and the same `SEARCH_SHADOW_ALLOW_WIDE=1` escape.
- Only slows down the other tools
  (`ugrep` / `ug` / `ag` / `ack` / `fd` / `fdfind` / `tree` / `du` / `ncdu` / `tokei` / `cloc`).
  `fd` is never refused.
  Scripts that relied on plain GNU or ripgrep behavior for a wide walk will stop.
  That is deliberate.
- Adds one line after a lone `--version` and one section after a lone `--help` (see "Version and help").
- Never starts a shadow as the real tool, of any version (see "Never starts a shadow").
- Passes arguments through byte for byte,
  including ones that are not valid UTF-8.

In Claude Code shells,
`grep` and `find` are shell functions that run Claude's embedded `ugrep` and `bfs`.
Functions win over PATH,
so the links reach `rg` there but not those two.

## Never starts a shadow

The owner's rule:
"We should never invoke ourselves recursively."
Two copies of `search-shadow` at different paths (a worktree's links and the main checkout's, for one)
used to choose each other as the real `grep`
and replace themselves with each other without end:
measured 2026-11-07 at about 105 starts per second under `strace`,
at a quarter of one CPU
(`doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`).
A path or a version says nothing about what a file is,
so the rule has three parts that do not depend on either:

- Identity.
  Every shadow program (`cargo-shadow`, `mpm-shadow`, `pnpm-shadow` and this one) carries the same text in an ELF section of its own,
  `.monochromatic_shadow_identity`:
  `MONOCHROMATIC-SHADOW-IDENTITY/1 <crate> <version>`.
  The lookup of the real tool skips a candidate that carries it,
  of any version and under any name,
  without starting it.
  A candidate with a launcher record beside it (`<launcher>.launch`, written by `mpm-shadow`)
  is judged by the program the record names,
  through further launchers (at most eight, a longer chain counts as a shadow);
  a launcher is a copy of `mpm-shadow`, which carries the text, so its own bytes never decide.
  A candidate that cannot be read, or a record that names a missing file, is no verdict.
- The start.
  The chosen program is opened once and checked while open,
  and a compiled program is started through that open file (`/proc/self/fd/<n>`),
  so a path made to name a shadow after the check cannot reach the start.
  A script, an unreadable program or a launcher is started by its path.
- Arrival.
  Every start of the real tool sets `MONOCHROMATIC_SHADOW_STARTED_BY` to the starter's mode (`exec` or `spawn`),
  process number, birth identity (the start time in `/proc/<pid>/stat`) and crate name and version.
  A shadow that finds the marker left for its own process (after `exec`) or for its direct parent (after a child start)
  was started by a shadow:
  it does nothing,
  prints one line naming both shadows and the tool,
  and exits with status 2.
  This stops what no check of a file sees (a script that runs a shadow, a launcher with no record).
  A script the real tool runs is further down and passes,
  so `grep` called from inside `rg`'s own children still works.
- Cost.
  Measured 2026-11-07 with `hyperfine` (500 runs per arm, `rg x` through a link to `true`, run under `env`):
  mean 2.2 ms for 0.2.2 against 2.3 ms for this change,
  and 2.1 ms against 2.4 ms in the repeat,
  so the added work is about 0.2 ms.
  The program file of every shadow is read in a few small reads whatever its size.

The same text,
the same check and the same marker are in the other three shadows
(`package/cargo-shadow/cli`,
`package/mpm-shadow/cli`,
`package/pnpm-shadow/cli`),
as byte-identical copies of `src/shadow_identity.rs`:
the four crates share no dependency,
and `tests/shadow_identity_copies.rs` fails when the copies differ.

## Version and help

A shadow is invisible by default,
so `rg --version` would print only ripgrep's version.
When the whole argument list is exactly one version flag or one help flag of the tool,
the shadow runs the real tool as a child (not by replacing itself),
waits,
and then adds text after the real tool's own output.
The real tool's standard output,
standard error and exit status are otherwise untouched:
`ugrep --help` still exits 2,
and a tool ended by a signal gives the shell's `128 + signal`.
The text is added whatever the status,
and the policies run first,
so a refused run prints only the refusal.

The version flag adds one standard-output line,
in the style of `hub` after git's own version line:

```text
# search-shadow output, after the real ripgrep's version output
search-shadow 0.2.0, shadowing rg at /usr/bin/rg
```

The path is the real binary found on PATH (made absolute,
symlinks not resolved),
the one `command -v` would show without the shadow.
The help flag adds a blank line,
the same header line,
and a short section that says the help above is the real tool's own
and what the shadow adds for that tool:

- `rg`:
  the `rg -r` refusal and the wide-walk refusal.
- `grep` / `egrep` / `fgrep`:
  the wide recursive refusal.
- `find` / `bfs`:
  the `-maxdepth` refusal.
- Every other tool:
  "never refuses a run".

Only a lone flag counts.
`rg --version foo`,
`rg --no-config --help` and every other argument list pass through untouched.
That keeps flags that mean something else from being taken for a request.
The flags are read off each real tool's own command line:

- `rg` / `fd` / `fdfind` / `ag` / `tokei`:
  version `--version` or `-V`;
  help `--help` or `-h`.
- `ncdu`:
  version `--version` or `-V`;
  help `--help` or `-h` or `-?`.
- `grep` / `egrep` / `fgrep`:
  version `--version` or `-V`;
  help `--help` (`-h` means no file names).
- `ugrep` / `ug`:
  version `--version` or `-V`;
  help `--help` or `-?` (`-h` means no file names).
- `find` / `bfs`:
  version `--version` or `-version`;
  help `--help` or `-help`.
- `ack` / `tree` / `du` / `cloc`:
  version `--version`;
  help `--help`.
  `-h` is not a help flag for these:
  it means no file names for `ack`,
  human-readable sizes for `tree` and `du`,
  and an ambiguous option for `cloc`.

`search-shadow --version` prints `search-shadow <version>, a shadow of <every shadowed tool>`,
and `search-shadow --help` opens by saying it is a shadow and not a tool of its own.

## Use

```bash
# Link every shadowed tool whose real binary is on PATH, then put the directory first on PATH.
search-shadow link .cache/search-shadow-bin
```

`link` links a name only when a real tool of that name is found on PATH (not counting a shadow of any version),
removes a link whose real tool has gone,
and reports what it linked,
removed and skipped.
Run it again after installing or removing a tool.

In this repository it is installed by `mpm-shadow` from the toolchain plan
(entry `cargo:monochromatic-search-shadow` in `toolchain.plan.jsonc`):
`mise run toolchain:install` runs `cargo binstall` for the locked version,
and `.config/mise/conf.d/20-env.toml` puts `.cache/search-shadow-bin` first on PATH,
ahead of `.cache/toolchain-bin`,
which holds the locked ripgrep.

## Exit codes

- `2`:
  the policy refused the walk (or a bad `search-shadow` command line).
- `2` also:
  a shadow started this one as the real tool (the arrival stop above).
- `126`:
  the real tool was found but could not be started
  (or, checked while open, turned out to be a shadow).
- `127`:
  no real tool on PATH.
- Anything else:
  the real tool's own exit code.

## Checks

```bash
mise run //package/search-shadow/cli:lint
mise run //package/search-shadow/cli:lint:clippy
mise run //package/search-shadow/cli:test
# Installs the tools this host lacks inside a podman container (no network at run time, 2 GiB, 2 CPUs, the lowest
# disk weight: --blkio-weight=10 and --cgroup-conf=io.weight=10) and checks
# every shadow against its real tool
# (a lone --version must equal the real tool's output plus the one announcement line).
mise run //package/search-shadow/cli:test:container
```
