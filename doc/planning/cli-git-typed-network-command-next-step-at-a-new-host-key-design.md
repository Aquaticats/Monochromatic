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

This is the second round on the same question.
The first round's proposal was merged as `41bdf36a2a` and sent to a gate review;
the review is kept at
`~/temp/agent/gate-review-typed-network-command-next-step-2026-11-24/keep/review-of-the-host-key-next-step-design.md`.
This round folds every finding,
adds the review's two candidates as designed options,
runs the option the first round could not run at all,
and re-ranks.
Where a statement here rests on a run,
the run is named;
where it rests on source,
the file and line are named;
where it rests on neither,
it says so.

Round folder:
`~/temp/agent/typed-network-command-next-step-after-its-review-2026-11-24/`
(`keep/` holds the transcripts and timings this document cites).
Worktree:
`.claude/worktrees/typed-network-command-next-step-after-its-review`,
branch `docs/planning-cli-git-typed-network-command-next-step-after-its-review`.
Nothing under `doc/decision/` or `doc/handover/` was changed by this round,
and no wrapper code was changed.

## What this revision changed

The gate review is a same-vendor second pass,
so it is the only review this design has had.
Its findings and this round's answer:

- B's transcripts were composed rather than probed,
  and B could not run at all where AppArmor confines the ssh that Git starts.
  This round found that a copy of `git` and `ssh` under `/work` runs unconfined inside the gate image
  (measured label `unconfined`),
  so B's contract was run live in a throwaway container,
  and the real wrapper was also run as the askpass program to show what today's binary does instead.
  B's transcripts here come from that run, with the mode's contract implemented as a stand-in,
  because no wrapper code was changed.
- B's passphrase sentence was false when ssh went on to a second key that worked.
  The sentence now says what is true at that moment,
  and the case that broke it is kept as its own arm:
  the command succeeds with the sentence printed (`keep/run4.log`, arm `b-passkey-then-good`, exit 0).
- B cannot know the login name,
  so the connect form it prints names no user,
  and the address's parenthesised duplicate is dropped:
  from `The authenticity of host '[127.0.0.1]:2222 ([127.0.0.1]:2222)'` it prints `ssh -p 2222 127.0.0.1`.
- The AppArmor need is stated as two stacked profiles and the exact paths,
  per build where the path carries a build,
  in "What building B would take".
- The selector no longer depends on a trailing `: ` or `? `:
  one argument that does not start with `-` and holds a space,
  or OpenSSH's own `SSH_ASKPASS_PROMPT`,
  now selects the mode,
  and an unrecognised question gets a generic sentence instead of falling through to Git.
- Every branch of the mode exits non-zero,
  and the reason is measured:
  an askpass that exits zero with nothing on standard output is read as **yes** by `ask_permission`,
  and a multiplexed session is then allowed (`keep/run6.log`).
- D now reads every file `ssh -G` names (both known-hosts lists),
  honours `HostKeyAlias`,
  reads the port,
  and names what `ssh -G` costs:
  it runs the person's `Match exec` commands (1 run for plain git, 2 with D's check, measured),
  and it can do DNS work under `CanonicalizeHostname` (source only, unrun).
- C's two losses are counted separately (pipe and process, not one change),
  the progress loss is measured again at a pseudo-terminal,
  and the other terminal-dependent behaviours and the security-key notice are named.
- E's cons are recounted with its own null arm and with the ssh starts counted.
- Every timing is recomputed against null arms run in the same rounds,
  and the ones that do not clear their own spread are called noise.
- The lows are folded:
  Git's askpass read is a size hint and not a 20-byte cap,
  the third sentence now carries the offending file and line because the earlier text was not on standard error,
  the no-prompt rule's current home is quoted,
  and the mode's hazards (flush before exit, server text, `BatchMode`, `StrictHostKeyChecking=yes`,
  a stale own path) are stated with the branch each one takes.
- The review's two candidates are now designed options:
  J, the sentence itself as `SSH_ASKPASS`'s value, and C2, Git kept as a child with the same streams.

One more thing changed because a probe showed it:
B's, J's, C's and E's words vanish when the person asks for quiet,
and C2's do not.
That is measured for all four (`keep/run4.log`, arms `b-quiet`, `j-quiet`, `c-quiet`, `e-quiet`, `c2-quiet`)
and it moves C2 ahead of E.

## What the person sees today

Measured through this round's build of the wrapper (`0.6.1-alpha.45`, dev profile)
against a throwaway `sshd` in a rootless container of the gate image,
with a fixture host key,
a fresh `known_hosts` and throwaway client keys.
The wrapper and the real Git were copies under `/work`,
which no AppArmor profile attaches to,
and the ssh each arm used was a copy under `/work` too,
so the arm shows OpenSSH's own behaviour and not this host's confinement.
`keep/run4.log` holds every transcript.

A new host key,
for `git clone`,
`git fetch`,
`git push` and `git ls-remote`,
ends this way:

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

`git pull` prints the same lines and ends 1, not 128 (measured).
`git ls-remote` prints the same two lines without `Cloning into`.

A changed host key for a host this machine already knows ends 128 with OpenSSH's own banner,
`Offending ED25519 key in <file>:<line>`,
`Host key for <host> has changed and you have requested strict checking.`
and the same two Git lines (`keep/run5.log`, arm `b-changed-key`).

A key whose passphrase nobody can be asked for ends this way:

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone'...
root@127.0.0.1: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The same commands run by the real Git with no overlay print the same lines in both cases;
the first round measured that byte for byte and this round reproduced the failure and its exit status,
not the byte comparison.

Why the wrapper cannot say anything today,
read from the source:

- a typed command is handed over by replacing the wrapper's process image,
  so nothing of the wrapper is left to read anything
  (`replace_process_with_real_git`,
  `package/git-policy/cli/src/native/invocation/forwarding.rs:572`,
  called at `src/native/invocation/entry.rs:385` and
  `src/native/invocation/entry_run_forwarded_actions.rs:145`):
  same process number,
  same streams,
  same signals,
  the terminal handed over.
- the rule that forbids the question is one list,
  `no_prompt_overlay`
  (`src/native/invocation/child_environment.rs:559`):
  `GIT_ASKPASS` empty,
  `GIT_TERMINAL_PROMPT=0`,
  `SSH_ASKPASS=false`,
  `SSH_ASKPASS_REQUIRE=force`.
- the wrapper already has two sentences of this kind,
  and they reach only the two places where the wrapper owns the child:
  the LFS upload's explanation
  (`explanation_after_failure`,
  `src/native/lfs/lfs_upload_explanations.rs:220`)
  and the service's push failure record
  (`retried_failure_record`,
  `src/native/push/push_retry_failure.rs:295`),
  both printing `host_key_not_known_sentence`
  (`src/native/lfs/lfs_pre_push.rs:644`)
  and `login_needed_message`
  (`src/native/lfs/lfs_pre_push.rs:694`).
  Both sentences are about a push ("then push again"),
  which is why a typed `fetch` or `clone` cannot reuse them word for word.

The owner's question for this waits in
`doc/handover/questions-for-the-owner.md`,
section "Words for a typed command at an unknown host key or a missing passphrase (the pass's L4)".

## The rule that forbids the question, and where it lives now

The review traced the rule to `SPEC.md` and to a handover entry that has since moved.
It lives in `package/git-policy/cli/SPEC.md:8490` onward,
and in `package/git-policy/cli/README.md:7265`,
and it reads:

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

