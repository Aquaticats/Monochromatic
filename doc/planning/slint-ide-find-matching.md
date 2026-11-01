# Slint IDE in-file find matching

## Decision

On 2026-10-05 the user questioned the planned matcher vetting:
"why would we need to vet a find matcher? Isn't it just substring match?"
In-file find therefore uses plain literal,
case-insensitive substring matching with the `regex` crate that `helix-core` already re-exports.
No new matching dependency is adopted,
and no technology-vetting run is needed for it.

Reproducing Chrome's ICU collation search is not a goal.
The earlier text in this plan treated browser matching fidelity as a settled requirement;
that was an agent inference from "editord's Ctrl+F is the browser's find",
not a user instruction,
and it is withdrawn.
The accepted scope calls editord a familiar behavioral reference,
not an exhaustive compatibility contract.

The measured browser corpus stays as the record of this deliberate difference:
the cases listed under "Comparison result" are the ones where the native find
intentionally behaves differently from editord's browser find.
The planned probe of editord's one-div-per-line DOM is dropped,
because it only mattered for reproducing the browser's cross-line behavior.

## Scope

In-file find is already part of the accepted read-only IDE scope.
The reference is editord's browser-provided Ctrl+F behavior,
not another regular-expression search surface.
The find bar follows ordinary find conventions:
Ctrl+F,
incremental matching,
Enter and Shift+Enter with wrap-around,
a match count,
and Esc to close.

The native implementation must keep original source positions and copy bytes,
reject stale results after file navigation or external reload,
and perform no project mutation.
Every deliberate difference from the browser reference is named in
`package/ide/app/README.md`.

## Measured reference

`inspect:find-reference` uses a disposable browser profile,
synthetic source strings,
positive and negative controls,
and closes its session in `finally`.
It does not inspect or change the host clipboard.
The profile's `Last Version` identifies Chrome `149.0.7827.54`.

The first capture is `/tmp/ide-find-reference-is2TFy/results.json`.
The extended capture is `/tmp/ide-find-reference-mskkxd/results.json`.
The latter is preserved in
`package/ide/app/tests/fixture/browser-find.json`.

Observed matches include case differences,
canonical accent equivalents,
plain letters against accented text,
`STRASSE` against `Straße`,
`office` against `oﬃce`,
final sigma,
dotted I,
NBSP against space,
curly against straight quotes,
and a word containing a soft hyphen against the unhyphenated query.
A partial `s` query does not match `ß`.
Empty patterns,
soft-hyphen-only patterns,
and combining-mark-only patterns do not match the tested inputs.

The kana cases are intentionally specific:
`カ` matches `か` and `ｶ`;
`や` does not match `ゃ`;
`か` does not match `が`;
composed `が` matches decomposed `か` plus a combining voicing mark;
`ガ` does not match the tested halfwidth voiced form `ｶﾞ`.

The fixture is one preformatted text element.
Literal newline queries match literal newlines in that fixture,
while an ordinary space query does not replace a newline or tab.
This does not yet establish multiline behavior in editord's actual one-div-per-line DOM.

## Pinned source trace

Source snapshots are under
`~/temp/agent/ide-find-chromium.s8LhH4ir`,
using Chromium tag `149.0.7827.54`.
The source fetch initially failed only for `third_party/icu/README.chromium`;
ICU is a separate checkout,
so the remaining fetch records Chromium's `DEPS` instead.

- `third_party/blink/renderer/core/frame/local_dom_window.cc:1543` implements `Window.find`.
  It sets case-insensitive behavior from the inverse of the supplied case-sensitive flag
  and calls `Editor::FindString`.
- `third_party/blink/renderer/core/editing/editor.cc:850` forwards to `FindRangeOfString`.
  `FindStringBetweenPositions` invokes `FindBuffer::FindMatchInRange`.
- `third_party/blink/renderer/core/editing/finder/text_finder.cc:244` also invokes `Editor::FindRangeOfString`
  for the find-in-page UI,
  with differences in navigation and ruby flags.
- `third_party/blink/renderer/core/editing/finder/find_buffer.cc:406` folds quotes and soft hyphens
  before constructing `FindResults`.
- `third_party/blink/renderer/core/editing/finder/find_results.cc:13` supplies the pattern and UTF-16 text
  to `TextSearcherICU`.
- `third_party/blink/renderer/core/editing/iterators/text_searcher_icu.cc:45` creates ICU StringSearch
  with the UI language plus `@collation=search`.
  Its `SetCaseSensitivity` at line 247 selects primary collation strength for case-insensitive matching.
  It filters zero-length matches and applies an additional NFC-based kana check.
- `third_party/blink/renderer/platform/text/unicode_utilities.cc` implements the quote,
  soft-hyphen,
  and kana handling.

These paths connect the probe and UI to the same matching engine.
They do not establish identical navigation,
shadow-tree,
ruby,
or DOM block-boundary behavior.

## Comparison result

`inspect:find-regex` (`proc_6c18`) ran the already adopted Helix `regex` engine
against the captured corpus in `tests/fixture/browser-find.json`.
Every query is escaped as a literal
and the builder enables Unicode case-insensitive matching,
so the comparison isolates matching semantics from pattern syntax.
Positive and negative controls are asserted inside the test.

The engines agree on 16 of 29 cases:
literal and negative controls,
ASCII case,
regex metacharacters,
significant spaces,
CJK,
final sigma,
newline and tab literals,
empty patterns,
and the small-kana,
voicing,
soft-hyphen-only,
and half-expansion negatives.

They differ on 13 cases:

- `canonical-accent`, `plain-accent`: browser folds decomposed accents.
- `case-expansion`: `STRASSE` matches `Straße`.
- `compatibility-ligature`: `office` matches `oﬃce`.
- `dotted-i`: `i` matches `İ`.
- `nbsp-as-space`: ordinary space matches NBSP.
- `kana-script`, `kana-width`, `kana-composed`: browser folds kana
  script/width and composed versus combining voicing marks.
- `single-quote`, `double-quote`: browser folds curly and straight quotes.
- `soft-hyphen`: browser makes soft hyphen ignorable inside a word.
- `combining-mark-only`: browser rejects a lone combining mark while
  regex matches it, so this is a false positive in the incumbent.

Conclusion:
literal escaping plus Unicode case folding does not reproduce
Chrome's ICU collation search.
Under the "Decision" section that gap is accepted:
these cases are the documented deliberate differences,
and `tests/find_reference.rs` pins the exact differing set so a behavior change is noticed.

## Next verification

- [x] Run `inspect:find-regex` to measure the existing Helix regex engine
  against the captured corpus.
  Result recorded in “Comparison result”.
- [x] Decide the matcher:
  plain literal case-insensitive substring matching,
  per the "Decision" section.
  The editord line-DOM probe and the technology-vetting run are dropped.
- [x] Implement the native find bar,
  worker,
  and match painting,
  with match ranges tagged by file generation,
  content revision,
  and query identity.
  Landed on `main` as `f51495788` through `4e1c2a808`.
- [x] Record the browser-find differences in `package/ide/app/README.md`.
