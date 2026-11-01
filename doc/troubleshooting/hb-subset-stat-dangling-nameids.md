# hb-subset-wasm 0.4.0 keeps STAT while pruning the name records it references, so Firefox's font sanitizer discards STAT

Subsetting a variable font with `hb-subset-wasm` default options copies
the `STAT` table through unchanged while pruning most name records.
`STAT`'s axis records and axis-value entries then point at name IDs
that no longer exist in the subset's `name` table.
 Firefox's
downloadable-font sanitizer (OTS) flags the first dangling reference,
discards the whole `STAT` table,
 and logs console errors on every load
of a page using the font.
 The font still renders;
 the failure is
console noise plus a subset that is no longer a conformant variable
font.

## Symptom

Loading a page whose `@font-face` uses the subsetted font logs,
 in
Firefox (151.0 under playwright 1.61.1,
 and user-observed in desktop
Firefox),
 two error-level console messages:

```txt
downloadable font: STAT: Invalid nameID: 298
  (font-family: "Inter" style:normal weight:100..900 stretch:100 src index:0)
  source: data:font/woff2;base64,d09GMg…
downloadable font: Table discarded
  (font-family: "Inter" style:normal weight:100..900 stretch:100 src index:0)
  source: data:font/woff2;base64,d09GMg…
```

The reported ID (`298` for Inter 4.001) is whichever `STAT`-referenced
name ID the sanitizer hits first among those the subsetter dropped.
Chromium loads the same font without logging anything.
 Text renders
correctly in both engines,
 including the variable weight axis.

Trigger:
 any `hb-subset-wasm` `subset()` call on a font with a `STAT`
table whose axis or axis-value name IDs fall outside the set the
subsetter retains,
 which for Inter is all of `STAT`'s IDs above 275.

## Root cause

Three components interact.
 The defect is owned by hb-subset-wasm's
build configuration,
 with a latent inconsistency in HarfBuzz that
makes such a configuration possible;
 OTS behaves correctly.
 Source
pins:
 `kyosuke/hb-subset-wasm@82fe83faa429fd4c1b9af1b3d99829d67fde50ba`
(the npm 0.4.0 `gitHead`),
 `harfbuzz/harfbuzz@3ef8709829a5884517ad91a97b32b9435b2f20d1`
(tag `10.4.0`,
 the version the wrapper bundles as its `deps/harfbuzz`
submodule,
 per its `.gitmodules` and `README.md:177-179`) plus
harfbuzz HEAD `4509695c7873a8ff9320613bb8cdd65426436970`,
 and
`khaledhosny/ots@2c594bcace426510c317a9bb9fb9acdfb71ea00a`.

An earlier reading in this repo was wrong and is recorded here so it
does not get re-derived:
 the wc README briefly claimed "the
`hb-subset-wasm` step leaves a STAT entry pointing at a dropped name
record" as though hb-subset inherently drops STAT-referenced names.
It does not.
 Stock HarfBuzz has collected STAT's name IDs during
