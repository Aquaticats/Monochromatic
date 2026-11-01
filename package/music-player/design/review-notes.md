# Defects found in phase one, and the rules that follow from them

Each entry is a real failure in this project's output,
 with the rule it produced.
They are written out in full because the rules only make sense next to the concrete
thing that went wrong.

---

## 1. Spec numbers were invented from memory, and three were wrong

**What happened.**
 Components were built to remembered Material Design 3 figures.
Three were wrong:
 two-line list rows were built at 56dp (the spec is 72dp — 56dp is
the *one*-line height),
 segmented buttons at 48dp visual height (the spec is 40dp
visual with a 48dp tap target),
 and slider handles at 4×26px (the updated M3 slider
handle is 4×44px on a 16px track).

**Caught in review** — on the grounds that neither side should be trusting a
memorised spec when the real values are a lookup away.

**The rule.**
 Read the real values.
 `m3.material.io` is JavaScript-rendered and cannot
be fetched,
 but **Google publishes the generated token files** in
`github.com/material-components/material-web` under `tokens/versions/latest/sass/`.
Those are the spec,
 machine-readable.
 md3-tokens.md records everything already
extracted and shows how to look up more.
 If a number is not in there and not read
from the repo,
 do not write it into a design.

---

## 2. Device dimensions were invented — including inside the question asked about them

**What happened.**
 Foldable layouts were drawn at 924×600 and 370×760 — numbers with
no source.
 Worse,
 the question that finally asked which device was being targeted
offered answer options containing made-up dp figures ("cover ≈390×844dp") — so the
guesswork was baked into the question itself.

**Caught in review**:
 the stated measurements did not match the rendered layouts.

**The rule.**
 Get the device model,
 then derive dp from published px and density.
device-metrics.md has the real numbers for the Pixel 9 Pro Fold and the arithmetic
used to get them.
 The inner display is **essentially square** — every earlier mockup
used a wide landscape frame and was structurally wrong,
 not just slightly off.

---

## 3. Candidates were presented without ever being looked at

**What happened.**
 Several designs went out with wide squashed play buttons —
a 96×60 rounded rectangle where a circle belongs — and later with a transport row
whose prev/next buttons were flung to opposite screen edges.

**Caught in review**,
 twice — first the fat transport buttons,
 later a layout that
was simply broken on arrival.

**The rule.**
 Screenshot every candidate and look at it before it goes in front of the
user.
 Note that **iframe content is not captured** by the screenshot tooling,
 so a
contact sheet of candidate files renders blank — check files one at a time.

---

## 4. Bulk find-and-replace was run without re-verifying the result

**What happened.**
 A batch edit across ~30 files fixed the play buttons by adding
`margin:0 auto` to centre them.
 In a flex row,
 `margin:0 auto` consumes all free space
on both sides — which pushed the neighbouring buttons to the container edges and broke
every single-column layout.
 It shipped.

**The rule.**
 After any bulk edit,
 open at least one file per affected layout family
and look at it.
 Batch edits are fine;
 batch edits with no verification are not.

---

## 5. The 1,000-folder scale rule was forgotten repeatedly

**What happened.**
 Folder-picker and fold layouts were repeatedly demonstrated with a
dozen folders,
 when the real library has ~1,000.
 It had to be raised in review more
than once,
 and cost two rejected candidates.

**The rule.**
 Every folder surface is shown at ~1,000 entries,
 using
candidates/artists.js.
 The interesting design problems (jump strip,
 letter targets,
wrapping,
 counts) only exist at that scale.

---

## 5b. The 1,000-folder rule was misread as "show 1,000 rows" — three times

**What happened.**
 unf-b,
 unf-c,
 unf-d and unf-e all rendered every folder as its own
48dp row and then bolted an index onto the side.
 The user,
 third time:
 "One sub-dir
per row would make the scrolling area absurdly large.
 We should be prepared to handle
1k sub-dirs."
 Decision D3 already said it — "wrapped 48dp targets,
 not a single narrow
column" — and picker-f already demonstrated it.
 Neither was read closely enough.

**The rule.**
 The 1k rule means the picker must **cope** with 1,000 folders,
 not
**list** them.
 A folder surface never shows more than about one screen of names:
