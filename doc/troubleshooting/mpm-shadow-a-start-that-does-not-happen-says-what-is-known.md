# mpm-shadow: a start that cannot happen says what is known about it

`mpm-shadow` prints one message for a real `mpm` that never ran
(`src/start_failure.rs`).
Which words it prints is chosen by what happened,
and this record lists every refusal that can reach it,
what became of each,
and what a person can still do about the ones with no arm of their own.
It is the durable half of issue 6606.

## Symptom

A start refused because the program is this tool's own shadow printed
the words for a fault nothing has classified,
including a step that shows nothing:

```text
mpm-shadow: the real `mpm` could not be started, so `mpm installed` did not run.
  command you ran: mpm installed
  program it tried to start: <path>/other/mpm-shadow (the first `mpm` on PATH that is not this shadow)
  stage: starting the real `mpm` to run that command
  cause: the operating system could not start the program (<path>/other/mpm-shadow is or leads to monochromatic-mpm-shadow 0.2.7-alpha.13, a shadow, so it was not started)
  what to do: run `<path>/other/mpm-shadow --version` by hand to see the same error outside the shadow, fix what it reports, then run the command again
  ...
```

Running that path by hand starts a shadow,
which answers as a shadow
and never shows the refusal.
The paths are the reproduction's;
the wording is alpha.13's before this change,
and is pinned by the crate's tests rather than by this record
(`src/start_failure_tests.rs`,
`tests/never_start_another_shadow/shadows_that_must_not_start_each_other.rs`).

## How the message is chosen

`StartFailure.refusal` says which of two things happened,
and `cause_and_remedy` picks the cause and the remedy from it
(`src/start_failure.rs`):

- `StartRefusal::System`:
  the system answered the open or the start with an error,
  and the arm follows the error's kind.
- `StartRefusal::Shadow`:
  the check the shim makes on the open program just before its start
  found this tool's own shadow,
  or a launcher record that leads to one
  (`VetRefusal::LeadsToShadow`,
  `src/shadow_identity/vetted_program.rs`).

Before this change the second arrived as `Error::other(sentence)`,
whose kind is `Other`,
so it reached the last arm and got that arm's advice.

## How a refused shadow is reached

`find_in_path` skips every candidate that carries the shadow identity,
or that is a launcher whose record leads to one
(`src/real_binary.rs`,
the `shadow_behind` call),
and `vet` makes the same judgement on the open file at the start
(`src/shadow_identity/vetted_program.rs`,
`identity_of_open_file_unless_launcher`).
Two checks that read the same things agree,
so the refusal is what a start meets when the entry the lookup chose changed between
the lookup and the start:
an install,
an upgrade,
or a launcher rewriting its record in another session
(`write_atomically`,
`src/toolchain/util.rs`).
That is why no `PATH` can be arranged to reach it deterministically,
and why the test drives the check the shim makes
(`start_and_wait`) on a copy of another shadow version
instead of starting a shadow through `PATH`.

## The launcher cannot meet it, and must not

`install` gives a repository-published tool a launcher:
a copy or hard link of the shadow binary with a `.launch` record beside it
(`src/toolchain/launcher/making_the_launcher.rs`,
`ensure_launcher`),
and for `mpm-shadow` the record names that tree's installed `mpm-shadow` binary,
which is a shadow
(`src/toolchain/exposure.rs`,
`exposed_target`;
`toolchain.lock.json`,
the `mpm-shadow` postinstall link named `mpm`).
A launcher starts whatever its record names,
and it makes no shadow check:
measured 2026-11-27,
a launcher named `mpm` whose record named a shadow binary,
with a real `mpm` script after it on `PATH`,
printed that script's own output and exited 0.
So the words for a refused shadow belong to the shadow's check alone,
and a launcher taught to refuse a shadow would refuse the link `install` places.
`tests/never_start_another_shadow/launchers_that_must_not_start_themselves.rs`
pins that arrangement.

## Every error that can reach the last arm

The callers are one:
`run_shim` prints the message for `ProgramRun::NotStarted`
(`src/shim.rs`),
whose refusal comes either from `vet`'s two opens
(`std::fs::File::open` and the `O_PATH` fallback)
or from `Command::spawn` of the vetted program
(`VettedSpawn::attempt`).
The kinds below were read from this toolchain's `std::io::ErrorKind`
with `Error::from_raw_os_error`,
2026-11-27,
rustc 1.102.0-nightly.

Arms that exist,
and the numbers behind them:

- `WouldBlock`:
  `EAGAIN`,
  a process or task limit.
- `ExecutableFileBusy`:
  `ETXTBSY`,
  another process holds the file open for writing.
- `PermissionDenied`:
  `EACCES` and `EPERM`,
  permission to run the program.
  The arm's remedy is the execute bit and the directories on the path,
  which fits `EACCES`;
  an `EPERM` can also come from a policy outside the file,
  and that is not known to be covered.
