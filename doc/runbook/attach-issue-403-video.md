# Attach the sanitized recording to issue 403

## What this proves

This procedure adds the sanitized Android music-player recording to
[issue 403](https://github.com/Aquaticat/Monochromatic/issues/403)
without exposing the Android status bar,
library paths or titles,
the notification shade,
or original creation-time metadata.

Automated bridges tried before this handoff:

- GitHub CLI `2.96.0` created the issue,
  but its `gh issue create --help` interface has no file-attachment input.
- `agent-browser` reached GitHub's sign-in page,
  but its available Chromium profile was not authenticated.
- BrowserOS MCP was unavailable at its configured local endpoint.

## Setup

Status:
TODO | DONE

Use a desktop browser signed in to a GitHub account that can comment on
`Aquaticat/Monochromatic`.
Obtain `ui-controls-scroll-sanitized.mp4` from the reporter.
On the originating workstation it is stored at:

```text
/var/home/user/Downloads/ui-controls-scroll-sanitized.mp4
```

Verify that the selected file has this SHA-256 digest:

```text
960979dd73aac535af676038c6de416697d35ad0e0d1368ac6dfc346a462b857
```

## Steps

Status:
TODO | DONE

1.  Open `https://github.com/Aquaticat/Monochromatic/issues/403` in the signed-in browser.
    The page shows `Android music player control row stays fixed while the library scrolls`.
2.  Click the **Add a comment** field.
    A text caret appears in the comment editor.
3.  Drag `ui-controls-scroll-sanitized.mp4` into the **Add a comment** field.
    The upload completes and the editor contains a link whose text includes
    `ui-controls-scroll-sanitized.mp4` and whose URL starts with
    `https://github.com/user-attachments/assets/`.
4.  Click **Comment**.
    A new comment appears with the uploaded recording rendered as a playable attachment.

## What to check

Status:
TODO | DONE

- The issue title is exactly
  `Android music player control row stays fixed while the library scrolls`.
- The attachment filename is exactly `ui-controls-scroll-sanitized.mp4`.
- Playing the attachment shows the control-row scrolling problem.
- The Android status bar is covered by a solid black mask for the complete recording.
- The tabs and file-list region are pixelated so paths,
  folder names,
  and music titles are unreadable.
- The recording ends on the music-player screen before the Android notification shade appears.

## Restore

Status:
TODO | DONE

No restoration is needed after the correct recording is attached.
If the wrong file was posted:

1.  Open the comment's **More options** menu.
    The comment action menu appears.
2.  Click **Delete**.
    GitHub asks for deletion confirmation.
3.  Confirm with **Delete**.
    The incorrect attachment comment disappears.
4.  Repeat the procedure with the file whose SHA-256 digest matches the value in **Setup**.
    The replacement comment contains the sanitized recording.
