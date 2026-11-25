# The next step a typed network command names at a new host key or a missing passphrase

## Status

A proposal,
put to the owner.
This is its fifth revision,
written after a gate review of the fourth,
which said "Not ready to put to the owner as written" and named what to fix
(`~/temp/agent/gate-review-host-key-next-step-after-askpass-false-2026-11-25/keep/review.md`).
Nothing here is a decision,
nothing in it is approved or final,
and this revision is not final either.
Only the owner declares anything complete.

It designs the words a typed `git clone`,
`fetch`,
`pull` or non-LFS `push` can print when OpenSSH refuses a host key,
or when a key needs a passphrase nobody can be asked for.
The owner's two answers "Never" for B and E stay
(`doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`),
and so does the rule against the wrapper starting itself
(`doc/planning/cli-git-no-self-invocation.md`).
`main` holds `SSH_ASKPASS=false` again
(`c336355bcd`, 2026-11-25T13:21:59-05:00;
`NO_ANSWER_PROGRAM` in `package/git-policy/cli/src/native/invocation/child_environment.rs`),
because the owner answered "Put `false` back (Recommended)"
(`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`).
That is the state every option here starts from.

Worktree:
`.claude/worktrees/host-key-next-step-design-after-its-gate-review`,
branch `docs/planning-host-key-next-step-design-after-its-gate-review`,
made from `main` at `90848c599c`.
Round folder:
`~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/`,
whose `keep/` holds this round's four container logs and their scripts,
the stand-ins they ran,
the sentences they printed,
and the derived medians.
Only this file changed in the repository,
and nothing under `doc/decision/` or `doc/handover/` was touched.

Every transcript is labelled,
because a read of an earlier revision found a round printing text its run had not printed:

- run means a program printed these bytes, and the log holding them is named with it;
- composed means no program printed them, and the text was put together from other facts;
- source means read from a file, at the tag or path named with it;
- unrun means neither.

## The decision, short

The person at a new host key or at a key no agent can unlock reads OpenSSH's line,
Git's `fatal:` line and nothing else,
and the wrapper adds no word of its own.
There is no next step in what they see,
and the way out is knowledge the words do not give.

Six deliveries were ranked,
by what each gives that person:

- K2, a pseudo-terminal on the child's standard error when the wrapper's own standard error is a terminal,
  and a pipe otherwise (ranked first);
- K, the same pseudo-terminal always;
- C, always a pipe;
- C2, Git kept as a child with the person's own streams, a sentence chosen after exit 128
  (in three shapes, of which one is ranked);
- D, a known-hosts check before Git starts;
- A, nothing, which is `main`.

The ranking is **K2 > K > C > C2 > D > A**,
and every adjacent pair's reason is under "The ranking".

The costs,
as the steps each one adds to a command that meets no question,
measured against the same command twice in the same rounds
(run, `keep/run-3-steps.log`; every figure is in `keep/derived-medians.txt`):

- the floor, one command run twice: median `-0.03 ms`, range `-1.20` to `+3.56`, positive 11 of 24;
- C2's fork and wait: `+0.68 ms` over the control, positive 23 of 24;
- C's pipe: `+0.68 ms`, positive 18 of 24;
- K's pseudo-terminal: `+0.85 ms`, positive 20 of 24;
- K2's adaptive choice: `+0.74 ms`, positive 20 of 24;
- K against C in the same rounds: `+0.16 ms`, range `-1.68` to `+1.80`, positive 13 of 24,
  which is not a difference this harness resolves;
- `ssh -G`: `+3.33 ms`, positive 24 of 24;
- `ssh-keygen -F`: `+2.38 ms`, positive 24 of 24;
- D's whole check, both starts, with a configuration that asks and a known-hosts file that holds the host:
  `+7.50 ms`, positive 24 of 24;
- the query that names the host as the person typed it, `git remote get-url`: `+0.85 ms`, positive 23 of 24.

On a typed command the same options are about two milliseconds of work against a network round trip,
and that harness resolves none of them:
its floor is `+0.11 ms` with a range of `-6.18` to `+7.93` (run, `keep/run-4-typed-path.log`).
Only D, at `+5.76 ms` with 23 of 24 positive, is larger than that floor.

What K changes in the bytes a command writes,
all four effects measured this round
(run, `keep/run-2-bytes-and-progress.log`):

- the progress display appears in a captured stream: 9127 bytes against plain Git's 37;
- every line gains a carriage return, and OpenSSH's own line gains a second one (7 against 1 on a refusal);
- Git's other terminal-only output appears there too: 107 escape bytes against none;
- every descendant that asks whether its standard error is a terminal is told yes: a hook's `test -t 2` prints
  `hook-stderr-tty: YES`, and a `ProxyCommand` program records `proxy-stderr-tty: YES`, where plain Git and C give
  `no`.

Two of those are removable by a shipped K (the doubled carriage return, by clearing the terminal's output
processing), and K2 removes all four for anyone who captures output.
A shipped K that clears that processing also lets the person have plain Git's bytes back:
with `--no-progress`, its transcript is plain Git's, byte for byte (run, same log, `BYTES ... identical=yes`).

## What this revision changed

The gate review's findings, and what each one became.
The review is not repeated here;
its own text is at the path under "Status".

The two findings it called High:

- The words K and C were shown printing cannot come from the transcript.
  The host-key sentence now says the machine does not know the key or holds an outdated one,
  which the transcript can support,
  and names the host as the person typed it,
  which the wrapper knows from the command and not from the transcript:
  `ssh://target/work/served.git` gives `target`.
  The passphrase sentence now uses the hedged wording the wrapper already ships for a key failure
  ("refused or could not be unlocked", with `ssh-add -l` to rule the second out,
  `login_needed_message` in `src/native/lfs/lfs_pre_push.rs`),
  and names no key path,
  because the transcript holds none.
  The differing-key sentence is gone:
  it needs the question's text, which only B reads.
  The `f-newkey` and `f-locked` arms were re-run with these exact sentences
  (run, `keep/run-1-behaviour.log`, families `f-newkey-ask` and `f-locked`).
  The cost of reading the remote is measured (run, `keep/run-3-steps.log`, arm `remote-url`).
- K changes the bytes a redirected command writes, and the document disclosed one effect of four.
  All four are disclosed, named and measured, with the transcripts shown through `cat -A`,
  and a shipped K's termios handling is in its build list.
  The third effect needed `TERM` set: without it Git treats the terminal as dumb and skips the escape prefix,
  which is why the earlier round's arms never showed it.

Its third High:

- The variant that removes K's redirected-stream cost was never ranked.
  It is ranked, as K2,
  with its own arms at a terminal and at a redirect,
  and C2 is split into its three shapes,
  of which the question names one.

Its Medium findings:

- D's cost arm never ran D's second start.
  Relabelled:
  the family that broke J is `f-noask`, "strict checking off, no known-hosts file",
  and D's cost arm now runs an `ask` configuration with a known-hosts file that holds the host,
  so both starts run (run, `keep/run-3-steps.log`, arm `precheck-ask-known`).
  A family with the host key known and the setting asking now runs too (`f-ask-known`),
  and so does `f-accept-new`, where a clone with `StrictHostKeyChecking=accept-new` succeeds.
- K's "misses" list contradicted the detectors.
  Corrected and measured:
  `BatchMode=yes` and `StrictHostKeyChecking=yes` do make K and C speak,
  the strict sentence now has words of its own,
  and the `LogLevel=FATAL` case where a changed key is read as an unknown one is named with its measurement.
- Three of the four adjacent-pair reasons were not true as written.
  Every pair is rewritten with a reason the evidence supports,
  including what C2 keeps that C loses,
  which C2 shape the pair means,
  and what D's wrong sentence costs against A's silence.
- Cost and noise statements went beyond what the runs support.
  Every cost now names its control, its twin, its per-round spread and its positive count,
  the `0.6 ms` pseudo-terminal difference is withdrawn as unresolved,
  and the `255 through ssh` figure is attributed to the release pass that measured it.
- A shipped K or C needs a gate the stand-in did not have.
  The stand-in now speaks only when the child ended non-zero,
  drops the lines Git relays from the remote before reading,
  and two controls show both:
  a push the server writes the refusal's words on and that succeeds, and the same line on a push that fails.
- K's scope and build list were under-specified against the code.
  The build list now names every child path the relay must reach,
  the transport question with its two answers,
  the relay's end rule with the measurement that shows what it prevents,
  and the signals.
- Present-tense statements about `main` were not yet true of `main`.
  They are true now, and the text says so without hedging:
  `main` holds `false`, and `SPEC.md` and `README.md` say the release names no next step at a new host key.

The Low findings are answered where they live:
the self-start rule is quoted with the paragraph it has now,
J's two byte counts are separated and named,
the confined arms say which two options ran under this host's own `ssh`,
the stale "two more limits" list is gone,
the localisation question has its own sentence,
the locked key now runs behind a `ProxyCommand` as well as directly,
and the prose was put through the project's Markdown rules once.

## The question, open again: the value J put in `SSH_ASKPASS` ended connections that worked

J put one sentence into `SSH_ASKPASS`,
on the reasoning that a value OpenSSH cannot execute starts no program and prints the sentence inside OpenSSH's own
failure line.
What the code also does,
read from OpenSSH `V_10_6_P2`:
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

That is what the alpha.46 release pass found, and it is why the owner retired J.
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
(C, the wrapper reads Git's error output; C2, a paragraph after every exit 128; D, a `known_hosts` check before Git
runs);
that choice goes to the owner separately, with the proposal".

So the question is the one this document ranks,
with three answers fixed:
B and E stay "Never",
and no option may hand OpenSSH a value it cannot start,
so `SSH_ASKPASS` stays `false` and `SSH_ASKPASS_REQUIRE` stays `force` on `main`.

This round ran the case H1 names in a capped container with a real `sshd` and OpenSSH 10.6p2,
and ran every remaining option against it.
Run (`keep/run-1-behaviour.log`, families `f-noask` and `f-ask-known`):
one served repository,
the first key offered with a passphrase nobody can give and a later key that works,
and the connection through a relay `ProxyCommand` or through `-J`:

- A: exit 0, the clone complete, nothing extra printed;
- J's sentence: exit 128 through Git, the line
  `ssh_askpass: exec(monochromatic-git-policy-cli: ...): File name too long`, then
  `fatal: Could not read from remote repository.`;
- the same sentence, with a relay that ignores SIGHUP: exit 0 and the clone complete, which names the signal as the
  cause;
- C, K, K2, C2 and D: exit 0, the clone complete, and nothing extra, because none of their conditions became true.

The `255 through ssh` figure for J's sentence is the release pass's measurement of `ssh` itself,
not this round's:
this round's arms run Git and report its status,
and those four numbers are what this round's log holds.

