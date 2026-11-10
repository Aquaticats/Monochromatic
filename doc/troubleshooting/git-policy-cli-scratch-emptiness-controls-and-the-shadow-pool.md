# Two scratch-emptiness controls counted the pool the service keeps

Two `package/git-policy/cli` binary controls read the repository scratch directory after a commit
and required it to hold nothing.
The pool of finished shadow repositories the service fills ahead lives in that same directory
(`package/git-policy/cli/SPEC.md`,
"A shadow taken from the pool",
and `src/native/shadow/shadow_pool.rs`,
`pool_root`),
so both failed on every host run once the pool landed in `3548c59c82` on 2026-11-10,
and neither failed in the last container gate,
which ran before that commit reached `main`.

## Symptom

The whole binary suite on `main` at `b4ed7deb73`,
2026-11-10T16:05:49-05:00
(`mise run //package/git-policy/cli:native:test:binary:host -- --no-fail-fast`,
918 tests,
909 passed,
9 failed):

```text
# markdown::a_commit_keeps_its_git_input_in_the_repository_scratch_directory
thread 'markdown::a_commit_keeps_its_git_input_in_the_repository_scratch_directory' (3035227) panicked at src/native/binary/binary_markdown_tests.rs:382:5:
nothing is left in .../native-binary-3035217-commit-scratch-location/repo/.git/monochromatic-git-policy-cli-scratch after 30s: ["shadow-pool"]

# service_scratch::a_commit_leaves_no_scratch_directory_once_its_landing_is_over
thread 'service_scratch::a_commit_leaves_no_scratch_directory_once_its_landing_is_over' (3874474) panicked at src/native/binary/binary_service_scratch_tests.rs:54:5:
the repository scratch directory holds nothing
```

## Root cause

The pool is `<common-dir>/monochromatic-git-policy-cli-scratch/shadow-pool/<worktree>`
(`shadow_pool::pool_root`),
one folder per worktree,
filled beside the landing loop and claimed by a commit with one `rename`.
It stands there for as long as the worktree has a service,
so it is not something a finished commit owes a removal of,
and a control that counted every entry of the directory reported it as a leak.

Both controls said why they counted every entry:
`left_behind` names only the trees the wrapper removes,
and `the_scratch_directory_holds_nothing` exists so that an entry those prefixes do not name still counts.
That intent is right for a leftover and wrong for a name the wrapper keeps on purpose.

## The fix

`kept_on_purpose` in `package/git-policy/cli/src/native/binary/binary_support_scratch_left_behind.rs`
is the one place that says which name of a scratch place the wrapper keeps:
`git_policy_cli::shadow_pool::SHADOW_POOL_NAME`.
Both controls leave that name out and count every other entry as before,
so a second pool folder or a name nothing keeps still fails them.
The control `a_scratch_directory_that_holds_an_entry_the_prefixes_do_not_name_is_not_empty`
is unchanged and still requires an unlisted entry to count.

## Verification

- `mise run //package/git-policy/cli:native:test:binary:host -- event_path_commit:: service_scratch:: markdown::a_commit_keeps_its_git_input_in_the_repository_scratch_directory`:
  15 of 15 pass,
  49.655 s (`Summary [49.655s] 15 tests run: 15 passed, 906 skipped`).
- The same two controls failed in the whole run quoted above and pass in that selection,
  which is the only change made to them.

## What is not claimed

- The pool folder a sweep may remove is unchanged:
  `service_scratch_abandonment.rs` removes any entry of a scratch place by age,
  and a pool removed that way costs a commit its claim and nothing else,
  because an empty pool is not a failure (`SPEC.md`,
  "A shadow taken from the pool").
  No control here probes that removal.
- The container gate was not re-run after the pool landed;
  this record is a host measurement.
