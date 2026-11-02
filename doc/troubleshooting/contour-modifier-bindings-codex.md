# Contour 0.6.3 intercepts modifiers and loses Codex partial-region scrollback

## Symptom

On Bazzite `testing-45.20261101`,
with `contour-terminal-0.6.3.8249-2.fc45` and Codex CLI `0.166.0`,
the user reported:

- Shift+Left does not work.
- Ctrl+Shift+C exits Codex instead of copying.
- Scrolling in Codex does not work.

The user then instructed:
"Disable tabs in Contour altogether.
Should help with debugging and I don't need tabs anyway."
This authorized removing tab controls from the live configuration.
It did not authorize closing existing terminal sessions.

## Verified causes

The original configuration bound Shift+Left and Shift+Right to tab switching.
It had no Ctrl+Shift+C copy binding.
Its Ctrl+C copy action was conditional on an active terminal selection.

A real Contour instance inside `monochromatic-nested-wayland-session` was driven through the nested compositor's
keyboard and wheel input.
A Node child recorded raw PTY input.
The child exercised normal screen,
alternate screen,
Kitty keyboard disambiguation,
and SGR mouse-reporting modes.
No synthetic input reached the host desktop.

Original configuration:

- Left delivered `ESC [ D`.
- Shift+Left delivered no bytes in any tested mode.
- Ctrl+C delivered byte `03` in legacy mode.
- Ctrl+Shift+C delivered `C` in legacy mode.
- Under Kitty disambiguation,
  Ctrl+C delivered `ESC [ 99 ; 5 u`,
  and Ctrl+Shift+C delivered `ESC [ 99 ; 6 u`.
- Wheel-up delivered SGR mouse events when the child enabled mouse reporting.
- Normal-screen scrollback moved from the bottom;
  alternate-screen output had no scrollback to expose.

### Shift+Left is consumed by the tab action

Contour executes matching input bindings before encoding input for the application.
`src/contour/TerminalSession.cpp:1062` at
`3aef6f7243f571fd4fda2642da190a244bb87a57` returns after an action executes:

```cpp
// Contour v0.6.3.8249, src/contour/TerminalSession.cpp:1066-1070
handleAction(actions, eventType, [&](auto const& actions) {
    executionCount = executeAllActions(actions);
});
if (executionCount > 0)
    return;
```

The tab-switching action returns success even when there is no useful adjacent tab.
Removing the binding changes the observed child input from nothing to `ESC [ 1 ; 2 D`.

### Ctrl+Shift+C reaches a Codex exit binding

The original Contour configuration has no clipboard binding for this chord.
With keyboard disambiguation enabled,
Contour correctly emits `ESC [ 99 ; 6 u`,
preserving both Control and Shift.

Codex CLI `0.166.0`,
commit `6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`,
explicitly treats this chord as quit in an external-writer view:

```rust
// codex-rs/tui/src/app/input.rs:428-435
KeyCode::Char('c' | 'C') => {
    modifiers == KeyModifiers::CONTROL
        || modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT)
}
// When this match sets quit:
self.app_event_tx.send(AppEvent::Exit(ExitMode::Immediate));
```

This matches the user's report of the attached Codex CLI exiting.
The implemented remedy consumes the chord in Contour for clipboard copying before it reaches Codex.

### Codex writes into a region that Contour does not retain as history

The user clarified that the wheel failed in the normal conversation view,
then provided a screenshot and a pasted excerpt showing the limited reachable history.
This narrowed the symptom from missing wheel input to missing native scrollback.

Codex's standard history-insertion path reserves the bottom rows for its composer,
then writes transcript lines into a partial-height scrolling region:

```rust
// codex-rs/tui/src/insert_history.rs:205-216, rust-v0.166.0
queue!(writer, SetScrollRegion(1..area.top()))?;
queue!(writer, MoveTo(/*x*/ 0, cursor_top))?;
for line in &wrapped {
    queue!(writer, Print("\r\n"))?;
    write_history_line(writer, line, wrap_width)?;
}
```

Contour's grid preserves departing lines in history for a full-height scroll:

```cpp
// Contour v0.6.3.8249, src/vtbackend/Grid.cpp:443-449
if (fullHorizontal)
{
    if (fullVertical)
        return scrollUp(n, defaultAttributes);
    // Partial-height regions use the separate in-place row movement path.
}
```

The partial-height path instead moves and resets rows inside that region,
then returns `LineCount(0)` at `src/vtbackend/Grid.cpp:499`.
It does not add those departing rows to native scrollback.
This is an interoperability problem between Codex's insertion strategy and Contour's scrollback behavior.

