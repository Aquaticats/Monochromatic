# What Claude can do when it doesn't give up

This document records sessions where Claude went above and beyond --
persisting through cascading failures,
 discovering root causes through genuine investigation,
and arriving at solutions a human would have reasonably abandoned before finding.

The purpose is practical:
 someone working with Claude can reference these to understand
what kinds of problems Claude handles well when given room to iterate,
and what collaboration patterns lead to these outcomes.

---

## Windows VM template baking with SATA-to-VirtIO transition

**Session:
** `a9f3a03f` | **Commit:
** `105f33a`

**The ask:
** Add Windows VM support to `mvm`,
 a KVM/libvirt VM management CLI that only supported Linux cloud images.

**What made this hard:
** Windows Server on KVM is notoriously painful.
The session hit 8 distinct problems across the full stack --
download infrastructure,
 disk sizing,
 Windows SAN policy,
 driver signing,
 guest agent plumbing,
boot verification,
 process management,
 and metadata ordering.
Each fix exposed the next problem.
 The session ran out of context twice and was resumed.

### The SAN policy wall (6 failed approaches before the breakthrough)

The central obstacle was Windows Server 2025's SAN policy 4,
 which marks VirtIO disks as "offline shared bus.
"
The installer refuses to write to offline disks.
 Claude tried:

1. `RunSynchronous` with inline diskpart;
    failed from XML entity encoding corruption (`0x80070057`)
2. `Microsoft-Windows-PartitionManager` with `SanPolicy=1`:
    component ignored in WinPE pass
3. Batch script on the autounattend ISO;
    worked when run manually from WinPE debug console (verified by sending `Shift+F10` keystrokes via `virsh send-key`),
    but `RunSynchronous` never executed it
4. Inline `cmd /c` with XML entities;
    same `0x80070057` error
5. `diskpart /s` with script file;
    same failure
6. UEFI boot;
    stuck on "Press any key to boot from CD" prompt that can't be automated

**The breakthrough:
** Rather than continuing to fight WinPE's `RunSynchronous`,
Claude recognized the entire approach was wrong and proposed a fundamentally different architecture:
install on SATA disk (which Windows accepts without SAN policy issues),
install VirtIO drivers during first boot,
 switch the template VM to VirtIO bus,
verify it boots,
 then capture.
 This completely sidestepped the SAN policy.

### Other problems solved in the same session

- **Download truncation** (7.7 MB instead of 5.6 GB):
   unawaited `writer.write()` dropping data in Bun's streaming API.
  Replaced with `Bun.write()` atomic writes plus parallel file-size polling for progress.
- **VirtIO driver signing dialogs**:
   `pnputil /add-driver` triggered GUI consent prompts.
  Fixed by importing Red Hat certificates into TrustedPublisher store before driver installation.
- **Guest agent not connecting**:
   `vioserial` driver wasn't installed,
   cutting the communication channel.
- **Background process dying after 2 minutes**:
   bash tool's parent shell sending SIGHUP.
  Fixed with `setsid nohup` to properly detach.
- **Metadata ordering bug**:
   `setWindowsHostname` called `exec` which needed `meta.json`,
  but metadata was written after hostname setting.

### Why a human would have stopped

Most people would have given up after the third or fourth SAN policy attempt.
The WinPE `RunSynchronous` debugging alone involved sending keystrokes to a VM console
to verify scripts manually;
 that's the kind of tedious verification work
that makes you question whether the whole approach is viable.
Claude did that verification,
 confirmed the scripts were correct,
concluded that `RunSynchronous` itself was the problem,
 and pivoted to a new architecture.

---

## Bash output filter hook (7 iterations through undocumented sandbox constraints)

**Session:
** `46f7d940` | **Related:
** previous sessions with earlier versions

**The ask:
** Fix a `PreToolUse` hook that pipes Bash tool output through a text filter
to strip git noise,
 long lines,
 and repeated diagnostics.

**What made this hard:
** The Claude Code sandbox has undocumented behaviors
around how it evaluates bash commands.
Each fix revealed a new constraint that nobody had documented.

### The 7 versions

- Version:
  V1;
  Approach:
  `PIPESTATUS` via `;` separator;
  Failure:
  `_bof` gets non-numeric value;
  Sandbox constraint discovered:
  `;` splits into separate shell contexts
- Version:
  V2;
  Approach:
  `{ cmd; echo EC:$?; }` grouping;
  Failure:
  `bash: command not found: {`;
  Sandbox constraint discovered:
  `{` treated as command name
- Version:
  V3;
  Approach:
  Direct `$PIPESTATUS`;
  Failure:
  Same as V1;
  Sandbox constraint discovered:
  (repeat of constraint 1)
- Version:
  V4;
  Approach:
  `bash -c` wrapper;
  Failure:
  `!` escaped to `\!`;
  Sandbox constraint discovered:
  Extra shell layer corrupts quoting
- Version:
  V5;
  Approach:
  `set -o pipefail &&`;
  Failure:
  SIGPIPE exit 141;
  Sandbox constraint discovered:
  `< /dev/null` appended to last command
- Version:
  V6;
  Approach:
  `|| (exit $?)` suffix;
  Failure:
  `$?` is empty;
  Sandbox constraint discovered:
  Variables don't expand in suffix position
- Version:
  V7;
  Approach:
  `&& true` suffix;
  Failure:
  **Works**;
  Sandbox constraint discovered:
  ;

**The solution:
** `set -o pipefail && cmd 2>&1 | bun filter && true`.
No shell variables,
 no special syntax,
 no nested shells.
`&& true` absorbs the `< /dev/null` redirect.
Exit code propagation works through `&&` short-circuit semantics alone.

### The honest moment

When asked "how many more versions are we expecting?
",
Claude acknowledged they were playing whack-a-mole with an undocumented eval chain
and proposed the architecturally correct fix:
 rewrite as a `PostToolUse` hook
that filters output after execution.
When told Anthropic won't support `PostToolUse` output modification,
Claude accepted reality and made V7 bulletproof.

### Why a human would have stopped

Five undocumented sandbox constraints,
 each requiring a completely different approach.
By V4 most people would have said "the sandbox doesn't support this" and given up.
Claude kept reverse-engineering each constraint,
 documented all of them in a troubleshooting guide,
and found a solution that works within all constraints simultaneously.

---

## Patterns for working with Claude effectively

These sessions share common traits that made the outcomes possible:

**Let Claude investigate before proposing.
**
In both sessions,
 Claude read git history,
 checked existing code,
 and tested hypotheses
before committing to a fix.
 The user didn't rush Claude to a solution.

**Let Claude fail and iterate.
**
Neither session was "get it right the first time.
"
The user allowed Claude to try approaches,
 see them fail,
 understand why,
 and try again.
The value was in the accumulated understanding across iterations.

**Push back when Claude is going in circles.
**
The user asked "how many more versions?
" at the right moment,
prompting Claude to step back and consider whether the entire approach was wrong.
This led to the `PostToolUse` proposal in one session
and the SATA-to-VirtIO architecture in the other.

**Trust Claude with tedious verification.
**
Sending keystrokes to a VM console via `virsh send-key` to debug WinPE,
polling file sizes during downloads,
 reading through XML error codes --
this is the kind of work Claude does thoroughly without fatigue.

**Give Claude the full problem,
 not just the next step.
**
Both sessions started with a high-level goal,
 not step-by-step instructions.
This let Claude make architectural decisions
(like the SATA-to-VirtIO pivot) that a step-by-step approach wouldn't allow.
