# First-run access and scoped no-audio design evidence

## Purpose and authorization

The human directed continued substantive design work after the filename
question was withdrawn.
This study advances D10's unresolved no-system-library/declined-source
surface using D10/D27's settled actions,
not another cosmetic ballot.
It changes no production code,
source-status API,
real library,
original AVD,
Search policy or template default.
No new IME or TalkBack experiment was performed.

The [continuation plan](../../../../doc/planning/music-player-first-run-access.md)
records fresh source findings and the independently verifiable queue.
D10 supplies `Open a folder`,
in-app Settings and an up-front explanation of analysis.
D27 supplies automatic use of an available system library and a separate
choice before full-library analysis.
No scan/dismiss answers are changed.

D84 (2026-10-05) supersedes that separate choice:
true-peak analysis is automatic and not optional,
so no ask-before-analysing prompt and no scan/dismiss answer remains.
The study first published here ended its explanation with
`After opening a library, choose whether to analyse it.`,
which D84 contradicts.
That study was left alone at first on the ground that its review verification bound the page by hash.
D88 (2026-10-05) rules that out:
nothing is locked by hash.
The study was rebuilt and republished the same day.
Its explanation now reads
`True peak is measured automatically for every audio file. Analysis uses CPU and battery; playback remains available while it runs.`
That wording is study copy that no decision chose;
D84 decides only that analysis is automatic and not optional,
and when analysis runs is undecided.

D95 (2026-10-06) says a library is always open:
when no session is stored the app opens the system's default library.
D100 keeps the state where Android's music permission is refused,
shown as that library open but unreadable,
offering to grant access or to open a folder,
and drops the state with no source opened.
D10's empty state now applies only to the declined case.
The study was rebuilt for that on 2026-10-06.
The declined state's wording,
button order and blank left half are the agent's version,
proposed in the continuation plan under `Rebuild for D100`,
and the human approved them as built (D101).

## Fresh source boundary

The source audit at `0c391a82556eeda61a285de2d70c1270fbc8994d` found
separate limitations,
not one common failure:

- `MainActivity.appRoot` renders only the broad audio-permission gate when
  that permission is absent,
  even though `LibrarySource.load` prioritizes a held folder grant.
- `LibraryRoot.heldRoot` clears a saved URI without a live persisted grant;
  its absent result does not distinguish revoked from never chosen.
- Whole-folder failures and skipped child reads can produce empty/partial
  track lists without an equivalent complete/partial/failure UI outcome.
- The launch path's automatic initial analysis has a separately accepted
  provisional runtime in `package/music-player/android-app/DECISION.peak-sweep-parallelism.md`,
  with better launch UX deferred.
  It is not labelled an unauthorized regression or reverted here.

The exact executable branches and current operation owners are cited in
the continuation plan.
No real picker,
permission recovery,
Settings-return refresh or analysis-consent transition was exercised.
A folder change changes source scope;
it is not repair of the previous source.

## Authored native fixture

The owned branch `prototype/music-player-first-run-access` starts from
`a5560abb223af9f700b9d9465eac1991a02aac07`.
The first publication captured prototype
`a534bf5ac985e091cc93d6c663c24d72ad82a91c`,
and the rebuild after D84 captured
`61e2cf7764f627d40a77bdf6039143e229ad36ea`.
This study captures prototype
`7a2ea6888cce59c9db23d556a520f4c60f8824c6`,
with APK SHA-256
`060ab216713c070c928e9ce2982748f84c3174b0d4f0dfe3beb79e43979a0c4c`.
The commit and digest say what produced the captures;
nothing is checked against them (D88).
This is a different source boundary from the current-production audit,
not a current `MainActivity` equivalence claim.

`FirstRunAccessActivity` inherits `ComponentActivity`,
not `MainActivity`.
The merged debug manifest disables the production launcher,
playback service and sweep service and removes automatic WorkManager
initialization.
Per-frame service dumps and private launch logs showed no observed
playback or sweep service during the accepted frames.
The native buttons have private debug callbacks only;
no real source operation,
permission request or settings page is connected or accepted.
Their callback activation was not studied.

Exact states are:

- Device-wide music access declined:
  the device music library is open but cannot be read.
  Its title is `The device music library can't be read`,
  and it offers `Allow access` as the filled primary,
  then `Open a folder` and Settings outlined,
  above the explanation of analysis.
- An explicitly authored complete device-music-library read with no
  eligible audio,
  not proof that the device has no music or files.
- An explicitly authored complete chosen-folder read with no eligible
  audio,
  naming only that folder and keeping source changes explicit.

There is no state without a source (D95).
The fixture refuses the scene name `not-opened` like any unknown scene.

The host-JVM fixture tests cover those scene branches,
the declined state's exact copy and actions,
complete/zero audio,
complete/nonzero audio,
partial reads with and without results,
failed/unread scopes with both counts,
unknown coverage,
negative counts and unknown scene rejection.
All 12 first-run tests passed,
and the app's whole unit task passed with 250.
These are fixture proofs,
not native discovery or gesture tests.

## Captured cohort and visible boundaries