filter to a letter (and to a 2-letter bucket when a letter has more than ~24 names),
then lay the names out as wrapped,
 content-width chips.
 Scrolling a thousand rows is
the failure,
 not the demonstration.
 unf-f is the first candidate to get this right.

## 5e. Question forms went out with no free-text field

**What happened.**
 The standing rule above was already written down,
 in softer words
("every question offers a way out"),
 and two forms in session 3 went out as pure
option pickers anyway — the picker-verdict form and the round-two follow-up.
 The user:
"I also said a standing rule is to always include free text input for me to fill out
for any option in a prev session,
 but clearly that didn't make it to the docs as
well."
 Both times the user had to type the real answer into chat instead,
 and both
times that answer was the one that mattered:
 the chips objection was about styling,
not layout;
 rows were dead regardless of column count.

**The rule.**
 A form without a free-text field is an unfinished form.
 Write the
free-text question first,
 before the options,
 so it cannot be forgotten at the end.
Where a form is a follow-up round,
 add a `user-questions` item too.

## 5d. Rejected patterns came back because the notes recorded the objection loosely

**What happened.**
 Two of the user's rejections were written down in these documents in
a narrower form than they were meant:

* "we don't really need this as tabs" (about the CA / CH / CL–CO split) was recorded in
  D17 as *no sub-buckets as tabs*.
   The segmentation itself was what died.
   It came back
  as a rail accordion in pk-d.
* "picker names not as a wall of chips" was recorded as presentation being "open" — and
  then written into D28 as *chips are still the presentation*.
* One name per 48dp row had been rejected for the long scroll (5b) and was rebuilt
  three more times (pk-a,
   pk-b,
   pk-c),
   then twice again in different column counts
  (pk-e,
   pk-f),
   each time as though the count of columns were the objection.

The user,
 on the fourth:
 "Rows again?
 That's also killed,
 at least 4 times today."

