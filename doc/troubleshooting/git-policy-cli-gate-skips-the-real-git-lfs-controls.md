# The wrapper's container gate skipped every control that proves the upload against the real `git-lfs`

## What a person meets

A gate can pass while no control ever starts the real `git-lfs`:
the base image the gate builds on,
`package/git-policy/cli/native-test-base.Containerfile`,
installed every tool the tests start except that one,
and every control that uploads against the real program printed one line and returned:

```text
skipped: git-lfs is not on PATH, so the real upload was not exercised
```

A gate-like run of the two modules of that kind
(a `PATH` naming one empty folder:

```sh
PATH=<empty folder> target/debug/deps/native_binary-<hash> --exact <name> --test-threads=1
```

) reported `22 passed` with 22 of those lines and took under half a second,
where the same 22 controls run for real on a host with `git-lfs` in about a minute.
The upload plan's own promise
(`doc/troubleshooting/git-policy-cli-lfs-upload-plan-trusts-a-clones-tracking-refs.md`)
therefore rested on a host run,
never on the gate:
a change that broke the real upload passed the gate.

## Root cause

The image's `pacman` line named `git`,
`cargo-nextest`,
`nodejs`,
`pnpm`,
`strace`,
`man-db`,
`perl`,
`python`,
`openssh`
and `procps-ng`,
and not `git-lfs` (`native-test-base.Containerfile`,
the `RUN --mount=type=cache,target=/var/cache/pacman/pkg` step).
Each control found the program by walking `PATH` and returned `None` without it:

```rust
fn real_lfs_program() -> Option<PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&path)
        .map(|directory| { return directory.join("git-lfs"); })
        .find(|candidate| { return candidate.is_file(); });
}
```

(`src/native/binary/binary_lfs_server_tests.rs`,
before this release).
A `None` printed the line and the control passed,
and nothing in the suite distinguished "checked and passed" from "checked nothing",
which is why the gate's summary read as full coverage.

The five files that carried such a skip were
`binary/binary_lfs_server_tests.rs` (`lfs_world`,
twenty controls),
`binary/binary_lfs_command_scope_tests.rs` (`link_git_lfs`,
two controls),
`binary/binary_hooks_off_tests.rs` (`the_recognized_shim_bytes_are_the_ones_the_real_git_lfs_writes`),
`lfs/lfs_pointer_tests.rs` (`the_port_agrees_with_the_real_decoder`)
and
`binary/binary_service_commit_tests.rs`
(`a_scoped_commit_beside_a_stat_dirty_lfs_asset_lands_through_the_service`).
The last one was not in the release pass's list:
its skip is keyed on the same absence and reads
`skipped: git-lfs is not installed, so no LFS filter ran`.

## What fixes it

- The base image installs `git-lfs` from pacman,
  so the program follows pacman's current release,
  as Git and Node already do.
- The test image sets `CLI_GIT_REQUIRE_REAL_GIT_LFS=1`
  (`bin/native-test-containerfile.ts`),
  and every one of the five skips goes through one decision
  (`src/native/testing/real_git_lfs_the_run_requires.rs`):
  with that variable set and no program found,
  the control fails and names the variable and the image step,
  so an image that loses `git-lfs` is red.
- The four end-to-end carry controls of `binary/binary_push_programs_end_to_end_tests.rs`
  run wherever they are,
  because every list they hand a push names folders of their own fixture alone:
  the stand-in's folder or the red arm's,
  and a folder of links to the programs a push needs.
  A list that named `/usr/bin` or `/bin` found a real `git-lfs` there first,
  so the stand-in never ran and the red control's refusal never happened.
  Those four once declined behind `CLI_GIT_END_TO_END_LFS_OCCUPIED_ACKNOWLEDGED=1`,
  which passed a control that ran nothing;
  the variable and its guard are gone,
  and nothing in that file can decline.
- Three controls prove the refusal for a program the list does not hold,
  and they used to name `/usr/bin` and `/bin`:
  on a host or image with `git-lfs` installed,
  the search found the real program and the control failed.
  They now search a folder of their own fixture that links only the programs the push needs
  (`binary_lfs_parent_tests.rs`,
  `binary_lfs_pre_push_tests.rs`,
  and `push_git_through_a_list_of` in `lfs/lfs_upload_tests.rs`).

## Verification

- A host with `git-lfs` 3.8.1 at `/usr/bin/git-lfs`:
  each of the five controls,
  run alone with `CLI_GIT_REQUIRE_REAL_GIT_LFS=1` and a `PATH` that names no program folder,
  failed naming the variable,
  where the same run without the variable printed its line and passed.
- The gate image's own run of the two real-`git-lfs` modules:
  `lfs_server::` and `lfs_command_scope::` ran and passed
  there rather than printing the skip line.
- The three missing-program controls passed on that host,
  whose `/usr/bin` holds `git-lfs`.
- The four end-to-end carry controls,
  on a host whose `/usr/bin` holds `git-lfs` 3.8.1,
  after the lists became their fixture's own folders:
  all four passed in one run of that file,
  24.39 s,
  and the same file's controls each ran their push,
  which their journals show
  (`~/temp/agent/end-to-end-push-program-controls-run-in-the-gate-2026-11-24/keep/`).
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  Before that change the red control failed after 120.44 s with the real program answering its push,
  and the restart control failed the same way,
  so the decline had hidden a control that could not run here at all.
- The four end-to-end carry controls in a gate image built from that change,
  which holds the real `/usr/bin/git-lfs` 3.8.1 and sets no acknowledgement variable:
  `4 tests run: 4 passed, 1335 skipped` in 14.91 s,
  each control passing in 1.57 to 6.84 s,
  where a declined control passes in no time at all
  (`keep/gate-image-run-of-the-four-controls.log` in that round's folder).
- The old controls with the acknowledgement the image set,
  run on this host to measure the decline the 2026-11-24 gate's own output no longer holds:
  four `END TO END CARRY CONTROL NOT RUN` lines,
  `4 passed` in 0.00 s,
  against 24.39 s for the same four when they run
  (`keep/measured-decline-of-the-old-controls-with-the-acknowledgement.log`).

## What does not work

- Relying on a host with `git-lfs` for these controls:
  the release pass's own gate recorded the skips,
  and an alpha is dispatched without host controls at all.
- Checking `git-lfs` for a presence and skipping when absent:
  that is the behaviour this doc records,
  and it reads as a pass in every summary.
- Handing a push a list that names a folder any machine installs programs in,
  and declaring the control unable to run where one of those folders holds the real program:
  the four end-to-end carry controls did that,
  the gate image installs the program,
  and each control then passed in no time without running.
  A list of the fixture's own folders keeps the control the same everywhere.