A bounded positive-control test wrote the same 120 numbered lines with full-screen and partial-region scrolling.
Contour's own state dumps reported:

```text
# scroll-dump/main-state.txt
history line count   : 121 (max 10000)
# scroll-dump/region-state.txt
history line count   : 29 (max 10000)
```

The partial-region output had no corresponding historical rows;
the retained rows were pre-existing visible-screen content preserved by the clear-screen transition.
The visible region ended at numbered line 119,
with its first surviving line 101.
This explains why a scrollbar can have some history while lacking most of the conversation.
The exact historical row count in the user's original live window was not read.

### The initial 1,000-row explanation was insufficient

Codex has an automatic 1,000-row replay ceiling for unidentified terminals,
and Contour is configured to retain 10,000 lines.
Those settings alone do not explain the user's roughly 200 visible lines.
The partial-region test loses history with only 120 input lines,
below either ceiling.
Increasing a row limit does not address this demonstrated loss mechanism.
No history-limit setting was changed.

### OdyTTY comparison

The user subsequently reported that Codex scrollback works in OdyTTY.
The running AppImage's native executable reports `odytty 0.17.5`.
Its source tag `v0.17.5` resolves to `a12b66dfdc97e2789677c973ce19e1289af6c696`.
OdyTTY implements its own terminal core.

That source contains an explicit primary-screen path for a top-anchored partial scrolling region:

```rust
// OdyTTY v0.17.5, src/core/screen/mod.rs:1221-1230, comments omitted
} else if region.top == 0 && self.primary_screen.is_none() {
    self.scroll_up_region_into_scrollback();
```

The called method preserves the departing row:

```rust
// OdyTTY v0.17.5, src/core/screen/ops.rs:139-146, comments omitted
let removed = self.rows.remove(region.top);
self.scrollback.push_row(removed);
```

