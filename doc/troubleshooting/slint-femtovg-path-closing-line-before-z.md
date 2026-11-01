# Slint 1.18.1 femtovg draws a `Path` wrongly when a contour ends with a line back to its start before `Z`

Tool under test:
Slint 1.18.1 (`slint`, `i-slint-backend-winit`) with the `winit-femtovg` renderer,
hosted in the nested Wayland compositor 0.3.0 on Fedora 44.

## Symptom

A `Path` whose `commands` string draws a rectangle contour that ends with a `L <start>` and then `Z`
renders as a wrong filled shape under `SLINT_BACKEND=winit-femtovg`.
The three 2 px bars of the Material Symbols `queue_music` icon merged into one solid blob.

A single contour reproduces it:

```text
M 3 16 L 3 14 L 11 14 L 11 16 L 3 16 Z
```

The same data renders correctly under the Skia renderer and the software renderer.

## Workaround

Drop the redundant closing line and let `Z` close the contour:

```text
M 3 16 L 3 14 L 11 14 L 11 16 Z
```

The desktop music player's embedded icons in `package/music-player/desktop-app/ui/settings.slint` use this form,
and the unit test `ui_settings::tests::the_icon_paths_are_explicit_absolute_commands_that_slint_draws_whole` guards it.
Upstream SVG path data from Material Symbols ends every contour with the redundant line,
so any icon converted from those SVGs needs the closing line removed.

## Evidence and limits

- The probe is `/home/user/temp/agent/agent-desktop-settings/probe2.slint`,
  hosted in the nested compositor with `PROBE_BACKEND=winit-femtovg`.
  It is a scratch file on the machine that found it, not in the repository.
- The cause inside femtovg or Slint's path conversion was not investigated.
- No upstream issue was searched for or filed.
- Only Slint 1.18.1 was tested.
