# Parley 0.11.1 `Cursor::from_point` returns positions inside a grapheme, between a letter and its combining marks or inside a ZWJ emoji sequence

## Symptom

Hit testing a Parley layout with `Cursor::from_point` can return a byte index that is not a grapheme boundary.
A caret placed there splits one user-perceived character:
it stands between `e` and a combining acute accent,
or between the members of an emoji sequence joined with U+200D ZERO WIDTH JOINER.

A standalone probe (see "Verification") swept `x` across three lines in 0.25 px steps
and recorded every index `Cursor::from_point` returned:

- `"cafe x"`:
  hits `{0, 1, 2, 3, 4, 5, 6}`,
  all grapheme boundaries.
- `"cafe\u{301} e\u{301}\u{302}x"`:
  hits `{0, 1, 2, 3, 4, 6, 7, 8, 10, 12, 13}`;
  `4`,
  `8`,
  and `10` are inside graphemes.
- `"a👩\u{200d}👩\u{200d}👧b"`:
  hits `{0, 1, 5, 8, 12, 15, 19, 20}`;
  `5`,
  `8`,
  `12`,
  and `15` are inside the family emoji.

The caret positions show how the inner positions arise.
The grapheme `e` plus U+0301 is 12 px wide and gets a midpoint at 6 px;
`e` plus U+0301 plus U+0302 gets thirds at 4 px;
the five-character emoji sequence,
36 px wide in the probe's font,
gets five parts of 7.2 px:

```text
# ~/temp/agent/upstream-prototype.m2ruMmIk/parley-probe.log
combining: caret x by byte 0@0.00 1@12.00 2@24.00 3@36.00 4@42.00 6@48.00 7@60.00 8@64.00 10@68.00 12@72.00 13@84.00
zwj-emoji: caret x by byte 0@0.00 1@12.00 5@19.20 8@26.40 12@33.60 15@40.80 19@48.00 20@60.00
```

In `package/ide/app`,
removing the snapping workaround from `src/shaped_row.rs` made
`every_pixel_hits_the_nearest_grapheme_boundary` fail with
`combining row 0: x 29.5 hit inside a grapheme at 4`:
a click at x 29.5 on `cafe\u{301}` put the caret between `e` and U+0301.

## Root cause

The shaper merges each grapheme into one glyph cluster,
and Parley then splits that cluster back into one equal-width piece per character.
Hit testing walks those pieces.
Code below is from the crates in the IDE build volume's cargo registry
(`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/index.crates.io-1949cf8c6b5b557f/`).

### HarfRust merges a grapheme into one cluster

Parley 0.11.1 fills the HarfRust buffer with one cluster value per character and sets no cluster level
(`parley-0.11.1/src/shape/mod.rs:440` to `470`).
HarfRust's default level is the grapheme-merging one:

```rust
// harfrust-0.12.0/src/lib.rs:116
impl Default for BufferClusterLevel {
    #[inline]
    fn default() -> Self {
        BufferClusterLevel::MonotoneGraphemes
    }
}
```

and it merges continuation characters,
such as combining marks and joined sequences,
into the cluster of their base:

```rust
// harfrust-0.12.0/src/hb/ot_shape.rs:792
fn form_clusters(buffer: &mut hb_buffer_t) {
    if buffer.scratch_flags & HB_BUFFER_SCRATCH_FLAG_HAS_CONTINUATIONS != 0 {
        foreach_grapheme!(buffer, start, end, {
            buffer.merge_grapheme_clusters(start, end);
        });
    }
}
```

### Parley splits a multi-character cluster into equal per-character parts

When it builds its own clusters from the shaped glyphs,
Parley counts the characters in each shaped cluster and divides the advance by that count,
treating every multi-character cluster as a ligature:

