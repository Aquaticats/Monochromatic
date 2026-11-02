# Native service mutation survivors

This handover dispositions the survivors that the cargo-mutants campaigns left in the timing,
claim,
queue and review
code of `package/git-policy/cli`,
and records the claim-fix review findings fixed on the same branch.
Respond with a veto of a named disposition,
or by naming evidence that a disposition does not hold.

The work is on the branch `survivors-service` (worktree `.claude/worktrees/survivors-service`),
not merged.

## How each mutant was shown red

Every mutant was applied by hand:
one exact text replacement in one source file,
mounted read-only over that file in the
`localhost/git-policy-native-test:integrate-gate38` image,
with the branch's `src/native` mounted read-only over the rest.
The container ran with `--network=none --cpus=3 --memory=6g --pids-limit=512`,
inside a systemd unit capped at 300% CPU,
8G of memory,
1024 tasks and an I/O weight of 30.
Only the tests named in the filter ran,
once per mutant.
The runner is `~/temp/agent/survivors-service/run.ts` and the mutant list is `~/temp/agent/survivors-service/mutants.ts`.

The green command,
for example:

```sh
# ~/temp/agent/survivors-service
node run.ts green1 test 'test(=service_review::tests::the_review_clock_reads_the_wall_clock)'
```

The red command adds the mutated file as `<host file>=<file name under src/native>`:

```sh
# ~/temp/agent/survivors-service
node run.ts m-review-clock-0 test '<same filter>' /var/home/user/temp/agent/survivors-service/mutated/review-clock-0.rs=service_review.rs
```

The logs are `~/temp/agent/survivors-service/m-<mutant>.log`.

## Killed

- `service_review.rs` `wall_clock_millis` to `0` and to `1`:
  `service_review::tests::the_review_clock_reads_the_wall_clock` brackets the reading between two readings of the system
  clock.
  Both mutants fail it.
  The defect is a review entry dated 1970 and a gateway call timed from the epoch.
- `commit_timing.rs` `whole_milliseconds` to `1`:
  `commit_timing::tests::the_stamp_is_the_wall_clock_at_the_start` and
  `commit_timing::tests::whole_milliseconds_drop_the_part_of_a_millisecond` both fail.
  The earlier test accepted any `startMillis` above zero.
- `work_item.rs` match guard `AlreadyExists` to `true` in `enqueue`:
  `work_item::tests::a_record_that_cannot_be_written_is_an_error_and_not_known`
  queues an object name one character longer than a file name may be,
  so the write fails with a name error.
  The defect is a failed write reported as an item that is already known,
  which loses the work.
- `work_item.rs` stray temporary age `24 * 60 * 60` to `24 + 60 * 60` (3624 seconds):
  `work_item::tests::a_stray_recovery_temporary_file_a_minute_short_of_a_day_is_kept`.
  The older test checked one hour only,
  which the mutant also kept.
  The constant now lives in `work_item_recovery.rs`.
- `service_journal.rs` `TEMPORARY_GRACE` `10 * 60` to `10 + 60`:
  `service_journal::tests::a_temporary_file_of_a_writer_stalled_for_minutes_is_left_alone`.
