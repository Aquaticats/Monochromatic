# The next step a typed network command names at a new host key or a missing passphrase

## Status

A proposal,
put to the owner.
It is not a decision,
and nothing here is approved or final.
It designs the words a typed `git clone`,
`fetch`,
`pull` or non-LFS `push` can print when OpenSSH refuses a new host key,
or a key whose passphrase nobody can be asked for,
and it ranks the ways the wrapper could do that under the project's recorded rules.
Each option that the recorded rules leave open was probed in this round's throwaway place,
and each probe's transcript is named below.
Nothing under `doc/decision/` or `doc/handover/` was changed by this round,
and no wrapper code was changed.

Round folder:
`~/temp/agent/typed-network-command-next-step-at-a-new-host-key-design-2026-11-24/`
(`keep/` holds the transcripts and timings this document cites).
Worktree:
`.claude/worktrees/typed-network-command-next-step-at-a-new-host-key-design`,
branch `docs/planning-cli-git-typed-network-command-next-step-at-a-new-host-key`.

## What the person sees today

Measured through this round's build of the wrapper (`0.6.1-alpha.45`) against a throwaway `sshd` on `127.0.0.1:2222`
in a rootless container of the gate image,
with a fixture host key,
a fresh `known_hosts` and throwaway client keys
(`keep/probe-transcripts.txt`,
`keep/probe-clone-transcripts.txt`,
`keep/probe-clone-passphrase-transcripts.txt`).

A new host key,
for `git clone`,
`git fetch`,
`git push` and `git ls-remote` alike,
ends this way at a terminal and without one:

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone-w'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

A key whose passphrase nobody can be asked for ends this way:

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git
Cloning into '/work/clone-p-w'...
root@127.0.0.1: Permission denied (publickey).
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

The same commands run against the same server by `git` itself,
without the wrapper,
print byte for byte the same lines in both cases.
The words that are missing are therefore the whole of what the no-prompt rule costs here,
and nothing else in the wrapper's route changes them.

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
  `SSH_ASKPASS_REQUIRE=force`
  (the owner's answer of 2026-11-19T16:58:00-05:00,
  recorded in `package/git-policy/cli/SPEC.md:8461` onward and in
  `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`).
  A typed command's streams are the caller's own,
  so the terminal is handed over and no part of the wrapper stays behind to read Git's standard error.
- the wrapper already has the two sentences this document is about,
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

The owner's question for this waits in
`doc/handover/questions-for-the-owner.md`,
section "Words for a typed command at an unknown host key or a missing passphrase (the pass's L4)".

## What OpenSSH does with an askpass program

Read from `V_10_6_P2`,
the release this machine runs (`ssh -V` prints `OpenSSH_10.6p2`).
The files are in the round's `scratch/src/`.

`ssh_askpass` runs the program,
in `readpass.c:47` to `:108`:

- the program's standard output is a pipe the program cannot see
  (`dup2(p[1], STDOUT_FILENO)`, `readpass.c:73`);
- one environment variable is added when the question is a permission question,
  `SSH_ASKPASS_PROMPT=confirm` (`readpass.c:75`, `:76`);
- nothing else of the environment changes:
  the program is a `fork` without an environment clean-up of any kind,
  so it keeps every variable `ssh` itself was given,
  including the four the wrapper puts there;
- the program is found on `PATH` and receives the question as its only argument:
  `execlp(askpass, askpass, msg, (char *)NULL);` (`readpass.c:77`);
- the parent reads at most 1023 bytes of standard output
  (`readpass.c:82` to `:92`);
- a program that did not exit zero,
  or that was killed,
  gives no answer at all: the function returns `NULL`
  (`readpass.c:99` to `:102`);
- everything from the first carriage return or line feed is dropped,
  so only the first line of standard output can be the answer
  (`readpass.c:104`);
- standard error is not redirected anywhere,
  so it reaches the person as the program writes it;
- standard input is inherited from `ssh`,
  which under Git is the pipe carrying the Git protocol,
  so a program that reads standard input can steal protocol bytes.

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

