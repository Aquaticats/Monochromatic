# Slot refusals and the shell aliases they cannot read

Status:
unapproved and non-final,
a proposal for the owner and not a decision.
Written on 2026-11-27 for the alpha.48 release pass.
What was done:
the slot refusals now read the same command the destructive-command guard reads,
so a `!git <words>` alias reaches the removal,
move and prune refusals as the
direct spelling does
(`package/git-policy/cli/src/native/invocation/forwarded_command.rs`,
`the_command_git_runs`;
the control is
`a_bound_slots_removal_is_refused_behind_every_shell_alias` and its siblings in
`entry_slot_refusals_read_every_spelling_tests.rs`).
What was not changed,
and is the choice this file asks for:
a shell alias the wrapper cannot read still reaches Git.
Nothing here has been accepted,
and only the owner declares it complete.

## What the wrapper reads today

`SPEC.md`'s removal passage (line 5475 onward) refuses a `git worktree remove`
of a slot whose record proves a whole worktree
"whatever options the command carries:
every spelling Git accepts is read for the worktree it names".
The guard's own passage (lines 24163 to 24198) reads one plain shell-alias form,
`!git <words>`,
whose bytes hold no quote,
backslash,
`$`,
backtick,
glob,
brace or separator,
and whose words name no global option of their own;
"`git by the word Git will run` is the boundary of this guard".
Every other shell alias,
`!sh -c 'git worktree remove -f -f one'` and
`!git -C <directory> worktree remove` included,
"runs as Git runs it:
through real Git,
with no guard and no index-writer coordination".

The release now applies that same boundary to the slot refusals,
by the same reading,
so the two rules cannot disagree about what an alias is.
The release pass measured that the unreadable form still removes a bound slot:
`git -c "alias.zz=!sh -c 'git worktree remove -f -f one'" zz` deleted the slot
folder and its registration in alpha.47 and alpha.48.
That is a deliberate spelling of an alias the wrapper cannot follow,
and it is outside every passage the project has recorded so far.

## Options

### A: refuse every shell alias in a repository that holds slot records

A `git worktree remove`,
`git worktree move` or `git worktree prune` is not
needed to reach this arm:
every command reached through a `!` alias whose value the wrapper cannot read
stops in a repository whose record folder holds a record.

Pros:
no shell alias can delete or rename a bound slot's registration or prune it,
including a quoted value,
`!sh -c`,
a shell builtin and a script file;
the guarantee is simple to state and to test.

Cons:
it refuses aliases that never touch a worktree,
such as `alias.st=!git status` or `alias.l=!git log --oneline`,
in every repository the ZFS route has served;
the branch is a large change in what people may run,
and it is still defeated by a person who runs the removal without an alias at
all,
which the wrapper reads.

### B: refuse a shell alias whose value text names one of the three slot commands

The wrapper reads the raw alias value and refuses only when it contains
`worktree` and one of `remove`,
`move` or `prune`,
whatever the quoting around them.

Pros:
it catches the obvious deliberate spellings without refusing unrelated aliases;
it is a smaller change than A for the same repositories.

Cons:
a text scan is neither sound nor complete:
`eval "$CMD"`,
a variable,
a script file or a helper program hides the command,
while `!git log --grep 'worktree remove'` is refused though it removes
nothing;
the answer would read as a guarantee the wrapper cannot keep.

### C: leave every shell alias the wrapper cannot follow to Git

This is today's behaviour and the guard's recorded rule.

Pros:
it is the one rule the project has already written down for the guard
(`SPEC.md`,
"git by the word Git will run",
and `README.md`'s "A shell alias" passage),
so the slot refusals and the guard agree;
no unrelated alias stops working;
the wrapper does not guess at shell text it cannot read.

Cons:
a person or an agent who writes
`!sh -c 'git worktree remove -f -f <slot>'`
can still delete a bound slot's registration,
and the removal's own refusal never runs.

## Ranking

C > B > A.

C over B:
C keeps the guard's recorded boundary and refuses nothing it cannot read,
while B trades a real workflow for a guess that `eval` or a script file defeats,
and B refuses aliases that merely mention the words.

B over A:
B refuses only aliases whose text names a worktree command,
while A refuses every shell alias in a repository that holds a record,
including `!git status`,
for a protection that is still defeated by the same `eval`.

A note against reading the ranking as indifference:
the owner's four answers of 2026-11-26 refused every slot command the wrapper
can read
(`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`),
so A would be the consistent choice if the cost to unrelated aliases is
accepted.

## What would settle it

An owner's answer naming one option,
recorded as a decision that says what a repository holding slot records does
with a shell alias the wrapper cannot read.
For A or B,
the release also needs the words a refusal prints and the controls that prove
the alias never reaches Git.
For C,
`SPEC.md`'s removal passage gains the sentence the guard already carries,
so a reader learns that the boundary stops at the forms the wrapper can read.

## Records used

- `package/git-policy/cli/SPEC.md`,
  "Removal of a ZFS worktree clone" and the guard's passage at lines 24163 to
  24198.
- `package/git-policy/cli/README.md`,
  the guard's "A shell alias" passage.
- `doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`.