The same entry in the handover it was sorted out of,
`git show c84e2f1dbf:doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md`
(line 44404 onward),
records the owner's words and what the round after it did with them:

```text
**The owner answered H1 and M4** (this entry): H1 "Hide long words only" (path words of 20 or more unbroken lowercase
letters print as `<credentials>`); M4, in the owner's words, "Avoid ever asking for someone to type a
username or password.". so no command the wrapper runs or forwards prompts for a credential, terminal or not, superseding
`SPEC.md`'s "A command the person typed is forwarded with the prompting Git has" for credentials; asked separately, an
SSH key's passphrase counts too ("Passphrases count too (Recommended)": a key not in ssh-agent fails at once, saying to
run `ssh-add`).
```

Two facts about that rule matter to this design,
and the review is right about both:

- the SPEC's parenthetical still points at the handover entry,
  which no longer exists;
  the rule's own text is the record now.
- `SPEC.md:8577` says "a host key is not a credential the push lacked",
  so the credential rule does not itself forbid answering a host-key question.
  What forbids it is the owner's answer of 2026-11-20 that the refusal is explained rather than answered
  (`SPEC.md:8479` onward:
  "A host key this machine does not know is no stop: the service keeps its usual retry schedule for it,
  and its record says that it pushes the branch by itself once the key is known",
  "the message says the host key is unknown, how to fix it,
  and that the service will push by itself once the key is known"),
  and the round's own brief.
  Every option here refuses to answer yes;
  the ones that could answer yes are listed and not ranked.

## What OpenSSH does with an askpass program

Read from `V_10_6_P2`,
the release this machine runs (`ssh -V` prints `OpenSSH_10.6p2`).
The files were fetched from `raw.githubusercontent.com/openssh/openssh-portable/V_10_6_P2/<file>`.

`ssh_askpass` runs the program,
in `readpass.c:47` to `:108`:

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
  Measured: an askpass that reads one byte hung the clone until a 20 s bound ended it
  (`keep/run4.log`, arm `b-askpass-reads-stdin`, exit 124).

`read_passphrase` decides whether the program runs at all,
in `readpass.c:119` to `:194`:

- `SSH_ASKPASS_REQUIRE=force` sets both `use_askpass` and `allow_askpass`
  (`readpass.c:130` to `:133`);
- `prefer`, `never` and an absent value leave the terminal in play
  (`readpass.c:134` to `:138`);
- a forced question with no answer from the program returns the empty string
  (`readpass.c:179` to `:182`),
  and the same happens to a question that must use the program while `never` forbids it
  (`readpass.c:169`, `:170`);
- otherwise the question goes to `/dev/tty` (`readpass.c:185`).

`ask_permission` (`readpass.c:196` to `:221`) is the one place where an empty answer means yes:

```c
	if (p != NULL) {
		/*
		 * Accept empty responses and responses consisting
		 * of the word "yes" as affirmative.
		 */
		if (*p == '\0' || *p == '\n' ||
		    strcasecmp(p, "yes") == 0)
			allowed = 1;
		free(p);
	}
```

So a program that prints nothing and exits zero answers yes to a multiplexing request,
an agent-addition request and a forwarding request.
Measured both ways (`keep/run6.log`):
a master started with an askpass that exits zero and prints nothing allowed a second session through it,
and the same master with an askpass that exits two refused it
(`Master refused session request: Permission denied`).

The host key question is `confirm`,
`sshconnect.c:603` to `:627`:
it loops on `read_passphrase(msg, RP_ECHO)`,
returns zero when that gives nothing
(`sshconnect.c:614`, `:615`),
and accepts only the empty string,
`no`,
`yes` or the fingerprint it was given as answers
(`sshconnect.c:616` to `:622`).
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
(`sshconnect.c:1211` to `:1246`).
Measured text (`keep/run4.log`, arm `q-newkey`):

```text
The authenticity of host '[127.0.0.1]:2222 ([127.0.0.1]:2222)' can't be established.
ED25519 key fingerprint is: SHA256:nuiqi+GecRF2R8r5i8kKeJX1HN4gSd2nxRFX3V8FkFk
This key is not known by any other names.
Are you sure you want to continue connecting (yes/no/[fingerprint])? 
```

Two facts about the other host-key states,
which the first round did not have:

- a **changed** host key never asks.
  With strict checking `ask` the code prints the warning banner and
  `Host key for <host> has changed and you have requested strict checking.`
  and fails (`sshconnect.c:1355` to `:1366`).
  Measured: no askpass start, exit 128 (`keep/run5.log`, arm `q-changed-key`).
- a **differing key for the address** (`CheckHostIP=yes`) does ask,
  and is the only differing-key question there is.
  Its text carries the offending file and line,
  measured from a non-loopback name, because loopback and a proxy command switch `CheckHostIP` off
  (`sshconnect.c:1040` to `:1045`; `keep/run5.log`, arm `q-differing-key`):

```text
Warning: the ED25519 host key for '[probe-host.test]:2222' differs from the key for the IP address '[192.168.253.119]:2222'
Offending key for IP in /work/out/kh-differing:2
Matching host key in /work/out/kh-differing:1
Are you sure you want to continue connecting (yes/no)? 
```

  `CheckHostIP` defaults to off (`readconf.c:2909`, `:2910`),
  so this question is rare and must not shape the design;
  the first round's third sentence assumed it on every differing key and named a line the person cannot see.

A key's passphrase is `load_identity_file`,
`sshconnect2.c:1533` to `:1575`:
the question is `Enter passphrase for key '<path>': `
(`sshconnect2.c:1547`, `:1548`),
and an empty answer is read as one nobody gave,
so the key is skipped and the next one is tried
(`sshconnect2.c:1553` to `:1558`).
That is why a sentence printed at this question cannot claim the connection failed.

Two settings remove the question instead of answering it,
and both were probed:

- `BatchMode=yes` returns zero from `confirm` before anything is asked
  (`sshconnect.c:610`, `:611`):
  measured, no askpass start and no words (`keep/run4.log`, arm `b-batchmode`).
- `StrictHostKeyChecking=yes` fails with OpenSSH's own sentence
  `No ED25519 host key is known for [127.0.0.1]:2222 and you have requested strict checking.`
  (`sshconnect.c:1198` to `:1206`):
  measured, no askpass start (`keep/run4.log`, arm `b-strict-yes`).

Things that change the connection's terminal do change the questions,
and the first round missed one of them:

- a security-key touch notice is written straight to standard error only when that stream is a terminal
  (`notify_start`, `readpass.c:252` and `:253`);
  otherwise it is sent to the askpass program with `SSH_ASKPASS_PROMPT=none`
  (`readpass.c:279`).
  An option that takes standard error away from OpenSSH therefore loses the touch notice,
  and an option that keeps it prints the notice itself.
  Source only: no security key was available to probe this.

Quiet mode suppresses OpenSSH's own lines, and it is worth naming exactly which,
because it decides four of the options here.
`-q` sets `options.log_level = SYSLOG_LEVEL_QUIET` (`ssh.c:927` to `:929`),
and `sshlogv` returns before anything is written when the level is above it
(`log.c:461`, `if (nlog_verbose == 0 && level > log_level) return;`).
`SYSLOG_LEVEL_QUIET` is the first enumerator (`log.h:41`), so even `fatal` is suppressed;
the `_f` variants only add the function name to the text (`log.h:120` to `:126`).
Measured with `-q` and with `LogLevel=QUIET`:
`Host key verification failed.` disappears,
Git's own `fatal:` line stays,
and a program's own write to standard error stays
(`keep/run4.log`, arms `b-quiet`, `j-quiet`, `c-quiet`, `e-quiet`, `c2-quiet`).

## What Git does around it

Read from Git 2.56.1 (`git --version`),
the version the wrapper shadows here.
The files were fetched from `raw.githubusercontent.com/git/git/v2.56.1/<file>`.

`git_prompt` and `do_askpass`,
`prompt.c:12` to `:77`:

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

Measured in the container in the first round,
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
- when the variant is not known,
  Git runs the command once more with `-G <host>` to detect it
  (`connect.c:1426` to `:1439`);
  measured with a counting ssh: 1 start for a plain `git ls-remote`, 2 when the ssh is named in a variable
  (`keep/run4.log`);
- protocol version 2 adds `-o SendEnv=GIT_PROTOCOL` and the variable itself
  (`connect.c:1342` to `:1348`);
- Git waits for the `ssh` child and,
  when it got no answer at all,
  prints
  `fatal: Could not read from remote repository.`
  with the two advice lines
  (`connect.c:61` to `:75`,
  `finish_connect`,
  `connect.c:1559` to `:1567`);
- that child's standard error is Git's own,
  which is why OpenSSH's line appears before Git's.

The exit status a person's shell sees,
measured (`keep/run4.log`, arms `status-notfound`, `status-refused`, `status-dns`;
`keep/run5.log`, arm `status-notfound-known-host`):

- a new host key: 128;
- a key nobody could unlock: 128;
- a connection refused: 128;
- a host name that does not resolve: 128;
- a repository that is not there, host key known: 128;
- `git pull` in a repository that exists: 1, with the same lines as 128 gives elsewhere.

That list is why C2's exit-status condition cannot tell the refusals apart.

## What the recorded rules leave open

- "One program only,
  ever"
  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`):
  no second shipped executable,
  so an askpass program of the wrapper's own is out.
  The record adds:
  "A later wish for another shipped program, such as an askpass helper, meets this answer first."
- the wrapper reads the name it was started under for the ZED hook and nothing else
  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`),
  and `SPEC.md:19136` says "the executable does not read the name it was started under",
  so an askpass mode cannot be selected by a link name.
  The review is right that the ZED record's reason (ZED cannot add an environment variable) does not carry over:
  the overlay can add one, and the record gives no other reason against it.
  An environment value as a selector is therefore open,
  and B is the option that needs one.