The host key question is `confirm`,
`sshconnect.c:603` to `:627`:
it loops on `read_passphrase(msg, RP_ECHO)`,
returns zero when that gives nothing
(`sshconnect.c:614`, `:615`),
and accepts only the empty string,
`no`,
`yes` or the fingerprint it was given as answers
(`sshconnect.c:616` to `:622`).
For a host this machine does not know,
`check_host_key` builds a question that carries the host,
the address,
the key type,
the fingerprint and,
for an address whose key differs,
a warning,
and ends with `Are you sure you want to continue connecting (yes/no/[fingerprint])? `
(`sshconnect.c:1211` to `:1246`).
A refused question ends the connection through
`fatal("Host key verification failed.")`
(`sshconnect2.c:104`).

A key's passphrase is `load_identity_file`,
`sshconnect2.c:1533` to `:1575`:
the question is `Enter passphrase for key '<path>': `
(`sshconnect2.c:1547`, `:1548`),
and an empty answer is read as one nobody gave,
so the key is skipped and the next one is tried
(`sshconnect2.c:1553` to `:1558`).
Password questions are built just before
(`sshconnect2.c:1057`,
`:1115`).

What follows for a program that would be the askpass program:

- it must write nothing to standard output except an answer,
  because the first line of standard output is taken as the answer,
  and a sentence written there is a wrong answer that can make `confirm` ask again
  (`sshconnect.c:612` loops,
  `:624` returns only on a recognised answer);
- it must exit non-zero,
  which is how it says "no answer" without writing anything;
- it sees the whole question,
  which is more than OpenSSH ever prints on standard error:
  under `StrictHostKeyChecking=ask` the question and its fingerprint exist only in that argument,
  and OpenSSH prints its own last line alone
  (`sshconnect.c:1207` to `:1254`);
- it must not read standard input.

Measured in the throwaway container (`keep/probe-transcripts.txt`):
today's value `false` gives no answer and the person sees `Host key verification failed.`;
a program the machine's confinement refuses to execute gives
`ssh_askpass: exec(<path>): Permission denied` and the same refusal after it,
which proves the child's standard error reaches the person
and that an exec failure is one more way to give no answer.

## What Git does around it

Read from Git 2.56.1 (`git --version`),
the version the wrapper shadows here.
The files are in the round's `scratch/src/git/`.

`git_prompt` and `do_askpass`,
`prompt.c:12` to `:77`:

- the program is chosen in this order and only a non-empty name counts:
  `GIT_ASKPASS`,
  then `core.askPass`,
  then `SSH_ASKPASS`
  (`prompt.c:53` to `:59`);
  an empty `GIT_ASKPASS` therefore stops the other two as well,
  which is what the wrapper relies on;
- the program receives the prompt as its only argument
  (`prompt.c:18`, `:19`) and its standard output is captured
  (`prompt.c:21`);
- Git reads at most 20 bytes of that output
  (`prompt.c:27`);
- a program that does not exit zero makes Git print
  `error: unable to read askpass response from '<program>'`
  and take no answer
  (`prompt.c:32` to `:39`);
- everything from the first line break is dropped (`prompt.c:41`);
- with no answer,
  `GIT_TERMINAL_PROMPT` decides between a terminal prompt and
  `could not read <prompt>terminal prompts disabled`
  (`prompt.c:62` to `:74`).

Measured in the container:
`GIT_ASKPASS=/work/bin/askpass` (a program that prints a sentence on standard error and exits non-zero) makes
`git credential fill` print the sentence,
then `error: unable to read askpass response from '/work/bin/askpass'`,
then `fatal: could not read Username for 'https://example.com': terminal prompts disabled`.
A `GIT_ASKPASS` naming a file that does not exist gives
`fatal: cannot exec '<path>': No such file or directory`.
So pointing Git's own helper at a program adds one line naming that program on every failed credential question,
where today's empty value adds none.

How Git reaches the server,
`connect.c`:

