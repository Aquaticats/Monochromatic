# Planning: agent-initiated context compaction

Status:
 design sketch only.
 Blocked on unnamed concerns the user has not yet
articulated.
 Do not implement until those concerns are surfaced and resolved.

Date of sketch:
 2026-05-11.

## The problem

Claude Code's `/compact` is owned by the human user (manual trigger) and the
harness (automatic trigger on context pressure).
 The assistant has no
documented mechanism to initiate compaction:

- `Skill` tool excludes `/compact` from its built-in allowlist.
   `skills.md`
  exposes `/init`,
   `/review`,
   `/security-review`;
   `/compact` is explicitly
  not exposed.
- Hook events `PreCompact` / `PostCompact` are response events,
   not
  triggers.
   `SessionStart` with matcher `"compact"` fires after compaction.
- MCP exposes tools but provides no IPC channel back into the host's
  command processor.
- No documented stdin injection,
   socket,
   or signal interface.
- Open feature request `anthropics/claude-code#52002`
  ("Allow programmatic/agent-initiated compaction"),
   marked duplicate of an
  unidentified canonical issue.

## Approaches considered

### Candidate: compaction by handoff via spawn-claude

Rather than triggering `/compact` on the current session,
 spawn a fresh
sibling whose starting context is a compressed brief.
 The parent is
abandoned for task work and kept open only for reference and as a
spawn-point for any branching task that diverges from the main line.

Mechanism:

1.  New CLI tool,
     "context compressor",
     likely under
    `package/dev-script/context-compressor/`.
     Reads the active transcript
    from `~/.claude/projects/.../sessions/<id>.jsonl`,
     hands it to a
    compressor model,
     prints a compressed brief on stdout.
2.  Claude invokes `compressor <session-path> | spawn-claude --prompt -`
    (exact stdin contract TBD;
     the existing `spawn-claude` accepts a prompt
    string,
     so piping may need a small wrapper or a new flag).
3.  `spawn-claude` launches a fresh Claude Code child in its own terminal
    window with the compressed brief as initial context.
     That child becomes
    the active session.
4.  The parent's run ends its current turn and does no further task work.
    It remains open in its terminal window solely for:
     (a) the user reading
    back the original conversation,
     (b) spawning additional children for
    branching tasks that share the same setup.

Strengths:

- No upstream Anthropic change required.
- Reuses existing `spawn-claude` infrastructure.
- Clean separation:
   handoff is explicit and visible (a new window
  appears),
   no hidden TTY manipulation.
- Branching is natural:
   the same parent can spawn multiple children for
  divergent follow-ups without re-deriving setup context.

Weaknesses:

- Not literal `/compact`;
   an observer sees two windows,
   not one shrunk
  session.
- Compressor model has a token cost on every handoff.
- Anything that lives only in the parent's tool-use state (open editors,
  uncommitted edits,
   mid-stream subprocess) does not transfer through a
  prose brief.
- Parent context never shrinks.
   Acceptable under the stated design
  (parent is abandoned),
   but means the parent cannot itself perform
  further handoffs once its own context fills.

### Alternative: hook plus terminal injection

A `Stop` or `PostToolUse` hook scans the assistant transcript for a
sentinel and shells out to inject `/compact\n` into the host's TTY.

Concrete injection mechanisms:

- `tmux send-keys -t "$TMUX_PANE" "/compact" Enter`,
   when claude-code runs
  under tmux.
   Cleanest variant,
   no privileges,
   only works in tmux.
- `xdotool type --window "$WID" "/compact"` followed by `xdotool key
  Return`,
   on X11.
   Does not work on Wayland.
- `kitty @ send-text --match "id:$KITTY_WINDOW_ID" '/compact\n'`,
   via the
  kitty terminal's remote-control socket.
- `ioctl(TIOCSTI)` write to the controlling TTY.
   Blocked on modern Linux
  kernels with `CONFIG_LEGACY_TIOCSTI=n` (Fedora 39 and later,
   current
  Debian,
   current Ubuntu).

Strengths:

- Triggers the literal `/compact` on the active session.
- Parent context shrinks in place;
   no second window,
   no second session id.

Weaknesses:

- Portability:
   each injection mechanism is bound to a specific terminal
  multiplexer,
   display server,
   or kernel config.
   No single mechanism
  covers every environment the user runs.