```rust
// parley-0.11.1/src/layout/data.rs:712
        if cluster_id != glyph_info.cluster {
            run_advance += cluster_advance;
            let num_components = num_components(glyph_info.cluster, cluster_id, last_cluster_id);
            cluster_advance /= num_components as f32;
            let is_newline = to_whitespace(cluster_start_char.1) == Whitespace::Newline;
            let cluster_type = if num_components > 1 {
                debug_assert!(!is_newline);
                ClusterType::LigatureStart
```

Each remaining character then gets a `LigatureComponent` cluster with the same divided advance
(`data.rs:750` to `771`;
the last cluster of a run takes the same path at `data.rs:813` to `854`).
The comment at `data.rs:688` to `692` gives the intended case,
a real ligature such as `fi`,
where a caret between the two letters is legitimate.
A base letter with a combining mark arrives on the same path,
because HarfRust has already merged it.

### Hit testing walks the parts and returns their edges

`Cluster::from_point` visits every cluster of the line,
components included,
and picks the one under `x` with no grapheme check
(`parley-0.11.1/src/layout/cluster.rs:96` to `114`).
`Cursor::from_point` then returns that cluster's start or end:

```rust
// parley-0.11.1/src/editing/cursor.rs:53
            } else {
                // We never want to position the cursor _after_ a hard
                // line since that cursor appears visually at the start
                // of the next line
                if is_leading || cluster.is_line_break() == Some(BreakReason::Explicit) {
                    (cluster.text_range().start, Affinity::Downstream)
                } else {
                    (cluster.text_range().end, Affinity::Upstream)
                }
            }
```

For a component cluster,
`text_range()` covers one character,
so the result can sit inside the grapheme.