- `GIT_SSH_COMMAND` wins,
  then `core.sshCommand`,
  then `GIT_SSH` (no shell),
  then `ssh`
  (`connect.c:1182` to `:1193`,
  `:1409` to `:1424`);
- a command Git is given may be shell text,
  and the remote-side command is single-quoted into it
  (`connect.c:1501` to `:1509`),
  which is why the remote command survives as one argument;
- when the variant is not known,
  Git runs the command once more with `-G <host>` to detect it
  (`connect.c:1426` to `:1439`);
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

## What the recorded rules leave open

- "One program only,
  ever"
  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`):
  no second shipped executable,
  so the pass's first option (a small askpass program of its own) is out.
- the wrapper reads the name it was started under for the ZED hook and nothing else
  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`),
  so an askpass mode cannot be selected by a link name.
- no command the wrapper runs or forwards asks a person for a credential
  (`SPEC.md:8461` onward),
  so no option may make OpenSSH's question reach a terminal.
- no option may answer "yes" on a person's behalf,
  which the brief for this round states and which rules out
  `StrictHostKeyChecking=accept-new`,
  a pre-added key,
  and an askpass program that prints `yes`.
- the wrapper never starts itself as Git
  (`SPEC.md:14043` onward),
  and its source guards refuse `current_exe`,
  `/proc/self/exe`,
  `argv[0]` dispatch,
  shell text and a start from its own path
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`).
  Nothing in the options below starts the wrapper as Git,
  and the guard's `own-executable` exception in `invocation/real_git.rs` already carries the wrapper's own path for
  identification.

## The options

Each option says what it starts,
what it reads,
whether it can answer yes,
what a person sees for the two cases (from the probes),
what it costs on a command that meets no prompt,
and why it is ranked where it is.

### A. Leave it as recorded

What it starts: the real Git, replacing the wrapper, as today.
What it reads: nothing.
Can it answer yes: no.

A person sees the two transcripts in "What the person sees today": OpenSSH's line,
Git's fatal line,
exit 128,
and no next step.
The one way out is knowledge the words do not give: connect once with plain `ssh <host>`,
which the wrapper does not touch,
or add the key to `known_hosts`.

Cost: nothing.

Pros: no code,
no new mode,
no policy change,
exec semantics untouched,
and the refusal is exactly the owner's recorded rule.
Cons: the first contact with every new host ends without a next step,
and a key whose passphrase was never loaded ends the same way,
which is the case a person cannot even name from the words.
The pass over alpha.45 ranked it last,
and nothing in this round's probes improves it.

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
  and exits non-zero,
  which OpenSSH reads as no answer (`readpass.c:99` to `:102`).
- the argument tells it which question it is,
  so the sentence can name the host,
  the address,
  the port and,
  for a passphrase question,
  the key's path:
  `Are you sure you want to continue connecting` is a new host key,
  `Enter passphrase for key '<path>'` is a key nobody could unlock,
  `password: ` is a password,
  and the differing-key-for-the-address question carries its own warning text
  (`sshconnect.c:1444` to `:1447`).

How the wrapper knows it is that program,
since the start name is not allowed and OpenSSH passes no word of the wrapper's own:
two independent signals,
both already in the start's arguments and environment:

- the start's environment still names the wrapper in `SSH_ASKPASS` or `GIT_ASKPASS`
  (the overlay put it there,
  and only a process started from the overlay's own Git can still carry it,
  its hooks and OpenSSH's askpass start alike), and
- the start has exactly one argument,
  and that argument reads as a question:
  it holds a space and ends with `: ` or `? `,
  which no Git subcommand does.

The mode is entered at the very top of `main`,
beside the service's leading-word check
(`src/native/main.rs:43`, whose service word is read at `:56`),
before any planning,
any Git start and any record.
The alternative selectors are worse:
a variable alone collides with a hook that runs a one-argument `git` inside the forwarded command,
the question's shape alone is one signal where two are available
(a person who sets `SSH_ASKPASS` to the wrapper themselves then gets the sentence for any one-argument
question-shaped command, which is a bounded false positive and not a hazard),
and a generated helper program written into the runtime directory is a second program on disk,
which the one-program answer forbids in spirit and which the source guard's shell-text rule forbids in letter.

What a person would see,
from the two probes that stand the mode in with a program printing the same words
(`keep/probe-clone-transcripts.txt`,
`keep/probe-clone-passphrase-transcripts.txt`;
the mode itself could not be run on this machine, see "What the probes did not show"):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the
wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain
`ssh -p 2222 root@127.0.0.1`, which the wrapper does not touch and which still asks, or add the host key of that machine
to `known_hosts`, then run this command again.
Cloning into '/work/clone-w'...
Host key verification failed.
fatal: Could not read from remote repository.
...
```

