# The next step a typed network command names at a new host key or a missing passphrase

## Status

A proposal,
put to the owner.
Nothing in it is approved or final:
this revision is unapproved and non-final,
and only the owner declares anything complete.
What this round did is one capped container, started three times
(the second corrected the first's missing `TERM`, the third added the shipped-shape capture and the revised
host-key sentence),
after the sixth round's seven container runs (eighteen container starts);
what it checked is named with every figure,
and what it could not check is under "What the probes did not show".

This text is the proposal as revised after the gate review kept at
`~/temp/agent/gate-review-host-key-next-step-sixth-revision-2026-11-25/keep/review.md`,
whose verdict was "not ready to put to the owner as written",
with one High finding,
eight Medium findings and a list of Low findings.
Every finding is answered under "What this revision changed",
each with the edit it caused or the measurement that showed it did not hold.
That review read the revision at `cfbfa7aaf6` (2026-11-25T17:20:11-05:00).

It designs the words a typed `git clone`,
`fetch`,
`pull` or non-LFS `push` can print when OpenSSH refuses a host key,
or when a key needs a passphrase nobody can be asked for.

The owner's two "Never" answers for B and E stay
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
and so does the rule against the wrapper starting itself
(`doc/planning/cli-git-no-self-invocation.md`).
`main` holds `SSH_ASKPASS=false` again
(`NO_ANSWER_PROGRAM` in `package/git-policy/cli/src/native/invocation/child_environment.rs:504`,
commit `c336355bcd`, 2026-11-25T13:21:59-05:00),
because the owner answered "Put `false` back (Recommended)"
(`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`).
That is the state every option here starts from.

Worktree:
`.claude/worktrees/host-key-next-step-design-seventh-revision`,
branch `docs/planning-host-key-next-step-design-seventh-revision`,
made from `main` at `4db44a2f1c`, merged with `main` at `69140e6791`
(2026-11-25T18:22:14-05:00) as `80e89d20e9`.
Round folder:
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/`,
whose `keep/` holds this round's container log and its script,
the launch line that names the container's limits,
`lib.sh`, the three stand-ins and the four sentence files.
A bare `keep/...` path anywhere else in this document means the sixth round's folder,
`~/temp/agent/host-key-next-step-design-sixth-revision-2026-11-25/keep/`,
whose log or file the sentence names with the arm it reads;
this round's own log is always named with its full folder path.
Only this file changed in the repository;
no code was changed,
and nothing under `doc/decision/` or `doc/handover/` was touched.

Every transcript is labelled,
because a read of the revision at `1f6f120f64` (2026-11-24T09:01:38-05:00)
found a round printing text its run had not printed:

- run means a program printed these bytes, and the log holding them is named with it;
- normalised means a program printed them, and the log line's own prefixes and the arm's own directory
  names were taken out so the block reads as the person would see it, with the arm named beside it;
- composed means no program printed them, and the text was put together from other facts;
- source means read from a file, at the tag or path named with it;
- unrun means neither.

## What the person sees today

These are the lines a person meets on `main`,
because `SSH_ASKPASS` holds `false` again (`NO_ANSWER_PROGRAM` in
`src/native/invocation/child_environment.rs`),
so the wrapper adds no line of its own at either refusal.
Measured in a capped container of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`, `OpenSSH_10.6p2`, `git 2.56.1`)
against a throwaway `sshd` on loopback,
with a fixture host key,
a fresh `known_hosts`,
throwaway client keys,
and the wrapper's two variables as `main` holds them.
The commands are copies under `/work`,
which no AppArmor profile attaches to,
so the arms show OpenSSH's own behaviour and not this host's confinement.

A new host key, for `git clone`
(run, `keep/run-11-the-words-as-revised.log`, arm `w-newkey-ask-direct-a-wrappers-value`;
normalised, the log's `err| ` prefixes and the arm's clone directory removed):

```text
# git clone ssh://target/work/served.git
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The command ends 128.
A key whose passphrase nobody can be asked for,
with the host key known,
ends 128 with `root@localhost: Permission denied (publickey).` between `Cloning into` and Git's `fatal:` line
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
arm `f-locked-direct-a-wrappers-value`;
the passphrase sentence this design uses is that round's file byte for byte).

A changed host key, at the default log level, ends 128 with OpenSSH's banner,
`Offending ED25519 key in /work/kh-stale:1`,
`Host key for [localhost]:2222 has changed and you have requested strict checking.`
and the same two Git lines
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
arm `f-changed-direct-a-wrappers-value`).
At `LogLevel=FATAL` the same refusal prints `Host key verification failed.` alone
(run, arm `f-changed-fatal-direct-a-wrappers-value`).

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

The owner's question waits in this document and in this round's report.
The handover entry it was expected to sit beside is gone:
`doc/handover/questions-for-the-owner.md` held one section for it,
"Words for a typed command at an unknown host key or a missing passphrase (the pass's L4)",
and commit `b53882be22` (2026-11-24T14:14:23-05:00, "docs(handover): L4 answered and moved to its decision record")
deleted its 26 lines.
The only handover that names the open question today is
`doc/handover/cli-git-rust-implementation.md:1224`
("the next-step delivery at a new host key (C, C2 or D of the host-key proposal) is the owner's next question
on that path, after `false` is back").
This round may edit neither file,
so the question is here and in this round's report,
and the coordinator adds it to `doc/handover/questions-for-the-owner.md` when it is put.

## The owner's question

This section stands alone:
the result in plain words,
one sentence per option,
what each shows a person,
which commands it covers,
what it makes the wrapper own,
what a signal sent to the typed process number does,
what it costs,
the ranking with a reason for every adjacent pair,
what each answer leaves,
and the second decision the ranking assumes.

### The result wanted, in plain words

At a new host key, and at a key no agent can unlock, a typed network command stops before any login and
tells the person nothing about the next step.
Every option here adds one paragraph after, or before, that refusal.
The differences are which refusal gets words,
whether the words fit what happened,
what the person gives up on the commands that meet no refusal,
and what the wrapper has to own to say it.

### What each option is, in one sentence

`A` changes nothing: the wrapper's process becomes Git, so no part of the wrapper is left to say anything.
`C` starts Git as the wrapper's child, reads its standard error through a pipe, and prints the sentence after the
child ends when the transcript reads as a refusal.
`K` does what C does but gives Git a pseudo-terminal on standard error instead of a pipe, so Git draws what a terminal
gets, and relays the terminal's bytes before printing the sentence.
`K2` is K when the wrapper's own standard error is a terminal and C's pipe when it is not, decided by one `isatty`
call.
`C2` starts Git as its child with the person's own streams untouched and, after exit 128, prints the sentence its
known-hosts check calls for.
`D` runs `ssh -G` and `ssh-keygen -F` before Git starts and prints the host-key sentence before the command when no
file holds a key and the setting is `ask`.

They are two decisions, and the ranking takes the family first and the terminal shape second: the family is a relay
(C, K or K2), a status check (C2), a pre-check (D) or nothing (A), and inside the relay the terminal shape is a pipe
(C), a pseudo-terminal (K) or the adaptive choice (K2).

### What each option shows a person at a new host key

Each block is a measured transcript from this round's log,
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/run-12-owner-question-arms.log`,
or, for A, the state on `main`;
`probe` stands where the arm's own clone directory stood,
and the sentence is cut short here and printed whole under "The words".
A block whose carriage returns were turned into line ends, or whose repeated progress updates were dropped, says so
in its label; no block here is composed.

A, nothing (run, arm `o-newkey-a-wrappers-value`):

```text
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.
```

C, the wrapper keeps Git as a child and reads a pipe
(run, arm `o-newkey-c-relay`;
normalised, the log's `err| ` prefixes and the arm's clone directory removed; the sentence is under "The words"):

```text
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ... run this command again.
```

K, at the same refusal (run, arm `o-newkey-k-pty`; normalised, the log's `err| ` prefixes removed):

```text
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ... run this command again.
```

The stand-in's default K writes seven carriage returns here against C's one (`o-newkey-k-pty` against
`o-newkey-c-relay`); a shipped K clears the terminal's output processing, and at a refusal its capture is then C's
byte for byte (sixth round, `refusal-k-pty-raw` against `refusal-c-relay`, identical).
The three print the same lines, and what K and K2 buy appears only on a command that draws a progress display.

### What K and K2 buy, and what each capture gets

At a terminal, a successful clone of two thousand objects, normalised: each carriage return starts a new line, the
`\033[K` erase-line prefix is dropped from the view, and the repeated percentage updates are dropped, so the block
shows the lines a person ends up reading.
Plain Git (run, arm `o-terminal-ok-a-wrappers-value`):

```text
Cloning into 'probe'...
remote: Enumerating objects: 2002, done.
remote: Counting objects: 100% (2002/2002), done.
remote: Compressing objects: 100% (2/2), done.
remote: Total 2002 (delta 0), reused 2002 (delta 0), pack-reused 0 (from 0)
Receiving objects: 100% (2002/2002), 84.30 KiB | 9.37 MiB/s, done.
```

C at the same terminal (run, arm `o-terminal-ok-c-relay`):

```text
Cloning into 'probe'...
```

K2 at the same terminal (run, arm `o-terminal-ok-k2-adaptive`): plain Git's lines, with 8618 bytes against plain
Git's 8617 and 216 carriage returns against 210, the six extra being the doubled line ends.

With standard error captured alone, the same clone gives plain Git 37 bytes and K2 37 bytes, identical byte for
byte (`o-capture-plain` and `o-capture-k2-adaptive`).
K without the output processing cleared gives 8592 bytes and 210 carriage returns (`o-capture-k-pty`); the shipped
shape, which clears it, gives 8586 bytes and 204 carriage returns (`o-capture-k-shipped`), and 107 escape bytes
either way.
Shown through `cat -A`, which prints a carriage return as `^M` and an escape as `^[`:

```text
# plain Git and K2, the whole capture (37 bytes)
Cloning into '/work/clones/probe'...$
```

```text
# K, the shipped shape, the first 320 bytes, the last line cut where the log cut it
Cloning into '/work/clones/probe'...$
^[[Kremote: Enumerating objects: 2002, done.$
^[[Kremote: Counting objects:   0% (1/2002)^M^[[Kremote: ...
```

C2, Git kept with the person's own streams, a sentence after exit 128
(run, arm `o-newkey-c2-check-unknown`):

```text
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.
...
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ... run this command again.
```

D, a check before Git starts
(run, arm `o-newkey-d-precheck`):

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ... run this command again.
Cloning into 'probe'...
Host key verification failed.
...
```

### Which commands each option covers

Measured on stand-ins, not on the wrapper.
`keep/stand-ins/standin.c` starts Git as its child and relays, and no arm traced the wrapper's own paths.
The wrapper has no single entry point for a typed network command, and a `pull` that merges runs its network step
itself as `git fetch --update-head-ok` through `run_wrapper_own_git` (`merge/pull_isolation.rs`), so a `pull`'s
fetch never passes the top-level forwarding arm.
The coverage in this list is what the stand-ins show; the paths it would reach in the wrapper were not traced end
to end.

- C, K and K2 cover `clone`, `fetch`, `pull` and a non-LFS `push`, because they read a refusal's transcript and
  every one of those ends non-zero: 128 for `clone`, `fetch` and a `push`, and 1 for a `pull` at a new host key
  (run, `keep/run-11-the-words-as-revised.log`, arms `w-pull-newkey-c-relay`, `w-pull-newkey-k-pty` and
  `w-pull-newkey-k2-adaptive`; and `keep/run-7-unreachable-host-and-pull.log`, arms `c-newkey-fetch-c-relay` 128
  and `c-newkey-push-c-relay` 128).
- C2 with its ranked gate, exit 128, covers `clone`, `fetch` and a `push`, and does not cover `pull`, because a
  `pull` at a new host key ends 1 (run, `keep/run-11-the-words-as-revised.log`, arms `w-pull-newkey-c2-status` and
  `w-pull-newkey-c2-check-unknown`, both exit 1 with no sentence; the `pull` status was first measured in
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
  arm `a-pull-newkey exit=1`).
  That gap is a gate choice and not a property of C2: a gate on any non-zero status would cover `pull`, but it would
  also speak after every other non-zero end (a rejected push, a merge conflict) whenever its check finds no key, and
  that variant is unrun, its effect an inference.
- D covers all four, because it speaks before the command whatever the command is (run, this round's log, arm
  `o-newkey-d-precheck`, and `keep/run-11-the-words-as-revised.log`, arm `w-pull-newkey-d-precheck`: exit 1, the
  sentence's line first and `Host key verification failed.` second, because a `pull` prints no `Cloning into`).
- A covers none.

The SPEC passage every child-keeping option amends is the one saying a plain forwarded `git fetch` or `git push`
replaces the wrapper process with Git, "so there is no \"after\" to look at"
(`package/git-policy/cli/SPEC.md:14821`): C, K, K2 and C2 have an after.
D adds a step before the handover and does not contradict that sentence, and A amends nothing.

### What each option makes the wrapper own

- C: the pipe, the read, the wait, the exit status and the signal ending, and the order of its own sentence after
  Git's output.
- K: C's list, plus the pseudo-terminal's life, the size it is given at the start, and its end after the child ends.
- K2: K's list in both shapes, and the one `isatty` call that chooses between them.
- C2: the child and the wait, the exit status and the signal ending, the sentence after exit 128, and on the failure
  path the check's two starts.
- D: the check's two starts before every command, and the sentence before the command.
- A: nothing.

### What a signal sent to the typed process number does

Today a typed command is Git: the wrapper replaces its own process, so `kill` of the typed process number, a
`timeout` or a harness stop reaches Git.
Every option that keeps a child changes that:

- C, K and K2: a terminal's control character signals the whole foreground group and still reaches Git, and a signal
  sent to the wrapper's own process number reaches the wrapper and Git only if the wrapper publishes the child to
  the relay it already has.
  That relay's header says what it is for: "a signal a process sends to the wrapper reaches that child until the
  token is dropped" (`package/git-policy/cli/src/native/process/signal_relay_relayed_child.rs`).
  The design uses it; no arm sent such a signal, so this paragraph is source and design, not measurement.
- C2: the wrapper starts Git with inherited streams and waits (`run_real_git` in `invocation/forwarding.rs`), which
  has no signal relay today, so a signal to the wrapper's own process number reaches the wrapper alone unless C2
  adds one; a terminal's control character still reaches Git because the child stays in the foreground group.
- D: a signal during the check reaches the wrapper and ends the check, and a signal during the command reaches Git,
  because the wrapper has replaced itself by then.
- A: everything reaches Git, as today.

### What each option costs, including the read of the remote

The figures are medians of per-round differences against a control measured in the same rounds,
with the range and the count of positive rounds
(`keep/derived-medians.txt`, from
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-3-steps.log`;
the typed-path figures are from `keep/run-10-typed-path-with-every-twin.log`).
The last term of each line is the read of the remote,
`+0.85 ms` for a named remote
(`git remote get-url`, range `-0.59` to `+1.50`, 23 of 24),
which the second question under "The owner's question" either pays on every command or pays only after a failure.

- K2: `+0.74 ms` for the relay, range `-1.38` to `+2.52`, 20 of 24,
  plus `+0.85 ms` if the remote is read first, so about `1.6 ms` on every typed network command.
  On the typed path it is `+5.44 ms` against a floor whose own twin differs by `-8.59 ms`,
  inside this harness's noise.
- K: `+0.85 ms`, range `-2.73` to `+2.61`, 20 of 24, plus the same `+0.85 ms`, so about `1.7 ms`;
  against C in the same rounds it is `+0.16 ms`, range `-1.68` to `+1.80`, 13 of 24,
  which this harness cannot tell from no difference.
- C: `+0.68 ms`, range `-1.40` to `+2.71`, 18 of 24, plus the same `+0.85 ms`, so about `1.5 ms`.
- C2: `+0.68 ms` for the fork and the wait, range `-0.07` to `+3.75`, 23 of 24,
  and `+0.85 ms` plus the check's two starts (`+7.50 ms`, range `+5.23` to `+10.55`, 24 of 24)
  only on the failure path.
  On the typed path it is `+4.44 ms`, 15 of 24, inside the noise.
