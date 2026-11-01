# Pi ask-user-question

Pi extension that lets the model ask free-form questions,
then blocks that model turn until the user finishes a multiline answer.

The question remains fully visible in Pi's transcript.
The answer opens in the configured editor inside a separate default-terminal window,
so Pi keeps its normal transcript controls while the answer is pending.

## Installation

Build the package,
then add its repository path to global Pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/ask-user-question"
  ]
}
```

The extension is user-specific.
Do not add it to project `.pi/settings.json`.

## Tool

The extension registers `ask_user_question` with one parameter:

```ts
ask_user_question({
  question: string,
})
```

The complete question is rendered in the Pi transcript without a line-count preview.
The tool is sequential,
so Pi does not open competing answer windows for sibling calls.

The tool works only in interactive TUI mode.
RPC,
JSON,
and print modes receive an unavailable-tool error rather than opening a desktop terminal.

## Answer workflow

The extension resolves the preferred terminal through
`@monochromatic-dev/terminal-exec-cli`.
It resolves the editor in this order:

1. `editor` in `~/.pi/agent/extensions/pi-ask-user-question.json`
2. `$VISUAL`
3. `$EDITOR`
4. `notepad.exe` on Windows or `vi` elsewhere

The user-level JSON config accepts an editor executable with optional quoted arguments:

```json
{
  "editor": "nano"
}
```

Unknown keys,
blank commands,
and unsupported shell syntax fail extension startup with the config path in the diagnostic.
The editor command must remain attached until editing finishes.
For graphical editors,
configure a wait flag such as `code --wait`.

When terminal-exec resolves Ghostty and the effective editor is `hx` or `helix`,
the extension emits a warning-level logger message about intermittent Escape handling.
Choose another editor through the user-level config to avoid that pairing.

The editor opens an empty,
answer-only UTF-8 file.
The question stays in Pi,
where scrolling,
search,
tool expansion,
selection,
copying,
and links remain available.

Save and exit to submit.
In Nano,
use `Ctrl+O`,
`Enter`,
then `Ctrl+X`.
In Helix,
Vim,
and Neovim,
use `:wq`.
The launched terminal title gives an editor-neutral save-and-exit reminder.

An empty or whitespace-only file cancels.
A nonzero editor exit,
a closed answer terminal,
or a disconnected helper also cancels.
The returned answer removes at most one final `\n` or `\r\n` added by the editor.
Every other character and line break is preserved.

Answers are ordinary,
non-secret model input.
Do not enter passwords,
tokens,
or credentials.

## Blocking and cleanup

A private request file transfers a random channel token and loopback endpoint to the detached helper.
The helper authenticates immediately and waits for the requester's acknowledgement before opening an editor.
It then keeps the channel open for the editor lifetime.
The Pi tool awaits that channel,
which blocks the model without taking input focus from the original Pi terminal.

Before opening the terminal,
the extension copies its self-contained helper into the private request workspace.
Replacing the installed helper after that point cannot remove the pending launch target.
The terminal starts an inline bootstrap rather than executing the temporary file directly.
If explicit cancellation removed the helper or request file before the terminal starts,
the bootstrap exits cleanly instead of producing a missing-module stack trace.
On Linux,
it uses the live Pi executable through procfs to survive removal of the original runtime path.
Other platforms and systems without accessible procfs use a checked original runtime path.
Missing launch inputs fail inside Pi with a path-specific diagnostic.

The channel keeps the requester alive while the detached helper starts.
Desktop startup and editing have no wall-clock deadline.
A terminal can legitimately defer command execution while a desktop is locked or a window is not rendered.
After 30 seconds without a connection,
the extension logs a warning but retains the pending question and its workspace.
If a terminal never opens,
inspect the desktop or cancel the tool in Pi;
the model does not continue merely because startup was slow.
Connected candidates have a separate bounded authentication wait,
which cannot expire the question.
Aborting the tool or shutting down the session closes every owned socket and removes the answer workspace.
An active helper then stops its editor without an unhandled shutdown error.

The bootstrap's cancellation message may disappear with its terminal window after a clean exit.
It is not a persistent notification.
Forcefully ending Pi before the terminal executes remains outside the supported request lifetime:
on Linux the retained runtime path also belongs to the Pi process.

Long answers follow Pi's tool-output limits.
The visible result is truncated when needed,
and the complete answer is retained in a private temporary file whose path is returned to the model.

## Validation

Run package validation from the repository root:

```sh
# Repository root.
mise run //package/pi-plugin/ask-user-question:build
mise run //package/pi-plugin/ask-user-question:test:unit
mise run //package/pi-plugin/ask-user-question:lint
mise run //package/pi-plugin/ask-user-question:verify:extension
mise run //package/pi-plugin/ask-user-question:verify:terminal
```

`verify:terminal` opens the real default terminal with a scripted editor,
checking multiline submission and blank cancellation without changing editor settings.
For diagnosed startup failures and remaining uncertainty,
see [the startup investigation](../../../doc/troubleshooting/pi-ask-user-question-startup.md).

End-to-end verification must also call the installed tool from a fresh interactive Pi session,
inspect the transcript while the editor remains open,
then exercise `:wq`,
empty cancellation,
and an overflowing question.
