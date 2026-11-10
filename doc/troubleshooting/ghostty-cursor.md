# Claude Code (Ink terminal UI) hides the real terminal cursor and renders an inverse-video block, overriding Ghostty's `cursor-style = bar`

## Symptom

Ghostty is configured with `cursor-style = bar`.
 The shell prompt
shows a bar cursor.
 Launching `claude` (Claude Code CLI) replaces
the cursor in the input area with a solid block:

1.  Set `cursor-style = bar` in `~/.config/ghostty/config`.
2.  Confirm the shell prompt shows a bar cursor.
3.  Launch `claude`.
4.  Observe a solid block at the `>` input prompt,
     regardless of
    Ghostty's configured cursor style.

On exiting Claude Code,
 the bar cursor returns to the shell prompt.

## Root cause

Claude Code's input area does not use the real terminal cursor at
all.
 The Ink terminal-UI framework (React-style rendering for TTYs)
hides the cursor on mount and renders a **visual** cursor by
inverse-video-printing a space character via Chalk's `inverse`
modifier.

Source locations in the extracted Claude Code bundle
(`claude-code-2.1.75.js`):

- The TextInput component sets a default cursor styler:
  `invert: f = f$.inverse` (Chalk's inverse modifier).
- When `showCursor && focus && terminalFocus` is true,
   the
  component renders `f(" ")`:
   an inverse-video space character
  positioned where the cursor would appear.
- When the component has focus,
   the cursor character uses
  `f$.inverse` unconditionally;
   there is no branch that emits a
  DECSCUSR cursor-style sequence.

On mount,
 Ink sends `\x1b[?25l` to hide the real terminal cursor.
The cursor stays hidden for the entire session,
 so Ghostty's
configured shape is moot:
 the user is looking at an inverse-video
block painted by the application,
 not the terminal cursor.

On exit,
 Ink sends `\x1b[?25h` to restore visibility,
 and Ghostty's
shell-integration `PS1` hook re-issues `\e[5 q` to restore the bar.

A separate,
 related finding (covered below) is in Ghostty's
terminfo,
 but it is independent of the Claude Code rendering choice.

## Verification

Version under test:

- Claude Code 2.1.75
- Ghostty 1.0+ with `cursor-style = bar`
- Bash or zsh with Ghostty shell integration enabled

Reproduce:
 set `cursor-style = bar` in Ghostty's config;
 verify the
shell prompt's bar cursor;
 launch `claude`;
 observe the block.

Inspecting the bundle for cursor-related sequences confirms the
finding:

- `claude-code-2.1.75.js` contains zero DECSCUSR sequences (`\e[N q`
  with any N).
- The only cursor-related escape sequences are visibility toggles:
  `\x1b[?25l` (hide) and `\x1b[?25h` (show).

Ghostty's side is behaving correctly:

- `stream_handler.zig:874-888`:
   `\e[0 q` handler restores
  `default_cursor_style` from config via `self.default_cursor_style`.
- `Config.zig:872`:
   `cursor-style = bar` maps to the `.bar` enum
  variant.
- `cursor.zig:4-15`:
   `.bar` is the bar shape with no lossy
  conversion.

The combination of "Claude Code hides the cursor" and "Claude Code
never emits a DECSCUSR" means no Ghostty setting can affect the
appearance,
 because the rendered glyph is application-layer,
 not
terminal-layer.

## Verified workaround

There is no workaround that recovers Ghostty's bar shape while
Claude Code is running.
 The cursor the user sees is application
output;
 the terminal does not control it.

Tradeoff:
 accept the visual divergence between shell prompt
(bar) and Claude Code prompt (block).
 The alternative is to wait
for a Claude Code update that supports cursor-style customisation
(see upstream tracking below).

## What does not work

- `cursor-style = bar` in `ghostty.config`:
   correctly sets the
  terminal default.
   Claude Code hides the real cursor,
   so the
  setting is irrelevant once the app starts.
- **Issuing `\e[0 q` from a `PS0` preexec hook before Claude Code
  starts**:
   correctly resets the shape to the configured default,
  but the shape no longer applies because the cursor is hidden.
- `Se` / `Ss` terminfo capabilities:
   Claude Code does not use
  terminfo for cursor shape;
   the hardcoded `f$.inverse` rendering
  bypasses any terminfo entry.
- `infocmp` / patching Ghostty's terminfo:
   same as above;
   the
  application does not consult terminfo for cursor shape.

## Why we do not file this upstream (Claude Code)

This is a known limitation tracked in multiple Claude Code issues.
Walking the 5 constraints:

1.  **Is it really upstream's fault?
    ** Borderline.
     Ink's
    inverse-video cursor is a deliberate cross-terminal design that
    trades native cursor support for layout reliability.
     Claude
    Code inherits that design.
2.  **Can upstream fix it?
    ** Yes;
     emit DECSCUSR sequences in the
    TextInput component when the terminal advertises support,
     or
    expose a config flag that switches between visual and native
    cursor.
     Both are plausible but require Ink-level changes.
3.  **Are they supporting this use case?
    ** Several open issues
    request customisation;
     Anthropic has acknowledged the friction
    but not committed to a fix.
4.  **Will they likely fix it?
    ** Unknown;
     the multiple open issues
    suggest demand exists but has not been prioritised.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no new upstream report.
 Subscribe to existing tickets if
needed.

### Existing upstream tickets

- [Cursor style interference (issue 674)][674]
- [Cursor style support (issue 7002)][7002]
- [Customise cursor style (issue 10215)][10215]
- [Environment variable to disable cursor shape changes (issue 10534)][10534]

[674]: https://github.com/anthropics/claude-code/issues/674
[7002]: https://github.com/anthropics/claude-code/issues/7002
[10215]: https://github.com/anthropics/claude-code/issues/10215
[10534]: https://github.com/anthropics/claude-code/issues/10534

## Separate finding: Ghostty's terminfo defines `Se=\E[2 q` (steady block) instead of `\E[0 q` (reset to user default)

Located at `ghostty.zig:116`.

When programs that consult terminfo (vim,
 neovim,
 tmux) need to
restore cursor style after switching modes,
 they emit the `Se`
capability.
 Ghostty's `Se` resolves to `\E[2 q` (steady block),
which triggers the `.steady_block` branch in `setCursorStyle`
instead of the `.default` branch that would restore the
configured `cursor-style`.
 The result is that programs which
faithfully use terminfo end up with a steady block rather than
the user's preferred bar.

This is **independent** of the Claude Code finding above (Claude
Code does not use terminfo at all).
 It affects different
applications,
 but the root cause is in Ghostty's terminfo
definition.

### Upstream filing audit (Ghostty)

1.  **Is it really upstream's fault?
    ** Yes;
     `Se` should encode
    "reset to default" (`\E[0 q`),
     not "steady block".
2.  **Can upstream fix it?
    ** Yes;
     one-line value change at
    `src/terminfo/ghostty.zig:116`.
3.  **Are they supporting this use case?
    ** Yes;
     Ghostty's mission
    includes accurate terminfo.
4.  **Will they likely fix it?
    ** Yes (already accepted;
     see
    "Upstream status" below).
5.  **Have we prototyped a minimal fix?
    ** Yes.
     The minimal patch is
    the one-line value swap below,
     identical to the upstream
    commit `6c68650920804a202a3208d7d355368c9dd28a46`:

    ```diff
    --- a/src/terminfo/ghostty.zig
    +++ b/src/terminfo/ghostty.zig
    @@ -112,8 +112,8 @@ pub const ghostty: Source = .{
             // Cursor styles
             .{ .name = "Ss", .value = .{ .string = "\\E[%p1%d q" } },

    -        // Cursor style reset
    -        .{ .name = "Se", .value = .{ .string = "\\E[2 q" } },
    +        // Cursor style reset (to user configured default)
    +        .{ .name = "Se", .value = .{ .string = "\\E[0 q" } },

             // OSC 52 Clipboard
             .{ .name = "Ms", .value = .{ .string = "\\E]52;%p1%s;%p2%s\\007" } },
    ```

    Verification:
     the patch changes only the user-defined extended
    capability `Se`,
     which round-trips byte-for-byte through `tic` to
    a binary terminfo database and back through `infocmp -L -x`.
     This
    was checked at the terminfo-source level rather than via the full
    `build_data_exe +terminfo` pipeline because the changed line is
    exactly one capability value in
    `Source.encode()`'s output;
     nothing in the encoder,
     tic compiler,
    or infocmp decoder transforms the value string.
     The targeted
    harness was a minimal `.ti` file with the broken vs fixed `Se`
    plus the `Ss` parameterised setter (so `tic` accepts both as a
    pair):

    ```sh
    $ tic -x -o "$DB" broken.ti  # ghostty-broken with Se=\E[2 q
    $ tic -x -o "$DB" fixed.ti   # ghostty-fixed   with Se=\E[0 q
    $ diff \
        <(TERMINFO="$DB" infocmp -1 -x ghostty-broken) \
        <(TERMINFO="$DB" infocmp -1 -x ghostty-fixed)
    1,3c1,3
    < #     Reconstructed via infocmp from file: .../db/./g/ghostty-broken
    < ghostty-broken|Ghostty before Se fix,
    <       Se=\E[2 q,
    ---
    > #     Reconstructed via infocmp from file: .../db/./g/ghostty-fixed
    > ghostty-fixed|Ghostty after Se fix,
    >       Se=\E[0 q,
    ```

    The only inter-file difference is exactly the byte the patch
    changes;
     the existing Ghostty `stream_handler.zig:874-888` handler
    already maps `\E[0 q` to "restore `default_cursor_style` from
    config" (cited in the Verification section above),
     so the cursor
    behaviour on real terminals follows from the terminfo emission
    without a separate runtime probe.

Decision:
 already fixed upstream;
 no new issue to file.
 See next
subsection for the upstream artefacts and the release window.

### Upstream status (Ghostty)

The fix was filed and merged before this audit ran:

- Issue:
   [Inconsistent terminfo entry for resetting cursor style
  (#12482)][gh-issue-12482] (opened 2026-04-26,
   closed 2026-04-27).
- PR:
   [fix: update Se terminfo entry to reset cursor to configured
  default (#12487)][gh-pr-12487] (merged 2026-04-27 by upstream
  contributor Kyle Sower).
- Merged commit on `main`:
   `6c68650920804a202a3208d7d355368c9dd28a46`.

The fix is not yet in any tagged release.
 As of audit time,
 the
latest tag is `v1.3.1`,
 which still ships `Se=\E[2 q`;
 upstream
`main` is 842 commits ahead of `v1.3.1` and carries the corrected
value.
 Users on releases hit the bug until the next tag;
 users
building from `main` are already covered.

Audit verified against upstream clone `ghostty-org/ghostty` at
commit `e90b7c9fadadb5b7f936506dfd4f995729093108` (`origin`
`https://github.com/ghostty-org/ghostty.git`).

[gh-issue-12482]: https://github.com/ghostty-org/ghostty/issues/12482
[gh-pr-12487]: https://github.com/ghostty-org/ghostty/pull/12487

### Draft upstream issue (kept for audit trail, do not file)

Do not file:
 duplicates closed [#12482][gh-issue-12482],
 fixed by
merged PR [#12487][gh-pr-12487].
 Kept here so the 5-constraint
audit above has the draft it would have produced,
 and so a future
session reading this doc can see what the proposed report would
have looked like.

````md
Title: terminfo: `Se` should be `\E[0 q` (reset to default) instead of `\E[2 q` (steady block)

Labels: bug, terminfo

## Description

`src/terminfo/ghostty.zig:116` defines the `Se` (reset cursor
style) capability as `\E[2 q`, which is the DECSCUSR "steady block"
sequence. The semantic intent of `Se` in terminfo is "reset to the
user-configured default", which Ghostty implements as `\E[0 q`
inside the stream handler:

- `src/terminfo/ghostty.zig:116`:
  `.{ .name = "Se", .value = .{ .string = "\E[2 q" } }`
- `src/terminal/stream_handler.zig:874-888`: `\E[0 q` restores
  `default_cursor_style` from config; `\E[2 q` forces steady block.

Consequence: applications that consult terminfo (vim, neovim,
tmux) emit `Se` to restore the cursor after a mode switch, and end
up with steady block regardless of the user's `cursor-style`
setting.

## Reproduction

1. Set `cursor-style = bar` in `~/.config/ghostty/config`.
2. Launch a Ghostty window; confirm bar cursor at the shell prompt.
3. Run `tput cnorm`, or open vim/neovim and exit, or open tmux and
   detach. Each of these emits `Se`.
4. Observe the cursor is now steady block instead of bar.

## Suggested fix

```diff
--- a/src/terminfo/ghostty.zig
+++ b/src/terminfo/ghostty.zig
@@ -112,8 +112,8 @@ pub const ghostty: Source = .{
         // Cursor styles
         .{ .name = "Ss", .value = .{ .string = "\E[%p1%d q" } },

-        // Cursor style reset
-        .{ .name = "Se", .value = .{ .string = "\E[2 q" } },
+        // Cursor style reset (to user configured default)
+        .{ .name = "Se", .value = .{ .string = "\E[0 q" } },
```
````
