# search-shadow

Shadows the tools that walk many files: `rg`, the grep family (`grep`, `egrep`, `fgrep`, `ugrep`, `ug`, `ag`, `ack`),
the finders (`find`, `fd`, `fdfind`, `bfs`, `tree`) and the counters (`du`, `ncdu`, `tokei`, `cloc`).
Started under one of those names, the binary lowers its own disk priority to best-effort level 7 (the same as
`ionice -c2 -n7`, set directly with the `ioprio_set` system call) and replaces itself with the real tool, which keeps
the priority. So it adds no process and keeps exit codes, signals and terminal behavior.
Started as `search-shadow`, it offers `link` and `--help`.

## Why

A broad search reads hundreds of thousands of files.
On this host's DRAM-less NVMe, concurrent cold readers raise everyone's read wait from about 1 ms to about 8 ms (median)
with 100 to 200 ms tails, which stalls the desktop.
The measurements are in `doc/planning/agent-search-io-throttle.md`.

## What it does

- Runs the real tool at best-effort level 7.
  The class is honored under the BFQ and mq-deadline schedulers, which `/etc/udev/rules.d/61-nvme-bfq.rules` sets on
  `nvme0n1`; under kyber it is accepted and ignored.
  If the kernel refuses the call, the tool runs at normal priority.
  This is the crate's only `unsafe` code, in `src/priority.rs`.
- Refuses the short flag `-r` for `rg`, anywhere, with exit code 2.
  In ripgrep `-r` is `--replace`, not grep's "recursive": `rg -rl PATTERN` replaces every match with the text `l` and
  lists no files. ripgrep already recurses, so drop `-r`; write `--replace TEXT` for real replacement (still allowed).
  There is no escape hatch.
- Refuses `rg` with `--hidden`, `--no-ignore*`, `-u` or `-.`, no search path and no piped input,
  when run from a git repository root, with exit code 2 and a message that names the command, the directory and
  two ways forward: name a narrower path, or set `SEARCH_SHADOW_ALLOW_WIDE=1` for one deliberate full walk.
- Refuses a recursive `grep`, `egrep` or `fgrep` (`-r`, `-R`, `--recursive`, `--dereference-recursive`,
  `-d recurse`) that names no path or names a wide one, and refuses `find` or `bfs` that start at a wide path without
  `-maxdepth` of 3 or less.
  A wide path is a git repository root, the home directory or the filesystem root. The message suggests a narrower
  directory, `rg PATTERN PATH` or `rg --files PATH`, and the same `SEARCH_SHADOW_ALLOW_WIDE=1` escape.
- Only slows down the other tools (`ugrep`, `ug`, `ag`, `ack`, `fd`, `fdfind`, `tree`, `du`, `ncdu`, `tokei`, `cloc`);
  `fd` is never refused.
  Scripts that relied on plain GNU or ripgrep behavior for a wide walk will stop; that is deliberate.
- Skips its own links when it looks for the real tool, so it never runs itself.
- Passes arguments through byte for byte, including ones that are not valid UTF-8.

In Claude Code shells, `grep` and `find` are shell functions that run Claude's embedded `ugrep` and `bfs`, and functions
win over PATH, so the links reach `rg` there but not those two.

## Use

```bash
# Link every shadowed tool whose real binary is on PATH, then put the directory first on PATH.
search-shadow link .cache/search-shadow-bin
```

`link` links a name only when a real tool of that name is found on PATH (not counting the shadow itself), removes a
link whose real tool has gone, and reports what it linked, removed and skipped.
Run it again after installing or removing a tool.

In this repository it is installed by `mpm-shadow` from the toolchain plan (entry `cargo:monochromatic-search-shadow` in
`toolchain.plan.jsonc`): `mise run toolchain:install` runs `cargo binstall` for the locked version, and
`.config/mise/conf.d/20-env.toml` puts `.cache/search-shadow-bin` first on PATH, ahead of `.cache/toolchain-bin`,
which holds the locked ripgrep.

## Exit codes

- `2`: the policy refused the walk (or a bad `search-shadow` command line).
- `126`: the real tool was found but could not be started.
- `127`: no real tool on PATH.
- Anything else: the real tool's own exit code.

## Checks

```bash
mise run //package/search-shadow/cli:lint
mise run //package/search-shadow/cli:lint:clippy
mise run //package/search-shadow/cli:test
# Installs the tools this host lacks inside a podman container (no network at run time, 2 GiB, 2 CPUs) and checks
# every shadow against its real tool.
mise run //package/search-shadow/cli:test:container
```
