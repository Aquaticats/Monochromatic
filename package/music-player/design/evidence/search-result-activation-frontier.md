# Search result activation evidence for D72 to D74

## What the rows currently mean

D47 gives Search a separate destination.
D48 keeps Back,
query and Clear together;
D51 retains the actual browser and complete deck beside unfolded Search.
D52 removes a redundant positive-results heading.
D58/D59 align and highlight results,
D60 limits membership to direct folder/track names,
and D61 mixes result kinds by illustrative relevance.
D63 to D68 settle entry focus,
visible Back,
Clear,
re-entry and same-query row visibility.
D69 to D71 separate unqueried,
completed-no-match and known unavailable statuses.
Those earlier decisions did not choose activation.
D72 to D74 now select the desired folder/track effects and return path,
but do **not** make the static rows actionable.
The present `question/current.html` screenshots are fixture states;
result captions remain unimplemented examples.

The debug-only
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/SearchPersistentDeckStudy.kt`
on `prototype/music-player-theme-compose` renders unclickable result lines.
An earlier two-row `cam` fixture displayed `Another Xronixle` through its
parent name;
D60 later excluded that parent-only track result.
The corrected D/M fixture instead shows directly named `Cam` and
`Camellia` results in mixed order,
but it still has no result click handler.
Its visible current deck track `Another Xronixle` is not a selected
`cam` hit.
No native screenshot or hierarchy proves a folder tap,
track tap,
queue effect or focus transition.
The installed corrected fixture APK had SHA-256
`1caee7060acfb5bbcd9a02142b4c9bada6b5886517157d25fef4a48b9b1e9c05`;
its static Search-status captures likewise do not test result activation.

## Existing player actions supplied the Search-design precedent

`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlayerController.kt`
implements `selectPage(page)` by updating the selected page,
its visible items and queue page scope,
without loading a replacement track or immediately changing playback.
`playIndex(index)` instead moves the queue cursor and starts that track.
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt`
routes an already-current player-list row tap to `togglePlay()` and
another player-list track row to `playIndex(item.index)`.
This is source-audited behavior for the existing list,
not a settled Search-row action.
If Search opens a different folder,
the queue scope could change even while the current audio keeps playing;
“opens folder” must not be described as “plays folder.”
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/Queue.kt`
changes page scope when `playIndex` receives an index outside that scope.
`PlayerController.playCurrent()` loads and plays the track and calls
`refresh(followCurrent = true)`,
which selects the containing page and refreshes visible items.
This source path supports an owning-folder selection in player state;
it does **not** establish scroll-to-row,
focus placement,
a native Search tap or how the right pane looks if Search stays open.

D51's left browser remains visible on the inner panel,
but the cover Search occupies the full width.
A folder result updating the left browser **without closing Search** would
therefore have no simultaneous counterpart on the cover.
Returning to the player and selecting the folder would work consistently
across panels,
at the cost of leaving the Search result list.
D66 already says opening a new Search visit starts with an empty query;
that rule alone does not choose whether activation ends the current visit.
The visible Back arrow is a separate explicit exit (D64).

## Incumbent effects and selected consequence

- **Folder result (D72):**
  return to player showing the selected folder,
  without autoplay.
  `selectPage` already performs the selection and queue-scope change;
  closing Search exposes the folder's normal track view on both panels.
  Updating the left browser while retaining Search would obscure that
  folder's track view on the inner panel and be invisible on the cover.
  This coherence recommendation is now the D72 design direction,
  not evidence of an existing Search handler.
- **Other track (D73):**
  start the directly named track using the
  existing non-current player-row semantics.
  `playIndex` and `playCurrent` also select the track's owning page in
  source;
  they do not prove a scroll or focus jump to its row.
  A “reveal but do not play” workflow would be new behavior without a
  prior user signal.
- **Already-current track (D73):**
  carry the existing player-row
  play/pause distinction into a Search result for a query that actually
  matches that track.
  `togglePlay()` pauses when playing,
  resumes the already-loaded current URI when paused,
  or loads it when necessary.
  Passing the current index to `playIndex` would instead follow its load
  path;
  do not label that route a toggle.
  The historical `cam` fixture's current deck track is **not** a valid
  result to test this case.
- **Track navigation (D74):**
  after a successful **track** result action,
  the user selected **Return to player**.
  The normal folder track view appears;
  a later Search visit begins with an empty query under D66.
  This applies to a different track starting and the current track
  toggling,
  as previewed by the logic comparison.
  D51's retained deck made Stay coherent but did not make it mandatory.
  A failed/stale tap does not count as a successful action.
- **Unavailable/stale target:**
  a result may disappear before activation.
  Treat that as an action failure requiring a truthful handler,
  not another user-preference axis.
  Do not substitute a different item,
  claim playback started or turn a failure into a no-match diagnosis.
  D9 already drops a vanished file from the list and uses a dismissible
  bar rather than leaving a dead row in place;
  that is the incumbent presentation to reuse where its cause matches.
  The actual Search action failure,
  bar timing and collision with another transient message still need
  implementation-boundary verification.

A visual/logic comparison must start from selected A with its real left
browser and full deck,
not the obsolete blank-left or two-row parent-only `cam` fixture.
The [archived logic-only interaction comparison](../question/archive/search-result-activation-before-return.html)
models a directly named `Cam` track,
a `Camellia` folder and an `Another Xronixle` current-track hit
under an `Another` query.
The user selected Return from its historical Stay/Return comparison;
D72/D73 carry incumbent folder/current-track meanings and show an
illustrative D9-style stale-result notice.
Its player,
queue and navigation outputs are a model,
not Android verification;
it does not simulate keyboard edit focus,
TalkBack,
file loading or a real tap handler.
Native TalkBack traversal and announcements remain the separate Search
accessibility review;
no further IME study is authorized without first making a compelling case
to the user.
