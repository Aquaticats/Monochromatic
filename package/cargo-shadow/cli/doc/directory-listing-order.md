# The directory listing order in cargo-shadow

`fs::read_dir` answers in whatever order the file system stores a directory in,
not in the order of the names,
and the order differs between machines and between two directories holding the same names.
So a comparison against a stated order,
and a message that names the first entry it found,
both depend on the file system rather than on this tool.

## Measured on this host

The cargo build folder and the tests' `test-scratch` beside the test executable
(`package/cargo-shadow/cli/src/test_scratch.rs`) are on `zpcachyos/nosnap/cargo-build`,
and `/home` is on `zpcachyos/ROOT/cos/home`.
Both are ZFS.

A directory made as the replacement protocol leaves one
(the target `Cargo.lock`,
a sibling `.{name}.shadow-{suffix}.tmp` swapped into place,
then the displaced file hard-linked to `Cargo.lock.displaced`,
then the temporary unlinked)
answered in name order three times and reversed five times in eight rounds
(`getdents64` through `os.scandir`,
measured 2026-11-27).
The answer was not a property of that directory alone:
the temporary's name carries a clock and a counter,
so no two rounds made the same three names,
and the answer changed with them.

A `tmpfs` answers in creation order instead,
so the same protocol,
whose `.displaced` link is always created last,
answers reversed every time:
eight of eight rounds in a private mount namespace.

## Where this tool sorts, and where it does not

`read_ready` (`src/journal_directory_readers.rs`) sorts the listed files and the files its indexes imply before it compares them,
and `list_journals` in the same file sorts the open journals and the done ones after collecting them.
`journal_dir` (`src/journal_support.rs`) asserts that a root holds exactly one journal and removes it,
so no order reaches its answer either.

Two paths let the order reach what the person reads,
and neither reaches a verdict:

- `unready_verdict` (`src/journal_unready_verdict.rs`) reports the first name it finds that this version does not know,
  so two unknown files are named in either order.
- `report_leftover_temporaries` (`src/journal_recover_one.rs`) pushes one message per leftover temporary in the order the directory answered,
  so two of them are reported in either order.

Both are defects of reproducibility and neither is fixed here.
The fix is to sort the names before either reads them,
and a control for it fails only on an arm whose listing order is not the name order,
which the recipe below makes deterministic.

## The order a refusal names a scope's members in

One listing of a scope's members is ordered by the walk and never by the file system
(`listings` in `src/control_group_stop_listing.rs`,
stated on `Listing` in `src/control_group_stop_report.rs`):
the scope's own group first,
then each group below it depth first,
with the greatest name first at each group,
and the processes of one group in ascending process id order.
Both orders are the tool's own:
`sub_groups` sorts the names of the groups below a group,
and the walk pushes them in that order onto a stack and takes the last one off,
while each group's processes are sorted before they are named.
The control is
`control_group_stop::leftover_tests::the_listing_names_the_scope_s_own_group_first_and_the_groups_below_it_greatest_name_first`,
which puts one member in the scope's own group,
one in `alpha` and two in `beta` below it,
says that the numbers were handed out in the order the members were started,
and then reads where each is named.
A listing that took the groups in the other order,
or one that sorted every member by process id across the groups,
fails it.

## Controlling a comparison, and running it on an arm where the order differs

A control that asks which names exist sorts them first:
`names_in` in `src/journal_tests.rs` is the one the journal controls use,
and issue 6553 is the control that compared an unsorted listing.

A run of one control on an arm whose order the test host does not have
gets a `tmpfs` over the test scratch in a mount namespace of its own,
whose order is creation order:

```sh
# run the built test binary with its scratch folder on a tmpfs
systemd-run --user --scope --slice=agent-work.slice --property=MemoryMax=4G --property=RuntimeMaxSec=600 -- \
  unshare --user --map-root-user --mount --propagation private -- \
  bash -c 'mkdir --parents <build folder>/debug/build/monochromatic-cargo-shadow/test-scratch &&
    mount --types tmpfs tmpfs <build folder>/debug/build/monochromatic-cargo-shadow/test-scratch &&
    <the test binary> --exact journal::journal_tests::a_compare_and_replace_that_finds_the_expected_text_replaces_it_and_leaves_no_temporary_file;
    umount <build folder>/debug/build/monochromatic-cargo-shadow/test-scratch'
```

The scratch folder is `current_exe()`'s third ancestor plus `test-scratch` (`src/test_scratch.rs`),
and `cargo nextest list --lib --message-format json` gives the binary's path.
The mount lives in that namespace and dies with it,
so nothing outside it is mounted or changed.