Upstream describes the same state in the description of
[linebender/parley#715](https://github.com/linebender/parley/pull/715):
"On main,
`ClusterData` encodes just a single character,
and graphemes are seen as ligatures;
this is a somewhat buggy state".

## Verification

Versions under test:
`parley` 0.11.1 (checksum `22d2ff88bd3f7d68d1d9b09c7e6209f9a8e8c05088295140a2bcf2e9b17038c5`)
with `harfrust` 0.12.0 (checksum `c03d949a14aa089bbb282f7dd76a498a7f684428e4257202efc119ec010376f9`),
both from `package/ide/app/Cargo.lock`.

### Standalone probe

The probe is a binary crate with these two files plus `src/JetBrainsMono-Variable.ttf`,
copied from `package/ide/app/asset/font/`.
Seeding it with the IDE's `Cargo.lock` keeps the same versions.

```toml
# parley-probe/Cargo.toml
[package]
name = "parley-hit-probe"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
parley = "=0.11.1"
unicode-segmentation = "1"

[workspace]
```

```rust
// parley-probe/src/main.rs
use parley::fontique::Blob;
use parley::{Affinity, Cursor, FontContext, FontFamily, Layout, LayoutContext, StyleProperty};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

const FONT: &[u8] = include_bytes!("JetBrainsMono-Variable.ttf");

fn shape(fonts: &mut FontContext, layouts: &mut LayoutContext<()>, text: &str) -> Layout<()> {
    let mut builder = layouts.ranged_builder(fonts, text, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(Cow::Borrowed("JetBrains Mono"))));
    builder.push_default(StyleProperty::FontSize(20.0));
    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout
}

fn x_before(layout: &Layout<()>, byte: usize) -> f64 {
    Cursor::from_byte_index(layout, byte, Affinity::Downstream).geometry(layout, 1.0).x0
}

fn main() {
    let mut fonts = FontContext::new();
    fonts.collection.register_fonts(Blob::new(Arc::new(FONT)), None);
    let mut layouts = LayoutContext::new();

    // Hit testing: every byte index that Cursor::from_point returns while sweeping x across the line.
    for (name, text) in [
        ("plain", "cafe x"),
        ("combining", "cafe\u{301} e\u{301}\u{302}x"),
        ("zwj-emoji", "a\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}b"),
    ] {
        let layout = shape(&mut fonts, &mut layouts, text);
        let graphemes: BTreeSet<usize> = text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain([text.len()])
            .collect();
        let mut hits = BTreeSet::new();
        let mut x = 0.0_f32;
        while x <= layout.full_width() + 10.0 {
            hits.insert(Cursor::from_point(&layout, x, layout.height() / 2.0).index());
            x += 0.25;
        }
        let inside: Vec<usize> = hits.iter().copied().filter(|hit| !graphemes.contains(hit)).collect();
        println!("{name}: {text:?} grapheme boundaries {graphemes:?}");
        println!("{name}: from_point hits {hits:?}; hits inside a grapheme {inside:?}");
        let edges: Vec<String> = text
            .char_indices()
            .map(|(index, _)| index)
            .chain([text.len()])
            .map(|index| format!("{index}@{:.2}", x_before(&layout, index)))
            .collect();
        println!("{name}: caret x by byte {}", edges.join(" "));
    }

    // Tabs: the advance of one tab after different amounts of text; stops would make the end positions agree.
    for text in ["\tx", "a\tx", "ab\tx", "abc\tx"] {
        let layout = shape(&mut fonts, &mut layouts, text);
        let tab = text.find('\t').expect("tab");
        let start = x_before(&layout, tab);
        let end = x_before(&layout, tab + 1);
        println!("tab in {text:?}: starts {start:.2}, ends {end:.2}, advance {:.2}", end - start);
    }
    let space = shape(&mut fonts, &mut layouts, " x");
    println!("space advance {:.2}", x_before(&space, 1) - x_before(&space, 0));
}
```

It ran offline in the IDE build image with a private copy of the IDE cargo registry
(`index` and `cache` copied to `cargo/registry/`):

```sh
# run from the directory that holds parley-probe/ and cargo/
podman run --rm --network=none --memory=2g --cpus=2 --pids-limit=512 --security-opt label=disable --volume "${PWD}:/proto" --volume "${PWD}/cargo:/cargo" --workdir /proto/parley-probe --env CARGO_HOME=/cargo --env CARGO_BUILD_JOBS=2 localhost/monochromatic/ide cargo run --offline
```

The 2026-10-05 run is in `~/temp/agent/upstream-prototype.m2ruMmIk/parley-probe.log`.
The tab lines belong to [`parley-tab-stops.md`](parley-tab-stops.md).

No color emoji font was registered,
so the emoji sequence was drawn with whatever glyphs the probe's font and fallback provided;
the equal split does not depend on what the glyphs look like.

### Works cleanly

- Text in which every grapheme is one character:
  `"cafe x"` returned only grapheme boundaries.
- Separate characters inside a real ligature:
  each `=` in `===` is its own grapheme,
  so a caret between them is correct;
  the IDE's `ligature` fixture row passes `every_pixel_hits_the_nearest_grapheme_boundary`.

### Fails

- Base letter plus combining marks:
  indices `4`,
  `8`,
  and `10` in the probe;
  index `4` at x 29.5 in the IDE guard run.
- Emoji joined with ZWJ:
  indices `5`,
  `8`,
  `12`,
  and `15` in the probe.
  The IDE's `emoji` fixture row was not reached in the guard run,
  because the test stops at the first failing row.

### IDE guard control

`package/ide/app/bin/inspect-source-guards.mjs`
(now `package/ide/app/bin/inspect-source-guards.ts`,
renamed 2026-10-22),
guard `grapheme-snap`,
replaces the snapped return in `ShapedRow::hit` with the raw Parley index in a disposable copy.
The run recorded here used the files of commit `8dab09180`;
logs are in `~/temp/agent/ide-source-guard-bX6gco/`
(`grapheme-snap-baseline.log` and `-restored.log` pass,
`grapheme-snap-removed.log` fails at `tests/reading_boundaries.rs:125`).

## Verified workarounds

Snap Parley's proposal to the nearer grapheme boundary computed independently of the shaper.
The IDE takes the boundaries from Helix's grapheme walk (`src/text_projection.rs:43` to `59`)
and snaps in `src/shaped_row.rs:91` to `111`
(`main` at `f9d08ae79`):

```rust
// package/ide/app/src/shaped_row.rs:91
    fn snapped(&self, local: usize, x: f32) -> usize {
        let mut before = 0;
        let mut after = self.source_len();
        // The stops ascend, so the last one at or before `local` and the first at or after it enclose it.
        for stop in &self.projection.stops {
            if *stop <= local {
                before = *stop;
            }
            if *stop >= local {
                after = *stop;
                break;
            }
        }
        if before == after {
            return local;
        }
        if (x - self.physical_x(before)).abs() <= (self.physical_x(after) - x).abs() {
            return before;
        }
        return after;
    }
```

`ShapedRow::hit` calls it on every `Cursor::from_point` result (`shaped_row.rs:129` to `132`).

Tradeoffs:

- Two extra caret-geometry queries when a hit lands inside a grapheme;
  none when Parley already returned a boundary.
- The choice uses the grapheme's two edges,
  so the boundary flips at the grapheme's visual midpoint.
  Inside a merged cluster that also contains a real ligature across graphemes,
  the edges still come from Parley's equal split.
- The boundaries are Helix's.
  Where Helix and the shaper disagree about what forms a grapheme,
  the caret follows Helix,
  which is also what keyboard movement uses.

After an upgrade to a Parley release that contains linebender/parley#715,
`from_point` should return grapheme edges and `snapped` becomes a no-op:
`before == after` for a proposal that is already a stop.
That release also changes the shaping and spacing code the IDE's tab workaround depends on;
see [`parley-tab-stops.md`](parley-tab-stops.md).

## What does not work

- Rounding to the nearest character boundary:
  that is what Parley already returns.
- Using `Cluster::is_ligature_start` and `is_ligature_continuation` to find the grapheme:
  in 0.11.1 those flags mark every multi-character shaped cluster,
  real ligatures included,
  so they cannot separate `===` (three graphemes) from `e` plus U+0301 (one).
  This was read from `data.rs:717` to `719`,
  not tried in code.

## Upstream filing decision

`.out-of-scope/` was inspected and has no Parley entry.

Candidate fix applicability:
linebender/parley#715
("Differentiate shaped clusters and graphemes,
introduce 'atoms'",
merged 2026-08-05 as `f132cbd25`)
makes `Cluster` span a full grapheme.
The `Unreleased` section of the changelog on `main` at `dd7e0b049` says
"`Cluster` now spans a full grapheme cluster instead of a single character"
and "Shaped clusters' advances are split evenly over the grapheme clusters they overlap."
`gh api repos/linebender/parley/compare/v0.11.1...f132cbd258be9f315bf45036a6683d926b844d68`
reports `diverged`,
`ahead_by` 51,
`behind_by` 1,
so 0.11.1 does not contain it,
and the installed `data.rs:714` to `715` still divides per character.
The behavior matches the installed source,
and upstream has already changed it on `main`.
The fix there was not built or run here.

Existing report:
[linebender/parley#694](https://github.com/linebender/parley/issues/694)
("Cursor motion steps are codepoints,
not grapheme clusters",
open)
already names "clicking to select a cursor position" among the affected operations.

Nothing to add,
and nothing is posted:

1.  Upstream's fault:
    yes,
    and upstream says so (the #715 description).
2.  Fixable:
    yes;
    #715 is the fix.
3.  Supported use case:
    yes;
    `Cursor::from_point` exists for caret placement.
4.  Contribution welcome:
    no,
    for this artifact.
    Linebender's LLM policy (`content/wiki/llm_policy.md` in `linebender/linebender.github.io`,
    last changed in `362ba9d86`)
    says:
    "In discussion spaces like Github comments and the Zulip server,
    please avoid posting AI-generated analyses,
    even if you vetted them."
    A comment drafted here would be exactly that.
5.  Likely to be fixed:
    already fixed on `main`.
6.  Prototype:
    not needed;
    upstream's merged change is the fix.

The only content not already in #694 or #715 is the probe's equal-split measurement,
which #715's description already states in prose.
