# The wrapper's alpha.46 release passes: the owner's four answers

Status: accepted by the owner on 2026-11-25, through the question tool, after the three Sonnet release passes over
`monochromatic-git-policy-cli-v0.6.1-alpha.45..955051b29f` (same vendor, second passes):
`~/temp/agent/release-pass-wrapper-alpha-46-2026-11-25/` (`one-program-root-modes/keep/review.md`,
`zfs-add-and-removal-route/keep/review.md`, `commit-push-lfs-lock-and-the-rest/keep/review.md`).
Each question carried its finding, its options with their costs, and a ranking.

## What `SSH_ASKPASS` holds

The finding (commit, push and LFS pass, H1):
the sentence of option J (`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`) cannot be started,
and OpenSSH's failure path for an askpass that cannot start kills the connection's `ProxyCommand`,
so behind a `ProxyCommand` or `ProxyJump`, when the first key the server accepts has a passphrase no agent holds and a later
key works, the command ends 255 where alpha.45's `false` ended 0
(real `sshd` and `ssh` 10.6p2 in a capped container; OpenSSH `readpass.c`, `clientloop.c`, `sshconnect.c`).
The owner had been told J's cost was a printed line, not a failing command.

Answer:
"Put `false` back (Recommended)".

So `SSH_ASKPASS` returns to `false`, alpha.45's value, on every connection.
The next step at a new host key then needs another delivery,
which brings back the options the owner did not pick for L4
(C, the wrapper reads Git's error output; C2, a paragraph after every exit 128; D, a `known_hosts` check before Git runs);
that choice goes to the owner separately, with the proposal
`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`.
Not chosen: "Sentence only without a proxy" and "Keep it, document the limit".

## How the root modes' usage reads

The finding (one-program pass, M1 and M2):
`--help` says the build "is installed as /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets"
and that ZED starts it by that name, while no release installs either (plan step 19's setup does),
and `usage: monochromatic-git-policy-cli zfs-root [mode]` typed as printed reaches Git as an unknown command,
because the words follow the program's path.

Answer:
"A: full spelling, not yet (Recommended)".

So the usage prints `usage: <file> monochromatic-git-policy-cli zfs-root <mode>` with `<file>` the root-owned file, drops the
brackets from `[mode]`, and says that plan step 19's setup installs `<file>` and links it for ZED and that no release installs
it yet; the sweep refusal that points at "the root-owned file the installed ZED event links to" says the same.
Not chosen: "B: also accept zfs-root first" and "C: leave it, add README text".

## Moving a plain worktree that shares a bound slot's last name

The finding (ZFS route pass, low 3):
with a bound slot at `.claude/worktrees/one`, a plain worktree elsewhere whose path ends in `/one` is refused by
`git worktree move` with "the record and this path disagree", by `SPEC.md`'s rule "one that names another path refuses".

Answer:
"B: refuse only the slot (Recommended)".

So the move refusal applies only when Git's own worktree list does not name the source as a different worktree;
the slot itself stays refused. `SPEC.md`'s rule is rewritten to that.
Not chosen: "A: keep the rule".

## Rolling back with `native:install`

The finding (one-program pass, low 4):
`native:install` checks that the installed program answers the root-mode words, which alpha.45 predates, so
`native:install 0.6.1-alpha.45` installs it and moves the lock before refusing.

Answer:
"B: skip the check below alpha.46 (Recommended)".

So below the first release that answers the words, the install checks what the older release can answer and skips the words
check, with that floor kept beside the release list.
Not chosen: "A: refuse up front, name the release".

## Settled without a question

Two findings of the commit, push and LFS pass follow earlier answers and are adopted as they stand:
the URL mask hides a path segment of 20 characters or more, because the owner's answer was "under 20 chars" (low 2), and the
agent command the next-step text names includes fish's form, because the 2026-11-20 answer names `eval (ssh-agent -c)` (low 1).
The ZFS route pass's high, that `git worktree remove -f -f <slot>` reaches Git, is fixed by refusing every spelling, because
`SPEC.md` says removal "is refused for a slot whose record proves a whole worktree" with no exemption.
