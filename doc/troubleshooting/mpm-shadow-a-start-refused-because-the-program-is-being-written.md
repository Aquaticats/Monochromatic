# mpm-shadow: a start refused with "Text file busy" is explained as a replacement, not retried

`mpm-shadow` 0.2.7-alpha.12,
running the real `mpm` from `PATH` (its shadow mode)
or replacing itself with a recorded tool (its launcher mode),
exits 126 with `the operating system could not start the program (Text file busy (os error 26))`
when another process holds the program's file open for writing.
This record measures which writers can put a program in that state,
and says why the refusal is explained rather than retried.
It is the durable half of issue 6585.

## Symptom

At a top-level `mpm refresh x`,
with another process holding the real `mpm` open for writing:

```text
mpm-shadow: the real `mpm` could not be started, so `mpm refresh x` did not run.
  command you ran: mpm refresh x
  program it tried to start: /home/user/.local/bin/mpm (the first `mpm` on PATH that is not this shadow)
  stage: starting the real `mpm` to run that command
  cause: the operating system could not start the program (Text file busy (os error 26))
  what to do: run `/home/user/.local/bin/mpm --version` by hand to see the same error outside the shadow, fix what it reports, then run the command again
  exit status: mpm-shadow exits with 126, ...
  what completed: nothing of `mpm refresh x` ran, ...; post-run cleanup did not run either, because no mpm command finished
```

The paths are the reproduction's,
and the wording is alpha.12's,
before this change.

Two things were wrong with that text,
whichever writer caused the refusal.
The advice cannot be followed:
the holder has closed the file by the time a person types `mpm --version`,
so the step shows nothing,
and "fix what it reports" names no fix that exists.
Nothing said the file was being replaced,
which is the one thing that makes the same command work again.

The launcher mode printed the same refusal as one bare line
(`mpm-shadow: could not start <path>: Text file busy (os error 26)`),
with no cause and no remedy.

## Root cause

`execve` refuses a file that any process holds open for writing.
`man 2 execve` says so,
and the page on this machine is `/usr/share/man/man2/execve.2.gz`:

```text
ETXTBSY
       The specified executable was  open  for  writing  by  one  or  more
       processes.
```

So the refusal is not about the program being broken or missing:
one process has that exact inode open for writing.
`mpm-shadow` starts the real `mpm` through `Command::spawn`
(`src/shim.rs`,
`VettedSpawn::attempt`),
and `start_retry::is_refusal` counted only `ErrorKind::WouldBlock`,
the system's refusal to create another process (`EAGAIN`).
`ErrorKind::ExecutableFileBusy` is errno 26,
and it therefore reached the catch-all arm of `start_failure::cause_and_remedy`,
which is written for a cause nobody has classified.

## Which writers can produce it

A start is refused only while the file at the path a start resolves is open for writing.
A writer that builds the new program under another name and then renames it into place
never opens the published path for writing:
`rename` replaces the directory entry,
and the write descriptor of the old inode belongs to a name no `execve` resolves.

Measured 2026-11-27 on this machine,
each writer below ran under `strace -f -ttt`
while an execve prober started the published path in a loop,
so every arm has both the syscall that opened the file and the starts it met.
The prober is `measurements-of-a-start-refused-because-the-program-is-being-written/start_probe.rs`
beside this record.
It counts `ETXTBSY` refusals,
`ENOENT` starts and successes.

Its positive control holds the published file open for writing for 200 ms
(`sh -c 'exec 3>>"$1"; sleep 0.2'`),
and the probe reports:

```text
START_PROBE path=arms/published/mpm span_ms=500 pause_us=200 ran=201 refused=589 absent=0 first_refusal_us=59068 last_refusal_us=261634
```

with one write open in the trace:

```text
openat(AT_FDCWD, "arms/published/mpm", O_WRONLY|O_CREAT|O_APPEND, 0666) = 3
```

So the instrument does see the refusal when there is one,
and a zero below is a zero it could have shown.

### The launcher placement

`src/toolchain/launcher/making_the_launcher.rs`,
`place_binary`:
a hard link or an `fs::copy` to `.<name>.<pid>.bin`,
then an `fs::rename`.

