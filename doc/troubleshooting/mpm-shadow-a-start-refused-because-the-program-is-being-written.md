# mpm-shadow: a start refused with "Text file busy" is explained, and not retried

`mpm-shadow` 0.2.7-alpha.12,
running the real `mpm` from `PATH` (its shadow mode)
or replacing itself with a recorded tool (its launcher mode),
exits 126 with `the operating system could not start the program (Text file busy (os error 26))`
when another process holds the program's file open for writing.
This record measures which writers can put a program in that state,
and says what the evidence supports about repeating the refused start.
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
Alpha.13 replaced the cause and the advice;
the words themselves are the other half of issue 6585
and are pinned by the crate's tests,
not by this record.

The launcher mode printed the same refusal as one bare line
(`mpm-shadow: could not start <path>: Text file busy (os error 26)`),
with no cause and no remedy.

## Root cause

`execve` refuses a file that any process holds open for writing.
`man 2 execve` says so,
and the page on this machine is `/usr/share/man/man2/execve.2.gz`:

```text
ETXTBSY
       The specified executable was  open  for  writing  by  one or more
       processes.
```

So the refusal is not about the program being broken or missing:
one process has that exact inode open for writing.
`mpm-shadow` starts the real `mpm` through `Command::spawn`
(`src/shim.rs`,
`VettedSpawn::attempt`),
and `start_retry::is_refusal` counts only `ErrorKind::WouldBlock`,
the system's refusal to create another process (`EAGAIN`).
`ErrorKind::ExecutableFileBusy` is errno 26.
In `0.2.7-alpha.12` it reached the catch-all arm of `start_failure::cause_and_remedy`,
which is written for a cause nobody has classified;
since `0.2.7-alpha.13` it has an arm of its own,
`being_written_cause_and_remedy`,
and the section "What the code does instead" says what that arm's words claim.

There are two ways the file comes to be open for writing:

- a writer holds it for as long as its own work takes;
- a `fork` copies a write descriptor out of the process that opened it,
  and the child holds that copy from its `fork` to its own `execve`,
  which the section "The window a single-threaded probe cannot see" measures.

## What this tool starts, and how each writer publishes

Every program this tool starts that writes a file is named here,
with how it puts the file it writes into place.
The read-only runs (`uv tool list`,
`rustup which`,
`rustc --version`,
`bin list`,
`gh auth token`)
write no program that a start resolves,
and are not listed again.
Each entry says whether that was traced in a throwaway root on this machine
(`strace --follow-forks`,
an absolute timestamp on every line,
with a prober starting the published path throughout)
or read from the producer's source and not run.
The date is 2026-11-27.

In the quotes below the fixture's own root is written `<fixture>`,
its relative parts are kept,
and the tracer's path decoding is left out where it only repeats the open's argument.

### `uv tool upgrade --all`, and `uv tool install --force`

Routes:
`mpm-uvx` (`src/toolchain/install_managers.rs`,
`install_mpm_uvx`),
which is how `mpm` itself is installed from the lock's wheel,
and the mechanism `mpm upgrade` uses for a uv-installed tool:
the installed 8.3.0 `meta-package-manager` wheel's `managers/uv.py` returns
`tool upgrade --all` from `upgrade_all_cli` and `tool upgrade <specs>` from `upgrade_one_cli`.
Traced,
with uv 0.13.8,
a hand-built wheel `mpmprobe` installed at `0.0.1` and upgraded to `0.0.2`
from a local wheel directory,
offline,
in a throwaway `HOME`,
`UV_TOOL_DIR`,
`UV_TOOL_BIN_DIR` and `UV_CACHE_DIR`.

The environment's console script is written under a temporary name and renamed,
and the advertised entry point is a symbolic link that is removed and made again:

```text
unlink("<fixture>/uv/tools/mpmprobe/lib/python3.14/site-packages/../../../bin/mpmprobe") = 0
openat(AT_FDCWD, "<fixture>/uv/tools/mpmprobe/lib/python3.14/site-packages/../../../bin/.tmpTetV0W", O_RDWR|O_CREAT|O_EXCL|O_CLOEXEC, 0666) = 12
rename("<fixture>/uv/tools/mpmprobe/lib/python3.14/site-packages/../../../bin/.tmpTetV0W", "<fixture>/uv/tools/mpmprobe/lib/python3.14/site-packages/../../../bin/mpmprobe") = 0
unlink("<fixture>/uv/bin/mpmprobe") = 0
symlink("<fixture>/uv/tools/mpmprobe/bin/mpmprobe", "<fixture>/uv/bin/mpmprobe") = 0
```

- The console script is open from `09:57:39.310429` to its rename at `09:57:39.310620`:
  191 us.
- The old console script is unlinked at `09:57:39.265949` and the new one renamed in at
  `09:57:39.310620`:
  the file the `bin` link resolves to is **absent for 44.7 ms**.
