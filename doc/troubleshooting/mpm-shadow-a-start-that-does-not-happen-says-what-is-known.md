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
  the program or the interpreter it names,
  which is the program after `#!` on a script's first line or a compiled program's dynamic linker.
- `ENOEXEC`:
  number 8,
  which maps to an unnamed kind;
  the arm was added by this change
  (see the next section).

Everything else reaches the last arm.
These numbers were measured to reach it,
with what a shell does for each:

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

So the advice alpha.14 shipped was wrong for four of them:
it sent every reader to a hand run of the program,
and that run shows nothing for the shadow refusal,
which is a different event and never an operating-system error,
nothing for `ENOEXEC`,
which a shell hides,
and something clean for the two shortages (`EMFILE` and `ENOMEM`),
because the shortage has passed by the time a shell starts the program.
Three of them cannot be cured by running the same command again at all:
`E2BIG`,
because the same argument list is refused the same way,
`ENAMETOOLONG`,
because the same path keeps its length,
and `ELOOP`,
because the link that loops keeps looping.

## What the code does now

- The two arms that name the other file a start needs name the interpreter the program names,
  and read it from the program's own bytes (`src/named_interpreter.rs`):
  the program after `#!` on a script's first line,
  or the `PT_INTERP` entry of a compiled program,
  which is its dynamic linker.
  A compiled program has no first line,
  so the sentence that said the interpreter was named there pointed at a file the program does not have;
  the realistic case is a system's own linker being replaced during an upgrade.
  The being-written arm names the interpreter and says the shadow cannot tell which of the two files is held.
  The not-there arm goes further when the program is still there:
  with its own file readable, and the interpreter that file names gone,
  it says which interpreter is missing and the remedy installs that file.
  Reproduced 2026-11-27 in a fixture:
  a compiled program linked against a copy of the system's own linker
  (`gcc -Wl,--dynamic-linker=<copy>`) with that copy held open for writing,
  and then with the copy taken away.
  The arm that says nothing about which file is missing is the one where the program itself is gone,
  because then no bytes are left to say.
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
  the list of every `mpm` on PATH in lookup order, which the message prints itself,
  and `mpm install` in a launcher's tree for a record that names a shadow.
  The sentence used to name `command -v -a mpm`,
  which bash refuses (`command: -a: invalid option`) and zsh refuses (`command not found: -v`);
  only fish's `command` takes `-a`.
  Nothing a person has to read now depends on a shell's own spelling:
  `real_binary::lookup_in_path` keeps every candidate it saw with the reason it passed it over,
  and the message prints that list.
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
- The last arm keeps the system's own words and names no step that cannot show the error:
  the hand run of the program is gone,
  because it cannot reproduce a refusal of this start,
  and for `E2BIG`, `ENAMETOOLONG` and `ELOOP` the arm says that running the same command again cannot succeed
  and names what to change instead.
  The other refusals keep the advice to run the command again,
  with the reason it can work (a shortage passes),
  and the words the system printed stay what to act on:
  words about the program's own file mean a damaged file,
  which installing it again replaces,
  and words about a limit mean work to wait for or a limit to raise.

## When the lookup finds no real `mpm`

The shadow's other message, for a `PATH` that holds no real `mpm` at all, exits 127
(`src/shim.rs`).
It said `mpm` is not installed,
that every `mpm` on `PATH` is this shadow or a launcher of one,
or that an install had moved the real `mpm` aside at the moment of the lookup.
A file named `mpm` with no execute bit is none of those:
the lookup skips it without a word,
so the message told the person to install an `mpm` they already had,
when the step the case needs is `chmod +x`.

Measured 2026-11-27 in a fixture:
with one `mpm` of mode 644 on `PATH` and nothing else,
alpha.14 printed that sentence and exited 127,
and alpha.13's shorter sentence ("no real mpm found on PATH") was true but named no file either.

The lookup now keeps every candidate it saw with the reason it passed it over
(`SkipReason` in `src/real_binary.rs`),
and the message prints that list in `PATH` order and builds the remedy from it:
`chmod +x <path>` for a file without its execute bit,
replacing a directory of that name or taking its directory off `PATH`,
pointing a symbolic link that leads nowhere at the real `mpm`,
and installing the real `mpm` only where nothing named `mpm` is there at all.
A directory on `PATH` that holds no `mpm` adds no line:
the list is of programs found, not of directories read.
The status stays 127 and standard output stays empty.

Measured on the same fixture with the changed binary:
the mode-644 case names the file and its mode and prints the `chmod`,
and a run with a directory named `mpm`, a mode-644 file and a link to no file names all three with their own repairs.

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
  (`src/start_failure_tests.rs`,
  `src/real_binary_tests.rs`).
- The interpreter naming:
  `src/named_interpreter.rs` and its tests,
  and a fixture run of both the alpha.14 binary and the changed one on the same files:
  a compiled program built with `gcc -Wl,--dynamic-linker=<a copy of the system's own linker>`,
  that copy held open for writing by a separate process (append mode, so its bytes never change),
  and then renamed away.
  `src/program_bytes.rs` builds the same shapes by hand for the unit tests,
  so no test depends on what a program on this machine happens to name.
- The shell behaviour:
  with two `mpm` files on a `PATH` of the run's own,
  `bash -c 'command -v -a mpm'` printed `bash: line 1: command: -a: invalid option` and exited 2,
  `zsh -c 'command -v -a mpm'` printed `zsh:1: command not found: -v` and exited 127,
  and `fish -c 'command -v -a mpm'` listed every `mpm` on `PATH` and exited 0,
  each run 2026-11-27.

Not run:
a start that meets the changed entry through `PATH`,
because the lookup and the start check judge the same file the same way
and only a change between them can make the two disagree;
the test drives the start check directly instead.
Windows and macOS are out of scope.

This record is unapproved and non-final.
What was done is recorded here and in the commits.
What is open is the owner's review,
the option of repeating a start refused as busy,
which this change leaves exactly as it was,
and the launcher's own sentence for a program that vanished,
which still says the interpreter is named on the program's first line
(`src/toolchain/launcher/starting_it.rs`).
That file belongs to another round of this release pass,
so this round read it and left it unchanged.