```text
openat(AT_FDCWD, "arms/published/.program.4242.bin", O_WRONLY|O_CREAT|O_EXCL, 0755) = 4
close(4) = 0        # 1795786497.171771 to .181404: 9.6 ms
```

```text
START_PROBE path=arms/published/program span_ms=400 pause_us=200 ran=395 refused=0 absent=0
```

### The plain and the gunzipped download

`src/toolchain/install_archive.rs`:
`unpack_into` copies with `fs::copy`,
and `gunzip_to` uses `File::create` plus `io::copy`.
Both write the hidden staging directory only.
The copy shape,
twice:

```text
openat(AT_FDCWD, "arms/tools/other/.1.0.0.staging.4243/program", O_WRONLY|O_CREAT|O_EXCL, 0755) = 4
close(4) = 0        # .006602 to .009468: 2.9 ms
```

```text
START_PROBE path=arms/tools/other/1.0.0/program span_ms=500 pause_us=200 ran=470 refused=0 absent=11
```

The gunzip shape ran on the same 5.5 MB program,
packed into a 2.3 MB archive,
as `gunzip --stdout archive > staged/program`:
that opens the staged file `O_WRONLY|O_CREAT|O_TRUNC` and keeps it open while it decompresses.

```text
START_PROBE path=arms/tools/gunzipped/1.0.0/program span_ms=600 pause_us=200 ran=500 refused=0 absent=8
```

The open is at `1795787188.803482` and the writing process exits at `.830150`,
so 26.7 ms under the trace.
Three runs of the same decompression without a trace took 202 ms (cold),
18 ms and 17 ms.

### The extraction by `tar`

`src/toolchain/install_support.rs`,
`extract_archive`.

```text
openat(4, "program", O_WRONLY|O_CREAT|O_EXCL|O_NOCTTY|O_NONBLOCK|O_CLOEXEC, 0755) = 5
close(5) = 0        # .116390 to .136531: 20.1 ms
```

```text
START_PROBE path=arms/tools/tarred/1.0.0/program span_ms=600 pause_us=200 ran=621 refused=0 absent=6
```

### The `uv` install of `mpm` itself

This tool installs `mpm` with `uv tool install`
(`src/toolchain/install_managers.rs`,
`install_mpm_uvx`),
and an `mpm` self-upgrade goes through the same mechanism:
`managers/uv.py` of the installed 8.3.0 wheel returns `tool upgrade --all` from `upgrade_all_cli`
and `tool upgrade <specs>` from `upgrade_cli`.

The entry point is written under a temporary name in `bin/` and renamed.
The trace of a second `--force` install of the same wheel shows both halves:

```text
openat(AT_FDCWD, ".../tools/mpmprobe/lib/python3.14/site-packages/../../../bin/.tmpiLCgCG", O_RDWR|O_CREAT|O_EXCL|O_CLOEXEC, 0666) = 12
```

```text
rename(".../tools/mpmprobe/lib/python3.14/site-packages/../../../bin/.tmpMtsI37", ".../tools/mpmprobe/lib/python3.14/site-packages/../../../bin/mpmprobe") = 0
```
The environment it lives in is removed and rebuilt first,
so a concurrent start finds the program absent rather than busy:

```text
START_PROBE path=arms/uv/bin/mpmprobe span_ms=2500 pause_us=300 ran=225 refused=0 absent=34
```

```text
START_PROBE path=arms/uv/tools/mpmprobe/bin/mpmprobe span_ms=2000 pause_us=300 ran=185 refused=0 absent=13
```

### The absence a concurrent start does meet

The archive install publishes a version by moving the old version directory aside
and renaming the staging directory in
(`src/toolchain/archive_staging.rs`,
`publish_staged`),
with two renames rather than one exchange:

```text
renameat2(AT_FDCWD, "arms/tools/other/1.0.0", AT_FDCWD, "arms/tools/other/.1.0.0.replaced.4243", RENAME_NOREPLACE) = 0
renameat2(AT_FDCWD, "arms/tools/other/.1.0.0.staging.4243", AT_FDCWD, "arms/tools/other/1.0.0", RENAME_NOREPLACE) = 0
```

