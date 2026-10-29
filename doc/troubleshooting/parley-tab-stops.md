# Parley 0.11.1 has no tab stops: a tab gets one fixed advance wherever it starts, so indentation needs consumer-side widening

## Symptom

Parley lays out a tab character as one glyph with a fixed advance.
The advance does not depend on where the tab starts,
so tabs after text of different widths do not line up,
and there is no style property to ask for stops.

A standalone probe shaped `x` after a tab that starts at four different positions,
in JetBrains Mono at 20 px:

```text
# ~/temp/agent/upstream-prototype.m2ruMmIk/parley-probe.log
tab in "\tx": starts 0.00, ends 12.00, advance 12.00
tab in "a\tx": starts 12.00, ends 24.00, advance 12.00
tab in "ab\tx": starts 24.00, ends 36.00, advance 12.00
tab in "abc\tx": starts 36.00, ends 48.00, advance 12.00
space advance 12.00
```

With stops every two space advances (24 px),
the four tabs would end at 24,
24,
48,
and 48.

The advance itself varies by setup.
[linebender/parley#302](https://github.com/linebender/parley/issues/302) reported in 2025,
before Parley moved its shaping to HarfRust,
that a tab had no width at all.
In this probe it took one space advance;
whether that comes from JetBrains Mono's character map or from the shaper was not determined.

## Root cause

Parley classifies the character and then never looks at the class again.
Code below is from `parley-0.11.1` in the IDE build volume's cargo registry
(`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/index.crates.io-1949cf8c6b5b557f/`).

```rust
// parley-0.11.1/src/layout/data.rs:104
const fn to_whitespace(c: char) -> Whitespace {
    const LINE_SEPARATOR: char = '\u{2028}';
    const PARAGRAPH_SEPARATOR: char = '\u{2029}';

    match c {
        ' ' => Whitespace::Space,
        '\t' => Whitespace::Tab,
```

`rg --word-regexp Tab parley-0.11.1/src` finds only that line and the enum declaration
(`src/analysis/cluster.rs:70`).
No layout,
line-breaking,
or spacing code matches on `Whitespace::Tab`,
and `rg --ignore-case '\btab' parley-0.11.1/src` finds no tab-size style:
the `StyleProperty` variants (`src/style/mod.rs:75` to `119`) cover fonts,
locale,
brush,
decoration,
line height,
word and letter spacing,
and line breaking only.
The tab's advance is therefore whatever the shaper gave its glyph.

The same holds on Parley's `main`:
its changelog at `dd7e0b049` mentions no tab support,
and linebender/parley#302 is still open.

### What the workaround relies on

Letter spacing is added to a cluster's advance after shaping:

```rust
// parley-0.11.1/src/layout/data.rs:525
    pub(crate) fn finish(&mut self) {
        for run in &self.runs {
            let word = run.word_spacing;
            let letter = run.letter_spacing;
            if nearly_zero(word) && nearly_zero(letter) {
                continue;
            }
            let clusters = &mut self.clusters[run.cluster_range.clone()];
            for cluster in clusters {
                let mut spacing = letter;
                if !nearly_zero(word) && cluster.info.whitespace().is_space_or_nbsp() {
                    spacing += word;
                }
                if !nearly_zero(spacing) {
                    cluster.advance += spacing;
```

and it is given in logical units and multiplied by the layout's scale
(`src/resolve/mod.rs:202`,
`letter_spacing: raw_style.letter_spacing * scale`).

## Verification

Versions under test:
`parley` 0.11.1 (checksum `22d2ff88bd3f7d68d1d9b09c7e6209f9a8e8c05088295140a2bcf2e9b17038c5`)
with `harfrust` 0.12.0,
from `package/ide/app/Cargo.lock`.

### Standalone probe

The probe that produced the symptom lines is listed in full,
with its manifest and run command,
under "Standalone probe" in
[`parley-cursor-from-point-inside-grapheme.md`](parley-cursor-from-point-inside-grapheme.md);
its last loop measures the tabs.
Output:
`~/temp/agent/upstream-prototype.m2ruMmIk/parley-probe.log` (2026-10-05).

### IDE guard controls

`package/ide/app/bin/inspect-source-guards.mjs`
(now `package/ide/app/bin/inspect-source-guards.ts`,
renamed 2026-10-22)
removes one tab guard at a time in a disposable copy
and requires the named test to fail.
The run recorded here used the files of commit `8dab09180`;
logs are in `~/temp/agent/ide-source-guard-bX6gco/`.

```sh
# package/ide/app/mise.toml task inspect:source-guards; arguments: cache, cargo home copy, guard names
mise run //package/ide/app:inspect:source-guards "${HOME}/temp/agent/<disposable-target>" "${HOME}/temp/agent/<cargo-home-copy>" tab-half-space,tab-unusable-space,tab-scale,tab-leading-shortcut
```

### Works cleanly

With the workaround in place,
each guard's baseline and restored runs pass:
`tabs_end_on_multiples_of_two_measured_space_advances`,
`tab_geometry_is_the_same_at_every_scale`,
`tab_narrower_than_half_a_space_uses_the_following_stop`,
and `unusable_space_advance_keeps_one_space_width`.

### Fails

- Raw Parley:
  the probe's tabs end at 12,
  24,
  36,
  and 48 instead of 24,
  24,
  48,
  and 48.
- Correction in physical instead of logical pixels (`tab-scale`):
  `"\tx" at 1: 20.25 at scale 1.25, 18 at scale 1`,
  because Parley multiplies letter spacing by the scale.
- Every tab treated as leading,
  so no measuring pass (`tab-leading-shortcut`):
  `"a\tx": the tab ends at 27, expected 18`.
- No half-space rule (`tab-half-space`):
  `a sliver tab was kept instead of the next stop`,
  left `3.0`,
  right `21.0`.
- No guard for an unusable space advance (`tab-unusable-space`):
  left `NaN`,
  right `0.0`.

## Verified workarounds

Shape each tab as a space and widen that one space with letter spacing until it ends on a stop.
`package/ide/app` does this in four places (`main` at `f9d08ae79`):

1.  `src/text_projection.rs:79` to `82` replaces each source tab with one space in the display text
    and records its byte,
    keeping the source/display maps so copying returns the tab.
2.  `src/tab_stop.rs:36` to `49` computes the width of a tab that starts at a pixel position:
    stops lie at multiples of two measured space advances,
    and a tab narrower than half a space runs on to the next stop
    (the CSS Text Level 3 rule behind `tab-size: 2`).
    `tab_spacings` (`tab_stop.rs:67` to `90`) accumulates the corrections left to right,
    because widening one tab moves the later ones.
3.  `src/tab_layout.rs:51` to `101` shapes a line once without corrections to measure where each tab starts,
    unless every tab is leading,
    then shapes it again with the corrections,
    each divided by the scale (`tab_layout.rs:98`).
4.  `src/shaped_text.rs:159` to `161` applies each correction as `StyleProperty::LetterSpacing`
    on the stand-in space's single byte:

```rust
// package/ide/app/src/shaped_text.rs:159
        // The stand-in space of a tab is one byte; spacing on it changes that one advance only.
        for (byte, extra) in spacings {
            builder.push(StyleProperty::LetterSpacing(*extra), *byte..*byte + 1);
        }
```

The equivalent loop moved on 2026-10-15 into `TextShaper::build_line` in `src/shaped_text/layout.rs`;
step 4 and the path comment of the snippet name `src/shaped_text.rs` as it was when this entry was written.

Tradeoffs:

- A line whose tabs follow other text is shaped twice.
- Stops are pixel positions from the line's start,
  not character columns,
  so a tab after a CJK glyph ends where a tab after Latin text ends;
  a column-counting reader would place it differently.
- The display text contains a space where the source has a tab.
  Anything that reads glyphs rather than the source maps,
  such as a future visible-whitespace mode,
  would see a space.
- The mechanism depends on how Parley applies letter spacing.
  Parley's `main` changes that code:
  linebender/parley#738 ("Improve spacing and justification model,
  stop mutating advances in-place",
  merged 2026-08-29) replaces the in-place `finish()` with a lazy calculation that spaces graphemes on their right,
  and linebender/parley#731 (merged 2026-08-06) stops optional ligatures from forming where letter spacing applies.
  The tab guards in `bin/inspect-source-guards.mjs`
  (now `bin/inspect-source-guards.ts`,
  renamed 2026-10-22)
  should be rerun after the upgrade.