- `service_client.rs` `COMMIT_WAIT` `10 * 60` to `10 + 60`:
  `service_client::tests::a_commit_waits_the_documented_ten_minutes_in_the_queue`,
  which pins the documented value
  (`doc/planning/cli-git-resident-service.md`,
  "bounded commit wait:
  default 10 minutes").
- `service_server.rs` `IDLE_EXIT` `3 * 60 * 60`,
  three mutants listed by the running campaign (`+` in either place,
  `/`):
  `service_server::tests::the_default_idle_exit_is_the_documented_three_hours`.
- `service_server.rs` `bind`,
  delete `!`,
  and `<` to `==`,
  `>` and `<=`,
  and the `==` and `||` mutants the running campaign
  also listed on the same lines:
  the check of `LISTEN_PID` and `LISTEN_FDS` is now `ensure_socket_passed_in(listen_pid, listen_fds, own_pid)`,
  a function of
  its arguments,
  so no test touches the process environment or file descriptor 3.
  `the_manager_passed_at_least_one_descriptor_to_this_process` and
  `activation_variables_for_another_process_or_unreadable_pass_nothing` kill all six mutants.
- `service_server.rs` `serve` grouping,
  `==` to `!=` and the three guard mutants,
  plus the sibling mutants of the same lines:
  the grouping is now `place_in_groups`,
  generic over the member,
  so a test passes plain numbers.
  `requests_of_one_branch_share_a_group_in_arrival_order`,
  `a_request_never_joins_another_branchs_or_repositorys_group` and
  `a_request_that_lands_alone_neither_joins_nor_is_joined` kill nine mutants of the function.
- `service_land.rs` `commits_from_marks`,
  `&&` to `||` (listed by the running campaign,
  not by the brief):
  `service_land::tests::marks_outside_the_batch_are_ignored_and_a_missing_mark_is_no_answer`.
  The mutant indexes past the batch and panics on a mark of zero or beyond the batch.

## Equivalent

### `service_client.rs` `read_message`, `read > 0` to `true`, and `>` to `>=`

`BufRead::read_line` returning `Ok(0)` leaves `line` empty.
`decode_service("")` runs `serde_json::from_str("")`,
which is an error (`service_protocol.rs`),
and `.ok()` makes it `None`,
the same value as the `_ => None` arm.
Both mutants therefore return `None` for a read of zero bytes,
exactly as the original does.
`service_client::tests::an_empty_or_damaged_line_decodes_to_no_message` fails if `decode_service("")` ever stops being an
error,
which is the only way the guard could start to matter.
Both mutants were run against the whole targeted set and passed it (`m-client-read-true.log`,
`m-client-read-ge.log`);
the third mutant of the line,
the guard to `false`,
fails `a_line_is_read_as_a_message_and_a_closed_connection_as_none`.

### `service_server.rs` `bind_path`, match guard `AddrInUse` to `true`

Unmutated,
any bind error other than `AddrInUse` is returned as it is.
Mutated,
such an error is kept as `first` and the code goes on to the two checks,
which both must hold to reach the
`remove_file`:
the path is a socket (`symlink_metadata`),
and a connect to it is refused (`ConnectionRefused`).
When either fails,
the function returns `Err(first)`,
the same error.
So the mutant differs only if the bind failed with another error while the path is a socket that refuses connections:

1. A bind to a path that exists fails with `EADDRINUSE` before any permission or read-only check.
   `net/unix/af_unix.c`,
   `unix_bind_bsd`,
   ends with `return err == -EEXIST ? -EADDRINUSE : err;`,
   and
   `fs/namei.c`,
   `filename_create`,
   looks the last component up with `LOOKUP_CREATE | LOOKUP_EXCL` (which gives `EEXIST` for a
   name that exists) before it acts on the `mnt_want_write` result.
   Both were read in the current `torvalds/linux` tree on 2026-11-02.
2. An error that comes from resolving the parent directory or from creating a name that was not there (`ENOENT`,
   `EACCES`,
   `ENOTDIR`,
   `ELOOP`,
   `EROFS`,
   `ENOSPC`) leaves no socket at the path,
   or one that `symlink_metadata` and `connect` cannot
   reach either,
   so a check fails.
3. A path too long or holding a NUL byte is refused by Rust before the system call,
   in `bind` and in `connect` alike
   (`InvalidInput`,
   not `ConnectionRefused`),
   so the refusal check fails.
4. An error from the socket call itself (`EMFILE`) fails the later `connect` the same way.

Link 1 is kernel behaviour that this crate cannot exercise;
the rest is in the code.
The only remaining difference is a race in which the path becomes a stale socket between the failed bind and the checks,
which no deterministic test can make.
No test was added;
the guard stays as it is.

## Tried and found equivalent in effect

The mutant that stamps the pending file before it is staged (`wi-stamp-pending`,
inserted by hand into the new claim code)
passes every test.
The claim code restores the pending age from the entry it listed before it queues the entry again,
so an early stamp on the pending file cannot change what remains.
It is recorded here and not killed,
because the claim code
does not stamp the pending file at all.

## Claim fix review (addendum)

The review is `~/temp/agent/review-claim-machine-2/sol-review-2.txt` (a cross-vendor reviewer;
not run by this branch).
Each finding was checked against the code on main before it was fixed.

- P1 stamp failures swallowed (`work_item.rs`):
  confirmed.
  `stamp_modification_time` now returns the error.
  A claim whose stamp fails is not taken and goes back to `pending`;
  a renewal whose stamp fails returns `None`.
  Controls:
  `a_claim_whose_stamp_fails_is_not_taken_and_keeps_its_age`,
  `a_renewal_whose_stamp_fails_renews_nothing`,
  `the_stamp_sets_the_time_and_reports_a_file_it_cannot_reach`.
  Red:
  the claim mutant that ignores the error fails the first and `a_stamp_set_before_a_failure_is_undone`;
  the renewal mutant fails the second.
  The stamp is injected into `claim_next_stamped` and `renew_claim_stamped`,
  because a file that another user owns cannot
  be made in a test that runs as an ordinary user,
  and a root user would not be refused.
- P1 file name and record disagree:
  confirmed.
  A record whose id differs from its file name is moved to `failed` when claimed and when recovered.
  Controls:
  `a_queued_file_holding_other_work_than_its_name_is_not_claimed`,
  `a_recovered_claim_holding_other_work_than_its_name_is_failed`;
  each fails with its guard removed by hand.
- P2 old malformed claims loop on `.retry`:
  confirmed in the code.
  `recover` checks the id before it builds the owned name and moves a malformed claim to `failed` by its own name.
  Control:
  `an_old_claim_with_an_id_that_would_not_fit_is_failed_in_one_pass`;
  red with the check disabled.
- P2 the stamp refreshes the pending age:
  confirmed.
  The entry is taken into `recovering` under its claim name,
  stamped there,
  and published to `claimed`;
  on failure it goes back to `pending` with the modification time it was listed with.
  The reviewer's warning about a race does not apply,
  because the staged file is the taker's alone.
  Controls:
  `a_claim_that_cannot_be_published_keeps_the_age_of_the_queued_record` (the `claimed` directory replaced by a
  file) and `a_stamp_set_before_a_failure_is_undone`;
  both fail with the restoring step removed.
  A taker killed between the take and the publication leaves the entry in `recovering`,
  where it runs out its lease.
- P2 `eprintln!` panics on a broken stderr:
  confirmed.
  The note now goes through `service_timing::log`,
  which drops the write result.
  Control:
  `a_failing_standard_error_does_not_stop_the_queue` points descriptor 2 at `/dev/full`;
  it fails with `eprintln!`
  put back.
- P3 SPEC sentence too broad:
  confirmed.
  `SPEC.md` now says only entries that start with a handled kind are quarantined,
  and describes the staged claim and the failing stamp.

## Not done, and limits

- `work_item.rs` was over the crate's code-line limit on main already (308 of 300 lines).
  Recovery moved to `work_item_recovery.rs`,
  which `work_item.rs` re-exports.
- The `rust/avoidable-closure` finding is reported all over the crate,
  including on lines this branch added,
  because the
  crate writes closure bodies with braces and `return`.
  It was left as the crate has it.
- The running campaign's `service_server.rs:44` mutants and `service_land.rs:132` are listed above;
  its final list may
  hold more.