This section describes the study rebuilt for D100.
Earlier cohorts are in git history.

The owned disposable Fold captured 24 first views:
the three authored states on the inner and cover panels,
in light and dark,
at 100% and 200% native text.
Every acquisition used a fresh hierarchy path,
a successful dump,
a closed keyboard,
stable app frames and a host created exactly once since its launch.
Three system dialogs were answered during the capture;
each scene a dialog covered was captured again,
and no kept view shows one.

After each first view the content was dragged upward until app pixels
stopped changing.
At 200% text the declined state moved on both panels,
so its end view is kept:
4 end views in all.
On the inner panel the explanation's rectangle ended at 2074 physical pixels,
where the navigation area begins,
and after dragging it ends at 2015.
On the cover panel it already ended at 2325,
above the navigation area at 2365,
and dragging moved it by 19 pixels.
Every state that moved carries the hierarchy's scrolling mark,
and every state that did not move carries none.
An unmoved state is still not evidence that a gesture was delivered.

The views come from 4 emulator visits,
and each view in the
[witness manifest](../questions/evidence/first-run-access-witnesses.json)
names its visit.
All of them ran the same installed APK,
and each view was checked in its own environment.

All 28 published app-area regions were inspected in full,
as light and dark pairs,
including the unused left side of the inner panel and the navigation area.
Every view draws its title,
body and buttons;
the declined state also draws the heading and the explanation of analysis,
and no view draws the withdrawn sentence or a state without a source.
Every view draws a gesture handle in its navigation strip.
This inspection does not verify the buttons' actions,
48dp action bounds,
contrast or accessibility focus.

Status strips were removed at the height the system reported for each panel:
136 inner pixels and 152 cover pixels.
Retained sizes are 2076 × 2016px and 1080 × 2272px at 390dpi.
Exact retained RGB,
changed-byte sensitivity,
opacity,
essential PNG chunks and dimensions passed.
No digest of an image is recorded,
and nothing is compared with a digest (D88);
the manifest states the APK digest and commit that produced the captures.
Raw status-bearing frames,
hierarchies and logs remain private.

## Review and lifecycle

[The first-run evidence review](../questions/first-run-access.html)
shows the three states and the declined state's version as approved (D101).
It asks nothing;
its last field takes optional observations.
It says that analysis is automatic and not optional,
and it shows a second view only where content moved.
[The verification record](../questions/evidence/first-run-access-review-verification.json)
quotes what the tasks printed and holds no digest (D88).

The builder derives the cohort from the captures and checks each image's
geometry and drawn text from the image and its record.
It refuses a view that draws the withdrawn sentence,
a no-audio state that explains analysis,
a declined state without `Allow access`,
`Open a folder` and Settings,
a missing state,
a drag outcome that disagrees with the kept views or with the
hierarchy's scrolling mark,
a page that names a visit count its views do not have,
a figure for a state that is not authored,
a state without its figure,
and a page without the section on the approved declined state.
`test:first-run:access` rejects 14 changed manifests and 4 changed pages,
accepts a synthetic second view with a displaced body and refuses one without,
and rejects a ballot,
the withdrawn sentence in the page and a changed output.
With each of five named guards deleted from a disposable builder,
the test failed on the matching rejection.

All 28 images decoded and opened in an offline Chromium check
in a container bounded to 2 GiB of memory and 2 CPUs.
The shown views,
the drag status line,
optional empty and blank observations,
inert adversarial notes and one modal with zoom,
pan,
reset and focus return were exercised in each of four contexts:
desktop and phone width,
light and dark.
Four closed-page A/AA axe audits had zero violations or incomplete checks,
and no console errors were observed.
A desktop light and a phone dark screenshot were looked at;
the phone one first showed the drag status counting 4 states after D100 left 3,
which the page and the check now count from the page's own figures.
No open-dialog audit or Firefox acceptance is claimed.
These checks verify this page,
not source recovery or native accessibility.

The native runtime was capped at 6 GiB and 2 CPUs.
While the visits ran,
`systemd-oomd` killed the session's shared command cgroup several times,
which other sessions' builds had filled to its limit;
one kill took the loop that ran the visits,
and its emulator was restored and stopped from a separate user unit.
The visits then ran as their own user unit,
outside that cgroup.
The last visit restored the recorded fields to the first visit's baseline
(text size 100%,
light appearance,
panel opened,
accessibility off) and read them back,
and container and matching emulator process absence were verified.
`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`
records the kills.
Separate device base-state and override configuration were not retained
individually,
so full equality of that configuration is not asserted.

## Remaining implementation gate

Design evidence does not create production source-status outcomes or
permission recovery.
Binding each state needs complete/partial/failure provenance for the
assessed scope and the current source identity.
Picker cancellation must not be treated as source success.
Changing a folder must not silently widen it to device-wide media.
No production implementation or default-template selection is authorized.

No new preference question is justified by this study.
The human closed custom Android media-notification design in D82:
accept the standard platform presentation.
No notification variants or visual preference question follows.
When this study was first published,
the next design item was the keyboard-map revision in `open-questions.md` section 6.
The current queue is in `HANDOFF.md`.