- `NotFound`:
  `ENOENT`,
  the program or the interpreter on its first line.
- `ENOEXEC`:
  number 8,
  which maps to an unnamed kind;
  the arm was added by this change
  (see the next section).

Everything else reaches the last arm.
These numbers were measured to reach it,
with what a hand run of the same program shows for each:

- `E2BIG`:
  `ArgumentListTooLong`,
  number 7;
  the shell repeats it for the same argument list.
- `EIO`:
  `InputOutputError`,
  number 5;
  the shell repeats it.
- `EFAULT`:
  an unnamed kind,
  number 14;
  a start whose argument pointers are bad,
  which a shell does not build,
  so a hand run cannot repeat it
  (from `man 2 execve`, not measured).
- `ENOTDIR` and `EISDIR`:
  numbers 20 and 21;
  the shell repeats them.
- `EINVAL`:
  `InvalidInput`,
  number 22;
  the shell repeats what the kernel says about the file.
- `EMFILE` and `ENFILE`:
  `TooManyOpenFiles`,
  numbers 24 and 23;
  not reliably,
  because the shortage belongs to the shadow's own process as much as to the machine.
- `ENOMEM`:
  `OutOfMemory`,
  number 12;
  not reliably,
  for the same reason.
- `ENAMETOOLONG` and `ELOOP`:
  `InvalidFilename` and `FilesystemLoop`,
  numbers 36 and 40;
  the shell repeats them.
- `ENXIO`,
  `EOVERFLOW` and `ELIBBAD`:
  unnamed kinds,
  numbers 6,
  75 and 80,
  each reaching the last arm;
  the shell repeats each as its own text.

So the last arm's advice was true for most of them and false or unreliable for four:
the shadow refusal,
which is a different event and never an operating-system error,
`ENOEXEC`,
which a shell hides,
and the two shortages (`EMFILE` and `ENOMEM`),
where the hand run can come out clean because the shortage has passed.

## What the code does now

- A refused shadow has its own arm:
  the path,
  the shadow the file is or leads to (the sentence `vet` built),
  why starting it would loop
  (a shadow looks for the real `mpm` again,
  and two shadows that each stood first for the other started each other without end,
  measured 2026-11-07 at about 105 starts a second),
  that the same path was not a shadow when the lookup read it a moment earlier,
  and the remedies:
  run the command again,
  `command -v -a mpm` to list every `mpm` on PATH in lookup order with the shadow first and the real `mpm` after it,
  and `mpm install` in a launcher's tree for a record that names a shadow.
  The line that names the program says what it turned out to be there,
  instead of "the first `mpm` on PATH that is not this shadow".
- `ENOEXEC` has its own arm:
  the file is empty,
  cut short,
  built for another system,
  or not a program,
  and the arm says that a shell does not show this refusal.
  Measured 2026-11-27 in a throwaway folder,
  two executable files with no shebang:

  ```text
  $ bash -c './empty-file --version'; echo $?
  0
  $ bash -c './text-file --version'; echo $?
  ./text-file: line 1: this: command not found
  3
  $ python3 -c "import subprocess; subprocess.run(['./text-file','--version'])"
  OSError 8 Exec format error
  ```

  The shell runs a file the kernel refuses as a script,
  or runs the empty file and exits 0;
  only a start without a shell reports the refusal.
- The last arm keeps the system's own words and says what a hand run can and cannot show:
  the same words when the program's own file is at fault,
  and something else or nothing when a limit of the machine,
  a limit of the shadow's own process,
  or the shape of the command line is the cause.

Nothing else changed:
`start_retry::is_refusal` is untouched and still counts `WouldBlock` alone,
so whether a refused start is repeated is unchanged and stays before the owner
(`doc/troubleshooting/mpm-shadow-a-start-refused-because-the-program-is-being-written.md`,
"What the evidence says about repeating a refused start").

## What this record rests on

- The errno to kind mapping:
  a program of this toolchain's own that printed
  `Error::from_raw_os_error(n).kind()` and its text for each number listed above.
- The `ENOEXEC` by-hand behaviour:
  the three commands quoted above,
  run in a throwaway folder on this machine on 2026-11-27.
- The launcher arrangement:
  `src/toolchain/exposure.rs` (`exposed_target`),
  `src/toolchain/launcher/making_the_launcher.rs` (`ensure_launcher`),
  the `mpm-shadow` postinstall link in `toolchain.lock.json`,
  and the run described above.
- The message wording:
  the crate's tests,
  which pin each sentence this record quotes
  (`src/start_failure_tests.rs`).

Not run:
a start that meets the changed entry through `PATH`,
because the lookup and the start check judge the same file the same way
and only a change between them can make the two disagree;
the test drives the start check directly instead.
Windows and macOS are out of scope.

This record is unapproved and non-final.
What was done is recorded here and in the commits.
What is open is the owner's review,
and the option of repeating a start refused as busy,
which this change leaves exactly as it was.
