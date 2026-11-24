# The next step a typed network command names at a new host key or a missing passphrase

## Status

A proposal,
put to the owner.
It is not a decision,
nothing here is approved or final,
and this revision is not final either.
It designs the words a typed `git clone`,
`fetch`,
`pull` or non-LFS `push` can print when OpenSSH refuses a new host key,
or when a key needs a passphrase nobody can be asked for,
and it ranks the ways the wrapper could do that under the project's recorded rules.

This is the third round on the same question.
The first round's proposal was merged as `41bdf36a2a`,
the second as `082c3ff8eb`,
and the second was read by a same-vendor second pass kept at
`~/temp/agent/gate-review-typed-network-command-revision-2026-11-24/keep/read-of-the-revised-host-key-design.md`.
This round folds that read,
re-runs the arms it called unresolved,
and ranks twice so the owner can answer the one question it raises either way.
The read is a same-vendor second pass,
so it is the only review this design has had,
and no finding of it is treated here as settled.

Round folder:
`~/temp/agent/typed-network-command-next-step-third-revision-2026-11-24/`
(`keep/run-1.log`, `keep/run-2.log` and `keep/run-3.log` hold every transcript and timing this document cites;
`keep/scripts/` holds the scripts that produced them).
Worktree:
`.claude/worktrees/typed-network-command-next-step-third-revision`,
branch `docs/planning-cli-git-typed-network-command-next-step-third-revision`,
made from `main` at `442710344e`.
Nothing under `doc/decision/` or `doc/handover/` was changed by this round,
and no wrapper code was changed.

Every transcript in this document is labelled,
because the read's first finding was that the second round printed text its run had not printed:

- **run** means a program printed these bytes, and the log that holds them is named with it;
- **composed** means no program printed them: the text was put together from other facts;
- **source** means read from a file, at the tag or the path named with it;
- **unrun** means neither.

## What this revision changed

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

