# search-shadow 0.2.2 copies at different paths choose each other as the real `grep` and start each other without end

Recorded 2026-11-07T15:40:00-05:00.

Status:
fixed in this repository's four shadow crates
(`monochromatic-search-shadow`,
`monochromatic-pnpm-shadow`,
`monochromatic-cargo-shadow`,
`monochromatic-mpm-shadow`;
the release records are in `doc/planning/one-release-each-batch-2026-11.md`,
section "Release record:
the shadows never start a shadow").
The tools are `package/search-shadow/cli`,
`package/pnpm-shadow/cli`,
`package/cargo-shadow/cli` and `package/mpm-shadow/cli`,
which this repository owns,
so there is no upstream to file against.
The owner's rules,
verbatim:
"We should never invoke ourselves recursively.
That's bad design.
Audit,
remove,
and ban that design aggressively.";
"nothing should ever wait forever or indefinitely".

## Symptom

A `cargo-shadow` test hung for more than twelve minutes
(measured by the cargo-shadow agent on 2026-11-07).
`strace -e execve` showed two `search-shadow` links named `grep`:
the main checkout's `.cache/search-shadow-bin/grep` (0.2.2)
and a linked worktree's stale link of 0.2.1.
Each picked the other as the real `grep` and replaced itself with it,
in the same process,
at about 89 percent of one CPU.
The agent killed that `grep`;
refreshing the worktree's links with `mise run toolchain:install` ended it for good.
The earlier launcher fix
(`doc/troubleshooting/mpm-shadow-launcher-names-a-deleted-release-folder-after-a-newer-install.md`)
does not reach it,
because the `search-shadow` links are not launchers.

## Root cause

`find_in_path` of `search-shadow` skipped a `PATH` candidate only when its resolved path equalled the running program's resolved path
(`package/search-shadow/cli/src/real_binary.rs`,
lines 196 to 200 before the fix):

```rust
if let Ok(real) = fs::canonicalize(&candidate)
    && real == skip
{
    continue;
}
```

`.cache/search-shadow-bin/*` are direct symlinks to the versioned program folder of the toolchain
(`ls -l`:
`rg -> .../cargo-monochromatic-search-shadow/0.2.2/monochromatic-search-shadow-0.2.2-x86_64-unknown-linux-gnu/search-shadow`),
so two trees that hold two copies at two paths each have the other's link first on its `PATH`:
the first shadow resolved to its own path and skipped itself,
found the second tree's link,
which resolves to another path,
took it for the real `grep`
and replaced itself with it (`replace_process`,
`package/search-shadow/cli/src/launch.rs`,
`command.exec()`);
the second did the same in return.
The version is not the trigger,
only the path:
two copies of one version at two paths loop the same way.
`exec` keeps the process number,
so there is no process limit to stop it
and nothing counts the hops:
`search-shadow` had no loop guard at all.

An earlier reading was wrong:
the sentence in the launcher document above,
"`pnpm-shadow` and `search-shadow` have the same skip rule and are covered by the same rewrite",
is not true.
`search-shadow` had no launcher-record rule and no guard,
and the other shadows had guards of their own
(`pnpm-shadow` and `cargo-shadow`:
an environment value that lists the candidates already picked and ends the chain after at most eight hops;
`mpm-shadow`:
a nesting count that allows four levels),
each of which lets a shadow start another shadow before it stops.

## The same design in the other shadows

Starting a shadow as the real tool was a property of every `*-shadow` lookup,
not of `search-shadow` alone:

- `pnpm-shadow`:
  `find_in_path` skipped by resolved path only
  (`package/pnpm-shadow/cli/src/real_binary.rs`,
  `skip.contains(&resolved)`).
  Another version at another path was started once,
  then the guard (`PNPM_SHADOW_GUARD`) made the second instance skip the first's pick,
  and the chain ended at the real pnpm after one or two extra shadow starts.
  Red test:
  the real pnpm saw no arrival marker (no shadow had been skipped) and a script that starts the other shadow ended with 127 instead of an arrival stop.
- `cargo-shadow`:
  `find_in_path_with_image` skipped the running image by inode,
  and launchers whose `.launch` record leads back to it.
  Another version at another path was started,
  and its own guard stopped it with the long arrival message and exit status 2:
  the whole `cargo` command failed
  although a real cargo stood behind both.
- `mpm-shadow` shim (`mpm`):
  `find_in_path` skipped its own path and launchers whose record names its own path.
  Another version was started as a child,
  each took the other as the real `mpm`,
  and `MPM_SHADOW_NESTING` ended the chain at the fourth level
  with exit status 126:
  four nested shadows and a failed command.
- `mpm-shadow` launcher (`run_launcher`,
  `package/mpm-shadow/cli/src/toolchain/launcher.rs`):
  it replaces itself with the program its record names
  and checked nothing about that program.
  A record that names its own launcher,
  or two records that name each other,
  loop in the same process without end.
  Both cases were reproduced by tests that ran into their five second bound.
- The Git wrapper (`package/git-policy/cli`) already has the arrival marker `CLI_GIT_STARTED_AS_GIT`
  (`src/native/invocation/started_as_git.rs`,
  tests in `src/native/binary/binary_never_self_start_tests.rs`);
  it is the model for the rule below and was not changed.
- Checked and not a shadow or a launcher:
  `package/monorepo-task/core-exec/src/mpm-shadow.ts` (runs a named program,
  never a `PATH` lookup of its own name),
  and the programs that start themselves by `current_exe` for a server (`file-enforcer`):
  they are not found through `PATH` under a shadowed name.

## Verification

Reproduction (a disposable directory under `${HOME}/temp/agent`,
two copies of the installed 0.2.2 program,
`PATH=<copy a>:<copy b>:/usr/bin`,
`strace --follow-forks --trace=execve`,
inside `systemd-run --user --scope` with `CPUQuota=25%`,
`MemoryMax=256M`,
`TasksMax=64`,
`IOWeight=10`,
`RuntimeMaxSec=20`,
killed by the script after four seconds):

```text
outcome: still running after 4 s, killed
execve lines: 419 in 4.00 s = 105 per second (under strace)
  210 x <scratch>/a/grep
  209 x <scratch>/b/grep
processes still naming the scratch directory: none
```

Before the fix,
the new tests (`package/search-shadow/cli/tests/never_start_another_shadow.rs`),
run once under a capped scope with a five second bound per run:
two controls passed;
three loops hit the bound
(`two_copies_of_different_versions_that_each_come_first_for_the_other_end_at_once_and_the_real_tool_runs`,
`two_copies_with_no_real_tool_behind_them_end_at_once_with_the_not_found_message`,
`a_script_that_starts_the_other_shadow_stops_on_arrival_with_one_line_and_a_non_zero_status`);
two failed at once (`a_launcher_record_that_names_another_shadow_version_is_skipped`,
the arrival marker test).
The other three crates have the same tests (`tests/never_start_another_shadow.rs`),
and the `mpm-shadow` launcher loops added two more bounded failures.

## Fix

One rule,
in three parts,
none of which looks at a path or a version
(`src/shadow_identity.rs`,
a byte-identical copy in each of the four crates;
`package/search-shadow/cli/tests/shadow_identity_copies.rs` fails when they differ):

1.  Identity.
    Every shadow program carries `MONOCHROMATIC-SHADOW-IDENTITY/1 <crate> <version>` in the ELF section `.monochromatic_shadow_identity`.
    A lookup skips a candidate that carries it,
    and follows a launcher's `.launch` record (at most eight hops) and judges the program it names.
    A launcher is a copy of `mpm-shadow`,
    which carries the text,
    so a launcher with a record is never judged by its own bytes.
2.  The start.
    The chosen program is opened once,
    checked while open and,
    when compiled and not a launcher,
    started through the open file (the way `cargo-shadow` vets its real cargo).
3.  Arrival.
    Every start of the real tool that a shadow's shim makes sets `MONOCHROMATIC_SHADOW_STARTED_BY` (mode,
    process number,
    birth identity,
    starter),
    and a shadow that finds it left for its own process or its direct parent stops at once with one line and exit status 2.
    This catches what no check of a file sees (a script that runs a shadow).
    A script that the real tool runs is further down and passes.

