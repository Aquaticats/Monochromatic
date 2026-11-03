# Restore Claude Code Shift+Enter in OdyTTY after an update

## Current-version check

On 2026-11-02,
Claude Code `2.1.317` in OdyTTY `0.17.5` passed a live input test without this patch:
Shift+Enter inserted a newline,
Enter submitted,
and Ctrl+C cleared input.
See [the current verification and its limits](../troubleshooting/claude-code-odytty-keyboard.md).
Do not apply the historical patch merely because the terminal-name allowlist lacks `odytty`.

## Historical failure and workaround

In OdyTTY,
 pressing **Shift+Enter** at the Claude Code prompt submits the message instead of inserting a newline,
and `/terminal-setup` refuses with `Terminal setup cannot be run from odytty.`.
Root cause:
 Claude Code sends the Kitty keyboard protocol enable sequence (`\x1b[>1u`) only when the terminal it
detects is on a hardcoded allowlist inside its binary;
 for unknown terminals detection returns `TERM_PROGRAM` verbatim (`odytty`),
 which is not on the list,
 so OdyTTY keeps encoding **Shift+Enter** as a plain carriage return.
Claude Code's input decoder parses the CSI-u sequence `\x1b[13;2u` (Shift+Enter) unconditionally,
and OdyTTY honors the protocol push
(its own shell integration emits `\x1b[>1u` around prompts),
so adding `"odytty"` to the allowlist is the whole fix.
This runbook patches the allowlist with `tweakcc`.

What this proves:
 after the procedure,
 **Shift+Enter** inserts a newline in the Claude Code input box,
 **Enter** still submits,
 and **Ctrl+C** still cancels
(the enabled mode is disambiguate-only,
 Kitty protocol flag 1).

Bridges tried,
 so this is not an unconsidered handoff:
 the patch itself is fully scriptable and an agent verified and applied it end to end,
 but every Claude Code auto-update replaces the patched binary,
 and the running session keeps pre-patch code in memory,
 so the operator must reapply and restart after updates.
Rejected alternatives:
 OdyTTY `keybinds` remap terminal-UI actions only and cannot send text to the PTY;
 OdyTTY `shell_key_enhancement = on` is prompt-scoped and popped before any command runs,
 so it never covers Claude Code;
 spoofing `TERM_PROGRAM=tmux` would enable the push but misidentifies the terminal for every other
 feature keyed off terminal detection.

Verified on Claude Code 2.1.215 (native binary install) with OdyTTY 0.9.1 (AppImage) via `tweakcc`,
2026-07-20.

## Setup

Status:
TODO

Prerequisites for a fresh machine:

- Linux with OdyTTY as the terminal.
  Confirm with `echo "$TERM_PROGRAM"`.
  Expected:
   `odytty`.
- Claude Code installed as a native binary.
  Confirm with `readlink --canonicalize "$(command -v claude)"`.
  Expected:
   a path ending `.local/share/claude/versions/<version>`.
- `tweakcc` available.
  Confirm with `tweakcc --version`.
  If absent,
   run it as `npx tweakcc` (Node.js required) or install it with `npm install --global tweakcc`.
- A shell outside any running Claude Code session
  (the patch lands on disk;
   running sessions are restarted in the steps).

## Steps

Status:
TODO

1.  Read the current allowlist from the installed binary.
    This step survives Claude Code updates that rename the minified variable or reorder the list.

    ```sh
    scratch="$(mktemp --directory)"
    tweakcc unpack "$scratch/cc.js" "$(readlink --canonicalize "$(command -v claude)")"
    grep --text --only-matching --extended-regexp \
      '\[("odytty",)?"iTerm.app","kitty"[^]]*\]' "$scratch/cc.js"
    ```

    Expected:
     exactly one line;
     on 2.1.215 it is
    `["iTerm.app","kitty","WezTerm","ghostty","tmux","windows-terminal","WarpTerminal"]`.
    If the line already starts with `["odytty",`,
     the patch is present;
     skip to What to check.
    If the list contents differ from the literal below,
     substitute the printed literal as the old string in step 2
     and the same literal with `"odytty",` prepended as the new string.

2.  Patch the allowlist in place:

    ```sh
    tweakcc adhoc-patch --confirm-possible-dangerous-patch \
      --string '["iTerm.app","kitty","WezTerm","ghostty","tmux","windows-terminal","WarpTerminal"]' \
               '["odytty","iTerm.app","kitty","WezTerm","ghostty","tmux","windows-terminal","WarpTerminal"]'
    ```

    Expected:
     `✓ Replaced 1 occurrence(s) in` followed by the versions path from Setup.

3.  Exit every running Claude Code session and relaunch `claude`
    (a running process keeps the pre-patch allowlist in memory).

    Expected:
     the new session starts normally.

4.  Type any text at the Claude Code prompt and press **Shift+Enter**.

    Expected:
     the cursor moves to a new line inside the input box and nothing is submitted.

## What to check

Status:
TODO

1.  Re-extract and confirm the patched list
    (the unpack must rerun because step 2 rewrote the binary):

    ```sh
    tweakcc unpack "$scratch/cc-patched.js" "$(readlink --canonicalize "$(command -v claude)")"
    grep --text --count '"odytty","iTerm.app"' "$scratch/cc-patched.js"
    ```

    Expected:
     `1`.

2.  In the relaunched session:
     **Shift+Enter** inserts a newline,
     **Enter** submits,
     **Ctrl+C** still cancels the current input.

3.  If **Shift+Enter** still submits after the restart,
     confirm the binary on `PATH` is the one patched
    (`readlink --canonicalize "$(command -v claude)"` matches the path step 2 reported)
    and that `echo "$TERM_PROGRAM"` still prints `odytty`
    (a multiplexer or SSH hop in between changes it).

## Restore

Status:
TODO

1.  To return Claude Code to its stock behavior,
     run `tweakcc --restore`.
    Caution:
     this restores the original binary wholesale,
     removing every tweakcc customization
     (themes and other patches included),
     not just this one;
     reapply the wanted ones afterwards with `tweakcc --apply`.

    Expected:
     after relaunching `claude`,
     **Shift+Enter** submits again.

2.  A Claude Code auto-update also replaces the binary and silently reverts to stock behavior;
    rerun Steps after each update to get **Shift+Enter** back.

3.  Remove the scratch extraction:

    ```sh
    rm -rf "$scratch"
    ```

    Expected:
     the directory is gone.
