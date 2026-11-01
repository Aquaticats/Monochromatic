# Ask user question end-to-end verification

## What this proves

This procedure verifies the installed extension
through a real fullscreen Pi session
and Nano selected by user-level config.
It covers multiline submission,
cancellation,
and continued transcript interaction while the model waits.

A pseudo-terminal Pi session successfully reached the real Ghostty and Helix boundary.
A later `ydotool` attempt sent keys to the wrong focused window and altered the active Pi session.
Do not use synthesized keyboard input for this procedure.

## Setup

Status:
TODO

1. Open a terminal in `/var/home/user/Monochromatic`.
   The shell prompt should show that repository path.
2. Run `mise run //package/pi-plugin/ask-user-question:build`.
   The command should exit successfully and list `index.mjs` and `answer-helper.mjs`.
3. Run `pi list`.
   The output should contain
   `/var/home/user/Monochromatic/package/pi-plugin/ask-user-question`.
4. Confirm `~/.pi/agent/extensions/pi-ask-user-question.json` contains `{"editor":"nano"}`.
   The effective editor override should be Nano.
5. Close any stale Ghostty answer window.
   Its title is:
   **`Pi answer: save and exit to submit`**.
   In Nano,
   press **Ctrl+X**.
   The stale window should close.
6. Start `pi --tui-mode fullscreen`.
   A fresh interactive Pi view should appear.

## Steps

Status:
TODO

1. Enter this prompt in Pi,
   then press **Enter**:

   ```text
   Call ask_user_question exactly once. Ask a multiline question containing at least 40 numbered lines, the marker FINAL TRANSCRIPT MARKER, and https://example.com. After the tool returns, repeat the answer exactly.
   ```

   Pi should show an `ask_user_question` call,
   and a separate Ghostty window should open in Nano.
2. Switch back to the Pi window with **Alt+Tab**.
   The Pi transcript should remain visible while the tool call is still pending.
3. Press **PageUp**,
   then press **PageDown**.
   The fullscreen transcript should scroll in both directions without cancelling the tool.
4. Press **Ctrl+Shift+F**.
   A transcript search field should appear.
5. Type **FINAL TRANSCRIPT MARKER**,
   then press **Enter**.
   Pi should select a matching line from the complete question.
6. Press **Escape**.
   Transcript search should close while the tool remains pending.
7. Press **Ctrl+O** twice.
   The tool output should collapse and expand without resolving the call.
8. Drag across visible transcript text with the primary mouse button.
   The selected text should become highlighted and copied without resolving the call.
9. Click the visible [**example.com**](https://example.com) link if the terminal renders it as a link.
   The default browser should open that address,
   and the Pi tool should remain pending.
10. Switch to the Nano answer window with **Alt+Tab**.
    The empty answer document should still be open.
11. Type **first answer line**,
    press **Enter**,
    then type **second answer line**.
    Nano should show exactly two answer lines.
12. Press **Ctrl+O**,
    press **Enter**,
    then press **Ctrl+X**.
    The Nano window should close and Pi should resume.
13. Inspect the Pi tool result.
    It should contain these exact lines:

    ```text
    User answered:
    first answer line
    second answer line
    ```

14. Enter this prompt in Pi,
    then press **Enter**:

    ```text
    Call ask_user_question exactly once with the question Cancel this verification call.
    ```

    A new Nano answer window should open.
15. In Nano,
    press **Ctrl+X**.
    The unchanged empty document should close without a save prompt.
16. Inspect the Pi tool result.
    It should contain the exact text `User cancelled the question.`.

## What to check

Status:
TODO

- `pi list` contains `/var/home/user/Monochromatic/package/pi-plugin/ask-user-question`.
- The complete question contains `FINAL TRANSCRIPT MARKER`.
- Transcript scrolling,
  search,
  collapse and expansion,
  selection,
  and links remain usable while Helix is open.
- Multiline submission preserves both `first answer line` and `second answer line`.
- Empty **Ctrl+X** cancellation returns `User cancelled the question.`.
- No `Answer helper failed:` diagnostic appears.

## Restore

Status:
TODO

1. Close any remaining Nano answer window with **Ctrl+X**.
   If Nano asks whether to save an unwanted draft,
   press **N**.
   No detached answer window should remain.
2. Return to Pi and clear any accidental editor text with **Ctrl+C** once.
   The Pi input editor should become empty.
3. Press **Ctrl+D** with the empty Pi editor.
   Pi should exit to the shell without changing extension settings.
