# The next step a typed network command names at a new host key: the owner's answers

Status: accepted by the owner on 2026-11-24T14:30:00-05:00, through the question tool.
The question is L4 of the alpha.45 release pass
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/commit-push-lfs-lock/keep/review.md`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
The proposal is `doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
third revision merged as `077c006094`.
The questions carried the corrections of a same-vendor read of that revision
(`~/temp/agent/gate-review-typed-network-command-third-revision-2026-11-24/keep/read-of-the-third-host-key-revision.md`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

The pass read L4 and did not run it.
Its finding:
a typed `git fetch`,
`pull`,
`clone` or non-LFS `push` meets an unknown SSH host key with OpenSSH's one line only,
because `host_key_not_known_sentence` (`lfs/lfs_pre_push.rs:644`) has callers only in the LFS upload and the push retry
(`lfs/lfs_upload_explanations.rs:226` and `push/push_retry_failure.rs:304`),
and `SPEC.md` around line 8010 says so.
The control the pass looked at asserts only "Host key verification failed" and exit 128,
so a typed command's first connection to a new host ends with that line and no way out,
outside those two places.
The pass notes this is the owner's decision of 2026-11-20 as recorded,
whose cost shows at first contact with every new host.

## The owner's rule the questions turned on

`doc/planning/cli-git-no-self-invocation.md`, the owner's words:
"We should never invoke ourselves recursively. That's bad design. Audit, remove, and ban that design aggressively."
The owner was told that the hook the record removed was Git starting the wrapper from a path the wrapper wrote,
and that options B and E have that shape.

## The answers

### B, the wrapper's program as OpenSSH's `SSH_ASKPASS`

Answer:
"Never (Recommended)".

### E, the wrapper's program as the ssh Git starts (`GIT_SSH_COMMAND`)

Answer:
"Never (Recommended)".

So the rule stays as written:
no program the wrapper did not start explains anything,
and the wrapper never hands its own path to ssh or to Git to start.

### The option

Answer:
"J: the sentence in SSH_ASKPASS (Recommended)".
The wrapper sets `SSH_ASKPASS` to the sentence itself,
so OpenSSH's failed attempt to start it prints one extra line before OpenSSH's own refusal,
of the form `ssh_askpass: exec(<the sentence>): File name too long`.
The owner was told its costs:
one generic text in OpenSSH's framing,
in which the person fills in the host;
the line also prints on some commands that succeed
(a passphrase key skipped before a working one,
or `AddKeysToAgent=ask`);
it is silent under `ssh -q`;
and its text must not name a host key outside single quotes,
or the wrapper's host-key detector (`lfs_pre_push.rs`, `reads_as_unknown_host_key`) needs a fix that lands with it,
so the upload's and the push service's own host-key sentences keep working.

## What the owner was shown and did not pick

C (the wrapper reads Git's error output),
C2 (a paragraph after every exit 128),
D (a `known_hosts` check before Git runs)
and A (change nothing).
The proposal keeps each with its evidence.

## State

The round that builds J and rewrites the design to the chosen option is the next step.
