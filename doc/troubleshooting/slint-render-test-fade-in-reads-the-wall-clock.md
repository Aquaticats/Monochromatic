# A Slint software-render test draws a fading page at partial ink on a fast build

Recorded 2026-11-07T13:27:00-05:00.

## Symptom

`ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis` in
`package/music-player/desktop-app` failed with `the long title ends in an ellipsis`
(`src/ui_lists_render_tests.rs`) under `cargo nextest run --release` in the CachyOS image
(`localhost/monochromatic/music-player`,
Zig 0.17.1),
and passed under the debug profile on the host and in the same image.
The failure was first recorded on 2026-10-17 on two base images
(`cachyos-image-migration-package-differences.md`,
section "A test that fails on both images"),
and was blamed on the daily update `d61dc1934a` on 2026-11-07 although that update moved no Slint crate
(`slint` stayed 1.18.1;
only `icu_*`,
`unicode-*` and `font-types` moved).

## Cause

The Search page fades in over `Theme.motion-enter` (`page-enter` in `ui/app_window.slint`).
The test platform used Slint's default clock,
the wall clock,
so how far the fade had gone when the test drew its
frames depended on how long the three draw passes took.
A debug build is slow enough that the fade had ended;
a release build drew all three passes within the first
milliseconds,
so the page was drawn at partial opacity.
The title row still ended in the ellipsis,
but its ink differed from the background by less than the 60 levels that
`ends_in_dots` needs,
so the probe found no ink.
A frame dumped from the failing release run shows the Search row faded,
with the previous page's list numbers showing
through,
and the frame from the debug run shows it at full ink.

## Verification

- Same image,
  same fonts,
  same `SLINT_EMIT_DEBUG_INFO=1`:
  `--release` failed in 0.057 s,
  the debug profile passed in 0.273 s.
- With a platform clock that stands still,
  the test fails on the host debug build too,
  and the six other tests of the
  module pass (commit `80e9ead693`).
- With the clock moved by one `motion-enter` per draw pass,
  all seven pass on the host debug build and in the image
  under `--release` (commit `96c1da743a`).

## Fix

`SoftwarePlatform::duration_since_start` returns a thread-local test clock,
and each draw pass in `render_window_with`
moves it by `Theme.motion-enter`.
Other render tests that read the wall clock (`src/ui_render_support.rs`) pass today
but have the same exposure to a faster build.

## Not a cause

Fonts (installing `noto-fonts` or `ttf-dejavu` changed nothing,
as recorded on 2026-10-17),
the base image,
and the
2026-11-07 lock update.