- The `bin` link itself is removed at `09:57:39.313733` and made again at `09:57:39.315059`:
  1.3 ms.

No process opened the published path for writing at any point.
The two probers running throughout saw the absence and no refusal:

```text
START_PROBE path=<fixture>/uv/bin/mpmprobe span_ms=4000 pause_us=300 ran=287 refused=0 absent=67 other=[] first_refusal_us=- last_refusal_us=-
START_PROBE path=<fixture>/uv/tools/mpmprobe/bin/mpmprobe span_ms=4000 pause_us=300 ran=288 refused=0 absent=62 other=[] first_refusal_us=- last_refusal_us=-
```

A second `uv tool install --force` of the same wheel,
probed the same way,
also counted no refusal and 15 and 27 absent starts.

The shape of the real `mpm` on this machine is the same one:
`ls -l ~/.local/bin/mpm` prints
`mpm -> /home/user/.local/share/uv/tools/meta-package-manager/bin/mpm`,
and that file is a script whose first line names the tool environment's `python`.
So a start of `/home/user/.local/bin/mpm` resolves to
`~/.local/share/uv/tools/meta-package-manager/bin/mpm`,
which is the file uv stages and renames.

### `go install`

Route:
`go-install` (`src/toolchain/install_managers.rs`,
`install_go_install`),
which runs `go install <module>@v<version>` with `GOBIN` set to
`<toolchain root>/<id>/<version>/bin` (`src/toolchain/layout/install_directories.rs`,
`go_bin_dir_of`).
Traced,
with Go 1.27.4,
on a local module (`go install .`,
no network),
`GOBIN` on this machine's ZFS home and Go's build directory in a tmpfs,
which is the cross-device case.

Go renames its build output into `GOBIN` and,
across file systems,
falls back to copying it into place:

```text
renameat(AT_FDCWD, "<tmpfs>/go-build/work/b001/exe/a.out", AT_FDCWD, "<fixture>/go/gobin/hello") = -1 EXDEV (Invalid cross-device link)
openat(AT_FDCWD, "<fixture>/go/gobin/hello", O_WRONLY|O_CREAT|O_TRUNC|O_CLOEXEC, 0777) = 8
close(8<<fixture>/go/gobin/hello>) = 0
```

`09:59:25.436332` to `09:59:25.439863`:
the published path is open for writing
for **3.531 ms** while a 2.4 MB program is copied into it.
The prober starting that path saw the refusal:

```text
START_PROBE path=<fixture>/go/gobin/hello span_ms=4000 pause_us=300 ran=1528 refused=7 absent=994 other=[] first_refusal_us=570481 last_refusal_us=573269
```

Seven refusals between 570.481 ms and 573.269 ms of the probe's span:
2.788 ms of refusals,
which is inside the 3.531 ms the writer held the file.

The fallback is Go's,
not this route's:
`moveOrCopyFile` tries `os.Rename` and then `CopyFile`
(`/usr/lib/go/src/cmd/go/internal/work/shell.go`),
and `CopyFile` opens the destination `O_WRONLY|O_CREATE|O_TRUNC`.
Two more runs of the same arm show what the fallback depends on:
with Go's work directory on the same file system it renamed instead
(`renameat(...) = 0`),
and with the target already up to date it wrote nothing at all.

### `pnpm add --global`

Route:
`mpm-pnpm` (`src/toolchain/install_managers.rs`,
`install_mpm_pnpm`).
Traced,
with pnpm 12.21.0,
a hand-built package installed globally from a local tarball,
offline,
in a throwaway `PNPM_HOME` and store.

The published shim is written in place:

```text
unlink("<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm") = 0
openat(AT_FDCWD, "<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm", O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC, 0666) = 10
write(10<<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm>, "#!/bin/sh\n# Resolve $0 through s"..., 5176) = 5176
close(10<<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm>) = 0
```

`10:01:23.424865` to `10:01:23.424959`:
**0.094 ms** open for writing,
with no temporary name and no rename.
The same run then removed the shim again,
which is why this arm has no refusal count to quote:

```text
openat(AT_FDCWD, "<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm", O_RDONLY|O_CLOEXEC) = 10
unlink("<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm") = 0
unlink("<fixture>/pnpm/global-bin/bin/mpmprobe-pnpm") = -1 ENOENT (No such file or directory)
```

A first install of `0.0.1` left the shim in place;
the two later installs over a different version wrote it and then removed it,
so nothing was published at the end of either,
and the prober's counts over those runs are absences of two different causes
and are not quoted here.

### `cargo install`

Route:
`cargo-git` (`src/toolchain/install_cargo.rs`,
`install_cargo_git`),
`cargo install --git <url> --rev <revision> --root <install dir>`.
The same `cargo install` writes the program for a registry route the plan does not use here.
Traced,
with the toolchain's cargo
(1.102.0-nightly),
on a local crate (`cargo install --path .`,
no network,
`CARGO_NET_OFFLINE=true`).