The sentence comes first because OpenSSH runs the program while it is deciding,
before its own last line.

Cost on a command that meets no prompt: no process and no read.
The only difference from today is the value of `SSH_ASKPASS`,
which the overlay already writes.
The cost on a question that is refused is one start of the wrapper,
which the round measured at 6.01 ms for a start with a trivial command
(`keep/timings2.txt`),
paid once per refused question and never on the path of a command that meets none.

What it starts: the wrapper, started by OpenSSH, once per refused question.
What it reads: its argument and two environment values; no repository, no Git.
Can it answer yes: no. It writes nothing to standard output and exits non-zero,
and OpenSSH's `confirm` reads only a recognised answer as yes
(`sshconnect.c:616` to `:622`).

Pros:

- the only option whose words name the host,
  the address,
  the port and the key,
  and the only one that can tell a new key from a key that differs for the address,
  because it sees the question itself;
- the only option that needs nothing on the path of a command that meets no prompt;
- it changes nothing about Git's process,
  streams,
  signals or exit status,
  and takes none of the person's transport settings;
- it gives the commit service's own pushes the same words,
  and its record already folds a repeated host-key sentence
  (`HOST_KEY_NOT_KNOWN_SENTENCE_OPENING`,
  `src/native/lfs/lfs_pre_push.rs:617`).

Cons:

- on this machine OpenSSH may not run it:
  `git` confines `ssh` in its own child profile,
  which allows executing only `ksshaskpass` and VS Code's script
  (`/etc/apparmor.d/git:121`, `:122`),
  and today's `false` works only because
  `/etc/apparmor.d/local/git_ssh` adds `@{bin}/false rix` for the wrapper's own controls.
  Measured:
  `ssh_askpass: exec(/work/bin/askpass): Permission denied`
  (`keep/state-so-far.md`,
  "A denial I did not expect").
  Without one more local rule naming the installed wrapper,
  B would print that denial and no words,
  which is worse than A.
- it introduces a new way for the wrapper to be started,
  with its own selector rule and its own control,
  and the guards that ban a start from the wrapper's own path need a reasoned reading:
  nothing here starts it, OpenSSH does,
  but the exception must be written where the guard list lives.
- the sentence must never reach standard output,
  and the mode must never read standard input,
  or it changes OpenSSH's decision instead of explaining it;
  both are new invariants a control must hold.
- it sees questions of every kind,
  including the agent's key-confirmation and `ControlMaster` questions
  (`ask_permission`,
  `readpass.c:196` to `:221`),
  which have no sentence yet and would need one or silence.

### C. The wrapper keeps the child and reads its standard error

Mechanism: a typed network command no longer replaces the process;
the wrapper starts Git as its child,
relays standard error through a pipe while Git writes it,
then appends the sentence when the transcript reads as one of the two refusals
(`reads_as_unknown_host_key`,
`src/native/lfs/lfs_pre_push.rs:523`,
and `reads_as_a_login_failure`,
`:409`).