See [the region-preserving implementation](https://github.com/ghreprimand/odytty/blob/a12b66dfdc97e2789677c973ce19e1289af6c696/src/core/screen/ops.rs#L122-L156).
Its comments describe precisely the fixed-bottom-composer layout used by Codex.
This source difference is consistent with the user's successful OdyTTY observation.
OdyTTY was not independently driven through the earlier runtime fixture,
and this comparison makes no wider reliability or policy-eligibility claim.

## Authorized tab change

Removed the 13 active tab-creation and tab-switching bindings from
`~/.config/contour/contour.yml`.
Removed the `tab_bar_position` and `tab_bar_visibility` entries:
the installed-version source does not implement those settings,
and its QML has no tab-bar widget.
The active keybindings were the relevant tab controls.

The original file was backed up privately to
`~/temp/agent/contour-input-20261101/contour.yml.before-tabs`.
The write checked that the live file still matched the original snapshot before changing it.
Existing sessions were not closed.

Before applying the change,
the modified copy was tested through the same input probe:
Shift+Left then delivered `ESC [ 1 ; 2 D` in every tested mode.
Normal Left,
Ctrl+C,
Ctrl+Shift+C,
and mouse-report delivery retained their measured behavior.

## Verified changes and workarounds

### Clipboard binding applied

The user clarified that Ctrl+Shift+C should copy.
Added these active bindings after testing them in a copied configuration:

```yaml
# ~/.config/contour/contour.yml, within input_mapping
- { mods: [Control,Shift], key: 'C', action: CopySelection, mode: 'Select' }
- { mods: [Control,Shift], key: 'C', action: SendChars, chars: '', mode: '~Select' }
```

The selected-text test copied `COPY-PROBE alpha beta` to the isolated Wayland clipboard.
After cancelling the selection,
Ctrl+Shift+C preserved that clipboard value.
Neither case delivered bytes to the terminal child.
Ctrl+C still delivered the application's interrupt key.
The second binding deliberately consumes the shortcut when no selection exists:
an unconditional CopySelection would clear the clipboard,
while a selection-only binding would leak the shortcut to Codex.

The original no-tabs configuration was backed up as `contour.yml.before-copy` in the private scratch directory.

The first clipboard harness invocation used an unsupported nested-compositor drag option,
`interval=`,
and received `err drag does not take interval=; its options are mods, steps, step-ms`.
It did not count as a clipboard pass.
The successful run used the documented `step-ms=20`.

### Input and screen probes

Private scratch directory:
`~/temp/agent/contour-input-20261101/`.
It contains `probe.ts`,
the original and candidate configurations,
input JSONL records,
and screenshots.
The probe runs a real Contour child with copied configuration:

```sh
# Read-only user configuration snapshot is passed to an isolated test terminal.
node "$HOME/temp/agent/contour-input-20261101/probe.ts" \
  "$HOME/temp/agent/contour-input-20261101/original.yml" original
```

Runs use independent systemd scopes with `CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=256`,
and `IOWeight=10`.
Each run has an internal deadline and closes its nested compositor.

### Codex's full transcript viewer retains the source history

A separate fixture used the installed Codex CLI binary,
with a disposable home mounted over `.codex`,
a read-only host filesystem,
network isolation,
and synthetic local conversation records.
No user session or authentication file was modified.
The fixture contained 30 question/answer pairs,
each answer containing 50 numbered diagnostic lines.

In ordinary terminal-scrollback mode,
the native view could not reach the beginning of this conversation.
The built-in transcript viewer opened with Ctrl+T,
and Ctrl+Home reached `Diagnostic question 0` and `DIAGNOSTIC turn-00 line-00`.
A subsequent wheel-down event moved the transcript viewport.
Screenshots recorded both states.

This is a verified way to read the full history without changing the user's persistent Codex mode.
The tradeoff is using Codex's transcript viewer rather than Contour's native scrollbar.
The user's existing `fullscreen_transcript = false` setting was preserved.

### Final configuration boundary

The final input/clipboard probe read the real
`~/.config/contour/contour.yml`,
not a candidate copy.
It verified Shift+Left forwarding,
Ctrl+Shift+C interception,
selected-text clipboard copying,
and clipboard preservation without a selection.
The user's file has live configuration reloading enabled.
No existing terminal process was restarted or closed.

## What does not work

- Keeping Shift+Left assigned to tab switching and expecting the application to receive it.
- A selection-only Ctrl+Shift+C binding:
  the unselected case would still reach Codex.
- An unconditional CopySelection binding:
  Contour copies an empty string when no selection exists,
  overwriting the clipboard.
- Increasing Contour's history limit to repair discarded partial-region lines.
- Treating Codex's replay ceiling as the measured number of rows in the user's window.
- Counting an empty initial fixture as a successful history test.
  The initial synthetic rollout had model messages but lacked UI event records;
  the corrected fixture supplied user-message and agent-message events and visibly rendered numbered answers.

## Diagnostic environment limits

The workspace's `bwrap` launcher resolved to a ZIP archive and failed with Bash's
`cannot execute binary file: Exec format error`.
`file --dereference` identified the archive.
The installed `/usr/bin/bwrap` was an x86-64 ELF executable and ran the isolated fixtures successfully.
This separate toolchain artifact problem was not repaired during the terminal investigation.

The first sandbox used a read-only root without a usable `/dev/null`.
The diagnostic shell wrapper reported `Permission denied`.
The corrected fixture used a private `/dev` and the native Codex binary directly.
Only the corrected numbered-history fixture supplied evidence for the scrolling result.

## Source references

- [Contour input dispatch](https://github.com/contour-terminal/contour/blob/3aef6f7243f571fd4fda2642da190a244bb87a57/src/contour/TerminalSession.cpp#L1059-L1075).
- [Contour full-height versus partial-height scroll](https://github.com/contour-terminal/contour/blob/3aef6f7243f571fd4fda2642da190a244bb87a57/src/vtbackend/Grid.cpp#L443-L499).
- [Codex standard history insertion](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/tui/src/insert_history.rs#L205-L217).
- [Codex external-writer quit handling](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/tui/src/app/input.rs#L423-L436).

The tagged clones were read locally.
Browser navigation verified the source identities;
the Codex quit clause also matched rendered page text.
The other large source views did not expose the searched symbols in their initial rendered DOM,
so their excerpts were verified against the clones rather than claimed as browser-rendered verification.

## Upstream filing decision

No external report has been drafted or posted.
The shortcut behavior is explained by user configuration and Codex's documented-in-source key handling.
The scrollback behavior is a reproduced integration limitation,
not evidence that the conversation's persisted data was deleted.

- Responsibility:
  no upstream defect is needed to explain the configured shortcuts;
  the history probe establishes an interoperability limitation,
  not a protocol-conformance verdict.
- Fixability:
  applications can use a full-window insertion strategy or maintain their own transcript viewport.
  Codex's existing transcript viewport was tested successfully.
- Supported use:
  interactive terminal applications are within Contour's scope.
- Contribution policy:
  no recommendation to file is made,
  so maintainer intake approval was not sought.
- Direction:
  the search `gh search issues 'Codex scrollback' --repo contour-terminal/contour` returned no matching issue;
  this is not evidence of maintainer willingness or unwillingness.
- Prototype:
  no upstream source patch was created.
  The authorized configuration changes and existing transcript-view workaround were validated at runtime.
