# A journal listing removed the temporary file another service instance was writing

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

`lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other` also failed with
"The commit service could not finish this commit, so it was not made" (about 1 run in 13 at one CPU), apart from the
60-second settle failure that `pdeathsig-fires-when-the-parent-thread-ends.md` explains.

## What was measured

The service logged nothing for the refusal. A temporary log line at every `FailureCode::Unavailable` site and at
`storage_failure` showed `storage_failure` called from the `write_intent` call in `service_land.rs` with
`Os { code: 2, kind: NotFound }`. `write_intent` (`service_journal.rs`) creates `<seq>.tmp`, syncs it and renames it to
`<seq>.intent`. `list_intents` removed every `.tmp` file it met, as the remains of a killed write. The fixtures start the
service of each worktree with one shared `--journal` directory, and every landing begins by listing the journal
(`settle_open_batches`), so one instance listed the journal between the other's create and rename and removed its file.

## Fix

`list_intents` removes a temporary file only when it is older than `TEMPORARY_GRACE` (10 minutes); a write takes
milliseconds. Production gives each instance its own journal (`--journal %S/cli-git/service/%i` in
`service_units/cli-git@.service`), so this was exposed only to the fixtures and to a development run that shares a journal.
`a_temporary_file_that_is_being_written_is_left_alone` in `service_journal_tests.rs` failed on the old behavior.
Limit: the age is judged on the wall clock, so a step forward at that exact moment could still remove a live file.
