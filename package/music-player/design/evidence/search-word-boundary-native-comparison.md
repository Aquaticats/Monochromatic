# Historical word-boundary fixture before D62 removed the UI question

## Exploratory question and fixed controls

D62 later records the user's correction:
the eventual fuzzy search library is not part of the current UI review,
and this W/A fixture is **not** a question to answer.
Neither W nor A was selected.
D60 searches each folder's own name and each track's final filename
component;
parent and ancestor names cannot generate track hits.
D61 mixes folder and track results by relevance,
with exact `Cam` before prefix `Camellia`.
The user's `cam`/`Cam` instruction and selected fixture also require
`Live at Camellia`,
where `Cam` begins a later word.
D59 keeps matching letters bold on an OS-accent fill adjusted in OKLCH.
The only unresolved dimension here is whether query `cam` should also
find an item's **own name** with those letters inside one uninterrupted
word:
folder `Scamper` or track `Dreamcam`.
The eventual W/A answer is intended for both eligible kinds under D60.
This comparison is not a scorer,
parser,
Search index or result action.

The debug-only `SearchRankingFixture.kt` on prototype commits
`72da1d925` and `cc66a0dcf` returns three settled controls in both
variants:
`Cam` track,
`Camellia` folder and `Live at Camellia` track,
in that mixed order.
The anywhere variant appends one `Scamper` folder and one `Dreamcam`
track;
the word-start variant omits them.
Their appended placement **makes the membership difference visible**;
it is not a chosen ranking tier for interior matches under D61.
This reduced comparison deliberately omits `Camera Obscura` and the two
`Camellia Waltz` direct hits from the selected D/M matrix.
Those items remain accepted matches;
their absence here is not an effect of W.
All five titles are direct names,
not matches inherited from `Cult of Luna` or another parent.
`SearchPersistentDeckStudy.kt` renders the inner list,
then delegates folded-cover rendering to `SearchLayoutStudy.kt`.
The result rows have no click handler;
“exact filename”,
“later word” and “middle of word” are illustrative captions.

## Native evidence and after-state checks

The disposable `Fold_No_Hardware_Probe` had a 2076 × 2152 inner panel,
1080 × 2424 cover,
390dpi and 200% text.
The installed debug APK and local build both had SHA-256
`1caee7060acfb5bbcd9a02142b4c9bada6b5886517157d25fef4a48b9b1e9c05`.
The earlier `5a484da7ad8f46db12d53cc5b84f2f9239f4e9a769486fe3e1804e9d6b46c351`
track-only attempt was superseded before the user saw a question.
Only keyboard-closed light-scheme captures were made:

- [Word start, inner][inner-word] retains the browser and complete deck
  with the three accepted direct-name rows.
- [Anywhere, inner][inner-any] retains that same browser/deck and app
  header,
  then shows an extra folder and track with `cam` highlighted inside
  each title.
- [Word start, cover][cover-word] retains one full-width Search page
  without the interior rows.
- [Anywhere, cover][cover-any] shows the highlighted `Scamper` folder
  and `Dreamcam` track after the shared fixture controls.

Private UI Automator hierarchies on both panels placed the exact `Cam`
first,
`Camellia` second and `Live at Camellia` third.
Only the anywhere variant then showed the `Scamper` folder with its
folder icon/detail and the `Dreamcam` track with its track icon/detail;
neither variant showed the rejected parent-only `Another Xronixle`
Search hit.
Result titles shared the query start x `1249` inner or x `156`
cover.
The last supporting line was fully visible without a swipe on the
observed 200% keyboard-closed panels.
A same-panel native PNG difference check found zero difference over the
inner left browser/deck,
both integrated headers,
and chosen rectangles containing all three shared control rows.
The inner results pane and cover full-width results differed (means
`0.0112196` and `0.0107833` respectively over the measured regions).
That positive control detects a real membership difference,
not a relabeled identical frame.

The sanitizer replaced the entire status region with the same panel's
generic reference,
verified exact app pixels underneath,
and removed PNG metadata.
Raw status-bearing screenshots,
XML and installed-path details stayed private.
The host-JVM debug-variant tests passed for stable D/M controls,
folder/track interior-only membership and highlight offsets
`Scamper[1,4)` and `Dreamcam[5,8)`.
Those tests do not implement a production Unicode word parser.
The source assets are retained in the
[archived comparison][review].
The selected-only [Search review][current-review] continues to show D/M;
it does not assume W or A or direct a user to choose a backend library.

## Unadopted comparison and scope correction

- **W,
  word-start-only fixture:**
  demonstrates the accepted controls and
  excludes the two interior-only names.
  A user remembering an interior fragment would not find them under
  this illustrative rule.
- **A,
  anywhere-substring fixture:**
  demonstrates optional `Scamper` and
  `Dreamcam` rows;
  additional interior matches could vary in usefulness.
  Real-library counts were not measured.

An earlier provisional W > A ranking preferred predictable word starts,
but D62 withdrew the entire W/A UI-choice premise.
These are historical fixture alternatives,
not ranked finalist designs or backend recommendations.
The displayed append position does not select an interior-hit relevance
tier.
Whitespace,
hyphens,
apostrophes,
filename extensions,
Unicode equivalence,
multiple terms and deterministic tie-breaks are outside the present
**UI** review and may be addressed with the eventual library during a
separately authorized implementation task.
No production Search or new IME experiment was authorized.

[inner-word]: ../question/render/search-word-boundary-review-inner-rankword-s200.png
[inner-any]: ../question/render/search-word-boundary-review-inner-rankany-s200.png
[cover-word]: ../question/render/search-word-boundary-review-cover-rankword-s200.png
[cover-any]: ../question/render/search-word-boundary-review-cover-rankany-s200.png
[review]: ../question/archive/search-word-boundary-deferred.html
[current-review]: ../question/current.html