`mpm-shadow`'s launcher additionally refuses a record chain that leads back to the launcher,
in a circle,
or through more than eight launchers
(`src/toolchain/launcher_chain.rs`),
with exit status 126 and a message,
before it replaces itself.

No shared crate was added:
the four crates share no dependency,
the existing `guard.rs` and `notice_marker.rs` are already copies between `cargo-shadow` and `pnpm-shadow`,
and a new published crate changes the release and pin surfaces and the crates.io footprint that the owner wants small.
A shared crate is the broader option and is left to the owner.

## Limits

- A tree whose links point at an older `search-shadow` (or any older shadow) is not protected until `mise run toolchain:install` runs in it:
  the older program has no identity and no arrival check,
  and the lookup of the newer one skips it by identity only when the older one carries the text.
  An older copy still starts a newer one as a real tool,
  and the newer one then stops on arrival with the one line,
  instead of looping.
- A launcher with no record that leads to a shadow is started once and stopped on arrival;
  the toolchain's own launchers always have a record.
- `cargo-shadow`'s install step starts the real cargo inside its contained runner (`src/cargo_run.rs`,
  `src/bounded_run.rs`,
  `src/control_group.rs`) and does not set the arrival marker on that start.
  That start keeps its vetting (`vet` refuses a shadow while the file is open,
  and `refuse_if_ourselves` runs again before the start),
  and the cargo it starts was chosen by a lookup that now skips every shadow by identity.
  Whether `CARGO_SHADOW_GUARD` reaches that start was not checked.
- A candidate that cannot be opened,
  or a record that names a file that does not exist,
  is no verdict:
  the arrival stop is the backstop.
- Only 64-bit little-endian ELF programs are read for the identity.
  A shell script or a byte copy of a shadow made without its section is not recognized by its bytes.
- The cost measured with `hyperfine` (500 runs per arm,
  `rg x` through a link to `true`,
  run under `env`):
  mean 2.2 ms for the installed 0.2.2 against 2.3 ms for the change,
  and 2.1 ms against 2.4 ms in the repeat.

## Upstream filing decision

Not applicable:
all four tools are this repository's own,
so there is no third party to report to.