- D: `+7.50 ms` for the check and the same `+0.85 ms` for the host when a remote is named, so about `8.4 ms`
  on every typed network command; on the typed path it is `+20.08 ms`, range `-12.06` to `+76.25`, 22 of 24,
  against a twin that differs by `-6.64 ms`, and it is the only option either typed-path harness resolves
  (the typed-path run of the round in
  `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/` measured `+5.76 ms`, 23 of 24).
- A: nothing, and no control can measure nothing.

On a typed command these sums are small against a network round trip,
and the reasoning about them is under "What the figures support, and what they do not".
These are runtime figures only; what each option would take to build is under "What building each option would
take", which is a cost too and not a ranking reason.

### The ranking, with a reason for every adjacent pair

**K2 > K > C > C2 > D > A.**
K2 is the recommendation,
which is what "(Recommended)" would mark in the question tool,
and the owner is free to pick any of the six.

- K2 over K.
  Both give the same words and keep the progress display at a terminal.
  When the person's standard error is not a terminal, plain Git draws nothing, and K2's pipe presents exactly
  that: measured, its captured transcript is plain Git's, byte for byte, 37 bytes with no carriage returns and no
  escapes, where the shipped K's is 8586 bytes with 204 carriage returns and 107 escapes
  (run, this round's log, arms `o-capture-plain`, `o-capture-k2-adaptive` and `o-capture-k-shipped`).
  Shown through `cat -A`, K2's whole capture is `Cloning into '/work/clones/probe'...$`, and K's is 8586 bytes of
  `^[[Kremote: ...` progress, each update ending `^M` except the last update of a line, which ends `$`, with an
  `^[[K` before every relayed line: that is the change no flag removes.
  A refusal, where Git draws nothing, is C's bytes under a shipped K (sixth round, `refusal-k-pty-raw` and
  `refusal-c-relay`, identical), so the change needs a command that draws something.
  K2 costs `+0.74 ms` where K costs `+0.85 ms`, a difference this harness does not resolve, so cost decides
  nothing here; what decides it is that K's captured-stream change falls on every agent session, hook, CI step
  and script that captures a typed command's output, and K2's falls on no one.
- K over C.
  At a terminal, the place where a person watches a clone and meets a security key's touch notice,
  K behaves as plain Git does:
  measured, plain Git and K each draw the progress lines and C draws none
  (run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
  the `TERMINAL` lines; and this round's log, arms `o-terminal-ok-*`),
  and OpenSSH writes the touch notice to standard error only when that stream is a terminal
  (`readpass.c:252`, `:253` and `:279`, source only).
  C also loses Git's colour: on a failed push at a terminal plain Git writes five escape bytes (one `\033[K` and
  four colour sequences on the rejected-ref and `error:` lines) and C writes none, while K writes the same five as
  plain Git (run, `keep/run-5-remote-line-under-a-terminal.log`, arms `r-fail-plain-terminal`,
  `r-fail-c-relay-terminal` and `r-fail-k-pty-terminal`).
  C's display comes back with `--progress`, and K's captured bytes come back with `--no-progress` once the
  terminal's output processing is cleared (measured: `redirect-k-pty-raw-no-progress` is 37 bytes, identical to
  plain Git's), so a flag the person types recovers each of those two in part.
  Two of K's four effects survive every flag: the escape `\033[K` in front of a relayed line stays with
  `--no-progress` and the processing cleared (run, `keep/run-9-check-without-a-source-and-no-progress.log`,
  arm `n-fail-k-pty-raw-no-progress`, five escape bytes, one `\033[K` and four colour sequences, and the relayed
  line printed as `\033[Kremote: ...`),
  and every descendant's own answer to `isatty(2)` stays too.
  So neither stream is purely recoverable: C loses the display, the touch notice and Git's colour at a terminal, K
  adds a carriage return per line, an escape per relayed line and a terminal answer for every descendant on a
  capture.
  A reader who weighs captured logs over terminal displays would put C first; this document weighs the person at
  the terminal, because the refusal is met there and the capture is a machine's log.
- C over C2.
  C's words are chosen from what OpenSSH printed, so they fit the refusal: measured, C is silent at a repository
  that is not there and speaks at a key nobody can unlock
  (run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
  arms `f-known-missing-direct-c-relay` and `f-locked-direct-c-relay`).
  C2's ranked shape reads a status and a known-hosts check, and it is silent at that same locked key,
  the refusal whose words the person most needs (measured, arm `f-locked-direct-c2-check-unknown`).
  It is also silent for `pull` and, at an unreachable host, it speaks wrongly (both measured in the sixth round's
  logs and listed under "C2").
  What C2 keeps over C is real:
  it holds the person's own streams, so its captured bytes are plain Git's and it loses neither the progress
  display nor the touch notice; and it survives `ssh -q` or `LogLevel=QUIET`, where C reads lines OpenSSH has
  stopped printing (`git clone -q` is not that word: it leaves C's line, measured this round, arms
  `o-quiet-clone-c` and `o-quiet-ssh-c`).
  Both pay one fork and one wait (`+0.68 ms` each in the same rounds).
  The person trades words that fit the refusal for a `ssh -q` transcript and an untouched terminal,
  and this document takes the words.
- C2 over D.
  C2's check runs only after a failure, where D's two starts run before every command and its sentence can
  precede a command that then fails for another reason or succeeds.
  Both were measured:
  D speaks before a clone that ends 0 when its check has no source for the person's ssh options
  (run, `keep/run-9-check-without-a-source-and-no-progress.log`, arm `s-known-no-source-d-precheck`),
  and C2's ranked shape speaks the host-key sentence on a connection refused
  (run, `keep/run-11-the-words-as-revised.log`, arm `w-never-seen-c2-check-unknown`).
  What favours the lower option is `pull`: D covers it and C2's ranked gate does not
  (`keep/run-11-the-words-as-revised.log`, `w-pull-newkey-d-precheck` against `w-pull-newkey-c2-check-unknown`).
  So the older claim that C2's shape never announces a refusal that did not happen is withdrawn:
  both can be wrong, and the difference is when and how often.
  C2 is wrong only after a 128 with no key known to its check, D is wrong before every command whose
  configuration its check cannot read, and D can also be wrong about a command that succeeds.
  On cost, C2 is `+0.68 ms` on every command and pays the check only after a failure, where D is about `8.4 ms`
  on every command and is the only option a typed-path harness resolves.
  `accept-new` is no longer part of this pair: D as ranked prints only for `ask`, so it is silent on a clone with
  `accept-new` (run,
  `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
  arm `f-accept-new-direct-d-precheck`, decision `none`),
  and the sentence the design at `79962f9ccc` printed there belongs to the arm `d-precheck-acceptnew`.
- D over A.
  D speaks for the plain new-key case, measured, and its words arrive before the refusal.
  A never speaks, and at the first contact with every host the person is left with no next step.
  D pays with two extra process starts on every typed network command, with a sentence that can be wrong about why
  the command failed, and with a sentence before a clone that then succeeds when its check cannot read the person's
  options (run, `keep/run-9-check-without-a-source-and-no-progress.log`, arm `s-known-no-source-d-precheck`, where
  the clone ends 0); A pays nothing and says nothing.

### What each answer leaves

- K2: the words arrive at both refusals OpenSSH prints, name the host as the person typed it,
  and nothing changes in what a captured command writes: measured for standard error captured alone (37 bytes,
  plain Git's) and for both streams merged into one file (a `git pull` of 245 bytes, identical to plain Git's;
  `o-merged-plain` and `o-merged-k2`).
  At a terminal the person keeps the progress display, Git's colour and the touch notice.
  A `ssh -q` or `LogLevel=QUIET` command still gets nothing, and a typed LFS push needs the double-sentence fix.
- K: the same words, and every captured stream gains Git's progress bytes, a carriage return per line, an escape per
  relayed line, Git's colour and a terminal answer for every descendant.
- C: the same words, plain Git's bytes in every capture, and no progress display, no colour and no touch notice at a
  terminal unless the person passes `--progress`.
- C2: `ssh -q` still works and nothing is lost on a successful command;
  a key nobody can unlock, a `pull` under its ranked gate, and any 128 whose check reads the wrong configuration
  still get no words or the wrong ones.
- D: advice before the command fails, one of the two refusals never covered, a sentence that can precede a clone that
  succeeds when the check cannot read the person's options, and two extra process starts on every typed network
  command.
- A: nothing changes, and a person at a new host key or with a key no agent holds still reads OpenSSH's line and
  Git's `fatal:` with no next step.

The host-key sentences offer an `ssh` command as the way forward.
That works when the person's options are in the configuration plain `ssh` reads; when they come from
`GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH`, the advised `ssh` reads other settings, adds the key to another
file, and the typed command is refused again: measured this round, with `-F` standing in for the person's
`~/.ssh/config`, the advised `ssh` accepted the key into the default `known_hosts` (one line) while the typed clone,
whose known-hosts file was named by `GIT_SSH_COMMAND`, still ended 128 (`o-advice-gsc-ssh` and
`o-advice-gsc-clone-after`); the same `ssh` with the person's own `-o UserKnownHostsFile=<their file>` accepted the
key there and the clone then ended 0 (`o-advice-gsc-ssh-with-option` and `o-advice-gsc-clone-after-option`).

### The second question: does the wrapper read the remote first

The ranking under "The owner's question" assumes the wrapper reads the remote before it decides whether to relay,
or relays every transport.
That is a separate decision.
It changes C, K and K2; for C2, D and A it changes nothing.

- Read the remote first, and relay only a remote ssh carries.
  For a `clone` the URL is in the command's own arguments, and the read is `git ls-remote --get-url`,
  which applies `url.<base>.insteadOf` rewriting (measured, `keep/probe-insteadof.log`: a remote
  `gh:served.git` prints as `ssh://target/work/served.git` through both `git remote get-url` and
  `git ls-remote --get-url`).
  For a named remote it is one start, `+0.85 ms` on every typed network command.
  For a typed `push` it must be `git remote get-url --push`, because a `pushurl` or a `pushInsteadOf` can send a
  push elsewhere and the read without `--push` does not return that URL
  (`doc/planning/cli-git-no-self-invocation.md`, the note naming the push URL).
  An `https` or `file` clone keeps its process replacement, its bytes and its signals.
- Relay every typed network command whatever the transport.
  Nothing is read on the happy path, and the host for the sentence is read after a failure,
  which costs the same `+0.85 ms` only where a sentence is printed.
  Every non-ssh command pays the fork and the wait (between `+0.68` and `+0.85 ms` by shape), loses its process
  replacement, and so no longer gives a signal sent to the typed process number straight to the transport; at a
  terminal K and K2 also put a pseudo-terminal on an `https` or `file` clone's standard error, changing what its
  descendants answer to `isatty(2)` and what a capture holds, where no ssh refusal can occur.

What the first answer makes of this:

- K2, K or C: read-first adds one Git start to every ssh command and keeps every other transport exactly as it is;
  relay-everything saves that start on ssh commands and spends the fork, the wait, the process replacement and, at a
  terminal, the pseudo-terminal on every other transport.
- C2: moot for relaying, because C2 relays nothing; a transport read would add `+0.85 ms` to every command and change
  nothing C2 prints, because its check already runs only after exit 128.
- D: moot, because D reads the host before every command already.
- A: moot, because A adds nothing.

Ranking: read the remote first, over relaying everything.
Read-first confines the change to commands ssh carries, while relaying everything changes the bytes, the process and
at a terminal the terminal answer of commands that can meet no such refusal, which is the cost K already carries and
K2 exists to avoid.
The owner's recorded "Spawn reduction: save as many processes as possible"
(`doc/planning/cli-git-spawn-and-sync-reduction.md`, "Decisions (user)") bears on both: read-first adds one Git
start to every ssh command, and relay-everything adds a fork and a wait to every non-ssh command.
The read may not need a start: the wrapper already reads `url.*.insteadof` and `url.*.pushinsteadof` in process for
the LFS plan (`lfs/lfs_upload_plan.rs`, `url_aliases`, and `repo_read/repo_read_query_refs.rs`, which refuses the
snapshot path when a rewrite exists), so a clone URL's rewrite can be applied in process; a named remote still pays
one `remote get-url` for its URL (`doc/planning/cli-git-no-self-invocation.md`, the note beginning
"Not changed: the interactive upload").
That in-process read is source and inference, not measured, and the spawn decision was recorded for the commit path,
so its reading here is this document's.

### The composition this document does not offer

K2 and C2's check are complements in one sense: a K2 whose transcript is empty under `ssh -q` could fall back on
the check.
This document does not offer that combination, because the check's own failure is measured:
at a connection refused for a host never seen it speaks the host-key sentence
(run, `keep/run-11-the-words-as-revised.log`, arm `w-never-seen-c2-check-unknown`),
and under `ssh -q` that is exactly the transcript K2 would have to fall back from,
so the combination would answer "no words" with the wrong words.
The owner may ask for it all the same;
it would need the check narrowed to the refusals it can tell apart, which is the work C2 already does not do.

## What this revision changed