- no command the wrapper runs or forwards asks a person for a credential
  (`SPEC.md:8490` onward),
  so no option may make OpenSSH's question reach a terminal.
- no option may answer "yes" on a person's behalf.
  That is this round's brief.
  The project's recorded answers do not settle it by themselves:
  `SPEC.md:8577` says a host key is not a credential the push lacked,
  and the owner's answer of 2026-11-20 is that the service keeps retrying and pushes once the key is known
  (`SPEC.md:8479` onward),
  which only has a point if the machine never makes itself know the key.
  The rule stands, and it rules out `StrictHostKeyChecking=accept-new`,
  a pre-added key,
  and an askpass program that prints `yes`.
- the wrapper never starts itself as Git
  (`SPEC.md:14072` onward),
  and its source guards refuse `current_exe`,
  `/proc/self/exe`,
  `argv[0]` dispatch,
  shell text and a start from its own path
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`,
  rules `own-executable`, `argv0-dispatch`, `shell-text`, `spawn-own-executable`).
  B and E need a start of the wrapper by another program;
  nothing in them starts the wrapper as Git,
  and `own-executable`'s existing exception in `invocation/real_git.rs:587` already carries the wrapper's own path for
  identification
  ("identification only: this path is excluded from the search for the real Git so the wrapper never selects itself;
  nothing is started from it").
  B's selector reads that path for identification in the same sense;
  E needs no such reading but does need shell text.
- E cannot take over a person's `GIT_SSH_COMMAND`, `core.sshCommand` or `GIT_SSH` without either losing that
  person's options or writing shell text, which the `shell-text` rule forbids.
  The review checked `core.sshCommand` with `StrictHostKeyChecking=yes` as a way to force the setting;
  it would override the person,
  because `ssh_config(5)` reads configuration from command-line options first,
  then the user's file,
  then the system file,
  and "for each configuration directive, the first specified value will be used".

## The options

Each option says what it starts,
what it reads,
whether it can answer yes,
what a person sees for each case from a run,
what it costs on a command that meets no question (against a null arm in the same rounds),
and its pros and cons.
The ranking and its adjacent-pair reasons are in "The ranking".

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

A person sees the two transcripts in "What the person sees today": OpenSSH's line, Git's fatal line, exit 128,
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
fixed after the review showed the shape test missing real questions.
A start is the mode when both hold:

- **the environment names this executable.**
  `SSH_ASKPASS` or `GIT_ASKPASS` holds the path of the running executable,
  compared as paths after canonicalising,
  with the file name compared when the remembered path no longer exists.
  The overlay writes that value,
  and only a process started from the overlay's own Git can still carry it,
  its nested Git starts and OpenSSH's askpass start alike.
  The design keeps `GIT_ASKPASS` empty and reads it as well,
  so a later decision to cover Git's own credential questions needs no second selector.
- **exactly one argument, and it cannot be a Git invocation.**
  The argument does not start with `-`,
  and it is not a bare word:
  a bare word uses only `A` to `Z`, `a` to `z`, `0` to `9` and `-`.
  Or OpenSSH set `SSH_ASKPASS_PROMPT`, which it does for a permission question and a notification.

Why that shape holds and the first round's did not:

- every question measured here holds a space:
  `Are you sure you want to continue connecting (yes/no/[fingerprint])? `,
  `Warning: the ED25519 host key for ... differs ...`,
  `Enter passphrase for key '/root/.ssh/id_probe': ` and
  `Allow shared connection to 127.0.0.1? `
  (`keep/run4.log`, `keep/run5.log`).
  The first round's rule wanted a trailing `: ` or `? `,
  which the review showed `Confirm user presence for key %s %s` (`sshconnect2.c:1261`) and
  `Add key %s (%s) to agent?` (`sshconnect.c:1794`) would not have,
  and a keyboard-interactive prompt from the server need not have either
  (`sshconnect2.c:2125` to `:2130`).
- no valid one-argument Git invocation gets past that shape:
  with one argument, the argument is the subcommand,
  and every subcommand is a bare word
  (`status`, `stash`, `fetch`),
  while a multi-word invocation has more than one argument
  and every option word starts with `-`.
  So a nested `git status` or `git stash` that a forwarded command starts is never the mode,
  which is the hazard the shape has to keep out.
  An alias whose name holds punctuation (`git !f`) would be taken for the mode;
  that is bounded, and the command it breaks was a one-word alias call.
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
- the mode flushes standard error before it exits,
  because a log line was lost at exit on 2026-11-14 (`main.rs`, the note on `write_what_waits`).

The zero-exit hazard this pins is measured:
an askpass that exits zero with nothing on standard output is read as yes by `ask_permission`,
and a multiplexed session is then allowed (`keep/run6.log`).

What a person sees,
from the run with a stand-in that implements exactly the contract in "B",
inside a container whose ssh no AppArmor profile confines
(`keep/run4.log`; the stand-in is `keep/probe-scripts/bin/askpass-b`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stops before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
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
...
```

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (a passphrase key first, then a key that works)
Cloning into '/work/clone'...
monochromatic-git-policy-cli: this command needed the passphrase of the key `/root/.ssh/id_probe`, and no command the wrapper runs asks a person for one, so that key was skipped. Start an agent with `eval "$(ssh-agent)"` (`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /root/.ssh/id_probe` before the next command, use a key without a passphrase, or have the server accept another key you hold.
```

The last one ends 0 and the clone lands (`keep/run4.log`, arm `b-passkey-then-good`).
That is B's cost on the asked-then-succeeded case: the sentence appears on a command that works,
because the question arrives before anyone knows the outcome.
The wording is true in both cases,
which the earlier wording was not.

What it does **not** cover,
each measured:

- a changed host key: no question, so nothing (`b-changed-key`);
- `BatchMode=yes`: no question, so nothing (`b-batchmode`);
- `StrictHostKeyChecking=yes`: OpenSSH's own strict sentence and nothing from B (`b-strict-yes`);
- a keyboard-interactive prompt that is itself a bare word (`Password`) and carries no `SSH_ASKPASS_PROMPT`:
  that is the one askpass start the shape still misses;
  it falls through to Git,
  which prints `git: '<prompt>' is not a git command` and exits 1,
  and OpenSSH still gets no answer,
  so the no-prompt rule holds and only the sentence is lost.
  Closing it needs a signal no composition provides, such as the parent process,
  which the start name's rule and the guards' `argv0-dispatch` reading put out of reach;
- a stale remembered own path in a long-running service (`landing/service_route.rs:63`):
  the environment's value no longer equals the running executable,
  so the start falls through and the person sees that Git line instead of the sentence.

No passphrase reaches the mode:
OpenSSH passes the question in the argument and reads the answer from standard output
(`readpass.c:77`, `:82` to `:94`),
so the mode never sees a secret, and its argument is no more exposed than the `false` it replaces.

What it starts: the wrapper, started by OpenSSH, once per refused question.
What it reads: its argument, two environment values, and its own path for identification; no repository, no Git.
Can it answer yes: no. It writes nothing to standard output and exits 2,
and `confirm` reads only a recognised answer as yes (`sshconnect.c:616` to `:622`).

Cost on a command that meets no question: nothing measured against the null arm that names the same ssh
(+0.84 ms median, range -13.83 to +8.53, which is inside the null's own spread;
`keep/run4.log`, arm `askpass_only`).
The cost on a refused question is one askpass start, and this round could not resolve it:
over seven rounds of a refused clone the shell stand-in's paired difference from the same clone with `false` was
+3.6 ms median, range -12.2 to +10.2 (`keep/run4.log`, the question arms),
which its own spread swallows.
For scale, the wrapper's start with one question-shaped argument, on today's fall-through path,
measured 22.26 ms over `true` (range 13.65 to 45.58, `wrapper_arg` against `true_plain`),
and the designed mode starts before the forwarding machinery, so its start is at most that.

Pros:

- the only option whose words name the host,
  the address,
  the port and the key,
  and the only one that can tell a new key from a key that differs for the address,
  because it sees the question itself;
- the only option that needs nothing on the path of a command that meets no question;
- it changes nothing about Git's process,
  streams,
  signals or exit status,
  and takes none of the person's transport settings;
- it survives `-q` and `LogLevel=QUIET`, where OpenSSH prints nothing at all
  (measured, `b-quiet` and `b-loglevel-quiet`);
- it keeps the security-key touch notice at a terminal, because OpenSSH still owns standard error;
- it gives the commit service's own pushes the same words,
  and its record already folds a repeated host-key sentence
  (`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`,
  `src/native/lfs/lfs_pre_push.rs:617`).

Cons:

- on this machine OpenSSH may not run it:
  `git` confines `ssh` in its own child profile,
  which allows executing only `ksshaskpass` and VS Code's script,
  and today's `false` works only because a local rule names it.
  The exact rules are in "What building B would take".
  Without them B prints `ssh_askpass: exec(<path>): Permission denied` and no words,
  which is worse than A. Measured in the first round, unchanged by this one.
- it introduces a new way for the wrapper to be started,
  with its own selector rule and its own controls,
  and the guards that ban a start from the wrapper's own path need the reasoned reading in
  "What building B would take".
- the sentence must never reach standard output,
  the mode must never read standard input,
  and every branch must exit non-zero:
  three invariants a control must hold,
  and the first round's wording of the last one was not enough.
- it sees questions of every kind,
  including the agent's key-confirmation,
  the multiplexing question and keyboard-interactive prompts,
  which get the generic sentence and nothing specific.
- a start whose question the shape test misses falls through to Git and prints a `git: ... is not a git command` line
  instead of a sentence.
- the sentence appears on a command that then succeeds, when a passphrase key is skipped for one that works.
- on this machine, the words need a root edit of two AppArmor files per build.

### J. The sentence itself as `SSH_ASKPASS`'s value

Mechanism: `SSH_ASKPASS` holds the sentence, not a program.
`execlp` tries to execute a file named the sentence, fails,
and OpenSSH prints its own failure line with the value in it
(`fatal_f("exec(%s): %s", askpass, strerror(errno))`, `readpass.c:78`),
which is the line the first round saw when confinement denied a program
(`ssh_askpass: exec(<path>): Permission denied`).
No program, no mode, no selector, no AppArmor rule, no process.

Measured (`keep/run4.log`, arms `j-newkey`, `j-passkey-only`, `j-passkey-then-good`, `j-quiet`, `j-loglevel-quiet`,
`j-batchmode`, `j-strict-yes`).
The value holds 380 bytes, so the failure is `File name too long` rather than `No such file or directory`:

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
ssh_askpass: exec(monochromatic-git-policy-cli: OpenSSH asked a question only a person can answer, and no command the wrapper runs asks one, so the answer was no. A new host key's question is answered by connecting once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks; a key's passphrase is answered by starting an agent and loading the key with `ssh-add`.): File name too long
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

What it starts: nothing (a failed `exec`).
What it reads: nothing.
Can it answer yes: no; the exec never happens, so no program can say anything.

The measured limits, each one a finding against it:

- **one text for every question.**
  The same sentence arrives for a new host key and for a passphrase,
  so it must cover both, and it cannot name the host or the key
  (`j-newkey`, `j-passkey-only` give the same line).
- **OpenSSH's framing.**
  The person reads the advice inside `ssh_askpass: exec(...): File name too long`,
  and the words are 380 bytes long in that position.
- **it prints on a command that then succeeds**:
  the clone with a passphrase key followed by a working key exits 0 with that line (`j-passkey-then-good`).
- **it disappears under quiet**:
  with `-q` and with `LogLevel=QUIET` the whole line vanishes,
  and only Git's `fatal:` remains (`j-quiet`, `j-loglevel-quiet`).
  That is measured, and it is the opposite of what the review's note guessed for the exec line:
  `sshlogv` returns before `do_log` for every level under `QUIET` (`log.c:461`).
- it asks nothing, so `BatchMode=yes` and `StrictHostKeyChecking=yes` print nothing, as for B
  (`j-batchmode`, `j-strict-yes`).

Cost on a command that meets no question: nothing; the value is one more environment string and no process,
and no arm can measure it better than the askpass-only arm's 0.84 ms inside the null's spread.

Pros: no code, no mode, no selector, no AppArmor rule, no process;
it speaks to both refusals at the moment they happen;
and it is the only option that needs nothing on any host, confined or not.

Cons: the framing,
the single generic text,
the false appearance on a command that succeeds,
and silence under `-q`,
which B does not have.
It also cannot be turned into more precise words without becoming B.

### C. The wrapper keeps the child and reads its standard error

Mechanism: a typed network command no longer replaces the process;
the wrapper starts Git as its child,
relays standard error through a pipe while Git writes it,
then appends the sentence when the transcript reads as one of the refusals
(`reads_as_unknown_host_key`,
`src/native/lfs/lfs_pre_push.rs:523`,
and `reads_as_a_login_failure`,
`:409`).
The detector deliberately leaves a changed key and a strict-checking refusal to OpenSSH's own words,
and it reads the differing-key question of `CheckHostIP` as an unknown key
(`SPEC.md:8583` onward, `:8602` onward),
so C covers the same two refusals as B plus that rare third one,
and it says the partly wrong thing for it:
the sentence's way to connect once is right and its way to add the key is not.

What a person sees,
from the stand-in that forks Git and relays its standard error
(`keep/run4.log`, arms `c-newkey`, `c-passkey-only`, `c-known`, `c-quiet`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stops before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
```

The words are B's,
after Git's last lines instead of between Git's first line and OpenSSH's,
and they can name the remote's host because the wrapper resolved the remote.

What it starts: Git as a child, one pipe, one relay thread or loop.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

Cost on a command that meets no question,
against the null arm that names the same ssh, in the same rounds:
+2.16 ms median, range -8.69 to +30.02 (`relay_git`;
`keep/run4.log`).
The range is wider than the median, so under the project's noise rule this is not a finding:
C's relay costs about one extra process start, which a null arm measured at +1.11 ms median, range -0.75 to +2.30
(`relay_true` against `true_plain`), and this run cannot resolve better than that.
On a successful run the stand-in's standard output,
standard error and exit status were byte-identical to the plain run in the first round,
and this round kept the same stand-in.

Pros: gives both cases words at the same place as the existing push sentences do,
names the host,
keeps every setting the person and the repository own,
and is the shape the wrapper already uses where it owns a child
(`run_real_git`,
`src/native/invocation/forwarding.rs:216`).

Cons, recounted, because the first round counted them as one loss and one of them is worse than it said:

- **the live progress display disappears.**
  Git draws progress only when its standard error is a terminal,
  and under C that stream is the relay's pipe.
  Measured at a pseudo-terminal on both sides (`keep/run5.log`):
  Git's standard error on the terminal gives a `Receiving objects` line and the `remote:` counting lines,
  the relay gives none of them, and `--progress` brings them back.
  Passing `--progress` is not free:
  it would mean adding a word to the caller's arguments,
  and the wrapper's own rule is that Git receives only the caller's arguments, in order (`main.rs:28` to `:34`).
- **more than progress depends on that terminal.**
  Git checks `isatty(2)` for the sideband's progress (`sideband.c:316`),
  for the transport's (`transport.c:1230`, `:1358`),
  and for advice such as the editor's waiting line (`editor.c:70`);
  the first round named only progress.
- **the security-key notice is lost.**
  OpenSSH writes the touch notice itself only when its standard error is a terminal,
  and otherwise sends it to the askpass program, which is `false` here (`readpass.c:252`, `:279`).
  Under C that stream is the relay's pipe, so a person with a FIDO key would see no notice.
  Source only; no key was available.
- **a signal sent to the wrapper alone no longer reaches Git.**
  Measured again for both C and C2: after `kill -TERM` of the wrapper the child is still running
  (`keep/run4.log`, "a signal aimed at the wrapper alone").
  A terminal's control character signals the whole foreground group, so this shows for a signal aimed at one process,
  but the exec path gives it for free today.
  The two losses have different causes: the pipe takes the terminal, keeping the process takes the signal.
- **it is blind under `-q`**:
  OpenSSH's line is suppressed, so the detector has nothing to read and no sentence appears
  (measured, `c-quiet`).
- the wrapper must now own what the exec gave it:
  the exit status,
  the signal ending (`end_as_git_ended`),
  the wait,
  and the ordering of its own notices against Git's output.

### C2. Git kept as a child with the same streams, read after exit 128

Mechanism: the wrapper no longer replaces the process,
but it does not take standard error either.
It starts Git as a child with the same standard input,
standard output,
standard error and terminal,
waits,
and when the child ended 128 runs the same known-hosts check D does
and prints a sentence.
Nothing is piped, no argument is added, no text is read.

What a person sees,
from the stand-in that forks Git with the same streams and acts on exit 128
(`keep/run4.log`, arms `c2-newkey`, `c2-passkey-only`, `c2-notfound`, `c2-refused`, `c2-known`, `c2-quiet`;
`keep/run5.log`, arm `c2-notfound-known-host`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, ...
```

The words arrive after the person has read the failure,
and the same shape shows its disqualifying problem at the same time.
Exit 128 does not mean "the host key" or "a passphrase";
measured, it also means connection refused, a name that does not resolve, and a repository that is not there.
So C2 either cries wolf or says nothing:

- with the exit status alone: a missing repository and a refused connection get a paragraph about keys,
  the both-branch one when the stand-in was given that text (`c2-notfound`, `c2-refused`),
  and the host-key sentence when the check says the key is unknown (`c2-notfound-known-host`);
- with the post-failure check: a missing repository whose host key is known still gets the passphrase sentence,
  because the check knows the key and nothing more (`c2-notfound-known-host`).

The only case where C2 can be both accurate and useful is the undefined one:
it cannot distinguish the two refusals without reading text, and reading text is C.

What it starts: Git as a child; on a failure, `ssh -G` and `ssh-keygen -F` as well.
What it reads: the exit status and, on 128, the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost on a command that meets no question,
against the same-transport null: +2.84 ms median, range -12.12 to +29.35 (`forkwait_git`).
Inside the spread, so not a finding;
on a success it is one fork and one wait, with the same streams.

Pros: keeps the terminal,
the progress display,
the signals from the terminal,
and every setting the person owns,
and adds no word to the caller's arguments;
it survives `-q`, because it reads a status and not a line (`c2-quiet`).
Cons: it cannot tell which refusal it was, which is measured above;
the wrapper still owns the wait,
the exit status and the signal ending,
as C does;
and the paragraph has to be long enough to cover both refusals, which is what makes it cry wolf.

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

The first round read only `userknownhostsfile`, and the review's finding is reproduced here:
with the key only in the global list, the real command exits 0 while the check says "no key"
(`keep/run4.log`, arms `d-known-in-global-file`, exit 0, no sentence;
`d-old-read-user-file-only`, exit 0 **with** the sentence).
Reading both lists fixes it, and the same round measured the rest.

What a person sees
(`keep/run4.log`, arms `d-newkey`, `d-passkey-only`, `d-known-in-global-file`, `d-old-read-user-file-only`,
`d-connection-refused`):

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
(`d-connection-refused`).
It is also wrong when the person's setting already accepts unknown keys:
`StrictHostKeyChecking=no` or `accept-new` means the command will not stop at the key,
so the design must read that setting from `ssh -G` and print nothing then.
The first round did not read it.

What it starts: `ssh -G` and `ssh-keygen -F` before the command.
What it reads: the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost on a command that meets no question,
against the same-transport null: +28.97 ms median,
range +8.23 to +41.83 (`precheck_git`),
and every round of the eighteen was positive,
so this one is a finding rather than noise:
it is 2.57 of the wrapper's own added cost.
Its parts were measured separately over ten rounds each:
`ssh -G` 3 to 7 ms, and `ssh-keygen -F` 2 to 3 ms with one 13 ms reading
(`keep/run6.log`).

Two costs the first round did not name, both measured or read:

- **`ssh -G` runs the person's `Match exec` commands.**
  With a `Match host 127.0.0.1 exec ...` line in the configuration,
  one plain `git ls-remote` ran it once and D's check plus the ssh it then runs ran it twice
  (`keep/run4.log`, the `Match exec` lines).
  So D would run the person's configuration commands before every network command,
  and Git's own ssh runs them again.
- **`ssh -G` can resolve names.**
  `CanonicalizeHostname` makes `ssh -G` do DNS work, which was not run here.
  Source: `ssh_config(5)`, "CanonicalizeHostname".
  The output can also carry `ProxyCommand` and `SetEnv` values, which must never be logged.

Other limits, unchanged:

- it is blind to a `HostName` alias whose key lives on a jump host,
  and to any known-hosts file a `ProxyCommand` or `ProxyJump` makes the far side read;
- it cannot name the key a passphrase was needed for, and it never sees that case;
- a changed host key looks known to it (`ssh-keygen -F` finds a key under the name),
  so it says nothing about the one refusal whose advice needs the offending line.

Its degenerate variant prints the sentence before every network command over SSH with no check at all:
no process,
no cost,
and words that cry wolf on every successful clone and fetch.

Pros: no change to how Git runs,
no new mode,
no policy change,
and the words arrive before the connection,
so a person can stop and fix the key first;
the check reads the same files OpenSSH reads, and the fix for the global list makes it accurate.
Cons: one of the two cases is out of reach,
the words can be wrong about the failure,
the cost is paid by every network command,
and the check runs the person's own configuration commands one extra time.

### E. The wrapper becomes the ssh that Git starts

Mechanism: the overlay names the wrapper in `GIT_SSH_COMMAND` (or `GIT_SSH`),
so the wrapper starts the real `ssh`,
relays its standard error,
and appends the sentence when the words read as one of the two refusals,
while Git keeps the terminal and the process stays replaced.

What a person sees,
from the stand-in
(`keep/run4.log`, arms `e-newkey`, `e-passkey-only`, `e-known`, `e-quiet`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone'...
Host key verification failed.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stops before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The live progress display survives,
because Git's standard error is still the terminal
(measured at a pseudo-terminal, `keep/run5.log`).

Cost on a command that meets no question,
against the same-transport null: +14.68 ms median, range -1.66 to +20.09 (`ssh_mode`).
This is 1.30 of the wrapper's own added cost.
The first round reported 1.10 from a control that did not carry the same options;
this figure comes from the null arm that names the same ssh and the same options in the same rounds,
which is the arm the first round reported separately as `git-ssh-var`.

What it starts: the wrapper's stand-in as the ssh,
and under it the real `ssh` twice:
once for Git's variant probe, because Git does not know a named command's variant
(`connect.c:1426` to `:1439`),
and once for the connection.
Measured with a counting ssh: 2 ssh starts both for plain git with a named ssh and for E's shape,
so the probe is not what E adds;
E adds one relay per ssh start, two of them here.
What it reads: the `ssh` child's standard error.
Can it answer yes: no.

Pros: covers both cases,
keeps the terminal,
the progress,
the signals and the exit statuses,
and costs the person nothing they own on a command that meets no question.

Cons, recounted:

- it takes over `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`,
  which belong to the person and to the repository.
  Replacing them loses their options;
  wrapping them means building a shell command out of a shell command,
  which the `shell-text` rule forbids
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`).
- it must find the real `ssh` the way `real_git.rs` finds Git,
  including the refusal to run back into itself,
  and it must keep working when `ssh` is a wrapper of the person's own.
- it needs a selector of its own, which the first round never designed;
  the only one left is B's, because the start name is forbidden.
- it is blind under `-q` and `LogLevel=QUIET`, because it reads lines OpenSSH then does not print
  (measured, `e-quiet`).
- **the security-key notice is lost**:
  ssh's standard error is the relay's pipe, so `notify_start` sends the touch notice to the askpass program,
  which is `false`, instead of writing it (source, `readpass.c:252`, `:279`).
- its words come from the same detectors C uses,
  so they miss the differing-key question and anything else only the question text holds.

### F. Automatic trust, and other ways to answer for the person

`StrictHostKeyChecking=accept-new`,
a pre-added key,
or an askpass program that prints `yes` would all make the commands succeed,
and all of them answer a yes/no question on a person's behalf,
which this round's brief forbids,
and which the owner's answer of 2026-11-20 also rules out by having the service wait for a key the machine learns
some other way (`SPEC.md:8479` onward).
They are listed so the reading is complete,
and they are not ranked.

### G. Relax the no-prompt rule for typed commands

Clearing `SSH_ASKPASS_REQUIRE` for a command a person typed,
or setting it to `never`,
would hand OpenSSH's question back to the terminal.
The owner's rule covers every command the wrapper runs or forwards
(`SPEC.md:8490` onward,
which supersedes the earlier answer that a typed command keeps Git's prompting),
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

The ranking is **B > C > J > D > C2 > E > A**.

What changed from the first round's B > D > C > E > A:

- J and C2 are new and enter the order;
- C now ranks ahead of D,
  because D's gap (no passphrase case) and its measured per-command cost and false warnings
  now outweigh C's progress loss,
  which this round measured at a terminal and could count;
- J sits between them because it speaks to both refusals at the moment they happen for nothing on any host;
- E now ranks behind C2 because C2 keeps the progress, the terminal and the person's settings,
  while E takes the transport settings away, needs shell text and a selector, and loses more.

Reasons for each adjacent pair:

- **B over C.**
  B's words arrive at the refusal and name the host, the address, the port and the key;
  C's arrive after Git's advice lines and can name the host but not the key.
  B changes nothing about Git's process and costs nothing on a command with no question;
  C costs the live progress display on every clone and fetch (measured: the progress lines through the pipe are
  none, against a `Receiving objects` line and the `remote:` lines at a terminal),
  takes the security-key notice away, and needs a signal and exit-status owner.
  B's price is a mode, a selector and, on this host, two local AppArmor rules;
  C's price is paid by every successful clone forever.
  C covers one refusal more than B, the differing key for the address,
  and there its "add the host key" advice is the wrong half of the sentence.
- **C over J.**
  Both speak to the passphrase case.
  C's words are plain, name the host, and choose the sentence for the refusal it read;
  J's words sit inside `ssh_askpass: exec(...): File name too long`, are one generic text for every question,
  and also appear on a command that then succeeds.
  Both vanish under `-q` (measured for both).
  J asks nothing of any host, which is its whole advantage,
  and it is not enough against words a person can read without decoding OpenSSH's framing.
- **J over D.**
  J speaks to both refusals at the moment they happen and adds nothing to any command or any host.
  D speaks only to the host-key case, warns before the command rather than at the refusal,
  can be wrong about why the command failed,
  and charges about 29 ms on every network command (2.57 of the wrapper's own added cost, sign-consistent over
  eighteen rounds), plus one extra run of the person's `Match exec` commands.
  D's advantage is clean words in a clean place for one case;
  J's is coverage, timing and zero cost for both.
- **D over C2.**
  D's host-key information is accurate after the global-list fix and arrives before the failure,
  which is the moment a person can still stop.
  C2's is accurate only after the failure and only for the host-key case,
  and its other branch prints on failures that have nothing to do with keys (measured: a repository that is not there,
  a refused connection).
  D pays on every command;
  C2 pays nothing on success and survives `-q`, which is why the gap is small.
- **C2 over E.**
  C2 keeps Git's streams, the terminal, the progress and the signals from the terminal,
  and takes nothing the person owns.
  E takes `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`,
  needs shell text to keep a person's own values (forbidden), needs a selector,
  loses the security-key notice, and costs 14.68 ms on every network command against C2's 2.84 ms.
  E does arrive between OpenSSH's line and Git's, which is the better place;
  it is not worth the ownership.
- **E over A.**
  E's cost and its takeover are real,
  but it gives both refusals words without taking Git's process,
  and it keeps the progress display.
  A leaves the person with no next step at first contact with every new host,
  which is the moment they are stopped.

If the owner will not have a new mode in the wrapper,
J is the option that needs no mode and no host change,
and D is the one that needs no mode but does need code in the forwarding path.
If the owner will have a mode but not an AppArmor rule on this machine,
C is the option that needs no rule.

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
and the connect form built from that address with no login name:

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the wrapper runs asks whether to trust a new one, so the connection stops before any login. Connect once with plain `ssh -p 2222 127.0.0.1`, which the wrapper does not touch and which still asks, or add that machine's host key to `known_hosts`, then run this command again.
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

The first round's version said "the line the error above names",
which was false:
for an askpass start the offending line exists only inside the question,
and OpenSSH prints only its last line on standard error (`sshconnect.c:1429` to `:1447`).
This question is also rare, because `CheckHostIP` defaults to off.

Any other question the mode sees gets one generic sentence,
because the text may come from the server and must not be echoed into a sentence
(a keyboard-interactive prompt is written by the server, `sshconnect2.c:2125` to `:2130`):

```text
monochromatic-git-policy-cli: OpenSSH asked a question only a person can answer, and no command the wrapper runs asks one, so the answer was no. A new host key's question is answered by connecting once with plain `ssh <host>`, which the wrapper does not touch and which still asks; a key's passphrase is answered by starting an agent and loading the key with `ssh-add`.
```

No text from a question reaches a sentence unescaped:
the host and the key path are printed only after every non-printable byte is dropped,
and the server's own prompt text is never printed at all.

## What building B would take

- `no_prompt_overlay` names the wrapper's own path in `SSH_ASKPASS`
  and keeps `SSH_ASKPASS_REQUIRE=force`
  (`src/native/invocation/child_environment.rs:559`).
  `GIT_ASKPASS` stays empty.
  Covering Git's own login questions too would cost one
  `error: unable to read askpass response from '<path>'` line per failed question (measured in the first round),
  and it would put the mode in a path where the failing host is known but the message is not one of the two refusals.
  That is a separate decision and not part of this one.
- a mode at the very top of `main`,
  before the service check,
  the panic hook and any planning
  (`src/native/main.rs:43` onward),
  selected by the two signals in "B",
  printing one sentence and exiting 2 on every branch.
- the sentences in one module beside `lfs_pre_push.rs`,
  with the address and the key filled from the question.
- the two detectors learn to skip a line that starts with the wrapper's own prefix
  (`monochromatic-git-policy-cli:`),
  because the mode's sentence now reaches the transcripts they read:
  a typed push's LFS upload runs under the same overlay,
  so an upload that meets a host-key question carries the mode's sentence,
  and `reads_as_unknown_host_key` would otherwise see a line naming a host key that is not OpenSSH's own
  and stop reading the refusal as an unknown key.
  The service's children carry the same overlay, and its push record reads the transcript the same way.
  The first round claimed the service's pushes would get the same words and never checked what its record does with
  them;
  with the prefix skipped, the record folds the refusal as it does today.
  Controls: one that an upload whose transcript carries the mode's sentence still prints the upload's own sentence, and
  one that the record still folds a refusal once.
- controls:
  - a new-host-key control and a passphrase control asserting the sentence, the exit status and today's lines;
  - a control that a start whose environment names the wrapper and whose one argument is a question prints and exits 2,
    and one that a nested one-argument `git status` inside a forwarded command still runs Git;
  - a control that a changed host key and `StrictHostKeyChecking=yes` and `BatchMode=yes` print nothing;
  - a control that standard output is empty and standard input unread;
  - a control that every branch, including a panic, exits non-zero;
  - a control that a command which meets no question starts no askpass program.
- a reading of the source guards' `own-executable` rule,
  with an exception entry beside `real_git.rs`'s,
  and a line in `package/git-policy/cli/README.md` saying that a confinement which denies executing the wrapper leaves
  OpenSSH's own `ssh_askpass: exec(...)` line instead of the sentence.
- the AppArmor rules this machine needs,
  which are larger than "one local rule" and which the review was right to size:
  the ssh that Git starts runs under a stack,
  measured as `git//&git//ssh (enforce)`
  (`gate-review-typed-network-command-next-step-2026-11-24/work/keep/probe-apparmor-labels.txt`),
  and an exec is allowed only when **both** stacked profiles allow it.
  So the same path needs a rule in **both** local files:
  - `/etc/apparmor.d/local/git_ssh`, which today holds `@{bin}/false rix` and `owner /work/** rwk`
    (read, write and lock, no execute), needs
    `<the wrapper's own path> rix,`
    and, for the controls, the paths the gate and the host tests start the wrapper from.
  - `/etc/apparmor.d/local/git`, which today holds `owner /work/** mix`,
    `owner @{HOME}/.cargo/build/** mix` and one exact build path,
    needs the same line whenever the wrapper's path is not the profile's own attachment
    (`{,usr/}bin/git*`).
  - On this development host the wrapper's path carries a build identity
    (`@{HOME}/Monochromatic/.cache/wrapper-built-by-cargo-from-main-<commit>-<date>/monochromatic-git-policy-cli`),
    so the rule is per build;
    a released install adds it once for the installed path.
  - The gate's `/work` path and the host tests' `@{HOME}/.cargo/build/**` path would need an execute grant in
    `git_ssh` that is directory-wide, which is broader than a per-build rule.
    Without it, B's controls fail in the gate and the sentence is replaced by
    `ssh_askpass: exec(<path>): Permission denied`.
  - A person's own `ssh` (for example one a hook starts) runs under `/etc/apparmor.d/ssh`,
    which allows only `@{lib}/ssh-askpass/x11-ssh-askpass`,
    so B shows no words there.
  - Nothing else is needed: the mode reads its environment, its argument and its own path,
    and `current_exe()` is already what the wrapper does under the `git` profile for identification
    (`src/native/invocation/real_git.rs:587`).
    That the read is allowed under the stack is an inference from the existing start, not a probe.

## What the probes did not show

- **No run of the wrapper's own askpass mode.**
  No wrapper code was changed this round,
  so the mode's contract was implemented as a stand-in in the container
  (`keep/probe-scripts/bin/askpass-b`, which reads the question, prints the sentence for its shape, writes nothing on
  standard output and exits 2),
  and the capture shows what OpenSSH does with that contract.
  The real wrapper was also run as the askpass program:
  it started, took the question for a Git subcommand, printed
  `git: 'The authenticity of host ...' is not a git command.`,
  wrote nothing on standard output, and ended 128 because the clone failed anyway
  (`keep/run4.log`, arm `b-today-wrapper-as-askpass`).
  That is the fall-through the selector has to prevent,
  and it is not the mode.
- **No AppArmor profile was added** (that needs root).
  The denials and the rules needed are reported instead,
  and the live run is unconfined by design, because a confined ssh denies every askpass program but `false`,
  `ksshaskpass` and VS Code's script.
- **No agent-based hazard probe**: `ssh-agent` cannot bind its socket in the container
  (`unix_listener: cannot bind to path /work/out/agent.sock: Permission denied`, `keep/run5.log`),
  so the exit-zero hazard was probed through a multiplexed master instead, which worked.
- **No security key**: the touch notice is source only.
- **No `HostKeyAlias`, `ProxyJump` or `CanonicalizeHostname` arm**, where D's and E's readings are weakest.
- **No Windows build**, and no OpenSSH release other than 10.6p2 or Git release other than 2.56.1.
- **No terminal at this machine's own confined `git`**: the container's arms are unconfined on purpose.
- **No byte-for-byte comparison** of the wrapper against plain Git in this round:
  the failure and its exit status were reproduced, the comparison is the first round's.
- **No whole container gate, and no `native:check`, clippy, `lint:types` or host tests**:
  the only repository change is documentation, and the round's note says no whole gate is a merge condition.

## Measurements

Container `typed-network-next-step-probe-2026-11-24` of the gate image
(`localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85`),
`--cpus=2 --memory=2g --pids-limit=400 --blkio-weight=10`,
started in a `systemd-run --user` scope with `RuntimeMaxSec`, `CPUQuota=200%`, `MemoryMax=6G`, `TasksMax=1024`
and `IOWeight=10`.
OpenSSH 10.6p2, Git 2.56.1, this round's wrapper build (dev profile, `target/debug`), `sshd` on 127.0.0.1:2222.
An arm whose ssh authenticated ended 0 unless its purpose was a refusal.

Method, so a difference can be told from the host's noise:

- eighteen measured rounds after two warm-up rounds;
- in each round every arm ran once, in an order rotated by the round number,
  so no arm kept the same predecessor (the first round's criticism of fixed neighbours);
- the figure for an arm is the median of its per-round difference from a null arm run in the same round,
  with the range of those eighteen differences beside it;
- two null arms are reported:
  `git` with the same ssh named in `GIT_SSH_COMMAND` and the same identity options (the same work, no option), and
  a relay around `true` against `true` (one extra process start and nothing else);
- under the project's noise rule, a median inside its own range of differences is called noise, not a finding.

The baseline, `git` with the same ssh and options, took 135.72 ms median (123.29 to 144.95).
Against it, in those same rounds:

- today's wrapper (exec): **+11.26 ms** (1.50 to 16.77), which is 1.00 by definition;
- B, on a command that meets no question: **+0.84 ms** (-13.83 to 8.53), 0.07 of the wrapper's own cost;
- C's relay: **+2.16 ms** (-8.69 to 30.02), 0.19;
- C2's fork and wait: **+2.84 ms** (-12.12 to 29.35), 0.25;
- D's check before the command: **+28.97 ms** (8.23 to 41.83), 2.57;
- E's relay around ssh: **+14.68 ms** (-1.66 to 20.09), 1.30;
- one extra process start alone: **+1.11 ms** (-0.75 to 2.30).

What that supports and what it does not:

- D's cost is the only one whose eighteen differences all keep their sign,
  so D costing more is a finding; its size, about 29 ms here, is host-specific.
- B's, C's, C2's and E's medians sit inside their own spreads,
  so this round cannot say that any of them costs a measurable amount on a command that meets no question.
  The one thing it can say is the shape of each: B adds no process, C and C2 add one child, D adds two processes
  before the command, E adds two relays.
- The first round's figures (wrapper +7.21 ms, C +0.86 ms, D 6.62 ms, E +9.07 ms over a control)
  were computed without a null arm and against a different baseline;
  this round's replace them.

Two facts about the baseline that a reader should not misread:

- The arm with no identity options, plain `git ls-remote`, took 68.26 ms (60.51 to 74.49),
  and it was **refused**: without `-i` no key is offered, so it measured a failed connection, not a baseline.
  Ten rounds of the same arm in a second container show the same split: a refused connection 56 to 62 ms,
  an authenticated one 116 to 124 ms (`keep/run7.log`, `keep/run8.log`).
- A command named in `GIT_SSH_COMMAND` makes Git start it once more with `-G` before the connection
  (`connect.c:1426` to `:1439`);
  with a counting ssh, one plain `git ls-remote` started ssh once and the same command with the ssh named in
  `GIT_SSH_COMMAND` started it twice (`keep/run4.log`, `keep/run6.log`).
  Every option is measured against an arm with that same cost already in it.

The refused-question arms, seven rounds of a clone that meets a new host key:

- with `SSH_ASKPASS=false` (today), 73.5 ms median;
- with the mode stand-in, 77.4 ms median;
- paired difference +3.6 ms, range -12.2 to +10.2, which is noise.
  A start of the wrapper itself with one question-shaped argument, on its fall-through path, measured +22.26 ms over
  `true` (13.65 to 45.58), which is an upper bound for the mode's start and not the mode's cost.

Progress counts at a pseudo-terminal, one clone of a two-thousand-object repository (`keep/run5.log`):

- Git's standard error on the terminal, or under C2, E or today's wrapper:
  a `Receiving objects` line, a `remote: Counting objects` line and a `remote: Enumerating objects` line;
- through C's pipe: none of the three;
- through C's pipe with `--progress`: all three again.

Counts and outcomes, not timings, and they carry no spread.

## References

- `package/git-policy/cli/src/native/invocation/child_environment.rs` (`no_prompt_overlay`, `invocation_overlay`).
- `package/git-policy/cli/src/native/invocation/forwarding.rs` (`run_real_git`, `replace_process_with_real_git`).
- `package/git-policy/cli/src/native/lfs/lfs_pre_push.rs` (the two sentences and the three detectors).
- `package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs`
  (the four rules and the exceptions).
- `package/git-policy/cli/SPEC.md`, "no command the wrapper runs or forwards asks a person for a credential",
  "The wrapper never starts itself as Git", and the push record's words for an unknown host key.
- `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
- `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- `doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`.
- OpenSSH `V_10_6_P2`: `readpass.c`, `sshconnect.c`, `sshconnect2.c`, `ssh.c`, `log.c`, `log.h`.
- Git 2.56.1: `prompt.c`, `connect.c`, `sideband.c`, `transport.c`, `editor.c`, `strbuf.c`.
- `keep/run4.log`, `keep/run5.log`, `keep/run6.log`, `keep/run7.log`, `keep/run8.log`,
  and `keep/probe-scripts/` in this round's folder.
