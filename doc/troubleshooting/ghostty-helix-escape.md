# Ghostty 1.3.1 with Helix 25.07.1 intermittently gives no response to physical Escape in a detached editor

## Symptom

A detached Ghostty window ran Helix as the answer editor for Pi's `ask_user_question` tool.
Pressing the physical Escape key sometimes had no visible effect.
The behavior was intermittent.
No stable key sequence,
time interval,
or editor state reproduced it on demand.

The report concerns the physical Escape key.
It is not the separate `Ctrl+[` compatibility behavior discussed in Helix issue [#6551][].

## Root cause

The root cause is not established.
There is no deterministic reproduction,
raw terminal-input capture,
Helix verbose log,
or Ghostty input log from a failing keypress.
Without that evidence,
the failure cannot be assigned to Ghostty,
Helix,
the input method,
or focus routing.

The source trace shows that both programs implement the expected Escape path.
It also identifies boundaries a future capture must distinguish.

### Ghostty maps GTK Escape to its internal key

At tag `v1.3.1`,
commit `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`,
Ghostty maps the GDK key at `src/apprt/gtk/key.zig:448-462`:

```zig
.{ gdk.KEY_Page_Up, .page_up },
.{ gdk.KEY_Page_Down, .page_down },
.{ gdk.KEY_Escape, .escape },
.{ gdk.KEY_Return, .enter },
.{ gdk.KEY_Tab, .tab },
```

The ordinary legacy encoder includes an unmodified Escape byte at
`src/input/function_keys.zig:222-238`:

```zig
result.set(.escape, &.{
    // Modified forms omitted.
    .{ .sequence = "\x1b" },
});
```

There is therefore no static omission of physical Escape from Ghostty's GTK mapping or legacy function-key table.

### Ghostty can stop before encoding when the input method consumes an event

`src/apprt/gtk/class/surface.zig:1281-1306` returns before key encoding for several input-method states:

```zig
if (im_handled) {
    if (priv.im_composing) return true;

    if (priv.in_keyevent == .composing) return true;

    if (priv.im_len == 0) return true;
}
```

This is a candidate boundary,
not a diagnosis.
The failing event was not captured with the values of `im_handled`,
`im_composing`,
`in_keyevent`,
or `im_len`.

### Helix enables enhanced keyboard reporting

At tag `25.07.1`,
commit `a05c151bb6e8e9c65ec390b0ae2afe7a5efd619b`,
Helix probes terminal support at
`helix-tui/src/backend/crossterm.rs:125-139`:

```rust
fn supports_keyboard_enhancement_protocol(&self) -> bool {
    *self.supports_keyboard_enhancement_protocol
        .get_or_init(|| {
            let supported = matches!(terminal::supports_keyboard_enhancement(), Ok(true));
            log::debug!(
                "The keyboard enhancement protocol is {}supported in this terminal (checked in {:?})",
                if supported { "" } else { "not " },
                Instant::now().duration_since(now)
            );
            supported
        })
}
```

When supported,
`helix-tui/src/backend/crossterm.rs:180-187` requests disambiguated Escape codes and alternate keys:

```rust
PushKeyboardEnhancementFlags(
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
)
```

This negotiation changes terminal encoding,
but no captured failure proves that protocol state was incorrect.

### Helix maps and handles the parsed Escape event

Crossterm Escape maps directly to Helix Escape in
`helix-view/src/keyboard.rs:330-342`:

```rust
CKeyCode::Char(character) => KeyCode::Char(character),
CKeyCode::Null => KeyCode::Null,
CKeyCode::Esc => KeyCode::Esc,
CKeyCode::CapsLock => KeyCode::CapsLock,
```

The keymap handles Escape specially at
`helix-term/src/keymap.rs:329-343`:

```rust
if key!(Esc) == key {
    if !self.state.is_empty() {
        return KeymapResult::Cancelled(self.state.drain(..).collect());
    }
    self.sticky = None;
}
```

There is no static omission after Crossterm produces `CKeyCode::Esc`.
A future diagnosis must establish whether the failing press reaches this conversion.

## Verification

### Versions and effective pairing

The affected environment measured:

```text
Ghostty 1.3.1
helix 25.07.1 (a05c151b)
EDITOR=hx
terminal-exec entry=com.mitchellh.ghostty.desktop
```

The user observed the missing response more than once,
but the incident had no deterministic trigger.
This is symptom evidence,
not a red-capable automated harness.

### Compatibility detection

The extension detects the effective executable name and terminal entry.
`package/pi-plugin/ask-user-question/src/editor-compatibility.ts:41-58` checks both dimensions:

```ts
export function isGhosttyHelixCombination({
  terminalEntryId,
  editorCommand,
}: {
  readonly terminalEntryId: string;
  readonly editorCommand: readonly string[];
}): boolean {
  if (!terminalEntryId.toLowerCase().includes(GHOSTTY_ID_FRAGMENT,))
    return false;
  const [executable,] = editorCommand;
  if (executable === undefined)
    return false;
  return HELIX_EXECUTABLE_NAMES.has(basename(executable,).toLowerCase(),);
}
```

`package/pi-plugin/ask-user-question/src/request-external-answer.ts:177-181`
emits the warning through the package's tagged logger:

```ts
if (((typeof terminalEntryId) !== 'symbol') && isGhosttyHelixCombination({
  terminalEntryId,
  editorCommand,
},))
  warn(GHOSTTY_HELIX_WARNING,);
```

The package unit suite produced the warning-level line:

```text
[warn] [ask-user-question:request-external-answer] Detected Ghostty with Helix. Escape may intermittently fail in this pairing; set editor to another command in the user-level pi-ask-user-question.json config.
```

The detection catalog covers:

- Ghostty desktop entry plus `hx`:
  warning.
- Case-insensitive Ghostty identity plus absolute `helix`:
  warning.
- Ghostty plus `nano`:
  no warning.
- Konsole plus `hx`:
  no warning.

Run the package checks with:

```sh
cd package/pi-plugin/ask-user-question
mise run build
mise run test:unit
mise run lint
mise run verify:extension
```

All commands passed after the workaround landed.

### Real Nano answer boundary

The user-level override is:

```json
{
  "editor": "nano"
}
```

at `~/.pi/agent/extensions/pi-ask-user-question.json`.
A full Pi restart loaded the override.
The extension opened Nano,
and the user saved and exited with `Ctrl+O`,
`Enter`,
then `Ctrl+X`.
The tool returned exactly:

```text
nano line one
nano line two
```

This proves the selected workaround at the real Pi,
terminal,
editor,
helper,
and model-tool boundary.

## Verified workarounds

### Select Nano in user-level extension config

Set:

```json
{
  "editor": "nano"
}
```

in `~/.pi/agent/extensions/pi-ask-user-question.json`,
then restart Pi.
Nano submission uses `Ctrl+O`,
`Enter`,
and `Ctrl+X` and does not depend on Escape.

Tradeoff:
users who prefer modal editing use a different editor for this answer workflow.
The setting is scoped to the ask-user extension and does not change `$EDITOR` for other programs.

### Choose another attached editor command

The same config field accepts an executable with optional quoted arguments.
The editor must remain attached until editing finishes.
A graphical editor therefore needs its wait option.

Tradeoff:
only Nano received real end-to-end verification in this incident.
Other editor commands are supported by parser and unit tests but are not evidence about the intermittent Escape symptom.

Ranking:
Nano is preferred over an unverified alternative because Nano crossed the complete user boundary successfully.
An alternative attached editor is preferred over retaining Helix because it can avoid the observed pairing,
but it needs its own user-boundary check.

## What does not work

- Continuing to depend on physical Escape in this Ghostty and Helix pairing leaves the intermittent failure exposed.
- Treating Helix issue [#6551][] as the same bug is incorrect.
  That issue concerns the ambiguous `Ctrl+[` control sequence under enhanced keyboard protocols,
  not a physical Escape key that sometimes has no effect.
- `ydotool` is not a safe verifier for this workflow.
  Focus changed between Pi and the detached editor,
  synthesized keys reached the wrong window,
  and the active Pi interaction was corrupted.
- Running Pi `/reload` after rebuilding the `.mjs` extension created a separate mixed-artifact failure.
  See [Pi `.mjs` extension reload](pi-mjs-extension-reload.md).
  Restarting Pi is required after rebuilding this extension on Pi 0.84.2.
- No source-only argument can select among focus loss,
  input-method consumption,
  protocol negotiation,
  terminal encoding,
  and editor parsing without a failing input capture.

## Upstream filing decision

The `.out-of-scope/` directory has no exemption for Ghostty,
Helix,
or terminal keyboard protocols.

Searches covered open and closed Ghostty issues for
`Escape Helix Ghostty`,
`escape key ignored`,
and `helix keyboard protocol`.
They also covered open and closed Helix issues for
`escape key terminal`,
`escape ignored`,
and `ghostty`.
No exact physical-Escape intermittent duplicate was found.

The six constraints are:

1.  **Is it really upstream's fault?**
    Not established.
    The failing keypress was not captured at either side of the PTY,
and both source trees contain the expected physical Escape path.
2.  **Can upstream fix it?**
    Unknown until the failing boundary is identified.
    Ghostty,
    Crossterm,
    or Helix could fix a defect in their owned layer if evidence locates one.
3.  **Are they supporting this use case?**
    Yes in general.
    Ghostty encodes physical Escape,
    and Helix explicitly negotiates enhanced keyboard input and maps parsed Escape events.
4.  **Would the repositories welcome the contribution?**
    Ghostty requires an issue-triage discussion,
    first-contributor vouch,
    human review,
    and AI disclosure under `CONTRIBUTING.md` and `AI_POLICY.md`.
    Helix's `docs/CONTRIBUTING.md` welcomes tested bug fixes and has no AI prohibition in the inspected policy.
5.  **Will they likely fix it?**
    Unknown because no actionable report identifies an owned defect.
    No matching tracker thread contains maintainer direction.
6.  **Have we prototyped a minimal compatible fix?**
    No.
    Constraints 1 and 5 fail,
    so the automatic prototype requirement does not trigger.
    Changing either source tree without a red-capable reproduction would be speculation.

Nothing should be filed upstream from the current evidence.
The following draft is deliberately marked do not file as-is:

~~~md
Title: Physical Escape intermittently has no effect in Helix 25.07.1 under Ghostty 1.3.1

Do not file as-is.
This draft lacks a deterministic reproduction,
PTY byte capture,
Ghostty input log,
and Helix verbose log from the same failed keypress.
Collect those artifacts first,
then file only against the project whose owned boundary drops or misclassifies the event.

Observed environment:
- Ghostty 1.3.1
- Helix 25.07.1 (a05c151b)
- physical Escape key,
  not Ctrl+[

AI assistance was used for source tracing and draft preparation.
A human must reproduce,
review,
edit,
and disclose the final report before submission.
~~~

[#6551]: https://github.com/helix-editor/helix/issues/6551