subsetting since 7.2.0 (that older default-build bug was
[harfbuzz#4162][],
 fixed by [harfbuzz PR #4168][] the next day),
 and a
control run with a HarfBuzz wasm build that lacks the wrapper's flag
(harfbuzzjs via subset-font 2.5.0) keeps every STAT-referenced name on
the same input.
 The wrapper's build flag is the cause.

### hb-subset-wasm compiles HarfBuzz with `HB_NO_STYLE`

`scripts/build-wasm.sh:85` at 82fe83f,
 inside the CFLAGS array:

```bash
  -DHB_NO_STYLE
```

(`HB_NO_VAR` is notably absent,
 so `fvar` name collection stays
enabled,
 which is why IDs 256 through 275 survive.)
 The wrapper never
touches name IDs otherwise:
 `wasm/wrapper.c` configures only flags,
unicode/glyph sets,
 passthrough/drop table tags,
 axis pinning,
 and
layout features;
 a repo-wide search for
`name_id|nameid|NAME_ID|HB_SUBSET_SETS_NAME` has zero matches.

### HarfBuzz under `HB_NO_STYLE`: closure compiled out, STAT still emitted

With `HB_NO_STYLE`,
 the STAT table accelerator does not exist;
harfbuzz@3ef8709 `src/hb-ot-face-table-list.hh:68-70`:

```cpp
#ifndef HB_NO_STYLE
HB_OT_CORE_TABLE (OT, STAT)
#endif
```

so the subset plan's name-ID closure must skip STAT;
`src/hb-subset-plan.cc:846-856`:

```cpp
static void
_nameid_closure (hb_subset_plan_t* plan,
		 hb_set_t* drop_tables)
{
#ifndef HB_NO_STYLE
  plan->source->table.STAT->collect_name_ids (&plan->user_axes_location, &plan->name_ids);
#endif
#ifndef HB_NO_VAR
  if (!plan->all_axes_pinned)
    plan->source->table.fvar->collect_name_ids (&plan->user_axes_location, &plan->axes_old_index_tag_map, &plan->name_ids);
#endif
```

The default retained set is tiny;
 `src/hb-subset-input.cc:43-44`:

```cpp
  hb_set_add_range (sets.name_ids, 0, 6);
  hb_set_add (sets.name_languages, 0x0409);
```

and the name subsetter filters records by that plan set;
`src/OT/name/name.hh:387-399`:

```cpp
    + nameRecordZ.as_array (count)
    | hb_filter (c->plan->name_ids, &NameRecord::nameID)
    | hb_filter (c->plan->name_languages, &NameRecord::languageID)
```

But the STAT dispatch in the subsetter is **not** guarded by
`HB_NO_STYLE`,
 and passes the table through verbatim when no axes are
being pinned;
 `src/hb-subset.cc:539-541`:

```cpp
  case HB_OT_TAG_STAT:
    if (!plan->user_axes_location.is_empty ()) return _subset<const OT::STAT> (plan, buf);
    else return _passthrough (plan, tag);
```

(the local harness confirms the 220-byte STAT is byte-identical
before and after subsetting).
 Net effect in an `HB_NO_STYLE` build:
`name` keeps 0-6 plus `fvar`'s references,
 STAT keeps all of its own,
and every STAT-only reference dangles.
 The same unguarded dispatch
exists at harfbuzz HEAD (`src/hb-subset.cc:282-284` at 4509695c7,
`_hb_subset_table_passthrough`),
 so this is configuration-dependent,
not version-dependent;
 harfbuzz's own `HB_LEAN` profile also defines
`HB_NO_STYLE` (`src/hb-config.hh:57,85`).
 When STAT's `collect_name_ids`
is compiled in,
 it collects the design-axis name IDs,
 the surviving
axis-value name IDs,
 and `elidedFallbackNameID`
(`src/hb-ot-stat-table.hh:518-540`).

### OTS (vendored in Firefox) correctly flags and discards the table

ots@2c594bc `src/stat.cc:14-29`:

```cpp
bool OpenTypeSTAT::ValidateNameId(uint16_t nameid) {
  OpenTypeNAME* name = static_cast<OpenTypeNAME*>(
      GetFont()->GetTypedTable(OTS_TAG_NAME));

  if (!name || !name->IsValidNameId(nameid)) {
    Drop("Invalid nameID: %d", nameid);
    return false;
  }
  ...
```

`IsValidNameId` is a membership test against the parsed name table
(`src/name.cc:366`,
 `return this->name_ids.count(nameID);`).
`Table::Drop` (`src/ots.cc:1141-1152`) sets `m_shouldSerialize =
false`,
 logs the formatted message (the `STAT: ` prefix comes from
`Table::Message`,
 `src/ots.cc:1117-1121`) plus the literal
`"Table discarded"`,
 and serialization then skips the table
(`src/ots.cc:871-874` via `Font::GetTable`,
 `src/ots.cc:1073-1077`)
while the font as a whole passes.
 Firefox vendors OTS
(`gfx/ots/moz.yaml` pins `origin.revision: 57df657…`;
 `stat.cc` and
`name.cc` are identical between that revision and 2c594bc);
 the
`downloadable font: ` prefix is added by Gecko's user-font error
reporting,
 not OTS.

[harfbuzz#4162]: https://github.com/harfbuzz/harfbuzz/issues/4162
[harfbuzz PR #4168]: https://github.com/harfbuzz/harfbuzz/pull/4168

## Verification

Versions under test:

- `hb-subset-wasm` 0.4.0 (npm),
   from the pnpm catalog
- Inter variable woff2 `Version 4.001;git-9221beed3`,
  sha256 `693b77d4f32ee9b8bfc995589b5fad5e99adf2832738661f5402f9978429a8e3`
  (`package/wc/app/fonts-source/inter.woff2`)
- Firefox 151.0 / Chromium 149.0.7827.55 (playwright 1.61.1's bundled
  browsers,
   run in the `monochromatic-playwright` podman image)

### Harness: dump retained vs referenced name IDs

Save as `stat-nameid-repro.tmp.mjs` inside
`package/wc/app` (module resolution follows the
script's location) and run `node stat-nameid-repro.tmp.mjs`:

```js
// stat-nameid-repro.tmp.mjs
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

import { init as initSubset, subset } from 'hb-subset-wasm';
import wawoff2 from 'wawoff2';

await initSubset(
  await readFile(
    fileURLToPath(import.meta.resolve('hb-subset-wasm/hb-subset.wasm')),
  ),
);

/** Parses the SFNT table directory into tag -> subarray. */
function tables(sfnt) {
  const numTables = sfnt.readUInt16BE(4);
  return Object.fromEntries(
    Array.from({ length: numTables }, (_u, i) => {
      const entry = 12 + (i * 16);
      const tag = sfnt.toString('latin1', entry, entry + 4);
      const offset = sfnt.readUInt32BE(entry + 8);
      const length = sfnt.readUInt32BE(entry + 12);
      return [tag, sfnt.subarray(offset, offset + length)];
    }),
  );
}

/** Collects the set of nameIDs present in a name table. */
function nameIds(name) {
  const count = name.readUInt16BE(2);
  return new Set(
    Array.from({ length: count }, (_u, i) => name.readUInt16BE(6 + (i * 12) + 6)),
  );
}

/** Collects every nameID the STAT table references. */
function statNameIds(stat) {
  const designAxisSize = stat.readUInt16BE(4);
  const designAxisCount = stat.readUInt16BE(6);
  const designAxesOffset = stat.readUInt32BE(8);
  const axisValueCount = stat.readUInt16BE(12);
  const axisValueOffsetsStart = stat.readUInt32BE(14);
  const elidedFallbackNameID = stat.readUInt16BE(18);

  const axisNameIds = Array.from({ length: designAxisCount }, (_u, i) =>
    stat.readUInt16BE(designAxesOffset + (i * designAxisSize) + 4));

  const valueNameIds = Array.from({ length: axisValueCount }, (_u, i) => {
    const rel = stat.readUInt16BE(axisValueOffsetsStart + (i * 2));
    const at = axisValueOffsetsStart + rel;
    // Formats 1-4 all keep valueNameID at byte offset 6.
    return stat.readUInt16BE(at + 6);
  });

  return { elidedFallbackNameID, axisNameIds, valueNameIds };
}

function report(label, sfnt) {
  const t = tables(sfnt);
  const kept = nameIds(t.name);
  if (!t.STAT) {
    console.log(`${label}: no STAT table; name IDs kept: [${[...kept].join(' ')}]`);
    return;
  }
  const refs = statNameIds(t.STAT);
  const all = [refs.elidedFallbackNameID, ...refs.axisNameIds, ...refs.valueNameIds];
  const dangling = [...new Set(all)].filter((id) => !kept.has(id)).sort((a, b) => a - b);
  console.log(`${label}:
  name IDs kept:          [${[...kept].sort((a, b) => a - b).join(' ')}]
  STAT elidedFallback:    ${refs.elidedFallbackNameID}
  STAT axisNameIDs:       [${refs.axisNameIds.join(' ')}]
  STAT valueNameIDs:      [${refs.valueNameIds.join(' ')}]
  dangling STAT refs:     [${dangling.join(' ')}]`);
}

const woff2 = await readFile('fonts-source/inter.woff2');
const original = Buffer.from(await wawoff2.decompress(woff2));
report('ORIGINAL (pre-subset)', original);

const subsetted = Buffer.from(
  await subset(original, { text: 'Hello wc 0123456789', layoutFeatures: '*' }),
);
report('SUBSET (hb-subset-wasm defaults)', subsetted);
```

Output on the pinned Inter (2026-07-02):

```txt
ORIGINAL (pre-subset):
  name IDs kept:          [0 1 2 3 4 5 6 7 8 9 11 12 13 14 25 256 257 … 302]
  STAT elidedFallback:    2
  STAT axisNameIDs:       [298 257 301]
  STAT valueNameIDs:      [299 300 258 260 262 264 266 268 270 272 274 302]
  dangling STAT refs:     []
SUBSET (hb-subset-wasm defaults):
  name IDs kept:          [0 1 2 3 4 5 6 256 257 258 259 260 … 275]
  STAT elidedFallback:    2
  STAT axisNameIDs:       [298 257 301]
  STAT valueNameIDs:      [299 300 258 260 262 264 266 268 270 272 274 302]
  dangling STAT refs:     [298 299 300 301 302]
```

Two catalogs fall out of the output:

- **Resolves cleanly**:
   every `fvar`-reachable name ID survives the
  subset (256 through 275:
   axis names and named-instance subfamily
  names),
   plus the standard IDs 0 through 6.
   `STAT` references that
  happen to coincide with `fvar`'s (257,
   258,
   260 … 274) also resolve.
- **Dangles**:
   every name ID only `STAT` references:
   298 and 301
  (axis records),
   299,
   300,
   and 302 (axis-value records).
   Firefox
  reports the first one its sanitizer visits (298).

### Harness: browser-level check

`package/wc/app/src/page.browser.test.ts` collects
every error-level console message and asserts the list is empty;
 run
`mise run test:browser:firefox -- wc/app` from the
repo root.
 With the defect present it fails with both messages quoted
under Symptom;
 with either workaround below it passes.

## Verified workarounds

Byte figures below come from subsetting the pinned Inter with an
identical charset and re-encoding to woff2,
 varying only the option
under test.

### Drop STAT during subsetting (shipped)

```js
await subset(sfnt, { text, layoutFeatures: '*', dropTables: ['STAT'] });
```

Output woff2:
 41,136 bytes vs 41,172 for the defaults.
 No sanitizer
messages;
 `fvar`/`gvar`/`avar` still carry the variable axes,
 so CSS
`font-weight: 100 900` keeps working.
 This is what
`package/wc/app/src/subset-fonts.ts` ships.

Tradeoffs:
 the subset is no longer a spec-conformant variable font
(the OpenType spec expects variable fonts to carry `STAT`),
 and any
consumer that reads `STAT` for style mapping (font managers,
 desktop
style pickers,
 `fc-query`) loses axis-value names.
 Browsers render
identically with and without it.

### Pass the name table through unsubsetted

```js
await subset(sfnt, { text, layoutFeatures: '*', passthroughTables: ['name'] });
```

Output woff2:
 41,980 bytes (+808 over the defaults).
 `STAT` survives
and every reference resolves (verified with the harness above:
`dangling STAT refs: []`),
 so the subset stays a conformant variable
font.

Tradeoffs:
 ships every upstream name record (copyright,
 license URL,
all named-instance strings) in a web-delivery artifact where nothing
reads them;
 the byte cost grows with the font's name-table size.

## What does not work

- **Doing nothing**:
   the font renders fine,
   but every Firefox page
  load logs two console errors,
   which poisons any "no console errors"
  verification gate and hides real failures in the noise.
- **Retaining the needed IDs through the wrapper**:
   `hb-subset-wasm`
  0.4.0 exposes no name-ID control.
   Its `SubsetOptions` are `text`,
  `unicodes`,
   `glyphIds`,
   `retainGids`,
   `noHinting`,
   `variationAxes`,
  `passthroughTables`,
   `dropTables`,
   and `layoutFeatures`
  (`node_modules/hb-subset-wasm/dist/types.d.ts`);
   HarfBuzz's own
  name-ID set (`hb_subset_input_set(…, HB_SUBSET_SETS_NAME_ID)`,
   the
  CLI's `--name-IDs=*`) is not reachable through it.
- **`passthroughTables: ['STAT']`**:
   passing `STAT` through changes
  nothing;
   the table already survives byte-identically by default.
   The
  dangling side is the pruned `name` table,
   not `STAT` itself.

## Upstream filing decision

`.out-of-scope/` was checked (2026-07-02):
 no exemption covers
hb-subset-wasm,
 HarfBuzz,
 or OTS,
 so the full audit applies.
 OTS needs
no filing at all:
 its sanitizer is doing exactly its job.
 Two
candidates remain,
 audited separately below.
 Duplicate search,
 both
trackers,
 open and closed:
 the kyosuke/hb-subset-wasm tracker has zero
issues and zero PRs ever;
 on the HarfBuzz tracker,
 all issues matching
"STAT" were enumerated plus targeted queries ("STAT nameID",
 "nameid
closure",
 "OTS STAT",
 "Invalid nameID",
 "HB_NO_STYLE").
 [harfbuzz#4162][]
is the same symptom for pre-7.2.0 **default** builds (closed,
 fixed by
[harfbuzz PR #4168][]);
 no issue exists about `HB_NO_STYLE` builds
emitting STAT with dangling references,
 so a new issue is not a
duplicate;
 the drafts below reference #4162 as prior art.
 Related but
distinct:
 harfbuzz PR #5623 (merged,
 first in 12.2.0) stops
*collecting* STAT name IDs when STAT is dropped,
 the inverse concern.

### Candidate 1: hb-subset-wasm (primary owner)

1. **Really upstream's fault?**
    Yes.
    The wrapper's own build script
   adds `-DHB_NO_STYLE` (`scripts/build-wasm.sh:85`),
    which silently
   turns correct HarfBuzz 10.4.0 behavior into invalid output for any
   STAT-bearing font.
    Not wording,
    not architecture:
    a flag choice.
2. **Can upstream fix it?**
    Yes:
    delete one line and republish.
   Verified by the prototype below.
3. **Are they supporting this use case?**
    Yes:
    the package's stated
   purpose is font subsetting,
    its README documents variable-axes
   support,
    and the build deliberately leaves `HB_NO_VAR` off so
   variable fonts work.
4. **Would the repo welcome our contribution?**
    No `CONTRIBUTING.md`,
   no issue templates,
    no policy on AI-assisted contributions was
   found (files and tracker checked 2026-07-02);
    the repo itself
   commits a `CLAUDE.md` and is visibly AI-assisted.
    No ban found;
   absence of policy is not a fail.
5. **Will they likely fix it?**
    Unknown:
    single maintainer,
    zero
   issues ever filed,
    last push 2026-04-21.
    No signal either way,
    and
   absence of signal is not a fail.
6. **Prototyped a minimal fix?**
    Yes.
    In a disposable clone of
   `kyosuke/hb-subset-wasm@82fe83f` (origin and HEAD verified,
    with
   the `deps/harfbuzz` submodule at 3ef8709/10.4.0),
    the fix is:

   ```diff
   --- a/scripts/build-wasm.sh
   +++ b/scripts/build-wasm.sh
   @@ -82,7 +82,6 @@ CFLAGS=(
      -DHB_NO_LEGACY
      -DHB_NO_DRAW
      -DHB_NO_PAINT
   -  -DHB_NO_STYLE
      -DHB_NO_MATH
      -DHB_NO_META
      -DHB_NO_HINTING
   ```

   Built with `bash scripts/build-wasm.sh` inside a bounded
   `emscripten/emsdk:latest` podman container (no host credentials,
   only the clone mounted).
    Verification:
    the harness above,
    run
   against the rebuilt `dist/hb-subset.wasm` on the pinned Inter,
   prints

   ```txt
   PATCHED WASM: STAT kept; name IDs kept: [0 1 2 3 4 5 6 256 … 275 298 299 300 301 302]; dangling STAT refs: []
   ```

   i.e. exactly the five previously dangling IDs are now retained.
   Size cost,
    same toolchain,
    same everything but the flag:
   590,665 bytes with `HB_NO_STYLE` vs 591,933 bytes without,
   +1,268 bytes (+0.21%).

All six constraints hold;
 the draft below is fileable as-is.

~~~md
Title: Default build emits STAT tables whose nameIDs dangle after name subsetting (Firefox/OTS discards STAT)

Subsetting any variable font that has a `STAT` table produces output
that OTS (and therefore Firefox) flags as invalid: `STAT` is passed
through byte-identical, but the name records its axis and axis-value
entries reference are pruned. Firefox logs, for every page load using
the font:

```
downloadable font: STAT: Invalid nameID: 298 (font-family: "Inter" …)
downloadable font: Table discarded (font-family: "Inter" …)
```

Reproduction (hb-subset-wasm 0.4.0, Inter 4.001 variable, but any
STAT-bearing font shows it):

```js
import { init, subset } from 'hb-subset-wasm';
// … init …
const out = await subset(interSfnt, { text: 'Hello' });
// out's name table keeps IDs 0-6 plus fvar's 256-275;
// out's STAT still references 298-302, which now dangle.
```

Cause: `scripts/build-wasm.sh` compiles HarfBuzz with `-DHB_NO_STYLE`.
That define compiles out the STAT accelerator
(`hb-ot-face-table-list.hh`) and with it the STAT branch of
`_nameid_closure` (`hb-subset-plan.cc`), while the STAT case in
`hb-subset.cc`'s table dispatch is not guarded and passes the table
through verbatim. Stock HarfBuzz 10.4.0 without that define collects
STAT's name IDs correctly (this was harfbuzz#4162, fixed in 7.2.0).

Suggested fix: remove `-DHB_NO_STYLE` from the CFLAGS array in
`scripts/build-wasm.sh` and rebuild. Measured on emsdk latest, the
wasm grows from 590,665 to 591,933 bytes (+0.21%), and the same subset
call then retains name IDs 298-302 with zero dangling STAT references.
Verified against Inter 4.001 and OTS. An alternative if the size
matters more than STAT: add `STAT` to the default drop set, since a
missing STAT is valid where a dangling one is not.

This report and the fix were prepared with AI assistance
(reproduction, source trace, and the rebuilt-wasm verification were
run end-to-end as described above).
~~~

### Candidate 2: HarfBuzz (latent config inconsistency)

1. **Really upstream's fault?**
    Yes,
    narrowly:
    `HB_NO_STYLE` is an
   official config knob (their `HB_LEAN` profile sets it,
   `src/hb-config.hh:57,85`),
    and under it hb-subset silently emits
   output that fails sanitizers.
    A config that cannot maintain a
   table's integrity should not emit that table by default.
2. **Can upstream fix it?**
    Yes;
    the guard below is small and verified.
3. **Are they supporting this use case?**
    Yes:
    subsetting is a core
   product,
    and the `HB_NO_*` config system is documented and
   maintained.
4. **Would the repo welcome our contribution?**
    Yes,
    with disclosure:
   `CODE_OF_AI_CONDUCT.md` (present at HEAD) accepts AI-assisted
   contributions,
    requires contributors to remain fully responsible,
   requires an `Assisted-by:` trailer on commits,
    and bans fabricated
   reproductions (ours is real and recorded here).
    README invites
   issues and pull requests.
5. **Will they likely fix it?**
    Plausible:
    the sibling default-build
   bug (#4162) was fixed within a day,
    and the project actively
   maintains config-gated correctness.
    No won't-fix signal found.
6. **Prototyped a minimal fix?**
    Yes.
    In a disposable clone of
   `harfbuzz/harfbuzz@4509695c7` (origin and HEAD verified),
    the
   `hb-subset` CLI was built with `-Dcpp_args=-DHB_NO_STYLE` (meson,
   Fedora 42 container;
    `hb-info` fails to link under this define
   because it uses the style API directly,
    so only the `util/hb-subset`
   target was built).
    Pre-patch,
    on the pinned Inter:

   ```txt
   PREPATCH (HB_NO_STYLE, HEAD 4509695c7): STAT kept; dangling STAT refs: [298 299 300 301 302]
   ```

   Patch:

   ```diff
   --- a/src/hb-subset.cc
   +++ b/src/hb-subset.cc
   @@ -280,8 +280,18 @@ _subset_table (hb_subset_plan_t *plan,
        return _hb_subset_table<const OT::head> (plan, buf);
    
      case HB_TAG('S','T','A','T'):
   +#ifndef HB_NO_STYLE
        if (!plan->user_axes_location.is_empty ()) return _hb_subset_table<const OT::STAT> (plan, buf);
        else return _hb_subset_table_passthrough (plan, tag);
   +#else
   +    /* With HB_NO_STYLE the STAT accelerator, and with it the STAT branch
   +     * of _nameid_closure, is compiled out; emitting STAT would leave its
   +     * axis and axis-value nameIDs dangling after name subsetting (OTS
   +     * then rejects the table).  Drop it: a missing STAT is valid output,
   +     * dangling references are not.  An explicit passthrough request via
   +     * no_subset_tables is honored above. */
   +    return true;
   +#endif
    
      case HB_TAG('c','v','t',' '):
    #ifndef HB_NO_VAR
   ```

   Post-patch,
    same command
   (`hb-subset --text="Hello wc 0123456789" --layout-features="*"`):

   ```txt
   POSTPATCH tables: GDEF GPOS GSUB HVAR MVAR OS/2 avar cmap fvar glyf gvar head hhea hmtx loca maxp name post
   STAT present: false
   ```

   The `#ifndef` branch is the untouched original,
    so default builds
   are unaffected;
    explicit `no_subset_tables` passthrough still wins
   (it is checked before the switch).

All six constraints hold;
 the draft below is fileable as-is.

~~~md
Title: [subset] HB_NO_STYLE builds emit STAT with dangling nameIDs (OTS rejects the table)

When HarfBuzz is compiled with `HB_NO_STYLE` (as the `HB_LEAN` profile
does), hb-subset still emits the `STAT` table (passthrough when not
instancing; `hb-subset.cc` `_subset_table`, the `case
HB_TAG('S','T','A','T')` is unguarded), but `_nameid_closure` cannot
collect STAT's name IDs because the STAT accelerator is compiled out
(`#ifndef HB_NO_STYLE` in `hb-subset-plan.cc` and
`hb-ot-face-table-list.hh`). The subset's name table then lacks the
records STAT references, and OTS drops the table
(`STAT: Invalid nameID: N` + `Table discarded`), so Firefox logs
console errors for every use of the font. This is #4162 all over
again, but config-induced instead of default: stock builds have been
correct since 7.2.0.

Reproduce (at 4509695c7):

```sh
meson setup build -Dcpp_args=-DHB_NO_STYLE -Dtests=disabled
ninja -C build util/hb-subset
./build/util/hb-subset --text="Hello" --output-file=out.ttf Inter-Variable.ttf
# out.ttf: name keeps IDs 0-6 + fvar's; STAT still references
# axis/axis-value nameIDs outside that set -> OTS: "STAT: Invalid nameID"
```

Suggested fix (verified at 4509695c7: pre-patch the subset dangles
nameIDs 298-302 on Inter 4.001; post-patch STAT is dropped and the
output sanitizes cleanly; default builds untouched, since the
`#ifndef` branch is the original code):

```diff
--- a/src/hb-subset.cc
+++ b/src/hb-subset.cc
@@ -280,8 +280,18 @@ _subset_table (hb_subset_plan_t *plan,
     return _hb_subset_table<const OT::head> (plan, buf);
 
   case HB_TAG('S','T','A','T'):
+#ifndef HB_NO_STYLE
     if (!plan->user_axes_location.is_empty ()) return _hb_subset_table<const OT::STAT> (plan, buf);
     else return _hb_subset_table_passthrough (plan, tag);
+#else
+    /* With HB_NO_STYLE the STAT accelerator, and with it the STAT branch
+     * of _nameid_closure, is compiled out; emitting STAT would leave its
+     * axis and axis-value nameIDs dangling after name subsetting (OTS
+     * then rejects the table).  Drop it: a missing STAT is valid output,
+     * dangling references are not.  An explicit passthrough request via
+     * no_subset_tables is honored above. */
+    return true;
+#endif
 
   case HB_TAG('c','v','t',' '):
 #ifndef HB_NO_VAR
```

An alternative shape would be decoupling STAT's `collect_name_ids`
from the style API so `HB_NO_STYLE` builds can keep emitting a valid
STAT, at the cost of keeping the STAT accelerator compiled in.

This report and the fix were prepared with AI assistance; the
reproduction and pre/post-patch verification were run end-to-end as
described.
~~~

Neither draft has been filed;
 filing is the user's call.
 If filed,
 the
HarfBuzz one must keep its AI-assistance disclosure to comply with
their `CODE_OF_AI_CONDUCT.md`,
 and a PR would carry an `Assisted-by:`
trailer per the same policy.
