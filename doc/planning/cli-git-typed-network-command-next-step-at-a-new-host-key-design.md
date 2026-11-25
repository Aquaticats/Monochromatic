# The next step a typed network command names at a new host key or a missing passphrase

## Status

A proposal,
put to the owner a fourth time.
The third revision of it led to the owner's answer of 2026-11-24T14:30:00-05:00,
recorded in `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`,
and the value that answer chose was retired on 2026-11-25
by the answer "Put `false` back (Recommended)" in
`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`,
because it ended connections that used to work instead of printing a line.
That is why the delivery question is open again,
and this revision puts it to the project with this round's evidence.
It is not a decision,
nothing of this design record is approved or final,
and this revision is not final either.
It designs the words a typed `git clone`,
`fetch`,
`pull` or non-LFS `push` can print when OpenSSH refuses a new host key,
or when a key needs a passphrase nobody can be asked for,
and it ranks the ways the wrapper could do that while keeping the owner's two answers "Never" for B and E and the
rule against starting itself.

Round folder:
`~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/`,
whose `keep/` holds `run-1-behaviour.log`, `run-2-costs-and-progress.log` and `run-3-steps.log` (every transcript and
timing this revision cites), `lib.sh` and `run-1-behaviour.sh`, `run-2-costs-and-progress.sh`, `run-3-steps.sh` (the
scripts), and `stand-ins/standin.c`, `stand-ins/proxy.c`, `sentences/host-key.txt`, `sentences/passphrase.txt` and
`sentences/j-value.txt` (the programs and the words those runs printed).
Worktree:
`.claude/worktrees/host-key-next-step-design-after-askpass-false`,
branch `docs/planning-host-key-next-step-design-after-askpass-false`,
made from `main` at `49029c6828`.
Nothing under `doc/decision/` or `doc/handover/` was changed by this round,
and no wrapper code was changed.
The round that puts `false` back runs at the same time,
on `fix/git-policy-cli-askpass-false-and-commit-push-pass-fixes`,
so `main` is described here as it will stand after that round.

The sections written by the earlier rounds are kept in place, and where one of them says "this round" it means the round
that wrote it.
Everything the fourth round measured names one of three logs,
`keep/run-1-behaviour.log`, `keep/run-2-costs-and-progress.log` or `keep/run-3-steps.log`,
and a figure that names one of those is this round's.

Every transcript in this document is labelled,
because a read of the second revision found that round printing text its run had not printed:

- **run** means a program printed these bytes, and the log that holds them is named with it;
- **composed** means no program printed them: the text was put together from other facts;
- **source** means read from a file, at the tag or the path named with it;
- **unrun** means neither.

## The question, open again: the value J put in `SSH_ASKPASS` ended connections that worked

J put one sentence into `SSH_ASKPASS`,
on the reasoning that a value OpenSSH cannot execute starts no program and prints the sentence inside OpenSSH's own
failure line.
What the code also does, read from OpenSSH `V_10_6_P2`:
`ssh_askpass` runs `execlp(askpass, askpass, msg)` and,
when that fails,
calls `fatal_f("exec(%s): %s", askpass, strerror(errno))`
(`readpass.c:77`, `:78`);
in the client `fatal_f` ends in `cleanup_exit`,
which calls `ssh_kill_proxy_command()`
(`clientloop.c:2886-2891`, `sshconnect.c:305-312`),
and the askpass child holds the parent's `proxy_command_pid`
(`sshconnect.c:288`),
so the child that could not start the sentence sends SIGHUP to the connection's own `ProxyCommand`.
OpenSSH starts every proxy as `$SHELL -c "exec <command>"` (`sshconnect.c:126`),
so the process killed is the proxy whatever the shell is.

So a value in `SSH_ASKPASS` is on the connection's critical path twice:
it answers the question,
and it must be a program OpenSSH can start,
or its failure reaches the transport itself.
`false` satisfies both: it starts, prints nothing and exits 1.

That is what the release pass found, and it is why the owner retired J.
The finding, quoted from
`~/temp/agent/release-pass-wrapper-alpha-46-2026-11-25/commit-push-lfs-lock-and-the-rest/keep/review.md`:

> #### H1. The askpass sentence makes ssh kill its own ProxyCommand, so a connection that worked now fails
>
> ... With a startable value such as `false` the child starts and exits 1, and no `fatal_f` runs. The sentence
> can never start, so every question asked on a connection that has a proxy kills the proxy. After an unknown host key
> the refusal that follows hides it. After a passphrase key that the server accepts but nobody can unlock, ssh should
> go on to the next key; the transport is already dead.