Between them the program does not exist,
which is what the `absent=` counts above are:
11 starts in 3.5 ms for the copy arm,
6 in 2.6 ms for the `tar` arm,
8 for the gunzip arm,
and 13 and 34 for the two `uv` arms while the environment was rebuilt.
`ErrorKind::NotFound` is a different branch of the message,
and it is not this refusal.

## What does not work

Retrying the start within a bound has no window to carry,
which is why issue 6585's first option was not taken.

Every writer above is a writer this tool starts or stands in front of,
and none of them ever has the published path open for writing.
The writers that do hold it open are outside this tool:
a person's `cp` over the file,
an editor,
an installer that writes in place.
Each holds it for as long as its own work takes,
which no bound measured here can cover.

A retry would therefore add a stall for a writer nobody measured,
and `AGENTS.md` rule `HLD` refuses a wait that names no recorded failure it prevents.
`START_RETRY_BUDGET` keeps its own cause,
a shortage of processes,
whose window is a process limit that clears in moments.

The measurement does not settle what an outside writer's window is,
because it did not measure one.
The launcher arm's `cp` of the 5.5 MB program over the published path took 9.6 ms,
and a `pip install --upgrade` into a live virtual environment rewrites its console script in place.
Neither is what this tool runs,
and neither was measured.

## What the code does instead

`start_failure::being_written_cause_and_remedy` says the program's file is being written by another
program,
that an upgrade or an install is replacing the program,
and that the same command works once the write has finished.
It names no step that cannot show the error again.
`src/shim.rs` and `src/toolchain/launcher/starting_it.rs` both print those words,
because `spawn` and `exec` meet the same refusal.

`start_retry::is_refusal` still counts only `WouldBlock`,
and a unit test pins that the busy-file kind is not retried.

The post-run cleanup stays skipped after a start that never started anything.
What it deletes is superseded cached copies of this repository's crates,
read from the install records and the registry
(`src/prune.rs`),
not from what the command did.
A command that never ran superseded nothing,
so the skipped cleanup leaves the cargo home exactly as it found it.
The shim test `a_start_refused_because_the_program_is_being_written_says_the_file_is_being_replaced`
compares the file set before and after,
and pins that.

## What a future writer must do

A program this tool publishes is written under a name no start resolves,
and published with `rename`:
a temporary file beside the target for one program,
a hidden staging directory beside a version directory for an archive.
A writer that opens the published path for writing makes every start of that program fail with
`ETXTBSY` for as long as it holds it,
whether the start is this tool's or any other program's.

## What this record rests on

- The kernel rule:
  `man 2 execve`,
  quoted above.
- The code claims:
  the repo-relative paths named beside them.
- The figures:
  the `strace` timestamps and the probe lines quoted above,
  from the run recorded in this round's `keep/` folder,
  on this machine,
  2026-11-27.
- The `mpm` upgrade mechanism:
  `managers/uv.py` of the installed 8.3.0 wheel,
  read at
  `~/.local/share/uv/tools/meta-package-manager/lib/python3.14/site-packages/meta_package_manager/managers/uv.py`.
- The `uv` arm:
  a hand-built wheel (`mpmprobe-0.0.1-py3-none-any.whl`,
  one `console_scripts` row in its `entry_points.txt`),
  installed into a throwaway `UV_TOOL_DIR`,
  `UV_TOOL_BIN_DIR`,
  `UV_CACHE_DIR` and `HOME`.
  The real `mpm`,
  its toolchain root and the real home are not to be touched,
  and no `meta-package-manager` wheel was on disk to install.

The reproduction is in the package's test suite:
`tests/shim/a_start_that_fails_and_the_nesting_bound.rs` holds the real program's file open for
writing from a child process of the test's own
(`tests/held_programs/through_a_child_process.rs`),
and starts the shadow in front of it in a fixture home.

This record is unapproved and non-final.
What was done is recorded here and in the commits.
What is open is the owner's review,
the in-place writers named under "What does not work" that no measurement covers yet,
and the `dylint` rule `rust/avoidable-closure`,
which this package's lint run could not load.