## What does not work

- Leaving the tab in the shaped text:
  its advance is the font's or shaper's,
  fixed and position-independent,
  as the probe shows.
- Expanding a tab to a fixed number of space characters:
  that counts columns,
  so a tab after a CJK glyph,
  whose fallback advance is not two Latin cells,
  would not end on the same pixel as a tab after Latin text.
  The IDE's earlier projection expanded tabs with a four-column terminal-width calculation
  (`doc/handover/slint-ide-0x.md`,
  "Workspace model and remaining verification details");
  it was replaced,
  not measured here.
- `WordSpacing` on the stand-in space:
  it also widens only spaces,
  but it applies to every space in the run,
  so it cannot target one tab.
  This is read from `data.rs:535` to `537`;
  it was not tried.

## Upstream filing decision

`.out-of-scope/` was inspected and has no Parley entry.

Existing report:
[linebender/parley#302](https://github.com/linebender/parley/issues/302)
("Display tab character",
open since 2025-03-19).
Its thread covers the missing width,
asks for a tab width relative to a space,
and on 2026-07-28 a maintainer pointed to CSS Text's tab-stop rules as the requirement.
It is listed in the egui and Blitz tracking issues
([#386](https://github.com/linebender/parley/issues/386) and [#208](https://github.com/linebender/parley/issues/208)).
Searches on 2026-10-05 with `gh search issues --repo linebender/parley tab` and `tab stops`,
and the same with `gh search prs`,
found no implementation in progress.

The consumer-side workaround is the only thing not already in that thread.
Nothing is posted:

1.  Upstream's fault:
    a capability gap,
    acknowledged in #302,
    not a defect in existing behavior.
2.  Fixable:
    yes;
    the layout code already classifies tabs.
3.  Supported use case:
    yes,
    by the tracking issues.
4.  Contribution welcome:
    no,
    for this artifact.
    Linebender's LLM policy (`content/wiki/llm_policy.md` in `linebender/linebender.github.io`,
    last changed in `362ba9d86`) asks contributors to
    "avoid posting AI-generated analyses,
    even if you vetted them" in GitHub comments.
5.  Likely to be fixed:
    no contrary signal;
    the issue is open and tracked.
6.  Prototype:
    none.
    A real fix would implement CSS tab stops inside Parley's line layout,
    which this work did not attempt;
    the IDE's workaround is consumer-side and not a patch to Parley.

Do not post as-is.
This draft records the additive content only;
under the policy in constraint 4 a person would have to write any comment themselves.

~~~md
A consumer-side workaround while tab stops are missing, in case it helps others:
shape each tab as a single space, measure where each stand-in space starts (one extra layout pass
when tabs follow other text), and push `StyleProperty::LetterSpacing(extra / scale)` on that one byte
so the space ends on the next stop. Letter spacing is multiplied by the layout scale
(`resolve/mod.rs:202` in 0.11.1), so the correction has to be given in logical units.
This depends on the 0.11 `finish()` letter-spacing model, which #738 replaces.
~~~