What a person sees,
from the stand-in that forks Git and relays its standard error
(`keep/probe-clone-transcripts.txt`,
`keep/probe-clone-passphrase-transcripts.txt`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone-c'...
Host key verification failed.
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, ... then run this
command again.
```

The words are the same as B's,
after Git's own lines instead of before them,
and they can name the remote's host because the wrapper resolved the remote,
but not the key's path for the passphrase case
(the login sentence names no key).

What it starts: Git as a child, one pipe, one relay thread or loop.
What it reads: Git's standard error as it arrives.
Can it answer yes: no; it only reads.

Cost on a command that meets no prompt,
measured as the median of per-round differences against plain `git` in the same rounds
(`keep/timings3.txt`,
eighteen rounds after two warm-up rounds,
arms interleaved in a rotating order):
+0.86 ms against the wrapper's +7.21 ms,
which is 0.12 of the wrapper's own added cost,
and the relay's own process start is 0.23 ms of it,
which the wrapper would not pay.
On a successful run the stand-in's standard output,
standard error and exit status were byte-identical to the plain run.

Pros: gives both cases words,
keeps every setting the person and the repository own,
and is the shape the wrapper already uses where it owns a child
(`run_real_git`,
`src/native/invocation/forwarding.rs:216`).

Cons, both measured, and both matter more than the cost:

- the live progress display disappears.
  Git draws progress only when its standard error is a terminal,
  so with the relay in between `git clone` printed no
  `Receiving objects:` lines at all and no `remote: Counting objects:` lines:
  102 of each with the terminal handed over and none through the relay
  (`keep/probe-progress-counts.txt`).
  A clone of a large repository would look frozen.
  Keeping it would need the relay to interpose a pseudo terminal,
  copy its window size,
  and pass the bytes through,
  which is more machinery and a new set of terminal behaviours to control.
- a signal sent to the wrapper alone no longer reaches Git.
  Measured: after `kill -TERM` of the relay,
  the child was still running
  (`keep/probe-precheck-false-positive-and-signal.txt`).
  A terminal's control character signals the whole foreground group,
  so this shows only for a signal aimed at one process,
  but it is a behaviour the exec path gives for free today.
- today the terminal is handed over and the wrapper's process is gone;
  C keeps that process alive and takes the terminal away from Git's standard error,
  and both losses above come from that one change.
- the wrapper must now own what the exec gave it:
  the exit status,
  the signal ending (`end_as_git_ended`),
  the wait,
  and the ordering of its own notices against Git's output.

### D. A check before Git starts

Mechanism: before replacing the process,
resolve what OpenSSH would use and ask whether this machine holds a key for it:
`ssh -G <host>` for the resolved host name,
port and `userknownhostsfile` values
(`ssh -G` connects to nothing),
then `ssh-keygen -F <host> -f <file>` over those files.
When no file holds a key,
print the sentence first,
then run the command as today.

What a person sees,
from the stand-in
(`keep/probe-clone-transcripts.txt`,
`keep/probe-clone-passphrase-transcripts.txt`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, ...
Cloning into '/work/clone-d'...
Host key verification failed.
fatal: Could not read from remote repository.
...
```

For the passphrase case it prints nothing at all:
the key is known,
so the check finds it,
and the person sees today's two lines and the passphrase is never mentioned.
That is D's gap,
measured,
and it is the case whose words the person most needs.

Worse, the sentence claims a failure that may not happen:
with nothing listening on the port,
the check still finds no key,
prints "the connection stopped before any login",
and then the real words are `ssh: connect to host 127.0.0.1 port 2299: Connection refused`
(`keep/probe-precheck-false-positive-and-signal.txt`).
Words printed before a failure is known have to be conditional,
which is weaker than a sentence that arrives with the refusal.

What it starts: `ssh -G` and `ssh-keygen -F` before the command.
What it reads: the resolved OpenSSH configuration and the known-hosts files.
Can it answer yes: no.

Cost on a command that meets no prompt,
measured the same way (`keep/timings3.txt`):
`ssh -G` 4.53 ms and `ssh-keygen -F` 2.09 ms,
so 6.62 ms on every network command,
which is 0.92 of the wrapper's own added cost:
the wrapper's overhead on each network command would almost double.
The check is also blind to a `HostName` alias whose key lives on a jump host,
and to any known-hosts file a `ProxyCommand` or `ProxyJump` makes the far side read.

Pros: no change to how Git runs,
no new mode,
no policy change,
and the words arrive before the connection,
so a person can stop and fix the key first.
Cons: one of the two cases is out of reach,
the words can be wrong about the failure,
and the cost is paid by every network command.

### E. The wrapper becomes the ssh that Git starts

Mechanism: the overlay names the wrapper in `GIT_SSH_COMMAND` (or `GIT_SSH`),
so the wrapper starts the real `ssh`,
relays its standard error,
and appends the sentence when the words read as one of the two refusals,
while Git keeps the terminal and the process stays replaced.

What a person sees,
from the stand-in
(`keep/probe-clone-transcripts.txt`,
`keep/probe-clone-passphrase-transcripts.txt`):

```text
# git clone ssh://root@127.0.0.1:2222/work/served.git          (new host key)
Cloning into '/work/clone-g'...
Host key verification failed.
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, ... then run this
command again.
fatal: Could not read from remote repository.
```

The live progress display survives,
because Git's standard error is still the terminal
(`Receiving objects:` 102 occurrences,
`keep/probe-progress-counts.txt`).

Cost on a command that meets no prompt,
measured the same way (`keep/timings3.txt`):
+9.07 ms against plain `git`,
and +7.92 ms against its own control arm that names `ssh` itself,
which is 1.10 of the wrapper's added cost;
part of that is Git's variant detection running the wrapper once more with `-G`
(`connect.c:1426` to `:1439`).

What it starts: the wrapper,
then the real `ssh`, and one more time for Git's variant detection.
What it reads: the `ssh` child's standard error.
Can it answer yes: no.

Pros: covers both cases,
keeps the terminal,
the progress,
the signals and the exit statuses,
and costs the person nothing they own.

Cons:

- it takes over `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`,
  which belong to the person and to the repository.
  Replacing them loses their options;
  wrapping them means building a shell command out of a shell command,
  which the source guard's `shell-text` rule forbids
  (`src/native/source_guard/no_self_invocation_rules_tests.rs`,
  rule `shell-text`).
- it must find the real `ssh` the way `real_git.rs` finds Git,
  including the refusal to run back into itself,
  and it must keep working when `ssh` is a wrapper of the person's own.
- its words come from the same detectors C uses,
  so they miss the differing-key-for-the-address question,
  which leaves only the last line on standard error
  (`SPEC.md:8573` onward),
  and they see nothing under `LogLevel=FATAL`.

### F. Automatic trust, and other ways to answer for the person

`StrictHostKeyChecking=accept-new`,
a pre-added key,
or an askpass program that prints `yes` would all make the commands succeed,
and all of them answer a yes/no question on a person's behalf,
which this round's brief forbids and which the owner's recorded rules about credentials also refuse.
They are listed so the reading is complete,
and they are not ranked.

### G. Relax the no-prompt rule for typed commands

Clearing `SSH_ASKPASS_REQUIRE` for a command a person typed,
or setting it to `never`,
would hand OpenSSH's question back to the terminal.
The owner's rule covers every command the wrapper runs or forwards
(`SPEC.md:8461` onward,
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

## The words

Two sentences,
both new,
verb-neutral,
and one more for the differing-key case.
They live beside the two that exist today
(`src/native/lfs/lfs_pre_push.rs`),
because the same detectors must not drift,
and the existing two stay for the LFS upload and the service's push record,
whose subjects are an upload and a push.

New host key,
with the connect form built from the remote's own user,
host and port:

```text
monochromatic-git-policy-cli: this machine does not know the SSH host key of `[127.0.0.1]:2222`, and no command the
wrapper runs asks whether to trust a new one, so the connection stopped before any login. Connect once with plain
`ssh -p 2222 root@127.0.0.1`, which the wrapper does not touch and which still asks, or add the host key of that machine
to `known_hosts`, then run this command again.
```

A key nobody could unlock,
with the key's path from the question,
and the two causes named as `login_needed_message` names them:

```text
monochromatic-git-policy-cli: this command needed the passphrase of the key `/root/.ssh/id_probe`, and no command the
wrapper runs asks a person for one, so the connection stopped at the key. Start an agent with `eval "$(ssh-agent)"`
(`eval (ssh-agent -c)` in fish) and load the key with `ssh-add /root/.ssh/id_probe`, use a key without a passphrase, or
have the server's account accept that key, then run this command again.
```

A key that differs for the address,
which today's push sentence deliberately leaves to OpenSSH's own warning:

```text
monochromatic-git-policy-cli: this machine holds a different SSH host key for the address of `[127.0.0.1]:2222` than
the server sent, and no command the wrapper runs asks whether to continue. Check the line the error above names in
`known_hosts` before going on, then run this command again.
```

The existing sentence says "then push again",
which is wrong for a `fetch` or a `clone`;
that is why a verb-neutral pair is proposed rather than reusing it.
For B the question's own text selects which sentence,
and the sentence needs no detector.
For C and E the detectors in `lfs_pre_push.rs` select it,
and the host comes from the remote the wrapper resolved.

## What the probes did not show

- No live run of B: the container's `git//ssh` profile allows only `false`, `ksshaskpass` and VS Code's script,
  so the wrapper could not be executed as the askpass program
  (`keep/state-so-far.md`,
  "A denial I did not expect").
  B's two words are shown through stand-ins that print the same text,
  and OpenSSH's side of the protocol is read from `readpass.c` with the line numbers above,
  of which only the exit-status half is measured live (`false` gives no answer).
  I did not add an AppArmor rule (that needs root) and did not move a program to a path the profile allows.
- No guest arm through `mvm`: another session's guest was running through the shared libvirt session daemon,
  which lives in the capped scope of whichever command started it.
- No exercise of a person's real `~/.ssh/config` with `HostName` aliases,
  `ProxyJump` or `Include`,
  which is where D's and E's readings are weakest.
- No push through the commit service (the container has no user manager);
  the pushes measured were Git's own.
- No Windows build,
  and no compare against OpenSSH releases other than 10.6p2 or Git releases other than 2.56.1.
- The askpass start's own cost is the wrapper's start with a trivial command (6.01 ms),
  not an askpass start,
  because the mode does not exist yet.

## Measurements

Container `cli-git-askpass-probe-2026-11-24` of the gate image,
`--cpus=2 --memory=2g --pids-limit=300`:
OpenSSH 10.6p2,
Git 2.56.1,
this round's wrapper build,
`sshd` on 127.0.0.1:2222,
`git ls-remote` of a local repository,
every arm authenticated and exiting zero.
Eighteen rounds after two warm-up rounds,
arms interleaved in a rotating order,
and the figure is the median of each arm's per-round difference from the plain `git` arm in the same round,
so drift between rounds cancels (`keep/timings3.txt`):

- plain `git ls-remote`: 123.42 ms median, 116.20 ms to 140.16 ms over the rounds;
- today's wrapper: +7.21 ms (range -1.08 ms to +17.12 ms),
  which is 1.00 by definition;
- C's stand-in: +0.86 ms (range -5.99 ms to +10.67 ms),
  0.12 of the wrapper's added cost;
- E's stand-in: +9.07 ms,
  1.10 of the wrapper's added cost against its own direct-`ssh` control;
- D's two processes: 4.53 ms and 2.09 ms,
  0.92 of the wrapper's added cost;
- B: no process and no read on this path,
  so nothing measurable beyond the environment value it changes,
  which is 0.00 of the wrapper's added cost;
  one wrapper start (6.01 ms measured) is paid per refused question.

The spread is wider than the smallest differences:
the wrapper's own added cost ranged from -1.08 ms to +17.12 ms across rounds on a host other sessions load,
so C's +0.86 ms and the wrapper's +7.21 ms are told apart by the median and by three repeated runs
(+7.41 ms,
+10.64 ms and +7.21 ms) rather than by any single round.
The progress counts and the signal behaviour are counts and outcomes,
not timings,
and carry no such spread.

## Ranking

**Ranking: B > D > C > E > A.**

- B over D: B's words arrive with the refusal,
  name the host and the key,
  cover the passphrase case,
  and cost nothing on a command that meets no prompt;
  D's arrive before the failure is known,
  can be wrong about it,
  cannot see the passphrase case at all,
  and cost 0.92 of the wrapper's own overhead on every network command.
  B's price is a new mode,
  a selector with two signals,
  and one local confinement rule on this machine;
  D's price is paid by every network command forever.
- D over C: D changes nothing about how Git runs,
  while C takes the live progress display away from every typed clone and fetch
  (102 lines to none)
  and lets a signal aimed at the wrapper leave Git running.
  C gives more words than D,
  which is why the gap is small,
  and C would overtake D if the relay interposed a pseudo terminal and kept the progress display.
- C over E: E keeps progress and needs no fork of Git,
  but it takes over `GIT_SSH_COMMAND`, `core.sshCommand` and `GIT_SSH`,
  and preserving a person's own value means writing shell text,
  which the wrapper's own guards forbid;
  C takes nothing the person owns.
- E over A: E's cost is real and its takeover is real,
  but it at least gives both refusals words without changing Git's process;
  A leaves the person with no next step at first contact with every new host.
- A last, for the reason the pass over alpha.45 gave:
  it costs the person at the moment they are stopped.

If the owner will not have one more mode in the wrapper,
D is the option that needs no mode,
and its words should then be written as a warning about what the command is about to do,
never as a claim about why it failed.

## What building B would take

- `no_prompt_overlay` names the wrapper's own path in `SSH_ASKPASS`
  and keeps `SSH_ASKPASS_REQUIRE=force`
  (`src/native/invocation/child_environment.rs:559`);
  `GIT_ASKPASS` stays empty unless the owner wants Git's own login prompts covered too,
  which costs one `error: unable to read askpass response from '<path>'` line per failed question.
- a mode at the top of `main`,
  selected by the environment's `SSH_ASKPASS` or `GIT_ASKPASS` naming this executable and by one question-shaped
  argument,
  that prints one of the sentences and exits 2.
- the sentences in one module beside `lfs_pre_push.rs`,
  with the connect form built from the user,
  host and port of the remote the command names.
- controls: a new-host-key control and a passphrase control that assert the sentence and exit 128 with today's lines
  still there;
  a control that a one-argument question-shaped start under `SSH_ASKPASS` prints and exits 2;
  a control that a hook or alias running `git status` inside a forwarded network command still runs Git;
  a control that standard output is empty and standard input unread;
  a control that a command that meets no prompt starts no askpass program.
- a reading of the source guards' `own-executable` and `spawn-own-executable` rules,
  with the exception written where the guard list lives,
  and a line in `package/git-policy/cli/README.md` saying that a confinement which denies executing the wrapper leaves
  OpenSSH's own `ssh_askpass: exec(...)` line instead of the sentence.
- one local rule on this machine naming the installed wrapper in the `git//ssh` profile,
  beside the `false` rule that is there today;
  without it this machine shows the denial and no words.

## References

- `package/git-policy/cli/src/native/invocation/child_environment.rs` (`no_prompt_overlay`, `invocation_overlay`).
- `package/git-policy/cli/src/native/invocation/forwarding.rs` (`run_real_git`, `replace_process_with_real_git`).
- `package/git-policy/cli/src/native/lfs/lfs_pre_push.rs` (the two sentences and the three detectors).
- `package/git-policy/cli/SPEC.md`, "no command the wrapper runs or forwards asks a person for a credential" and
  "The wrapper never starts itself as Git".
- `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
- `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- `doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`.
- OpenSSH `V_10_6_P2`, `readpass.c`, `sshconnect.c`, `sshconnect2.c`.
- Git 2.56.1, `prompt.c`, `connect.c`.