It stages the program in a temporary directory under the target's `bin`,
writes it there,
and renames it into place:

```text
mkdir("<fixture>/cargo-install/root/bin/cargo-install0HL2Sh", 0777) = 0
openat(AT_FDCWD, "<fixture>/cargo-install/root/bin/cargo-install0HL2Sh/mpmprobe-cargo", O_WRONLY|O_CREAT|O_TRUNC|O_CLOEXEC, 0100755) = 7
close(7<<fixture>/cargo-install/root/bin/cargo-install0HL2Sh/mpmprobe-cargo>) = 0
rename("<fixture>/cargo-install/root/bin/cargo-install0HL2Sh/mpmprobe-cargo", "<fixture>/cargo-install/root/bin/mpmprobe-cargo") = 0
```

`10:02:26.287813` to `10:02:26.312798`:
24.985 ms open for writing,
under the temporary name only,
with the rename at `10:02:26.313301`.
The prober saw no refusal:

```text
START_PROBE path=<fixture>/cargo-install/root/bin/mpmprobe-cargo span_ms=9000 pause_us=200 ran=5575 refused=0 absent=1038 other=[] first_refusal_us=- last_refusal_us=-
```

The 1038 absent starts are the same absence the archive install has:
before the first install there is no program at that path.

### `cargo binstall`

Route:
`cargo` (`src/toolchain/install_cargo.rs`,
`install_cargo_binstall`),
`cargo binstall --root <version directory>`,
which is how a repository-published tool is installed.
Read,
not run:
no crate archive can be fetched offline,
and no release archive for a crate of this repository is on this machine.

`cargo-binstall`'s `atomic-file-install` crate (`atomic_install`)
calls `fs::rename` first.
Its cross-file-system fallback copies into a `NamedTempFile` made in the
destination's own directory and then `persist`s it,
which is a rename,
so the published path is never opened for writing there either.
This is the crate's `main` branch on 2026-11-27,
not the version pinned by the lock,
and binstall's own call site was not read.

### `bin install`, `android sdk install`, `rustup`

Read from this tool's code only,
not run,
and their producers' publish steps were not read:

- `bin install --force <tag url>` (`src/toolchain/install_bin.rs`,
  `install_bin_sourced`)
  installs into `<toolchain bin dir>` through the `bin` CLI,
  and that CLI's own write is unread.
- `android sdk install <packages>` (`src/toolchain/install_rust_android.rs`,
  `install_android_sdk`)
  runs the SDK manager behind the installed `android` CLI with `ANDROID_HOME`
  set to the toolchain's SDK root,
  and needs the network.
- `rustup toolchain install`,
  `rustup component add` and `rustup target add`
  (`src/toolchain/install_rust_android.rs`,
  `install_rust_toolchain` and `install_rust_components`)
  write under rustup's own home,
  which is not a path a start through this tool resolves.

### The writers inside this tool

