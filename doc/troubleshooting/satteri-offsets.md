# Sätteri 0.10.5 source slices shift under the retained 0.9.4 offset correction

## Current 0.10.5 consumer failure

The historical 0.9.4 investigation is retained,
but its correction must not be applied to already-correct UTF-16 positions.
The installed `satteri@0.10.5` returns UTF-16 offsets on the measured Markdown cases.
The repository wrapper still applies the older conversion when astral text is present,
shifting those positions again.

This is tracked in [Monochromatic issue 559](https://github.com/Aquaticat/Monochromatic/issues/559).
The accepted [unified-linter decision](../handover/unified-linter.md) leaves the TypeScript linter unpatched
because the Rust replacement supersedes it.
The private auto-mode workaround respects that decision.
No production parser or dependency configuration was changed.

### Current symptom and source trace

A structural policy-index control on 2026-09-29 failed with Node `AssertionError [ERR_ASSERTION]`:
expected tagged paragraph `HON`,
but the wrapper's source interval began with `ON:`.
The constructor stopped before producing the actual policy index.
Its preceding controls and failure artifacts remain preserved.

`package/cli/markdown-lint/src/parse.ts:78` still applies the old conversion:

```typescript
// package/cli/markdown-lint/src/parse.ts:78
return hasAstralCodePoints(source,)
  ? correctAstralOffsets({
      tree: node,
      source,
    },)
  : node;
```

The helper treats an existing offset as a code-point index:

```typescript
// package/cli/markdown-lint/src/correct-astral-offsets.ts:97
const codePointOffset = point.offset;
if (codePointOffset !== undefined) {
  point.offset = map[codePointOffset] ?? codePointOffset;
}
```

The current read-only Sätteri checkout is `~/temp/agent/satteri-offset-source.dR6ZsYSZ`,
tag `satteri-v0.10.5`,
commit `b3d38e1e341c809b20b76a655e9b1601d11bd1f0`.
The first lookup of tag spelling `v0.10.5` returned HTTP 404;
the tag list supplied the package-prefixed name.
No upstream files or build scripts were changed or executed.

At that revision,
`crates/satteri-arena/src/line_index.rs:226` counts UTF-16 units:

```rust
// crates/satteri-arena/src/line_index.rs:226, selected statements
if (b & 0xC0) != 0x80 {
    count += if b >= 0xF0 { 2 } else { 1 };
}
```

The materialized-tree serialization converts offsets at the wire interface:

```rust
// crates/satteri-arena/src/raw_buffer.rs:116
let utf16_start = cursor.byte_to_utf16_offset(node.start_offset);
let utf16_end = cursor.byte_to_utf16_offset(node.end_offset);
```

The cached path at `raw_buffer.rs:87` also writes UTF-16 values.
The source revision and installed package label are recorded separately from measured runtime behavior;
this is not a native-build provenance attestation.

### Current verification

Evidence is retained under `~/temp/agent/auto-mode-consumer-contract.mDLkyNoP/contract`:

- `policy-index-started.json` and `policy-index-failure.json` retain the stopped constructor.
- `offset-diagnostic/result-initial.json` compares direct,
  wrapped,
  and explicitly corrected-clone trees under Node `v26.10.0`.
- `native-index/result.json` retains the corrected private adapter run.
- `native-docs/verification.json` retains native-coordinate lint composition checks.

On the original fixture,
`source.indexOf('HON:')` is 12.
Direct Sätteri reports 12;
the wrapper reports 13 and loses the first letter in its slice.
For the minimized source `"\u{1f680}\n\nx\n"`,
target `x` begins at 4:
direct Sätteri returns 4,
while the wrapper returns 5 and slices the newline.
Applying the correction directly to a cloned native tree reproduces the shifted wrapper result.
Selected paragraph/text point objects were not shared.

ASCII and BMP controls retained correct offsets.
With two preceding astral characters,
the measured paragraph start shifted from 14 to 16 after the extra conversion.
Do not infer a uniform shift for every end offset:
the helper's out-of-range fallback can retain an original value.
The current failure is not the historical native code-point result or a changed fixture reference.

Recorded invocations were:

```bash
# Private contract/offset-diagnostic directory
mise --no-env --no-hooks run probe
```

```bash
# Private contract/native-index directory
mise --no-env --no-hooks run index
```

These are completed create-new phases,
not instructions to replay them or delete their receipts.
A new reproduction requires a separate disposable output namespace.

### Current private workaround and limits

The private adapter uses the same installed `satteri@0.10.5`,
keeps GFM and frontmatter enabled,
and applies no second offset conversion.
It checks the admitted version and a fixed astral coordinate case,
and rejects its untested MDX path.
This is not a global dependency pin or a production linter patch.

All 14 recorded structural and admission controls passed,
including the original failing fixture and the duplicate-guard omission control.
The original index implementation and parent references remained unchanged.
The required policy then indexed as 276 tagged paragraphs and 47 headings,
with no unclassified top-level nodes.
That is structural coverage only,
not instruction interpretation or authority qualification.
No external model call occurred and `AGENTS.md` was unchanged.

For document checks,
the private composition gives each existing `Rule.check()` the unadjusted native tree
and uses the existing `applyFixes()` function.
It does not invoke the legacy parser through `runRules()` or `fixSource()` and disables no rule.
The fixed Unicode/bold case produced the required `semantic-line-breaks` insertion;
the legacy path produced no diagnostic.
The native composition reached an idempotent fixpoint with zero remaining diagnostics.
Heading,
blockquote,
strong,
code,
inline-code,
and link spans matched the predeclared raw fragments.
Image-bearing inputs are rejected rather than silently skipping required LFS context.

This covers the measured Markdown consumers on the admitted version,
not MDX,
visitor/plugin paths,
future versions,
or a complete native dependency closure.
No emoji was removed or mapped to another character to make the index controls pass.

### Current upstream filing disposition

The original issue is [Sätteri 172](https://github.com/bruits/satteri/issues/172),
closed by [PR 174](https://github.com/bruits/satteri/pull/174),
merge `6696c1c28b3024c5c8df760cc5af51dd713663fc`.
[PR 176](https://github.com/bruits/satteri/pull/176) added astral conformance follow-ups.
Bodies and comments of those threads and repository issue 559 were read.
The historical draft is not a new fileable report against 0.10.5.
No duplicate issue,
comment,
or upstream patch is proposed.

1.  Upstream fault:
    not established for the installed materialized path;
    the reproduced extra shift comes from the retained repository correction.
2.  Fixability:
    the private consumer workaround is built and verified;
    no impossibility claim is made about a future production change.
3.  Supported use:
    current source explicitly supplies UTF-16 positions for the JavaScript tree.
4.  Contribution policy:
    current `CONTRIBUTING.md` was read;
    it requires duplicate checks,
    tests,
    and changesets.
    No new upstream contribution is needed here.
5.  Maintainer willingness:
    the relevant upstream fix and follow-up tests are merged.
    Repository issue 559 intentionally defers the TypeScript-side fix to replacement work.
6.  Fix prototype:
    the private Markdown adapter and rule composition are verified,
    not an upstream patch or adopted production implementation.

No `.out-of-scope/` filename identifies Sätteri as excluded.
The existing duplicate and already-present fix make another upstream filing unnecessary.

## Historical 0.9.4 investigation

Sätteri (<https://satteri.bruits.org/>,
 npm `satteri@0.9.4`,
 git `bruits/satteri`
at commit `15b0f8574`) exposes standard mdast nodes whose `position.start.offset`
/ `position.end.offset` do not match the UTF-16 code-unit indices that
remark/micromark (and JavaScript string indexing) use.
 Two distinct code paths
get it wrong in two different ways.
 Any consumer that slices the source at those
offsets,
 or writes edits at them,
 corrupts output on documents containing
multibyte or astral (emoji) characters.

This was found while evaluating a migration of the repo's Markdown linter
(`package/cli/markdown-lint`) from `mdast-util-from-markdown` to Sätteri.
 The
linter recovers exact written forms and applies fixes at node offsets,
 so wrong
offsets misplace slices and edits.

## Historical 0.9.4 symptom

Two bugs,
 same theme (offset unit mismatch),
 different paths:

- Bug A,
   plugin/visitor path:
   nodes handed to an MDAST visitor (via the manual
  `visitMdastHandle` pipeline,
   and via `markdownToHtml` / `mdxToJs` mdast
  plugins) carry `position.*.offset` as UTF-8 **byte** offsets.
   On a document
  with 8 em-dashes before a node,
   `markdownToMdast` places that node at char
  offset 1842 while the visitor sees offset 1858 (its 16 extra bytes).
   Every
  `source.slice(node.position.start.offset, node.position.end.offset)` and every
  edit offset is shifted by (bytes minus UTF-16 units) of all preceding
  non-ASCII text.
- Bug B,
   materialized path:
   `markdownToMdast` / `markdownToHast` report
  `position.*.offset` as Unicode **code-point** offsets.
   These are correct for
  BMP characters (em-dash `—` U+2014,
   CJK,
   accents) but wrong for astral
  characters (emoji,
   U+1F680 `🚀`,
   math alphanumerics),
   which are one code point
  but two UTF-16 code units.
   A node the parser correctly gives
  `value: "3.1.1 "` slices to `"1. 3.1"` when three emoji precede it,
   because
  its stored offset is 3 short.

There is no thrown error in either case;
 the failure is silent corruption.
 A
downstream symptom seen in the linter:
 an autofix that converts a Markdown pipe
table to an HTML `<table>` slices table cells at the byte-shifted offsets
(Bug A),
 producing malformed HTML that Sätteri's MDX parser then rejects on
re-parse with `Expected the closing tag </table> ... (mdx-jsx:unexpected-character)`.
The clean HTML that a correct slice would produce parses fine,
 so that crash is
a consequence of Bug A,
 not a separate MDX defect.

## Historical 0.9.4 root cause

The arena stores raw UTF-8 byte offsets per node.
 remark's `position.*.offset`
is a UTF-16 code-unit index into the JS source string (confirmed below).
 Sätteri
never converts to UTF-16:
 the materialized path converts to code points (close,
but wrong on astral chars),
 and the walk path does not convert at all.

The wrong premise is stated in the source.
 `crates/satteri-arena/src/line_index.rs:5`:

```rust
/// Columns and offsets are counted as Unicode code points (matching the
/// CommonMark `position` convention used by remark/micromark), not bytes —
```

remark does not use code points;
 it uses UTF-16 code units.
 The counter that
implements this premise,
 `crates/satteri-arena/src/line_index.rs:152`:

```rust
fn code_point_count_bytes(bytes: &[u8]) -> u32 {
    let mut count: u32 = 0;
    for &b in bytes {
        if (b & 0xC0) != 0x80 {   // count every non-continuation byte = one code point
            count += 1;
        }
    }
    count
}
```

A 4-byte UTF-8 sequence (an astral scalar value) is counted as one,
 but it is
two UTF-16 code units.
 This drives both `byte_to_cp_offset`
(`line_index.rs:113`) and the per-line `line_cp_offsets`,
 so Bug B is every
offset past an astral character.

Bug B path (materialized).
 `crates/satteri-arena/src/raw_buffer.rs:74` and
`:107`-`117` build a `LineIndex` and rewrite each node's stored byte offset to a
code-point offset before serializing the wire buffer JS materializes:

```rust
// raw_buffer.rs:74
// The arena tracks `start_offset`/`end_offset` as **byte** offsets
// raw_buffer.rs:107,116-117
let line_index = LineIndex::from_source(&self.string_pool);
// ...
let cp_start = cursor.byte_to_cp_offset(node.start_offset);
let cp_end = cursor.byte_to_cp_offset(node.end_offset);
```

Bug A path (plugin/visitor).
 `crates/satteri-ast/src/walk.rs:208`-`209`
`write_walk_prelude` emits the arena's raw byte offsets with no conversion at all:

```rust
// Position (always present)
out.extend_from_slice(&node.start_offset.to_le_bytes());
out.extend_from_slice(&node.end_offset.to_le_bytes());
```

So the walk path is even further off than the materialized path:
 raw bytes
rather than code points.
 The same `write_walk_prelude` feeds both the manual
`visitMdastHandle` pipeline and the mdast-plugin stage of `markdownToHtml`,
 so
the whole plugin API is affected,
 not just the low-level pipeline.

Earlier hypotheses that were wrong,
 recorded so the next investigator does not
repeat them:

- "The divergence is a list-marker parse difference."
   The minimal input
  `1. 3.1.1 **Testing**` parses identically in Sätteri and mdast-util (marker
  consumed,
   `text: "3.1.1 "`).
   The real cause is Bug B:
   the affected file
  (`package/rss/app/TODO.index.md`) contains emoji,
   and the
  node's `value` is correct (`"3.1.1 "`) while its `position` offset is 3 short
  (three emoji precede it).
   Value-versus-slice mismatch,
   not a parse difference.
- "The materialized path is correct;
   only the visitor path is buggy."
   False:
  the materialized path is correct only for BMP text.
   It is wrong on astral
  characters (Bug B).
   The em-dash corpus file misled the first reading because
  `—` is BMP,
   so code point equals UTF-16 there.

## Historical 0.9.4 verification

Version under test:
 npm `satteri@0.9.4` (native `satteri-linux-x64-gnu`),
 git
source `bruits/satteri@15b0f8574`.
 Reference parser:
 `mdast-util-from-markdown`
2.0.3 with `micromark-extension-gfm`.

Reference behavior (remark uses UTF-16,
 for both offset and column).
 A link
after one astral emoji:

```js
// node --input-type=module
import { fromMarkdown } from 'mdast-util-from-markdown';
const md = 'a🪐b [x](http://e.com)\n';
const tree = fromMarkdown(md);
let link; (function w(n){ if (n.type === 'link') link = n; for (const c of n.children ?? []) w(c); })(tree);
console.log(link.position.start.offset, link.position.start.column);
// 5 6   -> UTF-16 index/column of "[x]"; code-point index/column would be 4 5
```

Bug B (materialized,
 code-point offsets).
 Value is right,
 slice is wrong:

```js
import { markdownToMdast } from 'satteri';
const md = '🎯 x\n\n- item, with a break. done.\n';
const t = markdownToMdast(md);
let n; (function w(x){ if (x.type === 'text' && x.value.includes('item')) n = x; for (const c of x.children ?? []) w(c); })(t);
console.log(JSON.stringify(n.value));                                   // "item, with a break. done."
console.log(JSON.stringify(md.slice(n.position.start.offset, n.position.end.offset)));
// " item, with a break. done"  -> off by one (one astral char before it)
```

Bug A (visitor,
 byte offsets).
 The same tree via the manual pipeline is off by
the byte-minus-UTF-16 delta of all preceding non-ASCII:

```js
import { createMdastHandle, dropHandle, visitMdastHandle, resolveMdastSubscriptions, markdownToMdast } from 'satteri';
const md = '— — — — — — — — `code` and then text, here.\n'; // 8 em-dashes, then a 2nd text node
const visitors = { text(node){ if (node.value.includes('and then')) console.log('visitor offset', node.position.start.offset); } };
const subs = resolveMdastSubscriptions(visitors);
const h = createMdastHandle(md);
visitMdastHandle(h, visitors, subs, md, undefined, {}, 'markdown');
dropHandle(h);
// visitor offset 38   (byte). markdownToMdast reports 22 (code-point/UTF-16, they coincide here since — is BMP).
```

Patterns that work cleanly (no astral chars,
 no non-ASCII before the node):

- Pure ASCII documents:
   all offsets correct in both paths.
- BMP-only non-ASCII (em-dash,
   accents,
   CJK,
   `ὐ`) via the materialized path:
  correct,
   because code point equals UTF-16 unit for BMP.

Patterns that fail:

- Bug A (byte offsets):
   any document with non-ASCII text before a node,
   read
  through `visitMdastHandle` or a `markdownToHtml` mdast plugin.
- Bug B (code-point offsets):
   any document with astral characters (emoji,
   etc.)
  before a node,
   read through `markdownToMdast` / `markdownToHast`.

## Historical 0.9.4 workarounds

Consumer-side,
 at our boundary (Sätteri is third-party;
 its clone is not edited
for a local fix).
 Both correct offsets back to UTF-16 before any slice or edit.

- For the materialized path (Bug B),
   used by the linter's parser swap:
   after
  `markdownToMdast`,
   when the source contains any astral character,
   build a
  code-point-index to UTF-16-index map by scanning the source once,
   then walk
  the tree and rewrite every `position.start.offset` / `position.end.offset`.
  BMP-only sources (`source.length === [...source].length`) need no correction,
  so the common case is a single length comparison.
   Tradeoff:
   one extra O(n)
  source scan plus one O(nodes) tree walk,
   but only on astral-bearing files;
  it does not fix columns (left as code points),
   which the linter does not use
  for edits.
- For the visitor path (Bug A):
   do not use it for any offset-based work.
   The
  byte offsets cannot be corrected as cheaply (they need a full byte-to-UTF-16
  map),
   and slicing the source at them is wrong for all non-ASCII,
   not just
  astral.
   Prefer the materialized path plus the Bug B correction.
   This is why
  the linter migration adopts the parser swap (materialized) and not the plugin
  rewrite (visitor);
   see `doc/handover/markdown-lint-satteri-benchmark.md`.

## Historical 0.9.4 rejected approaches

- Trusting the materialized path unconditionally.
   It is correct for BMP text,
  which hides Bug B until an emoji appears;
   the repo's docs contain emoji.
- Converting the visitor's byte offsets by counting code points.
   That reaches
  the materialized path's behavior,
   which is still wrong on astral characters.
  The only correct unit is UTF-16 code units.
- Using Sätteri's `position.*.column` as a stand-in.
   Columns are computed with
  the same code-point counter and are equally wrong on astral characters.

## Historical upstream filing decision

`.out-of-scope/` was checked:
 `claude-code-upstream-bugs.md` is specific to
Claude Code and does not exempt Sätteri,
 so upstream tracking is in scope.

Duplicate search:
 `gh issue list --repo bruits/satteri --state all` and
`gh search issues` for offset/position/utf terms.
 No existing issue covers the
offset unit.
 The nearest threads are unrelated:
 #87 (diagnostics not surfaced),
#94 (range API),
 #124 (`ctx.source` duplication).
 No duplicate;
 a new issue is
appropriate.

Six-constraint check:

1. Upstream's fault?
    Yes.
    Sätteri emits offsets in byte or code-point units
   where the ecosystem it targets uses UTF-16 code units.
2. Can upstream fix it?
    Yes,
    and prototyped below.
    Not architectural:
    a counter
   change plus one conversion at the walk site.
3. Supporting this use case?
    Yes.
    Standard mdast `position` compatible with
   remark/micromark is a stated goal,
    asserted in `line_index.rs:5-8`.
4. Would the repo welcome it?
    Yes.
    `CONTRIBUTING.md` invites issues and PRs and
   ships a bug-report template (`.github/ISSUE_TEMPLATE/01-bug-report.yml`);
    no
   ban on AI-assisted reports was found (`AGENTS.md` treats agents and humans
   equally and asks for a Sampo changeset on fixes).
    The draft discloses AI
   assistance accordingly.
5. Will they likely fix it?
    No signal against.
    Correctness is a stated value
   ("fast,
    correct,
    and extensible",
    `CONTRIBUTING.md`);
    the tracker is active;
   no won't-fix on positions.
6. Prototyped a minimal fix?
    Yes,
    verified.
    See below and `satteri-offsets.patch`.

Prototype (Bug B core,
 verified).
 In a fresh disposable clone
(`bruits/satteri@15b0f8574`),
 change the counter to count UTF-16 code units
(a 4-byte sequence adds two),
 which flows through `byte_to_cp_offset` and
`line_cp_offsets`.
 Full diff in `satteri-offsets.patch` (this directory).
The essential hunk:

```rust
// crates/satteri-arena/src/line_index.rs, code_point_count_bytes -> utf16_unit_count_bytes
if (b & 0xC0) != 0x80 {
    count += 1;
    if b >= 0xF0 {   // 4-byte UTF-8 = astral scalar = surrogate pair = 2 UTF-16 units
        count += 1;
    }
}
```

Verification with the crate's own harness (light:
 `satteri-arena` depends only
on `memchr` and `rustc-hash`),
 asserting the `b` after `🪐` lands at UTF-16
offset 2 and column 3:

```text
$ cargo test -p satteri-arena --lib astral_offset_and_column_are_utf16
# pre-patch:  assertion `left == right` failed  left: 1  right: 2   (code-point counting)
# post-patch: test result: ok. 20 passed; 0 failed
```

Bug A (walk site) uses the same corrected conversion.
 `walk.rs:208-209` must
mirror the materialized path (`raw_buffer.rs:107,116-117`):
 build a
`LineIndex::from_source` cursor once at the walk entry and emit
`cursor.byte_to_cp_offset(node.start_offset)` / `...end_offset` instead of the
raw fields.
 This half is described,
 not compiled,
 because verifying it end to
end needs a full napi rebuild;
 the offset-unit core it depends on is proven by
the cargo test above.

At that historical investigation,
all six constraints were recorded as satisfied.
The retained draft is historical evidence only:
the upstream defect is now covered by issue 172 and merged PRs 174 and 176.
Do not file this draft against the current 0.10.5 materialized path.

~~~md
Title: `position` offsets use the wrong unit: byte offsets from plugin visitors, code-point offsets from materialized trees (should be UTF-16 code units)

Labels: bug, parser, plugin api

### Summary

`position.start.offset` / `position.end.offset` on mdast nodes do not match the
UTF-16 code-unit offsets remark/micromark report (indices into the JS source
string). Two paths are wrong in two ways:

- Materialized (`markdownToMdast` / `markdownToHast`): code-point offsets.
  Correct for BMP, off by the astral-character count for emoji and other
  supplementary-plane characters.
- Plugin/visitor (`visitMdastHandle`, and `markdownToHtml` / `mdxToJs` mdast
  plugins): raw UTF-8 byte offsets, off for all non-ASCII text.

Any consumer slicing the source or writing edits at these offsets corrupts
output on non-ASCII / emoji documents.

### Reproduction

Materialized, code-point offsets (off by one astral char):

```js
import { markdownToMdast } from 'satteri';
const md = '🎯 x\n\n- item, with a break. done.\n';
const t = markdownToMdast(md);
let n; (function w(x){ if (x.type==='text' && x.value.includes('item')) n=x; for (const c of x.children ?? []) w(c);
})(t);
// n.value === "item, with a break. done." but
// md.slice(n.position.start.offset, n.position.end.offset) === " item, with a break. done"
```

Visitor, byte offsets (off by preceding non-ASCII byte delta):

```js
import { createMdastHandle, dropHandle, visitMdastHandle, resolveMdastSubscriptions } from 'satteri';
const md = '— — — — — — — — `code` and then text.\n';
const visitors = { text(n){ if (n.value.includes('and then')) console.log(n.position.start.offset); } };
const subs = resolveMdastSubscriptions(visitors);
const h = createMdastHandle(md);
visitMdastHandle(h, visitors, subs, md, undefined, {}, 'markdown'); // logs 38 (byte); should be 22
dropHandle(h);
```

remark reports UTF-16 (reference):

```js
import { fromMarkdown } from 'mdast-util-from-markdown';
const t = fromMarkdown('a🪐b [x](http://e.com)\n'); // link offset 5, column 6 (UTF-16), not 4/5 (code point)
```

### Root cause

The arena stores byte offsets. `crates/satteri-arena/src/line_index.rs`
converts to code points, not UTF-16 code units (`code_point_count_bytes`
counts one per scalar value; a 4-byte sequence is two UTF-16 units). The
premise in the doc comment ("counted as Unicode code points ... used by
remark/micromark") is the error: remark uses UTF-16.

- Materialized path converts via `raw_buffer.rs` (`byte_to_cp_offset`), so it
  is code-point-based.
- `crates/satteri-ast/src/walk.rs:208-209` (`write_walk_prelude`) emits the raw
  byte `start_offset`/`end_offset` with no conversion at all.

### Suggested fix

Count UTF-16 code units instead of code points in
`crates/satteri-arena/src/line_index.rs` (a 4-byte lead byte adds two), which
corrects `byte_to_cp_offset`, `line_cp_offsets`, and the column counter. Then
apply that conversion at the walk site
(`crates/satteri-ast/src/walk.rs:208-209`) as the materialized path already
does (`raw_buffer.rs:107,116-117`). A verified crate-level diff for the counter
(with a pre/post test) is available.

Note: this report was prepared with AI assistance; the reproductions, the
source trace, and the counter fix were verified against
`satteri@0.9.4` and `bruits/satteri@15b0f8574` (`cargo test -p satteri-arena`).
~~~
