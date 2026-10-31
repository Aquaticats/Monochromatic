# search-shadow

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
- Skips its own links when it looks for the real tool,
  so it never runs itself.
- Passes arguments through byte for byte,
  including ones that are not valid UTF-8.

In Claude Code shells,
`grep` and `find` are shell functions that run Claude's embedded `ugrep` and `bfs`.
Functions win over PATH,
so the links reach `rg` there but not those two.

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

`link` links a name only when a real tool of that name is found on PATH (not counting the shadow itself),
removes a link whose real tool has gone,
and reports what it linked,
removed and skipped.
Run it again after installing or removing a tool.

In this repository it is installed through `mpm` by the toolchain plan
(entry `cargo:monochromatic-search-shadow` in the plan data under `package/file-enforcer/core/src/toolchain/`):
`mise run toolchain:sync` runs `cargo binstall` for the locked version,
and the generated `mise.toml` puts `.cache/search-shadow-bin` first on PATH
(its source is `file-enforcer.config.ts`),
ahead of `.cache/toolchain-bin`,
which holds the locked ripgrep.

## Exit codes

- `2`:
  the policy refused the walk (or a bad `search-shadow` command line).
- `126`:
  the real tool was found but could not be started.
- `127`:
  no real tool on PATH.
- Anything else:
  the real tool's own exit code.

## Checks

```bash
mise run //package/search-shadow/cli:lint
mise run //package/search-shadow/cli:lint:clippy
mise run //package/search-shadow/cli:test
# Installs the tools this host lacks inside a podman container (no network at run time, 2 GiB, 2 CPUs) and checks
# every shadow against its real tool
# (a lone --version must equal the real tool's output plus the one announcement line).
mise run //package/search-shadow/cli:test:container
```