The gate review's findings at
`~/temp/agent/gate-review-host-key-next-step-sixth-revision-2026-11-25/keep/review.md`,
and what each one became.
Its verdict was "not ready to put to the owner as written": one High, eight Mediums and a list of Lows.
The arms are in this round's log,
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/run-12-owner-question-arms.log`;
a bare `keep/...` path is the sixth round's folder, as "Status" says.

### The High finding

- High 1, that the owner's question never said what K and K2 are and printed a transcript no command can print.
  A, C, K, K2, C2 and D now have one sentence each before the transcripts.
  The composed K and K2 block is gone: K's refusal is measured (`o-newkey-k-pty`), the section shows a successful
  clone at a terminal under plain Git, C and K2 (`o-terminal-ok-*`), and a `cat -A` of K's capture against plain
  Git's and K2's (`o-capture-plain`, `o-capture-k2-adaptive`, `o-capture-k-shipped`).
  D's block ends with an ellipsis.
  The K2-over-K reason cites the `cat -A` example as well as the counts.

### The Medium findings

- Medium 1, that the second question's reason held for K and not for K2.
  "The second question" now states what read-first and relay-everything do for each first answer (C, K, K2, C2, D
  and A), says the choice is moot for C2, D and A, cites the owner's "Spawn reduction" decision, names the
  in-process `insteadOf` read the LFS plan already makes and that a named remote still pays one `remote get-url`,
  and names `git remote get-url --push` for a typed `push`.
- Medium 2, that the signal design contradicted its own precedent and the owner's section was silent.
  The owner's section has "What a signal sent to the typed process number does", per option, citing
  `process/signal_relay_relayed_child.rs`; the false claim that the relay is Git's parent is gone; the C2-over-C
  reason no longer says C2 "keeps the terminal's signals"; C2's pros no longer claim signals, and its cons say
  `run_real_git` has no relay.
- Medium 3, that `-q` was undefined and C2's advantage measured on `ssh -q`.
  Every "`-q`" now says `ssh -q` or `LogLevel=QUIET`; `git clone -q` is measured separately and still leaves C's
  line (`o-quiet-clone-c`); C2's `ssh -q` survival is measured (`o-quiet-ssh-c2`).
  C2's advantage is no longer stated as if `git -q` were the same word.
- Medium 4, that K2's "nothing changes in a capture" was measured with standard error alone.
  A `git pull` with both streams merged into one file is measured: 245 bytes, identical to plain Git's
  (`o-merged-plain`, `o-merged-k2`).
  The K2 section says what that one arm does and does not show.
- Medium 5, that the advice was measured with `-F` and the limit stated only for D and C2.
  The advice is measured with the person's known-hosts file coming from `GIT_SSH_COMMAND`: plain `ssh` accepts the
  key into the default `known_hosts` and the typed clone is still refused, and the same `ssh` with the option
  works (`o-advice-gsc-*`).
  The limit is in "The words" and in "What each answer leaves".
- Medium 6, that C and K were mis-described at a terminal.
  C's cons now say C loses Git's colour, five escapes against none (`r-fail-plain-terminal`,
  `r-fail-c-relay-terminal`); K's effects say the five escape bytes are one `\033[K` and four colour sequences, so
  a K capture gains colour too; and the `sideband.c` citation is the assignment at `:316` to `:318`.
- Medium 7, that two pair reasons omitted what favours the lower option and one gap was presented as intrinsic.
  C2 over D now says D covers `pull` and C2's ranked gate does not; D over A now says D can print before a clone
  that succeeds; C2's `pull` gap is a gate choice with the non-zero-status variant named, unrun and inference.
- Medium 8, that the section promised "what the wrapper has to own" and that coverage read as traced.
  The section has "What each option makes the wrapper own"; "Which commands each option covers" says the coverage
  is measured on stand-ins, names `run_wrapper_own_git`'s untraced `pull` path, and names the SPEC passage at
  `package/git-policy/cli/SPEC.md:14821` that C, K, K2 and C2 amend.

### The Low findings

One pass, each Low and its change:

- the D `pull` arm's first line is the sentence, not `Cloning into`: fixed where the four commands are listed.
- "the same log" in the owner's C2 and D blocks: each block names its arm, and the block intro names the log.
- the change log's nonexistent folder: this change log replaces that text.
- `KnownHostsCommand` cited to the after-its-review round: now
  `typed-network-command-next-step-third-revision-2026-11-24/keep/run-1.log:280-290`.
- `c-relay` at a host never seen cited to run-11: now `keep/run-7-unreachable-host-and-pull.log`,
  `u-never-seen-c-relay`.
- `c2-status` and `c2-check-known` at strict checking and `BatchMode` cited to run-11: now the arms in
  `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
  `f-strict-yes-direct-*` and `f-batchmode-direct-*`, named where each family is.
- `STANDIN_STRIP_ANSI=0` said to make C speak: C's pipe carries no escape and stays silent; only
  `STANDIN_DROP_REMOTE=0` makes it speak.
  K2's strip control is now measured (`o-remote-fail-terminal-k2-adaptive-*`).
- K2's strip rule called inference: now K2's own with-strip and no-strip arms.
- C and K at H1's case "directly and behind both proxies" cited to run-6: run-6 holds the `-J` arms, and the direct
  and `ProxyCommand` arms are `after-its-gate-review-2026-11-25/keep/run-1-behaviour.log:531-639`.
- `run-3.log:55` for the `File name too long` line: now `:61`.
- "both of its failures" listing three, and D's "two of the three classes": reworded to say what each count is.
- three sentences "printed ... in the arms this round ran": the host-key sentence is run-12's, and strict and
  passphrase are run-11's unchanged files.
- K's 8593 bytes and 210 carriage returns called the shape: now "the stand-in with the output processing left on",
  with 8587 and 204 as the shipped shape, and the owner's section quotes `o-capture-k-shipped`.
- "quoted whole" omitting the owner's two further sentences: both are quoted under "The rule that forbids the
  question".
- the terminal calls said to come from `libc` "already a direct dependency": now the owner's `rustix` choice, its
  pinned features, and rustix 1.1.6's `pty` and `termios` features named as a pick for the owner.
- `SPEC.md:9460`, `README.md:5787` and `:7597`: the sentences are cited, not the lines.
- `sideband.c:311-316`: the assignment is `:316` to `:318`.
- "seven capped containers": seven container runs, eighteen container starts, and this round's launch line kept.
- "this round", "the first round" and "the round before it": each now names the folder, commit or timestamp it
  means.
- the six options mixing two axes: the owner's section says they are two decisions, the family first and the
  terminal shape second.
- the host-key sentence's "outdated" and its removal advice: the sentence now says to compare the fingerprint
  before accepting a changed key, and "The words" says why.

No finding was answered by saying it did not hold.

## The case every option is tried against

Every option in this document was run at the case H1 names,
and at the refusals a next step is meant for,
in capped containers with a real `sshd` and the OpenSSH this image carries.
Run (`keep/run-11-the-words-as-revised.log`, and for the mechanisms named beside each family the log that owns
that measurement; the stand-ins are `keep/stand-ins/standin.c`, `keep/stand-ins/proxy.c` and
`keep/stand-ins/ptyrun.c`).

Container:
`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`, `--rm`, `--network=none`,
`--cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
started in a `systemd-run --user` scope with `--slice=agent-work.slice`, `RuntimeMaxSec`,
`CPUQuota=300%`, `MemoryMax=5G`, `TasksMax=512` and `IOWeight=10`.
OpenSSH 10.6p2, Git 2.56.1, `sshd` on 127.0.0.1:2222 with a fixture host key.

### The setup a person would have

- two client keys in `authorized_keys`: `key_locked`, whose passphrase is nobody's to give, listed first,
  and `key_good`, which works;
- a served repository at `ssh://target/work/served.git`,
  where `target` resolves to `localhost:2222` and offers the two keys in that order with `IdentitiesOnly yes`;
- a relay `ProxyCommand` (a compiled program, `keep/stand-ins/proxy.c`), and a `ProxyJump` through
  `root@127.0.0.1:2222`;
  the jump host is `127.0.0.1` and the destination `localhost`, because `ssh` refuses a jump whose host, port and
  user equal the destination's (`ssh.c:1289-1291` at `V_10_6_P2`, source);
- the wrapper's two variables as `main` holds them:
  `SSH_ASKPASS=false` and `SSH_ASKPASS_REQUIRE=force`;
- an `ssh` at `/work/bin/ssh`, a copy of the image's own, for the reason under
  "What the probes did not show".

### The twelve families, and the transport each ran

The families differ only in what a person's settings and the server make true.

- `f-newkey-ask`: the host key is unknown and `StrictHostKeyChecking` is `ask`.
  Direct, behind the `ProxyCommand`, and behind `-J`.
- `f-locked`: the host key is known and only the locked key is offered.
  Direct, behind the `ProxyCommand`, and behind `-J`.
- `f-noask`: strict checking off and no known-hosts file at all,
  with the locked key offered before the working one.
  This is the case that broke J. Direct, behind the `ProxyCommand`, and behind `-J`.
- `f-ask-known`: the host key is known and the setting is `ask`,
  with the locked key offered before the working one:
  the same keys as `f-noask`, with the host key known, so D's decision is exercised.
  Direct only.
- `f-known-missing`: the host key is known and the repository is not there:
  a failure that is not about a key at all. Direct only.
- `f-strict-yes`: the host key is unknown and `StrictHostKeyChecking` is `yes`,
  so OpenSSH prints its own strict sentence and asks nothing. Direct only.
- `f-batchmode`: the host key is unknown and `BatchMode` is `yes`, so nothing is asked. Direct only.
- `f-accept-new`: the host key is unknown and `StrictHostKeyChecking` is `accept-new`,
  so the clone succeeds and the key is added. Direct only.
- `f-changed`: the file holds a different key for the destination. Direct only.
- `f-changed-fatal`: the same, at `LogLevel FATAL`, where OpenSSH prints the last line alone. Direct only.
- `f-remote-ok`: a push whose server-side hook writes `Host key verification failed.` on its standard error,
  which Git relays as a `remote: ` line, after which the push succeeds.
  Run in a pipe and on a terminal, with `TERM=xterm-256color`.
- `f-remote-fail`: the same line on a push the same hook then refuses.
  Run in a pipe and on a terminal, with `TERM=xterm-256color`.

Two more cases were added in the sixth round, because the review of the fifth revision found them unrun:
a host never seen at a port nothing listens on (`w-never-seen-*`), and `git pull` at a new host key
(`w-pull-newkey-*`, and the `fetch`, `pull` and `push` arms of `keep/run-7-unreachable-host-and-pull.log`).

### The ten arms

Each family ran the same arms:

- `a-wrappers-value`: `SSH_ASKPASS=false` and nothing else, which is `main`;
- `c-relay`: C, a pipe on the child's standard error, read after the child ends;
- `k-pty`: K, a pseudo-terminal on the child's standard error;
- `k2-adaptive`: K2, the same with a pseudo-terminal only when the wrapper's own standard error is a terminal;
- `c2-status`: C2 with the exit status alone;
- `c2-check-unknown`: C2 whose check speaks the host-key sentence only when no file holds a key;
- `c2-check-known`: the same, speaking the passphrase sentence only when a file does;
- `c2-check-both`: the same, choosing between the two on what the check found;
- `d-precheck`: D, `ssh -G` and `ssh-keygen -F` before Git starts,
  printing only when the resolved setting is `ask` and no file holds a key;
- `d-precheck-acceptnew`: the same shape as the revision at `79962f9ccc` had it,
  where `accept-new` counts as asking, which is the false positive the review predicted.

### What the arms are not

- They are stand-ins, not the wrapper.
  `keep/stand-ins/standin.c` is one binary with a mode per option: it forks, relays on a pipe or a pseudo-terminal,
  waits, and prints a sentence file's text when its condition holds.
  It reads no repository and applies no policy.
  The host in the sentences is the fixture's, `target`,
  and not filled from a transcript, because there is nothing in these transcripts to fill it from.
- The stand-in's reader is a port of the shipped detector, not a substring test.
  It requires the last line, the strict sentence, or the wrapper's own sentence as the only host-key-naming lines,
  so it is silent at a changed key and speaks at `BatchMode=yes`,
  exactly as `reads_as_unknown_host_key` does.
  The round whose folder is `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/` used a
  substring test, and so its arms spoke over a changed key where the shipped reader is silent.
- The stand-in speaks only when the child ended non-zero,
  it drops the lines Git prefixes with `remote: ` before it reads,
  and it strips a leading `\033[K` from a line before testing for that prefix
  (`STANDIN_STRIP_ANSI=0` turns the strip off, which is the red control).
  All three are what a shipped C, K or K2 needs, and the controls are under "The measurements".
- D's arm runs `ssh -G` and `ssh-keygen -F` for real.
  Its check reads `stricthostkeychecking` and prints only for `ask`,
  so `accept-new` gets no sentence from the fixed arm.
  Its options come from `STANDIN_SSH_OPTS`, which is a stand-in for the source a shipped wrapper would have to
  find; the arms without that source are under "What each option gave, family by family".
- No arm has `Match exec`, `CanonicalizeHostname`, `KnownHostsCommand`, `HostKeyAlias` or `VerifyHostKeyDNS` set.
  The round whose folder is `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/`
  measured `KnownHostsCommand` (which makes D speak while the clone succeeds) and `@cert-authority` (which does
  not); those results stand and are quoted in D's section.
  The round whose folder is `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/` measured
  `Match exec`, which is cited under D's costs.
- No arm ran with a signal sent to the wrapper alone, and none ran on Windows.
- The sentence's new fingerprint clause is prose and no arm exercises a person reading a fingerprint; what the arms
  show is the acceptance path it sits in (`o-advice-gsc-*`, and the sixth round's `keep/run-8-...`).

## What each option gave, family by family

Run (`keep/run-11-the-words-as-revised.log` for the fixture host at three transports and for `pull`,
`keep/run-5-remote-line-under-a-terminal.log` for the remote-written line at a terminal,
`keep/run-6-j-arms-and-transports.log` for J's value and for `-J`,
`keep/run-7-unreachable-host-and-pull.log` for the host never seen and for `fetch` and `push`,
`keep/run-9-check-without-a-source-and-no-progress.log` for a check with no source).
The comments on the arm lines are normalised.

At a new host key, directly and behind both proxies:

- `a-wrappers-value`: exit 128, OpenSSH's line, Git's lines, and no next step;
- `c-relay`, `k-pty`, `k2-adaptive`: exit 128, the same lines, then the host-key sentence;
- `c2-status`, `c2-check-unknown`, `c2-check-both`: exit 128, then the host-key sentence,
  which is right here;
- `c2-check-known`: exit 128 and nothing, because the check finds no key and this shape speaks only when it finds one;
- `d-precheck`: exit 128, the host-key sentence **before** `Cloning into`, then OpenSSH's and Git's lines.

At a key nobody can unlock, with the host key known, directly and behind the proxy
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`):

- `a-wrappers-value`: exit 128, `root@localhost: Permission denied (publickey).`, Git's lines, nothing else;
- `c-relay`, `k-pty`, `k2-adaptive`: exit 128, then the passphrase sentence, which fits what the transcript says;
- `c2-status`: exit 128, then the **host-key** sentence, which is wrong here;
- `c2-check-unknown`: silent, so the refusal the person needs words for gets none;
- `c2-check-known`, `c2-check-both`: exit 128, then the passphrase sentence, which is right here;
- `d-precheck`: silent, because the key is known and D never sees a passphrase.

At `f-noask`, the case that broke J, in every transport
(run, `keep/run-6-j-arms-and-transports.log`):

- every option ends 0 and the clone completes;
- `k-pty` writes 423 bytes behind `-J` where the others write 57 to 70, which is the progress display plus nine
  carriage returns;
- no option prints a sentence, because no refusal reached it;
- A's value ends 0 as well, and J's value ends 0 directly, 128 behind the `ProxyCommand` and 128 behind `-J`.

At a repository that is not there with the host key known:

- `a-wrappers-value`, `c-relay`, `k-pty`, `k2-adaptive`, `d-precheck`: silent, which is right;
- `c2-status`: exit 128, Git's lines, then the **host-key** sentence, which is wrong;
- `c2-check-unknown`: silent, which is right here;
- `c2-check-known`, `c2-check-both`: exit 128, Git's lines, then the **passphrase** sentence, which is wrong:
  no key was involved.

At `StrictHostKeyChecking=yes` (run, `keep/run-11-the-words-as-revised.log`; `c2-status` and `c2-check-known` are
`f-strict-yes-direct-c2-status` and `-c2-check-known` in
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`):

- `a-wrappers-value`: OpenSSH's strict sentence and the last line;
- `c-relay`, `k-pty`, `k2-adaptive`: the strict sentence, whose advice overrides their own setting for one
  connection;
- `c2-status`, `c2-check-unknown`, `c2-check-both`: the host-key sentence, whose advice plain `ssh target` cannot
  carry out under that setting;
- `c2-check-known`: silent;
- `d-precheck`: silent, and right, because no question will be asked at all.

At `BatchMode=yes` (run, `keep/run-11-the-words-as-revised.log`; `c2-status` and `c2-check-known` are
`f-batchmode-direct-c2-status` and `-c2-check-known` in
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`):

- `a-wrappers-value`: the last line alone;
- `c-relay`, `k-pty`, `k2-adaptive`, `c2-status`, `c2-check-unknown`, `c2-check-both`: the host-key sentence,
  whose advice cannot ask either, because `BatchMode` is in the configuration plain `ssh` reads;
- `c2-check-known`: silent;
- `d-precheck`: the host-key sentence before Git starts.

At `accept-new` (run, the log named with family `f-accept-new` under "The case every option is tried against"):

- every arm ends 0 and the clone succeeds;
- `d-precheck` stays silent, because the setting is not `ask`;
- `d-precheck-acceptnew` prints the host-key sentence on a clone that then succeeds,
  which is the false positive the design at `79962f9ccc` had with its `ask` or `accept-new` condition.

At a changed key, at the default log level:

- `a-wrappers-value`: OpenSSH's banner, which names the offending line;
- `c-relay`, `k-pty`, `k2-adaptive`, `d-precheck`, `c2-check-unknown`: silent, which the detector's positive
  reading requires and which leaves the person OpenSSH's own words about the key that changed;
- `c2-status`: the host-key sentence, which is wrong: the machine knows a key for that host;
- `c2-check-known`, `c2-check-both`: the passphrase sentence, which is wrong.

At the same changed key with `LogLevel=FATAL` (run, `keep/run-11-the-words-as-revised.log`):

- `a-wrappers-value`: `Host key verification failed.` alone;
- `c-relay`, `k-pty`, `k2-adaptive`: the host-key sentence, which the detector cannot tell from an unknown key's
  transcript;
- `c2-check-unknown`: the host-key sentence when its check has no source for the person's settings
  (run, `keep/run-9-check-without-a-source-and-no-progress.log`, arm `s-changed-fatal-no-source-c2-check-unknown`),
  and silence when it has
  (run, `keep/run-11-the-words-as-revised.log`, arm `w-changed-fatal-direct-c2-check-unknown`);
- `c2-check-known`, `c2-check-both`: the passphrase sentence, which is wrong.

At a host never seen, whose refusal is a connection refused
(run, `keep/run-11-the-words-as-revised.log`, arms `w-never-seen-*`):

- `a-wrappers-value`: exit 128 with `ssh: connect to host 127.0.0.1 port 2299: Connection refused`;
- `c-relay`: exit 128 and silent, which is right
  (run, `keep/run-7-unreachable-host-and-pull.log`, arm `u-never-seen-c-relay`; the sixth round's `w-never-seen-*`
  arms have no `c-relay` at this case, and K and K2 were not run at it);
- `c2-status`, `c2-check-unknown`: the host-key sentence, which is wrong: no key was involved;
- `c2-check-known`: silent;
- `d-precheck`: the host-key sentence before the command, which is wrong for the same reason.

At `git pull` at a new host key (run, `keep/run-11-the-words-as-revised.log`, arms `w-pull-newkey-*`):

- every arm ends 1, not 128;
- `c-relay`, `k-pty`, `k2-adaptive`, `d-precheck`: the host-key sentence;
- `c2-status`, `c2-check-unknown`: silent, so C2's ranked shape says nothing for one of the four commands this
  design covers.

At the remote-written line (run, `keep/run-5-remote-line-under-a-terminal.log` and
`keep/run-11-the-words-as-revised.log`):

- `f-remote-ok`: exit 0, the transcript holds `remote: Host key verification failed.`, and no arm prints a
  sentence, with or without the drop rule, because the status gates it;
- `f-remote-fail`: exit 1 with the line relayed as `remote: Host key verification failed.` plus eight spaces in a
  pipe, and as `\033[Kremote: Host key verification failed.` on a terminal;
  `c-relay` and `k-pty` print nothing with the rule; the `c-relay` arm with `STANDIN_DROP_REMOTE=0` prints the
  host-key sentence, and `STANDIN_STRIP_ANSI=0` changes nothing for `c-relay`, whose pipe carries no escape
  (`w-remote-fail-terminal-c-relay-no-strip` stays silent).
  The strip control belongs to K and K2, where the escape is present: `w-remote-fail-terminal-k-pty-no-strip`
  speaks, and this round's `o-remote-fail-terminal-k2-adaptive-no-strip` speaks where its `-with-strip` sibling is
  silent.

K2, when the wrapper's own standard error is not a terminal, is C byte for byte:
run with the same clone directory, `c-relay` and `k2-adaptive` transcripts compare identical
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
`BYTES refusal-k2-adaptive refusal-c-relay identical=yes`);
run with the arm's own directory in the path, the two transcripts differ by exactly the four characters the two
arm names differ by.

## The rule that forbids the question, and where it lives now

The owner's instruction, quoted whole from `doc/planning/cli-git-no-self-invocation.md`, "The rule":

```text
"We should never invoke ourselves recursively.
That's bad design.
Audit,
remove,
and
ban that design aggressively."
```

The same section holds the two other owner sentences that bear on this design, quoted whole:

```text
and "There are clean ways to have extensive explanatory error messages even with the
ban."
The owner then added:
"we don't have to use any shell!"
```

The first supports adding an explanation rather than a prompt, which is what every option here does; the second is
why no option builds a shell command to print one.

The same record's third bullet of the rule as built, quoted whole:

```text
- An explanation of a failure is printed by the parent process that owns the child,
  from facts the parent can read
  itself.
  Nothing that Git or a hook starts is asked to explain anything.
```

The decision record that answered this design's two questions states the same rule in its own words
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`):

```text
So the rule stays as written:
no program the wrapper did not start explains anything,
and the wrapper never hands its own path to ssh or to Git to start.
```

Every option here keeps that rule as those three texts state it:
nothing Git or a hook starts explains anything,
and no program the wrapper did not start explains anything either.
The wrapper's own children (`ssh -G`, `ssh-keygen -F`, or a relay it forks) are the wrapper's to start,
and the sentence is printed by the parent that owns the child,
which is what the third bullet asks for.

The credential rule is the other one that matters,
and it lives in
`package/git-policy/cli/SPEC.md`,
in the paragraph beginning "No command the wrapper runs or forwards asks a person for a credential",
and in `package/git-policy/cli/README.md` in the paragraph that ends
"because no command the wrapper runs or forwards asks a person for a credential"
(the paragraph in `package/git-policy/cli/README.md` on prompts and `SSH_ASKPASS`, and the paragraph in
`package/git-policy/cli/SPEC.md` that says this release names no next step at a new host key).
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

The SPEC's parenthetical still points at a handover entry that no longer exists,
to be sorted out where that file is edited;
the rule's own text is the record now.
`SPEC.md` also says "a host key is not a credential the push lacked",
so the credential rule does not itself forbid answering a host-key question.
What forbids it is the owner's answer of 2026-11-20 that the refusal is explained rather than answered,
and this design's own brief.
Every option here refuses to answer yes;
the ones that could answer yes are listed and not ranked.

## The options shown and not kept

This document ranks six options and hands one of them to the owner,
so no option here is built until the owner answers.
Each entry says what it starts,
what it reads,
whether it can answer yes,
what a person sees for each family of "The case every option is tried against",
what it costs on a command that meets no question,
which commands it covers,
and its pros and cons.

B, E and J carry answers already given,
and each says which answer and what became of it.

The costs are given twice.
On a typed command,
against a control run in the same rounds,
the sixth round's harness resolves only D: its floor is `+4.94 ms` with a range of `-34.13` to `+35.51`
(run, `keep/run-10-typed-path-with-every-twin.log`).
As the steps each option adds, against a same-command twin in the same rounds,
the steps harness of the round in
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/` resolved all but K against C and K2
against C (`keep/derived-medians.txt`, from
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-3-steps.log`).
Where a figure is a median of per-round differences, its range and the count of positive rounds are given with it,
and the twin it is quoted against is named.

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

At H1's case A is what `main` does: it is the only option that adds nothing at all, and it ends 0 in every
transport because nothing is added to the connection
(run, `keep/run-6-j-arms-and-transports.log`, arms `j-noask-*` and `f-noask-proxyjump-*`; the direct and
`ProxyCommand` arms are `after-its-gate-review-2026-11-25/keep/run-1-behaviour.log:531-639`, `f-noask-direct-*` and
`f-noask-proxycommand-*`).
That is also why the owner's answer of 2026-11-25 is safe: it restores exactly this.

A person sees the transcripts under "What the person sees today": OpenSSH's line, Git's fatal line, exit 128,
and no next step.
The one way out is knowledge the words do not give: connect once with plain `ssh <host>`,
which the wrapper does not touch,
or add the key to `known_hosts`.

Cost: nothing, and no control can measure nothing.
Covers: none of the four commands.

Its documentation-only complement:
`package/git-policy/cli/README.md` and the man page could say that the first contact with a host needs one plain
`ssh <host>`,
which costs no code and does nothing for the person who meets the refusal without having read it.
That is a complement to any option here and never an option of its own.

Pros: no code, no new mode, no policy change, exec semantics untouched, and the refusal is exactly the owner's
recorded rule.
Cons: the first contact with every new host ends without a next step,
a key whose passphrase was never loaded ends the same way,
and a changed key ends with OpenSSH's banner and no advice about the offending line.

### B. The wrapper itself as the askpass program

Answered "Never (Recommended)" by the owner on 2026-11-24
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
so this is recorded for its evidence and is not ranked.
The mechanism is kept because it is the only option that would name the host, the address, the port and the key at
the question.

Mechanism: the overlay keeps `SSH_ASKPASS_REQUIRE=force` and names the wrapper's own path in `SSH_ASKPASS`
instead of `false`.
A program, unlike the retired sentence, is a value OpenSSH can start, which is why B survives H1 where J does not.
OpenSSH starts the wrapper with the question as its only argument (`readpass.c:77`),
and the start writes the sentence for that question to standard error,
writes nothing to standard output,
reads nothing from standard input,
and exits 2, which OpenSSH reads as no answer (`readpass.c:99` to `:102`).

The selector, as the round whose folder is
`~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/` revised it:
a start is the mode when the environment names this executable (`SSH_ASKPASS` or `GIT_ASKPASS` holds the running
executable's path, compared after canonicalising),
and the start has exactly one argument which holds a space or OpenSSH set `SSH_ASKPASS_PROMPT`.
Every askpass prompt read holds a space, including a keyboard-interactive prompt built as `(%s@%s) %s`
(`sshconnect2.c:2125` to `:2130`),
and no one-argument Git invocation puts a space inside the argument.

What a person sees, from the round whose folder is
`~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/`
(run, its `keep/run-1-behaviour.log`, and the arms named there):

- a new host key: the sentence with the host, the port and the address, then OpenSSH's line, then Git's;
- a key nobody could unlock: the sentence with the key's path from the question, then
  `Permission denied (publickey).`, then Git's lines;
- a differing key for the address at `CheckHostIP=yes`: the offending file and line;
- a changed key, `BatchMode=yes` and `StrictHostKeyChecking=yes`: nothing from B.

What it starts: the wrapper, started by OpenSSH, once per question.
What it reads: its argument, two environment values, and its own path for identification.
Can it answer yes: no; it writes nothing to standard output and exits 2, and `confirm` reads no answer as no.

Cost on a command that meets no question: nothing measurable, because no process starts when no question is asked
(the askpass-only arm of the typed path in the round whose folder is
`~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/` was
`askpass_only median=-0.463`, inside its own floor, `keep/run-1.log:624`).
The cost on a refused question is one askpass start.
Covers: all four commands, at any question.

Pros: the only option whose words name the host, the address, the port and the key,
and the only one that can tell a new key from a key that differs for the address;
it needs nothing on the path of a command that meets no question;
it changes nothing about Git's process, streams, signals or exit status;
it survives `ssh -q` and `LogLevel=QUIET`;
it keeps the security-key touch notice at a terminal;
and it meets the service's own pushes with the same words.

Cons: on this machine OpenSSH may not run it,
because `git` confines `ssh` in a child profile that allows executing only `ksshaskpass` and VS Code's script
(`/etc/apparmor.d/git`, read this round: `@{lib}/code/extensions/git/dist/askpass.sh px`),
and today's `false` works only because a local rule names it
(`/etc/apparmor.d/local/git_ssh`, read this round: `@{bin}/false rix`);
it introduces a new way for the wrapper to be started,
which is the owner's "Never";
the mode must never write to standard output, never read standard input, and always exit non-zero;
it sees questions of every kind;
the sentence appears on a command that then succeeds, when a passphrase key is skipped for one that works;
a typed LFS push would say the same thing twice;
and on this machine the words need two AppArmor files edited per build.

### C. The wrapper keeps the child and reads its standard error through a pipe

Mechanism: a typed network command no longer replaces the process;
the wrapper starts Git as its child,
relays standard error through a pipe while Git writes it,
then appends the sentence when the transcript reads as one of the refusals
(`reads_as_unknown_host_key` and `reads_as_a_key_failure`, `src/native/lfs/lfs_pre_push.rs`),
and only when the child ended non-zero.
The passphrase sentence is gated on the second of those, whose only phrase is OpenSSH's `Permission denied (`,
so an `https` credential failure, whose words are Git's and `git-lfs`'s, never gets an SSH key sentence;
`reads_as_a_login_failure` is the upload's wider gate and is not used here.
The detector leaves a changed key to OpenSSH's own words,
reads the strict-checking sentence as an unknown key's own,
and reads the last line alone as an unknown key,
so C speaks for a new key, a passphrase or refused key, `BatchMode=yes`, `StrictHostKeyChecking=yes`
(with the strict sentence) and the rare differing-key question.

What a person sees, at a new host key, is the transcript under "The owner's question", C's block;
at a locked key it is the passphrase sentence instead.
The sentences are the ones under "The words".

At H1's case C ends 0 and prints nothing, directly and behind both proxies, because no refusal reached it
(run, `keep/run-6-j-arms-and-transports.log`, arms `f-noask-proxyjump-c-relay` and `j-noask-*`; the direct and
`ProxyCommand` arms are `after-its-gate-review-2026-11-25/keep/run-1-behaviour.log:531-639`,
`f-noask-direct-c-relay` and `f-noask-proxycommand-c-relay`).
At a changed key it is silent, which leaves OpenSSH's banner and names nothing itself
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
family `f-changed`).
At a repository that is not there it is silent (run, family `f-known-missing`).
Under `ssh -q` or `LogLevel=QUIET` it is blind, because OpenSSH prints nothing to read
(run, `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`, arm `c-quiet`;
this round's `o-quiet-ssh-c` is the same shape and silent).
`git clone -q` is not that word: measured this round, it leaves C's line and the sentence is printed
(`o-quiet-clone-c`, decision `hostkey`).

What it starts: Git as a child, one pipe, one relay.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

Cost: `+0.68 ms` over the control, range `-1.40` to `+2.71`, positive 18 of 24
(`keep/derived-medians.txt`, arm `relay-start` against `env-twin`;
the twin of the same arm, `relay-twin` against `relay-start`, is median `-0.03 ms`, range `-2.16` to `+2.32`),
plus `+0.85 ms` for the read of the remote when a remote is named.
On the typed path it is `+5.35 ms`, range `-61.76` to `+58.52`, 16 of 24, inside that harness's floor
(run, `keep/run-10-typed-path-with-every-twin.log`).
Covers: `clone`, `fetch`, `pull` and a non-LFS `push`.

Pros: gives the words at both refusals at the place the existing push sentences do,
chooses them from what OpenSSH printed,
leaves every stream in a captured command byte for byte as plain Git leaves it (measured),
keeps every setting the person and the repository own,
and is the shape the wrapper already uses where it captures a child's output
(`bounded_push` and `lfs_upload`).

Cons:

- the live progress display disappears: Git draws it only when standard error is a terminal,
  and under C that stream is a pipe.
  Measured: plain Git and K each show a `Receiving objects` and a `remote: Counting objects` line at a terminal,
  and C shows neither
  (`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
  the `TERMINAL` lines).
  Passing `--progress` brings them back, the same way K's captured bytes come back with `--no-progress`,
  but it means adding a word to the caller's arguments.
- the security-key touch notice is lost, because OpenSSH writes it to standard error only when that stream is a
  terminal, and otherwise sends it to the askpass program, which is `false` here (`readpass.c:252`, `:279`).
  Source only; no key was available.
- Git's colour is lost too: on a failed push at a terminal plain Git writes five escape bytes (one `\033[K` and
  four colour sequences on the rejected-ref and `error:` lines) and C writes none, because its pipe is not a
  terminal (run, `keep/run-5-remote-line-under-a-terminal.log`, arms `r-fail-plain-terminal` with five escapes and
  `r-fail-c-relay-terminal` with none).
- a signal sent to the wrapper alone does not reach Git unless the wrapper relays it:
  the wrapper starts Git as its direct child, and neither takes a new session nor a controlling terminal, so a
  terminal's control character signals the whole foreground group and reaches Git, while a signal aimed at the
  wrapper's own process number reaches the wrapper alone.
  The wrapper already has the relay that removes that loss: a signal a process sends to the wrapper "reaches that
  child" (`package/git-policy/cli/src/native/process/signal_relay_relayed_child.rs`), which the interactive LFS
  upload uses, so the loss is a choice this shape need not make.
  Measured only for the stand-in of the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`, which has no relay (the child is
  still running after `kill -TERM` of the wrapper); with the relay it is source and design.
- it is blind under `ssh -q` or `LogLevel=QUIET`, measured in that same round, while `git clone -q` leaves its line
  (measured this round, `o-quiet-clone-c`).
- the wrapper must now own the exit status, the signal ending, the wait and the ordering of its own notices against
  Git's output.

### K. The wrapper keeps the child, with standard error on a pseudo-terminal

Mechanism: C's shape, with one difference.
The wrapper starts Git as its child and gives it a pseudo-terminal on standard error instead of a pipe,
relays whatever that terminal produces to the wrapper's own standard error,
and after the child ends prints the sentence the transcript calls for, using the same detectors C uses and the same
non-zero gate.
Standard input and standard output stay the child's own, no argument is added, no program the wrapper did not start
explains anything, and nothing Git or a hook starts is asked to explain anything.

Why the pseudo-terminal at all:
Git, OpenSSH and `git-lfs` all ask `isatty(2)` before drawing anything a person watches
(`transport.c:1230`, `progress.c:146`, `readpass.c:252`).
A pipe answers no to that question, and a pseudo-terminal answers yes.

What a person sees at a new host key is C's transcript with `target` in the sentence, and at a locked key the
passphrase sentence
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
arms `f-newkey-ask-direct-k-pty` and `f-locked-direct-k-pty`).
At H1's case it prints nothing and the clone ends 0, directly and behind both proxies
(run, `keep/run-6-j-arms-and-transports.log`, arm `f-noask-proxyjump-k-pty`; and
`after-its-gate-review-2026-11-25/keep/run-1-behaviour.log:531-639`, arms `f-noask-direct-k-pty` and
`f-noask-proxycommand-k-pty`).

What it costs the bytes a person does not see is measured under "The measurements":
the progress display lands in any captured stream,
a carriage return lands on every line unless the terminal's output processing is cleared,
Git's terminal-only escapes land there as well and stay under every flag,
and every descendant that asks whether its standard error is a terminal is told yes.

What it starts: Git as a child, one pseudo-terminal pair, one relay.
What it reads: the terminal's text as it arrives.
Can it answer yes: no; it only reads.

Cost: `+0.85 ms` over the control, range `-2.73` to `+2.61`, positive 20 of 24
(`keep/derived-medians.txt`, arm `pty-start` against `env-twin`;
its twin, `pty-twin` against `pty-start`, is median `-0.06 ms`),
plus `+0.85 ms` for the read of the remote when a remote is named.
Against C, in the same rounds, it is `+0.16 ms` with a range of `-1.68` to `+1.80` and 13 of 24 positive,
which this harness cannot tell from no difference.
On the typed path it is `+0.31 ms`, 12 of 24, inside that harness's floor.
Covers: `clone`, `fetch`, `pull` and a non-LFS `push`.

Pros: everything C gives, and it keeps the progress display and the security-key touch notice at a terminal
(measured for the display; source for the notice);
and a person who captures output can have plain Git's bytes back with `--no-progress`, because a shipped K that
clears the terminal's output processing writes exactly plain Git's bytes then (measured, `BYTES ... identical=yes`).
Cons: the four effects listed under "The measurements" on every captured stream, of which one is removable by the
wrapper (the doubled carriage return) and the display by a flag the person types, while the escape in front of a
relayed line and the terminal answer for every descendant are the price of the terminal;
a captured stream also gains Git's colour, one `\033[K` and four colour sequences on each rejected-ref and `error:`
line of a failed push, which C's pipe never carries;
the wrapper owns the pty's life and the child's wait, as C owns the pipe's;
and a `ssh -q` transcript still yields nothing.

### K2. K when the wrapper's own standard error is a terminal, C's pipe otherwise

Mechanism: one `isatty` call on the wrapper's own standard error decides which of K's two shapes runs.
When that stream is not a terminal, plain Git would draw nothing, and the pipe presents exactly that,
so the child behaves as it does today and the relay copies its bytes unchanged.
When it is a terminal, the pseudo-terminal presents what Git sees today.

What a person sees: at a terminal, K's transcript (progress and the touch notice's terminal, run,
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
`TERMINAL k2-adaptive receiving=1 counting=1`; and this round's log, `o-terminal-ok-k2-adaptive`, 8618 bytes
against plain Git's 8617, the same 107 escapes and six more carriage returns);
in a captured stream, C's transcript, which is plain Git's, byte for byte
(run, this round's log, `CAPTURE-COMPARE k2-against-plain identical=yes`, 37 bytes, no carriage returns, no
escapes).

What it starts and reads: what K starts and reads, plus one `isatty` call.
Can it answer yes: no.

Cost: `+0.74 ms` over the control, range `-1.38` to `+2.52`, positive 20 of 24
(`keep/derived-medians.txt`, arm `adaptive-start` against `env-twin`;
that run gave this arm no twin, so its floor is the other arms' and not its own),
plus `+0.85 ms` for the read of the remote when a remote is named.
Against C in the same rounds, `+0.12 ms`, range `-1.40` to `+1.38`, 14 of 24 positive, which is no difference this
harness resolves.
On the typed path it is `+5.44 ms`, 13 of 24, inside that harness's floor.
Covers: `clone`, `fetch`, `pull` and a non-LFS `push`.

Pros: no captured byte changes for anyone who captures standard error alone, and the captured stream is plain Git's
byte for byte;
at a terminal the person gets what plain Git gives today, with the progress display and the touch notice's terminal;
the words are C's, chosen from the transcript, and the strict sentence has its own words;
one `isatty` call is all it adds to C's cost.
Cons: it is two shapes to build, test and explain, and both of them must be controlled;
the decision is made once per process on the wrapper's own stream, and every descendant then sees that answer, so a
program that would have used a pipe's answer gets a terminal's whenever the person's stream is a terminal, which is
K's trade applied only where a terminal is there;
the pseudo-terminal is a second device whose size is set once and whose resize is not forwarded, so a window
change after the start reaches neither the child nor the programs that ask;
and it keeps C's other cons, the wait, the signal, Git's colour at a terminal and the `ssh -q` blindness.

A command whose two streams are captured together (`2>&1` into one file or pipe) is C's shape under K2, by the
same one `isatty` call: any capture makes the wrapper's own standard error a non-terminal.
Measured this round with a `git pull` of a fast-forward, both streams merged into one file: plain Git and K2 each
write 245 bytes, identical byte for byte, in the same order (`o-merged-plain`, `o-merged-k2`).
That is one command and one order; a line Git wrote to standard error before a line on standard output can still be
copied after it by a relay that reads before it writes, and no arm forced that order.

### C2. Git kept as a child with the same streams, and a sentence after exit 128

Mechanism: the wrapper no longer replaces the process,
but it does not take standard error either.
It starts Git as a child with the same standard input, standard output, standard error and terminal, waits,
and after exit 128 prints a sentence.
Nothing is piped, no argument is added, no text is read.

Four shapes, and they are four different options, because a check changes which refusal gets words:

- the exit status alone;
- the check speaks the host-key sentence only when no file `ssh -G` names holds a key;
- the check speaks the passphrase sentence only when one does;
- and the shape both of those make together, choosing on what the check found.

The shape the ranking means is the second, the one that speaks only when no file holds a key.
It is not as safe as the revision at `79962f9ccc` said, and three things about it are measured in the sixth round's
logs:

- a connection refused ends 128 with no key known, so the shape speaks the host-key sentence for a host it has
  never seen (run, `keep/run-11-the-words-as-revised.log`, arm `w-never-seen-c2-check-unknown`);
  a name that does not resolve ends 128 the same way, and that arm is unrun;
- `git pull` at a new host key ends 1, so with its ranked gate, exit 128, every C2 shape is silent for `pull`
  (run, `keep/run-11-the-words-as-revised.log`, arm `w-pull-newkey-c2-check-unknown`).
  That is a gate choice: a gate on any non-zero status would cover `pull`, but it would also speak after every
  other non-zero end (a rejected push, a merge conflict) whenever its check finds no key.
  That variant is unrun, and its effect is inference;
- the check runs `ssh -G <host>` with no source for the person's options, so a person whose settings live in
  `GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH` gets a check that reads other known-hosts files and another
  host and port: measured, `ssh -G target` without the fixture's `-F` file resolves hostname `target`, port 22 and
  `userknownhostsfile /root/.ssh/known_hosts`
  (run, `keep/run-9-check-without-a-source-and-no-progress.log`, the resolution printed at the top), and with that
  check a changed key at `LogLevel=FATAL` gets the host-key sentence and a clone that succeeds gets the same
  sentence before it (arms `s-changed-fatal-no-source-c2-check-unknown` and `s-known-no-source-d-precheck`).
  A shipped check therefore has to find the source of the person's options, which is work D's section names too.

The others are worse, and measured in the log of the round in
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/`
(`keep/run-1-behaviour.log`):

- with the exit status alone, a repository that is not there and a refused connection get a paragraph about keys
  (arm `f-known-missing-direct-c2-status`);
- with the check speaking only when a key is known, a repository that is not there gets the passphrase sentence, and a
  new host key gets nothing (arms `f-known-missing-direct-c2-check-known` and
  `f-newkey-ask-direct-c2-check-known`);
- with both branches, a repository that is not there gets the passphrase sentence, a changed key gets the passphrase
  sentence, and a locked key gets it too (arms `f-known-missing-direct-c2-check-both`,
  `f-changed-direct-c2-check-both`, `f-locked-direct-c2-check-both`).

What the ranked shape gives, measured:

- at a new host key, the host-key sentence after Git's lines: right;
- at a locked key, nothing: the one refusal the person needs words for gets none;
- at a repository that is not there, nothing: right;
- at a changed key, nothing: right, and OpenSSH's banner says what changed;
- at `StrictHostKeyChecking=yes`, the general host-key sentence, whose advice that person's own setting refuses
  (run, `keep/run-11-the-words-as-revised.log`, arm `w-strict-yes-direct-c2-check-unknown`);
- under `ssh -q` or `LogLevel=QUIET`, the same as without it, because it reads a status and not a line (run, this
  round's log, arm `o-quiet-ssh-c2`, decision `check-unknown`).

What it starts: Git as a child; on a failure, `ssh -G` and `ssh-keygen -F` as well.
What it reads: the exit status, and with the check the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost: `+0.68 ms` for the fork and the wait
(`keep/derived-medians.txt`, `standin-start` against `env-twin`, positive 23 of 24),
and on the failure path the check's two starts, measured at `+7.50 ms` for D and the same work here,
plus `+0.85 ms` for the read of the remote when a remote is named.
On the typed path, `+4.44 ms`, 15 of 24, inside the floor.
Covers: `clone`, `fetch` and a non-LFS `push`; not `pull` under its ranked gate, and a gate on any non-zero status
would cover `pull` with the false positives named in this section.

Pros: keeps the terminal, the progress display and every setting the person owns;
adds no word to the caller's arguments; touches no stream, so a captured command's bytes are plain Git's;
and it survives `ssh -q` or `LogLevel=QUIET`.
Cons: it cannot tell the refusals apart, so its ranked shape is silent at a key nobody can unlock and, under its
ranked gate, for `pull`;
it speaks wrongly at a connection refused for a host never seen, and at any failure whose check cannot read the
person's own ssh options, which the fix has to reach for the check to read the right files;
a sentence printed before the failure, as D prints it, is not this shape's problem, but its own sentence's premise
("no connection got past the key") is a statement about a refusal that has happened;
the wrapper still owns the wait, the exit status and the signal ending, and a signal sent to the wrapper's own
process number does not reach Git because `run_real_git` has no relay (`invocation/forwarding.rs`);
and the shapes that do speak at a locked key print a wrong sentence at a repository that is not there.

### D. A check before Git starts

Mechanism: before replacing the process,
resolve what OpenSSH would use and ask whether this machine holds a key for it:
`ssh -G <host>` for the resolved host name, port, `hostkeyalias`, `userknownhostsfile` and `globalknownhostsfile`,
then `ssh-keygen -F` over every file `ssh -G` named,
under `[host]:port` when the port is not 22 and under the alias when there is one.
`ssh -G` connects to nothing.
When no file holds a key and the resolved `stricthostkeychecking` is `ask`,
print the sentence first, then run the command as today.

The condition is `ask` alone.
`accept-new` accepts an unknown key without asking, so a D that treats it as asking prints before a clone that
succeeds; measured, both ways
(run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-1-behaviour.log`,
`f-accept-new-direct-d-precheck-acceptnew` prints, `f-accept-new-direct-d-precheck` is silent, and both clones end 0).

What a person sees at a new host key is the transcript under "The owner's question", D's block:
the sentence, then `Cloning into`, then OpenSSH's refusal and Git's lines.
For the passphrase case it prints nothing, because the key is known (run, family `f-locked`).
That is D's gap, measured, and it is the case whose words the person most needs.

The sentence can also be wrong about why a command failed, and the rounds named here measured three ways:

- with nothing listening on the port, D prints the host-key sentence and the real words are
  `ssh: connect to host 127.0.0.1 port 2299: Connection refused`
  (run, `keep/run-11-the-words-as-revised.log`, arm `w-never-seen-d-precheck`);
- the check has no source for the person's options, so it can read other files and another host and port than the
  command will use; measured, a clone that ends 0 gets the sentence before it when the check runs without the
  fixture's configuration (run, `keep/run-9-check-without-a-source-and-no-progress.log`, arm
  `s-known-no-source-d-precheck`), and the plain `ssh -G target` resolves hostname `target`, port 22 and
  `/root/.ssh/known_hosts`;
- `KnownHostsCommand` supplies the key, so the real clone exits 0 and D speaks
  (run, `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/run-1.log:280-290`, the
  control arm first and then the arm with `KnownHostsCommand`).

Of the three classes the revision at `79962f9ccc` listed, one holds, one does not and one is unrun:
`KnownHostsCommand` holds, as `run-1.log:280-290` shows;
`@cert-authority` supplies the key by certificate and `ssh-keygen -F` finds the line, so D stays silent, which is
right (run, `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/run-2.log:82`);
and `VerifyHostKeyDNS=yes` would be a third, and is unrun, because no DNSSEC setup was available.

Two limits stand:

- a changed host key looks known to it, because `ssh-keygen -F` finds a key under the name,
  so it says nothing about the refusal whose advice needs the offending line;
- it is blind to a `HostName` alias whose key lives on a jump host, and to any known-hosts file a `ProxyCommand` or
  `ProxyJump` makes the far side read.

What it starts: `ssh -G` and `ssh-keygen -F` before the command.
What it reads: the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost: `+7.50 ms` over the control, range `+5.23` to `+10.55`, positive 24 of 24,
for the whole check with both starts (`keep/derived-medians.txt`, arm `precheck-ask-known` against `env-twin`;
its twin, `precheck-twin` against `precheck-ask-known`, is median `-0.15 ms`),
plus `+0.85 ms` for the read of the remote when a remote is named.
The two starts separately are `ssh -G` at `+3.33 ms` (24 of 24) and `ssh-keygen -F` at `+2.38 ms` (24 of 24),
each against one process start.
On the typed path it is `+20.08 ms`, range `-12.06` to `+76.25`, positive 22 of 24, against a twin that differs by
`-6.64 ms`: the only option that harness resolves
(run, `keep/run-10-typed-path-with-every-twin.log`).
Covers: `clone`, `fetch`, `pull` and a non-LFS `push`, because it speaks before whatever command it is given
(measured on the stand-ins; see "Which commands each option covers" for the wrapper paths that were not traced).

Two costs the round whose folder is
`~/temp/agent/typed-network-command-next-step-at-a-new-host-key-design-2026-11-24/` did not name, both read from the
source:

- `ssh -G` runs the person's `Match exec` commands, measured in the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`: one plain `git ls-remote` ran it once,
  and D's check plus the ssh it then runs ran it twice.
- `ssh -G` can resolve names: `CanonicalizeHostname` makes it do DNS work, which was not run here
  (`ssh_config(5)`, "CanonicalizeHostname").
  Its output can also carry `ProxyCommand` and `SetEnv` values, which must never be logged.

Pros: no change to how Git runs, no new mode, no policy change, and the words arrive before the connection.
Cons: one of the two refusals is out of reach,
its words can be wrong about a command that then fails for another reason and wrong on commands that succeed,
because its check reads `~/.ssh/config` and the files there unless the wrapper finds the source of the person's
options (`GIT_SSH_COMMAND`, `core.sshCommand`, `GIT_SSH`),
the cost is paid by every network command,
and the check runs the person's own configuration commands one extra time.
Its sentence also states that a connection stopped at the key, which is a prediction when D prints before the
command.

### E. The wrapper becomes the ssh that Git starts

Answered "Never (Recommended)" by the owner on 2026-11-24,
so this is recorded for its evidence and is not ranked.
The owner's reason is the rule under "The rule that forbids the question":
the process that explains would be one Git started,
and the wrapper would hand its own path to Git to start.

Mechanism: the overlay names the wrapper in `GIT_SSH_COMMAND` (or `GIT_SSH`),
so the wrapper starts the real `ssh`, relays its standard error, and appends the sentence when the words read as one of
the two refusals, while Git keeps the terminal and the process stays replaced.

What a person sees, from the round whose folder is
`~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/` (`keep/run4.log`):
OpenSSH's line, the sentence, then Git's `fatal:` line.

Cost, in that round's measurement: `+5.97 ms` against its control, 24 of 24 positive,
of which about `4.9 ms` is the probe any unrecognised named ssh pays and about `1.0 ms` is E's relay for two starts.

Cons: it takes over `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`, which belong to the person and the repository;
wrapping them means building a shell command out of a shell command, which the `shell-text` rule forbids;
it must find the real `ssh` the way `real_git.rs` finds Git;
it is blind under `ssh -q` or `LogLevel=QUIET`;
a signal sent to the wrapper alone does not reach ssh;
the security-key notice is lost;
its words come from the same detectors C uses, so they miss a changed key;
and the process that explains was started by Git, which is the owner's question.

### J. The sentence itself as `SSH_ASKPASS`'s value

Shown, chosen on 2026-11-24, and retired by the owner on 2026-11-25:
`SSH_ASKPASS` holds `false` again.
The reason is H1, and it is not a defect of the wording:
a value OpenSSH cannot start takes the connection down when the connection has a proxy.

J's value is 369 bytes and is kept as `keep/sentences/j-value.txt`.
The sentence the round whose folder is
`~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/` built measured 571 bytes
(its `keep/run-1-behaviour.log:4`);
the two figures are different texts, and the 369-byte value's failure line is in
`~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/run-3.log:61`
(`File name too long`), which is the value the sixth round ran.

Measured in the sixth round, per transport
(run, `keep/run-6-j-arms-and-transports.log`, arms `j-noask-*`):

- direct: exit 0 and the clone completes, because the askpass child dies and ssh goes on to the next key,
  and the person reads `ssh_askpass: exec(<the sentence>): File name too long`;
- behind a `ProxyCommand`: exit 128 and the clone fails, with the same line and Git's `fatal:`;
- behind `-J`: exit 128 and the clone fails, the same;
- the same arm behind a relay that ignores SIGHUP: exit 0 and the clone completes behind the `ProxyCommand`,
  which names the signal as the cause, and exit 128 behind `-J`, where there is no client-side relay to ignore it.

Measured limits, each one a finding against it:

- one text for every question, so it cannot name the host or the key;
- OpenSSH's framing: the person reads the advice inside `ssh_askpass: exec(...): File name too long`;
- it prints on a command that then succeeds, when a passphrase key is skipped for one that works;
- it disappears under quiet, because the whole line is OpenSSH's and `ssh -q` suppresses it;
- it also answers questions that are not refusals, such as the multiplexing question;
- it is not a program, so OpenSSH's failure path runs, and behind a proxy that path ends the connection.

What is left of J on `main` after the askpass round: nothing.
The value is `false` again, `NO_ANSWER_SENTENCE` is gone, the tests that read it are gone with it,
and `SPEC.md` and `README.md` say the release names no next step at a new host key.
`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING` stays, because the upload's explanation and the push record still print the
wrapper's own host-key sentence.

### F, G and H: not ranked

- F. Automatic trust, meaning `StrictHostKeyChecking=accept-new`, a pre-added key, or an askpass program that
  prints `yes`: all of them answer a yes/no question on a person's behalf, which this design's brief forbids and the
  owner's answer of 2026-11-20 also rules out by having the service wait for a key the machine learns some other way.
- G. Relax the no-prompt rule for typed commands, clearing `SSH_ASKPASS_REQUIRE` or setting it to `never`:
  the owner's rule covers every command the wrapper runs or forwards, and a typed command inside a script or a hook
  has no terminal to answer on.
- H. Run the transport itself: nothing in the two refusals needs it, the cost is a transport implementation and
  its security surface, and it takes the network path away from Git for every command.

## The words

Three sentences,
all new.
They live beside the two that exist today (`src/native/lfs/lfs_pre_push.rs`),
because the same detectors must not drift,
and the existing two stay for the LFS upload and the service's push record,
whose subjects are an upload and a push.
They are verb-neutral ("run this command again", never "push again").

Nothing on `main` holds them today:
`NO_ANSWER_SENTENCE` went with J,
and `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING` names the older push sentence, not these.

The words of the revision at `bda96883d5` (2026-11-25T13:54:27-05:00) are kept under
"The record of the rounds before this one" as what was argued over;
the sentences a shipped C, K or K2 prints are the three here,
printed byte for byte from `keep/sentences/host-key.txt`, `keep/sentences/strict.txt` and
`keep/sentences/passphrase.txt`: the host-key sentence in the arms of this round's
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/run-12-owner-question-arms.log`,
and the strict and passphrase sentences in the sixth round's `keep/run-11-the-words-as-revised.log`, whose files are
unchanged.

### A new host key, which may also be a key that changed

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, and no command the wrapper runs asks whether to trust a new one, so no connection got past the key. Connect with plain `ssh target`, which the wrapper does not touch, and answer the question it asks; if it names a key that changed instead, compare the fingerprint it shows with the one you know to be right before you accept it, then remove the line it names and connect once more, which then asks about the new key, and run this command again.
```

The claims, each one true of what the transcript can show, and each one run:

- `does not know ... or holds an outdated one` covers the three cases a transcript can be: an unknown key, a changed
  key, and the rare differing key for an address.
  It is the hedge the wrapper already ships for a key failure ("refused or could not be unlocked"),
  applied to the other refusal.
- The advice works, with one connection for a new key and two for a changed one:
  measured, one plain `ssh` with the person's answer, exit 0, the key added, and the clone then ends 0
  (run, `keep/run-8-whether-the-advice-works.log`, arms `advised-ssh-newkey` and `advice-2-clone-after`);
  and for a changed key the plain `ssh` names the offending line and adds nothing, so removing that line alone
  leaves the clone refused (`advice-2b-clone-after-removal-alone` ends 128) and a second plain `ssh` accepts the new
  key, after which the clone ends 0 (`advised-ssh-changed-again`, `advice-4-clone-after-removal`).
- The sentence no longer says to add the key to `known_hosts`,
  because that remedy is wrong for a changed key and because plain `ssh` is the path the wrapper does not touch.
- A changed key can be an attack as well as a machine that moved, so the sentence now says to compare the
  fingerprint plain `ssh` shows with the one the person knows before accepting it; the clause is what makes
  removing the line safe, and it is the answer to the review's Low about "outdated" and the removal advice.

What the sentence cannot promise, and the section "The measurements" records:
at `LogLevel=FATAL` a plain `ssh` prints `Host key verification failed.` alone, so "it names a key that changed"
does not hold there;
with `BatchMode` in the configuration plain `ssh` reads, that `ssh` cannot ask either;
and when the person's options come from `GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH`, the advised `ssh target`
reads other settings: measured this round, with `-F` standing in for the person's `~/.ssh/config`, the advised
`ssh` accepted the key into the default `known_hosts` while the typed command, whose known-hosts file the variable
named, was refused again (`o-advice-gsc-ssh` and `o-advice-gsc-clone-after`); the advice works when the person
passes the same option to it (`o-advice-gsc-ssh-with-option` then `o-advice-gsc-clone-after-option` ends 0).

### The same refusal when the person's own setting refuses to ask

```text
monochromatic-git-policy-cli: this machine has no SSH host key recorded for `target`, and your settings refuse to ask about a new one (`StrictHostKeyChecking=yes`), so the connection stopped before any login. Connect once with plain `ssh -o StrictHostKeyChecking=ask target`, which the wrapper does not touch and which asks once, then run this command again; or add that host's key to `known_hosts` yourself.
```

This sentence is chosen from the transcript:
OpenSSH prints `No ED25519 host key is known for [localhost]:2222 and you have requested strict checking.`
and the detector accepts that line as an unknown key's own,
so the reader knows both the refusal and the setting that caused it.
Without its own words the general sentence would tell that person to run `ssh target`,
which reads the same configuration and refuses the same way
(measured: `No ED25519 host key is known ...` and `Host key verification failed.`, run,
`keep/run-8-whether-the-advice-works.log`, arm `advised-ssh-strict-plain`),
where the sentence's own command then works and the clone ends 0
(arms `advised-ssh-strict-ask` and `advice-5-clone-after-strict`).

### A key that was refused or could not be unlocked

```text
monochromatic-git-policy-cli: the key this connection offered was refused or could not be unlocked, and no command the wrapper runs asks a person for a passphrase, so a key no agent holds cannot be used and a key the server does not accept answers the same way. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add` (`ssh-add -l` lists what the agent holds), or use a key without a passphrase, then run this command again; a key the server still refuses needs the server's account to accept it.
```

What changed from the words of the revision at `bda96883d5`, and why:

- the key's path is gone:
  `Permission denied (publickey).` carries no path,
  and the path was the fixture's, so no shipped sentence can name it from this transcript;
- the cause is hedged,
  because `Permission denied (publickey).` arrives both for a key nobody could unlock and for a key the server
  refuses, which the project already records (`reads_as_a_key_failure`,
  `src/native/lfs/lfs_pre_push.rs`);
- `ssh-add -l` is offered because it rules the second cause out,
  exactly as `login_needed_message` offers it.

The branch that can be run was run: a key without a passphrase, offered alone, clones with exit 0
(run, `keep/run-8-whether-the-advice-works.log`, arm `advice-6-clone-with-a-key-without-a-passphrase`).
The branch that names the locked key cannot be run here, because its passphrase is nobody's to give;
`ssh-add` with an agent is the wrapper-free step, and no arm measured it.

### Why there is no fourth sentence for a differing key

The differing-key question is rare (`CheckHostIP` defaults to off),
and its text, which carries the offending file and line, reaches only the askpass program,
so only B could print it.
Under C, K or K2 the transcript is the last line alone and the general sentence is printed:
its way to connect once is right for that case, and it makes no claim about adding a key,
which the revision at `bda96883d5` got wrong.

## Where the host comes from, and what reading it costs

The host is the one the person's remote names, not the address it resolves to,
because a `known_hosts` line is recorded under the name the connection asked about:
the fixture's `target` carries `HostName localhost`, so the clone records `[localhost]:2222`,
and advice that named `127.0.0.1` would ask the person to accept a second key for the same machine.

- A `clone` carries the URL in its own arguments, and reading it directly costs nothing.
  It is not, however, the URL Git connects to: `url.<base>.insteadOf` rewrites it,
  so the host named and the transport chosen can differ from the ones Git uses.
  The read that applies the rewrite is `git ls-remote --get-url`, which is one process start
  and which was measured on a throwaway repository: a remote `gh:served.git` with
  `url.ssh://target/work/.insteadOf` set to `gh:` prints `ssh://target/work/served.git`
  (run, `keep/probe-insteadof.log`).
  A `clone` therefore pays one start for it, or pays it only when a refusal has to be worded,
  whichever the transport answer under "The owner's question" chooses.
- A `fetch` or `pull` that names a remote, or none, needs the remote's URL from the repository:
  `git remote get-url <remote>`, one start, measured at `+0.85 ms` against one process start,
  range `-0.59` to `+1.50`, positive 23 of 24 (`keep/derived-medians.txt`, arm `remote-url`),
  and it applies `insteadOf` too (run, `keep/probe-insteadof.log`).
- The connect form is built from that URL's own grammar:
  `ssh://[<user>@]<host>[:<port>]/<path>` and the scp-like `[<user>@]<host>:<path>`.
  A port that is not 22 is written `-p <port>`, a user the URL names is written before the host,
  and a standard port and an unnamed user are left out.
- A host, user or port that is not a plain host name, address or number is named in backticks and not put into a
  command, so nothing the sentence offers can be read by a shell as anything but a word.

## Three rules about printing text that came from outside

- No text from a transcript or a question reaches a sentence unescaped:
  the host is printed only after every byte a host name cannot hold is dropped,
  and the server's own prompt text is never printed at all.
- Nothing the sentence offers as a command is built from that text without regard for the shell's grammar.
- The passphrase sentence offers no command built from a path,
  so OpenSSH's 100-character cut of the key path (`sshconnect2.c:1547`) cannot turn into a command that names no file.

## What a shipped K, K2 or C must get right

C, K and K2 share all of this except the terminal itself.
Each item says what would go wrong without it, and the measurement or the source that says so.

- The pseudo-terminal's output processing is cleared: `c_oflag &= ~OPOST`.
  Without it, every line feed the child writes becomes a carriage return and a line feed,
  so a captured transcript gains a byte on every line and OpenSSH's own line ends in two carriage returns.
  Measured: a refusal's transcript is 651 bytes with 7 carriage returns by default and 645 bytes with 1 when
  `OPOST` is cleared, which is C's transcript byte for byte
  (run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
  `BYTES refusal-k-pty-raw refusal-c-relay identical=yes`).
- The lines Git relays from the remote are dropped before the detectors read, and the drop rule strips the escape
  prefix first.
  Git prefixes every line the other end writes with `remote: `,
  and with a terminal and a real `TERM` it writes `\033[K` before that prefix,
  so a rule that compares the line's first eight bytes misses the line exactly where K and K2 run at a terminal.
  Measured: with the strip, K's pipe and terminal arms and K2's terminal arm print nothing on a push whose server
  writes the refusal's words and then refuses; without it, the same arms print the host-key sentence
  (run, `keep/run-5-remote-line-under-a-terminal.log`, arms `r-fail-k-pty-redirect`, `r-fail-k-pty-terminal`,
  `r-fail-k2-adaptive-terminal` silent, and `r-fail-k-pty-redirect-nostrip`, `r-fail-k-pty-terminal-nostrip`
  speaking; run, `keep/run-11-the-words-as-revised.log`, arms `w-remote-fail-terminal-*`).
  The shipped detector does not drop those lines today, so a transcript holding
  `remote: Host key verification failed.` reads as an unknown key's.
  The same hole is in the explanation the LFS upload prints, because it reads the same detector;
  fixing the detector fixes both.
- Nothing is printed unless the child ended non-zero.
  A transcript can hold the refusal's words on a command that succeeded:
  measured, a push whose server writes the line and accepts gets no sentence, with or without the drop rule
  (`f-remote-ok-c-relay`, `f-remote-ok-c-relay-keep-remote`).
- No `setsid` and no `TIOCSCTTY`.
  The person's terminal must keep signalling Git,
  and Git's progress is drawn only when `is_foreground_fd(fileno(stderr))` holds or the descriptor has no foreground
  group at all (`progress.c:106` to `:110`, `:146`).
  A new session that owns the pty would put Git's group outside the foreground one.
  The stand-in has neither call; no arm sent a signal to the wrapper alone, so that part is design.
  What the design is: a signal sent to the wrapper alone does not reach Git,
  and a terminal's control character signals the whole foreground group and does reach Git.
- A size is set on the terminal, and it is not what decides Git's progress width.
  Git's width comes from `term_columns()` reading **standard output** (`pager.c:213` to `:241`),
  which K leaves alone, so copying the wrapper's window size onto the pty changed nothing in any transcript:
  40 and 120 columns both cut this clone's progress at the same place
  (run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
  the `WIDTH` lines).
  The claim of the revision at `bda96883d5` that the arms used eighty by twenty-four was unsupported;
  the arms' pty had no size, and Git read its `stdout`, which was a file, and fell back to 80.
  A shipped option sets a size for the programs that ask their own standard error's terminal, and nothing more.
  A window change after the start is not forwarded, so those programs keep the size they were given.
- The relay ends when the child does, after draining what is already readable.
  A hook that leaves a process holding the child's standard error otherwise holds the wrapper:
  measured, a `post-checkout` hook that starts `sleep 5` makes the clone take 5170 ms where the child ends in about
  200 ms (run,
  `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`,
  arms `sleeper-c-relay` and `sleeper-fixed-c-relay`).
  With the rule, the same arm ends in 206 ms.
- A signal and a window change are not forwarded by the relay.
  The wrapper must not take the child's signals from the person's terminal,
  and `SIGWINCH` after the start does not reach the pty, as it does not reach a pipe.
  The interactive LFS upload already relays a signal sent to the wrapper and holds no deadline of its own
  (`doc/planning/cli-git-no-self-invocation.md`, the note beginning
  "Not changed: the interactive upload has no deadline of its own"), which is the precedent for the shape.
  A shipped option says so wherever it documents what it changes.
- The tail the detectors read is bounded.
  A clone prints megabytes of progress; the tail holds what the words need.
  The stand-in keeps the last 65536 bytes, and the wrapper's own LFS explanation already keeps a bounded
  tail.
- The child's standard input and standard output are its own, and the exit status and the signal ending are
  Git's: `end_as_git_ended` is the existing shape for that.
- The sentence is printed after the child's own output, on the wrapper's standard error.
- Under `ssh -q` there is nothing to read, and the option is silent; that is not fixable by reading harder.
  `git clone -q` is a different word: it still leaves the line (measured this round, arm `o-quiet-clone-c`).
- A typed LFS push would say it twice, because the upload's explanation reads the same transcript;
  the sentence must be the shorter of the two for that path, or the upload must learn to stay silent when it already
  spoke.
- Which commands: see "Which commands each option covers", because the wrapper has no single entry point for a typed
  network command.

## What building each option would take

### B, answered "Never"

- `no_prompt_overlay` names the wrapper's own path in `SSH_ASKPASS` and keeps `SSH_ASKPASS_REQUIRE=force`
  (`src/native/invocation/child_environment.rs`).
  `GIT_ASKPASS` stays empty.
- a mode at the top of `main`, after `install_payload_free_panic_hook` and before any planning,
  selected by the two signals under "B. The wrapper itself as the askpass program",
  printing one sentence and exiting 2 on every branch,
  and keeping its last log line through `end_after_the_log`.
- the sentences in one module beside `lfs_pre_push.rs`, with the address and the key filled from the question.
- controls: a new key, a passphrase key and a command that meets no question;
  a nested one-argument `git status` that still runs Git;
  a changed key, `StrictHostKeyChecking=yes` and `BatchMode=yes` that print nothing;
  standard output empty and standard input unread;
  every branch non-zero, including a panic whose message holds no question text;
  an LFS push that prints one sentence and not two.
- a reading of the source guards' `own-executable` rule, with an exception beside `real_git.rs`'s.
- the AppArmor rules this machine needs, in both stacked local files:
  `/etc/apparmor.d/local/git_ssh` (which today holds `@{bin}/false rix` and `owner /work/** rwk`) and
  `/etc/apparmor.d/local/git`.
  Without them B prints `ssh_askpass: exec(<path>): Permission denied` and no words.

### K, K2 and C

- One relay, wired into every path that starts Git for a typed command.
  There is no single entry point, and these are the paths, read from the source this round:

  - `Action::Forward` runs `replace_process_with_real_git` (`invocation/entry.rs`, the arm at `:388` to `:408`);
  - `Action::ForwardCoordinated` runs `run_coordinated_forward`, which either starts Git as a child through
    `run_forwarded_git` or falls back to `replace_process_with_real_git`
    (`invocation/entry_run_forwarded_actions.rs`, the coordinated arm);
  - `Action::ForwardPush`, `Action::ForwardAnnounced` and `Action::ForwardSettled` each start or replace for reasons
    of their own (`invocation/action.rs`, `push/stacked_merge_push.rs`);
  - a `pull` that merges runs its fetch itself, as `git fetch --update-head-ok`, through `run_wrapper_own_git`
    (`merge/pull_isolation.rs`), so a `pull`'s network step never passes the top-level forwarding arm;
  - the LFS upload and the push record already read a child's output and keep their own sentences.

  Which of those a given typed command reaches was not traced end to end;
  what is read here is that a single entry point does not cover them.

  `run_real_git` is not K's shape:
  its child is started with inherited streams and waited for
  (`spawn_retrying` then `wait`, `invocation/forwarding.rs`), which is C2's shape.
  C's shape is the captured-output pattern the wrapper already uses in `bounded_push` and `lfs_upload`.
  The relay is new code at each of those paths, or a new function they all call.
- The transport question, decided rather than assumed; it is put to the owner under "The owner's question".
  Nothing in `src/native` classifies "a remote that ssh carries": the only URL handling is the LFS upload's
  `insteadOf` reading (`lfs/lfs_upload_settings.rs`).
  So either the relay runs for every typed network command whatever the transport,
  and an `https` or `file` clone gets a pseudo-terminal and no words, or the wrapper reads the remote first
  (`git ls-remote --get-url`, which applies `insteadOf`), which for a URL in the arguments costs one start and for a
  named remote `+0.85 ms`.
- The detectors.  `reads_as_unknown_host_key` and `reads_as_a_key_failure` stay where they are,
  and the strict sentence is chosen from the same text.
  The `remote: ` rule, the escape-prefix strip and the non-zero gate are the three additions, and all three belong to
  the detector rather than to the relay, so the upload's explanation gets them too.
- The words, in one module beside `lfs_pre_push.rs`, filled from the remote as typed.
- The exit status, the signal ending and the wait, which the LFS upload's `SignalGuard` machinery already shows.
- The terminal: `openpty` and the termios calls.
  `libc` is a direct dependency (`package/git-policy/cli/Cargo.toml`), but the owner chose `rustix` over hand-written
  `libc` calls (2026-11-07, "Linux calls: rustix", the comment at `Cargo.toml:163-165`), and `rustix` is pinned to
  `process`, `fs` and `thread` (`Cargo.toml:166`).
  rustix 1.1.6's own `Cargo.toml` has `pty = ["fs"]` and `termios = []` (source, `~/.cargo/registry`), so K and K2
  through rustix would add those two features, which is a pick for the owner; going through `libc` instead would go
  against the recorded choice.
  That is a cost of K and K2 and not a ranking reason.
  The README says the wrapper ships for Linux only, so no Windows work is implied.
- C's difference: a pipe instead of the terminal, and nothing else.
- K2's difference: a pseudo-terminal when the wrapper's own standard error is a terminal, and C's pipe when it is
  not, so both of K2's shapes are built and both are controlled.
- Controls, at least:
  - a typed `clone` at a new host key prints the sentence after Git's lines and ends 128, direct and behind a
    `ProxyCommand` and a `ProxyJump`;
  - a typed `clone` with a locked first key and a working second ends 0 and prints nothing, direct and behind both
    proxies;
  - a typed `clone` with only a locked key prints the passphrase sentence and ends 128;
  - a `pull` at a new host key prints the sentence and ends 1;
  - a clone at `StrictHostKeyChecking=yes` prints the strict sentence;
  - a clone with `BatchMode=yes` prints the general sentence;
  - a clone of a repository that is not there with the host key known prints nothing about keys;
  - a changed key at the default level prints nothing from the wrapper, and the same at `LogLevel=FATAL` prints the
    general sentence;
  - a command that ends 0 prints nothing even when its transcript holds the refusal's words;
  - a server that writes `remote: Host key verification failed.` on a failing command prints nothing, and the same
    under a terminal with `TERM` set prints nothing either;
  - with both streams redirected, a captured transcript is plain Git's, byte for byte, under K2 and under C;
  - with `--no-progress` and `OPOST` cleared, K's captured transcript is plain Git's byte for byte, and a relayed
    line's escape is dropped rather than printed;
  - a hook that leaves a process holding the child's stream does not hold the wrapper;
  - a signal aimed at the wrapper alone does not reach Git, a terminal's control character still reaches Git,
    and the wrapper ends the way Git ended;
  - the LFS upload and the push record print one host-key sentence, not two.

### C2

C2's ranked shape needs the same child and wait, plus D's two starts on the failure path only,
and it needs the source of the person's ssh options that D's section names, or its check reads other files.
Its controls are the cases the shapes get wrong:
a repository that is not there, a changed key, a locked key, a connection refused for a host never seen,
and `git pull` at a new host key.

### D

D needs the resolution and the check that `keep/stand-ins/standin.c` runs, before the process is replaced,
with the sentence printed before `Cloning into`, its condition `ask` alone,
and the source of the person's ssh options for the check.

## The ranking

One ranking,
because the three questions the rankable options turn on are answered:
B and E are "Never",
so no program Git started explains anything,
and J is retired because its value cannot be started.
What is left is K2, K, C, C2, D and A.

Everything is ranked by what each delivers to the person who typed the command:
whether the words arrive at the refusal that stopped them,
whether they name what the person must do,
whether they appear when nothing is wrong,
and what the person gives up on every other command.

K2 > K > C > C2 > D > A, and every adjacent pair's reason is under "The owner's question",
which is where the owner reads it.
The reasons in one line each:
K2 over K because K2's captured stream is plain Git's and K's is not;
K over C because the person at the terminal keeps the display and the touch notice;
C over C2 because C's words fit the refusal and C2's ranked shape is silent at a locked key and at `pull`;
C2 over D because C2 pays its check only after a failure and D pays two starts on every command;
D over A because D says something and A says nothing.

## The measurements

### The runs

The sixth round ran seven container runs of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`), eighteen container starts in all,
each `--rm --init --network=none --cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
each started in a `systemd-run --user` scope with `--slice=agent-work.slice`, `RuntimeMaxSec`, `CPUQuota=300%`,
`MemoryMax=5G`, `TasksMax=512` and `IOWeight=10`.
Its report holds the eighteen starts, and its launch lines were not kept; this round keeps its own at
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/run-12-launch.txt`:

- `keep/run-5-remote-line-under-a-terminal.log`, 18 arms: the `remote: ` drop rule with `TERM=xterm-256color`,
  in a pipe and on a terminal, and the red control without the escape strip;
- `keep/run-6-j-arms-and-transports.log`, 38 arms: J's value direct, behind a `ProxyCommand` and behind `-J`;
  the SIGHUP-ignoring plant on two transports; every option at H1's case behind `-J`; and a new key and a locked
  key behind `-J` for every option;
- `keep/run-7-unreachable-host-and-pull.log`, 27 arms: C2's check and D at a host never seen; `pull` at a new host
  key for every option; `fetch`, `pull` and `push` at a new host key for four options;
- `keep/run-8-whether-the-advice-works.log`, 16 arm lines: eight clones and eight advised commands, so each
  sentence's advice is run and then the clone repeated;
- `keep/run-9-check-without-a-source-and-no-progress.log`, 12 arms: D and C2's check with and without a source for
  the person's ssh options; K's relayed line with and without `--no-progress`;
- `keep/run-10-typed-path-with-every-twin.log`, 12 arms and 24 rotated rounds, every arm with its own twin;
- `keep/run-11-the-words-as-revised.log`, 64 arms: the transcripts this document quotes, printed from the three
  sentence files as revised, at three transports, plus `pull`, a host never seen and the strip control.

This round ran one container, started three times (the second corrected the first's missing `TERM`, the third
added the shipped-shape capture and the revised host-key sentence), and kept the last start:
`~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/run-12-owner-question-arms.log`,
28 arm lines: the six new-key refusals, the three terminal clones, the three captured clones plus the shipped
shape, `git clone -q` and `ssh -q` under C and C2, the merged-stream pull, the five advice arms with the
`GIT_SSH_COMMAND` option, and K's and K2's strip arms.

The four runs and three exploration runs of the round in
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/` are cited in the option
sections by the arm and the log, and are not repeated here.
Every run's script sits beside its log under `keep/`, with `lib.sh`, the three stand-ins and the four sentence
files.

Method, so a difference can be told from this harness's noise:

- every arm ran in every round, and each round started one arm further along the list, so an arm sits at every
  position;
- warm-up rounds ran before the measured ones, and the sixth round's typed-path run warms every arm
  (the script of the round in `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/` read
  `for arm in $ARM_LIST`, which in `/bin/sh` is the list's first element only,
  so only `null-a` was warmed there);
- every arm of the sixth round's typed-path run has a twin, a second arm running the identical command, and the twin's
  per-round difference is the floor that arm is quoted against;
- an arm's figure is the median of its per-round difference from its control in the same round, with the range of
  those differences and the count of positive rounds beside it;
- the typed-path control is `git ls-remote` with the same `ssh`, the same options and `SSH_ASKPASS=false`,
  and the typed-path floor is that same command run twice in the same round;
- the step control is `/usr/bin/true`, and every stand-in arm also pays the `env` exec the control behind its twin
  pays;
- the steps run's `adaptive-start` had no twin, so K2's step figure is quoted against the other arms' floors, and
  that run sent every arm's standard error to `/dev/null`, so its K2 arm ran the pipe shape and its K arm ran the
  terminal shape.

### The four effects of a pseudo-terminal on a redirected command

Run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`.
Every arm cloned into `/work/clones/probe`, removed before each arm, so two arms' transcripts differ only in what the
option changed; every arm ran with `TERM=xterm-256color`.

A refusal, where Git draws nothing, shown through `cat -A` so that a carriage return is visible as `^M`:

```text
##### cat -A refusal-plain
Cloning into '/work/clones/probe'...$
Host key verification failed.^M$
fatal: Could not read from remote repository.$
$
Please make sure you have the correct access rights$
and the repository exists.$
```

```text
##### cat -A refusal-k-pty
Cloning into '/work/clones/probe'...^M$
Host key verification failed.^M^M$
fatal: Could not read from remote repository.^M$
^M$
Please make sure you have the correct access rights^M$
and the repository exists.^M$
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ...
```

```text
##### cat -A refusal-k-pty-raw
Cloning into '/work/clones/probe'...$
Host key verification failed.^M$
fatal: Could not read from remote repository.$
$
Please make sure you have the correct access rights$
and the repository exists.$
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, ...
```

Measured counts for the same refusal:
plain Git 194 bytes and 1 carriage return,
C 645 bytes and 1,
K 651 bytes and 7,
K with `OPOST` cleared 645 bytes and 1,
K2 645 bytes and 1.
The row with `OPOST` cleared is the shipped shape, and it is the one `refusal-k-pty-raw` measures;
`refusal-k-pty-raw` and `refusal-k2-adaptive` compare identical to `refusal-c-relay`, and `refusal-k-pty`, the
stand-in with the output processing left on, does not.

A clone of two thousand objects with both streams redirected to a file, so no terminal on either side:

- plain Git: 37 bytes, no progress line, no carriage return, no escape byte;
- C: the same 37 bytes, identical to plain Git;
- K2: the same 37 bytes, identical to plain Git;
- K, the stand-in with the output processing left on: 8593 bytes, a `Receiving objects` line, a `remote: Counting
  objects` line, a `remote: Enumerating objects` line, 210 carriage returns and 107 escape bytes;
- K with `OPOST` cleared, which is the shipped shape: 8587 bytes, the same three progress lines, 204 carriage
  returns and 107 escape bytes;
- K with `OPOST` cleared and `--no-progress`: 37 bytes, identical to plain Git with `--no-progress`.

That is the four effects, one by one, with the source that gates each:

- the progress display in a captured stream: `transport.c:1230`, `ret->progress = isatty(2)`,
  and `progress.c:146`;
- a carriage return on every line, and a second one on OpenSSH's own line, because the terminal's output
  processing turns each line feed into a carriage return and a line feed and OpenSSH writes its own `\r\n`;
  removed by clearing `OPOST`, which leaves only the carriage returns Git writes for its own progress;
- Git's other terminal-only output: the escape `\033[K` that prefixes every `remote: ` line when standard error is
  a terminal and `TERM` is not dumb (the assignment `prefix = ANSI_PREFIX DISPLAY_PREFIX` is the two lines after the
  `isatty(2)` test, `sideband.c:316` to `:318` at `v2.56.1`), measured as 107 escape bytes in the captured file
  against none under plain Git or C, and still present with `--no-progress` and `OPOST` cleared
  (run, `keep/run-9-check-without-a-source-and-no-progress.log`, arm `n-fail-k-pty-raw-no-progress`: five escape
  bytes, one `\033[K` and four colour sequences from the rejected-ref and `error:` lines, with the relayed line
  printed as `\033[Kremote: ...`).
  So a capture under K gains Git's colour as well as the erase prefix, and the same five escapes appear on K's
  terminal arm where C's has none (`keep/run-5-remote-line-under-a-terminal.log`, arms `r-fail-k-pty-terminal` and
  `r-fail-c-relay-terminal`).
  The editor's waiting hint is gated the same way (`editor.c:70`) and no clone draws it, so that one is source only.
- every descendant's own answer to `isatty(2)`: a `post-checkout` hook prints `hook-stderr-tty: YES` under K and
  `no` under plain Git, C and K2, and a `ProxyCommand` program records `proxy-stderr-tty: YES` under K and `no` under
  the others.

At a terminal (`ptyrun 80 24`):

- plain Git draws the progress lines and 210 carriage returns;
- C draws neither the progress lines nor a second carriage return, and ends with 1 carriage return;
- K and K2 draw the progress lines and 216 carriage returns, the 6 extra being the doubled line ends.

### The end rule

Run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-2-bytes-and-progress.log`.
A `post-checkout` hook starts `(sleep 5) &` with standard error inherited, so a process holds the child's stream after
the hook exits and after Git ends.
A relay that reads until the stream closes takes 5170 ms (C) and 5172 ms (K) for a clone whose child ends in about
200 ms; a relay that ends when the child has exited takes 206 ms (C) and 204 ms (K).

### The steps

Run, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-3-steps.log`, 24 rotated
rounds, and `keep/derived-medians.txt` for every figure here.
Each claim names its control and its own twin's floor.

- The floor, one command run twice: `+0.03 ms` short of zero, range `-1.20` to `+3.56`, 11 of 24 positive
  (`true-twin` against `true-only`).
  With the `env` exec every stand-in pays: median `+0.00 ms`, range `-1.50` to `+2.17`, 12 of 24
  (`env-twin` against `env-true`).
- C2's fork and wait: `+0.68 ms`, range `-0.07` to `+3.75`, 23 of 24 positive (`standin-start` against `env-twin`);
  its twin is `-0.26 ms`.
- C's pipe: `+0.68 ms`, range `-1.40` to `+2.71`, 18 of 24 (`relay-start` against `env-twin`); its twin is
  `-0.03 ms`.
- K's pseudo-terminal: `+0.85 ms`, range `-2.73` to `+2.61`, 20 of 24 (`pty-start` against `env-twin`); its twin is
  `-0.06 ms`.
- K2's adaptive choice: `+0.74 ms`, range `-1.38` to `+2.52`, 20 of 24 (`adaptive-start` against `env-twin`),
  with no twin of its own.
- K against C, paired in the same rounds: `+0.16 ms`, range `-1.68` to `+1.80`, 13 of 24 positive,
  which is not a difference this harness resolves;
  the "about `0.6 ms`" of the revision at `bda96883d5` was the difference of two medians and is withdrawn.
  K2 against C is `+0.12 ms`, range `-1.40` to `+1.38`, 14 of 24.
- `ssh -G` with the configuration D resolves under: `+3.33 ms` over one process start,
  range `+2.40` to `+6.48`, 24 of 24; its twin is `+0.01 ms`.
- `ssh-keygen -F` over a file that holds the host: `+2.38 ms`, range `+1.15` to `+4.23`, 24 of 24;
  its twin is `-0.27 ms`.
- D's whole check, both starts, with the configuration that asks and the file that holds the host:
  `+7.50 ms`, range `+5.23` to `+10.55`, 24 of 24; its twin is `-0.15 ms`.
  The `+8.62 ms` of the revision at `bda96883d5` measured one start, because its fixture short-circuited the
  second.
- `git remote get-url`, the query that names the host as the person typed it when they named a remote:
  `+0.85 ms`, range `-0.59` to `+1.50`, 23 of 24; its twin is `-0.21 ms`.

So the steps run resolves every step except K against C and K2 against C,
which is the one pair this harness cannot tell apart.

### The typed path

Run, `keep/run-10-typed-path-with-every-twin.log`, 24 rotated rounds of `git ls-remote` of one repository over
loopback, with a twin for every arm and a warm-up that reaches every arm.
The control's own median is 195.07 ms and its twin's 200.30 ms.

- the floor: `+4.94 ms`, range `-34.13` to `+35.51`, 15 of 24 positive;
- C: `+5.35 ms`, same-twin floor `-2.59 ms`, 16 of 24, inside the floor;
- K: `+0.31 ms`, same-twin floor `-2.32 ms`, 12 of 24, inside the floor;
- K2: `+5.44 ms`, same-twin floor `-8.59 ms`, 13 of 24, inside the floor;
- C2's status-only shape: `+4.44 ms`, same-twin floor `+0.03 ms`, 15 of 24, inside the floor;
- D with both starts: `+20.08 ms`, same-twin floor `-6.64 ms`, range `-12.06` to `+76.25`, 22 of 24,
  the only figure this harness resolves.

The other typed-path run, whose log is
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/run-4-typed-path.log`,
measured a floor of `+0.11 ms` and D at `+5.76 ms` with 23 of 24 positive, and resolved nothing else.
Its warm-up reached one arm only, which the rotation spreads over the arm list rather than over the control alone,
and both runs agree on the conclusion.

### What the figures support, and what they do not

- C, K, K2 and C2 all add between half a millisecond and one and a half milliseconds of work to a command that
  meets no question, and no typed-path harness here can tell them apart.
- D costs about `8.4 ms` of work in two starts and one read on every typed network command, which is the only
  option cost a person would feel on a fast connection.
- Every figure is a loopback round trip inside a two-CPU container on this machine.
  A person's network dominates all of them, so the numbers say what the wrapper adds, not what a command takes.
- The two typed-path runs disagree by a factor the sixth round did not chase: the sixth round's floor is `+4.94 ms`
  with a range of `-34.13` to `+35.51` against the other's `+0.11 ms` with `-6.18` to `+7.93`, and the sixth round's
  absolute medians are about 195 ms against that run's about 127 ms.
  The host was under memory pressure during the sixth round's run, which is normal here and is not an excuse for a
  claim: what the sixth round's run supports is that only D is resolved, and that is what both runs say.
- The figures do not transfer to another machine, another container, or another release of Git or OpenSSH.
  That is why every figure here is quoted against a twin measured in its own rounds.

## What the probes did not show

- No run of the wrapper itself.
  No wrapper code was changed this round, so every option is stood in for by a compiled stand-in
  (`keep/stand-ins/standin.c`), and every sentence printed is one of the words under "The words".
  What the arms prove is the delivery, the decision and the cost, not the wrapper's reading of a repository.
- Nothing ran under this host's own `ssh` except the arms of the round whose folder is
  `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/`.
  The `git` profile attaches to `/usr/bin/git` and stacks `&git//ssh` for `/usr/bin/ssh`
  (`/etc/apparmor.d/git`, read this round: `@{bin}/ssh rcx -> &git//ssh`), and that stack refuses the shell a
  `ProxyCommand` needs:
  the two `f-confined-proxycommand` arms of that round failed with `/bin/sh: Permission denied`,
  while the direct arms ended 0 (run, that round's `keep/run-1-behaviour.log`).
  So no proxy arm can run under this image's own `ssh` here,
  and every arm of this round uses a copy of `ssh` at a path no attachment names,
  which runs under the `git` profile alone and may execute anything under `/work`
  (`/etc/apparmor.d/local/git`, read this round: `owner /work/** mix`).
  On this host the H1 failure cannot be produced,
  and the H1 arms describe a host whose `ssh` carries no such stack.
- No security key.
  The touch notice's behaviour under a pipe or a pseudo-terminal is read from `readpass.c:252`, `:253` and `:279`,
  not measured: no key was available.
- No signal aimed at the wrapper alone.
  The stand-in runs no `setsid` and no `TIOCSCTTY`, so the arms' signals are the person's,
  but no arm sends a signal to the wrapper only, so what the wrapper must do with one is design.
- No arm with `Match exec`, `CanonicalizeHostname`, `HostKeyAlias` or `VerifyHostKeyDNS`.
  The `KnownHostsCommand` and `@cert-authority` results are other rounds' and are quoted in D's section.
- One arm merged the two streams of a command under K2: a fast-forward `git pull` into one file, identical to plain
  Git's byte for byte (`o-merged-plain`, `o-merged-k2`).
  That is one command and one order; a stderr line Git writes before a stdout line can still be copied after it by a
  relay that reads before it writes, and no arm forced that order.
- No arm measured `ssh-add` with an agent, and no arm could measure a locked key's passphrase.
- No Windows, no OpenSSH release other than 10.6p2, and no Git release other than 2.56.1.
- `ssh -G` from the person's own configuration was measured in the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`; `CanonicalizeHostname` was not run.
- No whole container gate, and no `native:check`, clippy, `lint:types` or host tests.
  The only repository change is documentation, and the round's note says no whole gate is a merge condition.
- The Markdown lint did not run.
  `monochromatic-lint` is not installed in this worktree and `mise run toolchain:install` is barred this round;
  the document was rendered with `pandoc -f gfm -t html` and read for what a renderer shows.

## The record of the rounds before this one

The rounds that led here, named by the folder they wrote to and the commit that merged each revision,
because the questions they turned on are answered and their evidence is still cited in each option's section.

### The revision at `1f6f120f64` (2026-11-24T09:01:38-05:00)

Its findings, with the evidence against the ones that did not hold:

1.  B's transcript was not the run's bytes.  The round that followed re-ran every B arm with the kept stand-in and
   printed the bytes.
2.  The owner's self-start rule was not quoted, and B and E meet it.
    The rule is quoted under "The rule that forbids the question";
   the owner then answered "Never" for both.
3.  The rotation was not a rotation and the noise test was empty.  The timings were re-run in a balanced Latin
   square, and the test became a sign in every round plus a median larger than a same-command twin.
   This round keeps the twin and states the positive count instead of requiring every round.
4.  The `-G` start count was wrong for the baseline.  A named `ssh` is started once, a differently named program
   twice; E's arm was relabelled and split.
5.  C2's arm did not show what the text said.  The revision at `bda96883d5` ran a real check, and this one
   runs all four shapes of it.
6.  The coverage sentence in the B over C pair was wrong in both directions.  B covers the differing key; C also
   speaks for `BatchMode=yes` and `StrictHostKeyChecking=yes`.
7.  D has a false positive the round at `1f6f120f64` did not find.  Reproduced with `KnownHostsCommand`, and the
   `@cert-authority` guess did not hold.
8.  J's example named a host J cannot know.  Its value was rebuilt with a placeholder.
9.  The selector's supporting claims were partly wrong.  The file-name fallback and the bare-word claim were
   dropped.
10. "Nothing else is needed" for AppArmor was inference, and now says so.
11. The "byte for byte" claim was not measured.  It was measured then, and again in the sixth round for C, K2 and K with
    its fixes.
12. Stale line numbers.  Cites are now a heading, a symbol, or a path with the sentence quoted.
13. A misquoted comma in the old handover's sentence.
14. Printed commands were unquoted, and OpenSSH cuts a key path at 100 characters.
15. The panic hook ran after the mode would sit, and "flush" was the wrong mechanism.
16. A typed LFS push would say the same thing twice.
17. A container outlived the report of the round that started it.
    It is not the sixth round's, and the sixth round left both it and
    `typed-network-next-step-probe-2026-11-24` alone, because both predate it.

### The revision at `bda96883d5` (2026-11-25T13:54:27-05:00), merged as `ec0b9e7ea3`

That revision retired J and added K.
Its words are kept here because a gate review found that they could not be printed from what K reads,
and because what replaced them is the evidence that the finding was right.

Its host-key sentence:

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

Its passphrase sentence:

```text
monochromatic-git-policy-cli: this command needed the passphrase of the key `/work/key_locked`, and no command the wrapper runs asks a person for one, so that key was skipped. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /work/key_locked` before the next command, use a key without a passphrase, or have the server accept another key you hold.
```

Three things were wrong with them, and each is fixed under "The words":

- the address `[127.0.0.1]:2222` came from the fixture,
  and a real transcript holds no address:
  the fixture's `target` carries `HostName localhost`, so the key it records is `[localhost]:2222`,
  and advice naming `127.0.0.1` would have the person accept a second key for the same machine;
- the key path `/work/key_locked` came from the fixture,
  and `Permission denied (publickey).` carries no path at all;
- the passphrase sentence claimed a cause the transcript cannot show,
  since that line arrives for a key the server refuses as well as for one nobody could unlock.

Its C2 shape was one option where there are four, its ranked pair reasons compared things that are not alike,
and its D cost arm never ran D's second start.
All three are answered in the sections named with them.

### The revision at `79962f9ccc` (2026-11-25T15:57:22-05:00), merged as `800db64aa1`

The gate review kept at
`~/temp/agent/gate-review-host-key-next-step-fifth-revision-2026-11-25/keep/review.md`
answered it with nine Medium findings and a list of Low findings.
The revision that followed it (`cfbfa7aaf6`, 2026-11-25T17:20:11-05:00) answered all of them, and the git history
holds that change log.
Nothing it found reverses K2 first.

## References

- `package/git-policy/cli/src/native/invocation/child_environment.rs` (`no_prompt_overlay`, `NO_ANSWER_PROGRAM`,
  `ssh_no_answer_overlay`).
- `package/git-policy/cli/src/native/invocation/forwarding.rs` (`run_real_git`, `replace_process_with_real_git`,
  `end_as_git_ended`).
- `package/git-policy/cli/src/native/invocation/entry.rs` (the `Action::Forward` and `Action::ForwardCoordinated`
  arms), `invocation/action.rs`, `invocation/entry_run_forwarded_actions.rs`,
  `invocation/entry_forward_action_planning.rs`, `worktree_copying/worktree_copy_forward.rs`
  (`run_forwarded_git`), `merge/pull_isolation.rs` (`pull_step`), `lock/index_writer_forward.rs`,
  `lock/index_writer_commands.rs`.
- `package/git-policy/cli/src/native/lfs/lfs_pre_push.rs` (`reads_as_unknown_host_key`,
  `reads_as_a_key_failure`, `reads_as_a_login_failure`, `login_needed_message`, `host_key_not_known_sentence`,
  `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`) and `lfs/lfs_upload_explanations.rs` (`explanation_after_failure`).
- `package/git-policy/cli/src/native/push/push_retry_failure.rs` (`retried_failure_record`).
- `package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs` (the rules and their
  exceptions), `src/native/invocation/real_git.rs` (`own-executable`'s exception).
- `package/git-policy/cli/Cargo.toml` (`libc` at `:162`, `rustix`'s features at `:166`).
- `package/git-policy/cli/SPEC.md`, "No command the wrapper runs or forwards asks a person for a credential",
  "The wrapper never starts itself as Git", and the paragraph saying this release names no next step at a new host key
  (`:9460`).
- `package/git-policy/cli/README.md`, the paragraph on prompts and `SSH_ASKPASS`.
- `doc/planning/cli-git-no-self-invocation.md`, "The rule".
- `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md` (the answers "Never" for B and E, and
  "J" for the option that was then retired).
- `doc/decision/cli-git-alpha-46-release-pass-four-answers.md`, "What `SSH_ASKPASS` holds".
- `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- `doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`.
- `doc/handover/cli-git-rust-implementation.md:1224`, the only handover that names the open question.
- `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
- `/etc/apparmor.d/git`, `/etc/apparmor.d/local/git` and `/etc/apparmor.d/local/git_ssh`, read this round.
- OpenSSH `V_10_6_P2`: `readpass.c`, `sshconnect.c`, `sshconnect2.c`, `ssh.c`, `clientloop.c`, `log.c`, `log.h`,
  `readconf.c`, `mux.c`.
- Git v2.56.1: `pager.c` (`term_columns`), `progress.c` (`is_foreground_fd`, the progress gate),
  `sideband.c` (the `remote: ` prefix), `transport.c` (`ret->progress`), `editor.c` (the waiting hint),
  `connect.c`, `prompt.c`, `strbuf.c`.
- The sixth round's folder, `~/temp/agent/host-key-next-step-design-sixth-revision-2026-11-25/keep/`:
  `run-5-remote-line-under-a-terminal.log` and its script,
  `run-6-j-arms-and-transports.log` and its script,
  `run-7-unreachable-host-and-pull.log` and its script,
  `run-8-whether-the-advice-works.log` and its script,
  `run-9-check-without-a-source-and-no-progress.log` and its script,
  `run-10-typed-path-with-every-twin.log` and its script,
  `run-11-the-words-as-revised.log` and its script,
  `probe-insteadof.log` and its script, `lib.sh`,
  `stand-ins/standin.c`, `stand-ins/proxy.c`, `stand-ins/ptyrun.c`,
  `sentences/host-key.txt`, `sentences/strict.txt`, `sentences/passphrase.txt`, `sentences/j-value.txt`,
  `sentences/remote-line.txt`, `derived-medians.txt`, `derive-medians.py` and `state-so-far.md`.
- This round's folder, `~/temp/agent/host-key-next-step-design-seventh-revision-2026-11-25/keep/`:
  `run-12-owner-question-arms.log` and its script, `run-12-launch.txt`,
  the same `lib.sh`, `stand-ins/standin.c`, `stand-ins/proxy.c` and `stand-ins/ptyrun.c`,
  and `sentences/host-key.txt`, `sentences/strict.txt`, `sentences/passphrase.txt` and `sentences/remote-line.txt`.
- The round whose logs carry the behaviour, the four effects, the steps and the typed path:
  `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/`.
- The gate review this revision answers:
  `~/temp/agent/gate-review-host-key-next-step-sixth-revision-2026-11-25/keep/review.md`.
- The round that measured `pull` at a new host key, the `ssh -q` arms, E's cost and `Match exec`:
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/`.
- The round that measured `KnownHostsCommand`, `@cert-authority` and the 369-byte J value:
  `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/`.
- The round that measured B's transcripts, the 571-byte J value and the confined arms:
  `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/keep/`.
- The release pass's finding and its container run:
  `~/temp/agent/release-pass-wrapper-alpha-46-2026-11-25/commit-push-lfs-lock-and-the-rest/keep/review.md`
  (H1 and "For the owner"), and the OpenSSH sources kept beside it.
