# Material Design 3 Design System

A design system built from Google's official **Material Design 3** documentation site (m3.material.io),
 including the "M3 Expressive" update.
 This is a personal-use design-system recreation for prototyping — **any product built from it must credit Google's
 Material Design 3** as the source design language.
 Not affiliated with or endorsed by Google.

**Source material:**
 a full offline (SingleFile) capture of `m3.material.io` — styles,
 foundations,
 component specs,
 and images — supplemented with exact token values read from Google's open-source `material-components/material-web`
 GitHub repo (`tokens/versions/latest/sass/`),
 which is the canonical machine-readable source for M3's color roles,
 type scale,
 shape,
 elevation,
 motion,
 and state tokens (design system version 34.0.21,
 "M3 Expressive").

## What Material Design 3 is

Material Design 3 is Google's cross-platform design system for building adaptive,
 accessible UI across Android,
 iOS,
 Flutter,
 and web.
 It's organized as **Foundations** (color,
 typography,
 shape,
 elevation,
 motion,
 spacing,
 icons) and **Components** (buttons,
 cards,
 navigation,
 forms,
 feedback — ~35 families),
 all driven by a token system (`md.ref.*` raw values → `md.sys.*` semantic roles → `md.comp.*` per-component tokens).
 "M3 Expressive" (2025) is the current evolution:
 bolder shapes,
 springier motion,
 and a wider size ladder on components like buttons and FABs — this system uses the Expressive tokens throughout.

## Components represented

There is one product here:
 **the Material Design 3 system itself** — a UI kit (`ui_kits/m3-docs-site/`) recreates the m3.material.io
 documentation site's own chrome (nav rail,
 hero cards,
 styles/components index) as a demonstration of the tokens and components in use.

## Content fundamentals

- **Voice:**
   second person,
   instructional,
   calm — "Use typography to make content readable,"
   "Build beautiful,
   usable products."
   Imperative verbs open most headings.
- **Casing:**
   sentence case everywhere — headings,
   buttons,
   nav labels ("Get started,"
   not "Get Started").
