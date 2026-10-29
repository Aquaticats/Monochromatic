# Slint 1.18.1 fluent style draws selected text black on the `#0078D4` selection fill in the dark scheme

## Symptom

In the fluent style's dark scheme,
selected text in `LineEdit` and `TextEdit`,
and anything an application colors with `Palette.selection-foreground`,
is black on a mid blue.
The light scheme draws the same fill with white text.

A software-rendered probe (see "Verification") printed the palette and counted
pure-dark and pure-light pixels inside each widget's selection rectangle:

```text
# ~/temp/agent/upstream-prototype.m2ruMmIk/run-probe.out, released Slint 1.18.1
light: Palette.background #FAFAFAFF selection-background #0078D4FF selection-foreground #FFFFFFFF
light: LineEdit selection fill px 5305, dark-ink px 0, light-ink px 1125
light: TextEdit selection fill px 5498, dark-ink px 0, light-ink px 1125
dark: Palette.background #1C1C1CFF selection-background #0078D4FF selection-foreground #000000FF
dark: LineEdit selection fill px 5318, dark-ink px 1172, light-ink px 0
dark: TextEdit selection fill px 5511, dark-ink px 1172, light-ink px 0
```

The two inks differ in legibility far more than the WCAG 2 ratio suggests:

- Black on `#0078D4`:
  WCAG 2 ratio 4.64,
  APCA Lc 33.4.
- White on `#0078D4`:
  WCAG 2 ratio 4.53,
  APCA Lc -76.2.

The WCAG 2 ratio rates black slightly higher.
APCA,
which models the polarity of dark text on a mid tone,
rates it at less than half of white.
The APCA reference font table (`fontLookupAPCA` in `Myndex/apca-w3` `src/apca-w3.js`,
table "Public Beta 0.1.7 (G)")
gives a minimum size of 100 px for Lc 33.4 at weight 400,
and 18 px for Lc 76.2.

In `package/ide/app`,
the source view's selected text was black on blue in the dark scheme until commit `ad6f7565f`.
The file tree and project search rows still use `Palette.selection-foreground`
(`ui/tree.slint:110` and `119`,
`ui/search.slint:120` and `130` on `main` at `f9d08ae79`),
and the find bar is a fluent `LineEdit`,
so selected rows and selected find text are still black on blue in the dark scheme.

## Root cause

The fluent palette pairs the selection fill with the ink Windows uses on a different,
lighter accent fill.
Code below is from `i-slint-compiler-1.18.1/widgets/fluent/` in the IDE build volume's cargo registry
(`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/index.crates.io-1949cf8c6b5b557f/`).
The same lines are unchanged on Slint `master` at `ad24fd95c` (2026-10-05).

### The palette

```slint
// i-slint-compiler-1.18.1/widgets/fluent/styling.slint:42
    out property <brush> accent-background: dark-color-scheme ? accentify(#60CDFF) : accentify(#005FB8);
    out property <brush> accent-foreground: dark-color-scheme ? #000000 : #FFFFFF;
    out property <brush> selection-background: accentify(#0078D4);
    out property <brush> selection-foreground: dark-color-scheme ? #000000 : #FFFFFF;
```

The selection fill is the same mid blue in both schemes,
but its ink flips to black in the dark scheme,
exactly like `accent-foreground`,
whose dark fill is the much lighter `#60CDFF`.
`Palette.selection-background` and `Palette.selection-foreground` are these two values
(`widgets/fluent/style-base.slint:31` to `32`).
`accentify` keeps the OKLCH lightness of its argument and takes hue and chroma from the desktop accent color,
when there is one (`styling.slint:25` to `33`).

### The widgets

`TextEdit` uses the palette pair (`widgets/fluent/textedit.slint:94` to `95`).
`LineEdit` and `SpinBox` use the selection fill with the accent ink:

```slint
// i-slint-compiler-1.18.1/widgets/fluent/lineedit.slint:49
            base := LineEditBase {
                selection-background-color: FluentPalette.selection-background;
                selection-foreground-color: FluentPalette.accent-foreground;
```

(`widgets/fluent/spinbox.slint:101` to `102` is the same pair.)

### What WinUI does

The current `microsoft/microsoft-ui-xaml` source (`main` at `4a04a2433`) keeps selected text white in both themes.
In `controls/dev/CommonStyles/Common_themeresources_any.xaml`,
the `Default` dictionary is the dark theme
(its `TextFillColorPrimary` is `#FFFFFF` at line 5,
against `#E4000000` in `Light` at line 209):