- Detection responsibility lives in the hook;
   the sentinel-to-action loop
  is a side channel that can fire spuriously if the sentinel string ever
  appears legitimately in output.
- Race conditions if the assistant emits the sentinel before its turn
  finishes streaming.

### Alternative: PTY wrapper

Launch claude-code under a PTY layer (`expect`,
 `script`,
 `socat`,
 or a
custom Node or Python wrapper).
 The wrapper watches the PTY's stdout
stream for a sentinel and writes `/compact\n` to the PTY master.

Strengths:

- Portable across terminal emulators and display servers;
   the wrapper
  owns the trigger and the terminal is transparent to it.
- Sentinel detection happens at the same layer that owns the input
  channel,
   removing the cross-process race.

Weaknesses:

- Changes the user's startup flow:
   claude-code is no longer launched
  directly but through the wrapper.
- The wrapper has to faithfully proxy resize signals,
   ANSI escape
  sequences,
   mouse events,
   bracketed paste,
   and any other interactive
  protocols the CLI uses.
   Subtle bugs are likely.
- Adds a new process to the path between the user and the CLI,
   with its
  own crash and resource modes.

### Alternative: exec self-replacement via resume

A hook collects a hand-crafted summary,
 then `exec claude --resume <id>`
with the summary injected via a `SessionStart` hook's `additionalContext`.

Strengths:

- Effective context shrink achieved without invoking `/compact`.
- Session id is preserved across the resume,
   so external references to
  the session id remain valid.

Weaknesses:

- Not literal `/compact`;
   resume semantics differ from compaction
  semantics in ways that may matter (transcript shape,
   tool history
  presentation).
- `exec` replaces the parent process.
   Any in-flight tool calls or open
  file descriptors that have not yet been flushed to disk are lost.
- The summary content depends entirely on what the hook computes,
  inheriting the same compressor-quality concerns as the candidate
  approach,
   with no help from the harness's built-in compaction logic.

### Alternative: SIGUSR1 to the running claude-code process

`kill -USR1 <claude-pid>` from Bash,
 if the CLI happens to install a
user-signal handler that triggers compaction.

Status:
 unverified.
 Not documented.
 Would require either `strace -e signal`
against a running instance or grepping the distributed binary to determine
whether any such handler exists.
 Until that check is performed,
 this is
speculation.
 Listed for completeness so that future readers know it was
considered and not yet ruled out by evidence (only by absence of
documentation).

## Components, if the candidate is pursued

- Compressor CLI package with a choice of compressor model (Morph,
   smaller
  Claude,
   local LLM) configurable per invocation.
- Spec for the stdin contract between compressor and `spawn-claude`,
  including how the child distinguishes "I am a compaction continuation"
  from "I am a delegated subtask".
- Operational note added to `AGENTS.md` once the mechanism exists,
   so future
  sessions know when reaching for it is appropriate and when it is not.
- Discovery story:
   how the running parent finds its own session JSONL
  path.
   Claude Code exposes session id in some contexts;
   needs
  verification.
- Convention for what the parent does after issuing the handoff so the
  user sees a clear signal that this window is now reference-only.

## Unresolved concerns

The user expressed a bad feeling about the candidate design but did not
name the specific concerns in the session that produced this sketch.
Concerns are unnamed and unresolved.
 Do not proceed to implementation
until they are surfaced.

Possible categories worth probing when the user is ready to articulate
them.
 These are speculative;
 they are not the user's stated reasons.

- Semantic drift between the compressor model's brief and the actual
  transcript content.
   A wrong-but-confident summary is worse than no
  summary.
- Loss of in-flight tool-use state (open file handles,
   partial diffs,
  mid-refactor staging) that does not survive serialization to a prose
  brief.
- Token cost of the compressor model on every invocation versus the cost
  of letting the harness's automatic compaction run.
- Audit trail and reproducibility when work spans N spawned descendants
  rather than one linear conversation;
   the parent transcript stops
  recording the real work as soon as the handoff happens.
- Window proliferation:
   a long-running task that compacts several times
  leaves a trail of abandoned windows.
   UX cost to the user.
- The "branching task" justification for keeping the parent open relies
  on the parent's setup context being reusable.
   If the unnamed concerns
  include a belief that this reusability does not hold,
   the candidate's
  main differentiator collapses.

These are guesses,
 not the user's reasons.
 The user's reasons remain to
be named.
