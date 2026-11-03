# Subvolume and service-commit survivors of the resumed campaigns

This handover cross-checks the survivors of two mutation campaigns that finished after a reboot against the dispositions already
recorded,
maps each to current main,
and records what was killed here.
Respond with a veto of a named disposition,
or with evidence that one does not hold.

The evidence is `~/temp/agent/mut-q/subvolume-2-survivors.txt` and `subvolume-2-resumed.log` (gate 33's image),
`timing-service-resumed.log` (gate 34's image),
and the `mutants.out/diff` files of
`.claude/worktrees/integrate-060/.../native-mutation-4fa889528dfd` and
`.claude/worktrees/integrate-061/.../native-mutation-a39e48392ce1`.
The branch is `test/git-policy-cli-subvolume-timing-survivors`,
based on main `2a0b8e7417`.

## Mapping to current main

Every one of the 25 subvolume-2 lines names code that is on current main unchanged,
at the same file and line:
the mutant's original line was found by its text in `fs_mount_options.rs`,
`subvolume_sha.rs`,
`subvolume_platform.rs`,
`subvolume_sync_attrs.rs` and `subvolume_sync.rs` (a few texts also occur on other lines,
such as `+= 1` counters).
The two `service_commit.rs` survivors moved:
`141:83` is `arguments_after_subcommand` here and `213:75` is
`switched_branch_name`.

## Already dispositioned: the subvolume-2 survivors

The ladder agent killed or argued every one of them on `btrfs-worktree-ladder`,
commit `b6078d4bf4` (tests) with the record in
`938b85d751`,
section "2026-11-02:
subvolume campaign survivors,
dispositions".
That branch is not on main yet,
so on main they still stand until the ladder lands;
nothing was done for them here,
because the
ladder's commit also changes `subvolume_sync.rs`,
`subvolume_sync_attrs.rs` and `subvolume_platform.rs` and a second copy of its
tests would conflict.
The tests exist on the ladder branch under these names (checked with `git grep`):

- `fs_mount_options.rs` `parse_line` (three operators,
  line 104):
  `a_line_is_read_only_with_the_separator_late_enough_and_three_fields_after_it`.
- `fs_mount_options.rs` `longest_mount_holding` (145 and 151):
  `a_path_no_chain_of_ids_reaches_falls_back_to_the_longest_mount_point`.
- `subvolume_sync_attrs.rs` `set_owner` (the body,
  `&&`,
  both `==`,
  `!=`,
  `+=`) and `carried_attributes_equal`:
  `the_owner_is_left_alone_only_when_user_and_group_both_agree`,
  `an_owner_that_cannot_be_given_is_counted_and_an_equal_one_is_not_touched`,
  `a_group_the_user_belongs_to_is_given_and_not_counted`,
  `carried_attributes_are_equal_only_when_names_and_values_agree`.
- `subvolume_sync.rs` `Pass::visit` (244,
  245,
  264 three operators) and `Pass::copy_file` (461,
  467,
  472):
  `a_vanished_entry_is_counted_and_any_other_failure_is_returned`,
  `a_directory_that_vanished_is_counted_and_one_that_cannot_be_listed_is_an_error`,
  `an_unopenable_source_file_is_vanished_unreadable_or_an_error_by_its_reason`,
  `only_a_file_with_several_names_is_recorded_and_linked`.
- `subvolume_platform.rs` `is_subvolume_root` and `create_subvolume` (76,
  80):
  `btrfs_the_platform_creates_a_subvolume_and_tells_it_from_a_directory`.
- `subvolume_sha.rs:76:31` and `:128:41`:
  equivalent,
  by reading (their record).
  Corroborated here by exhausting the three input bits:
  `(b & c) | (!b & d)` equals `(b & c) ^ (!b & d)`,
  and
  `(e & f) ^ (!e & g)` equals `(e & f) | (!e & g)`,
  for every bit combination,
  and the functions work bitwise.

A limit of that record that a rerun will show:
the platform control runs only on a btrfs host
(it prints a loud not-run line elsewhere),
so on the container image both `subvolume_platform.rs` mutants stay missed
until a control that needs no btrfs exists.

## New

### `fs_mount_options.rs:182:81`, `==` to `!=` in `mount_holding` (a timeout, not a miss)

The mutant turns the test of `here` (`mount.mount_point == prefix`) around.
Applied by hand to the code of this branch and run with the tests of `fs_mount_options`:
one test fails by assertion (`the_longest_ancestor_mount_holds_the_path`),
and three never return
(`a_mount_hidden_under_a_later_mount_does_not_hold_the_path`,
`stacked_mounts_resolve_to_the_top_and_an_unlinked_table_to_the_longest_mount_point`,
`the_live_table_answers_for_an_existing_path`):
the inner `loop` keeps taking another mount,
because a mount not at the prefix is
always another one than the one held.
So it is detected,
and cargo-mutants scores it as a timeout only because the hanging tests keep the run from ending in its
bound (the same reading as the 17 loop mutants of the foundation handover:
"neither caught nor missed").
No test was added and no bound put into the loop:
the original loop cannot cycle (a mount is taken only when it is not the one
held,
and the parent ids lead down from the root of the walk,
so a cycle is not reachable from a real or a cut table),
and a bound that nothing can reach would be code written for the mutant.

### `service_commit.rs:141:83`, `+` to `*` (and `-`)

`let after = arguments.get(subcommand_index + 1..)`:
with `*` the words after the subcommand began at the subcommand itself.
The same expression at the start of `standard_input_for` was caught;
this one decided `opens_editor`,
the parse of the service
commit line and the quiet flag,
and nothing compared the slice.
It is now `arguments_after_subcommand` (both uses),
with `the_words_of_the_command_start_after_the_subcommand`:
red with `*` and with `-` (applied by hand:
`subcommand_index * 1..`,
`subcommand_index - 1..`),
green after the file was restored.

### `service_commit.rs:213:75`, `==` to `!=`

The closure that named a branch for the message of a worktree that switched under a commit
(`reference == "HEAD"` gives "a detached head") is now `switched_branch_name`,
with
`a_switched_branch_is_named_as_the_person_knows_it`:
red with `!=`,
green after.
The message wording itself was already controlled in `service_messages_tests.rs`;
what was not is which reference gets which name.

## How each was shown red

The mutant was applied to the worktree source by one exact text replacement,
only the named tests were run in a capped systemd
unit (CPU 300%,
memory 8G,
tasks 512,
I/O weight 30) with `cargo nextest`,
and the file was restored.
The runner and the list are `~/temp/agent/survivors-subvolume-timing/run.ts` and `mutate.ts`;
the logs are `m-<mutant>.log` there.
