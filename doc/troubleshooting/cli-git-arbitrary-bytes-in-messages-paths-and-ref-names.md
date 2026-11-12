# A commit message, a path or a branch name that is not UTF-8

## What a person saw

Three commands native Git completes failed or changed the person's data through the wrapper,
audited on 0.6.1-alpha.10 (the audit's own report is quoted in each section):

- `git commit -m <message>` with a message byte that is not UTF-8:
  the commit landed carrying U+FFFD where the person's byte was.
- `git commit --pathspec-from-file=<name>`, the name holding a byte that is not UTF-8:
  the command ended with `transaction-failed`, and Git said it could not open a path holding U+FFFD.
- `git commit` on a branch whose name holds a byte that is not UTF-8:
  the command ended with `Git returned non-UTF-8 output.` and nothing was committed.

## Why

Git's names and messages are bytes, and the wrapper read them as text.
`String::to_string_lossy` replaces each byte that is not UTF-8 with U+FFFD,
so the value that reached real Git was a different message, a different file name, and,
in the capture, no value at all: `TextDecoder('utf-8', { fatal: true })` refuses it.

Measured on 2026-11-11 with the release binary of 0.6.1-alpha.10 in a disposable repository:
`git commit -m 'message-\xff'` stored `6d6573736167652dc3bf0a0a` through native Git
and `6d6573736167652defbfbd0a0a` through the wrapper.

## What the wrapper does now

- Every argument of the commit's own line is carried as `OsString` from the command line to the child
  (`src/native/commit/commit_args.rs`), so the transforms move tokens and never spell them.
- A `--pathspec-from-file` value becomes a `Path` from its bytes, and the file is read where it was named
  (`src/native/landing/service_prepare_changes_config_and_pathspec_steps.rs`).
- A reference name is a `RefName` holding the bytes Git printed
  (`src/native/foundation/ref_name.rs`).
  It travels through the capture, the shadow, the request line and the landing as bytes,
  and a record writes the plain `ref=` field only when text spells every byte of it,
  with `refHex=` always holding the whole name.
  This matters for recovery: an earlier release reading a lossy `ref=` would move another branch.
- A message, a greeting or a stop names such a branch in the `$'...'` form
  (`src/native/worktree/shell_quote.rs`), so a person can paste it back into a shell.

## The one thing the wrapper does not do for such a branch

A push record names its branch in text
(`src/native/landing/work_item.rs`),
and the push subsystem compares it with `refs/heads/` prefixes and pushes it by name.
A commit on a branch whose name is not text therefore leaves no push record,
and the commit's hand-off says so once:
the commit is on the branch, and a push the person makes sends it.

Queueing one would be worse than saying it:
a record holding `refs/heads/topic-\xef\xbf\xbd` would push another branch,
or fail every time it ran.

The removal is a later change:
`service_handoff.rs`, `service_server_land_group_owed_pushes.rs`, `push/auto_push.rs` and
`landing/work_item.rs` would hold the name as bytes, and every existing record would keep its text field.

## How it was found

The audit `alpha-10-commit-path` of Kimi K3.1 Pro reported all three,
with a reproduction for each.
Each reproduction is a disposable repository, a fixture `HOME` and runtime directory,
and a service started from the built binary;
each compared the wrapper with native Git on the same fixture.