Measured in a rootless container of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85`, `OpenSSH_10.6p2`, `git 2.56.1`)
against a throwaway `sshd` on loopback,
with a fixture host key,
a fresh `known_hosts` and throwaway client keys.
The wrapper is the worktree's release build (`0.6.1-alpha.44`, 27971352 bytes).
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

That is run, this round (`keep/run-1.log`, `keep/run-2.log`; script `02-byte-compare.sh` and `06-corrections.sh`).

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

Whether an askpass start, or a wrapper started as ssh, is "asked to explain" is therefore the owner's call,
and it is not settled here.
The ranking is given twice so either answer can be read off it:
its first form excludes B and E,
and its second grants B's exception.
That question is put in "The owner's question".

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

## The options

Each option says what it starts,
what it reads,
whether it can answer yes,
what a person sees for each case from a run,
what it costs on a command that meets no question,
and its pros and cons.
The ranking and its adjacent-pair reasons are in "The ranking".
Every cost is against a null arm run in the same round:
`git ls-remote` with the same ssh and the same identity options.
The noise floor is the difference between two runs of that same command in the same round:
median `-0.29 ms`, range `-2.20` to `+1.87 ms`, nine of twenty-four rounds positive
(run, `keep/run-2.log`).
A formula below that does not clear `+1.87 ms` with a consistent sign is called noise.

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

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

Mechanism: the overlay keeps `SSH_ASKPASS_REQUIRE=force` and names the wrapper's own path in `SSH_ASKPASS`
instead of `false`.
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

Mechanism: `SSH_ASKPASS` holds the sentence, not a program.
`execlp` tries to execute a file named the sentence, fails,
and OpenSSH prints its own failure line with the value in it
(`fatal_f("exec(%s): %s", askpass, strerror(errno))`, `readpass.c:78`).
No program, no mode, no selector, no AppArmor rule, no process.

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

What it starts: Git as a child, one pipe, one relay thread or loop.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

Cost on a command that meets no question,
against the null arm in the same rounds: `+0.43 ms` median, range `-1.47` to `+2.38`, seventeen of twenty-four
positive
(run, `keep/run-2.log`, arm `relay_git`).
That is inside the noise floor, so this round cannot say C costs a measurable amount on such a command.

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

Cost on a command that meets no question,
against the null arm: `+0.72 ms` median, range `-1.47` to `+2.31`, eighteen of twenty-four positive
(run, `keep/run-2.log`, arm `forkwait_git`).
Inside the floor, so not a finding;
on a success it is one fork and one wait, with the same streams.

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

### E. The wrapper becomes the ssh that Git starts

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

Two rankings,
because the self-start rule has two readings and the owner has not chosen between them.
Both rank by what each option delivers to the person who typed the command:
whether the words arrive at the refusal that stopped them,
whether they name what the person must do,
whether they appear when nothing is wrong,
and what the person gives up on every other command.

### Read strictly: B and E leave the field

**J > C > C2 > D > A.**

- **J over C.**
  J speaks to both refusals at the moment they happen and takes nothing from any command:
  no process, no progress, no notice, no setting.
  C's words are plainer and name the host,
  which J cannot,
  but C takes the live progress display and the security-key touch notice from every clone and fetch,
  and its words arrive after Git has already said the connection failed.
  That trade is the pair's whole content,
  and it reverses the second round's C over J,
  whose stated reason left out the progress loss that its own earlier pair called decisive.
- **C over C2.**
  C's words are chosen from what OpenSSH actually printed, so they are right about the refusal.
  C2 guesses from exit 128, which a missing repository, a refused connection, a changed key, `BatchMode=yes` and
  `StrictHostKeyChecking=yes` all produce,
  so it prints a paragraph about keys on failures that have nothing to do with them (measured).
  C2's advantages are real: it keeps the progress display, the terminal and the signals, and it survives `-q`.
  Wrong words at a refusal misteach more than silence does,
  and C2 ranks above D and A for its cost and its streams, not for its accuracy.
- **C2 over D.**
  D's words arrive before the command, but the command runs anyway,
  so they cannot let a person stop;
  they never cover a passphrase;
  they are wrong on commands that succeed when `KnownHostsCommand` supplies the key (measured this round);
  and every network command pays about 14 ms and one extra run of the person's `Match exec` commands.
  C2 costs nothing measurable, keeps everything on a successful command, and speaks on failures.
  Both can misteach; C2's misteaching is an extra paragraph after a real error,
  and D's is a first line that can be wrong before the truth arrives, plus a cost on every command.
- **D over A.**
  D speaks accurately for the plain new-key case, which is the common one, and A never speaks.
  D pays for that on every command;
  A pays nothing and leaves the person with no next step at the first contact with each host.

### With B's exception granted

**B > J > C > E > C2 > D > A.**

Granting B's exception covers E as well, because both are the same act:
a process Git started explains the failure.
With that allowed, delivery decides, and the order of the options that were already in the strict ranking does
not change.

- **B over J.**
  B's words name the host, the address, the port and the key,
  arrive between Git's first line and OpenSSH's,
  and survive `-q`, which J's do not;
  B costs nothing on a command that meets no question, as J does.
  B's price is a mode, a selector, one askpass start per question, and, on this host, two AppArmor rules per build.
  Before those rules are in place, B's words do not reach the person on this host at all
  (`ssh_askpass: exec(<path>): Permission denied`), and B ranks below A until they are added.
- **J over C**, for the reasons in the strict ranking: zero cost and both refusals against plain words bought with
  the progress display.
- **C over E.**
  Both read the same detector words, so both are right about the same refusals and miss the same ones.
  E's words sit closer to the cause, between OpenSSH's line and Git's;
  C's arrive after Git's advice.
  C keeps the person's transport settings and takes the progress display;
  E keeps the progress display and takes the transport settings,
  which can change which host, port or key the command uses, and cannot be preserved without shell text.
  A wrong setting breaks the command;
  a lost progress bar does not.
  E also costs about 6 ms on every network command, and C's cost is inside the noise floor.
- **E over C2.**
  E's words are chosen from the refusal; C2's are guessed from a status that many failures share.
  E pays about 6 ms and takes a setting;
  C2 costs nothing measurable and keeps everything.
  This is the closest pair after J over C, and it turns on how much the person's own transport command matters.
- **C2 over D**, and **D over A**, for the reasons in the strict ranking.

### What each answer gives the owner

- Read strictly: J is the first option, and it needs no code, no mode and no host change.
  C is the option the recorded shape prefers, at the price of the progress display.
- With B's exception and the two AppArmor rules: B is first, and the words are the best this design can print.
- With B's exception but no root edit on this host: B's words do not print here, so J and then C are the field, as
  under the strict reading.

## The owner's question

The read raised it and this round does not answer it:

> Does the self-start rule's third bullet
> ("Nothing that Git or a hook starts is asked to explain anything")
> allow an askpass start of the wrapper, and a wrapper started as Git's ssh, to print the explanation?

Two answers, with what each one leaves:

- **Yes, exception granted.** The ranking is B > J > C > E > C2 > D > A.
  B's words are the best available, and E's are the best placed.
  The price is two AppArmor rules per build on this host, a selector, and a mode that is started by another
  program.
- **No, read strictly.** B and E leave the field, and the ranking is J > C > C2 > D > A.
  The price is that no option names the host and the key at the refusal except C's, which costs the progress
  display.

The question belongs beside that L4 entry in `doc/handover/questions-for-the-owner.md`;
this round may not edit that file,
so it is put here and in this round's report,
and the proposal waits on the answer before any of the three is built.

## The words

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

## What building B would take

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

## What the probes did not show

- **No run of the wrapper's own askpass mode.**
  No wrapper code was changed this round,
  so B's contract is the kept stand-in from the second round,
  whose bytes this round printed beside its digest
  (`keep/run-1.log`; `askpass-b`, 1513 bytes).
- **No AppArmor profile was added** (that needs root).
  The denials and the rules needed are reported instead,
  and the live run is unconfined by design,
  because a confined ssh denies every askpass program but the three the profile names.
- **No agent-based hazard probe**: `ssh-agent` cannot bind its socket in the container
  (the second round's `keep/run5.log`),
  so the exit-zero hazard was probed through a multiplexed master instead, which worked.
- **No security key**: the touch notice is source only.
- **No `HostKeyAlias`, `ProxyJump` or `CanonicalizeHostname` arm**, where D's and E's readings are weakest.
- **No `VerifyHostKeyDNS` arm**: source only, as D's section says.
- **No C arm at `BatchMode=yes` or `StrictHostKeyChecking=yes`**: the detector's source says C would speak, and only
  C2's stand-in ran those cases.
- **No C2 check**: no arm ran a post-failure known-hosts check,
  so C2's accuracy with one is design.
- **No Windows build**, and no OpenSSH release other than 10.6p2 or Git release other than 2.56.1.
- **No terminal at this machine's own confined `git`**: the container's arms are unconfined on purpose.
- **No repetition of the refused-question timing**: it is the second round's `+3.6 ms` figure, which its spread
  swallowed.
- **No whole container gate, and no `native:check`, clippy, `lint:types` or host tests**:
  the only repository change is documentation, and the round's note says no whole gate is a merge condition.

## Measurements

Three container runs of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85`),
each `--cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
started in a `systemd-run --user` scope with `RuntimeMaxSec`, `CPUQuota=200%`, `MemoryMax=4G`, `TasksMax=1024`
and `IOWeight=10`, each `--rm`, each named
`typed-network-next-step-third-revision-probe<n>-2026-11-24`.
OpenSSH 10.6p2, Git 2.56.1, the worktree's release wrapper build (`0.6.1-alpha.44`, 27971352 bytes),
`sshd` on 127.0.0.1:2222.
Logs: `keep/run-1.log` (the arms and the comparison), `keep/run-2.log` (D's classes, the timings),
`keep/run-3.log` (J's value, the progress counts).

Method, so a difference can be told from the host's noise:

- a balanced Latin square of order 12 for eleven arms and one unmeasured slot,
  so every arm follows every other arm twice and sits at every position twice,
  twenty-four measured rounds after two warm-up rounds
  (checked from the order table in `keep/run-2.log`: every ordered pair appears twice);
- the figure for an arm is the median of its per-round difference from `null_a` in the same round,
  with the range of those twenty-four differences beside it;
- the noise floor is `null_b` against `null_a`,
  two runs of the same command in the same round:
  median `-0.29 ms`, range `-2.20` to `+1.87 ms`;
- an arm is a finding when its sign holds in every round and its median clears the floor's widest reading.

The baseline, `git ls-remote` with the same ssh and identity options, took `119.49 ms` median
(`117.67` to `120.48`).
Against it, in those same rounds:

- today's wrapper (exec, release build): **`+3.48 ms`** (`1.50` to `5.90`), 24 of 24 positive;
- B, on a command that meets no question: **`-0.50 ms`** (`-2.07` to `2.17`), 9 of 24 positive, noise;
- C's relay: **`+0.43 ms`** (`-1.47` to `2.38`), 17 of 24, noise;
- C2's fork and wait: **`+0.72 ms`** (`-1.47` to `2.31`), 18 of 24, noise;
- D's check before the command: **`+14.13 ms`** (`12.21` to `16.55`), 24 of 24, a finding;
- E's relay around an unrecognised ssh name: **`+5.97 ms`** (`3.57` to `8.12`), 24 of 24, a finding;
- the same ssh under a name Git does not recognise, without any relay: **`+4.93 ms`** (`2.03` to `7.03`),
  24 of 24, so about `1.04 ms` of E's figure is the relay and the rest is Git's `-G` probe;
- one extra process start alone: `relay_true` took `2.02 ms` absolute, against a command it cannot be paired with
  in this design, so it is context and not a finding;
- the wrapper's own start with one question-shaped argument: `11.67 ms` absolute (`10.50` to `12.31`).

What that supports and what it does not:

- D and E cost more on every network command, and the wrapper's own start does too;
  all three keep their sign in every round and clear the floor.
- B, C and C2 cannot be shown to cost anything measurable on a command that meets no question,
  and B costs nothing by construction, because no process starts.
- The second round's figures (wrapper `+11.26 ms` on a dev build, C `+2.16`, C2 `+2.84`, D `+28.97`,
  E `+14.68`) came from a cyclic order in which each arm kept its predecessor, and from a dev build.
  This round's replace them for the decisions they support;
  D's magnitude disagrees between the two runs and its sign does not.
- Every figure here is a loopback round trip inside a two-CPU container.
  A person's network dominates all of them, so the absolute times say what the wrapper adds, not what a command
  takes.

Counts and outcomes:

- **progress at a pseudo-terminal**, one clone of two thousand objects (`keep/run-3.log`):
  Git's standard error on the terminal, and under C2, E and today's wrapper, show a `Receiving objects` line,
  a `remote: Counting objects` line and a `remote: Enumerating objects` line;
  through C's pipe none of the three appear;
  through C's pipe with `--progress` all three return.
- **starts** (`keep/run-1.log`): plain git with `ssh` on `PATH` 1; a named `ssh` 1; a named `renamed-ssh` 2;
  E's stand-in 2, with 2 ssh starts under it.
- **D's classes** (`keep/run-1.log`, `keep/run-2.log`):
  a key supplied by `KnownHostsCommand` gets D's sentence while the real clone exits 0;
  an `@cert-authority` line is found by `ssh-keygen -F`, and D stays silent while the real clone exits 0;
  the control without either gets the sentence and exit 128.
- **the deleted running binary** (run on the host): `readlink /proc/<pid>/exe` gives
  `<path> (deleted)` after the file is removed, which is the path a long-running service remembers.
- **the byte comparison** (`keep/run-1.log`, `keep/run-2.log`):
  identical standard output, standard error and exit status for a new host key (128),
  a passphrase key (128), and a `pull` (1).

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
- OpenSSH `V_10_6_P2`: `readpass.c`, `sshconnect.c`, `sshconnect2.c`, `ssh.c`, `log.c`, `log.h`, `readconf.c`, `mux.c`.
- Git 2.56.1: `prompt.c`, `connect.c`, `sideband.c`, `transport.c`, `editor.c`, `strbuf.c`.
- `keep/run-1.log`, `keep/run-2.log`, `keep/run-3.log` and `keep/scripts/` in this round's folder.
- The second round's logs, cited arm by arm:
  `~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/keep/run4.log`,
  `run5.log`, `run6.log`, `run9.log`, and its `probe-bin/` and `probe-scripts/`.