Read from the code,
and traced on the same syscall shapes
(`cp`,
`mv`,
`tar` and `gunzip`,
GNU's,
in the same directories the code uses;
the code's own calls differ in the flags the standard library adds,
such as `O_CLOEXEC`,
and `fs::rename` in place of `mv`'s `renameat2`):

- The launcher placement (`src/toolchain/launcher/making_the_launcher.rs`,
  `place_binary`):
  a hard link,
  or `fs::copy`,
  to `.<name>.<pid>.bin`,
  then `fs::rename`.

  ```text
  openat(AT_FDCWD<<fixture>/arms/published>, ".program.4242.bin", O_WRONLY|O_CREAT|O_EXCL, 0755) = 4<<fixture>/arms/published/.program.4242.bin>
  close(4<<fixture>/arms/published/.program.4242.bin>) = 0
  renameat2(AT_FDCWD<<fixture>/arms/published>, ".program.4242.bin", AT_FDCWD<<fixture>/arms/published>, "program", 0) = 0
  ```

  `09:59:53.990774` to `09:59:53.993776`:
  3.002 ms,
  on the temporary name.

  ```text
  START_PROBE path=<fixture>/arms/published/program span_ms=1500 pause_us=200 ran=1037 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
  ```

- The plain copy (`src/toolchain/install_archive.rs`,
  `unpack_into`:
  `fs::copy`)
  and the gunzip (`gunzip_to`:
  `File::create` plus `io::copy`),
  both into the hidden staging directory.

  ```text
  openat(AT_FDCWD<<fixture>/arms/tools/other/.1.0.0.staging.4243>, "program", O_WRONLY|O_CREAT|O_EXCL, 0755) = 4<<fixture>/arms/tools/other/.1.0.0.staging.4243/program>
  close(4<<fixture>/arms/tools/other/.1.0.0.staging.4243/program>) = 0
  ```

  `10:00:13.533651` to `10:00:13.534171`:
  0.520 ms,
  because GNU `cp` copies this file with `copy_file_range` after one 65536-byte write,
  so the interval is a property of this file (5.5 MB of it is zeros this machine's ZFS compresses)
  and not a figure to carry elsewhere.

  ```text
  START_PROBE path=<fixture>/arms/tools/other/1.0.0/program span_ms=1200 pause_us=200 ran=728 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
  ```

  The gunzip shape,
  `gunzip --stdout <archive> > staged/program`:

  ```text
  openat(AT_FDCWD<<fixture>/arms/tools/gunzipped/.1.0.0.staging.4245>, "program", O_WRONLY|O_CREAT|O_TRUNC, 0666) = 3<<fixture>/arms/tools/gunzipped/.1.0.0.staging.4245/program>
  ```

  from `10:00:35.526280` to the writer's exit at `10:00:35.563339`:
  37.059 ms,
  on the staged name.

  ```text
  START_PROBE path=<fixture>/arms/tools/gunzipped/1.0.0/program span_ms=1200 pause_us=200 ran=541 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
  ```

- The extraction by `tar` (`src/toolchain/install_support.rs`,
  `extract_archive`,
  `--directory` the staging directory).

  ```text
  openat(AT_FDCWD<<fixture>/arms/tools/tarred/.1.0.0.staging.4244>, "program", O_WRONLY|O_CREAT|O_EXCL|O_NOCTTY|O_NONBLOCK|O_CLOEXEC, 0755) = 4<<fixture>/arms/tools/tarred/.1.0.0.staging.4244/program>
  close(4<<fixture>/arms/tools/tarred/.1.0.0.staging.4244/program>) = 0
  ```

  `10:00:29.007274` to `10:00:29.040386`:
  33.112 ms,
  on the staged name.

  ```text
  START_PROBE path=<fixture>/arms/tools/tarred/1.0.0/program span_ms=1200 pause_us=200 ran=602 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
  ```

- The download (`src/toolchain/install_support.rs`,
  `download_verified`)
  writes `<name>.download` and renames it,
  and the archive publish (`src/toolchain/archive_staging.rs`,
  `publish_staged`)
  gives the version's name the finished copy in one call:
  an exchange of the two names where the file system can make one
  (`replace_the_copy_that_is_there`,
  `archive_staging/exchanging_the_two_names.rs`),
  and two renames where it cannot (`two_renames`).
  A name nothing holds yet gets the finished copy in one `rename`
  (`move_the_finished_install`),
  and a finished copy already there is kept.
  Read from the code,
  and traced on 2026-11-27 with both released binaries
  (the section "The absence a concurrent start meets" and the transcripts beside this record).

## Which of those paths a start through this tool resolves

The shadow starts one program:
the real `mpm` it found on `PATH`
(`src/shim.rs`,
`start_and_wait`).
The launcher starts one program:
the path its record names (`src/toolchain/launcher/starting_it.rs`,
the record's `real` field),
which for a tool this repository publishes is that tool's installed program.

For those two paths the writers are:

- the real `mpm`:
  `uv tool install --force` from the `mpm` wheel,
  traced above,
  which stages the console script and renames it,
  and swaps the `bin` link;
- a launcher's program:
  `cargo binstall` (read),
  `cargo install` (traced) and this tool's archive install (traced,
  its publish included).

`go install` and `pnpm add --global` write a published path in place,
and neither writes a path a start through this tool resolves:
a Go-route tool is linked straight into the exposure directory
(`src/toolchain/exposure.rs`,
`exposed_target`:
only a repository-published tool gets a launcher),
and a pnpm global shim is not a path this tool starts either.

So the sentence this record used to carry,
that every writer this tool starts or stands in front of renames,
is false as it stood:
`go install` and `pnpm add --global` are both writers this tool starts and both open a
published path for writing.
What is true is narrower,
and it is what the decision rests on:
the writers of the two paths a start through this tool resolves
stage the file and give it its name without opening that name for writing:
a rename,
an exchange where the file system can make one,
and two renames where it cannot.

## The window a single-threaded probe cannot see

The prober
(`measurements-of-a-start-refused-because-the-program-is-being-written/start_probe.rs`)
holds no descriptor open for writing on anything it starts,
so no child of its own can inherit one,
and it sees only a holder that lives in a process for as long as that process writes.
A `fork` copies the descriptor table,
so a child another thread forks while a descriptor is open for writing
holds that copy from the fork to its own `execve`,
and the published file is refused to any start in that interval.
This repository measured that interval before this round:

- `doc/troubleshooting/shadow-tests-that-write-a-program-and-start-it-are-refused-with-etxtbsy.md`
  (this crate,
  2026-11-27):
  the hold is "from that `fork` to its own `execve` (about 1 to 3 ms)",
  a threaded probe was refused in 4 of 20 and 1 of 20 attempts,
  and before the fix the test binaries that write a program they start failed on it.
- `doc/troubleshooting/git-policy-cli-a-fixture-git-is-refused-with-etxtbsy.md`
  (2026-11-26):
  the holder "released its descriptor 554 us after the refused start",
  and a scan of every process's descriptors at the moment of the error found none.
- `doc/handover/cli-git-native-foundation.md`
  "Intermittent test failure":
  74 of 400 repeated test runs failed before the fix,
  68 of them with "Text file busy",
  at 16 test threads and from a child another test thread forked.
- `doc/planning/cargo-shadow-guarantees.md`
  "Limits that stay" lists an `ETXTBSY` refusal while a writer holds the file
  as not covered.

A threaded arm was added to the prober:

```sh
# one write of the program, held open, then closed and started; a second thread forks throughout
start_probe forked-holder <program> <attempts> [hold-microseconds] [forker-pause-microseconds]
# the same loop with no forking thread, its control
start_probe writer-only <program> <attempts> [hold-microseconds]
```

The writer is the prober's own main thread,
so no descriptor of it outlives the write,
and the forking thread starts `/bin/true` in a loop,
so each of its children holds a copy of that descriptor from its own fork to its own start.
Two runs of the arm,
and two of its control:

```text
START_PROBE mode=forked-holder path=<fixture>/arms/published/control attempts=20 hold_us=2000 forks=37 elapsed_us=92034 ran=15 refused=5 absent=0 other=[] first_refusal_us=2433 last_refusal_us=69857
START_PROBE mode=forked-holder path=<fixture>/arms/published/control attempts=20 hold_us=2000 forks=38 elapsed_us=95862 ran=10 refused=10 absent=0 other=[] first_refusal_us=21687 last_refusal_us=90170
START_PROBE mode=writer-only path=<fixture>/arms/published/control attempts=20 hold_us=2000 forks=0 elapsed_us=104159 ran=20 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
START_PROBE mode=writer-only path=<fixture>/arms/published/control attempts=20 hold_us=2000 forks=0 elapsed_us=100771 ran=20 refused=0 absent=0 other=[] first_refusal_us=- last_refusal_us=-
```

Five and ten of twenty starts were refused with the forking thread,
and none of twenty in either control run,
on the same file,
in the same loop,
with the same 2 ms hold.
The counts move with how many forks the host lets the forking thread make in the same span
(37 and 38 here,
51 and 67 in two earlier runs of the same arm,
which were refused 8 and 12 of 20),
so the figure to keep is the shape and not the count:
the cause needs a fork while the descriptor is open,
and it refuses a start when one lands.

What this does and does not show:

- It shows the cause the sibling records describe is reachable in this crate's own shape,
  and that a copy of a descriptor into a child of another thread is enough:
  nothing wrote the published path at the moment of the refusal.
- It does not show which production writer produces it.
  None of the traced writers above forks while it holds a file open for writing,
  and `uv`,
  `go`,
  `cargo` and `pnpm` are separate processes whose children are their own.
  The holder in the two sibling records is a test process that starts programs,
  and the holder in issue 6585's reproduction was made by hand.
- It does not size the window for a production program.
  The 1 to 3 ms is the sibling record's figure for a script's start;
  a bigger program spends longer between `fork` and `execve`,
  and that was not measured.

## The absence a concurrent start meets

First the probe,
on a publish slowed on purpose,
so that its silence on an unslowed publish means something.
`strace --inject=rename,renameat2:delay_enter=300ms`
holds the process 300 ms before it enters each rename,
which for the two-rename publish opens the version's name for that long:

- the poller saw the name absent for 300.175 ms;
- the trace's own arithmetic gave 300.086 ms,
  from the two renames' entry times 300.126 ms apart less the first rename's own 40 us;
- so both instruments see an absence of the length that is there,
  and agree with it and with each other within 90 us.

With the exchange slowed the same way,
the poller counted no absence in 54,266 polls:
the publish was later,
and the name was never absent.

On an unslowed publish the archive install puts the finished copy at the version's name in one call,
and since `0.2.7-alpha.14` that call is `renameat2` with `RENAME_EXCHANGE`
(`publish_staged`,
`replace_the_copy_that_is_there`,
`archive_staging/exchanging_the_two_names.rs`),
and `man 2 renameat2` says the two names change places atomically,
so a reader of the version's name finds either the copy that was there or the finished one.
Measured on 2026-11-27,
with the released `0.2.7-alpha.14` binary
(sha256 `b60ba895f3392bad1cdb0396695a127a6b348a47f166dcffc22f82ad66cbb250`,
the bytes of `/home/user/Monochromatic/.cache/toolchain-bin/mpm-shadow` too),
in a fixture toolchain root of its own,
over an unfinished copy at the version's name (a program with no completion marker),
three publishes on this host's home file system (ZFS) and three on tmpfs,
each under `strace --follow-forks` with absolute microsecond entry times and each call's own duration,
with the same poller `stat`ing the version's name throughout
(`measurements-of-a-start-refused-because-the-program-is-being-written/`,
the logs and their raw traces):

- the publish is that one call,
  and its own work took 32 to 41 us on the home file system
  and 7 to 11 us on tmpfs;
- the poller counted no absence in any of the six publishes
  (`absent=0` over 54,692 to 55,440 polls each,
  one poll every 72 to 73 us),
  and saw the copy that was there,
  then the finished copy;
- that zero rests on the call being one atomic exchange and not on the count,
  because the poller cannot resolve an absence shorter than its own poll.

Where the file system answers the exchange with `EINVAL`,
`ENOTSUP` (`EOPNOTSUPP`),
`EPERM`,
`EACCES` or `ENOSYS` (`cannot_exchange`),
the code publishes in two renames (`two_renames`):
the copy at the version's name moves to a hidden `.replaced` sibling,
and the finished one is renamed in after it.
That pair leaves the instant the exchange removes,
and it is the only window this module has.
Measured on the same fixture and date,
with the same released binary and strace answering the exchange `EINVAL`,
which is what a file system that cannot exchange gives:

- the version's name was absent for 24 to 38 us on the home file system
  and 26 to 41 us on tmpfs,
  from the two renames' entry times 34 to 67 us apart less the first rename's own 8 to 29 us;
- that is the released binary's own fallback,
  reached with the `EINVAL` a file system gives,
  not a stand-in for it;
- `0.2.7-alpha.13`
  (sha256 `087df55ec17f5a0c2b2d74c3e9d07cba0bf2730acc9882a736beb4a53b5b2181`,
  fetched with `gh release download`),
  whose publish over a copy at the version's name is always that pair,
  measured 25 to 35 us on the home file system
  and 26 to 48 us on tmpfs,
  in three publishes on each;
- the interval is therefore 24 to 48 us across the twelve publishes,
  two orders of magnitude below the 5.025 ms this record used to state;
  that figure came from two separate `mv` commands standing in for the publish,
  and each `mv` paid its own process start,
  which the two renames of one install do not;
- the poller,
  one poll every 72 to 100 us in these runs,
  landed inside the instant in 5 of the 12 unslowed publishes and reported 77 to 87 us,
  which is its own poll's cost inside the gap and an upper bound,
  not the figure;
  the entry times and the first rename's own duration are the figure.

`ErrorKind::NotFound` is a different branch of the message,
and it is not this refusal.
The uv arms meet the same absence for longer:
62 and 67 absent starts while uv rebuilt the tool environment in place.
A first install still leaves the name absent until it publishes,
because there is nothing there to exchange with,
and `move_the_finished_install` renames the finished copy into the absent name.
`package/mpm-shadow/cli/src/toolchain/archive_staging.rs` and its parts say what a kill in the fallback's instant leaves,
and how a later install finishes it.

## What does not work

Retrying the start has nothing to carry from the traced writers,
but it is no longer true that nothing measured is a window it could carry:
the fork-held descriptor is a measured window in this crate's own tests,
1 to 3 ms at the sibling record's figures,
and a single-threaded prober cannot see it.

For a writer this tool starts or stands in front of,
no traced or read one opens the two paths a start resolves:
they stage the new copy and give it the name without holding that name open for writing.
The writers that do hold a published path open are:
`go install` across file systems (3.531 ms,
traced,
and a refusal it caused),
`pnpm add --global` (0.094 ms,
traced),
and,
outside this tool,
a person's `cp` over the file,
an editor,
an installer that writes in place,
each for as long as its own work takes.

## What the code does instead

`start_failure::being_written_cause_and_remedy` says that a file this start needs is open for writing by
some other process,
that the file is the program itself or the interpreter named on the program's first line,
and that mpm-shadow cannot tell which process holds it or why.
It names no usual cause,
so it does not say that an upgrade or an install is one,
and it does not claim a replacement is in progress:
the system's refusal says only that some process holds the file open for writing.
Its remedy is to run the same command again once that process has finished,
because nothing of this command ran,
and to install or upgrade the program again if a write was cut short and left a broken file.
It names no step that cannot show the error again:
a hand run shows nothing once the write is over.
`src/shim.rs` and `src/toolchain/launcher/starting_it.rs` both print those words,
because `spawn` and `exec` meet the same refusal.

`start_retry::is_refusal` counts only `WouldBlock`,
and a unit test pins that the busy-file kind is not retried.

The post-run cleanup stays skipped after a start that never started anything.
What it deletes is superseded cached copies of this repository's crates,
read from the install records and the registry
(`src/prune.rs`),
not from what the command did.
A command that never ran superseded nothing,
so the skipped cleanup leaves the cargo home exactly as it found it.
A superseded copy an earlier run left,
including one a killed run left,
stays until the next run that starts,
which is the behaviour the README's shadow-mode section already states.
The shim test
`a_start_refused_because_the_program_is_being_written_says_what_the_refusal_proves_and_what_to_do`
(`package/mpm-shadow/cli/tests/shim/a_start_that_fails_and_the_nesting_bound.rs`)
compares the cargo home's file set before and after,
and pins that.

## What a future writer must do

A program this tool publishes is written under a name no start resolves,
and given its own name without opening that name for writing:
a `rename` of a temporary file beside the target for one program,
and for an archive a hidden staging directory beside the version directory,
which one `renameat2` with `RENAME_EXCHANGE` puts in place where the file system can,
and two renames put in place where it cannot.
A writer that opens the published path for writing makes every start of that program fail with
`ETXTBSY` for as long as it holds it,
whether the start is this tool's or any other program's,
and two of the writers this tool starts do exactly that today
(`go install` and `pnpm add --global`),
for paths no start through this tool resolves.

## What this record rests on

Each figure above is derived from a trace line or a prober line,
quoted beside it or kept in this record's measurements folder,
and every quoted line comes from a run made in a throwaway root on this machine,
2026-11-27.
The prober source is beside this record;
the traces and the prober's output lines are quoted here,
and nothing here cites a scratch folder that a later reader cannot open.

The figures for the archive publish's absence are not quoted line by line;
their runs are beside this record,
in `measurements-of-a-start-refused-because-the-program-is-being-written/`:
the harness that builds the fixture and runs each publish
(`measure_a_publish_and_read_its_absence.py`),
the poller it starts (`poll_the_published_name.c`),
one log per arm with every figure the harness computed
(`exchange-on-home-zfs.log.txt`,
`exchange-on-tmpfs.log.txt`,
`fallback-on-home-zfs.log.txt`,
`fallback-on-tmpfs.log.txt`,
`two-renames-on-home-zfs.log.txt`,
`two-renames-on-tmpfs.log.txt`,
`control-two-renames-slowed-300ms-on-home-zfs.log.txt`,
`control-exchange-slowed-300ms-on-home-zfs.log.txt`),
and each publish's own `strace` output beside them
(`exchange-on-home-zfs-publish-1.strace.txt`,
`fallback-on-home-zfs-exchange-answered-EINVAL-publish-1.strace.txt`,
`two-renames-on-home-zfs-publish-1.strace.txt`,
`control-two-renames-slowed-300ms-on-home-zfs-publish-1.strace.txt`,
and the rest).
The fixture's own paths are written `<fixture>`,
and the round's scratch folder `<scratch>`,
in the copies kept here;
nothing else in them is changed.
The figure for each publish is computed from one trace in this folder:
the two renames' entry timestamps,
and the first rename's own duration from its `-T` field,
which is the elapsed time from the call's entry to its exit.
That strace's stamp is the call's entry,
not its exit,
was checked on this machine with a read another process holds open for 200 ms
(`strace-entry-times-under-a-held-read.strace.txt`):
the read's stamp sits 50 us after the program's own clock before the read,
and 197.0 ms before its clock after it.
Each publish had a toolchain root,
a home and a cache of its own,
and none of them was the installed one:
the installed `mpm-shadow` and the real toolchain root were read,
never written.
The tmpfs fixtures were made under `/dev/shm`,
the tmpfs this machine has,
and removed after the round.

Read,
not run:
`cargo-binstall`'s `atomic-file-install` (`main` branch,
2026-11-27),
the messages and the route code named beside each claim,
and the installed 8.3.0 `meta-package-manager` wheel's `managers/uv.py`
(`~/.local/share/uv/tools/meta-package-manager/lib/python3.14/site-packages/meta_package_manager/managers/uv.py`).

The kernel rule:
`man 2 execve`,
quoted above,
and `man 2 renameat2` for the exchange that leaves the version's name in place.

What was not run or read:

- `cargo binstall` itself:
  no crate archive can be fetched offline,
  and no release
  archive for a crate of this repository is on this machine.
- `bin install`:
  the `bin` CLI's own publish step.
- `android sdk install` and the SDK manager behind it:
  it needs the network.
- `rustup`'s own writes.
- `unzip`'s extraction;
  only `tar`'s was traced.
- The writers named under "What does not work" as outside this tool:
  no in-place writer's window was measured here.
- A file system that refuses the exchange:
  this host's ZFS home and its tmpfs both answer it,
  so the two-rename fallback was reached with the `EINVAL` strace answered
  rather than with a file system of this machine.
- A start of the real `mpm` while `uv tool upgrade` replaces it:
  the mechanism was traced,
  the end-to-end start was not.
- Windows and macOS.

Corrections this version carries:

- It said every writer this tool starts or stands in front of renames its file into place.
  Traced on 2026-11-27,
  `go install` across file systems and `pnpm add --global` open a published path for writing,
  though neither writes a path a start through this tool resolves.
- It said there was no measured window a retry could carry.
  The fork-held descriptor is one,
  measured in two sibling records and now in this record's own threaded arm.
- Its figures now come from the lines quoted here.
  A range that omitted the gunzip arm,
  a copy figure taken from one excerpt and quoted as two,
  and a 9.6 ms that was the copy to the hidden name rather than a copy over the published path
  are gone.
  The millisecond figures for the copies are not carried as figures to reuse:
  the copy arm's interval depends on the file and the copy method
  (0.520 ms here with `copy_file_range` over a file this machine's ZFS compresses).
- The uv section quoted one command's trace and said the other used the same mechanism.
  Both are traced here,
  and the self-upgrade command `mpm` runs is the one traced.
- It said a start refused with `Text file busy` is explained as a replacement (its title),
  and that the message names an upgrade or an install as `the usual cause`.
  `being_written_cause_and_remedy` names no cause,
  and `src/start_failure_tests.rs` and `tests/launcher.rs` assert that the words `usual cause` never appear.
- It named a shim test,
  `a_start_refused_because_the_program_is_being_written_says_the_file_is_being_replaced`,
  which no code holds.
  The test is
  `a_start_refused_because_the_program_is_being_written_says_what_the_refusal_proves_and_what_to_do`
  in `package/mpm-shadow/cli/tests/shim/a_start_that_fails_and_the_nesting_bound.rs`,
  and it compares the cargo home's file set before and after,
  which is what the record says it pins.
- It said the archive publish moves the old version directory aside and renames the staging directory in,
  with two renames rather than one exchange,
  and that the program does not exist for 5.025 ms.
  Since `0.2.7-alpha.14` the publish is one exchange where the file system can make one,
  the two renames are its fallback,
  and that fallback's instant measured 24 to 48 us on 2026-11-27.
  The 5.025 ms was two `mv` processes standing in for the publish,
  not the install's two renames in one process.
- It said a program this tool publishes is published with `rename`.
  An archive over a copy already at the version's name is published with one exchange where the file system can make one.

## What the evidence says about repeating a refused start

Two options,
with what each costs.
Neither is chosen here:
the pick recorded with issue 6585 is before the owner again,
and this record only says what the measurements now support.

What this round changed for the choice:
nothing.
Both options are about a start the system refuses because a file is open for writing (`ETXTBSY`),
and the archive publish's instant was an absence,
which a start meets as `NotFound` and the message explains as a different arm.
The exchange removes that absence and leaves the refusal the options are about untouched,
so the window each option weighs is still the fork-held descriptor's 1 to 3 ms,
measured in this record's threaded arm and in the two sibling records.

### Explain only, as released

The refusal is reported at once with the cause and the advice alpha.13 prints,
and nothing is started twice.

Pros:

- No added latency for either kind of holder:
  an outside writer that holds the file for a second gives its message in the same second.
- The code path stays the one the tests pin,
  and `is_refusal` keeps one meaning for one cause.
- Nothing is claimed about a window that a production writer was not shown to have:
  no traced writer of a path a start resolves opens it for writing.

Cons:

- The measured fork-held window,
  1 to 3 ms in two sibling records and 5 to 10 of 20 attempts here,
  reaches a person as a hard failure with advice to run the command again,
  when a few milliseconds of waiting would have started the program.
  That is the transient a busy session meets,
  and it is the one the sibling records show is common in threaded test runs.
- The decision then rests on a claim about writers,
  while the failure can come from something that is not a writer at all.

### Repeat within a bound sized from the measured windows, then explain

The start is tried again,
pausing between tries,
for a budget sized from the measured hold (1 to 3 ms,
three orders of magnitude below `START_RETRY_BUDGET`'s 5 s),
and the alpha.13 message is printed if the budget runs out.

Pros:

- It carries the one measured transient,
  and the number it carries has two records and this round's arm behind it.
- A start that met a fork-held descriptor stops being a failure a person has to understand.
- The wait names a recorded failure it prevents,
  which is what `AGENTS.md` rule `HLD` asks of a wait.

Cons:

- It adds the budget to every holder that does not clear in time,
  which is every outside writer:
  a person's `cp` over the file,
  an editor,
  an in-place installer.
  The bound has to be small for that reason,
  and a small bound does not cover a writer that takes 20 ms or more,
  so the message arrives later and still arrives.
- It makes a refusal that is real look like a stall for the length of the budget.
- It needs the busy-file kind to be retried,
  which changes what `is_refusal` means,
  and the tests that pin the current rule have to be rewritten with it.
- No production writer was shown to produce the window it carries,
  so it would be sized from test-process and probe evidence alone.

Ranking:
repeat within a bound > explain only,
because the only measured window is 1 to 3 ms and refusing it is a failure a person cannot act on,
while the cost of the bound falls on a holder that ends in the same message either way.
The counter-argument,
which the owner may weigh the other way,
is that the window is measured in probes and test processes and never in a production writer,
so the bound would be sized from evidence that is not about the tool's own work.

This record is unapproved and non-final.
What was done is recorded here and in the commits.
What is open is the owner's review and the choice between the two options above.