- `TextOnAccentFillColorSelectedText` is `#FFFFFF` in `Default` (line 11) and in `Light` (line 215).
- `TextOnAccentFillColorPrimary`,
  the ink for accent-filled controls,
  is `#000000` in `Default` (line 12) and `#FFFFFF` in `Light` (line 216).
- `AccentFillColorDefaultBrush` is `SystemAccentColorLight2` in `Default` (line 125)
  and `SystemAccentColorDark1` in `Light` (line 329):
  the dark theme's accent fill is a lighter tint,
  which is why its ink is black.
- `AccentFillColorSelectedTextBackgroundBrush` is `SystemAccentColor` in both (lines 124 and 328),
  and `TextBox` uses it as its selection highlight
  (`controls/dev/CommonStyles/TextBox_themeresources.xaml:39`,
  `146`,
  and the `SelectionHighlightColor` setter at `183`).

The text engine host returns white for the selected-text system color whatever the theme,
outside high contrast:

```cpp
// dxaml/xcp/core/native/text/Controls/TextServicesHost.cpp:549
COLORREF TextServicesHost::TxGetSysColor(_In_ INT index)
{
    COLORREF sysColor = GetSysColor(index);

    if (!UseHighContrastSelection(m_pTextBox->GetContext()))
    {
        switch (index)
        {
            case COLOR_HIGHLIGHT:
            {
                CSolidColorBrush* selectionHighlightColorNoRef = m_pTextBox->GetSelectionHighlightColorNoRef();
                // ...
            }

            case COLOR_HIGHLIGHTTEXT:
                return RGB(0xFF, 0xFF, 0xFF);
```

Slint's dark scheme thus uses the ink of WinUI's lighter accent fill (`TextOnAccentFillColorPrimary`)
on the fill of WinUI's selection (`SystemAccentColor`).
Black on `#60CDFF` measures WCAG 2 ratio 11.67 and APCA Lc 70.8,
so the accent pair itself is fine;
the selection pair is not.

### Where the pairing came from