**The rules.**
1. **Record a rejection as a NO,
    not as a nuance.**
    Write the general form of what died
   ("no sub-letter segmentation,
    in any form"),
    then the quote,
    then the specific
   candidate.
    If the general form is not obvious,
    ask which it is before writing it.
2. **Keep the NO list next to the decision** — D17 now carries all three picker NOs.
3. **Check a new candidate against the NO list before showing it.**
    Every rejected pk-*
   candidate would have failed that check in one line.
4. **A rejection can be about the styling and not the layout** (chips),
    or about the
   layout and not the styling (rows).
    Ask which,
    rather than assuming the whole
   treatment died.

## 5c. Device layouts were judged in a stretched preview

**What happened.**
 The file-options preview stretched a 411×923 layout to fill a large
window.
 The user saw a distorted render and reasonably asked whether it had been
checked.
 **The rule.**
 Every device candidate renders inside a fixed device-size frame
that scales to fit (see unf-f,
 cover-c,
 desc-d for the pattern).

## 6. Questions were asked that the design had already answered

**What happened.**
 A whole round was spent asking what should happen after the last
track in a folder — but the mode control (Repeat / In order / Shuffle folder /
Shuffle all) already answers it.
 The round produced two candidates that were moot
before they were built.

**The rule.**
 Before asking,
 check whether an existing decision determines the answer.
Every question must change what gets built next.

---

## 7. Questions were asked in words when only a demo could answer them

**What happened.**
 Several rounds of prose questions about things that can only be
judged by looking:
 which design generation,
 which row density,
 how the list behaves
when playback moves.
 Each round had to be redone as built demos.

**The rule.**
 If the question is about appearance or behaviour,
 build 2–3 real
candidates first and ask by showing them.

---

## 8. A layout bug that will recur if you do not know about it

`height:100%` on the root of a Design Component resolves against nothing,
 so
fixed-proportion layouts collapse to zero and overlap.
 Every candidate in this project
had this bug at one point.
 Use `height:100vh` plus flex with `min-height:0` on
scrolling children.

---

## 5g. A light-theme study was shown on a dark desk, and never looked at

**What happened.**
 The light-theme candidates were correct inside the device frame,
 but
every frame sat on a #4A4458 purple desk with a near-black caption bar,
 and the frames
only filled the top third of a tall preview — so two thirds of the screen was dark and
the study read as "not a light theme" at a glance.
 The user,
 having already asked five
times for output to be checked:
 "The light theme isn't light theme.
 Did you even check
your own output as I told you the 5th time?"

**Second and third failure,
 same question.**
 The corrected files were shown in a
question form,
 and all three light candidates rendered DARK in their thumbnails.
 Two
separate causes,
 and the first fix did not address the real one:

* the token sheets were linked `../_ds/…`,
   which does not resolve in every context
  that renders a candidate;
   and
* **the real cause** — the design system publishes its dark scheme under
  `[data-theme="dark"]`,
   the HOST sets that attribute on `<html>` when the user's app
  is in dark mode,
   and custom properties inherit.
   So a "light" study built on
  `var(--md-sys-color-surface)` renders in the DARK scheme on a dark-mode host,
   while
  looking perfectly light in a light-mode preview.
   Baseline fallbacks did not help:
  the variables were defined,
   just defined dark.

The fix that holds:
 **declare the scheme inline on the study's own root element**
(`style="--md-sys-color-surface:#FEF7FF;…"`),
 where nothing upstream can override it,
and give `html`/`body` the literal desk colour since they sit above that root.

**The rules.**
0.
 **A design-system reference must resolve from wherever the file is rendered,
 and
   must not inherit the host's scheme.**
 Copy the token sheets next to the candidates
   and link them without `../` traversal;
 pin the intended scheme's roles inline on the
   design's own root element;
 keep baseline fallbacks in every `var()`.
 Then verify by
   forcing the opposite scheme on `<html>` (`data-theme="dark"`) and screenshotting —
   a light study must stay light.
 Neither a missing stylesheet nor a dark-mode host may
   be able to invert a theme.
1. **A theme study is presented on that theme's own desk.**
    Light candidates get a
   light desk,
    a light caption bar and light badge text;
    dark candidates get a dark
   one.
    The surround is part of what the user judges — it is never left at whatever the
   previous candidate used.
2. **Frames fill the viewport.**
    A fixed design height scaled to fit leaves dead desk
   below it,
    which dominates the impression.
    Let the frame take the available height.
3. **Look at the rendered output before asking about it** — and when the question is
   "does this read as X",
    look at it in the USER's view,
    at their window size,
    not only
   in a preview iframe of a different shape.

## 5f. Rules were written down in a form too soft to be followed

**What happened.**
 Three of the user's rules were recorded as hedged prose,
 and each
was then broken by someone reading those exact words:

* "every question offers a way out" — read as a preference;
   two forms went out with no
  free-text field at all (5e).
* "no sub-buckets *as tabs*" — read as banning one presentation;
   the segmentation came
  back as a rail accordion (5d).
* "picker names not as a wall of chips" was filed under presentation being *open* —
  and then written back into a decision as chips (5d).

The user:
 "Record that never to record rules as 'soft' again."

**The rule — how a rule gets written here.**
1. **Imperative,
    absolute,
    no hedging.**
    "Every form ends with a free-text field.
    No
   exceptions."
    Never "usually",
    "generally",
    "prefer to",
    "where it helps",
    "consider",
   "as a rule of thumb",
    or "when the options feel thin".
2. **Write the general form,
    then the quote,
    then the instance.**
    The general form is
   the rule;
    the quote is the evidence;
    the candidate name is only an example.
    A rule
   recorded as its instance shrinks to that instance.
3. **State the scope explicitly when it is narrower than the words** — "this is about
   the styling,
    not the layout" — rather than leaving the reader to guess which half
   died.
    If the scope is not known,
    ask before writing it down.
4. **No rule lives only in prose.**
    It goes in the numbered standing standards below or
   in a lettered decision,
    where it can be cited and checked against.
5. **Anything the user calls a rule,
    a constraint or a standing instruction is
   absolute** unless they later relax it — and a relaxation is recorded the same way.

## 5h. The cost of this session, measured

Nine question forms went out in one session;
 five had to be re-asked,
 none because the
user changed their mind:

<table>
<thead>
<tr>
<th>#</th>
<th>What was asked</th>
<th>Why it was wasted</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>pk-a/b/c picker presentations</td>
<td>rebuilt a pattern rejected 3× already (rows) — 5d</td>
</tr>
<tr>
<td>2</td>
<td>pk-e/pk-f</td>
<td>rows again, in different column counts — 5d</td>
</tr>
<tr>
<td>3</td>
<td>light/dark/dbtp studies</td>
<td>one file per question, so no actual choice to make</td>
</tr>
<tr>
<td>4</td>
<td>same, split into files</td>
<td>light studies rendered dark on the user&#39;s host — 5g</td>
</tr>
<tr>
<td>5</td>
<td>same, with my picks</td>
<td>pros and cons were in chat, not where the user was looking — 5e/6</td>
</tr>
</tbody>
</table>

Three of the five were failures of **checking**,
 not of design:
 the candidate was never
compared against the recorded NOs,
 or never looked at in the user's own view before the
question went out.
 The user,
 mid-session:
 "Did you even check your own output as I told
you the 5th time?"
 and,
 at the end:
 "I think we both might not be feeling well today,
given how many mistakes we made."

**The rule.**
 Before a form goes out,
 three checks,
 every time:
1. Does each candidate violate anything on the NO list for that surface?
    (D17-style
   NOs,
    kept next to the decision.)
2. Has each candidate been rendered and looked at — in the user's view,
    at their
   window size,
    in their colour scheme?
3. Does each option carry pros,
    cons and my pick,
    in list form,
    in all three places
   the user might read them?

## 5i. A visual decision round was delivered as chat text

**What happened.**
 Session 5 resumed from the archive and correctly found the two
built theme choices,
 but presented their options,
 assessments and recommendations as
a long chat prompt.
 The user:
 "Ask me properly via a self-contained HTML rendering
you then open in Helium."

**The current rule.**
 Every design-question round is one self-contained HTML form.
The user withdrew the general visible-window requirement on 2026-09-09 because it was added in error.
Window activation and monitor placement are not completion gates;
headless browser verification is valid.
The quoted request records what was asked in that incident, not a continuing requirement to open every HTML artifact.
 Embed every visual candidate,
 option assessment,
 ranked recommendation and
answer control in the file.
 End with free text.
 Require no external stylesheet,
script,
 font or server.
 Render,
 inspect and exercise the form before delivering it.
 Every
device mock must use a raster captured at the cited physical panel resolution,
display at the cited dp dimensions at 100%,
 and expose physical px,
 logical dp,
current scale and a reset control.
 Never upscale a dp-sized bitmap.
 Keep the device
silhouette realistic and opaque:
 use a measured target-specific chassis,
 bezel,
hinge and corner treatment;
 place the screenshot inside its screen opening;
 never
clip it to reveal the questionnaire surface.
 Android mocks always include the target's
status bar and navigation bar;
 keep app controls out of cutouts and system insets.
Chat may report that the round is ready;
it never substitutes for the form.

**Handover timing.**
 The user then instructed:
 "Update the handover as you go."
Record every correction,
 answer,
 decision,
 candidate and verification result in
HANDOFF.md when it happens.
 Never wait until the session ends.

## 5j. An absent control was mistaken for a current-screen refinement

**What happened.**
 After being told to stay on the current accepted screen,
 the agent
chose a volume control that existed in an old decision but was absent from the accepted
screenshot.
 It built nine volume variants instead of critiquing the composition already
on screen.
 The user identified the missed work directly:
 the existing playback deck is
not centered.

**The rule.**
 For an existing-screen refinement,
 the accepted screenshot is the feature
boundary.
 Measure and critique its visible composition first.
 Candidates may move,
resize,
 align,
 or restyle only elements already visible there.
 A previously documented
but absent control is a new feature and requires explicit approval.

## 5k. Text scaling replaced a settled component

**What happened.**
 The segmented playback-mode control was settled,
 but the 200% text
adaptation silently replaced it with four plain radio rows.
 The user circled those rows
and corrected them twice:
 they must remain segmented buttons,
 and the large-text
control can be purely vertical.

**The rule.**
 Accessibility adaptation preserves settled component identity.
 Reflow the
same control when space changes.
 For this mode selector,
 default text uses one horizontal
segmented row;
 large text uses one vertical segmented stack with full labels.

## 5l. A sparse state icon consumed width from every track

The current-track treatment reserved a leading column on every list row and drew a
play icon only on the active one.
 The agent treated the icon as a compact non-color cue,
but the empty slots reduced title width throughout the list.
 The user rejected the
shape and said to kill it.

**The rule.**
 Sparse state decoration must not reserve permanent content space on
unaffected items.
 Current-track cues stay within the row's existing title,
 supporting
line,
 background,
 or overlaid boundary;
 every title keeps the full common width.

## 5m. Prepared output outlived its inputs, and embedded rasters were checked as files

The dark matrix form kept a prepared answer visible after the user changed a radio choice
or the correction text,
 so a stale answer could be copied as if current.
 Prepared output
now hides on any input change until regenerated.
 Separately,
 the validator proved each
source PNG existed but not that the bytes displayed in the form matched it;
 a swapped
data URL would have passed.
 Validation now compares every displayed data URL with its
named raster byte-for-byte and samples disclosed folder,
 rail,
 deck,
 and current-row
pixels against resolved role values.

**The rule.**
 A prepared answer must die with its inputs,
 and a generated artifact's
checks must cover the bytes the viewer sees,
 not only the files on disk.

## 5n. A component was improvised without reading the spec page that owns it

The picker trigger shipped as a 48dp outlined pill-ish box with a caret because the chips,
menus, and sheets pages had been read but the text-field specs had not.
 The user asked
plainly whether the archive had really been read.
 Against the text-field specs the field
was wrong in four ways: container height 48dp instead of 56dp, no floating label on the
outline, uniform 16dp padding instead of 12dp sides with icons plus 16dp between icon and
text, and an untinted trailing icon.

**The rule.**
 Before drawing any Material component, read that component's own specs and
guidelines pages in the local archive (`~/Downloads/m3.material.io`, mirrored in
`md3-tokens.md`), and list every deviation from the measured values in the form prose with
its reason.
 Adjacent component pages are not evidence for the component being drawn.

## 5o. A native raster was mistaken for Material 3 visual evidence

The desktop command-bar study used real Slint captures and a validated offline form,
but drew a generic outlined palette rather than MD3's fully rounded 56dp Search bar,
28dp docked Search view,
 official icons,
 BodyLarge typography and list treatment.
The user rejected the comparison at sight.
 Passing screenshot dimensions,
 color
samples and Axe on the questionnaire did not establish component lineage.

**The rule.**
 Before presenting a Material-based visual round,
 inspect its own local
archive images and tokens,
 apply the visible anatomy to the native prototype,
 and
compare the rendered component with that reference at the same logical scale.
Record each intentional platform adaptation and keep prototype renderability,
form accessibility,
 and actual visual fidelity as separate verification claims.

## 5p. Desktop mock widths displaced the actual target screens

The Search-page study used Slint-native screenshots at 360 × 640,
 480 × 600 and
1100 × 640px because the desktop app exposed a 480 × 600 preferred size.
 The user
corrected the premise:
 all visual decisions follow the Pixel 9 Pro Fold cover and
unfolded panels,
 with desktop inheriting even when that is awkward.
 The previous
rasters proved Slint drawing at invented window sizes,
 not the device result.

**The rule.**
 For this music player,
 compose the visual review from native Android
captures at the measured cover and inner panel physical resolution in both schemes.
Keep desktop porting and window-size choices subordinate to those visual decisions;
never relabel a desktop-native screenshot as target-device evidence.

## 5q. The first connector correction banned too much

The first unfolded Compose Search page used `fillMaxWidth()` for its header and
`ListItem`s.
 Their surfaces,
 divider and row hit regions crossed the centred
24dp connector.
 I treated the user's objection as a ban on **all app-owned
pixels and hit regions**,
 without distinguishing readable information from
structural paint.
 The resulting all-black/all-white strip rule forced a
left-header/right-results split that the user rejected.
 The user then
clarified that connector "content" means informational material such as text.

**The corrected rule.**
 Inspect where readable text,
 results,
 labels and
meaning-bearing marks actually appear.
 Such information must stay clear of
the connector;
 background,
 input/row surfaces,
 dividers and hit regions can
cross it.
 Do not derive a two-pane Search composition or a mandatory strip
color from E2.
 The original full-width mock is not thereby accepted;
 it
still needs a compositional review and native evidence under the corrected
rule.
 The old full-strip pixel guard and its successful red controls remain
historical tests of a superseded constraint,
 not product acceptance criteria.

## 5r. The withdrawn Fold study passed mechanical checks only

The corrected capture contains opaque 1080 × 2424px cover and 2076 × 2152px
inner screenshots of the player and Search page in both schemes,
 plus paired
hierarchies,
 four Android role records and per-frame state metadata.
 Direct
`wm density` and `dumpsys display` probes corrected the cover's unmeasured
411 × 923dp estimate:
 this AVD uses 390dpi on **both** panels,
 so its cover is
approximately 443 × 994dp.
 The new capture asserts density,
 font scale,
mode,
 panel and frame dimensions before recording metadata.
 Its connector guard
passed all unfolded scenes at 100% and player/results at 200%,
 checking
semantic/clickable bounds and **every RGBA pixel** in physical x `[1009,1068)`,
y `[136,2074)` against D41 black or D34 white.
 These checks enforced the
obsolete blanket-strip rule,
 not the clarified E2.
 Separate red controls in a
disposable copy rejected a crossing target,
 a dark tonal band and one
near-white pixel;
 they establish only that the old validator could detect those
changes,
 not that the design was appropriate.

The inner 200% player's last mode target is only 94px visible in the initial
frame (`[73,1980][936,2074]`),
 so that frame alone cannot prove 48dp
reachability.
 A native swipe in its independently scrollable deck moved it to
`[73,1904][936,2035]`,
 showing the full 131px target and border above the
navigation inset that starts at y=2074.
 The scrolled PNG/XML is kept as
supplementary evidence;
 the right track list scrolls independently.
 A static
screenshot does not establish Search navigation,
 focus,
 TalkBack or result
execution.

The now-withdrawn self-contained review,
 archived at `questions/archive/search-rejected-fold-review.html`,
paired both native panels with measured chassis frames,
 shows the viewer's system light/dark scheme,
 starts at
the player Search target,
 exercises Back and canned query/unavailable states,
and ends in free-text correction.
 Browser interaction checks exercised both
panel paths,
 preview fit/reset/zoom/close,
 top reset when switching scenes,
return focus,
 keyboard panning/Escape,
 and form output.
 An online fetch to a
local test endpoint succeeded;
 with browser offline mode enabled,
 the same
fetch failed while the self-contained file reloaded and every embedded image
decoded.
 Axe reported no violations in either chrome scheme or in the open
preview dialog after its scrollable region gained keyboard focus.
 The preview
frame retained measured pixel geometry at a 20px root font;
 the browser
reported no console or page errors.
 No KWin automation was used.

## 5s. Connector compliance was mistaken for an acceptable Search composition

The user rejected the unfolded Search page as obviously bad.
 This correction
applies to the **left-header/right-results composition**,
 not to D47's separate
page,
 D48's single header,
 or E2's restriction on information in the connector.
 The inspected native
`questions/render/fold-search-inner-open-results-light-s100.png` shows a tonal
Back/query/Clear header above a completely blank left body,
 while the results
heading and two rows sit alone in the right pane.
 The header visually promises
content under it,
 but governs empty space instead.
 The right pane carries all
result information at half the screen width;
 in the 200% capture its supporting
text wraps while the left body is unused.
 The empty state makes the split more
obvious:
 the query bounds `[156,166][970,283]` are separated from the
instruction bounds `[1351,1184][1793,1253]` horizontally and vertically
(`questions/evidence/fold-search-inner-open-empty-dark-s100.xml`).

I saw these screenshots and still called the page ready.
 I treated E2 as if it
required the header on one side and content on the other,
 then verified that
invented layout for connector pixels,
 semantics,
 density,
 accessibility of
the HTML wrapper and screenshot provenance.
 Those checks did not measure
whether the query and its content read as one page.
 The failure was not a
missing screenshot;
 it was failing to make a visual judgment from the screenshot
before asking the user to correct it.

**The package-specific gate.**
 After geometric and accessibility checks,
 inspect
every target-panel state as a composition:
 which region the header appears to
govern,
 whether instructions and results visibly belong to their query,
 whether
space is needlessly reserved while content is constrained,
 and what changes at
200% text.
 Reject a candidate that fails this review before labeling it
"active" or requesting user feedback.
 Keeping informative material off the connector is a boundary constraint,
not a mandate to split Search into two independent halves.
 Preserve
the rejected images as evidence,
 not as the current design.

## 5t. A physical dent was confused with a dp pane spacer

I first used M3's 24dp expanded-pane spacer as if it were the Pixel Fold's
physical crease,
 then banned every painted pixel and hit region from that
strip.
 The user clarified that only **informational material** needs to
clear the dent;
 decoration,
 padding,
 field/list surfaces and hit regions
can span it.
 The player's earlier fixed 24dp empty gap and fixed 414dp panes
are not protected decisions.
 The user's rule is
`max(min_padding, crease_width)` for **information clearance**,
 not a
mandatory visual gutter between halves.

The user initially estimated 10mm,
 then corrected the visible dent to
**about 7.5mm**.
 With the inner display's approximately 141.08mm active
width,
 that is roughly **110 physical px** centered at x 1038 on the
2076px panel,
 or x `[983,1093)`px.
 At the current AVD's 390dpi it converts
to about 45dp,
 but dp changes with Android display scaling while the dent
does not.
 Use physical millimeters or panel pixels for the invariant and
convert only at runtime for Compose layout.
 The emulator's zero-width hinge
sensor area measures occlusion,
 not the visible dent.
 Do not borrow the
unrelated 12dp mode-control text-padding floor as `min_padding` here.
After native player and Search comparisons,
the user set `min_padding` to **7.5mm total opposing-information separation**,
including future devices with a narrower crease.
Decorative surfaces and hit regions remain outside that floor.

I initially praised the user's YouTube thumbnail timestamp/title example as
safe near-crease placement.
 In the native screenshot those glyphs reach
within the approximate 7.5mm band;
 it is a **negative** precedent for
readable material on the dent.
 Inspect the center at native physical pixels
rather than assuming that a familiar app has satisfied this product rule.

## 5u. Search result columns were offset from the integrated header

The debug-only Search result row placed a 24dp type icon plus 12dp spacer
before its title.
The header instead placed Back inside a 48dp leading slot.
In the measured 200% inner capture,
query `cam` began at x `1249` while the result title `Cam` began at x `1220`;
on the cover they began at x `156` and x `127`.
The user required the Back arrow to align with the result music icon and
query text to align with the result title.
This is horizontal **paint alignment**,
not merely a match between hit-region rectangles.
Use the same 48dp leading-slot anchor for the header and result rows on both
panels,
center the 24dp type icon within it,
and check the native after-state raster and text bounds before presenting it.
The corrected keyboard-closed 100% and 200% debug captures put `cam`
and `Cam` at the same x `1249` inner and x `156` cover.
Pixel sampling at 200% also placed Back,
folder and note glyph centers at x `1190` inner and x `97` cover;
a pre-correction positive control detected the prior 29px offset.
The 48dp Back target remains,
but a decorative 48dp result slot is not a tappable row or an
accessibility pass.
See `evidence/search-header-result-alignment.md` for bounded proof.
Do not reduce the 48dp interaction minimum to achieve this alignment.

## Standing standards for this project

1. **EVERY question form ends with a free-text field.
    No exceptions.**
    Not "usually",
   not "when the options feel thin" — every form,
    every round.
    Options are always a
   partial guess at the problem space,
    and this project's best decisions arrived as
   free text rather than as one of the offered choices:
    tabletop-c,
    the true-black
   requirement,
    the argument against a bespoke design system,
    the Todoist-style undo
   toast,
    the first-run "Scan once / Always scan / Dismiss once / Dismiss forever"
   prompt,
    and the correction that chips were a styling objection and not a layout
   one.
    Two forms in session 3 shipped without one (see 5e) and both cost a round.
   The field is labelled openly — "anything else you want settled or changed" — and
   sits last.
    A `user-questions` item,
    letting the user raise what the questions
   missed,
    belongs on any follow-up round.
2. **Visual and behavioural questions are asked by building**,
    not by describing.
3. **Design calls belong to the designer.**
    Bring a recommendation with reasoning;
   ask only what the owner alone can answer — their library,
    hardware and habits.
4. **Known defects are stated up front**,
    in the same breath as the work.
5. **Project state lives in Markdown.**
    These documents exist so a session starting
   from nothing can pick the work up without losing decisions.
    Keep them current as
   decisions land.
6. **Every option is presented with my own assessment:
    pros,
    cons,
    and a pick —
   written as a LIST.**
    The user asks for choices,
    not a menu read out flatly —
   "you're supposed to comment on each option with what you think yourself of the pros
   and cons of that option" — and wants it structured:
    "I would like pros and cons in
   list form."
    So,
    per candidate:
   ```text
   PROS   · one point per line
          · one point per line
   CONS   · one point per line
   MY READ · pick / fallback / reject, and why in one line
   ```
   Never a prose paragraph,
    never a run-on sentence with semicolons.
    This applies in
   three places at once,
    because the user may be looking at any of them:
    the chat
   message,
    the form's question subtitle,
    and the caption printed inside the candidate
   file itself.
    A neutral list of equally-weighted options is an unfinished
   presentation;
    recommending one does not pre-empt the decision,
    and withholding the
   judgement wastes a round.
7. **True black (#000) is a standing rule,
    not a preference.**
    The dark scheme's
   background is pure black and stays pure black.
    It survives the bound MD3 design
   system,
    whose own dark surface role is neutral6 (#141218) with a five-step container
   ramp above it — where MD3 and true black disagree,
    **the surface roles get
   overridden,
    not the rule** (decisions.md B1).
    Never re-ask this,
    and never quietly
   let a token file reintroduce #141218.
8. **Rules are recorded absolutely,
    never softly** (see 5f).
    Imperative wording,
    the
   general form first,
    the scope stated,
    no hedge words.
    Every rule in this file and
   in decisions.md is to be read as binding,
    not advisory.
9. **Every design-question round is one self-contained HTML form.**
   Embed the built visual options,
    pros,
    cons,
    ranked recommendation,
    answer controls
   and final free-text field.
    Require no external stylesheet,
    script,
    font or server.
   Render,
    inspect and exercise it before delivering it.
    Every device mock must use a
   raster captured at cited physical panel resolution,
    display at cited dp dimensions
   at 100%,
    and show physical px,
    logical dp,
    current scale and a reset control.
    Never
   upscale a dp-sized bitmap.
    Keep the device silhouette opaque:
    rounded display
   corners require a measured target-specific chassis,
    bezel,
    hinge and corner
   treatment.
    Place the screenshot inside its screen opening;
    never clip it to reveal
   the questionnaire surface.
    Chat-only delivery is invalid.
10. **Update HANDOFF.md continuously.**
     Record each correction,
     answer,
     decision,
    candidate and verification result when it happens.
     Never defer updates until the
    end of a session.
11. **The accepted screenshot is the feature boundary for existing-screen
    refinement.**
     Measure and critique its visible composition before proposing
    candidates.
     Move,
     resize,
     align,
     or restyle only elements visible in that
    screenshot.
     A teaching example changes presentation method,
     not scope.
     A documented
    but absent control is a new feature and requires explicit approval.
12. **Accessibility adaptation preserves settled component identity.**
     Reflow the same
    control when space changes.
     This mode selector is one horizontal segmented row at
    default text and one vertical full-label segmented stack at large text.
13. **State cues preserve content space and use redundant channels.**
     The current track
    gets no play icon or reserved leading column.
     Its visible treatment must combine at
    least two of color,
     weight,
     boundary,
     or container while titles retain full row
    width.
14. **Prepared answers invalidate with their inputs.**
     Changing any radio group or the
    free-text correction hides prepared output until it is regenerated.
15. **Embedded evidence is validated as displayed bytes.**
     Questionnaire validators
    compare each data URL with its named source raster and sample disclosed role mappings
    pixel by pixel.
16. **Nothing is locked by hash.**
     No check refuses a page,
    template,
    builder,
    test or record because it differs from a recorded digest,
    and a recorded digest is never a reason to leave something uncorrected.
    D88 holds the human's words and its scope:
    this package's evidence and capture tooling,
    not the repository at large.
17. **What the human must see goes through the question tool.**
     On 2026-10-06 the human said the transcript does not show everything with reasonable effort:
    "For reliably making me see something,
    use the ask user question tool."
    A handoff,
    a proposal or an open choice written only as chat prose may never be read,
    so its silence means nothing.