One thing this host adds, and every proxy arm says so:
the `git` profile attaches to `/usr/bin/git` and stacks `&git//ssh` for `/usr/bin/ssh`
(`/etc/apparmor.d/git`, read),
and that stack refuses the shell a `ProxyCommand` needs:
the two `f-confined-proxycommand` arms of the round whose folder is
`~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/` failed with
`/bin/sh: Permission denied`,
while the direct arms ended 0 (run, that round's `keep/run-1-behaviour.log`).
So no proxy arm can run under this image's own `ssh` here,
and every other arm uses a copy of `ssh` at a path no attachment names,
which runs under the `git` profile alone and may execute anything under `/work`
(`/etc/apparmor.d/local/git`, read: `owner /work/** mix`).
On this host the H1 failure cannot be produced,
and the H1 arms describe a host whose `ssh` carries no such stack.

## What the person sees today

These are the lines a person meets on `main`,
because `SSH_ASKPASS` holds `false` again (`NO_ANSWER_PROGRAM` in
`src/native/invocation/child_environment.rs`),
so the wrapper adds no line of its own at either refusal.
Nothing changed in these transcripts when J was built and then removed:
they were measured with `false`, which is the value `main` carries.

Measured in a rootless container of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`, `OpenSSH_10.6p2`, `git 2.56.1`)
against a throwaway `sshd` on loopback,
with a fixture host key,
a fresh `known_hosts` and throwaway client keys,
and the wrapper's two variables as `main` holds them.
The commands are copies under `/work`,
which no AppArmor profile attaches to,
so the arms show OpenSSH's own behaviour and not this host's confinement.

A new host key, for `git clone` (run, `keep/run-1-behaviour.log`, arm `f-newkey-ask-direct-a-wrappers-value`):

```text
# git clone ssh://target/work/served.git
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

A key whose passphrase nobody can be asked for, with the host key known
(run, `keep/run-1-behaviour.log`, arm `f-locked-direct-a-wrappers-value`):

```text
# git clone ssh://target/work/served.git
Cloning into 'probe'...
root@localhost: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

A changed host key, at the default log level, ends 128 with OpenSSH's own banner,
`Offending ED25519 key in /work/kh-stale:1`,
`Host key for [localhost]:2222 has changed and you have requested strict checking.`
and the same two Git lines
(run, `keep/run-1-behaviour.log`, arm `f-changed-direct-a-wrappers-value`).
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

The owner's question for this waits in
`doc/handover/questions-for-the-owner.md`,
section "Words for a typed command at an unknown host key or a missing passphrase (the pass's L4)",
which this round may not edit;
it is also under "The owner's question" here and in this round's report.

## The case every option is tried against

Every option in this document was run at the case H1 names,
and at the refusals a next step is meant for,
in one capped container with a real `sshd` and the OpenSSH this image carries.
Run (`keep/run-1-behaviour.log`, script `keep/run-1-behaviour.sh`, stand-ins
`keep/stand-ins/standin.c`, `keep/stand-ins/proxy.c` and `keep/stand-ins/ptyrun.c`).

Container:
`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`, `--rm`, `--network=none`,
`--cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
started in a `systemd-run --user` scope with `RuntimeMaxSec=1500`, `CPUQuota=300%`, `MemoryMax=5G`,
`TasksMax=512` and `IOWeight=10`, named `host-key-review-run1`.
OpenSSH 10.6p2, Git 2.56.1, `sshd` on 127.0.0.1:2222 with a fixture host key.

### The setup a person would have

- two client keys in `authorized_keys`: `key_locked`, whose passphrase is nobody's to give, listed first,
  and `key_good`, which works;
- a served repository at `ssh://target/work/served.git`,
  where `target` resolves to `localhost:2222` and offers the two keys in that order with `IdentitiesOnly yes`;
- a relay `ProxyCommand` (a compiled program, `keep/stand-ins/proxy.c`), and a `ProxyJump` through
  `root@127.0.0.1:2222`;
  the jump host is `127.0.0.1` and the destination `localhost`, because `ssh` refuses a jump whose host, port and user
  equal the destination's (`ssh.c:1289-1291` at `V_10_6_P2`, read);
- the wrapper's two variables as `main` holds them:
  `SSH_ASKPASS=false` and `SSH_ASKPASS_REQUIRE=force`;
- an `ssh` at `/work/bin/ssh`, a copy of the image's own, for the reason under "The question, open again".

### The twelve families

The families differ only in what a person's settings and the server make true.
The first three ran in more than one transport;
the rest ran directly.

- `f-newkey-ask`: the host key is unknown and `StrictHostKeyChecking` is `ask`.
  Direct, behind the `ProxyCommand`, and behind `-J`.
- `f-locked`: the host key is known and only the locked key is offered.
  Direct and behind the `ProxyCommand`.
- `f-noask`: strict checking off and no known-hosts file at all,
  with the locked key offered before the working one.
  This is the case that broke J. Direct and behind the `ProxyCommand`.
- `f-ask-known`: the host key is known and the setting is `ask`,
  with the locked key offered before the working one:
  the same keys as `f-noask`, with the host key known, so D's decision is exercised.
- `f-known-missing`: the host key is known and the repository is not there:
  a failure that is not about a key at all.
- `f-strict-yes`: the host key is unknown and `StrictHostKeyChecking` is `yes`,
  so OpenSSH prints its own strict sentence and asks nothing.
- `f-batchmode`: the host key is unknown and `BatchMode` is `yes`, so nothing is asked.
- `f-accept-new`: the host key is unknown and `StrictHostKeyChecking` is `accept-new`,
  so the clone succeeds and the key is added.
- `f-changed`: the file holds a different key for the destination.
- `f-changed-fatal`: the same, at `LogLevel FATAL`, where OpenSSH prints the last line alone.
- `f-remote-ok`: a push whose server-side hook writes `Host key verification failed.` on its standard error,
  which Git relays as a `remote: ` line, after which the push succeeds.
- `f-remote-fail`: the same line on a push the same hook then refuses.

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
- `d-precheck-acceptnew`: the same shape as the earlier revision had it,
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
  and it drops the lines Git prefixes with `remote: ` before it reads,
  both of which a shipped C, K or K2 needs (the controls are under "The measurements").
- D's arm runs `ssh -G` and `ssh-keygen -F` for real.
  Its check reads `stricthostkeychecking` and prints only for `ask`,
  so `accept-new` gets no sentence from the fixed arm.
- No arm has `Match exec`, `CanonicalizeHostname`, `KnownHostsCommand`, `HostKeyAlias` or `VerifyHostKeyDNS` set.
  The round whose folder is `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`
  measured `KnownHostsCommand` (which makes D speak while the clone succeeds),
  and the round whose folder is `~/temp/agent/typed-network-command-next-step-after-askpass-false-2026-11-25/`
  measured `@cert-authority` (which does not);
  those results stand and are quoted in D's section.
- No arm ran with a signal sent to the wrapper alone, and none ran on Windows.

## What each option gave, family by family

Run (`keep/run-1-behaviour.log`;
the exit statuses, the decision each arm logged and the lines are that log's;
`keep/derived-run-1-summary.txt` holds the same as one line an arm).

At a new host key, directly and behind both proxies:

- `a-wrappers-value`: exit 128, OpenSSH's line, Git's lines, and no next step;
- `c-relay`, `k-pty`, `k2-adaptive`: exit 128, the same lines, then the host-key sentence;
- `c2-status`, `c2-check-unknown`, `c2-check-both`: exit 128, then the host-key sentence,
  which is right here;
- `c2-check-known`: exit 128 and nothing, because the check finds no key and this shape speaks only when it finds one;
- `d-precheck`: exit 128, the host-key sentence **before** `Cloning into`, then OpenSSH's and Git's lines.

At a key nobody can unlock, with the host key known, directly and behind the proxy:

- `a-wrappers-value`: exit 128, `root@localhost: Permission denied (publickey).`, Git's lines, nothing else;
- `c-relay`, `k-pty`, `k2-adaptive`: exit 128, then the passphrase sentence, which fits what the transcript says;
- `c2-status`: exit 128, then the **host-key** sentence, which is wrong here;
- `c2-check-unknown`: silent, so the refusal the person needs words for gets none;
- `c2-check-known`, `c2-check-both`: exit 128, then the passphrase sentence, which is right here;
- `d-precheck`: silent, because the key is known and D never sees a passphrase.

At `f-noask` and `f-ask-known`, the case that broke J:

- every arm ends 0 and the clone completes;
- `k-pty` writes 420 bytes on the captured stream where `c-relay` writes 54 and plain Git 63,
  which is the progress display, and nothing else differs;
- no arm prints a sentence, because no refusal reached it.

At a repository that is not there with the host key known:

- `a-wrappers-value`, `c-relay`, `k-pty`, `k2-adaptive`, `d-precheck`: silent, which is right;
- `c2-status`: exit 128, Git's lines, then the **host-key** sentence, which is wrong;
- `c2-check-unknown`: silent, which is right here;
- `c2-check-known`, `c2-check-both`: exit 128, Git's lines, then the **passphrase** sentence, which is wrong:
  no key was involved.

At `StrictHostKeyChecking=yes`:

- `a-wrappers-value`: OpenSSH's strict sentence and the last line;
- `c-relay`, `k-pty`, `k2-adaptive`: the strict sentence, whose advice overrides their own setting for one
  connection;
- `c2-status`, `c2-check-unknown`, `c2-check-both`: the host-key sentence, whose advice plain `ssh target` cannot
  carry out under that setting, because plain `ssh` reads the same configuration;
- `c2-check-known`: silent;
- `d-precheck`: silent, and right, because no question will be asked at all.

At `BatchMode=yes`:

- `a-wrappers-value`: the last line alone;
- `c-relay`, `k-pty`, `k2-adaptive`, `c2-status`, `c2-check-unknown`, `c2-check-both`: the host-key sentence,
  whose advice is right for this person: plain `ssh target` asks unless it is given `BatchMode` too;
- `c2-check-known`: silent;
- `d-precheck`: the host-key sentence before Git starts.

At `accept-new`:

- every arm ends 0 and the clone succeeds;
- `d-precheck` stays silent, because the setting is not `ask`;
- `d-precheck-acceptnew` prints the host-key sentence on a clone that then succeeds,
  which is the false positive the earlier design's `ask` or `accept-new` condition had.

At a changed key, at the default log level:

- `a-wrappers-value`: OpenSSH's banner, which names the offending line;
- `c-relay`, `k-pty`, `k2-adaptive`, `d-precheck`, `c2-check-unknown`: silent, which the detector's positive reading
  requires and which leaves the person OpenSSH's own words about the key that changed;
- `c2-status`: the host-key sentence, which is wrong: the machine knows a key for that host;
- `c2-check-known`, `c2-check-both`: the passphrase sentence, which is wrong.

At the same changed key with `LogLevel=FATAL`:

- `a-wrappers-value`: `Host key verification failed.` alone;
- `c-relay`, `k-pty`, `k2-adaptive`: the host-key sentence, which the detector cannot tell from an unknown key's
  transcript;
- `c2-check-known`, `c2-check-both`: the passphrase sentence, which is wrong.

At the remote-written line:

- `f-remote-ok`: exit 0, the transcript holds `remote: Host key verification failed.`, and no arm prints a
  sentence, with or without the drop rule, because the status gates it;
- `f-remote-fail`: exit 1 with the same line in the transcript;
  `c-relay` and `k-pty` print nothing, and the same arm with `STANDIN_DROP_REMOTE=0` prints the host-key sentence,
  which is the arm without the rule and names what stops it.

K2, when the wrapper's own standard error is not a terminal, is C byte for byte:
run with the same clone directory, `c-relay` and `k2-adaptive` transcripts compare identical
(run, `keep/run-2-bytes-and-progress.log`, `BYTES refusal-k2-adaptive refusal-c-relay identical=yes`);
run with the arm's own directory in the path, the two transcripts differ by exactly the four characters the two arm
names differ by (667 and 671 bytes, run, `keep/run-1-behaviour.log`).
At a terminal K2 gives what K gives:
progress lines and the touch notice's terminal, measured under "The measurements".

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

Two facts about that rule matter to this design:

- the SPEC's parenthetical still points at the handover entry,
  which no longer exists;
  the rule's own text is the record now.
- `SPEC.md` also says "a host key is not a credential the push lacked",
  so the credential rule does not itself forbid answering a host-key question.
  What forbids it is the owner's answer of 2026-11-20 that the refusal is explained rather than answered
  ("A host key this machine does not know is no stop: the service keeps its usual retry schedule for it,
  and its record says that it pushes the branch by itself once the key is known"),
  and this design's own brief.
  Every option here refuses to answer yes;
  the ones that could answer yes are listed and not ranked.

## What the recorded rules leave open

The owner's self-start rule, from `doc/planning/cli-git-no-self-invocation.md`,
an excerpt from its section "The rule":
the owner's instruction and the three bullets that make the rule as built are quoted whole,
and the first bullet's ZED exception is left out,
because it is about a file name selecting a root mode and bears on none of these options
(`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`).

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
  or through dispatch on the name it was started as (`argv[0]`),
  [an exception for the ZED history event's own file name, left out of this excerpt].
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

That third bullet is the one these options meet,
and they meet it in two ways at most:

- B. Git starts ssh,
  ssh starts the wrapper from the path the overlay wrote into `SSH_ASKPASS`,
  and that start prints the explanation.
  The wrapper does not start itself,
  so the first two bullets are kept by their letter;
  but the process that explains was started by Git's descendant,
  which the third bullet forbids.
- E. Git starts the wrapper as its ssh,
  the wrapper starts the real ssh as its child,
  and the wrapper explains from the child's words.
  The explaining process was itself started by Git, which the third bullet forbids on the same reading,
  even though it is the parent of the ssh it explains about.

Two precedents in the same record bear on the reading and neither settles it:

- `remember_own_executable` (`src/native/push/service_route.rs`)
  writes the wrapper's own path into the unit that starts the service mode.
  The unit is started by systemd and explains nothing,
  so that case covers "names its own path" and not "explains from a Git descendant".
- the one-program record allows a later askpass program to reopen its answer,
  which is not this question.

Whether an askpass start, or a wrapper started as ssh, is "asked to explain" was the owner's call,
and the owner answered it on 2026-11-24,
recorded in `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md`:
"Never (Recommended)" for B and "Never (Recommended)" for E.
That is the strict reading of the third bullet,
so no option here starts the wrapper from Git or from ssh.

The other recorded rules:

- "One program only, ever"
  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`):
  no second shipped executable,
  so a separate askpass program of the wrapper's own is out.
  The record adds:
  "A later wish for another shipped program, such as an askpass helper, meets this answer first."
- the wrapper reads the name it was started under for the ZED hook and nothing else
  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`),
  and `SPEC.md` says "the executable does not read the name it was started under",
  so an askpass mode cannot be selected by a link name.
  The ZED record's reason (ZED cannot add an environment variable) does not carry over:
  the overlay can add one, and the record gives no other reason against it.
- no option may answer "yes" on a person's behalf,
  so `StrictHostKeyChecking=accept-new`, a pre-added key and an askpass program that prints `yes` are out.
- the wrapper never starts itself as Git
  (`SPEC.md`, section "The wrapper never starts itself as Git"),
  and its source guards refuse `current_exe`,
  `/proc/self/exe`,
  `argv[0]` dispatch,
  shell text and a start from its own path
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`,
  rules `own-executable`, `argv0-dispatch`, `shell-text`, `spawn-own-executable`).
- E cannot take over a person's `GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH` without either losing that
  person's options or writing shell text, which the `shell-text` rule forbids.
  `ssh_config(5)` reads command-line options first,
  then the user's file,
  then the system file,
  and "for each configuration directive, the first specified value will be used",
  so a `core.sshCommand` rewritten to force a setting would override the person.

## What OpenSSH does with an askpass program

Read from `V_10_6_P2`,
the release this machine runs (`ssh -V` prints `OpenSSH_10.6p2`).

`ssh_askpass` runs the program,
in `readpass.c` lines 46 to 107:

- the program's standard output is a pipe the program cannot see (`dup2(p[1], STDOUT_FILENO)`, `readpass.c:73`);
- one environment variable is added when the question is a permission question,
  `SSH_ASKPASS_PROMPT=confirm` (`readpass.c:75`, `:76`);
  a security-key notification gets `SSH_ASKPASS_PROMPT=none` (`readpass.c:279`);
- the program is found on `PATH` and receives the question as its only argument,
  and its `argv[0]` is that same string: `execlp(askpass, askpass, msg, (char *)NULL)` (`readpass.c:77`);
- a program that cannot be executed never starts,
  and OpenSSH prints `fatal_f("exec(%s): %s", askpass, strerror(errno))` (`readpass.c:78`);
  in the client that path ends the connection's `ProxyCommand`, which is H1;
- everything from the first carriage return or line feed is dropped,
  so only the first line of standard output can be the answer (`readpass.c:104`);
- standard error is not redirected, so it reaches the person as the program writes it;
- standard input is inherited from `ssh`.
  Under Git that stream is the protocol pipe:
  a program that reads it waits for a byte that cannot arrive until the connection stops.
  Measured in the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`:
  an askpass that reads one byte hung the clone until a 20 s bound ended it (`keep/run4.log`, exit 124).

`read_passphrase` decides whether the program runs at all, in `readpass.c` lines 119 to 194:

- `SSH_ASKPASS_REQUIRE=force` sets both `use_askpass` and `allow_askpass` (`readpass.c:130` to `:133`);
- a forced question with no answer from the program returns the empty string (`readpass.c:179` to `:182`).

Two answer paths read an empty answer differently,
and the difference decides which exit statuses an askpass program must never give:

- `ask_permission` (`readpass.c:197` to `:220`) accepts the empty string and the word `yes` as affirmative.
  A program that prints nothing and exits zero therefore agrees to the multiplexing question, the key-confirmation
  question and the forwarding questions.
- `confirm` (`sshconnect.c` lines 601 to 627) reads a `NULL` answer as no, and the empty string as no.
  So the host-key question is safe either way;
  the hazard belongs to `ask_permission` alone.

A security-key touch notice is written straight to standard error only when that stream is a terminal
(`notify_start`, `readpass.c:252` and `:253`);
otherwise it is sent to the askpass program with `SSH_ASKPASS_PROMPT=none` (`readpass.c:279`).
An option that takes standard error away from OpenSSH therefore loses the touch notice.
Source only: no security key was available to probe this.

A changed host key never asks:
with strict checking `ask` the code prints the warning banner and
`Host key for <host> has changed and you have requested strict checking.`,
then `Host key verification failed.`, and fails (`sshconnect.c:1338` to `:1366`).
Measured this round (run, `keep/run-1-behaviour.log`, family `f-changed`).

`BatchMode=yes` returns zero from `confirm` before anything is asked
(`sshconnect.c:609`, `:610`),
so the only line is `Host key verification failed.`
(run, family `f-batchmode`).

`StrictHostKeyChecking=yes` fails with
`No ED25519 host key is known for [localhost]:2222 and you have requested strict checking.`
(`sshconnect.c:1198` to `:1206`) and asks nothing
(run, family `f-strict-yes`).

Quiet mode suppresses OpenSSH's own lines:
`-q` sets `options.log_level = SYSLOG_LEVEL_QUIET` (`ssh.c:927` to `:929`),
`sshlogv` returns before anything is written when the message's level is higher
(`log.c:461`),
and `SYSLOG_LEVEL_QUIET` is the first enumerator (`log.h:41`), so even `fatal` is suppressed.
Measured with `-q` and with `LogLevel=QUIET` in the round whose folder is
`~/temp/agent/host-key-next-step-after-askpass-false-2026-11-25/`:
`Host key verification failed.` disappears, Git's `fatal:` line stays, and an askpass program's own write to standard
error stays.

OpenSSH's diagnostics are not translated:
its tree at `V_10_6_P2` has no message catalogue and no `gettext` call,
so the words these detectors read are C literals,
which is why matching them is safe where matching Git's translated text was rejected
(`doc/planning/cli-git-no-self-invocation.md`, "The text is translated").
Git's own messages are translated, so the detector reads Git's `fatal:` line only through words Git prints
untranslated.

## What Git does around it

Read from Git 2.56.1, the version the wrapper shadows here.

- A clone's progress is drawn only when standard error is a terminal:
  `transport.c:1230`, `ret->progress = isatty(2);`
  and `progress.c:146`, `if (is_foreground_fd(fileno(stderr)) || done)`.
- `is_foreground_fd` is `tcgetpgrp(fd)` returning a group that is this process's,
  or a negative value, which is what a pseudo-terminal with no foreground group answers
  (`progress.c:106` to `:110`).
- The width a progress line is cut to comes from **standard output**, not standard error:
  `term_columns()` reads `$COLUMNS`, then `ioctl(1, TIOCGWINSZ)`, then falls back to 80 (`pager.c:213` to `:241`).
- Git's lines from the remote end are prefixed `remote: `,
  and with `TERM` unset or `dumb` it appends eight spaces to each;
  with a terminal and a real `TERM` it prefixes them with the escape `\033[K`
  (`sideband.c:311` to `:316`).
- The editor's waiting hint is gated on `isatty(2)` (`editor.c:70`).
- Git recognises a command as OpenSSH only by the base name `ssh` or `ssh.exe`
  (`determine_ssh_variant`, `connect.c:1225` to `:1273`);
  for any other name it runs the command once more with `-G <host>` and reads that run's exit status alone.
- Git waits for the `ssh` child and, when it got no answer at all, prints
  `fatal: Could not read from remote repository.` with the two advice lines
  (`connect.c:61` to `:75`, `finish_connect`, `connect.c:1559` to `:1567`);
  that child's standard error is Git's own, which is why OpenSSH's line appears before Git's.
- The exit status a person's shell sees, measured:
  a new host key 128, a key nobody could unlock 128, a connection refused 128, a name that does not resolve 128,
  a repository that is not there with the key known 128, a changed key 128,
  a push whose server hook refuses 1, and a `git pull` in a repository that exists 1.
  That list is why C2's exit-status condition cannot tell the refusals apart.

## The options shown and not kept

This revision ranks five options and hands one of them to the owner,
so no option here is built until the owner answers.
Each entry says what it starts,
what it reads,
whether it can answer yes,
what a person sees for each family of "The case every option is tried against",
what it costs on a command that meets no question,
and its pros and cons.

B, E and J carry answers already given,
and each says which answer and what became of it.

The costs are given twice.
On a typed command,
against a control run in the same rounds,
this harness resolves none of them (its floor is `+0.11 ms` with a range of `-6.18` to `+7.93`,
run, `keep/run-4-typed-path.log`).
As the steps each option adds, against a same-command twin in the same rounds,
it resolves all of them but the pseudo-terminal (`keep/derived-medians.txt`, from `keep/run-3-steps.log`).
Where a figure is a median of per-round differences, its range and the count of positive rounds are given with it,
and the twin it is quoted against is named.

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

At H1's case A is what `main` does, and it is the only arm that ends 0 and prints nothing, because nothing is added to
the connection: exit 0 directly and behind the `ProxyCommand` (run, `keep/run-1-behaviour.log`, families `f-noask` and
`f-ask-known`).
That is also why the owner's answer of 2026-11-25 is safe: it restores exactly this.

A person sees the transcripts under "What the person sees today": OpenSSH's line, Git's fatal line, exit 128,
and no next step.
The one way out is knowledge the words do not give: connect once with plain `ssh <host>`,
which the wrapper does not touch,
or add the key to `known_hosts`.

Cost: nothing, and no control can measure nothing.

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
The mechanism is kept because it is the only option that would name the host, the address, the port and the key at the
question.

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
`~/temp/agent/typed-network-command-next-step-after-askpass-false-2026-11-25/`
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
`~/temp/agent/host-key-next-step-after-askpass-false-2026-11-25/` was `-0.50 ms`, inside its own floor).
The cost on a refused question is one askpass start.

Pros: the only option whose words name the host, the address, the port and the key,
and the only one that can tell a new key from a key that differs for the address;
it needs nothing on the path of a command that meets no question;
it changes nothing about Git's process, streams, signals or exit status;
it survives `-q` and `LogLevel=QUIET`;
it keeps the security-key touch notice at a terminal;
and it meets the service's own pushes with the same words.

Cons: on this machine OpenSSH may not run it,
because `git` confines `ssh` in a child profile that allows executing only `ksshaskpass` and VS Code's script,
and today's `false` works only because a local rule names it;
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
(`reads_as_unknown_host_key` and `reads_as_a_login_failure`, `src/native/lfs/lfs_pre_push.rs`),
and only when the child ended non-zero.
The detector leaves a changed key to OpenSSH's own words,
reads the strict-checking sentence as an unknown key's own,
and reads the last line alone as an unknown key,
so C speaks for a new key, a passphrase or refused key, `BatchMode=yes`, `StrictHostKeyChecking=yes`
(with the strict sentence) and the rare differing-key question.

What a person sees, at a new host key and at a locked key
(run, `keep/run-1-behaviour.log`, arms `f-newkey-ask-direct-c-relay` and `f-locked-direct-c-relay`;
the sentences are the ones under "The words"):

```text
# git clone ssh://target/work/served.git
Cloning into 'probe'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh target`, which the wrapper does not touch: it asks about a key this machine has not seen, and it names the key that changed, which is then the line to remove before running this command again.
```

```text
# git clone ssh://target/work/served.git        (host key known, only a key nobody can unlock offered)
Cloning into 'probe'...
root@localhost: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: the key this connection offered was refused or could not be unlocked, and no command the wrapper runs asks a person for a passphrase, so a key no agent holds cannot be used and a key the server does not accept answers the same way. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add` (`ssh-add -l` lists what the agent holds), or use a key without a passphrase, then run this command again; a key the server still refuses needs the server's account to accept it.
```

At H1's case C ends 0 and prints nothing, directly and behind both proxies, because no refusal reached it
(run, family `f-noask`).
At a changed key it is silent, which leaves OpenSSH's banner and names nothing itself
(run, family `f-changed`).
At a repository that is not there it is silent (run, family `f-known-missing`).
Under `-q` it is blind, because OpenSSH prints nothing to read.

What it starts: Git as a child, one pipe, one relay.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

Cost: `+0.68 ms` over the control, range `-1.40` to `+2.71`, positive 18 of 24
(run, `keep/run-3-steps.log`, arm `relay-start` against `env-twin`;
the twin of the same arm, `relay-twin` against `relay-start`, is median `-0.03 ms`, range `-2.16` to `+2.32`).
On the typed path it is `+1.62 ms`, range `-6.57` to `+5.83`, positive 17 of 24, which is inside that harness's
floor (run, `keep/run-4-typed-path.log`).

Pros: gives the words at both refusals at the place the existing push sentences do,
chooses them from what OpenSSH printed,
leaves every stream in a captured command byte for byte as plain Git leaves it (measured),
keeps every setting the person and the repository own,
and is the shape the wrapper already uses where it owns a child
(`run_real_git`, `src/native/invocation/forwarding.rs`).

Cons:

- the live progress display disappears: Git draws it only when standard error is a terminal,
  and under C that stream is a pipe.
  Measured: plain Git and K each show a `Receiving objects` and a `remote: Counting objects` line at a terminal,
  and C shows neither (`keep/run-2-bytes-and-progress.log`, the `TERMINAL` lines).
  Passing `--progress` would bring them back, but it would mean adding a word to the caller's arguments.
- the security-key touch notice is lost, because OpenSSH writes it to standard error only when that stream is a
  terminal, and otherwise sends it to the askpass program, which is `false` here (`readpass.c:252`, `:279`).
  Source only; no key was available.
- a signal sent to the wrapper alone no longer reaches Git:
  the relay is a child of the wrapper and Git is the relay's child.
  Measured for a later signal in the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/` (the child is still running after
  `kill -TERM` of the wrapper); a terminal's control character signals the whole foreground group, so the exec path
  gives this for free today.
- it is blind under `-q`, measured in the same earlier round.
- the wrapper must now own the exit status, the signal ending, the wait and the ordering of its own notices against
  Git's output.

### K. The wrapper keeps the child, with standard error on a pseudo-terminal

Mechanism: C's shape, with one difference.
The wrapper starts Git as its child and gives it a pseudo-terminal on standard error instead of a pipe,
relays whatever that terminal produces to the wrapper's own standard error,
and after the child ends prints the sentence the transcript calls for, using the same detectors C uses and the same
non-zero gate.
Standard input and standard output stay the child's own, no argument is added, and no program Git did not start is
asked anything, so the owner's rules and the two "Never" answers hold.

Why the pseudo-terminal at all:
Git, OpenSSH and `git-lfs` all ask `isatty(2)` before drawing anything a person watches
(`transport.c:1230`, `progress.c:146`, `readpass.c:252`).
A pipe answers no to that question, and a pseudo-terminal answers yes.

What a person sees at a new host key is C's transcript with `target` in the sentence, and at a locked key the
passphrase sentence (run, `keep/run-1-behaviour.log`, arms `f-newkey-ask-direct-k-pty` and `f-locked-direct-k-pty`).
At H1's case it prints nothing and the clone ends 0, directly and behind both proxies.

What it costs the bytes a person does not see is measured under "The measurements":
the progress display lands in any captured stream,
every line gains a carriage return unless the terminal's output processing is cleared,
Git's terminal-only escapes land there as well,
and every descendant that asks whether its standard error is a terminal is told yes.

What it starts: Git as a child, one pseudo-terminal pair, one relay.
What it reads: the terminal's text as it arrives.
Can it answer yes: no; it only reads.

Cost: `+0.85 ms` over the control, range `-2.73` to `+2.61`, positive 20 of 24
(run, `keep/run-3-steps.log`, arm `pty-start` against `env-twin`;
its twin, `pty-twin` against `pty-start`, is median `-0.06 ms`).
Against C, in the same rounds, it is `+0.16 ms` with a range of `-1.68` to `+1.80` and 13 of 24 positive,
which this harness cannot tell from no difference.
On the typed path it is `+1.05 ms`, positive 17 of 24, inside that harness's floor.

Pros: everything C gives, and it keeps the progress display and the security-key touch notice at a terminal
(measured for the display; source for the notice);
a person who captures output can have plain Git's bytes back, because a shipped K that clears the terminal's output
processing writes exactly plain Git's bytes with `--no-progress` (measured, `BYTES ... identical=yes`).
Cons: the four effects listed under "The measurements" on every captured stream, of which one is removable by the
wrapper and the rest are the price of the terminal;
the wrapper owns the pty's life and the child's wait, as C owns the pipe's;
and a `-q` transcript still yields nothing.

### K2. K when the wrapper's own standard error is a terminal, C's pipe otherwise

Mechanism: one `isatty` call on the wrapper's own standard error decides which of K's two shapes runs.
When that stream is not a terminal, plain Git would draw nothing, and the pipe presents exactly that,
so the child behaves as it does today and the relay copies its bytes unchanged.
When it is a terminal, the pseudo-terminal presents what Git sees today.

What a person sees: at a terminal, K's transcript (progress and the touch notice's terminal, run,
`keep/run-2-bytes-and-progress.log`, `TERMINAL k2-adaptive receiving=1 counting=1`);
in a captured stream, C's transcript, which is plain Git's, byte for byte
(run, same log, `BYTES redirect-k2-adaptive redirect-plain identical=yes`, 37 bytes, no carriage returns, no escapes).

What it starts and reads: what K starts and reads, plus one `isatty` call.
Can it answer yes: no.

Cost: `+0.74 ms` over the control, range `-1.38` to `+2.52`, positive 20 of 24
(run, `keep/run-3-steps.log`, arm `adaptive-start` against `env-twin`);
against C in the same rounds, `+0.12 ms`, range `-1.40` to `+1.38`, 14 of 24 positive, which is no difference this
harness resolves.

Pros: every population gets plain Git's bytes or plain Git's terminal on every command;
the words are C's, chosen from the transcript, and the strict sentence has its own words;
one `isatty` call is all it adds to C's cost;
it has none of K's redirected-stream effects, and none of C's terminal loss.
Cons: it is two shapes to build, test and explain, and both of them must be controlled;
the decision is made once per process on the wrapper's own stream, and every descendant then sees that answer, so a
program that would have used a pipe's answer gets a terminal's whenever the person's stream is a terminal, which is
K's trade applied only where a terminal is there;
it keeps C's other cons, the wait, the signal and the `-q` blindness.

### C2. Git kept as a child with the same streams, and a sentence after exit 128

Mechanism: the wrapper no longer replaces the process,
but it does not take standard error either.
It starts Git as a child with the same standard input, standard output, standard error and terminal, waits,
and after exit 128 prints a sentence.
Nothing is piped, no argument is added, no text is read.

Three shapes, and they are three different options, because a check changes which refusal gets words:

- the exit status alone;
- the check speaks the host-key sentence only when no file `ssh -G` names holds a key;
- the check speaks the passphrase sentence only when one does;
- and the shape both of those make together, choosing on what the check found.

The shape the ranking means is the second, the one that speaks only when no file holds a key:
it is the only one of the four that never prints a sentence the case contradicts.
The others are worse, and measured:

- with the exit status alone, a repository that is not there and a refused connection get a paragraph about keys
  (run, `keep/run-1-behaviour.log`, `f-known-missing-direct-c2-status`);
- with the check speaking only when a key is known, a repository that is not there gets the passphrase sentence, and a
  new host key gets nothing
  (run, `f-known-missing-direct-c2-check-known` and `f-newkey-ask-direct-c2-check-known`);
- with both branches, a repository that is not there gets the passphrase sentence, a changed key gets the passphrase
  sentence, and a locked key gets it too
  (run, `f-known-missing-direct-c2-check-both`, `f-changed-direct-c2-check-both`,
  `f-locked-direct-c2-check-both`).

What the ranked shape gives, measured:

- at a new host key, the host-key sentence after Git's lines: right;
- at a locked key, nothing: the one refusal the person needs words for gets none;
- at a repository that is not there, nothing: right;
- at a changed key, nothing: right, and OpenSSH's banner says what changed;
- under `-q`, the same as without it, because it reads a status and not a line.

What it starts: Git as a child; on a failure, `ssh -G` and `ssh-keygen -F` as well.
What it reads: the exit status, and with the check the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost: `+0.68 ms` for the fork and the wait (run, `keep/run-3-steps.log`, `standin-start` against `env-twin`,
positive 23 of 24),
and on the failure path the check's two starts, measured at `+7.50 ms` for D and the same work here.
On the typed path, `+1.26 ms`, positive 15 of 24, inside the floor.

Pros: keeps the terminal, the progress display, the signals from the terminal and every setting the person owns;
adds no word to the caller's arguments; touches no stream, so a captured command's bytes are plain Git's;
and it survives `-q`.
Cons: it cannot tell the refusals apart, so its ranked shape is silent at a key nobody can unlock;
the wrapper still owns the wait, the exit status and the signal ending;
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
succeeds; measured this round, both ways
(run, `keep/run-1-behaviour.log`, `f-accept-new-direct-d-precheck-acceptnew` prints, `f-accept-new-direct-d-precheck`
is silent, and both clones end 0).

What a person sees at a new host key
(run, `keep/run-1-behaviour.log`, arm `f-newkey-ask-direct-d-precheck`): the sentence, then `Cloning into`,
then OpenSSH's refusal and Git's lines.
For the passphrase case it prints nothing, because the key is known (run, family `f-locked`).
That is D's gap, measured, and it is the case whose words the person most needs.

The sentence can also be wrong about why a command failed:
measured in the round whose folder is
`~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`,
with nothing listening on the port, D prints the host-key sentence and the real words are
`ssh: connect to host 127.0.0.1 port 2299: Connection refused`.
Two false-positive classes are also measured:

- `KnownHostsCommand` supplies the key: the real clone exits 0 and D speaks
  (run, `~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/keep/run-1.log`);
- `@cert-authority` supplies the key by certificate: `ssh-keygen -F` finds the line and D stays silent, which is
  right (the round whose folder is `~/temp/agent/host-key-next-step-after-askpass-false-2026-11-25/`, its
  `keep/run-2.log`).
- `VerifyHostKeyDNS=yes` would be a third, and is unrun: no DNSSEC setup was available.

Two limits stand:

- a changed host key looks known to it, because `ssh-keygen -F` finds a key under the name,
  so it says nothing about the refusal whose advice needs the offending line;
- it is blind to a `HostName` alias whose key lives on a jump host, and to any known-hosts file a `ProxyCommand` or
  `ProxyJump` makes the far side read.

What it starts: `ssh -G` and `ssh-keygen -F` before the command.
What it reads: the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost: `+7.50 ms` over the control, range `+5.23` to `+10.55`, positive 24 of 24,
for the whole check with both starts (run, `keep/run-3-steps.log`, arm `precheck-ask-known` against `env-twin`);
its twin, `precheck-twin` against `precheck-ask-known`, is median `-0.15 ms`.
The two starts separately are `ssh -G` at `+3.33 ms` (24 of 24) and `ssh-keygen -F` at `+2.38 ms` (24 of 24),
each against one process start.
On the typed path it is `+5.76 ms`, range `-3.62` to `+14.01`, positive 23 of 24: the only option that harness
resolves.

Two costs the first round did not name, both read from the source:

- `ssh -G` runs the person's `Match exec` commands, measured in the round whose folder is
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`: one plain `git ls-remote` ran it once,
  and D's check plus the ssh it then runs ran it twice.
- `ssh -G` can resolve names: `CanonicalizeHostname` makes it do DNS work, which was not run here
  (`ssh_config(5)`, "CanonicalizeHostname").
  Its output can also carry `ProxyCommand` and `SetEnv` values, which must never be logged.

Pros: no change to how Git runs, no new mode, no policy change, and the words arrive before the connection.
Cons: one of the two refusals is out of reach,
its words can be wrong about a command that then fails for another reason and wrong on commands that succeed,
the cost is paid by every network command,
and the check runs the person's own configuration commands one extra time.

### E. The wrapper becomes the ssh that Git starts

Answered "Never (Recommended)" by the owner on 2026-11-24,
so this is recorded for its evidence and is not ranked.
The owner's reason is the rule this section's first con breaks:
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
it is blind under `-q`;
a signal sent to the wrapper alone does not reach ssh;
the security-key notice is lost;
its words come from the same detectors C uses, so they miss a changed key;
and the process that explains was started by Git, which is the owner's question.

### J. The sentence itself as `SSH_ASKPASS`'s value

Shown, chosen on 2026-11-24, and retired by the owner on 2026-11-25:
`SSH_ASKPASS` holds `false` again.
The reason is H1, and it is not a defect of the wording:
a value OpenSSH cannot start takes the connection down when the connection has a proxy,
which is the mechanism quoted under "The question, open again".

The value a shipped J would use cannot name the host, so it is one generic text.
That text is 369 bytes,
and the built sentence of the round whose folder is
`~/temp/agent/host-key-next-step-after-askpass-false-2026-11-25/` measured 571 bytes;
the two figures are different texts, and the earlier round's arms ran the 571-byte one
(its `keep/run-2.log`, arms `b-askpass-*`).
The 369-byte value is the one whose failure line this document's own earlier revision quoted
(that round's `keep/run-3.log`, `File name too long`).

Measured limits, each one a finding against it:

- one text for every question, so it cannot name the host or the key;
- OpenSSH's framing: the person reads the advice inside `ssh_askpass: exec(...): File name too long`;
- it prints on a command that then succeeds, when a passphrase key is skipped for one that works;
- it disappears under quiet, because the whole line is OpenSSH's and `-q` suppresses it;
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

The words of the fourth revision are kept under "The earlier revisions, kept for their record" as what was argued
over;
the sentences a shipped C, K or K2 prints are the three here.

### A new host key, which may also be a key that changed

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, and no command the wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain `ssh target`, which the wrapper does not touch: it asks about a key this machine has not seen, and it names the key that changed, which is then the line to remove before running this command again.
```

Three claims, each one true of what the transcript can show:

- `does not know ... or holds an outdated one` covers the three cases a transcript can be: an unknown key, a changed
  key, and the rare differing key for an address.
  It is the hedge the wrapper already ships for a key failure ("refused or could not be unlocked"),
  applied to the other refusal.
- The advice is one command,
  and it works in each case: plain `ssh` asks about a key it has not seen, and names the key that changed.
- The sentence no longer says to add the key to `known_hosts`,
  because that remedy is wrong for a changed key and because plain `ssh` is the path the wrapper does not touch.

### The same refusal when the person's own setting refuses to ask

```text
monochromatic-git-policy-cli: this machine has no SSH host key recorded for `target`, and your settings refuse to ask about a new one (`StrictHostKeyChecking=yes`), so the connection stopped before any login. Connect once with plain `ssh -o StrictHostKeyChecking=ask target`, which the wrapper does not touch and which asks once, then run this command again; or add that host's key to `known_hosts` yourself.
```

This sentence is chosen from the transcript:
OpenSSH prints `No ED25519 host key is known for [localhost]:2222 and you have requested strict checking.`
and the detector accepts that line as an unknown key's own,
so the reader knows both the refusal and the setting that caused it.
Without its own words the general sentence would tell that person to run `ssh target`,
which reads the same configuration and refuses the same way.

### A key that was refused or could not be unlocked

```text
monochromatic-git-policy-cli: the key this connection offered was refused or could not be unlocked, and no command the wrapper runs asks a person for a passphrase, so a key no agent holds cannot be used and a key the server does not accept answers the same way. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add` (`ssh-add -l` lists what the agent holds), or use a key without a passphrase, then run this command again; a key the server still refuses needs the server's account to accept it.
```

What changed from the fourth revision, and why:

- the key's path is gone:
  `Permission denied (publickey).` carries no path,
  and the path was the fixture's, so no shipped sentence can name it from this transcript;
- the cause is hedged,
  because `Permission denied (publickey).` arrives both for a key nobody could unlock and for a key the server
  refuses, which the project already records (`reads_as_a_key_failure`,
  `src/native/lfs/lfs_pre_push.rs`);
- `ssh-add -l` is offered because it rules the second cause out,
  exactly as `login_needed_message` offers it.

### Why there is no fourth sentence for a differing key

The differing-key question is rare (`CheckHostIP` defaults to off),
and its text, which carries the offending file and line, reaches only the askpass program,
so only B could print it.
Under C, K or K2 the transcript is the last line alone and the general sentence is printed:
its way to connect once is right for that case, and it makes no claim about adding a key,
which was the fourth revision's wrong advice.

### Where the host comes from, and what reading it costs

The host is the one the person's remote names, not the address it resolves to,
because a `known_hosts` line is recorded under the name the connection asked about:
the fixture's `target` carries `HostName localhost`, so the clone records `[localhost]:2222`,
and advice that named `127.0.0.1` would ask the person to accept a second key for the same machine.

- A `clone` carries the URL in its own arguments, so the host costs nothing to read.
- A `fetch` or `pull` that names a remote, or none, needs the remote's URL from the repository:
  `git remote get-url <remote>`, one start, measured at `+0.85 ms` against one process start,
  range `-0.59` to `+1.50`, positive 23 of 24 (run, `keep/run-3-steps.log`, arm `remote-url`).
- The connect form is built from that URL's own grammar:
  `ssh://[<user>@]<host>[:<port>]/<path>` and the scp-like `[<user>@]<host>:<path>`.
  A port that is not 22 is written `-p <port>`, a user the URL names is written before the host,
  and a standard port and an unnamed user are left out.
- A host, user or port that is not a plain host name, address or number is named in backticks and not put into a
  command, so nothing the sentence offers can be read by a shell as anything but a word.

### Three rules about printing text that came from outside

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
  `OPOST` is cleared, which is C's transcript byte for byte (run, `keep/run-2-bytes-and-progress.log`,
  `BYTES refusal-k-pty-raw refusal-c-relay identical=yes`).
- No `setsid` and no `TIOCSCTTY`.
  The person's terminal must keep signalling Git,
  and Git's progress is drawn only when `is_foreground_fd(fileno(stderr))` holds or the descriptor has no foreground
  group at all (`progress.c:106` to `:110`, `:146`).
  A new session that owns the pty would put Git's group outside the foreground one.
  The stand-in this round ran has neither call; no arm sent a signal to the wrapper alone, so that part is design.
- A size is set on the terminal, and it is not what decides Git's progress width.
  Git's width comes from `term_columns()` reading **standard output** (`pager.c:213` to `:241`),
  which K leaves alone, so copying the wrapper's window size onto the pty changed nothing in any transcript:
  40 and 120 columns both cut this clone's progress at the same place
  (run, `keep/run-2-bytes-and-progress.log`, the `WIDTH` lines).
  The fourth revision's claim that the arms used eighty by twenty-four was unsupported;
  the arms' pty had no size, and Git read its `stdout`, which was a file, and fell back to 80.
  A shipped option sets a size for the programs that ask their own standard error's terminal, and nothing more.
- The relay ends when the child does, after draining what is already readable.
  A hook that leaves a process holding the child's standard error otherwise holds the wrapper:
  measured, a `post-checkout` hook that starts `sleep 5` makes the clone take 5161 ms where the child ends in 200 ms
  (run, `keep/run-2-bytes-and-progress.log`, arms `sleeper-c-relay` and `sleeper-fixed-c-relay`).
  With the rule, the same arm ends in 207 ms.
- A signal and a window change are not forwarded by the relay.
  The wrapper must not take the child's signals from the person's terminal,
  and `SIGWINCH` after the start does not reach the pty, as it does not reach a pipe.
  The interactive LFS upload already relays a signal sent to the wrapper and holds no deadline of its own
  (`doc/planning/cli-git-no-self-invocation.md`, the note beginning "Not changed: the interactive upload has no
  deadline of its own"), which is the precedent for the shape.
  A shipped option says so wherever it documents what it changes.
- The tail the detectors read is bounded.
  A clone prints megabytes of progress; the tail holds what the words need.
  This round's stand-in keeps the last 65536 bytes, and the wrapper's own LFS explanation already keeps a bounded
  tail.
- The lines Git relays from the remote are dropped before the detectors read.
  Git prefixes every line the other end writes with `remote: `,
  and a server or a `ProxyCommand` program can write a refusal's words there.
  The shipped detector does not drop them today, so a transcript holding
  `remote: Host key verification failed.` reads as an unknown key's;
  measured with the rule off, a push whose server writes that line and then refuses gets the host-key sentence,
  and with the rule on it gets nothing
  (run, `keep/run-1-behaviour.log`, `f-remote-fail-c-relay-keep-remote` and `f-remote-fail-c-relay`).
  The same hole is in the explanation the LFS upload prints, because it reads the same detector;
  fixing the detector fixes both.
- Nothing is printed unless the child ended non-zero.
  A transcript can hold the refusal's words on a command that succeeded:
  measured, a push whose server writes the line and accepts gets no sentence, with or without the drop rule
  (`f-remote-ok-c-relay`, `f-remote-ok-c-relay-keep-remote`).
- The child's standard input and standard output are its own, and the exit status and the signal ending are
  Git's: `end_as_git_ended` is the existing shape for that.
- The sentence is printed after the child's own output, on the wrapper's standard error.
- Under `-q` there is nothing to read, and the option is silent; that is not fixable by reading harder.
- A typed LFS push would say it twice, because the upload's explanation reads the same transcript;
  the sentence must be the shorter of the two for that path, or the upload must learn to stay silent when it already
  spoke.
- Which commands: see the next section, because the wrapper has no single entry point for a typed network command.

## What building each option would take

### B, answered "Never"

- `no_prompt_overlay` names the wrapper's own path in `SSH_ASKPASS` and keeps `SSH_ASKPASS_REQUIRE=force`
  (`src/native/invocation/child_environment.rs`).
  `GIT_ASKPASS` stays empty.
- a mode at the top of `main`, after `install_payload_free_panic_hook` and before any planning,
  selected by the two signals under "B",
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
  There is no single entry point, and these are the paths, read from the source:

  - `Action::Forward` runs `replace_process_with_real_git` (`invocation/entry.rs`, the arm at `:388` to `:408`);
  - `Action::ForwardCoordinated` runs `run_coordinated_forward`, which either starts Git as a child through
    `run_forwarded_git` or falls back to `replace_process_with_real_git`
    (`invocation/entry_run_forwarded_actions.rs`, the coordinated arm);
  - `Action::ForwardPush`, `Action::ForwardAnnounced` and `Action::ForwardSettled` each start or replace for reasons
    of their own (`invocation/action.rs`, `push/stacked_merge_push.rs`);
  - a `pull` that merges runs its fetch itself, as `git fetch --update-head-ok`, through `run_wrapper_own_git`
    (`merge/pull_isolation.rs`, "the fetch of a pull that merges runs, then the merge goes through the isolated
    merge"), so a `pull`'s network step never passes the top-level forwarding arm;
  - the LFS upload and the push record already read a child's output and keep their own sentences.

  Which of those a given typed command reaches was not traced end to end;
  what is read here is that a single entry point does not cover them.

  `run_real_git` is not K's shape:
  its documented contract is that the child "shares the caller's terminal and streams directly; nothing is
  buffered or copied" (`invocation/forwarding.rs`),
  which is C2's shape.
  The relay is new code at each of those paths, or a new function they all call.
- The transport question, decided rather than assumed.
  Nothing in `src/native` classifies "a remote that ssh carries": the only URL handling is the LFS upload's
  `insteadOf` reading (`lfs/lfs_upload_settings.rs`).
  So either the relay runs for every typed network command whatever the transport,
  and an `https` or `file` clone gets a pseudo-terminal and no words, or the wrapper reads the remote first,
  which for a URL in the arguments costs nothing and for a named remote costs `+0.85 ms`.
  The ranking assumes the second, because the first changes the bytes of commands that can meet no such refusal.
- The detectors.  `reads_as_unknown_host_key` and `reads_as_a_login_failure` stay where they are,
  and the strict sentence is chosen from the same text.
  The `remote: ` rule and the non-zero gate are the two additions, and both belong to the detector rather than to the
  relay, so the upload's explanation gets them too.
- The words, in one module beside `lfs_pre_push.rs`, filled from the remote as typed.
- The exit status, the signal ending and the wait, which the LFS upload's `SignalGuard` machinery already shows.
- C's difference: a pipe instead of the terminal, and nothing else.
- K2's difference: a pseudo-terminal when the wrapper's own standard error is a terminal, and C's pipe when it is
  not, so both of K2's shapes are built and both are controlled.
- Controls, at least:
  - a typed `clone` at a new host key prints the sentence after Git's lines and ends 128, direct and behind a
    `ProxyCommand` and a `ProxyJump`;
  - a typed `clone` with a locked first key and a working second ends 0 and prints nothing, direct and behind both
    proxies;
  - a typed `clone` with only a locked key prints the passphrase sentence and ends 128;
  - a clone at `StrictHostKeyChecking=yes` prints the strict sentence;
  - a clone with `BatchMode=yes` prints the general sentence;
  - a clone of a repository that is not there with the host key known prints nothing about keys;
  - a changed key at the default level prints nothing from the wrapper, and the same at `LogLevel=FATAL` prints the
    general sentence;
  - a command that ends 0 prints nothing even when its transcript holds the refusal's words;
  - a server that writes `remote: Host key verification failed.` on a failing command prints nothing;
  - with both streams redirected, a captured transcript is plain Git's, byte for byte, under K2 and under C;
  - with `--no-progress` and `OPOST` cleared, K's captured transcript is plain Git's byte for byte;
  - a hook that leaves a process holding the child's stream does not hold the wrapper;
  - a signal aimed at the wrapper alone still reaches Git, and the wrapper ends the way Git ended;
  - the LFS upload and the push record print one host-key sentence, not two.

### C2

C2's ranked shape needs the same child and wait, plus D's two starts on the failure path only,
and its three controls are the three cases the shapes get wrong:
a repository that is not there, a changed key, and a locked key.

### D

D needs the resolution and the check that `keep/stand-ins/standin.c` runs, before the process is replaced,
with the sentence printed before `Cloning into`, and its condition `ask` alone.

## The ranking

One ranking,
because the questions the earlier rankings turned on are answered:
B and E are "Never",
so no program Git started explains anything,
and J is retired because its value cannot be started.
What is left is K2, K, C, C2, D and A.

Everything is ranked by what each delivers to the person who typed the command:
whether the words arrive at the refusal that stopped them,
whether they name what the person must do,
whether they appear when nothing is wrong,
and what the person gives up on every other command.

K2 > K > C > C2 > D > A.

- K2 over K.
  Both give the same words and keep the progress display at a terminal.
  When the person's standard error is not a terminal, plain Git draws nothing, and K2's pipe presents exactly that:
  measured, its captured transcript is plain Git's, byte for byte, at 37 bytes with no carriage returns and no
  escapes, where K's is 9127 bytes with 210 carriage returns and 107 escapes
  (run, `keep/run-2-bytes-and-progress.log`).
  K2 decides with one `isatty` call and costs what C costs, `+0.74 ms` against the control, which is inside the
  difference this harness cannot resolve; K costs `+0.85 ms`.
  K's captured-stream change falls on every agent session, hook, CI step and script that captures a typed command's
  output, and K2's does not fall on anyone.
- K over C.
  At a terminal, the place where a person watches a clone and meets the touch notice, K behaves as plain Git does:
  measured, plain Git and K each draw the progress lines and C draws none, and OpenSSH writes a security key's touch
  notice to standard error only when that stream is a terminal (`readpass.c:252`, `:279`, source only).
  C's loss cannot be undone from the command line; K's captured-stream change can, because `--no-progress` with the
  terminal's output processing cleared writes plain Git's bytes (measured).
  Their costs are the same as far as this harness resolves (`+0.16 ms` for K against C, 13 of 24 positive).
  A reader who weighs captured logs over terminal displays would put C first; this document weighs the person at the
  terminal, because the refusal is met there and the capture is a machine's log.
- C over C2.
  C's words are chosen from what OpenSSH printed, so they fit the refusal: measured, C is silent at a repository that
  is not there and speaks at a key nobody can unlock.
  C2's ranked shape reads a status and a known-hosts check, and it is silent at that same locked key,
  which is the refusal whose words the person most needs (measured).
  C2's other shapes are worse than silence there: the status-only shape prints the host-key sentence at a repository
  that is not there, and the shape with both branches prints the passphrase sentence there and at a changed key
  (all measured).
  What C2 keeps over C is real and belongs in this pair:
  it holds the person's own streams, so its captured bytes are plain Git's and it loses neither the progress display
  nor the touch notice;
  it keeps the terminal's signals; and it survives `-q`, where C reads lines OpenSSH has stopped printing.
  Both pay one fork and one wait (`+0.68 ms` each in the same rounds).
  So the person trades words that fit the refusal for a `-q` transcript and an untouched terminal,
  and this document takes the words.
- C2 over D.
  C2's ranked shape never prints on a command that succeeds, and its check runs only after a failure,
  where D's two starts run before every command and its sentence can precede a command that then fails for another
  reason or succeeds (`KnownHostsCommand`, `accept-new`, a connection refused: all measured, in this round or the one
  whose folder is `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`).
  C2 costs `+0.68 ms` of work on every command; D costs `+7.50 ms` in two starts, the only option a typed-path
  harness resolves at `+5.76 ms`, 23 of 24 positive.
  Both can misteach: C2's ranked shape says nothing where words were needed, and D's first line can be wrong before
  the truth arrives, on every command.
- D over A.
  D speaks for the plain new-key case, measured: the sentence before `Cloning into`, then OpenSSH's refusal.
  A never speaks, and at the first contact with every host the person is left with no next step.
  D pays for that with two extra process starts on every typed network command, and with a sentence that can be
  wrong about why the command failed; A pays nothing and says nothing.
  A ranked last is not a verdict that D is useless: D is the only option whose words arrive before the failure,
  and a person who reads quickly sees the advice before the error.

### What each answer gives the owner

- K2 is the first.
  It needs the sentences under "The words", the two detectors that already exist with the two additions they need,
  one relay wired into the child paths named under "What building each option would take", and one `isatty`
  decision.
  It costs what C costs, about `0.7 ms` of work on every typed network command,
  and it changes no captured byte for anyone.
- K is the same with a terminal always: the same words, the progress display and the touch notice for everyone,
  and a captured stream that holds the progress, the carriage returns and the escapes.
- C is the same with a pipe always: plain Git's captured bytes, and no progress display and no touch notice at a
  terminal.
- C2 reads no text: it survives `-q` and touches no stream, and its ranked shape says nothing at a key nobody can
  unlock.
- D is the only option whose words arrive before the command, and the only one that pays two process starts on
  every command.
- A is what `main` does: nothing, at both refusals.

## The owner's question

> Which delivery should the wrapper build, now that `SSH_ASKPASS` holds `false` again:
> **K2**, a pseudo-terminal on the child's standard error when the wrapper's own standard error is a terminal and a
> pipe otherwise (ranked first);
> **K**, the same pseudo-terminal always;
> **C**, the same with a pipe always;
> **C2**, Git kept as a child with the person's own streams and, after exit 128, a sentence when a known-hosts check
> finds no key for the host (its other shapes are named under "C2");
> **D**, a `known_hosts` check before Git runs;
> or **A**, nothing?

The ranking is K2 > K > C > C2 > D > A, and each adjacent pair's reason is under "The ranking".

What each answer leaves:

- K2: the words arrive at the refusals OpenSSH prints, name the host as the person typed it,
  and nothing changes in what a captured command writes;
  at a terminal the person keeps the progress display and the touch notice.
  A `-q` command still gets nothing, and a typed LFS push needs the double-sentence fix.
- K: the same words, and every captured stream gains Git's progress bytes, a carriage return per line and Git's
  terminal escapes.
- C: the same words, plain Git's bytes in every capture, and no progress display and no touch notice at a
  terminal.
- C2: `-q` still works and nothing is lost on a successful command, and a key nobody can unlock still gets no
  words.
- D: advice before the command fails, one of the two refusals never covered, and two extra process starts on
  every typed network command.
- A: nothing changes, and a person at a new host key or with a key no agent holds still reads OpenSSH's line and
  Git's `fatal:` with no next step.

Until the owner answers, `main` holds `false` and names no next step:
that is the state the askpass round leaves, and `SPEC.md` and `README.md` say so.
The question belongs beside the L4 entry in `doc/handover/questions-for-the-owner.md`;
this round may not edit that file,
so it is here and in this round's report.

## The measurements

### The runs

Four container runs of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-bbd969e09a87`),
each `--rm --init --network=none --cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
each started in a `systemd-run --user` scope with `MemoryMax=5G`, `TasksMax=512` and `IOWeight=10`,
and each ended by `RuntimeMaxSec=1500`:

- `host-key-review-run1`, `CPUQuota=300%`: the behaviour arms, log `keep/run-1-behaviour.log`, script
  `keep/run-1-behaviour.sh`, 135 arms;
- `host-key-review-run2`, `CPUQuota=300%`: the bytes and the progress, log `keep/run-2-bytes-and-progress.log`,
  script `keep/run-2-bytes-and-progress.sh`;
- `host-key-review-run-3-steps`, `CPUQuota=300%`: the steps, log `keep/run-3-steps.log`,
  script `keep/run-3-steps.sh`;
- `host-key-review-run-4-typed-path`, `CPUQuota=300%`: the typed path, log `keep/run-4-typed-path.log`,
  script `keep/run-4-typed-path.sh`.

Three exploration runs came before them and are kept because each proves a mechanism the measurement runs depend on:
`keep/explore.log` (the image's tools, and that a post-checkout hook and a `ProxyCommand` see standard error),
`keep/explore2.log` (a server-side hook whose line Git relays as `remote: ` on a push that succeeds and on one that
fails, the end rule, and `OPOST`), and `keep/explore3.log` (that a terminal of a chosen width reaches the child, and
that Git's progress width does not follow it).

Method, so a difference can be told from this harness's noise:

- every arm ran in every round, and each round started one arm further along the list, so an arm sits at every
  position;
- warm-up rounds ran before the measured ones;
- every arm has a twin, a second arm running the identical command, and the twin's per-round difference is the floor
  that arm is quoted against;
- an arm's figure is the median of its per-round difference from its control in the same round, with the range of
  those differences and the count of positive rounds beside it;
- the typed-path control is `git ls-remote` with the same `ssh`, the same options and `SSH_ASKPASS=false`,
  and the typed-path floor is that same command run twice in the same round;
- the step control is `/usr/bin/true`, and every stand-in arm also pays the `env` exec the control behind its twin
  pays.

### The four effects of a pseudo-terminal on a redirected command

Run, `keep/run-2-bytes-and-progress.log`.
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
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, ...
```

```text
##### cat -A refusal-k-pty-raw
Cloning into '/work/clones/probe'...$
Host key verification failed.^M$
fatal: Could not read from remote repository.$
$
Please make sure you have the correct access rights$
and the repository exists.$
monochromatic-git-policy-cli: this machine does not know the SSH host key of `target`, or holds an outdated one for it, ...
```

Measured counts for the same refusal:
plain Git 194 bytes and 1 carriage return,
C 645 bytes and 1,
K 651 bytes and 7,
K with `OPOST` cleared 645 bytes and 1,
K2 645 bytes and 1.
`refusal-k-pty-raw` and `refusal-k2-adaptive` compare identical to `refusal-c-relay`, and `refusal-k-pty` does not.

A clone of two thousand objects with both streams redirected to a file, so no terminal on either side:

- plain Git: 37 bytes, no progress line, no carriage return, no escape byte;
- C: the same 37 bytes, identical to plain Git;
- K2: the same 37 bytes, identical to plain Git;
- K: 9127 bytes, a `Receiving objects` line, a `remote: Counting objects` line, a `remote: Enumerating objects` line,
  210 carriage returns and 107 escape bytes;
- K with `OPOST` cleared: 9122 bytes, the same three progress lines, 204 carriage returns and 107 escape bytes;
- K with `OPOST` cleared and `--no-progress`: 37 bytes, identical to plain Git with `--no-progress`.

That is the four effects, one by one, with the source that gates each:

- the progress display in a captured stream: `transport.c:1230`, `ret->progress = isatty(2)`,
  and `progress.c:146`;
- a carriage return on every line, and a second one on OpenSSH's own line, because the terminal's output
  processing turns each line feed into a carriage return and a line feed and OpenSSH writes its own `\r\n`;
  removed by clearing `OPOST`, which leaves only the carriage returns Git writes for its own progress;
- Git's other terminal-only output: the escape `\033[K` that prefixes every `remote: ` line when standard error is
  a terminal and `TERM` is not dumb (`sideband.c:311` to `:316`), measured as 107 escape bytes in the captured file
  against none under plain Git or C.
  The editor's waiting hint is gated the same way (`editor.c:70`) and no clone draws it, so that one is source only.
- every descendant's own answer to `isatty(2)`: a `post-checkout` hook prints `hook-stderr-tty: YES` under K and
  `no` under plain Git, C and K2, and a `ProxyCommand` program records `proxy-stderr-tty: YES` under K and `no` under
  the others.

At a terminal (`ptyrun 80 24`):

- plain Git draws the progress lines and 210 carriage returns;
- C draws neither the progress lines nor a second carriage return, and ends with 1 carriage return;
- K and K2 draw the progress lines and 216 carriage returns, the 6 extra being the doubled line ends.

### The end rule

Run, `keep/run-2-bytes-and-progress.log`.
A `post-checkout` hook starts `(sleep 5) &` with standard error inherited, so a process holds the child's stream after
the hook exits and after Git ends.
A relay that reads until the stream closes takes 5161 ms (C) and 5173 ms (K) for a clone whose child ends in about
200 ms; a relay that ends when the child has exited takes 207 ms (C) and 205 ms (K).

### The steps

Run, `keep/run-3-steps.log`, 24 rotated rounds, and `keep/derived-medians.txt` for every figure here.
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
- K2's adaptive choice: `+0.74 ms`, range `-1.38` to `+2.52`, 20 of 24 (`adaptive-start` against `env-twin`).
- K against C, paired in the same rounds: `+0.16 ms`, range `-1.68` to `+1.80`, 13 of 24 positive,
  which is not a difference this harness resolves;
  the fourth revision's "about `0.6 ms`" was the difference of two medians and is withdrawn.
  K2 against C is `+0.12 ms`, range `-1.40` to `+1.38`, 14 of 24.
- `ssh -G` with the configuration D resolves under: `+3.33 ms` over one process start,
  range `+2.40` to `+6.48`, 24 of 24; its twin is `+0.01 ms`.
- `ssh-keygen -F` over a file that holds the host: `+2.38 ms`, range `+1.15` to `+4.23`, 24 of 24;
  its twin is `-0.27 ms`.
- D's whole check, both starts, with the configuration that asks and the file that holds the host:
  `+7.50 ms`, range `+5.23` to `+10.55`, 24 of 24; its twin is `-0.15 ms`.
  The fourth revision's `+8.62 ms` measured one start, because its fixture short-circuited the second.
- `git remote get-url`, the query that names the host as the person typed it when they named a remote:
  `+0.85 ms`, range `-0.59` to `+1.50`, 23 of 24; its twin is `-0.21 ms`.

### The typed path

Run, `keep/run-4-typed-path.log`, 24 rotated rounds of `git ls-remote` of one repository over loopback,
with the control at 126.92 ms and the same command run twice at 127.56 ms.

- the floor: `+0.11 ms`, range `-6.18` to `+7.93`, 13 of 24 positive;
- C: `+1.62 ms`, range `-6.57` to `+5.83`, 17 of 24, inside the floor;
- K: `+1.05 ms`, range `-5.89` to `+9.94`, 17 of 24, inside the floor;
- K2: `+0.54 ms`, range `-7.86` to `+5.13`, 17 of 24, inside the floor;
- C2's status-only shape: `+1.26 ms`, range `-5.39` to `+8.80`, 15 of 24, inside the floor;
- D with both starts: `+5.76 ms`, range `-3.62` to `+14.01`, 23 of 24, the only figure this harness resolves.

### What the figures support, and what they do not

- C, K, K2 and C2 all add between half a millisecond and one millisecond of work to a command that meets no question,
  and no typed-path harness here can tell them apart.
- D costs about `7.5 ms` of work in two starts on every typed network command, which is the only option cost a person
  would feel on a fast connection.
- Every figure is a loopback round trip inside a two-CPU container on this machine.
  A person's network dominates all of them, so the numbers say what the wrapper adds, not what a command takes.
- The figures do not transfer to another machine, another container, or another release of Git or OpenSSH.
  Even two rounds on this image disagree: the fourth revision's typed-path floor was median `+1.59 ms` with a range
  of `-8.23` to `+18.15`, and this round's is `+0.11 ms` with `-6.18` to `+7.93`.
  That is why every figure here is quoted against a twin measured in its own rounds.

## What the probes did not show

- No run of the wrapper itself.
  No wrapper code was changed this round, so every option is stood in for by a compiled stand-in
  (`keep/stand-ins/standin.c`), and every sentence printed is one of the words under "The words".
  What the arms prove is the delivery, the decision and the cost, not the wrapper's reading of a repository.
- Nothing ran under this host's own `ssh` except two arms.
  The `git//ssh` stack refuses the shell a `ProxyCommand` needs, so no proxy arm can run under `/usr/bin/ssh` here,
  which is why every arm uses a copy of `ssh` under `/work`.
  That copy runs under the `git` profile alone, which is not what a person on this host meets.
  The two arms that did run under `/usr/bin/ssh` are the `f-confined` arms of the round whose folder is
  `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/`,
  which showed A's value and J's value direct and through a `ProxyCommand`, and no other option ran there.
  So a K, K2, C, C2 or D arm under the `git//ssh` stack is unexercised,
  and the profile's console abstractions say nothing about whether a pseudo-terminal under it would be allowed.
- No security key.
  The touch notice's behaviour under a pipe or a pseudo-terminal is read from `readpass.c:252`, `:253` and `:279`,
  not measured: no key was available.
- No signal aimed at the wrapper alone.
  The stand-in runs no `setsid` and no `TIOCSCTTY`, so the arms' signals are the person's,
  but no arm sends a signal to the wrapper only, so what the wrapper must do with one is design.
- No arm with `Match exec`, `CanonicalizeHostname`, `HostKeyAlias` or `VerifyHostKeyDNS`.
  The `KnownHostsCommand` and `@cert-authority` results are earlier rounds' and are quoted in D's section.
- `f-known-missing` ran directly only.
  Every other failure family ran behind a `ProxyCommand` as well, and `f-locked` behind one too.
- No Windows, no OpenSSH release other than 10.6p2, and no Git release other than 2.56.1.
- No whole container gate, and no `native:check`, clippy, `lint:types` or host tests.
  The only repository change is documentation, and the round's note says no whole gate is a merge condition.
- The Markdown lint did not run.
  `monochromatic-lint` is not installed in this worktree and `mise run toolchain:install` is barred this round;
  the document was rendered with `pandoc -f gfm -t html` and read for what a renderer shows.

## The earlier revisions, kept for their record

The record of the rounds that led here, named by the folder they wrote to and the commit that merged each revision,
because the two questions they turned on are answered and their evidence is still cited in each option's section.

### The third revision, `1f6f120f64` (2026-11-24T09:01:38-05:00)

Its findings, with the evidence against the ones that did not hold:

1.  B's transcript was not the run's bytes.  The round that followed re-ran every B arm with the kept stand-in and
   printed the bytes.
2.  The owner's self-start rule was not quoted, and B and E meet it.
    The rule is quoted under "What the recorded rules leave open";
   the owner then answered "Never" for both.
3.  The rotation was not a rotation and the noise test was empty.  The timings were re-run in a balanced Latin
   square, and the test became a sign in every round plus a median larger than a same-command twin.
   This round keeps the twin and states the positive count instead of requiring every round.
4.  The `-G` start count was wrong for the baseline.  A named `ssh` is started once, a differently named program
   twice; E's arm was relabelled and split.
5.  C2's arm did not show what the text said.  The fourth revision ran a real check, and this one runs three
   shapes of it.
6.  The coverage sentence in the B over C pair was wrong in both directions.  B covers the differing key; C also
   speaks for `BatchMode=yes` and `StrictHostKeyChecking=yes`.
7.  D has a false positive the second round did not find.  Reproduced with `KnownHostsCommand`, and the
   `@cert-authority` guess did not hold.
8.  J's example named a host J cannot know.  Its value was rebuilt with a placeholder.
9.  The selector's supporting claims were partly wrong.  The file-name fallback and the bare-word claim were
   dropped.
10. "Nothing else is needed" for AppArmor was inference, and now says so.
11. The "byte for byte" claim was not measured.  It was measured then, and again this round for C, K2 and K with
    its fixes.
12. Stale line numbers.  Cites are now a heading, a symbol, or a path with the sentence quoted.
13. A misquoted comma in the old handover's sentence.
14. Printed commands were unquoted, and OpenSSH cuts a key path at 100 characters.
15. The panic hook ran after the mode would sit, and "flush" was the wrong mechanism.
16. A typed LFS push would say the same thing twice.
17. A container outlived the second round's report.
    It is not this round's, and this round left `quizzical_leavitt` and `typed-network-next-step-probe-2026-11-24`
    alone, because both predate it.

### The fourth revision, `bda96883d5` (2026-11-25T13:54:27-05:00), merged as `ec0b9e7ea3`

That revision retired J and added K.
Its words are kept here because the gate review found that they could not be printed from what K reads,
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

Its C2 shape was one option where there are three, its ranked pair reasons compared things that are not alike,
and its D cost arm never ran D's second start. All three are answered in the sections named with them.

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
  `reads_as_a_key_failure`, `login_needed_message`, `host_key_not_known_sentence`,
  `HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`) and `lfs/lfs_upload_explanations.rs` (`explanation_after_failure`).
- `package/git-policy/cli/src/native/push/push_retry_failure.rs` (`retried_failure_record`).
- `package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs` (the rules and their
  exceptions), `src/native/invocation/real_git.rs` (`own-executable`'s exception).
- `package/git-policy/cli/SPEC.md`, "No command the wrapper runs or forwards asks a person for a credential",
  "The wrapper never starts itself as Git", and the paragraph saying this release names no next step at a new host key.
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
- Git v2.56.1: `pager.c` (`term_columns`), `progress.c` (`is_foreground_fd`, the progress gate),
  `sideband.c` (the `remote: ` prefix), `transport.c` (`ret->progress`), `editor.c` (the waiting hint),
  `connect.c`, `prompt.c`, `strbuf.c`.
- This round's folder, `~/temp/agent/host-key-next-step-design-after-its-gate-review-2026-11-25/keep/`:
  `run-1-behaviour.log` and `run-1-behaviour.sh`,
  `run-2-bytes-and-progress.log` and its script,
  `run-3-steps.log` and its script,
  `run-4-typed-path.log` and its script,
  `explore.log`, `explore2.log`, `explore3.log` and their scripts,
  `lib.sh`, `stand-ins/standin.c`, `stand-ins/proxy.c`, `stand-ins/ptyrun.c`,
  `sentences/host-key.txt`, `sentences/strict.txt`, `sentences/passphrase.txt`, `sentences/remote-line.txt`,
  `derived-run-1-summary.txt`, `derived-medians.txt`, and `state-so-far.md`.
- The gate review this revision answers:
  `~/temp/agent/gate-review-host-key-next-step-after-askpass-false-2026-11-25/keep/review.md`.
- The askpass round's fix, `c336355bcd` (2026-11-25T13:21:59-05:00): `NO_ANSWER_PROGRAM`, the `ProxyCommand`
  control, and the `SPEC.md` and `README.md` text.
- The release pass's finding and its container run:
  `~/temp/agent/release-pass-wrapper-alpha-46-2026-11-25/commit-push-lfs-lock-and-the-rest/keep/review.md`
  (H1 and "For the owner"), and the OpenSSH sources kept beside it.
- The round that first measured the byte comparison and the changed-key arms:
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`, `run5.log`, `run6.log`,
  `run9.log`, and its `probe-bin/` and `probe-scripts/`.
- The round that wrote the fourth revision and measured C2's shapes, `OPOST`'s transcript and K's redirected
  progress: `~/temp/agent/host-key-next-step-design-after-askpass-false-2026-11-25/keep/`, merged as `ec0b9e7ea3`
  (2026-11-25T13:57:48-05:00).