[slint-ui/slint#3515](https://github.com/slint-ui/slint/pull/3515)
("Fluent:
fix selection foreground color",
merged 2023-09-26 as `4398a64ad`)
set the selected-text ink of `LineEdit`,
`SpinBox`,
and `TextEdit` to `Palette.text-on-accent-primary`.
At that commit,
`internal/compiler/widgets/fluent-base/styling.slint` defined
`accent-selected-text: #0078D4;` (line 37),
`accent-default: dark-color-scheme ? #60CDFF : #005FB8;` (line 31),
and `text-on-accent-primary: dark-color-scheme ? #000000 : #FFFFFF;` (line 41).
The later palette rename kept the values.

## Verification

Versions under test:
Slint `1.18.1`
(`slint` checksum `15e477d5ff6fec20909100ff8f7062432ebfbda7a62a0f37d0252aa126f38c83`,
`i-slint-compiler` checksum `f55d48ebef05e2bf09ba5a1fc67bcb11033b9e12a2a50ed3105c7d25e2b539d0`,
from `package/ide/app/Cargo.lock`),
and the tag `v1.18.1` (`372cf0ee5`) of `https://github.com/slint-ui/slint.git`,
whose `internal/compiler/widgets/fluent/` is byte-identical to the crate's `widgets/fluent/`
(`diff --recursive` printed nothing).

### Probe

A binary crate with the fluent style,
no windowing backend,
and Slint's software renderer drawing into a buffer.
`src/InterVariable.ttf` is copied from `package/ide/app/asset/font/`,
and the IDE's `Cargo.lock` seeds the versions.

```toml
# probe/Cargo.toml
[package]
name = "fluent-selection-probe"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-2", "renderer-software"] }
# The generated code loads embedded widget images through the decoder entry point.
i-slint-core = { version = "=1.18.1", features = ["image-decoders", "svg"] }

[workspace]
```

```rust
// probe/src/main.rs
use slint::Rgb8Pixel;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, PlatformError};
use std::rc::Rc;

slint::slint! {
    import { LineEdit, TextEdit, Palette } from "std-widgets.slint";
    import "InterVariable.ttf";
    export component Probe inherits Window {
        width: 400px;
        height: 200px;
        default-font-family: "Inter Variable";
        out property <brush> palette-ink: Palette.selection-foreground;
        out property <brush> palette-fill: Palette.selection-background;
        out property <brush> palette-background: Palette.background;
        public function scheme(dark: bool) {
            Palette.color-scheme = dark ? ColorScheme.dark : ColorScheme.light;
        }
        public function select-line() {
            line.focus();
            line.select-all();
        }
        public function select-text() {
            text.focus();
            text.select-all();
        }
        line := LineEdit { x: 10px; y: 10px; width: 380px; height: 56px; font-size: 32px; text: "WWWWWW"; }
        text := TextEdit { x: 10px; y: 80px; width: 380px; height: 110px; font-size: 32px; text: "WWWWWW"; }
    }
}

struct Headless {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

fn render(window: &MinimalSoftwareWindow) -> Vec<Rgb8Pixel> {
    slint::platform::update_timers_and_animations();
    window.request_redraw();
    let mut buffer = vec![Rgb8Pixel { r: 0, g: 0, b: 0 }; (WIDTH * HEIGHT) as usize];
    window.draw_if_needed(|renderer| {
        renderer.render(&mut buffer, WIDTH as usize);
    });
    buffer
}

// Count selection-fill, dark-ink, and light-ink pixels inside the fill's bounding box within one widget area.
fn inks(buffer: &[Rgb8Pixel], top: u32, bottom: u32) -> (usize, usize, usize) {
    let is_fill = |p: &Rgb8Pixel| p.r <= 8 && (112..=128).contains(&p.g) && (204..=220).contains(&p.b);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    let mut fill = 0;
    for y in top..bottom {
        for x in 0..WIDTH {
            if is_fill(&buffer[(y * WIDTH + x) as usize]) {
                fill += 1;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    let (mut dark, mut light) = (0, 0);
    if fill > 0 {
        for y in y0 + 1..y1 {
            for x in x0 + 1..x1 {
                let p = &buffer[(y * WIDTH + x) as usize];
                if p.r.max(p.g).max(p.b) <= 40 {
                    dark += 1;
                }
                if p.r.min(p.g).min(p.b) >= 215 {
                    light += 1;
                }
            }
        }
    }
    (fill, dark, light)
}

fn hex(brush: slint::Brush) -> String {
    let c = brush.color();
    format!("#{:02X}{:02X}{:02X}{:02X}", c.red(), c.green(), c.blue(), c.alpha())
}

fn main() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless { window: window.clone() })).unwrap();
    let ui = Probe::new().unwrap();
    window.set_size(PhysicalSize::new(WIDTH, HEIGHT));
    ui.show().unwrap();
    for dark in [false, true] {
        ui.invoke_scheme(dark);
        let name = if dark { "dark" } else { "light" };
        println!(
            "{name}: Palette.background {} selection-background {} selection-foreground {}",
            hex(ui.get_palette_background()),
            hex(ui.get_palette_fill()),
            hex(ui.get_palette_ink())
        );
        ui.invoke_select_line();
        let (fill, dark_ink, light_ink) = inks(&render(&window), 10, 66);
        println!("{name}: LineEdit selection fill px {fill}, dark-ink px {dark_ink}, light-ink px {light_ink}");
        ui.invoke_select_text();
        let (fill, dark_ink, light_ink) = inks(&render(&window), 80, 190);
        println!("{name}: TextEdit selection fill px {fill}, dark-ink px {dark_ink}, light-ink px {light_ink}");
    }
}
```

The fill count is the positive control:
it proves the selection was drawn and located before the ink counts are read.
The probe ran offline in the IDE build image with a private copy of the IDE cargo registry,
once against the released compiler and once with the patched copy described under "Upstream filing decision":

```sh
# run from the directory that holds probe/, cargo/, and i-slint-compiler-patched/
podman run --rm --network=none --memory=2g --cpus=2 --pids-limit=512 --security-opt label=disable --volume "${PWD}:/proto" --volume "${PWD}/cargo:/cargo" --workdir /proto/probe --env CARGO_HOME=/cargo --env CARGO_BUILD_JOBS=2 --env SLINT_STYLE=fluent --env SLINT_EMBED_RESOURCES=true localhost/monochromatic/ide cargo run --offline
# the patched run appends:
#   --config 'patch.crates-io.i-slint-compiler.path="/proto/i-slint-compiler-patched"'
```

### Contrast measurement

`Myndex/apca-w3` `src/apca-w3.js` at `6c5c066cf`,
with its unused `colorparsley` import removed and saved as `apca-w3.mjs`:

```js
// contrast.mjs
import { APCAcontrast, sRGBtoY } from './apca-w3.mjs';

const linear = channel => {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
};
const luminance = ([red, green, blue]) => 0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue);
const ratio = (first, second) => {
  const [low, high] = [luminance(first), luminance(second)].sort((a, b) => a - b);
  return (high + 0.05) / (low + 0.05);
};
const hex = value => [value >> 16 & 0xff, value >> 8 & 0xff, value & 0xff];
for (const [ink, fill] of [[0x000000, 0x0078d4], [0xffffff, 0x0078d4], [0x000000, 0x60cdff], [0xffffff, 0x60cdff]]) {
  const lc = APCAcontrast(sRGBtoY(hex(ink)), sRGBtoY(hex(fill)));
  console.log(ink.toString(16), 'on', fill.toString(16), ratio(hex(ink), hex(fill)).toFixed(2), Number(lc).toFixed(1));
}
```

Results:
`#000000` on `#0078D4` 4.64 and Lc 33.4;
`#FFFFFF` on `#0078D4` 4.53 and Lc -76.2;
`#000000` on `#60CDFF` 11.67 and Lc 70.8;
`#FFFFFF` on `#60CDFF` 1.80 and Lc -37.1.
`fontLookupAPCA(33.4)` returned `["33.40",777,777,112,100,84,72,56,56,56]`
and `fontLookupAPCA(-76.2)` returned `["-76.20",59,41,24,18,16,15,14,16,18]`
(minimum px by weight 100 to 900).

### Works cleanly

- Light scheme,
  released and patched:
  `LineEdit` and `TextEdit` selections have 1125 light-ink pixels and no dark-ink pixels.
- Dark scheme with the patch:
  `selection-foreground #FFFFFFFF`,
  and both widgets have 1125 light-ink pixels and no dark-ink pixels.

### Fails

- Dark scheme,
  released:
  `selection-foreground #000000FF`;
  `LineEdit` and `TextEdit` selections have 1172 dark-ink pixels and no light-ink pixels.

`SpinBox` was not rendered;
its markup uses the same pair as `LineEdit`.

### IDE guard control

The IDE's ink rule has its own guard:
`package/ide/app/bin/inspect-source-guards.mjs`
(now `package/ide/app/bin/inspect-source-guards.ts`,
renamed 2026-10-22),
guard `ink-white-first`,
replaces the rule with "pick the higher WCAG 2 ratio" in a disposable copy.
On the tree committed as `374ea9a36`
(`~/temp/agent/ide-source-guard-Qu70pZ/ink-white-first-removed.log`)
`dark_scheme_selection_ink_is_light_on_the_fluent_selection_background` failed with
`dark-scheme selected text stayed dark on the selection background`,
left `[0, 0, 0, 255]`,
right `[255, 255, 255, 255]`.

## Verified workarounds

### Choose the ink from the fill in application code

`package/ide/app/src/selection_ink.rs:92` to `100` (`main` at `f9d08ae79`)
takes white while white reaches a WCAG 2 ratio of 3 against an opaque fill,
black otherwise,
and keeps the palette ink for a translucent fill:

```rust
// package/ide/app/src/selection_ink.rs:92
pub fn legible_ink(background: [u8; 4], palette: [u8; 4]) -> [u8; 4] {
    if background[3] != 255 {
        return palette;
    }
    if contrast(WHITE, background) >= WHITE_MINIMUM {
        return WHITE;
    }
    return BLACK;
}
```

`ui/app.slint:109` to `111` exports `Palette.selection-foreground` and `Palette.selection-background`,
and `src/native/render.rs:50` to `52` passes them in when the source raster is drawn.

Tradeoffs:

- It covers only text the application draws itself.
  Fluent `LineEdit`,
  `TextEdit`,
  and `SpinBox` keep their internal ink,
  so the IDE's find input still shows black selected text in the dark scheme.
- The IDE's tree and search rows still bind `Palette.selection-foreground`;
  they need the same treatment.
- A translucent fill falls back to the palette ink,
  because the color behind it is unknown at that point.
- The rule follows the fill,
  so it also holds for an accentified fill whose hue comes from the desktop;
  such fills were not measured.

## What does not work

- Choosing the ink with the higher WCAG 2 ratio:
  it picks black (4.64 against 4.53).
  The `ink-white-first` guard control is exactly this rule and fails the dark-scheme test.
- `Palette.selection-foreground`:
  it is the black ink this document is about.

## Upstream filing decision

`.out-of-scope/` was inspected.
Its only Slint mention (`cargo-workspace.md:18`) concerns container builds,
not upstream tracking,
so no exemption applies.

Duplicate search on 2026-10-05,
open and closed,
issues and pull requests,
`gh search issues --repo slint-ui/slint` and `gh search prs --repo slint-ui/slint` with
`selection-foreground`;
`selection foreground dark`;
`selection color dark`;
`selected text black`;
`selection contrast`;
`fluent selection color`;
`accent-foreground`;
`text-on-accent-primary`;
`LineEdit selection black`
(words passed as separate arguments,
since one quoted argument becomes a phrase search).
No report of this pairing was found.
The nearest threads are
[slint-ui/slint#3515](https://github.com/slint-ui/slint/pull/3515) (the 2023 change),
[slint-ui/slint#6326](https://github.com/slint-ui/slint/issues/6326)
(closed,
"`TextInput`'s selection colors should depend on style"),
and [slint-ui/slint#9122](https://github.com/slint-ui/slint/issues/9122)
(open,
a missing hover color in `Palette`).

1.  Upstream's fault:
    yes.
    The fill is the same in both schemes;
    only the ink flips,
    and the dark ink is the one WinUI pairs with a lighter fill.
    WinUI's own selected-text ink is white in both themes.
2.  Fixable:
    yes,
    in three lines of widget markup.
3.  Supported use case:
    yes.
    Fluent is a built-in style,
    `LineEdit`,
    `TextEdit`,
    and `SpinBox` are standard widgets,
    and `Palette.selection-foreground` is public palette API.
4.  Contribution welcome:
    yes.
    `CONTRIBUTING.md` on `master` (line 11) points AI coding assistants to `AGENTS.md`;
    no ban on assisted reports was found in `CONTRIBUTING.md`,
    `AGENTS.md`,
    `.github/pull_request_template.md`,
    or `.github/ISSUE_TEMPLATE/1-bug-report.yaml`.
    A maintainer answered the AI-labeled report
    [slint-ui/slint#13007](https://github.com/slint-ui/slint/issues/13007)
    with a status table marked "(this table was generated with LLM)".
    The draft discloses assistance.
5.  Likely to be fixed:
    yes,
    no contrary signal.
    Upstream fixed the previous fluent selection-ink problem in #3515 and changed the accent palette in 2026
    ([slint-ui/slint#11262](https://github.com/slint-ui/slint/pull/11262),
    [slint-ui/slint#11465](https://github.com/slint-ui/slint/pull/11465)).
6.  Prototype:
    yes.
    [`slint-fluent-dark-selection-foreground.patch`](slint-fluent-dark-selection-foreground.patch)
    changes `styling.slint`,
    `lineedit.slint`,
    and `spinbox.slint` against `v1.18.1`.
    It was applied in a disposable clone (`origin` `https://github.com/slint-ui/slint.git`,
    `HEAD` `372cf0ee5`,
    `git describe` `v1.18.1`)
    and,
    identically,
    to a copy of the `i-slint-compiler` 1.18.1 crate that the probe used through a Cargo `[patch]`.
    The probe output under "Verification" shows the dark selection ink turning from black to white
    in `Palette`,
    `LineEdit`,
    and `TextEdit`,
    with the light scheme unchanged.
    No case under `tests/screenshots` in the clone uses `LineEdit`,
    `TextEdit`,
    or `SpinBox`,
    and the text-selection screenshot cases set explicit colors,
    so no reference image is expected to change;
    Slint's own test suite was not run.

All six constraints hold.
Not filed:
this task did not authorize posting upstream,
so filing is the repository owner's decision.

~~~md
Title: Fluent dark scheme: selected text is black on the #0078D4 selection background

Labels: (bug report template default)

### Bug Description

In the fluent style's dark scheme, `Palette.selection-foreground` is `#000000` while
`Palette.selection-background` stays `#0078D4`, so selected text in `TextEdit`, and in anything
that uses the palette pair, is black on mid blue. `LineEdit` and `SpinBox` draw their selection
with `FluentPalette.accent-foreground`, which is also `#000000` in the dark scheme.
The light scheme uses white on the same fill.

`internal/compiler/widgets/fluent/styling.slint`:

    out property <brush> accent-background: dark-color-scheme ? accentify(#60CDFF) : accentify(#005FB8);
    out property <brush> accent-foreground: dark-color-scheme ? #000000 : #FFFFFF;
    out property <brush> selection-background: accentify(#0078D4);
    out property <brush> selection-foreground: dark-color-scheme ? #000000 : #FFFFFF;

The dark ink matches WinUI's `TextOnAccentFillColorPrimary`, which WinUI uses on its lighter
dark-theme accent fill (`AccentFillColorDefaultBrush` = `SystemAccentColorLight2`). For selected
text WinUI uses `TextOnAccentFillColorSelectedText`, `#FFFFFF` in both the `Default` and `Light`
dictionaries of `controls/dev/CommonStyles/Common_themeresources_any.xaml`, and its text host
returns `RGB(0xFF, 0xFF, 0xFF)` for `COLOR_HIGHLIGHTTEXT` outside high contrast
(`dxaml/xcp/core/native/text/Controls/TextServicesHost.cpp`, `TxGetSysColor`).

Contrast on `#0078D4`: black is WCAG 2 4.64 / APCA Lc 33.4, white is 4.53 / Lc -76.2.

Expected: white selected text in both schemes, as in WinUI.

### Reproducible Code

Rendered with the software renderer and `SLINT_STYLE=fluent`, counting pixels inside the selection.
The probe also imported Inter Variable and set it as `default-font-family`; the counts below were
measured that way.

```slint
import { LineEdit, TextEdit, Palette } from "std-widgets.slint";
export component Probe inherits Window {
    width: 400px;
    height: 200px;
    out property <brush> palette-ink: Palette.selection-foreground;
    public function scheme(dark: bool) {
        Palette.color-scheme = dark ? ColorScheme.dark : ColorScheme.light;
    }
    public function select-line() { line.focus(); line.select-all(); }
    public function select-text() { text.focus(); text.select-all(); }
    line := LineEdit { x: 10px; y: 10px; width: 380px; height: 56px; font-size: 32px; text: "WWWWWW"; }
    text := TextEdit { x: 10px; y: 80px; width: 380px; height: 110px; font-size: 32px; text: "WWWWWW"; }
}
```

Dark scheme, 1.18.1: `palette-ink` `#000000`; inside both selections 1172 near-black pixels and
no near-white ones. Light scheme: 1125 near-white pixels and no near-black ones.

### Suggested fix

```diff
--- a/internal/compiler/widgets/fluent/styling.slint
+++ b/internal/compiler/widgets/fluent/styling.slint
-    out property <brush> selection-foreground: dark-color-scheme ? #000000 : #FFFFFF;
+    out property <brush> selection-foreground: #FFFFFF;
--- a/internal/compiler/widgets/fluent/lineedit.slint
+++ b/internal/compiler/widgets/fluent/lineedit.slint
-                selection-foreground-color: FluentPalette.accent-foreground;
+                selection-foreground-color: FluentPalette.selection-foreground;
--- a/internal/compiler/widgets/fluent/spinbox.slint
+++ b/internal/compiler/widgets/fluent/spinbox.slint
-                    selection-foreground-color: FluentPalette.accent-foreground;
+                    selection-foreground-color: FluentPalette.selection-foreground;
```

With this change applied to the 1.18.1 compiler, the same probe reports `#FFFFFF` and 1125
near-white, 0 near-black pixels in both widgets in the dark scheme; the light scheme is unchanged.
`SpinBox` was not rendered. No `tests/screenshots` case uses these widgets.

### Environment Details

- Slint 1.18.1 (same lines on master at ad24fd95c, 2026-10-05)
- Style: fluent
- Rust, Linux container, software renderer

### Product Impact

A read-only code viewer built on Slint draws its own selected text and had to override
`Palette.selection-foreground` to stay legible in the dark scheme; its fluent `LineEdit` cannot be overridden.

This report was prepared with AI assistance. The probe was built and run against the released
1.18.1 crates and against the patched compiler, and the WinUI references were read from the
current microsoft-ui-xaml source.
~~~