The owner's answer, recorded in `doc/decision/cli-git-alpha-46-release-pass-four-answers.md`:
"Put `false` back (Recommended)".
That record states the failure and what the answer leaves:
"the sentence of option J ... cannot be started,
and OpenSSH's failure path for an askpass that cannot start kills the connection's `ProxyCommand`,
so behind a `ProxyCommand` or `ProxyJump`, when the first key the server accepts has a passphrase no agent holds and a
later key works, the command ends 255 where alpha.45's `false` ended 0",
and
"The next step at a new host key then needs another delivery,
which brings back the options the owner did not pick for L4
(C, the wrapper reads Git's error output; C2, a paragraph after every exit 128; D, a `known_hosts` check before Git runs);
that choice goes to the owner separately, with the proposal".

So the question is the one this document was written to rank,
with three answers fixed:
B and E stay "Never" (`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
and no option may hand OpenSSH a value it cannot start,
so `SSH_ASKPASS` stays `false` and `SSH_ASKPASS_REQUIRE` stays `force` on `main` after the askpass round.

This round ran the case H1 names in a capped container with a real `sshd` and OpenSSH 10.6p2,
and ran every remaining option against it.
Run (`keep/run-1-behaviour.log`, families `f-noask-proxycommand` and `f-noask-proxyjump`):
one served repository,
the first key offered with a passphrase nobody can give and a later key that works,
and the connection through a relay `ProxyCommand` or through `-J`:

- **A, and today's `main` after the askpass round** (`SSH_ASKPASS=false`): exit 0, the clone complete, nothing extra
  printed;
- **J's sentence**: exit 128 through Git (255 through `ssh`), the line
  `ssh_askpass: exec(monochromatic-git-policy-cli: …): File name too long`, then
  `fatal: Could not read from remote repository.`;
- **the same sentence, with a relay that ignores SIGHUP**: exit 0 and the clone complete, which names the signal as the
  cause;
- **C, C2, D and K**: exit 0, the clone complete, and nothing extra, because none of their conditions became true.

Those results are the reason J is listed below with the options that were shown and not kept.

One thing this host adds, and every proxy arm below says so:
the `git` profile attaches to `/usr/bin/git` and stacks `&git//ssh` for `/usr/bin/ssh`
(`/etc/apparmor.d/git`, read),
and that stack refuses the shell a `ProxyCommand` needs:
the two `f-confined-proxycommand` arms fail with `/bin/sh: Permission denied`, whether `SSH_ASKPASS` holds `false` or
the sentence, while the two direct arms end 0 (run, `keep/run-1-behaviour.log`).
So no proxy arm can run under this image's own `ssh` here,
and every other arm uses a copy of `ssh` at a path no attachment names,
which runs under the `git` profile alone and may execute anything under `/work`
(`/etc/apparmor.d/local/git`, read: `owner /work/** mix`).
On this host the H1 failure cannot be produced, and the H1 arms describe a host whose `ssh` carries no such stack.

## What became of the answer J

The owner answered on 2026-11-24T14:30:00-05:00,
recorded in `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`:
"J: the sentence in SSH_ASKPASS (Recommended)",
and each of B and E was answered "Never (Recommended)".

Built in the round whose folder is
`~/temp/agent/build-host-key-next-step-option-j-2026-11-24/`,
worktree `.claude/worktrees/build-host-key-next-step-option-j`,
branch `feat/git-policy-cli-host-key-next-step-in-askpass`,
and specified in `package/git-policy/cli/SPEC.md`
under "No command the wrapper runs or forwards asks a person for a credential".

What the build changed:

- `ssh_no_answer_overlay` in `src/native/invocation/child_environment.rs` writes `NO_ANSWER_SENTENCE` into
  `SSH_ASKPASS` instead of `false`, and `SSH_ASKPASS_REQUIRE=force` still sends every question there.
  The sentence is 571 bytes, so `execlp` fails with `File name too long`,
  and the line a person reads is
  `ssh_askpass: exec(monochromatic-git-policy-cli: …): File name too long`.
- The read's finding 2 is met by the wording and not by a detector change:
  the sentence names no host key and no certificate outside single quotes,
  so `reads_as_unknown_host_key` still answers true on a transcript that holds its line.
- The read's finding 3 is named where a person reads it:
  the sentence's last clause says that the question alone never says whether another key answered it,
  and `SPEC.md` and `README.md` name the two cases where it prints on a command that succeeds.

Controls, red on `main` and green with the change, in a narrowed run of the gate image
(`keep/gate-image-red-run.log`, `keep/gate-image-green-run.log`):

- a typed `git clone` and a typed `git fetch` at a new host key print the sentence, then
  `Host key verification failed.`, and end 128;
- the same two at a key whose passphrase nobody can give print it, then `Permission denied (publickey).`, and end 128;
- a key whose passphrase nobody can give, skipped for one that works, prints it and the clone ends 0;
- on `main` the same five controls find no such line at all, because `false` starts and answers nothing;
- the upload's explanation and the service's push record still print the wrapper's own sentence for an unknown host key
  on a transcript that holds the rule's own line.

Not built, and not this answer's to build:
the three per-question sentences under "The words" belong to B and to C,
and a typed LFS push and the service's push record print the rule's sentence and then the wrapper's own sentence for
one unknown host key,
because both read the same transcript.

What became of it, in order:

- the release pass of 2026-11-25 found H1 (quoted under "The question, open again"):
  behind a `ProxyCommand` or a `ProxyJump`, a connection whose first accepted key had a passphrase no agent held, and
a later key that worked, ended 255 through `ssh` (128 through Git) where `false` ended 0;
- the owner answered "Put `false` back (Recommended)";
- the round on `fix/git-policy-cli-askpass-false-and-commit-push-pass-fixes` removed it:
  `NO_ANSWER_PROGRAM = "false"` in `src/native/invocation/child_environment.rs`,
  the tests that read the sentence gone with it,
  and a new control that runs the connection through a `ProxyCommand`
  (`a_key_nobody_can_unlock_is_skipped_for_one_that_works_behind_a_proxy_command`);
  `SPEC.md` and `README.md` now say that the release names no next step at a new host key.

The lesson this design keeps:
a value in `SSH_ASKPASS` is on the connection's critical path,
so no option here puts anything there but a program that starts and answers nothing,
and every option's control set carries an arm with a `ProxyCommand` and an arm with `ProxyJump`.
That arm is the control the gates did not have:
the build's own controls proved the sentence on direct connections only,
and the gate passed because of that (source: the review's H1, "The control that should have caught it does not exist").

## What this revision changed

This fourth round's changes, and what each one is for.

1. **J is retired, and its section says why.**
   H1 is quoted whole under "The question, open again" and in J's own section,
   and J sits with the options shown and not kept.
2. **Every remaining option is tried at the case that broke J.**
   A `ProxyCommand`, a `ProxyJump`, a first key whose passphrase no agent holds and a later key that works:
   the case, every arm's exit status and every line a person reads, under "The case every option is tried against".
3. **A new option, K, removes the con that decided the earlier ranking.**
   C lost the live progress display because Git draws it only when standard error is a terminal;
   K gives the child a pseudo-terminal on standard error and relays it,
   which keeps the display and the security-key touch notice.
   Measured: the display's lines come back through K and not through C.
4. **C2's two shapes are separated and both run.**
   With the exit status alone, and with D's known-hosts check after it;
   each shape speaks wrongly at one of the two refusals, measured on the same arms.
5. **Costs are measured twice, so a cost can be told from this harness's noise.**
   On the typed path against a same-binary control, where C, K, C2 and D all fall inside the noise;
   and as the steps themselves (one fork and wait, one pipe relay, one pseudo-terminal relay, `ssh -G`, `ssh-keygen -F`),
   where each one clears it.
6. **One ranking replaces the earlier two.**
   B and E are answered "Never", so the second ranking has no readers;
   the earlier one is kept only as the record quoted under "The ranking".
7. **What the person sees today is restated for `main` with `false`.**
   The transcripts are those of the earlier round's run, which used `false`,
   and the registry of what each option adds is this round's.
8. **The design no longer plans text inside `SSH_ASKPASS`.**
   `NO_ANSWER_SENTENCE` is gone from `main`;
   the words live in the option that owns a child.
9. **`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING` is described as it stands.**
   The constant remains, because the push record and the upload still print the wrapper's own host-key sentence;
   the sentence that used to be in `SSH_ASKPASS` is no longer one of its readers.
10. **The words of the earlier revision are kept, with their status named.**
    They were designed for B and for C and were not built;
    they are what an option that owns a child prints, and K and C would print them.
11. **The container arms say what this host's policy does to them.**
    The stack on `/usr/bin/ssh` refuses a `ProxyCommand`'s shell, so the arms use a copy of `ssh` and say so,
    and the failure this host would meet with any option is named where a reader meets it.
12. **The probe list is this round's.**
    What was run, what was not, and which claims are source only.

The third revision's own list of corrections is kept below, because each item is evidence about how this document was
corrected and about what a reader should check for in the next round.

### What the third revision changed, kept for its record

The read's findings, and what each one changed.
Where a finding did not hold,
the evidence against it is named here and in the section it belongs to.

1. **B's transcript was not the run's bytes.**
   This round re-ran every B arm with the kept stand-in,
   printed the bytes,
   and records the stand-in's size and digest beside them
   (run, `keep/run-1.log`, arms `b-newkey` and the rest, stand-in 1513 bytes).
   The second round's text is gone from this document.
   The arm it mislabelled is fixed:
   this round's `b-changed-key` meets no question and prints OpenSSH's banner alone
   (run, `keep/run-1.log`).
2. **The owner's self-start rule was not quoted, and B and E meet it.**
   The rule is quoted whole under "What the recorded rules leave open",
   and the ranking is given twice:
   once with the rule read strictly, where B and E leave the field,
   and once with B's exception granted.
   Whether the exception is granted is the owner's question, and this document does not answer it.
3. **The rotation was not a rotation and the noise test was empty.**
   The timings were re-run in a balanced Latin square of order 12:
   every arm follows every other arm twice, and sits at every position twice
   (run, `keep/run-2.log`, `07-timings.sh`).
   The test is now stated and applied to every arm:
   a difference counts as a finding only when its sign holds in every round and its median clears the
   difference between two runs of the same command in the same rounds.
   The figures are new, and the second round's table is superseded.
4. **The `-G` start count was wrong for the baseline.**
   This round counted the starts of each shape:
   a named `ssh` is started once,
   a differently named program twice,
   and E's stand-in twice, with two ssh starts under it
   (run, `keep/run-1.log`).
   E's arm is relabelled and split:
   most of its measured cost is the probe Git runs for a name it does not recognise,
   and only about a millisecond is E's relay.
5. **C2's arm did not show what the text said.**
   The section now says the status-only stand-in ran,
   that it prints whichever sentence file it is given,
   and that no arm exercised a post-failure check at all
   (run, `keep/run-1.log`, arm `c2-notfound-known-host-status-only`).
6. **The coverage sentence in the B over C pair was wrong in both directions.**
   B does cover the differing key, with its own sentence and this round's arm,
   and C also covers `BatchMode=yes` and `StrictHostKeyChecking=yes`,
   which is in the detector's source.
7. **D has a false positive the second round did not find.**
   Reproduced:
   with `KnownHostsCommand` supplying the key, the real clone exits 0 while D's stand-in says the machine
   does not know the key
   (run, `keep/run-1.log`).
   The read's guess about `@cert-authority` does **not** hold:
   `ssh-keygen -F` finds such a line, D's stand-in stays silent, and the real clone succeeds
   (run, `keep/run-2.log`).
   Both facts are in D's section.
8. **J's example named a host J cannot know.**
   J's value is rebuilt with the placeholder a shipped J must use,
   and its transcript is this round's run of that value
   (run, `keep/run-3.log`, 369-byte value).
9. **The selector's supporting claims were partly wrong.**
   The file-name fallback is dropped,
   the "bare word" claim is dropped with its source reason,
   and the stale-path branch now says what a probe shows:
   `readlink` of a deleted running binary gives `<path> (deleted)`,
   so ssh cannot start it and the person sees ssh's exec line.
10. **"Nothing else is needed" for AppArmor was inference.**
    It now reads "nothing else is known to be needed",
    and the reads the mode makes under the stack are named as inference, not probe.
11. **The "byte for byte" claim was not measured.**
    This round ran the comparison:
    the wrapper's and plain Git's standard output, standard error and exit status are identical for a new host key,
    for a passphrase key, and for a `pull` that ends 1
    (run, `keep/run-1.log` and `keep/run-2.log`).
12. **Stale line numbers.** Every cite is now a heading, a symbol, or a path with the sentence quoted;
    line numbers appear only where the file's own tag pins them
    (the OpenSSH and Git sources at the tags named in "References").
13. **A misquoted comma.** The old handover's sentence is
    `username or password.", so no command ...`, with a comma and one period;
    the second round printed `password.". so`.
14. **Printed commands were unquoted, and a key path can be cut.**
    The sentences no longer build a copyable command out of a host or a path;
    the reason (the shell grammar of the printed text, and OpenSSH's 100-character cut of the key path)
    is in "The words".
15. **The panic hook ran after the mode would sit, and "flush" was the wrong mechanism.**
    The mode's place now comes after the hook,
    and the wait that keeps a last log line is named instead of a flush.
16. **A typed LFS push would say the same thing twice.**
    Named in B's section and in "What building B would take",
    together with the fact that the host-key detector already skips the wrapper's own sentence.
17. **A container outlived the second round's report.**
    `typed-network-next-step-probe-2026-11-24` was still listed by `podman ps --all`, exited,
    while that report said its list was empty.
    It is not this round's,
    and this round left it,
    as it left `quizzical_leavitt`.

The read's own two "does the ranking stand" verdicts are answered in "The ranking":
the pair reasons are rewritten,
the progress loss is counted in the pair that needs it,
and the choice between plain words and no everyday cost is named as the owner's.

## What the person sees today

These are the lines a person meets on `main` as it stands after the askpass round,
because `SSH_ASKPASS` holds `false` again (`NO_ANSWER_PROGRAM` in
`src/native/invocation/child_environment.rs`),
so the wrapper adds no line of its own at either refusal.
Nothing changed in these transcripts when J was built and then removed:
they were measured with `false`, which is the value `main` carries again
(run, the third revision's `keep/run-1.log` and `keep/run-2.log`; this round re-measured the same shapes with `false`
in `keep/run-1-behaviour.log`, families `f-newkey-direct` and `f-locked-direct`).

Measured in a rootless container of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85`, `OpenSSH_10.6p2`, `git 2.56.1`)
against a throwaway `sshd` on loopback,
with a fixture host key,
a fresh `known_hosts` and throwaway client keys.
The wrapper is the third revision round's release build (`0.6.1-alpha.44`, 27971352 bytes).
The wrapper and Git are copies under `/work`,
which no AppArmor profile attaches to,
so the arms show OpenSSH's own behaviour and not this host's confinement.

A new host key,
for `git clone`,
`git fetch`,
`git push` and `git ls-remote`,
ends this way
(run, `keep/run-1.log`, the byte comparison):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

`git pull` in a repository that exists prints the same lines and ends 1, not 128
(run, `keep/run-2.log`).
`git ls-remote` prints the same two lines without `Cloning into`.

The comparison the second round claimed and had not run:

```text
##### new host key, clone
exit: plain git=128 wrapper=128
stderr bytes: plain=187 wrapper=187
stdout: identical
stderr: identical
##### a passphrase key nobody can unlock, clone
exit: plain git=128 wrapper=128
stderr bytes: plain=204 wrapper=204
stdout: identical
stderr: identical
##### a pull that meets a new host key
plain git pull exit=1
wrapper pull exit=1
pull stdout and stderr: identical
```

That is run, in the third revision's round (`keep/run-1.log`, `keep/run-2.log`; script `02-byte-compare.sh` and
`06-corrections.sh`).

A changed host key for a host this machine already knows ends 128 with OpenSSH's own banner,
`Offending ED25519 key in <file>:<line>`,
`Host key for <host> has changed and you have requested strict checking.`
and the same two Git lines.
This round's arm meets no question at all, so the stand-in played no part in it
(run, `keep/run-1.log`, arm `b-changed-key`).

A key whose passphrase nobody can be asked for ends this way
(run, the byte comparison):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone'...
root@127.0.0.1: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

Why the wrapper cannot say anything today,
read from the source:

- a typed command is handed over by replacing the wrapper's process image,
  so nothing of the wrapper is left to read anything
  (`replace_process_with_real_git` in
  `package/git-policy/cli/src/native/invocation/forwarding.rs`):
  same process number,
  same streams,
  same signals,
  the terminal handed over.
- the rule that forbids the question is one list,
  `no_prompt_overlay`
  (`src/native/invocation/child_environment.rs`):
  `GIT_ASKPASS` empty,
  `GIT_TERMINAL_PROMPT=0`,
  `SSH_ASKPASS=false`,
  `SSH_ASKPASS_REQUIRE=force`.
- the wrapper already has two sentences of this kind,
  and they reach only the two places where the wrapper owns the child:
  the LFS upload's explanation
  (`explanation_after_failure` in `src/native/lfs/lfs_upload_explanations.rs`)
  and the service's push failure record
  (`retried_failure_record` in `src/native/push/push_retry_failure.rs`),
  both printing `host_key_not_known_sentence`
  and `login_needed_message`
  from `src/native/lfs/lfs_pre_push.rs`.
  Both sentences are about a push ("then push again"),
  which is why a typed `fetch` or `clone` cannot reuse them word for word.

The owner's question for this waits in
`doc/handover/questions-for-the-owner.md`,
section "Words for a typed command at an unknown host key or a missing passphrase (the pass's L4)".

## The case every option is tried against

Every option in this document was run at the case H1 names,
and at the three other refusals a next step is meant for,
in one capped container with a real `sshd` and the OpenSSH this image carries.
Run (`keep/run-1-behaviour.log`, script `keep/run-1-behaviour.sh`, stand-ins `keep/stand-ins/standin.c` and
`keep/stand-ins/proxy.c`).

Container: `localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`, `--rm`, `--network=none`,
`--cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`, started in a `systemd-run --user` scope with
`RuntimeMaxSec=1800`, `CPUQuota=300%`, `MemoryMax=6G`, `TasksMax=1024` and `IOWeight=10`, named
`host-key-next-step-options-2026-11-25`.
OpenSSH 10.6p2, Git 2.56.1, `sshd` on port 2222 with a fixture host key.

### The setup a person would have

- two client keys in `authorized_keys`: `key_locked`, whose passphrase is nobody's to give, listed first, and
  `key_good`, which works;
- a served repository at `ssh://target/work/served.git`, where `target` resolves to `localhost:2222` and offers the two
  keys in that order with `IdentitiesOnly yes`;
- a relay `ProxyCommand` (a compiled program, `keep/stand-ins/proxy.c`), and a `ProxyJump` through `root@127.0.0.1:2222`;
  the jump host is `127.0.0.1` and the destination `localhost`, because `ssh` refuses a jump whose host, port and user
  equal the destination's (`ssh.c:1289-1291` at `V_10_6_P2`, read);
- the wrapper's two variables as `main` holds them after the askpass round:
  `SSH_ASKPASS=false` and `SSH_ASKPASS_REQUIRE=force`;
- an `ssh` at `/work/bin/ssh`, a copy of the image's own, because the stack this host attaches to `/usr/bin/ssh` refuses
  the shell a `ProxyCommand` needs (the `f-confined` arms and the section "The question, open again" show it).

### The four families

The families differ only in what a person's settings and the server make true.

- `f-noask`: the host key is known and no host-key question is asked, so the questions are the passphrase ones.
  **This is the case that broke J.**
- `f-newkey`: the host key is unknown and the person's `StrictHostKeyChecking` is `ask`, so the host-key question is
  asked, with a direct connection and behind both proxies.
- `f-known`: the host key is known and the repository is not there: a failure that is not about a key at all.
- `f-locked`: the host key is known and only the locked key is offered: a key nobody can unlock.

Each family ran the same eight arms:
`a-wrappers-value` (`SSH_ASKPASS=false` and nothing else, which is `main`),
`j-sentence` (the retired value, kept as the positive control),
`c-relay`, `c-pty`, `c2-status`, `c2-check-unknown`, `c2-check-known` and `d-precheck`.
The first two families ran each arm three times: directly, behind the `ProxyCommand`, and behind the `ProxyJump`.

### What each option gave, family by family

Run (`keep/run-1-behaviour.log`; the exit statuses and the lines are that log's).

At `f-noask`, which is H1's case:

- `a-wrappers-value`: exit 0 in all three transports, and nothing extra printed;
- `j-sentence`: exit 0 direct, 128 behind the `ProxyCommand`, 128 behind the `ProxyJump`, with the line
  `ssh_askpass: exec(monochromatic-git-policy-cli: …): File name too long`;
- `j-sentence` with a relay that ignores SIGHUP: exit 0, the clone complete, which names the signal as the cause;
- `c-relay`, `c-pty`, `c2-status`, `c2-check-unknown`, `c2-check-known`, `d-precheck`: exit 0 in all three transports, and
  nothing extra printed, because no refusal reached them.

At `f-newkey`, directly and behind both proxies:

- `a-wrappers-value`: exit 128 and OpenSSH's own lines, and no next step;
- `j-sentence`: exit 128, the exec line, then `Host key verification failed.`;
- `c-relay` and `c-pty`: exit 128, OpenSSH's and Git's lines, then the host-key sentence;
- `c2-status` and `c2-check-unknown`: exit 128, then the host-key sentence;
- `c2-check-known`: exit 128 and nothing, because the check found no key and this shape speaks only when it finds one;
- `d-precheck`: exit 128, the host-key sentence **before** `Cloning into`, then OpenSSH's and Git's lines.

At `f-known` (a repository that is not there, host key known):

- `a-wrappers-value`: exit 128, Git's two `fatal:` lines, nothing about keys;
- `c-relay` and `c-pty`: the same, and silent, which is C being right;
- `c2-status`: exit 128, Git's lines, then the **host-key** sentence, which is wrong;
- `c2-check-unknown`: silent, which is right here;
- `c2-check-known`: exit 128, Git's lines, then the **passphrase** sentence, which is wrong;
- `d-precheck`: silent, because the key is known.

At `f-locked` (the passphrase refusal, host key known):

- `a-wrappers-value`: exit 128, `Permission denied (publickey).`, nothing else;
- `c-relay` and `c-pty`: exit 128, then the passphrase sentence, which is right;
  the transcript is
  `root@localhost: Permission denied (publickey).`, Git's two `fatal:` lines, then
  `monochromatic-git-policy-cli: this command needed the passphrase of the key `/work/key_locked` …`
  (run, arm `f-locked-direct-c-relay`);
- `c2-status`: exit 128, then the **host-key** sentence, which is wrong;
- `c2-check-unknown`: silent, so the refusal the person needs words for gets none;
- `c2-check-known`: exit 128, then the passphrase sentence, which is right here;
- `d-precheck`: silent, because the key is known and D never sees a passphrase.

### What the arms are not

- They are stand-ins, not the wrapper.
  `keep/stand-ins/standin.c` is one binary with a mode per option: it forks, relays on a pipe or a pseudo-terminal,
  waits, and prints a sentence file's text when its condition holds.
  It reads no repository, applies no policy and prints the words of "The words" unchanged,
  so the host and the key inside those sentences are the fixture's, `[127.0.0.1]:2222` and `/work/key_locked`,
  and not filled from the transcript as a shipped option would fill them.
  The arms show the delivery and the cost, not the wrapper's own reading.
- C2's two shapes are two arms of that binary: `forkwait-check-unknown-only` prints the host-key sentence only when no
  file `ssh -G` named holds a key for the host, and `forkwait-check-key-known` prints the passphrase sentence only when
  one does.
  Neither branch is what the earlier revision assumed: it said "a paragraph after every exit 128", and a check changes
  which paragraph, not whether one arrives.
- D's arm runs `ssh -G` and `ssh-keygen -F` for real, and prints only when `ssh -G` reports `stricthostkeychecking ask`
  or `accept-new` and no named file holds a key.
- No arm has `Match exec`, `CanonicalizeHostname`, `KnownHostsCommand`, `HostKeyAlias` or `VerifyHostKeyDNS` set.
  The third revision's runs measured `KnownHostsCommand` (which makes D speak while the clone succeeds) and
  `@cert-authority` (which does not); those results stand and are quoted in D's section.
- No arm ran with a signal sent to the wrapper alone, and none ran on Windows.

## The rule that forbids the question, and where it lives now

The rule lives in
`package/git-policy/cli/SPEC.md`,
in the paragraph beginning "No command the wrapper runs or forwards asks a person for a credential",
and in `package/git-policy/cli/README.md` in the paragraph that ends "because no command the wrapper runs or forwards
asks a person for a credential".
It reads, as `SPEC.md` has it:

```text
No command the wrapper runs or forwards asks a person for a credential
(owner decision,
2026-11-19T16:58:00-05:00:
"Avoid ever asking for someone to type a username or password.",
with an SSH key's passphrase counted as a credential:
"Passphrases count too (Recommended)";
recorded in `doc/handover/cli-git-rust-implementation.md`,
the entry beginning "owner answered H1 and M4").
```

The entry that record was sorted out of,
`git show c84e2f1dbf:doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md`,
records the owner's words and what the round after it did with them:

```text
**The owner answered H1 and M4** (this entry): H1 "Hide long words only" (path words of 20 or more unbroken lowercase
letters print as `<credentials>`); M4, in the owner's words, "Avoid ever asking for someone to type a
username or password.", so no command the wrapper runs or forwards prompts for a credential, terminal or not, superseding
`SPEC.md`'s "A command the person typed is forwarded with the prompting Git has" for credentials; asked separately, an
SSH key's passphrase counts too ("Passphrases count too (Recommended)": a key not in ssh-agent fails at once, saying to
run `ssh-add`).
```

Two facts about that rule matter to this design,
and the read is right about both:

- the SPEC's parenthetical still points at the handover entry,
  which no longer exists;
  the rule's own text is the record now.
- `SPEC.md` also says "a host key is not a credential the push lacked",
  so the credential rule does not itself forbid answering a host-key question.
  What forbids it is the owner's answer of 2026-11-20 that the refusal is explained rather than answered
  ("A host key this machine does not know is no stop: the service keeps its usual retry schedule for it,
  and its record says that it pushes the branch by itself once the key is known",
  "the message says the host key is unknown, how to fix it,
  and that the service will push by itself once the key is known"),
  and the round's own brief.
  Every option here refuses to answer yes;
  the ones that could answer yes are listed and not ranked.

## What OpenSSH does with an askpass program

Read from `V_10_6_P2`,
the release this machine runs (`ssh -V` prints `OpenSSH_10.6p2`),
and fetched this round from
`raw.githubusercontent.com/openssh/openssh-portable/V_10_6_P2/<file>`
into `scratch/sources/`.

`ssh_askpass` runs the program,
in `readpass.c` lines 46 to 107:

- the program's standard output is a pipe the program cannot see
  (`dup2(p[1], STDOUT_FILENO)`, `readpass.c:73`);
- one environment variable is added when the question is a permission question,
  `SSH_ASKPASS_PROMPT=confirm` (`readpass.c:75`, `:76`);
  a security-key notification gets `SSH_ASKPASS_PROMPT=none` (`readpass.c:279`);
- nothing else of the environment changes:
  the program is a `fork` without an environment clean-up,
  so it keeps every variable `ssh` itself was given,
  including the four the wrapper puts there;
- the program is found on `PATH` and receives the question as its only argument,
  and its `argv[0]` is that same string:
  `execlp(askpass, askpass, msg, (char *)NULL);` (`readpass.c:77`);
- a program that cannot be executed never starts,
  and OpenSSH prints its own line for that:
  `fatal_f("exec(%s): %s", askpass, strerror(errno))` (`readpass.c:78`);
  and that line is not the end of it.
  In the client `fatal_f` ends in `cleanup_exit`,
  which calls `ssh_kill_proxy_command()`
  (`clientloop.c:2886-2891`; `sshconnect.c:305-312`: `kill(proxy_command_pid, SIGHUP)`),
  and the askpass child holds the parent's `proxy_command_pid`
  (`sshconnect.c:288`),
  so a value OpenSSH cannot start sends SIGHUP to the connection's own `ProxyCommand`,
  which `ssh` started as `$SHELL -c "exec <command>"` (`sshconnect.c:126`).
  This is H1, and it is why no option here but a program's path may be the value.
  OpenSSH also sends SIGHUP to the proxy at its own normal exit
  (the same call, from the same path),
  so a proxy that records SIGHUP does not by itself say anything failed:
  the exit status of the connection is what does (run, `keep/run-1-behaviour.log`, where the arms that succeed behind a
  `ProxyCommand` record it too).
- the parent reads standard output in one buffer of about a thousand bytes
  (`readpass.c:82` to `:94`);
- a program that did not exit zero,
  or that was killed,
  gives no answer at all: the function returns `NULL`
  (`readpass.c:99` to `:102`);
- everything from the first carriage return or line feed is dropped,
  so only the first line of standard output can be the answer
  (`readpass.c:104`);
- standard error is not redirected anywhere,
  so it reaches the person as the program writes it;
- standard input is inherited from `ssh`.
  Under Git that stream is the protocol pipe:
  a program that reads it does not merely steal a byte,
  it waits for a byte that cannot arrive until the connection proceeds,
  and the connection stops.
  Measured in the second round: an askpass that reads one byte hung the clone until a 20 s bound ended it
  (run, `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
  arm `b-askpass-reads-stdin`, exit 124).

`read_passphrase` decides whether the program runs at all,
in `readpass.c` lines 119 to 194:

- `SSH_ASKPASS_REQUIRE=force` sets both `use_askpass` and `allow_askpass`
  (`readpass.c:130` to `:133`);
- `prefer`, `never` and an absent value leave the terminal in play
  (`readpass.c:134` to `:138`);
- a forced question with no answer from the program returns the empty string
  (`readpass.c:179` to `:182`),
  and the same happens to a question that must use the program while `never` forbids it
  (`readpass.c:169`, `:170`);
- otherwise the question goes to `/dev/tty` (`readpass.c:185`).

Two answer paths read an empty answer differently,
and the difference decides which exit statuses an askpass program must never give:

- `ask_permission` (`readpass.c:197` to `:220`) accepts the empty string and the word `yes` as affirmative.
  That covers the multiplexing question,
  the key-confirmation question and the forwarding questions.
  A program that prints nothing and exits zero therefore agrees to those.
- `confirm` (`sshconnect.c` lines 601 to 627) reads a `NULL` answer as no,
  and also reads the empty string as no
  (`if (p[0] == '\0' || strcasecmp(p, "no") == 0) ret = 0;`).
  So the host-key question is safe either way;
  the hazard belongs to `ask_permission` alone.
  Measured both ways in the second round (`keep/run6.log`):
  a master started with an askpass that exits zero and prints nothing allowed a second session through it,
  and the same master with an askpass that exits two refused it
  (`Master refused session request: Permission denied`).

The host key question is `confirm`,
`sshconnect.c` lines 601 to 627:
it loops on `read_passphrase(msg, RP_ECHO)`,
returns zero when that gives nothing
(`sshconnect.c:613`, `:614`),
and accepts only the empty string,
`no`,
`yes` or the fingerprint it was given as answers
(`sshconnect.c:615` to `:621`).
A refused question ends the connection through
`fatal("Host key verification failed.")`
(`sshconnect2.c:104`).
For a host this machine does not know,
`check_host_key` builds a question that carries the host,
the address in parentheses,
the key type,
the fingerprint and,
for other names that hold this key,
a sentence about that,
and ends with
`Are you sure you want to continue connecting (yes/no/[fingerprint])? `
(source, `sshconnect.c` lines 1207 to 1246).

Every askpass prompt in the three files this round read holds a space,
which matters to B's selector:

- the host-key question above (`sshconnect.c:1211` to `:1246`);
- `Add key %s (%s) to agent?` (`sshconnect.c:1794`, through `ask_permission`);
- the password prompts (`sshconnect2.c:1057`, `:1115`, `:1125`, `:1134`);
- `Enter passphrase for key '%.100s': ` (`sshconnect2.c:1547`);
- the `ask_permission` prompts of a shared connection
  (`Allow shared connection to %s? ` and its siblings, `mux.c:437`, `:548`, `:833`, `:1055`, `:1057`, `:1145`, `:1177`);
- a keyboard-interactive prompt,
  which `sshconnect2.c:2125` to `:2130` builds as `(%s@%s) %s` around whatever the server sent.
  So even a server prompt that is one bare word arrives with the account and the host in front of it.

Two facts about the other host-key states,
which the first round did not have:

- a **changed** host key never asks.
  With strict checking `ask` the code prints the warning banner and
  `Host key for <host> has changed and you have requested strict checking.`
  and fails (`sshconnect.c:1338` to `:1366`).
  Measured: no askpass start, exit 128
  (run, this round, `keep/run-1.log`, arm `b-changed-key`, which prints the banner and no sentence).
- a **differing key for the address** (`CheckHostIP=yes`) does ask,
  and is the only differing-key question there is.
  Its text carries the offending file and line,
  measured from a non-loopback name, because loopback and a proxy command switch `CheckHostIP` off
  (`sshconnect.c:1035` to `:1045`; run, the second round, `keep/run5.log`, arm `q-differing-key`).
  `CheckHostIP` defaults to off (`readconf.c:2909`, `:2910`),
  so this question is rare and must not shape the design.

A key's passphrase is `load_identity_file`,
`sshconnect2.c` lines 1533 to 1575:
the question is `Enter passphrase for key '<path>': `,
and the path is cut at 100 characters by the `%.100s` in the `snprintf` (`sshconnect2.c:1547`).
An empty answer is read as one nobody gave,
so the key is skipped and the next one is tried
(`sshconnect2.c:1553` to `:1558`).
That is why a sentence printed at this question cannot claim the connection failed,
and why it must not offer a copyable `ssh-add <path>` for a path that may have been cut:
see "The words".

Two settings remove the question instead of answering it,
and both were probed:

- `BatchMode=yes` returns zero from `confirm` before anything is asked
  (`sshconnect.c:609`, `:610`):
  measured, no askpass start and no words
  (run, this round, `keep/run-1.log`, arm `b-batchmode`).
- `StrictHostKeyChecking=yes` fails with OpenSSH's own sentence
  `No ED25519 host key is known for [127.0.0.1]:2222 and you have requested strict checking.`
  (`sshconnect.c:1198` to `:1206`):
  measured, no askpass start
  (run, this round, `keep/run-1.log`, arm `b-strict-yes`).

A security-key touch notice is written straight to standard error only when that stream is a terminal
(`notify_start`, `readpass.c:252` and `:253`);
otherwise it is sent to the askpass program with `SSH_ASKPASS_PROMPT=none`
(`readpass.c:279`).
An option that takes standard error away from OpenSSH therefore loses the touch notice,
and an option that keeps it prints the notice itself.
Source only: no security key was available to probe this.

Quiet mode suppresses OpenSSH's own lines,
and it decides four of the options here.
`-q` sets `options.log_level = SYSLOG_LEVEL_QUIET` (`ssh.c:927` to `:929`),
and `sshlogv` returns before anything is written when the level is above it
(`log.c:461`, `if (nlog_verbose == 0 && level > log_level) return;`).
`SYSLOG_LEVEL_QUIET` is the first enumerator (`log.h:41`), so even `fatal` is suppressed;
the `_f` variants only add the function name to the text (`log.h:120` to `:126`).
Measured with `-q` and with `LogLevel=QUIET`:
`Host key verification failed.` disappears,
Git's own `fatal:` line stays,
and a program's own write to standard error stays
(run, this round, `keep/run-1.log`, arms `b-quiet` and `b-loglevel-quiet`;
the second round's `c-quiet`, `c2-quiet`, `e-quiet`, `j-quiet`).

## What Git does around it

Read from Git 2.56.1 (`git --version`),
the version the wrapper shadows here,
fetched this round from `raw.githubusercontent.com/git/git/v2.56.1/<file>`.

`git_prompt` and `do_askpass`,
`prompt.c` lines 12 to 77:

- the program is chosen in this order and only a non-empty name counts:
  `GIT_ASKPASS`,
  then `core.askPass`,
  then `SSH_ASKPASS`
  (`prompt.c:53` to `:58`);
  an empty `GIT_ASKPASS` therefore stops the other two as well,
  which is what the wrapper relies on;
- the program receives the prompt as its only argument
  (`prompt.c:18`, `:19`) and its standard output is captured
  (`prompt.c:21`);
- `strbuf_read(&buffer, pass.out, 20)` passes a size **hint** for the first allocation,
  not a cap:
  the read runs to end of file (`strbuf_read`, `strbuf.c`),
  and the first line break is then cut off (`prompt.c:27`, `:41`).
  The first round read this as "at most 20 bytes" and was wrong.
- a program that does not exit zero makes Git print
  `error: unable to read askpass response from '<program>'`
  and take no answer
  (`prompt.c:35` to `:39`);
- with no answer,
  `GIT_TERMINAL_PROMPT` decides between a terminal prompt and
  `could not read <prompt>terminal prompts disabled`
  (`prompt.c:65` to `:73`).

Measured in the second round,
and unchanged by this one:
`GIT_ASKPASS=<program>` (a program that prints on standard error and exits non-zero) makes Git print that
sentence, then `error: unable to read askpass response from '<program>'`,
then `fatal: could not read Username for 'https://example.com': terminal prompts disabled`.
So pointing Git's own helper at a program adds one line naming that program on every failed credential question,
where today's empty value adds none.

How Git reaches the server,
`connect.c`:

- `GIT_SSH_COMMAND` wins,
  then `core.sshCommand`,
  then `GIT_SSH` (no shell),
  then `ssh`
  (`connect.c:1182` to `:1193`, `:1409` to `:1424`);
- a command Git is given may be shell text,
  and the remote-side command is single-quoted into it
  (`connect.c:1501` to `:1509`);
- Git recognises a command as OpenSSH only by the base name `ssh` or `ssh.exe`
  (`determine_ssh_variant`, `connect.c:1225` to `:1273`);
  for any other name it runs the command once more with `-G <host>` and reads that run's exit status alone,
  discarding its output (`connect.c:1426` to `:1439`).
  Counted this round with a counting program of each name:
  a program called `ssh` is started once,
  a program called `renamed-ssh` twice,
  and a program called something else twice
  (run, `keep/run-1.log`; script `03-start-counts.sh`).
  `GIT_SSH_VARIANT=ssh` or `ssh.variant=ssh` overrides that detection
  (`override_ssh_variant`, `connect.c:1206` to `:1224`),
  which is a note under E;
- protocol version 2 adds `-o SendEnv=GIT_PROTOCOL` and the variable itself
  (`connect.c:1342` to `:1348`);
- Git waits for the `ssh` child and,
  when it got no answer at all,
  prints
  `fatal: Could not read from remote repository.`
  with the two advice lines
  (`connect.c:61` to `:75`,
  `finish_connect`, `connect.c:1559` to `:1567`);
- that child's standard error is Git's own,
  which is why OpenSSH's line appears before Git's.

The exit status a person's shell sees,
measured this round and in the second round:

- a new host key: 128
  (run, this round);
- a key nobody could unlock: 128
  (run, this round);
- a connection refused: 128
  (run, the second round, `keep/run4.log`, arm `status-refused`);
- a host name that does not resolve: 128
  (run, the second round, arm `status-dns`);
- a repository that is not there, host key known: 128
  (run, the second round, `keep/run5.log`, arm `status-notfound-known-host`);
- `git pull` in a repository that exists: 1, with the same lines as 128 gives elsewhere
  (run, this round, `keep/run-2.log`).

That list is why C2's exit-status condition cannot tell the refusals apart.

## What the recorded rules leave open

The owner's self-start rule,
from `doc/planning/cli-git-no-self-invocation.md`,
quoted whole from its own section "The rule"
(lines 34 to 62 of this base):

```text
The owner's instruction,
quoted:
"We should never invoke ourselves recursively.
That's bad design.
Audit,
remove,
and
ban that design aggressively."
and "There are clean ways to have extensive explanatory error messages even with the
ban."
The owner then added:
"we don't have to use any shell!"

The rule as built:

- The wrapper never starts its own executable,
  whether as a child,
  as a Git hook command or `-c` argument,
  as a shim it writes,
  or through dispatch on the name it was started as (`argv[0]`).
- The wrapper never writes shell text:
  no `sh -c`,
  no script file,
  no shell function,
  no `printf` or `$?` in a hook.
- An explanation of a failure is printed by the parent process that owns the child,
  from facts the parent can read
  itself.
  Nothing that Git or a hook starts is asked to explain anything.
```

That third bullet is the one this design meets,
and it meets it in two different ways:

- **B.** Git starts ssh,
  ssh starts the wrapper from the path the overlay wrote into `SSH_ASKPASS`,
  and that start prints the explanation.
  The wrapper does not start itself,
  so the first two bullets are kept by their letter;
  but the process that explains was started by Git's descendant,
  which the third bullet forbids.
- **E.** Git starts the wrapper as its ssh,
  the wrapper starts the real ssh as its child,
  and the wrapper explains from the child's words.
  The explaining process was itself started by Git,
  which the third bullet forbids on the same reading,
  even though it is the parent of the ssh it explains about.

Two precedents in the same record bear on the reading and neither settles it:

- `remember_own_executable` (`src/native/push/service_route.rs`)
  writes the wrapper's own path into the unit that starts the service mode.
  The unit is started by systemd and explains nothing,
  so that case covers "names its own path" and not "explains from a Git descendant".
- the one-program record allows a later askpass *program* to reopen its answer,
  which is not this question.

Whether an askpass start, or a wrapper started as ssh, is "asked to explain" was the owner's call,
and the owner answered it on 2026-11-24,
recorded in `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`:
"Never (Recommended)" for B and "Never (Recommended)" for E.
That is the strict reading of the third bullet,
so no option here starts the wrapper from Git or from ssh,
and the two options that would are listed with the answers that retired them.
The rule therefore stands as written, and a later round cannot reopen this by finding better words.

The other recorded rules:

- "One program only,
  ever"
  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`):
  no second shipped executable,
  so a separate askpass program of the wrapper's own is out.
  The record adds:
  "A later wish for another shipped program, such as an askpass helper, meets this answer first."
  B keeps one program: it starts the wrapper that already exists.
- the wrapper reads the name it was started under for the ZED hook and nothing else
  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`),
  and `SPEC.md` says "the executable does not read the name it was started under",
  so an askpass mode cannot be selected by a link name.
  The ZED record's reason (ZED cannot add an environment variable) does not carry over:
  the overlay can add one, and the record gives no other reason against it.
  An environment value as a selector is therefore open,
  and B is the option that needs one.
- no command the wrapper runs or forwards asks a person for a credential,
  so no option may make OpenSSH's question reach a terminal.
- no option may answer "yes" on a person's behalf.
  That is this round's brief.
  The project's recorded answers do not settle it by themselves:
  `SPEC.md` says a host key is not a credential the push lacked,
  and the owner's answer of 2026-11-20 is that the service keeps retrying and pushes once the key is known,
  which only has a point if the machine never makes itself know the key.
  The rule stands, and it rules out `StrictHostKeyChecking=accept-new`,
  a pre-added key,
  and an askpass program that prints `yes`.
- the wrapper never starts itself as Git
  (`SPEC.md`, section "The wrapper never starts itself as Git"),
  and its source guards refuse `current_exe`,
  `/proc/self/exe`,
  `argv[0]` dispatch,
  shell text and a start from its own path
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`,
  rules `own-executable`, `argv0-dispatch`, `shell-text`, `spawn-own-executable`).
  B and E need a start of the wrapper by another program;
  nothing in them starts the wrapper as Git,
  and `own-executable`'s existing exception in `invocation/real_git.rs` already carries the wrapper's own path for
  identification
  ("identification only: this path is excluded from the search for the real Git so the wrapper never selects itself;
  nothing is started from it").
  B's selector reads that path for identification in the same sense;
  E needs no such reading but does need shell text,
  which is under "The options" as E's first cons.
- E cannot take over a person's `GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH` without either losing that
  person's options or writing shell text, which the `shell-text` rule forbids.
  `ssh_config(5)` reads command-line options first,
  then the user's file,
  then the system file,
  and "for each configuration directive, the first specified value will be used",
  so a `core.sshCommand` rewritten to force a setting would override the person.

One source fact narrows B's outstanding work,
and the second round did not have it:
the host-key detector already treats the wrapper's own sentence as the refusal's own line.
`reads_as_unknown_host_key` (`src/native/lfs/lfs_pre_push.rs`)
accepts a line that contains `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`
(`"this machine does not know the ssh host key"`),
and that constant was added by `232dc52a91`
("the host-key sentence is added for an unknown key alone, and once")
for a pre-push hook that echoes the sentence.
The login detector needs no such line:
none of its five words (`terminal prompts disabled`,
`could not read Username`,
`could not read Password`,
`too many authentication attempts`,
`Permission denied (`) appears in the three sentences this design adds.

## The options shown and not kept

This revision ranks five options and hands only one of them to the owner,
so no option here is built until the owner answers.
Each entry says what it starts,
what it reads,
whether it can answer yes,
what a person sees for each family of "The case every option is tried against",
what it costs on a command that meets no question,
and its pros and cons.
The ranking and its adjacent-pair reasons are in "The ranking".

B, E and J carry answers already given,
and each says which answer and what became of it.
B and E are out by the owner's answer of 2026-11-24,
so nothing in this revision ranks them;
J is out by the owner's answer of 2026-11-25,
and its own section keeps the finding that retired it.

Costs are given twice, because one measurement cannot carry them all.
On the typed path,
an arm is measured against a null arm run in the same round:
`git ls-remote` with the same ssh, the same identity options and a value that needs no question.
**In this round's typed-path run that measurement cannot resolve any of these options**:
the noise floor, two runs of the same command in the same round, is
median `+1.59 ms` with a range of `-8.23` to `+18.15 ms` on a command of about 146 ms
(run, `keep/run-2-costs-and-progress.log`),
and C, K and C2 land inside it.
So the work each option adds is measured as the steps it runs,
against `/usr/bin/true`, in the same container and the same rotation
(run, `keep/run-3-steps.log`),
and that measurement resolves all of them.
The third revision's typed-path figures are quoted beside the new ones where the two differ.

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

At H1's case A is what `main` does, and it is the only arm that ends 0 and prints nothing, because nothing is added to
the connection:
exit 0 direct, behind the `ProxyCommand` and behind the `ProxyJump`, with `SSH_ASKPASS=false`
(run, `keep/run-1-behaviour.log`, family `f-noask`).
That is also why the owner's answer of 2026-11-25 is safe: it restores exactly this.

A person sees the transcripts in "What the person sees today": OpenSSH's line, Git's fatal line, exit 128,
and no next step.
The one way out is knowledge the words do not give: connect once with plain `ssh <host>`,
which the wrapper does not touch,
or add the key to `known_hosts`.

Cost: nothing, and no null arm can measure nothing.

Its documentation-only complement:
`package/git-policy/cli/README.md` and the man page could say that the first contact with a host needs one plain
`ssh <host>`,
which costs no code and does nothing for the person who meets the refusal without having read it.
That is a complement to any option here and never an option of its own.

Pros: no code,
no new mode,
no policy change,
exec semantics untouched,
and the refusal is exactly the owner's recorded rule.
Cons: the first contact with every new host ends without a next step,
a key whose passphrase was never loaded ends the same way,
and a changed key ends with OpenSSH's banner and no advice about the offending line.

### B. The wrapper itself as the askpass program

Answered "Never (Recommended)" by the owner on 2026-11-24
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
so this is recorded for its evidence and is not ranked.
The mechanism is kept here because it is the only option that would name the host, the address, the port and the key at
the question, and because a later round may need to know what it would have cost.

Mechanism: the overlay keeps `SSH_ASKPASS_REQUIRE=force` and names the wrapper's own path in `SSH_ASKPASS`
instead of `false`.
A program, unlike the retired sentence, is a value OpenSSH can start, which is why B survives H1 where J does not.
OpenSSH starts the wrapper with the question as its only argument
(`readpass.c:77`),
so the start that answers no question becomes the start that prints the next step:

- it writes the sentence for that question to standard error,
  writes nothing to standard output,
  reads nothing from standard input,
  starts no Git,
  reads no repository,
  and exits 2,
  which OpenSSH reads as no answer (`readpass.c:99` to `:102`).

The selector,
fixed after the read showed the second round's shape test missing real questions and keeping impossible ones.
A start is the mode when both hold:

- **the environment names this executable.**
  `SSH_ASKPASS` or `GIT_ASKPASS` holds the path of the running executable,
  compared as paths after canonicalising.
  The overlay writes that value,
  and only a process started from the overlay's own Git can still carry it,
  its nested Git starts and OpenSSH's askpass start alike.
  The design keeps `GIT_ASKPASS` empty and reads it as well,
  so a later decision to cover Git's own credential questions needs no second selector.
- **exactly one argument, and it holds a space.**
  The argument does not start with `-`,
  and it holds at least one space,
  or OpenSSH set `SSH_ASKPASS_PROMPT`, which it does for a permission question and a notification.

Why that shape holds, corrected:

- every askpass prompt measured or read in OpenSSH 10.6p2 holds a space,
  including a keyboard-interactive prompt,
  which is built as `(%s@%s) %s` around the server's text
  (source, the list under "What OpenSSH does with an askpass program").
  So the space test is enough,
  and the second round's bare-word rule and its "one askpass start the shape still misses" are both withdrawn.
- no one-argument Git invocation puts a space inside the argument:
  with one argument, the argument is the subcommand,
  and a subcommand's name is one word,
  so a nested `git status`, `git foo_bar` or `git foo.sh` that a forwarded command starts is never the mode.
  The one shape the test can still take is a one-argument call to an alias whose name holds a space,
  which answers with the generic sentence and exits 2 instead of running Git:
  bounded, and the same class of breakage the second round accepted for a punctuated alias name.
- a question the mode does not recognise still gets an answer:
  the generic sentence in "The words", never a fall-through to Git.
  Only a start whose environment does not name the wrapper falls through,
  and that start is not an askpass start.
- `SSH_ASKPASS_PROMPT=none` marks a security-key notice rather than a question
  (`readpass.c:279`):
  the mode writes nothing and exits 2,
  because there is nothing to answer and, under B, the terminal already holds the notice.

Exit status, pinned on every branch:

- a recognised question: sentence, exit 2;
- an unrecognised question: generic sentence, exit 2;
- `SSH_ASKPASS_PROMPT=none`: nothing, exit 2;
- a panic in the mode: exit 101, which OpenSSH also reads as no answer (`readpass.c:99` to `:102`);
- every branch exits non-zero, and the mode sits **after** `install_payload_free_panic_hook`, so a panic's default
  message cannot echo the question text (`src/native/main.rs`, the hook's own comment says why it precedes anything
  that can scan).
  The last log line is kept by the wrapper's existing `end_after_the_log` wait,
  which is the mechanism the 2026-11-14 lost-line incident installed;
  a flush would not do it, because Rust's standard error is unbuffered.

What a person sees,
from this round's run with the kept stand-in
(run, `keep/run-1.log`, arms `b-newkey`, `b-passkey-only`, `b-differing-key`;
the stand-in is `probe-bin/askpass-b` in the second round's `keep/` folder,
1513 bytes, digest `b1b82b15a34e43f56b93f3155583f0e4efa3f97944695b60f80a97f044102e33`,
printed beside the transcripts in that log):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (key nobody could unlock)
Cloning into '/work/clone'...
monochromatic-git-policy-cli: this command needed the passphrase of the key `/root/.ssh/id_probe`, and no command the wrapper runs asks a person for one, so that key was skipped. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /root/.ssh/id_probe` before the next command, use a key without a passphrase, or have the server accept another key you hold.
root@127.0.0.1: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

```text
# git clone ssh://root@probe-host.test:2222/work/served.git     (a differing key for the address, CheckHostIP=yes)
Cloning into '/work/clone'...
monochromatic-git-policy-cli: this machine holds a different SSH host key for the address of `[probe-host.test]:2222` than the server sent (the offending key is in /work/out/kh-differing:2), and no command the wrapper runs asks whether to continue. If that key is old, remove that line from `known_hosts`, then run this command again.
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The passphrase case that then succeeds still prints the sentence and ends 0
(run, `keep/run-1.log`, arm `b-passkey-then-good`).
That is B's cost on the asked-then-succeeded case:
the sentence appears on a command that works,
because the question arrives before anyone knows the outcome.
The wording is true in both cases,
which the second round's wording was not.

What it does **not** cover,
each one measured this round or read from the source:

- a changed host key: no question, so nothing, and OpenSSH's banner alone (`b-changed-key`);
- `BatchMode=yes`: no question, so nothing (`b-batchmode`);
- `StrictHostKeyChecking=yes`: OpenSSH's own strict sentence and nothing from B (`b-strict-yes`);
- a start whose environment does not name the wrapper:
  a stale remembered path in a long-running service,
  where `readlink` of a running binary that was deleted gives `<path> (deleted)`
  (run on the host, this round),
  so ssh cannot start it and prints its own `fatal_f("exec(%s): %s", ...)` line;
  the wrapper never starts and no sentence appears.
  It is not a fall-through to Git, which the second round said.

No passphrase reaches the mode:
OpenSSH passes the question in the argument and reads the answer from standard output
(`readpass.c:77`, `:82` to `:94`),
so the mode never sees a secret, and its argument is no more exposed than the `false` it replaces.

What it starts: the wrapper, started by OpenSSH, once per question.
What it reads: its argument, two environment values, and its own path for identification; no repository, no Git.
Can it answer yes: no. It writes nothing to standard output and exits 2,
and `confirm` reads no answer as no (`sshconnect.c:601` to `:627`).

Cost on a command that meets no question: nothing measurable.
The arm that names the same ssh and adds the askpass value took `-0.50 ms` (range `-2.07` to `+2.17`,
nine of twenty-four positive), which is the noise floor itself,
because no process starts when no question is asked
(run, `keep/run-2.log`, arm `askpass_only`).
The cost on a refused question is one askpass start, and neither round resolved it:
the second round's seven paired rounds gave `+3.6 ms` (range `-12.2` to `+10.2`),
which its own spread swallows,
and this round did not repeat that arm.
For scale, the wrapper's own start with one question-shaped argument, on today's fall-through path,
is `11.67 ms` absolute (range `10.50` to `12.31`, `keep/run-2.log`, arm `wrapper_arg`);
the designed mode starts before the forwarding machinery, so a refused question costs at most that.

Pros:

- the only option whose words name the host,
  the address,
  the port and the key,
  and the only one that can tell a new key from a key that differs for the address,
  because it sees the question itself;
- the only option that needs nothing on the path of a command that meets no question,
  measured as nothing;
- it changes nothing about Git's process,
  streams,
  signals or exit status,
  and takes none of the person's transport settings;
- it survives `-q` and `LogLevel=QUIET`, where OpenSSH prints nothing at all
  (run, `b-quiet` and `b-loglevel-quiet`);
- it keeps the security-key touch notice at a terminal, because OpenSSH still owns standard error;
- it gives the commit service's own pushes the same words,
  and its record already folds a repeated host-key sentence
  (`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`).

Cons:

- on this machine OpenSSH may not run it:
  `git` confines `ssh` in its own child profile,
  which allows executing only `ksshaskpass` and VS Code's script,
  and today's `false` works only because a local rule names it.
  The exact rules are in "What building B would take".
  Without them B prints `ssh_askpass: exec(<path>): Permission denied` and no words,
  which is worse than A. Measured in the first round, unchanged since.
- it introduces a new way for the wrapper to be started,
  with its own selector rule and its own controls,
  and the self-start rule's third bullet, which is the owner's question.
- the sentence must never reach standard output,
  the mode must never read standard input,
  and every branch must exit non-zero:
  three invariants a control must hold.
- it sees questions of every kind,
  including the agent's key-confirmation,
  the multiplexing question and keyboard-interactive prompts,
  which get the generic sentence and nothing specific.
- the sentence appears on a command that then succeeds, when a passphrase key is skipped for one that works.
- a typed LFS push under B would say the same thing twice:
  the mode's sentence, then the upload's own sentence from the same refusal,
  because the upload reads the transcript and finds the host-key refusal there
  (source: `explanation_after_failure` and `reads_as_unknown_host_key`;
  the detector already skips the wrapper's own sentence, so the upload still speaks).
  The words must be shortened to one for that path, or the upload must learn to skip its own when the mode already
  spoke. That is a change this design needs and does not have.
- on this machine, the words need a root edit of two AppArmor files per build.

### J. The sentence itself as `SSH_ASKPASS`'s value

Shown, chosen on 2026-11-24, and retired by the owner on 2026-11-25:
`SSH_ASKPASS` holds `false` again
(`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`, "What `SSH_ASKPASS` holds":
"Put `false` back (Recommended)").
The reason is H1, and it is not a defect of the wording:
a value OpenSSH cannot start takes the connection down when the connection has a proxy
(the mechanism is quoted from OpenSSH's source under "The question, open again").
This round's arms reproduce it:
behind a `ProxyCommand` or a `ProxyJump`, with a locked first key and a working second, the clone ends 128 where
`false` ends 0, and the same arm with a relay that ignores SIGHUP ends 0
(run, `keep/run-1-behaviour.log`, family `f-noask`, arms `j-sentence` and `j-sentence-plant`).

The mechanism as it was designed:
`execlp` tries to execute a file named the sentence, fails,
and OpenSSH prints its own failure line with the value in it
(`fatal_f("exec(%s): %s", askpass, strerror(errno))`, `readpass.c:78`).
No program, no mode, no selector, no AppArmor rule, no process.
Everything below is that design's dated record, and none of it is a reason to build it again:
the failure line is not harmless, it reaches `cleanup_exit`.

Measured this round with the value a shipped J must use,
which cannot name the host
(run, `keep/run-3.log`; the value is 369 bytes, so the failure is `File name too long`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
ssh_askpass: exec(monochromatic-git-policy-cli: OpenSSH asked a question only a person can answer, and no command the wrapper runs asks one, so the answer was no. A new host key's question is answered by connecting once with plain `ssh <host>`, which the wrapper does not touch and which still asks; a key's passphrase is answered by starting an agent and loading the key with `ssh-add`.): File name too long
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The same line arrives for the passphrase case, with `Permission denied (publickey).` after it
(run, `keep/run-3.log`, arm `j-generic-passkey-only`).

What it starts: nothing (a failed `exec`).
What it reads: nothing.
Can it answer yes: no; the exec never happens, so no program can say anything.

The measured limits, each one a finding against it:

- **one text for every question.**
  The same sentence arrives for a new host key and for a passphrase,
  so it must cover both, and it cannot name the host or the key.
  The host is written `ssh <host>`, and the person supplies it from the command they just typed.
- **OpenSSH's framing.**
  The person reads the advice inside `ssh_askpass: exec(...): File name too long`,
  and the words are 369 bytes long in that position.
- **it prints on a command that then succeeds**:
  the clone with a passphrase key followed by a working key exits 0 with that line
  (run, the second round, `keep/run4.log`, arm `j-passkey-then-good`).
- **it disappears under quiet**:
  with `-q` the whole line vanishes and only Git's `fatal:` remains
  (run, `keep/run-3.log`, arm `j-generic-quiet`).
- **it also answers questions that are not refusals**:
  a multiplexing question gets the same text, whose "the answer was no" is right and whose remedies are not.

Cost on a command that meets no question: nothing; the value is one more environment string and no process,
and the askpass-only arm above is the same nothing.

Pros: no code, no mode, no selector, no AppArmor rule, no process;
it speaks to both refusals at the moment they happen;
and it is the only option that needs nothing on any host, confined or not.

Cons: the framing,
the single generic text,
the false appearance on a command that succeeds,
and silence under `-q`.
It also cannot be turned into more precise words without becoming B.
And the con that was not weighed when it was chosen:
it is not a program, so OpenSSH's failure path for it runs, and behind a proxy that path ends the connection.
The owner was told the cost was a printed line; the line was the least of it.

What is left of J on `main` after the askpass round:
nothing.
The value is `false` again,
`NO_ANSWER_SENTENCE` is gone,
the tests that read it are gone with it,
and `SPEC.md` and `README.md` say that the release names no next step at a new host key.
`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING` stays, because the upload's explanation and the push record still print the
wrapper's own host-key sentence.

### C. The wrapper keeps the child and reads its standard error

Mechanism: a typed network command no longer replaces the process;
the wrapper starts Git as its child,
relays standard error through a pipe while Git writes it,
then appends the sentence when the transcript reads as one of the refusals
(`reads_as_unknown_host_key`
and `reads_as_a_login_failure`, `src/native/lfs/lfs_pre_push.rs`).
The detector deliberately leaves a changed key and a revoked key to OpenSSH's own words,
it *does* accept the strict-checking sentence as an unknown key's own,
and it reads the differing-key question as an unknown key,
so C speaks for a new key, a passphrase or refused key, `BatchMode=yes`, `StrictHostKeyChecking=yes`,
and the rare differing-key question.
For that last one the sentence's way to add the key is wrong and its way to connect once is right,
which is the detector's own recorded limit.

What a person sees,
from the second round's run of the stand-in that forks Git and relays its standard error
(run, `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
arms `c-newkey`, `c-passkey-only`, `c-known`, `c-quiet`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

The words are B's,
after Git's last lines instead of between Git's first line and OpenSSH's.
The host in them is the stand-in's fixed text;
a real C would fill it from the remote it resolved,
which is composed, not run.

This round ran C's shape against a real `sshd` again, at the passphrase refusal and unchanged from the transcripts above
in everything but the address inside the sentence (run, `keep/run-1-behaviour.log`, arm `f-locked-direct-c-relay`):

```text
$ git clone ssh://target/work/served.git        (host key known, only a key nobody can unlock offered)
Cloning into '/work/clones/f-locked-direct-c-relay'...
root@localhost: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this command needed the passphrase of the key `/work/key_locked`, and no command the wrapper runs asks a person for one, so that key was skipped. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /work/key_locked` before the next command, use a key without a passphrase, or have the server accept another key you hold.
```

What it starts: Git as a child, one pipe, one relay thread or loop.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

At H1's case C ends 0 and prints nothing, directly and behind both proxies, because no refusal reached it
(run, `keep/run-1-behaviour.log`, family `f-noask`, arm `c-relay`).
Behind the `ProxyCommand` the proxy still received SIGHUP, which is what `ssh` does to its proxy at its own normal exit;
the person's clone was complete.

Cost on a command that meets no question,
against the null arm in the same rounds: `+0.43 ms` median, range `-1.47` to `+2.38`, seventeen of twenty-four
positive
(run, `keep/run-2.log`, arm `relay_git`).
That is inside the noise floor, so this round cannot say C costs a measurable amount on such a command.
This round measured it twice:
on the typed path, `+2.82 ms` median with a range of `-7.04` to `+18.65 ms` and seventeen of twenty-four positive,
which is inside this harness's noise floor of `+1.59 ms` (`-8.23` to `+18.15`)
(run, `keep/run-2-costs-and-progress.log`);
and as the work it adds, one fork, one pipe relay and one wait,
`+2.04 ms` against `/usr/bin/true`, positive in twenty-two of twenty-four rounds
(run, `keep/run-3-steps.log`, arm `relay-start`).
So C adds about two milliseconds of work to every typed network command, and the second figure is the one this harness
can resolve.

Pros: gives both cases words at the same place as the existing push sentences do,
names the host,
keeps every setting the person and the repository own,
and is the shape the wrapper already uses where it owns a child
(`run_real_git`, `src/native/invocation/forwarding.rs`),
and the shape the self-start rule's third bullet asks for.

Cons, recounted:

- **the live progress display disappears.**
  Git draws progress only when its standard error is a terminal,
  and under C that stream is the relay's pipe.
  Measured this round at a pseudo-terminal on both sides (`keep/run-3.log`, one clone of two thousand objects):
  Git's standard error on the terminal gives a `Receiving objects` line, a `remote: Counting objects` line and a
  `remote: Enumerating objects` line;
  through C's pipe all three are absent;
  with `--progress` all three return.
  **This round measured the same count through K, the option that puts a pseudo-terminal there instead of a pipe, and
  all three lines return** (run, `keep/run-2-costs-and-progress.log`: plain `1/1/1`, C's pipe `0/0/0`, K `1/1/1`).
  So this con belongs to the pipe and not to keeping the child, which is what K is for.
  Passing `--progress` is not free:
  it would mean adding a word to the caller's arguments,
  and the wrapper's own rule is that Git receives only the caller's arguments, in order
  (`src/native/main.rs`, the argument collection).
- **more than progress depends on that terminal.**
  Git checks `isatty(2)` for the sideband's progress (`sideband.c:316`),
  for the transport's (`transport.c:1230`, `:1358`),
  and for advice such as the editor's waiting line (`editor.c:70`).
- **the security-key notice is lost.**
  OpenSSH writes the touch notice itself only when its standard error is a terminal,
  and otherwise sends it to the askpass program, which is `false` here (`readpass.c:252`, `:279`).
  Source only; no key was available.
- **a signal sent to the wrapper alone no longer reaches Git.**
  Measured in the second round for both C and C2: after `kill -TERM` of the wrapper the child is still running
  (run, `keep/run4.log`, "a signal aimed at the wrapper alone").
  A terminal's control character signals the whole foreground group, so this shows for a signal aimed at one process,
  but the exec path gives it for free today.
- **it is blind under `-q`**:
  OpenSSH's line is suppressed, so the detector has nothing to read and no sentence appears
  (run, the second round, `c-quiet`).
- the wrapper must now own what the exec gave it:
  the exit status,
  the signal ending,
  the wait,
  and the ordering of its own notices against Git's output.

### C2. Git kept as a child with the same streams, read after exit 128

Mechanism: the wrapper no longer replaces the process,
but it does not take standard error either.
It starts Git as a child with the same standard input,
standard output,
standard error and terminal,
waits,
and when the child ended 128 reads the exit status and, if the design adds it, the same known-hosts check D does,
then prints a sentence.
Nothing is piped, no argument is added, no text is read.

What a person sees,
from the status-only stand-in, which prints whichever sentence file it is given and checks nothing
(run, this round, `keep/run-1.log`; the second round's arms are `c2-newkey`, `c2-passkey-only`, `c2-quiet`):

```text
# git clone ssh://root@127.0.0.1:2222/work/no-such-repository.git   (host key known; the repository is not there)
Cloning into '/work/clone'...
fatal: '/work/no-such-repository.git' does not appear to be a git repository
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

That is the disqualifying problem at the same time as the shape:
exit 128 does not mean "the host key" or "a passphrase".
Measured, it also means connection refused, a name that does not resolve, a changed key, `BatchMode=yes`,
`StrictHostKeyChecking=yes`, and a repository that is not there.
So C2 either cries wolf or says nothing:

- with the exit status alone: a missing repository and a refused connection get a paragraph about keys,
  the both-branch one when the stand-in was given that text
  (run, this round, arm `c2-refused-both-branch`, and the second round's `c2-notfound`, `c2-refused`);
- with a post-failure check: a missing repository whose host key is known still gets the passphrase sentence,
  because the check knows the key and nothing more
  (run, the second round, `keep/run5.log`, arm `c2-notfound-known-host`;
  the sentence there was the stand-in's fixed file, and no arm ran a real check).

**This round ran a real check**, in both directions, and the two shapes are separate options in everything below.
The check is the one D makes (`ssh -G` for the resolved host, port and known-hosts files, then `ssh-keygen -F` over each).
The first shape, `c2-check-unknown`, prints the host-key sentence only when the check finds no key;
the second, `c2-check-known`, prints the passphrase sentence only when the check finds one.
Run (`keep/run-1-behaviour.log`, families `f-known` and `f-locked`), the two shapes take each other's case:

- at a repository that is not there with the key known, `c2-check-unknown` stays silent (right) and `c2-check-known`
  prints the **passphrase** sentence (wrong: no key was involved);
- at a key nobody can unlock with the key known, `c2-check-known` prints the passphrase sentence (right) and
  `c2-check-unknown` stays silent (wrong: the one refusal the person needed words for gets none).

So a check does not fix C2; it chooses which of the two refusals C2 answers, and the other one gets either silence or a
false paragraph.
That is the pair of facts the earlier revision could not show, because no arm ran a real check.

Three more arms of the same stand-in, run in the second round (`keep/run9.log`):

- a passphrase key followed by a key that works ends 0 and prints nothing,
  which is the one case where C2 is better than B: the question did not decide anything, and nothing is said;
- `BatchMode=yes` and `StrictHostKeyChecking=yes` end 128 and get the new-key paragraph,
  with its first branch right and its second branch irrelevant;
- a **changed** host key ends 128 and gets the paragraph about a key this machine "does not know",
  which is false: it knows a different one,
  and the sentence that says so is the one thing the person needs there.

The only case where C2 can be both accurate and useful is the one it cannot see.
It cannot distinguish the refusals without reading text, and reading text is C.

What it starts: Git as a child; on a failure, `ssh -G` and `ssh-keygen -F` as well if the design adds the check.
What it reads: the exit status and, with the check, the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

At H1's case C2 ends 0 and prints nothing, in all three transports, because no refusal became a status it watches
(run, `keep/run-1-behaviour.log`, family `f-noask`).
That is the one case where the shape is better than C's: nothing is said about a question that did not decide anything.

Cost on a command that meets no question,
against the null arm: `+0.72 ms` median, range `-1.47` to `+2.31`, eighteen of twenty-four positive
(run, `keep/run-2.log`, arm `forkwait_git`).
Inside the floor, so not a finding;
on a success it is one fork and one wait, with the same streams.
This round: `+3.01 ms` median, range `-6.13` to `+11.95`, sixteen of twenty-four positive on the typed path, inside the
noise floor again (run, `keep/run-2-costs-and-progress.log`, arm `c2-status`);
and `+2.10 ms` as the work it adds, one fork and one wait, positive in twenty-three of twenty-four rounds
(run, `keep/run-3-steps.log`, arm `standin-start`).
With the check added the failure path pays D's resolution as well, which is measured in D's section.

Pros: keeps the terminal,
the progress display,
the signals from the terminal,
and every setting the person owns,
and adds no word to the caller's arguments;
it survives `-q`, because it reads a status and not a line
(run, the second round, `c2-quiet`).
Cons: it cannot tell which refusal it was, which is measured above;
the wrapper still owns the wait,
the exit status and the signal ending,
as C does;
and the paragraph has to be long enough to cover refusals it cannot see, which is what makes it cry wolf.

### K. The wrapper keeps the child, with standard error on a pseudo-terminal

Mechanism: C's shape, with one difference.
The wrapper starts Git as its child and gives it a pseudo-terminal on standard error instead of a pipe,
relays whatever that terminal produces to the wrapper's own standard error,
and after the child ends prints the sentence the transcript calls for, using the same two detectors C uses.
Standard input and standard output stay the child's own, no argument is added, and no program Git did not start is
asked anything, so the owner's three rules and the two "Never" answers all hold.

Why the pseudo-terminal at all:
Git, OpenSSH and `git-lfs` all ask `isatty(2)` before drawing anything a person watches.
Git draws its progress through `sideband.c:316` and `transport.c:1230`, `:1358`;
OpenSSH writes a security key's touch notice to standard error only when that stream is a terminal
(`notify_start`, `readpass.c:252`, `:253`, and otherwise sends it to the askpass program, `:279`).
A pipe answers no to that question, and a pseudo-terminal answers yes.

A concrete case.
The person's setup: `git clone ssh://target/work/served.git` typed in a shell,
`target` in `~/.ssh/config` with `HostName localhost`, `Port 2222` and two `IdentityFile` lines,
the first key's passphrase held by no agent and the second key without one,
and a `ProxyCommand` in the same file.
What the person sees at a new host key, with K in place
(run, `keep/run-1-behaviour.log`, arm `f-newkey-proxycommand-c-pty`; the sentence is the one in "The words"):

```text
# git clone ssh://target/work/served.git
Cloning into '/work/clones/f-newkey-proxycommand-c-pty'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

At a key nobody can unlock, with the host key known, the same shape prints the passphrase sentence
(run, arm `f-locked-direct-c-pty`).
At H1's case, a locked first key followed by a working one, it prints nothing and the clone ends 0, directly and behind
both proxies (run, family `f-noask`).

What it starts: Git as a child, one pseudo-terminal pair, one relay loop.
What it reads: the terminal's text as it arrives.
Can it answer yes: no; it only reads.

Cost, measured as the work it adds:
one fork, one pseudo-terminal relay and one wait cost `+2.66 ms` against `/usr/bin/true`, positive in twenty-three of
twenty-four rounds (run, `keep/run-3-steps.log`, arm `pty-start`).
C's pipe costs `+2.04 ms` in the same rounds (`relay-start`), so the pseudo-terminal is about `0.6 ms` more than the
pipe on every typed network command, and both are about one process start.
On the typed path the arm lands inside this harness's noise, as C's does
(run, `keep/run-2-costs-and-progress.log`, arm `c-pty`: `+1.37 ms` median, `-7.90` to `+13.91`, thirteen of twenty-four
positive).

What it keeps that C loses, measured (run, `keep/run-2-costs-and-progress.log`, one clone of two thousand objects,
standard error on a terminal):

- the terminal's own transcript shows a `Receiving objects` line, a `remote: Counting objects` line and a
  `remote: Enumerating objects` line under plain Git, under C2 and under K (`1`, `1`, `1` of each);
  through C's pipe none of the three appears (`0`, `0`, `0`);
- with `--progress` C's pipe shows them too, which is the workaround C would need and cannot add on a person's behalf.

What it costs that C does not:
the pseudo-terminal answers `isatty(2)` with yes even when the person redirected standard error,
so a person who runs `git clone ... > log 2>&1` gets the progress bytes in the file, where today there are none.
Measured: with both streams redirected to a file, plain Git writes no `Receiving objects` line and K writes one
(run, `keep/run-2-costs-and-progress.log`, the `REDIRECT` lines).
That is a real change in what a script's log holds, and it is the price of keeping the display.

What else a shipped K would have to get right, and which this round's stand-in does not:

- the pseudo-terminal is only standard error: no new session, no controlling terminal, so the person's own control
  characters still reach Git. The stand-in calls `setsid()` and `TIOCSCTTY`, which a shipped K must not do,
  and the arms' own signals are therefore not the ones a person's terminal would send;
- the window size should be copied from the person's standard error when it is a terminal (`TIOCGWINSZ` then
  `TIOCSWINSZ`), so progress lines are cut where the person's terminal cuts them, and a default of eighty by twenty-four
  is what the arms used;
- everything else the relay reads is text the wrapper must not print back: the arm relays it unchanged, and a shipped K
  relays it unchanged too, which is what C does.

What it misses, which is everything C misses:

- under `-q` OpenSSH prints nothing, so there is no refusal in the transcript and no sentence;
- a changed host key never asks and prints OpenSSH's banner, which the detectors leave to OpenSSH;
- `BatchMode=yes` and `StrictHostKeyChecking=yes` produce no question, so no sentence from K for them;
- it changes nothing about the transport, so a `ProxyCommand` or `ProxyJump` is Git's and not the wrapper's.

Pros: everything C gives (words chosen from what OpenSSH printed, the host named, the person's settings untouched, the
shape the self-start rule's third bullet asks for) and none of its loss of the progress display or the touch notice;
the cost is one process start plus about `0.6 ms`, which is what C costs.
Cons: the pseudo-terminal makes output look like a terminal to a child that writes to a redirected stream;
the wrapper now owns the pty's life and the child's wait, as C owns the pipe's;
and a `-q` transcript still yields nothing.

### D. A check before Git starts

Mechanism: before replacing the process,
resolve what OpenSSH would use and ask whether this machine holds a key for it:
`ssh -G <host>` for the resolved host name,
port,
`hostkeyalias`,
`userknownhostsfile` and `globalknownhostsfile`,
then `ssh-keygen -F` over every file `ssh -G` named,
under `[host]:port` when the port is not 22 and under the alias when there is one.
`ssh -G` connects to nothing.
When no file holds a key and the resolved `stricthostkeychecking` is `ask`,
print the sentence first,
then run the command as today.

The first round read only `userknownhostsfile`, and that finding was reproduced in the second:
with the key only in the global list, the real command exits 0 while the check says "no key"
(run, the second round, `keep/run4.log`, arms `d-known-in-global-file` and `d-old-read-user-file-only`).
Reading both lists fixes that case.

What a person sees
(run, the second round, `keep/run4.log`, arms `d-newkey`, `d-passkey-only`, `d-connection-refused`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, ...
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.
...
```

For the passphrase case it prints nothing at all:
the key is known,
so the check finds it,
and the person sees today's two lines and the passphrase is never mentioned (`d-passkey-only`).
That is D's gap,
measured,
and it is the case whose words the person most needs.

The sentence can also be wrong about why the command failed:
with nothing listening on the port the check still finds no key and prints the host-key sentence,
and the real words are `ssh: connect to host 127.0.0.1 port 2299: Connection refused`
(run, the second round, `d-connection-refused`).

Three false-positive classes, and what a probe says about each:

- **`KnownHostsCommand` supplies the key.**
  Reproduced this round:
  with `-o KnownHostsCommand=/work/bin/khc` and empty known-hosts files, the real clone exits 0,
  `ssh -G` names `knownhostscommand /work/bin/khc`,
  and D's stand-in prints its "does not know the key" sentence
  (run, `keep/run-1.log`; the control without the option prints the sentence and fails 128).
  D reads two files and never this command,
  so it speaks against a command that succeeds.
- **`@cert-authority` supplies the key by certificate.**
  The read guessed this was the same class; it is not.
  Measured with a host certificate signed by a fixture CA and an `@cert-authority` line as the only record:
  `ssh-keygen -F` finds the line, the real clone exits 0, and D's stand-in prints nothing
  (run, `keep/run-2.log`).
  The control without the line prints the sentence and fails.
  So D is right for this class, and the read's guess does not hold.
- **`VerifyHostKeyDNS=yes`.**
  `ssh -G` names the setting and D does not read it,
  so a key supplied by DNS would get the sentence while the connection succeeds.
  Unrun: no DNSSEC setup was available, so this is source only.

Two more limits:

- the stand-in never reads `stricthostkeychecking`,
  although the design says D must print nothing when the person's setting already accepts unknown keys
  (source: the stand-in reads `hostname`, `port`, `hostkeyalias`, `userknownhostsfile` and `globalknownhostsfile`
  only).
  `ssh -G` does report it (`stricthostkeychecking ask`, run, `keep/run-2.log`),
  so the design's condition is one read away and no arm exercises it.
- a changed host key looks known to it (`ssh-keygen -F` finds a key under the name),
  so it says nothing about the refusal whose advice needs the offending line.
- it is blind to a `HostName` alias whose key lives on a jump host,
  and to any known-hosts file a `ProxyCommand` or `ProxyJump` makes the far side read.

What it starts: `ssh -G` and `ssh-keygen -F` before the command.
What it reads: the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost on a command that meets no question,
against the null arm in the same rounds: `+14.13 ms` median, range `+12.21` to `+16.55`,
and every round of the twenty-four was positive,
so this one is a finding rather than noise:
it is 4.1 times the wrapper's own start cost in the same run
(run, `keep/run-2.log`, arms `precheck_git` and `wrapper`).
The second round's eighteen rounds gave `+28.97 ms` on a different container load;
the two runs disagree in size and agree in sign and significance,
and neither magnitude transfers off this host.
This round measured D twice.
On the typed path it is `+7.33 ms` median with a range of `-3.91` to `+34.61` and twenty-two of twenty-four positive,
which the noise floor of that harness (`+1.59 ms`, `-8.23` to `+18.15`) cannot separate from noise
(run, `keep/run-2-costs-and-progress.log`, arm `d-precheck`).
Measured as the work it does, the check costs `+8.62 ms` against `/usr/bin/true`, positive in twenty-four of twenty-four
rounds, and it splits into its two starts:
`ssh -G` alone costs `+5.35 ms` (`10.89 ms` absolute) and `ssh-keygen -F` costs `+2.99 ms` against a file that holds
the host and `+3.21 ms` against one that does not (run, `keep/run-3-steps.log`, arms `precheck-start`, `ssh-g`,
`keygen-known` and `keygen-empty`).
So D adds about `8.6 ms` of work, in two extra process starts, to every typed network command.

Two costs the first round did not name:

- **`ssh -G` runs the person's `Match exec` commands.**
  With a `Match host 127.0.0.1 exec ...` line in the configuration,
  one plain `git ls-remote` ran it once and D's check plus the ssh it then runs ran it twice
  (run, the second round, `keep/run4.log`, the `Match exec` lines).
- **`ssh -G` can resolve names.**
  `CanonicalizeHostname` makes `ssh -G` do DNS work, which was not run here.
  Source: `ssh_config(5)`, "CanonicalizeHostname".
  The output can also carry `ProxyCommand` and `SetEnv` values, which must never be logged.

Pros: no change to how Git runs,
no new mode,
no policy change,
and the words arrive before the connection,
so a person who reads quickly sees the advice above the failure;
it reads the same files OpenSSH reads for the plain case.
Cons: one of the two cases is out of reach,
the words can be wrong about the failure and wrong on commands that succeed,
the cost is paid by every network command,
and the check runs the person's own configuration commands one extra time.

At H1's case D ends 0 and prints nothing, in all three transports, because the host key is known and
the `ProxyCommand` and `ProxyJump` change nothing in the check (run, `keep/run-1-behaviour.log`, family `f-noask`).
At a new host key it speaks before Git starts, directly and behind both proxies
(run, `f-newkey-proxycommand-d-precheck`: the sentence, then `Cloning into`, then OpenSSH's refusal),
and the check's silence where the key is known is the same fact that makes it useless for a passphrase.

### E. The wrapper becomes the ssh that Git starts

Answered "Never (Recommended)" by the owner on 2026-11-24
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
so this is recorded for its evidence and is not ranked.
The owner's reason is the rule this section's first con breaks:
the process that explains would be one Git started,
and the wrapper would hand its own path to Git to start.

Mechanism: the overlay names the wrapper in `GIT_SSH_COMMAND` (or `GIT_SSH`),
so the wrapper starts the real `ssh`,
relays its standard error,
and appends the sentence when the words read as one of the two refusals,
while Git keeps the terminal and the process stays replaced.

What a person sees,
from the second round's run of the stand-in
(run, `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
arms `e-newkey`, `e-passkey-only`, `e-known`, `e-quiet`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
Host key verification failed.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The live progress display survives,
because Git's standard error is still the terminal
(run, this round, `keep/run-3.log`).

Cost on a command that meets no question,
against the null arm in the same rounds: `+5.97 ms` median, range `+3.57` to `+8.12`, twenty-four of twenty-four
positive
(run, `keep/run-2.log`, arm `ssh_mode`).
The second round reported `+14.68 ms` for the same arm on a dev build and a loaded host;
this round's release build on a quiet host gives the smaller figure and the same sign.

**E's arm relabelled, and its cost split.**
The arm names a program Git does not recognise,
so Git starts it twice: once with `-G` to learn the variant and once for the connection.
Measured this round with a counting program:
two starts, and two ssh starts under it
(run, `keep/run-1.log`; script `03-start-counts.sh`).
A real E named in `GIT_SSH_COMMAND` pays the same two wrapper starts and two ssh starts,
and `GIT_SSH_VARIANT=ssh` in the overlay would remove the probe
(`override_ssh_variant`, `connect.c:1206` to `:1224`).
So the like-for-like baseline for E is the same ssh under a name Git does not recognise,
which this round measured at `+4.93 ms` (range `+2.03` to `+7.03`), twenty-four of twenty-four positive
(arm `ssh_renamed`).
Of E's `+5.97 ms`, about `4.9` is the probe any unrecognised named ssh pays and about `1.0` is E's relay for two
starts.
A real E must therefore also pass `-G` through and end with ssh's own status,
because Git reads only that status and discards the probe's streams
(`connect.c:1426` to `:1439`).

What it starts: the wrapper twice on a network command, and the real `ssh` twice under it,
once for the probe and once for the connection.
What it reads: the `ssh` child's standard error.
Can it answer yes: no.

Pros: covers both cases,
keeps the terminal,
the progress and ssh's exit status,
and leaves the person's command untouched beyond its own start and the `-G` probe Git runs for a name it does not
recognise.

Cons, recounted:

- it takes over `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`,
  which belong to the person and to the repository.
  Replacing them loses their options;
  wrapping them means building a shell command out of a shell command,
  which the `shell-text` rule forbids
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`).
  A person with `core.sshCommand` set to their own proxy or key options loses them, and their command
  fails differently.
- it must find the real `ssh` the way `real_git.rs` finds Git,
  including the refusal to run back into itself,
  and it must keep working when `ssh` is a wrapper of the person's own.
- it needs a selector of its own,
  and the only one left is B's, because the start name is forbidden.
- it is blind under `-q` and `LogLevel=QUIET`, because it reads lines OpenSSH then does not print
  (run, the second round, `e-quiet`).
- **a signal sent to the wrapper alone does not reach ssh**, for the same reason as under C:
  the relay is a child of the wrapper and ssh is the relay's child, and the relay forwards no signal
  (source: the stand-in's `relay.c` installs no handler).
  The second round measured this for C and C2 and not for E, so E's line is read and not run.
- **the security-key notice is lost**:
  ssh's standard error is the relay's pipe, so `notify_start` sends the touch notice to the askpass program,
  which is `false`, instead of writing it (source, `readpass.c:252`, `:279`).
- its words come from the same detectors C uses,
  so they miss the changed key and anything else only the question text holds.
- it meets the self-start rule's third bullet: the process that explains was started by Git,
  which is the owner's question.

### F. Automatic trust, and other ways to answer for the person

`StrictHostKeyChecking=accept-new`,
a pre-added key,
or an askpass program that prints `yes` would all make the commands succeed,
and all of them answer a yes/no question on a person's behalf,
which this round's brief forbids,
and which the owner's answer of 2026-11-20 also rules out by having the service wait for a key the machine learns
some other way.
They are listed so the reading is complete,
and they are not ranked.

### G. Relax the no-prompt rule for typed commands

Clearing `SSH_ASKPASS_REQUIRE` for a command a person typed,
or setting it to `never`,
would hand OpenSSH's question back to the terminal.
The owner's rule covers every command the wrapper runs or forwards,
and a typed command inside a script or a hook has no terminal to answer on.
It is not ranked.

### H. Run the transport itself

The wrapper would speak SSH and the Git protocol,
or shell out to `ssh` for the whole exchange rather than to Git.
Nothing in the two refusals needs it,
the cost is a transport implementation and its security surface,
and it takes the network path away from Git for every command.
It is not ranked.

## The ranking

One ranking,
because the two questions the earlier rankings turned on are answered:
B and E are "Never" (`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
so no program Git started explains anything,
and J is retired because its value cannot be started.
What is left is C, C2, D, A and this round's K.
Everything is ranked by what each delivers to the person who typed the command:
whether the words arrive at the refusal that stopped them,
whether they name what the person must do,
whether they appear when nothing is wrong,
and what the person gives up on every other command.

**K > C > C2 > D > A.**

- **K over C.**
  Both read the same transcript, name the host, keep every setting the person owns, and add one process start.
  Measured, that start and its relay cost `+2.04 ms` as a pipe and `+2.66 ms` as a pseudo-terminal, in the same rounds
  and against the same control (run, `keep/run-3-steps.log`).
  The `0.6 ms` buys back the two things a pipe takes from every clone and fetch:
  Git's live progress display, measured as three lines under plain Git, none through C's pipe and all three through K's
  terminal (run, `keep/run-2-costs-and-progress.log`),
  and OpenSSH's security-key touch notice, which is written to standard error only when that stream is a terminal
  (`readpass.c:252`, `:279`).
  K's own price is one behaviour a person can see:
  with both streams redirected to a file, the progress bytes land in the file where today there are none (measured).
  That is a smaller loss than taking a live display away from everyone, and it is the only pair where the option with
  the plainer words is also the one that gives back more.
- **C over C2.**
  C's words are chosen from what OpenSSH printed, so they fit the refusal that happened:
  measured, C is silent at a repository that is not there and speaks at a key nobody can unlock.
  C2 with the exit status alone speaks about a host key at a repository that is not there (measured).
  C2 with a known-hosts check does not fix that, it chooses which refusal to answer:
  the shape that speaks only when the key is unknown loses the passphrase refusal,
  and the shape that speaks only when a key is known prints the passphrase sentence at a repository that is not there
  (both measured, families `f-locked` and `f-known`).
  What C2 keeps over C and K is `-q`, where it still reads a status while they read lines OpenSSH has stopped printing;
  against C it has that and nothing else, and it pays with words that can be wrong.
- **C2 over D.**
  D's words arrive before a command that runs anyway, so they cannot let a person stop;
  they never cover a passphrase (measured: silent at the locked key);
  they are wrong on commands that succeed when `KnownHostsCommand` supplies the key (measured in the third revision's
  run), and wrong about a connection refused (measured in the second revision's run);
  and D pays two extra process starts on every typed network command, `+8.62 ms` of work against C2's `+2.10 ms`
  (run, `keep/run-3-steps.log`).
  Both can misteach: C2's is an extra paragraph after a real error, and D's is a first line that can be wrong before the
  truth arrives, plus a cost on every command.
- **D over A.**
  D speaks accurately for the plain new-key case, which is the common one, and A never speaks.
  D pays for that on every network command;
  A pays nothing and leaves the person with no next step at the first contact with each host.

### What each answer gives the owner

- **K** is the first option.
  It needs the sentences in "The words", the two detectors that already exist, a child the wrapper already knows how to
  keep, and a pseudo-terminal, with the three things named in its section to get right.
  It costs about `2.7 ms` of work on every typed network command, and it changes what a redirected log holds.
- **C** is the same shape with a pipe: about `0.6 ms` cheaper, and it takes the live progress display and the security
  key's touch notice from every clone and fetch.
- **C2** is the least work on the failure path and the least accurate: it names no host and can describe the wrong
  failure.
- **D** is the only option whose words arrive before the command, and the only one that pays two process starts on every
  command.
- **A** is what `main` does after the askpass round: nothing, at both refusals.

### What the earlier rankings said, kept for the record

The third revision ranked twice, because the self-start rule has a reading the owner had not then chosen between:
`J > C > C2 > D > A` read strictly,
and `B > J > C > E > C2 > D > A` with B's exception granted.
The owner answered that round's three questions ("Never" for B, "Never" for E, and "J" for the option),
so the second ranking never had a reader,
and the first is superseded by this round's:
J is retired, and K is the addition that changes the order, because C no longer has to lose the progress display to give
the person words.
The pair reasons of the earlier rankings that still hold are the ones about C over C2 and C2 over D, and this round kept
them and measured them again.

## The owner's question

> Which delivery should the wrapper build, now that `SSH_ASKPASS` holds `false` again:
> **K**, the child's standard error on a pseudo-terminal, with the words chosen from the transcript (ranked first);
> **C**, the same with a pipe;
> **C2**, a paragraph chosen from the exit status and a known-hosts check;
> **D**, a `known_hosts` check before Git runs;
> or **A**, nothing?

The ranking is K > C > C2 > D > A, and each adjacent pair's reason is under "The ranking".

What each answer leaves:

- **K**: the next step appears at every refusal OpenSSH or Git prints, names the host, and costs one process start and
  about `0.6 ms` more than a pipe on every typed network command.
  A person who redirects standard error gets Git's progress bytes in the file, and a `-q` command still gets nothing.
- **C**: the same words for about `0.6 ms` less, and every clone and fetch loses the live progress display and the
  security key's touch notice.
- **C2**: no transcript is read, so `-q` still works and nothing is lost on a successful command, and the words can
  name the wrong failure whichever of its two shapes is built.
- **D**: advice that arrives before the command fails, one of the two refusals never covered, and two extra process
  starts on every typed network command.
- **A**: nothing changes, and a person at a new host key or with a key no agent holds still reads OpenSSH's line and
  Git's `fatal:` with no next step.

Until the owner answers, `main` holds `false` and names no next step:
that is the state the askpass round leaves, and `SPEC.md` and `README.md` say so.
The question belongs beside the L4 entry in `doc/handover/questions-for-the-owner.md`;
this round may not edit that file,
so it is put here and in this round's report.

## The words

These three sentences were designed for the options that own a child, and were not built.
They are what K and C would print, and what B would have printed,
so the option the owner picks starts from words that were already read and argued over.
Nothing on `main` holds them today:
`NO_ANSWER_SENTENCE` went with J (see "What became of the answer J"),
and the wrapper's own two sentences for the LFS upload and the service's push record are separate, because their
subjects are an upload and a push.

Three sentences,
all new,
verb-neutral (`run this command again`, never `push again`),
and one generic fallback for a question the mode does not recognise.
They live beside the two that exist today
(`src/native/lfs/lfs_pre_push.rs`),
because the same detectors must not drift,
and the existing two stay for the LFS upload and the service's push record,
whose subjects are an upload and a push.

New host key,
with the address from the question or the remote,
and the way to connect built from that address with no login name:

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

The login name is left out on purpose.
An askpass start cannot know it, and every option prints the same words,
so no option claims a name it does not have.
For a standard port the form is `ssh <host>`.

A key nobody could unlock,
with the key's path from the question,
true whether or not the next key works:

```text
monochromatic-git-policy-cli: this command needed the passphrase of the key `/root/.ssh/id_probe`, and no command the wrapper runs asks a person for one, so that key was skipped. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /root/.ssh/id_probe` before the next command, use a key without a passphrase, or have the server accept another key you hold.
```

Two things were deliberate:
`so that key was skipped` is what happens at that moment
(`sshconnect2.c:1553` to `:1558`),
and the sentence says `before the next command` rather than `then run this command again`,
because the command may already have succeeded with the next key.

A key that differs for the address,
with the offending location taken from the question itself:

```text
monochromatic-git-policy-cli: this machine holds a different SSH host key for the address of `[probe-host.test]:2222` than the server sent (the offending key is in /work/out/kh-differing:2), and no command the wrapper runs asks whether to continue. If that key is old, remove that line from `known_hosts`, then run this command again.
```

This question is rare, because `CheckHostIP` defaults to off,
and for it the sentence's way to add the key is wrong and its way to connect once is right.

Any other question the mode sees gets one generic sentence,
because the text may come from the server and must not be echoed into a sentence
(a keyboard-interactive prompt is written by the server, `sshconnect2.c:2125` to `:2130`):

```text
monochromatic-git-policy-cli: OpenSSH asked a question only a person can answer, and no command the wrapper runs asks one, so the answer was no. A new host key's question is answered by connecting once with plain `ssh <host>`, which the wrapper does not touch and which still asks; a key's passphrase is answered by starting an agent and loading the key with `ssh-add`.
```

Three rules about printing text that came from outside:

- no text from a question reaches a sentence unescaped:
  the host and the key path are printed only after every non-printable byte is dropped,
  and the server's own prompt text is never printed at all.
- nothing the sentence offers as a command is built from that text without regard for the shell's grammar.
  A host is printed in the connect form only when it is a plain host name or address;
  otherwise the sentence names the host in backticks and says to connect to it with plain `ssh`.
  The same goes for a key path holding a space, `$` or a quote.
- a key path cut by OpenSSH's `%.100s` is not offered as a command at all:
  `ssh-add <the path the question names>` becomes `load the key the question names with `ssh-add``,
  because the cut path may name no file.
  The mode cannot tell a cut path from a long one, so the rule is a length test on the question's own text.

## What building each option would take

B's list is kept first, because it is the longest and because the next round may have to read it against a future answer;
nothing of it is built.
K's list follows, and C's and C2's are K's with the middle step changed.

### B, answered "Never"

- `no_prompt_overlay` names the wrapper's own path in `SSH_ASKPASS`
  and keeps `SSH_ASKPASS_REQUIRE=force`
  (`src/native/invocation/child_environment.rs`).
  `GIT_ASKPASS` stays empty.
  Covering Git's own login questions too would cost one
  `error: unable to read askpass response from '<path>'` line per failed question (measured in the first round),
  and it would put the mode in a path where the failing host is known but the message is not one of the two refusals.
  That is a separate decision and not part of this one.
- a mode at the top of `main`,
  after `install_payload_free_panic_hook` and before any planning:
  the second round put it before both, which would let a panic's default message echo the question text.
  It is selected by the two signals in "B",
  prints one sentence and exits 2 on every branch,
  and keeps its last log line through `end_after_the_log`, not a flush.
- the sentences in one module beside `lfs_pre_push.rs`,
  with the address and the key filled from the question.
- the host-key detector needs no change:
  `reads_as_unknown_host_key` already skips the wrapper's own sentence through
  `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`,
  and the login detector's five words appear in none of the new sentences
  (source, `src/native/lfs/lfs_pre_push.rs`).
  What does need a decision is the LFS double sentence:
  a typed LFS push under B would print the mode's sentence and then the upload's own sentence for the same refusal,
  because both read the same transcript.
  Either the upload learns to stay silent when the transcript already holds the mode's sentence,
  or the mode's sentence is the shorter of the two for that path.
- controls:
  - a new-host-key control and a passphrase control asserting the sentence, the exit status and today's lines;
  - a control that a start whose environment names the wrapper and whose one argument is a question prints and
    exits 2, and one that a nested one-argument `git status` inside a forwarded command still runs Git;
  - a control that a changed host key and `StrictHostKeyChecking=yes` and `BatchMode=yes` print nothing;
  - a control that standard output is empty and standard input unread;
  - a control that every branch, including a panic, exits non-zero, and that a panic's message holds no question text;
  - a control that a command which meets no question starts no askpass program;
  - a control that an LFS push meeting a host-key question prints one sentence, not two.
- a reading of the source guards' `own-executable` rule,
  with an exception entry beside `real_git.rs`'s,
  and a line in `package/git-policy/cli/README.md` saying that a confinement which denies executing the wrapper leaves
  OpenSSH's own `ssh_askpass: exec(...)` line instead of the sentence.
- the AppArmor rules this machine needs:
  the ssh that Git starts runs under a stack,
  measured as `git//&git//ssh (enforce)`,
  and an exec is allowed only when **both** stacked profiles allow it.
  So the same path needs a rule in **both** local files:
  - `/etc/apparmor.d/local/git_ssh`, which today holds `@{bin}/false rix` and `owner /work/** rwk`
    (read, write and lock, no execute), needs
    `<the wrapper's own path> rix,`
    and, for the controls, the paths the gate and the host tests start the wrapper from.
  - `/etc/apparmor.d/local/git`, which today holds `owner /work/** mix`,
    `owner @{HOME}/.cargo/build/** mix` and one exact build path, needs the same line whenever the wrapper's path is
    not already covered there.
  - On this development host the wrapper's path carries a build identity
    (the exact build path in `local/git` names one commit's build),
    so the rule is per build;
    a released install adds it once for the installed path.
  - The gate's `/work` path and the host tests' `@{HOME}/.cargo/build/**` path are already `mix` in `local/git`,
    which is an execute grant for the `git` half of the stack;
    `local/git_ssh` has no execute grant, so B's controls fail in the gate without one there.
    Without it, the sentence is replaced by `ssh_askpass: exec(<path>): Permission denied`.
  - A person's own `ssh` (for example one a hook starts) runs under `/etc/apparmor.d/ssh`,
    which allows only `@{lib}/ssh-askpass/x11-ssh-askpass pux`,
    so B shows no words there.
  - Nothing else is known to be needed:
    the mode reads its environment, its argument and its own path,
    and `current_exe()` is already what the wrapper does under the `git` profile for identification
    (`src/native/invocation/real_git.rs`).
    That these reads are allowed under the `git//ssh` stack is an inference from the existing start,
    not a probe:
    the `git` profile needed eight batches of read rules for the whole wrapper,
    and `git_ssh` has none of them.

### K, the first option

K changes where a typed network command's child keeps standard error, and nothing else about it.

- `replace_process_with_real_git` (`src/native/invocation/forwarding.rs`) stops being the path for a typed
  `clone`, `fetch`, `pull` and non-LFS `push` at a remote that ssh carries, and `run_real_git` starts Git instead,
  which is the call the wrapper already uses where it owns a child.
  Which commands these are is a rule the wrapper already applies when it decides whether a command is forwarded;
  the set must be the same one, or a person meets two different behaviours by subcommand.
- standard error of the child is one end of a pseudo-terminal pair (`openpty`, then `dup2` of the slave),
  and the wrapper relays the master to its own standard error while keeping a bounded tail for the detectors,
  as `lfs_upload_explanations` already does for a child it owns.
  No new session and no `TIOCSCTTY`: the person's terminal must keep sending its own signals to Git.
- the window size is copied from the wrapper's standard error when that is a terminal, and eighty by twenty-four
  otherwise, so a progress line is cut where the person's terminal cuts it.
- the sentences from "The words" go in one module beside `lfs_pre_push.rs`, with the address and the key filled from the
  transcript, and the two detectors stay where they are.
- the exit status, the signal ending and the wait belong to the wrapper now, which they already do for the LFS upload;
  that path's `SignalGuard` machinery is the shape to copy.
- controls:
  - a typed `clone` at a new host key prints the sentence after Git's lines and ends 128, direct and behind a
    `ProxyCommand` and a `ProxyJump`;
  - a typed `clone` with a locked first key and a working second ends 0 and prints nothing, direct and behind both
    proxies (the control the gates did not have for J);
  - a typed `clone` with only a locked key prints the passphrase sentence and ends 128;
  - a clone of a repository that is not there with the host key known prints nothing about keys;
  - a clone at a terminal still draws `Receiving objects`, and one with both streams redirected also does, on purpose,
    which the control should assert so the change is a decision and not a surprise;
  - a signal aimed at the wrapper alone still reaches Git, and the wrapper ends the way Git ended;
  - the LFS upload and the push record print one host-key sentence, not two,
    for the same reason B needed that control: both read the same transcript.

C is this list with a pipe on standard error, and its two controls change to assert that no progress line appears;
C2 is this list without a relay, with the status watched and, if its shape carries the check, D's two starts on the
failure path only.
D is `ssh -G` and `ssh-keygen -F` before the process is replaced, with the sentence printed before `Cloning into`.

## What the probes did not show

- **No run of the wrapper itself.**
  No wrapper code was changed this round, so every option is stood in for by a compiled stand-in
  (`keep/stand-ins/standin.c`), and every sentence printed is one of the design's own words from "The words".
  What the arms prove is the delivery and its cost, not the wrapper's reading of a repository.
- **Nothing ran under this host's own `ssh`.**
  The `git//ssh` stack refuses the shell a `ProxyCommand` needs (measured: `/bin/sh: Permission denied`), so a proxy arm
  cannot run under `/usr/bin/ssh` here at all, which is why every arm uses a copy of `ssh` under `/work`.
  That copy runs under the `git` profile alone, which is not what a person on this host meets.
- **No security key.**
  The touch notice's behaviour under a pipe or a pseudo-terminal is read from `readpass.c:252`, `:253` and `:279`, not
  measured: no key was available.
- **No signal aimed at the wrapper alone.**
  K's claim that the person's own control characters still reach Git is design; the stand-in calls `setsid` and
  `TIOCSCTTY`, which a shipped K must not, and no arm sends a signal to the wrapper only.
- **No arm with `Match exec`, `CanonicalizeHostname`, `KnownHostsCommand`, `HostKeyAlias` or `VerifyHostKeyDNS`.**
  The third revision's runs measured `KnownHostsCommand` (D speaks while the clone succeeds) and `@cert-authority`
  (D stays silent while the clone succeeds); those results stand and are quoted in D's section. The rest is source.
- **No changed host key, `BatchMode=yes` or `StrictHostKeyChecking=yes` arm this round.**
  The second revision measured those for C2's stand-in, and the third for B's; for C, C2 and K the detectors' source says
  they stay silent, and nothing here contradicts it.
- **The typed-path timing cannot resolve C, K or C2.**
  Every one of them lands inside this harness's noise floor, and only the step measurement separates them.
  A reader who wants a typed-path figure for K should read the step figure and add it to a command's own time, not the
  other way round.
- **No Windows**, and no OpenSSH release other than 10.6p2 or Git release other than 2.56.1.
- **No whole container gate, and no `native:check`, clippy, `lint:types` or host tests.**
  The only repository change is documentation, and the round's note says no whole gate is a merge condition.

## Measurements

Three container runs of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`),
each `--rm --init --network=none --cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
each started in a `systemd-run --user` scope with `RuntimeMaxSec=1800` or `1200`, `CPUQuota=300%` or `200%`,
`MemoryMax=6G` or `4G`, `TasksMax=1024` or `512`, and `IOWeight=10`:

- `host-key-next-step-options-2026-11-25`: the behaviour arms, log `keep/run-1-behaviour.log`;
- `host-key-next-step-costs-2026-11-25`: the typed-path timings and the progress counts, log
  `keep/run-2-costs-and-progress.log`;
- `host-key-next-step-steps-2026-11-25`: the step costs, log `keep/run-3-steps.log`.

OpenSSH 10.6p2, Git 2.56.1, `sshd` on 127.0.0.1:2222, and the stand-ins built inside each container by `cc`
(`keep/lib.sh`).

Method, so a difference can be told from this harness's noise:

- every arm ran in every round, and each round rotated the order by one, so an arm sits at every position;
- two warm-up rounds ran before the twenty-four measured ones;
- an arm's figure is the median of its per-round difference from the control in the same round, with the range of those
  differences and the count of positive rounds beside it;
- the typed-path control is `git ls-remote` with the same `ssh` binary, the same options and `SSH_ASKPASS=false`;
- the typed-path noise floor is the same command run twice in the same round (`null-a` against `null-b`);
- the step-cost control is `/usr/bin/true`, one process start.

Typed path: `git ls-remote` of one repository over loopback, `145.63 ms` for the control
(run, `keep/run-2-costs-and-progress.log`):

- the same command twice (`null-b` against `null-a`), the noise floor: **`+1.59 ms`**, range `-8.23` to `+18.15`,
  fourteen of twenty-four positive;
- the same Git with no ssh options at all (`plain-git`): `11.66 ms` absolute, and not a control, because without
  `-F` the destination `target` is a name nothing resolves, so that arm ends before it connects;
- C's relay: `+2.82 ms`, range `-7.04` to `+18.65`, seventeen of twenty-four positive, **inside the floor**;
- K's pseudo-terminal relay: `+1.37 ms`, range `-7.90` to `+13.91`, thirteen of twenty-four positive, **inside the floor**;
- C2's fork and wait: `+3.01 ms`, range `-6.13` to `+11.95`, sixteen of twenty-four positive, **inside the floor**;
- D's check: `+7.33 ms`, range `-3.91` to `+34.61`, twenty-two of twenty-four positive, **inside the floor**;
- one stand-in start wrapped around `true`: `-139.12 ms`, context only, since it measures no connection.

The steps themselves, against `/usr/bin/true` at `5.94 ms` absolute
(run, `keep/run-3-steps.log`, twenty-four rounds, every arm clear of the floor by its sign):

- the fork and wait C2 adds (`standin-start`): **`+2.10 ms`**, range `-0.37` to `+6.18`, twenty-three of twenty-four;
- C's pipe relay (`relay-start`): **`+2.04 ms`**, range `-1.08` to `+4.44`, twenty-two of twenty-four;
- K's pseudo-terminal relay (`pty-start`): **`+2.66 ms`**, range `-0.85` to `+5.18`, twenty-three of twenty-four, so the
  pseudo-terminal costs about `0.6 ms` more than the pipe;
- `ssh -G -F <config> target` (`ssh-g`): **`+5.35 ms`**, range `+2.61` to `+8.74`, twenty-four of twenty-four,
  `10.89 ms` absolute;
- `ssh-keygen -F '[localhost]:2222' -f <file>` over a file that holds the host (`keygen-known`): **`+2.99 ms`**,
  range `+1.13` to `+5.24`, twenty-four of twenty-four, `8.68 ms` absolute;
- the same over a file that does not (`keygen-empty`): **`+3.21 ms`**, range `+1.37` to `+6.91`, twenty-four of
  twenty-four, `9.19 ms` absolute;
- D's whole check before a program that does nothing (`precheck-start`): **`+8.62 ms`**, range `+6.54` to `+10.78`,
  twenty-four of twenty-four, `14.53 ms` absolute.

Progress, one clone of two thousand objects at a pseudo-terminal on both sides
(run, `keep/run-2-costs-and-progress.log`, the `PROGRESS` lines, counting `Receiving objects`,
`remote: Counting objects` and `remote: Enumerating objects`):

- plain Git: `1`, `1`, `1`;
- through C's pipe: `0`, `0`, `0`;
- through K's pseudo-terminal: `1`, `1`, `1`;
- through C2's inherited streams: `1`, `1`, `1`.

With both streams redirected to a file, no terminal on either side (the same log's `REDIRECT` lines):

- plain Git: no `Receiving objects` line;
- K: one, which is the change K makes to what a redirected log holds.

What the figures support and what they do not:

- K, C and C2 all add about two milliseconds of work and one process start; K's pseudo-terminal adds about `0.6 ms` to
  C's pipe, and no measurement here resolves that difference on a network command, which is why the ranking turns on what
  the streams keep rather than on time.
- D adds about `8.6 ms` of work in two starts on every typed network command, and that is the only cost in this round
  that a person would feel on a fast connection.
- Every figure is a loopback round trip inside a two-CPU container on this machine.
  A person's network dominates all of them, so the numbers say what the wrapper adds, not what a command takes.

The third revision's typed-path figures are kept in its own section's words and agree in sign:
C `+0.43 ms`, C2 `+0.72 ms`, D `+14.13 ms`, plain wrapper start `+3.48 ms`.
Their magnitudes do not transfer to this round's container, and neither set transfers off this machine.

## References

- `package/git-policy/cli/src/native/invocation/child_environment.rs` (`no_prompt_overlay`, `invocation_overlay`).
- `package/git-policy/cli/src/native/invocation/forwarding.rs` (`run_real_git`, `replace_process_with_real_git`).
- `package/git-policy/cli/src/native/lfs/lfs_pre_push.rs` (the two sentences, the three detectors, and
  `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`).
- `package/git-policy/cli/src/native/lfs/lfs_upload_explanations.rs` (`explanation_after_failure`).
- `package/git-policy/cli/src/native/main.rs` (the argument collection, the service mode, the panic hook).
- `package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs` (the rules and the exceptions).
- `package/git-policy/cli/SPEC.md`, "No command the wrapper runs or forwards asks a person for a credential",
  "The wrapper never starts itself as Git", and the push record's words for an unknown host key.
- `package/git-policy/cli/README.md`, the paragraph on prompts and `SSH_ASKPASS`.
- `doc/planning/cli-git-no-self-invocation.md`, "The rule".
- `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
- `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- `doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`.
- `doc/decision/cli-git-alpha-46-release-pass-four-answers.md`, "What `SSH_ASKPASS` holds".
- `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md` (the answers "Never" for B and E, and
  "J" for the option that was then retired).
- OpenSSH `V_10_6_P2`: `readpass.c`, `sshconnect.c`, `sshconnect2.c`, `ssh.c`, `clientloop.c`, `log.c`, `log.h`,
  `readconf.c`, `mux.c`.
- Git 2.56.1: `prompt.c`, `connect.c`, `sideband.c`, `transport.c`, `editor.c`, `strbuf.c`.
- This round's folder, `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/keep/`:
  `run-1-behaviour.log` and `run-1-behaviour.sh`, `run-2-costs-and-progress.log` and its script, `run-3-steps.log` and
  its script, `lib.sh`, `stand-ins/standin.c`, `stand-ins/proxy.c`, `sentences/host-key.txt`,
  `sentences/passphrase.txt`, `sentences/j-value.txt`, and `state-so-far.md`.
- The askpass round's fix, `c336355bcd` on `fix/git-policy-cli-askpass-false-and-commit-push-pass-fixes`:
  `NO_ANSWER_PROGRAM`, the `ProxyCommand` control, and the `SPEC.md` and `README.md` text.
- The release pass's finding and its container run:
  `~/temp/agent/release-pass-wrapper-alpha-46-2026-11-25/commit-push-lfs-lock-and-the-rest/keep/review.md` (H1 and
  "For the owner"), `askpass-proxy-experiment-run.sh`, `askpass-proxy-experiment-container-run.log`, and the OpenSSH
  sources it kept beside them.
- The third revision's logs, cited arm by arm:
  `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/run-1.log`, `run-2.log`, `run-3.log`
  and `keep/scripts/`.
- The second round's logs, cited arm by arm:
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
  `run5.log`, `run6.log`, `run9.log`, and its `probe-bin/` and `probe-scripts/`.
