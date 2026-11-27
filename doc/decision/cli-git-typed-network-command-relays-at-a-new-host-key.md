# A typed ssh network command relays Git's error stream and names the next step at a new host key

Status: decided by the owner on 2026-11-25T19:07-05:00, answering three questions put from the proposal
`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md` (seventh revision, `5c63224dbb`), with
its scoped gate read's corrections
(`~/temp/agent/gate-review-host-key-next-step-seventh-revision-2026-11-25/keep/review.md`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
Folded into the questions:

- "When a typed network command (clone, fetch, pull, push over ssh) meets a new host key, or a key no agent can unlock,
  how should the wrapper give the person the next step?" with "Relay (Recommended)".
- "Only if you chose Relay: what should Git's error stream be connected to?" with "K2: adapt (Recommended)".
- "Only if you chose Relay: should the wrapper read the remote first and relay only commands that ssh carries, or relay
  every typed network command?" with "Read first (Recommended)".

This record amends `doc/decision/cli-git-typed-network-command-next-step-at-a-new-host-key.md` and keeps
`doc/decision/cli-git-alpha-46-release-pass-four-answers.md` and `doc/planning/cli-git-no-self-invocation.md`.

## What was decided

- The wrapper reads a typed network command's remote URL first (`git remote get-url`, with `--push` for a push; a clone's
  URL from its own arguments). A command ssh does not carry keeps today's process replacement, bytes and signals.
- For a command ssh carries, the wrapper starts Git as its child and reads Git's error stream. When the transcript reads as
  OpenSSH's new-host-key refusal or a key no agent can unlock, it adds the sentence naming the `ssh` command to run.
- Git's error stream is a pseudo-terminal when the wrapper's own error stream is a terminal, and a pipe otherwise, chosen by
  one `isatty` call (K2).
- The child is published to the wrapper's existing signal relay (`process/signal_relay_relayed_child.rs`), so `SIGINT`,
  `SIGTERM`, `SIGHUP` and `SIGQUIT` sent to the typed process number reach Git. A terminal's control character reaches Git
  through the foreground group.

## Why

- Relay's words come from what OpenSSH printed: silent at a repository that is not there, speaking at a key nobody can
  unlock. It covers `clone`, `fetch`, `pull` and a non-LFS `push` (measured on stand-ins, not on the wrapper).
- K2 behaves as plain Git in both places: at a terminal it keeps the progress display, Git's colour and a security key's
  touch notice; in a capture its bytes are plain Git's (a clone's captured stream 37 bytes, a merged-stream `pull`
  245 bytes, both identical to plain Git's).
- Reading first leaves every command ssh does not carry exactly as it is today.

## Limits, stated with the decision

- `kill -9` of the typed process number ends the wrapper and leaves Git running, which process replacement never allowed.
- Under `ssh -q` or `LogLevel=QUIET` OpenSSH prints nothing to read, so no sentence is added.
- A typed LFS push needs the double-sentence fix the proposal names.
- The advised `ssh` command reads the person's plain ssh configuration; options set only in `GIT_SSH_COMMAND`,
  `core.sshCommand` or `GIT_SSH` can send the advised command to another known-hosts file (measured).
- The read of the remote costs about 0.85 ms per typed network command with a named remote; reading a clone URL's rewrite
  in process instead of starting Git is unmeasured.
- The coverage above is measured on stand-ins; the wrapper's own paths (a merging `pull` runs its fetch through
  `run_wrapper_own_git`) were not traced end to end.
- The `SPEC.md` passage saying a plain forwarded `git fetch` or `git push` replaces the wrapper, "so there is no \"after\"
  to look at", is amended for commands ssh carries.

## Options that were not chosen

- C2 (a child with untouched streams and a known-hosts check after exit 128), D (a check before Git starts), A (no change).
- K (always a pseudo-terminal) and C (always a pipe).
- Relaying every typed network command whatever the transport.
