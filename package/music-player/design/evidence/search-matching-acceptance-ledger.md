# Selected Search visuals and deferred engine distinctions

## Why this ledger exists

D60 selected matching each item's **own** folder name or final track
filename component,
not a parent folder that merely contains a track.
D61 selected mixed cross-type relevance,
not a complete comparator.
D59 settled in-place highlight of visible query substrings from the
OS accent in OKLCH.
The accepted `cam` result examples constrain more behavior than the
phrase "matching grammar remains open" suggested.
This ledger keeps selected **visible outcomes** separate from
illustrative debug captions and eventual engine behavior.
D62 records the user's correction that selecting a fuzzy search library
is not part of the current UI review and need not change the UI.
The engine distinctions are not a new user-choice menu here.
No production Search implementation is authorized by these examples.

`Track.displayPath` is a source-root-relative slash path
(`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/Track.kt`).
`PageEntry.name` may be a queue-relative path such as
`Artist/Album/01.flac`
(`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/Page.kt`).
Neither entire path is equivalent to the final filename component.
A parent path can identify and distinguish a track without generating a
Search match under D60.

## Settled visible outcomes

- Query `cam` returns a directly named track `Cam` and highlights all of
  its title.
  The user's `cam`/`Cam` correction rules out a
  case-sensitive-only result for this example.
- Query `cam` returns the folder `Camellia` and highlights the `Cam`
  prefix.
  Whole-name exact-only matching is inconsistent with the selected
  design.
- Query `cam` returns the direct track `Live at Camellia` and highlights
  `Cam` at a **later word start**.
  Whole-name prefix-only and whole-word-only alternatives are
  inconsistent with the selected design.
  This example does not prove matching inside an uninterrupted word.
- Both `Camellia Waltz` tracks in different folders remain separate
  results;
  their parent labels distinguish them.
  A matching parent label can also contain a highlighted `Cam` without
  becoming an eligibility field.
- `Another Xronixle` is **not** a Search hit for `cam` solely because it
  belongs to `Camellia`.
  Its name remains on the retained playback deck as the current track.
- The exact `Cam` track precedes the prefix `Camellia` folder in the
  accepted mixed fixture.
  This outcome does not select numeric weights or a tie-break.

The current native selected-only evidence is
`package/music-player/design/question/current.html`;
`package/music-player/design/evidence/search-match-emphasis-native.md`
records after-state bounds and image provenance.
The D/P and M/F/T comparisons are historical at
`package/music-player/design/question/archive/search-ranking-before-dm.html`.

## Deferred engine distinctions, not current UI questions

### Middle-of-word eligibility

`Scamper` includes `cam` starting after `S` inside one uninterrupted
word;
`Dreamcam` ends with the same fragment.
`Live at Camellia` is not a control for this case because `Cam` starts
a separate word.
An [archived W/A native exploration][word-review] displayed a direct
`Scamper` folder and `Dreamcam` track while retaining accepted controls.
The user did **not** select W or A;
D62 removed the library-driven W/A choice from the UI frontier.
Its appended fixture order does not decide either result's rank.

### Filename extension and exactness

The visible `Cam` fixture is extensionless,
while the actual library can expose final filenames such as `Cam.flac`.
Future implementation must distinguish:

- extension-only hit eligibility from a query such as `flac`;
- whether query `cam` treats `Cam.flac` as an exact stem or as a prefix
  of the full filename;
- the rendered title's extension presentation,
  which is a separate visual concern and is not automatically the match
  field.

A path segment like `Album` may appear beside `Cam.flac` as context,
but `Album` cannot itself generate this track hit under D60.

### Unicode and query terms

The accepted ASCII `cam`/`Cam` example does not settle canonical composed
versus decomposed spellings,
accent-sensitive versus accent-insensitive matching,
locale-specific case mappings or scripts without letter case.
A multiple-term query such as `live cam` might be parsed differently by
an eventual fuzzy engine.
These examples bound future implementation research,
not questions to put to the user in the current UI session.

### Deterministic ties and result volume

M rejects global folder-first and track-first grouping,
but does not select how two equally relevant titles from different
folders are ordered.
An existing library path order,
a basename order,
a locale-aware order and type-as-tie-break are distinct policies.
A stable order is required so results do not jump between repeated
queries;
its concrete comparator has not been selected.
No limit,
pagination or streaming behavior for a large library was chosen by
D60/D61.

## Verification boundary

The debug `SearchRankingFixture.kt` hands Compose fixed rows in a fixed
order;
it does **not** implement a matcher.
`SearchFixtureHighlights.kt` visibly colors every literal `cam` range,
but that visual helper is not a Unicode search grammar.
If future engine behavior creates a **visible** conflict with D59,
D60 or D61,
return that concrete state for design review while keeping the actual
browser/deck and nonfunctional captions clear.
Do not proactively choose a library,
ask W/A,
or turn illustrative captions into navigation or playback now.

[word-review]: ../question/archive/search-word-boundary-deferred.html
