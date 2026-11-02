# Contour 0.6.3 intercepts Shift+Left and forwards Ctrl+Shift+C to Codex

## Symptom

On Bazzite `testing-45.20261101`,
with `contour-terminal-0.6.3.8249-2.fc45` and Codex CLI `0.166.0`,
the user reported:

- Shift+Left does not work.
- Ctrl+Shift+C exits Codex instead of copying.
- Scrolling in Codex does not work.

The user then instructed:
"Disable tabs in Contour altogether. Should help with debugging and I don't need tabs anyway."
This authorized removing tab controls from the live configuration.
It did not authorize closing existing terminal sessions.

## Findings under investigation

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

These observations do not establish the full Codex-specific copy and scrolling causes.
They show that Contour can deliver modified keys and mouse reports,
and that Shift+Left is intercepted by configuration.

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

## Verification artifacts

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

## Remaining work

- Verify the final tab and copy changes against the live configuration file.
- Trace and reproduce Codex's handling of Ctrl+Shift+C.
- Determine Codex's actual screen and mouse modes for the scrolling report.
  The user confirmed wheel scrolling fails in the normal conversation view,
  not just a question dialog.
- Record source selections,
  verified remedies,
  and any remaining uncertainty.

## Upstream filing decision

No external report has been drafted or posted.
The observed shortcut interception is currently explained by configuration;
the Codex-specific behavior is still being investigated.