- **Tone:**
   confident but not salesy;
   explains *why* a rule exists ("Display styles are reserved for short,
   important text... because they work best on large screens").
- **Emoji:**
   never used.
   Iconography (Material Symbols) carries visual emphasis instead.
- **Length:**
   short declarative sentences;
   docs favor one idea per sentence over long compound explanations.

## Visual foundations

- **Color:**
   a 6-role system (primary/secondary/tertiary + error,
   each with a "container" and "on-" pair) generated from a single seed color via HCT tonal palettes,
   plus a neutral surface ramp (`surface-container-lowest` → `highest`,
   5 steps) that replaces flat elevation shadows in most cases.
   Light and dark schemes swap tone indices on the same roles (e.g. primary = tone 40 light,
   tone 80 dark).
- **Type:**
   Roboto (`md.ref.typeface.brand` and `.plain`,
   both Roboto in the default theme) across 5 roles × 3 sizes (display/headline/title/body/label × large/medium/small)
   = 15 styles,
   each with an "emphasized" (heavier-weight) counterpart new in M3 Expressive.
- **Spacing:**
   an 8dp grid (`space100`=8 … `space900`=72),
   plus window-size-class edge margins (16dp compact,
   24dp medium/expanded).
- **Backgrounds:**
   flat tonal surfaces,
   no gradients,
   no photographic full-bleed heroes on the docs site itself (product screenshots are the imagery);
   occasional soft squiggle-line dividers between sections.
- **Animation:**
   two easing families — "standard" (functional,
   quick) and "emphasized" (spring-like,
   used for shape morphs and selection).
   M3 Expressive adds physical spring tokens (damping/stiffness) for FABs,
   toggles,
   and menus.
   No bounce/elastic overshoot outside springs.
- **Hover / press states:**
   a single mechanism — a **state layer**:
   a scrim of the foreground ("on-") color over the container at 8% opacity (hover),
   10% (press/focus),
   16% (drag).
   Never a lightened/darkened background color swap.
- **Borders:**
   thin 1dp `outline`/`outline-variant` hairlines on outlined components;
   no double borders,
   no colored borders except for error/warning semantics.
- **Shadows / elevation:**
   6 levels (0–5),
   each a two-part umbra/penumbra shadow tinted with `md.sys.color.shadow` (black).
   Increasingly,
   elevation is signaled by *tonal surface* (a lighter/darker fill) rather than shadow depth — shadows are reserved for
   genuinely floating elements (FAB,
   menu,
   dialog).
- **Corner radius:**
   a 10-step scale from `none` (0) to `full` (9999px/pill),
   stepping 4/8/12/16/20/28/32/48.
   Buttons and chips are typically `full`;
   cards are `medium` (12dp);
   dialogs and sheets are `extra-large` (28dp).
- **Cards:**
   three variants — elevated (level-1 shadow,
   `surface-container-low`),
   filled (flat,
   `surface-container-highest`),
   outlined (1dp `outline-variant` border,
   no shadow) — always 12dp corner.
- **Transparency/blur:**
   scrims (32% black) behind modal dialogs/sheets;
   blur is not a default M3 device — clarity favors tonal opacity over backdrop-filter.
- **Imagery:**
   product screenshots and UI mockups only,
   warm-neutral color grading,
   no stock photography or illustration style baked into the system itself.

## Iconography

- **System:**
   Material Symbols — a single variable icon font (axes:
   FILL 0–1,
   weight 300–600,
   GRAD -50–200,
   optical size 20–48).
   One font,
   thousands of glyphs,
   referenced by name ("search",
   "home",
   "more_vert") rather than per-icon SVGs.
- **Usage:**
   ligature text inside a `<span class="md-symbol">` (or the `Icon` component here).
   Filled (`FILL 1`) marks the *active* state — e.g. the selected nav-bar destination — outlined is default/inactive.
- **Emoji:**
   never used as UI icons.
- **Assets copied in:**
   `assets/fonts/google-symbols.woff2` (the variable icon font,
   self-hosted from the captured site) and the M3 favicon mark (`assets/logo/m3-mark.svg`).
- **No custom SVG icon set** exists in the source;
   do not hand-draw new glyphs — extend the Material Symbols name list instead.

## Assets

- `assets/logo/m3-mark.svg` — the Material Design 3 favicon/wordmark glyph (the only "logo" the source provides;
   M3 has no separate brand logotype beyond this mark and the plain wordmark "Material Design 3").
- `assets/fonts/` — self-hosted Material Symbols variable font + Google Sans / Google Sans Text (used only by the
docs-site UI kit recreation;
   product UI uses Roboto,
   loaded from Google Fonts in `tokens/fonts.css`).
- `assets/img/` — hero and section imagery captured from m3.material.io (home page cards,
   color/typography section illustrations).

**No separate brand logo exists beyond the M3 mark** — Material Design 3 is a design language,
 not a consumer brand with a wordmark identity.

## Tokens

- `tokens/palette.css` — `md.ref.palette`:
   every tonal step (0–100) for primary/secondary/tertiary/error/neutral/neutral-variant plus 11 static hues (red,
   orange,
   yellow,
   green,
   cyan,
   blue,
   blue-variant,
   purple,
   pink,
   grey,
   grey-variant).
- `tokens/color.css` — `md.sys.color`:
   the ~40 semantic roles,
   light scheme under `:root`,
   dark scheme under `[data-theme="dark"]` (also respects `prefers-color-scheme`).
- `tokens/typography.css` — `md.sys.typescale`:
   15 styles × standard + emphasized,
   plus `.md-typescale-*` utility classes.
- `tokens/shape.css` — `md.sys.shape`:
   the 10-step corner radius scale + directional variants.
- `tokens/elevation.css` — `md.sys.elevation`:
   6 levels as both dp values and ready-to-use `box-shadow` recipes.
- `tokens/motion.css` — `md.sys.motion`:
   easing curves,
   durations,
   and M3 Expressive spring constants.
- `tokens/spacing.css` — `md.sys.spacing`:
   the 8dp space scale + layout margins.
- `tokens/state.css` — `md.sys.state`:
   state-layer opacities (hover/focus/press/drag/disabled).
- `tokens/fonts.css` — `@font-face` declarations (Material Symbols self-hosted;
   Roboto via Google Fonts CDN;
   Google Sans/Text self-hosted for the docs-site recreation only).
- `tokens/base.css` — the `.md-surface-root`,
   link states,
   and `.md-state-layer` hover/press mechanism shared by every component.

All values are copied verbatim from `material-components/material-web@main` (`tokens/versions/latest/sass/`) — never
rounded or approximated.

## Components (31)

- **Actions:**
   `Button`,
   `IconButton`,
   `Fab`,
   `ExtendedFab`,
   `FabMenu`,
   `SplitButton`,
   `ButtonGroup`,
   `SegmentedButton`
- **Forms:**
   `TextField`,
   `Checkbox`,
   `RadioButton`,
   `Switch`,
   `Slider`,
   `SearchBar`
- **Containment:**
   `Card`,
   `Chip`,
   `Badge`,
   `BottomSheet`,
   `SideSheet`
- **Feedback:**
   `ProgressIndicator`,
   `LoadingIndicator`,
   `Snackbar`,
   `Tooltip`,
   `Dialog`
- **Navigation:**
   `Tabs`,
   `NavigationBar`,
   `NavigationRail`,
   `NavigationDrawer`,
   `TopAppBar`,
   `Menu`,
   `Toolbar`
- **Layout:**
   `ListItem`,
   `Divider`
- **Core:**
   `Icon`

**Intentional additions:**
 `Icon` (a thin wrapper around the Material Symbols glyph font — needed since M3 icons aren't a per-icon SVG set)
 and `Toolbar` (a simplified version of M3's floating/docked toolbar spec).
 `Carousel`,
 `DatePicker`,
 and `TimePicker` from the full M3 catalog were **not built** — they are complex,
 stateful components better hand-rolled per product than approximated here;
 flag if you need them.

## UI kit

`ui_kits/m3-docs-site/` — an interactive recreation of m3.material.io's own site chrome:
 a left navigation rail,
 a Home screen (hero + topic cards),
 a Styles index,
 and a Components index.
 Demonstrates the tokens and primitives above in a real layout.
 Open `ui_kits/m3-docs-site/index.html`.

## Index

```text
styles.css                     entry point — @imports every token file
tokens/                         palette, color, typography, shape, elevation, motion, spacing, state, fonts, base
guidelines/                     foundation specimen cards (Colors, Type, Spacing, Brand groups)
components/core/                Icon
components/actions/             Button, IconButton, Fab, ExtendedFab, FabMenu, SplitButton, ButtonGroup, SegmentedButton
components/forms/                TextField, Checkbox, RadioButton, Switch, Slider, SearchBar
components/containment/         Card, Chip, Badge, BottomSheet, SideSheet
components/feedback/             ProgressIndicator, LoadingIndicator, Snackbar, Tooltip, Dialog
components/navigation/          Tabs, NavigationBar, NavigationRail, NavigationDrawer, TopAppBar, Menu, Toolbar
components/layout/               ListItem, Divider
ui_kits/m3-docs-site/           interactive recreation of the M3 docs site
assets/                         logo mark, fonts, imagery
SKILL.md                        Claude Code / Agent Skills compatible entry point
```

## Caveats

- Fonts:
   **Roboto Flex** (product UI,
   `md.ref.typeface.brand`/`.plain`) is self-hosted at `assets/fonts/RobotoFlex.ttf`,
   copied from Google's official `google/fonts` GitHub repo (OFL-licensed) — nothing is loaded live.
   **Google Sans / Google Sans Text** are self-hosted from the capture but are proprietary to Google;
   they're used *only* in the docs-site UI kit recreation to match that site's own chrome,
   never presented as your own typeface.
- `Carousel`,
   `DatePicker`,
   `TimePicker` are documented in the M3 catalog but not built here (see "Intentional additions" above).
- The static reference palette (11 extra hues beyond primary/secondary/tertiary) is included in `tokens/palette.css`
but has no semantic role mapping — use it only for data-viz/illustration accents,
   not UI chrome.
