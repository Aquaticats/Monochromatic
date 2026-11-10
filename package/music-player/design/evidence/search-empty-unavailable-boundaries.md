# Search empty and unavailable state evidence for D69 to D71

## Existing decisions and visible sources

D47/D48 give Search a separate destination and one Back/query/Clear header.
D51 retains the actual folder browser and complete playback deck on the
left of the unfolded Fold.
D52 removes a **positive-results** heading that merely repeats the query;
it does not settle no-match copy.
D60 limits visible hits to direct folder/track names;
D63 requests query edit focus and a keyboard when Search opens;
D65 keeps the current edit-focus/keyboard state after Clear;
D66 starts each new Search visit with an empty query.
D10's player first-run empty view applies when no system library exists or
the user declined access;
it is not automatically the Search no-match view.

The selected-A keyboard-closed
`package/music-player/design/question/render/search-selected-review-inner-empty-light-s100.png`
shows “Search your music” and “Type a name to explore your library” in
the right pane,
with the real left browser and complete deck retained.
It is a debug fixture,
not native proof of D63's newer focus/keyboard request.
The earlier
`package/music-player/design/question/render/fold-search-inner-open-none-light-s100.png`
and
`package/music-player/design/question/render/fold-search-inner-open-unavailable-dark-s100.png`
show “No results for ‘zzq’” and “Library unavailable” in an
**obsolete blank-left** layout.
They preserve historical wording evidence,
not valid selected-A comparisons.
The debug-only
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/SearchPersistentDeckStudy.kt`
on branch `prototype/music-player-theme-compose` (commit `cc66a0dcf`)
sets a fixed
`zzq` query for a `-none` marker;
`-unavailable` overrides results even with an empty query.
The current fixture's unavailable detail says “Search returns when the
library is available” and provides no recovery control.
No real search algorithm,
permission recovery or result activation is implemented by these fixtures.

A subsequent **keyboard-closed** static study used the same corrected debug
APK (SHA-256
`1caee7060acfb5bbcd9a02142b4c9bada6b5886517157d25fef4a48b9b1e9c05`)
on the disposable `Fold_No_Hardware_Probe` inside a measured 6 GiB/2 CPU
container.
`question/evidence/search-status-review-manifest.json` indexes the
sanitized inner 2076 × 2152px and cover 1080 × 2424px rasters for empty
query,
fixed `zzq` no-match and forced unavailable fixtures in light/dark at
100%/200% text.
Every published PNG replaced its complete top status strip with the
previously checked generic 9:41 bar;
the private raw pixels below that strip matched the published app region
exactly,
and output was opaque with metadata stripped.
The [self-contained status gallery](../question/search-status-evidence.html)
allows inspecting each panel at its cited dp size.
All captured inner states retain the actual left browser,
Open control and complete playback deck.
At 200% text the right-side prompt and unavailable detail wrap within
their allotted content region in this keyboard-closed fixture.
The unavailable page simultaneously shows a populated browser and an
already-playing track,
so the broad label “Library unavailable” implies more than the fixture
establishes.
No Gboard,
other IME,
backend Search,
recovery control or D63 automatic edit focus was exercised by these
captures.
The generic fixture copy remains historical,
not selected just because it was captured.

## Distinct evidence states

- **Empty query,
  library usable:**
  no lookup needs to have failed.
  A prompt can explain Search's folder/track scope rather than claim “no
  results.”
  D63's intended keyboard-open entry means a keyboard-closed old screenshot
  cannot certify final fit.
- **Nonempty query,
  zero direct-name hits:**
  evaluation must have
  **completed** for the current query and established source scope with
  enough coverage to assert there are no matching folder or track names.
  Keep the query and Clear accessible;
  a temporarily empty batch,
  stale prior-query results or an incomplete read do not prove no match.
  Folder hits without track hits (or the reverse) are ordinary positive
  results,
  not a no-match state.
- **Confirmed empty searchable inventory:**
  no searchable items are
  established in the selected scope,
  not merely a zero-length track list.
  `LibrarySource.load` returns tracks;
  it does not prove that there are no folder names D60 could match.
  D51's visible browser location does not determine whether Search spans
  the current folder or a wider library.
  D10's player-level empty treatment and Search's right-pane message may
  need different affordances without duplicating them.
- **Known access or source failure:**
  only a genuinely known cause can
  justify “unavailable” and a matching recovery action.
  This state can take precedence over either empty query or no-match
  messages;
  avoid telling the user to try another name when no source is readable.
  A playable file becoming inaccessible is not by itself proof that
  folder/track names cannot be searched;
  row activation belongs to the later result-action review.
- **Source loading,
  partial coverage or refresh:**
  a temporarily empty
  result list is not a completed failed query or an inaccessible library.
  Do not pass off last-known results from another query/source as current
  matches,
  nor claim a full no-match verdict before current enumeration ends.
- **Analysis in progress:**
  peak analysis is distinct from discovering
  searchable folder/track names.
  D27 allows playback before analysis;
  do not gate Search or show “library unavailable” merely because analysis
  is pending.

`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibrarySource.kt`
currently chooses a live held folder **before** checking the device-wide
audio permission,
then uses MediaStore if that permission exists,
then returns an empty list.
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibraryRoot.kt`
checks the persisted folder read grant in `heldRoot` rather than trusting
a saved URI alone.
A denied device-wide permission therefore does not independently prove the
source cannot be read,
while held permission does not guarantee a successful read.
`LibrarySource.scanRoot` also turns a failed whole-folder scan into an
empty list after logging;
its `onBatch` callback permits partial enumeration.
The current
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt`
executable branch,
however,
shows the player only if its own audio permission check passes and
otherwise renders a permission gate.
That current gate does not demonstrate that Search can be opened without
audio permission even when the source-selection seam could see a held
folder.
An empty returned track list cannot distinguish an intentionally empty
folder,
no permission,
incomplete enumeration and some scan failures,
let alone establish that no folder names are searchable.
A designed unavailable state must be conditional on actual source status
available to the eventual implementation;
static text must not pretend that a missing status already exists.
This is an evidence constraint,
not authorization to change the production library seam now.

## Truthful transitions and recovery

The fixture's “Search returns when the library is available” implies a
status change is detected and the current query is reevaluated.
Its fixed unavailable marker demonstrates none of those mechanisms.
Retry cannot be promised to restore a revoked grant;
choosing a different folder changes scope rather than repairing the
original one;
Clear cannot repair unreadable storage;
opening Search again need not rescan successfully.
A recovery button belongs only to an identified cause and an available
owner with a verified action.
Do not widen an intentionally chosen empty folder to device-wide media
without an explicit separate decision.

When source status changes during a visit,
D63 to D66 still govern focus,
Clear,
visible Back and fresh-query **re-entry**.
An outage during typing must not erase the query or close Search;
Clear during an outage must not relabel the source usable;
a source becoming usable while Search stays open is not a D66 new visit;
a status refresh must not steal edit focus or reopen a dismissed keyboard.
D68 only concerns the same result set through keyboard refocus;
it cannot ensure that a removed result row remains visible after the
result set changes.
Retain the actual left browser and deck without portraying stale browser
content as newly verified or declaring an already-playing stream stopped.

## Selected UI direction and remaining implementation evidence

D69 requires a completed current-query/source evaluation before a
no-match verdict and distinguishes that verdict from an unqueried prompt,
confirmed empty searchable inventory and known source failure.
D70 retains the selected-A unqueried prompt and a no-match diagnostic
that names the current query and offers another-name guidance;
the static `zzq` capture is wording/fit evidence,
not a real Search outcome.
D71 rejects the generic unavailable/recovery fixture copy.
A Search-specific failure explanation must name an actually known cause
and show a recovery action only when its owner can perform that action.
The exact cause-specific text/control awaits a real source-status signal;
no generic Retry,
permission request or new folder operation is selected by a fixture.
These are design decisions with a veto path,
not production code or a new user questionnaire.

Do not replace the left folder browser/deck with the obsolete blank-left
fixture,
introduce a positive-results heading,
reopen matcher-library selection,
or settle result-row actions or TalkBack behavior.
Any Android screen comparison requires captures from the disposable Fold
at physical panel pixels,
with the same selected A/E2 context and status-strip sanitization.
No new IME experiment is authorized without first making a compelling
case to the user.
The repo's bare `#127` to `#129` and `#118` Markdown references are
**internal review-task numbers**;
GitHub currently has unrelated issues at those numbers,
so future design prose should name the review area or say “review task”
instead of generating misleading issue links.
