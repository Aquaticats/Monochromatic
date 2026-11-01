# Music player template editor: KWGT as precedent

## Purpose

D81 makes supporting text user-configurable through templates in Settings.
Section 11e of `package/music-player/design/open-questions.md` leaves fields,
grammar,
editor controls,
preview,
validation,
fallback behavior,
other row-type coverage and title customization undesigned.
The human named KWGT (Kustom Widget Maker) as precedent on 2026-10-05.
This note lists what KWGT's formula language and formula editor do,
and records which of those features the player keeps and omits.
The human answered that question on 2026-10-05;
D89 and D90 record the answers.
No production implementation is authorized.

## What is already settled

- D81:
  supporting text is templated and configured in Settings;
  title customization is not implied.
- D35:
  the default supporting line uses the neutral `onSurfaceVariant` role;
  its content is not fixed.
- D77:
  accessible-action naming and visible distinction before activation remain required.
- D87:
  the Settings page stays and is empty until the template configuration is designed.

## Sources

### Documentation read on 2026-10-05

- [Music info function `mi`](https://docs.kustom.rocks/docs/reference/functions/mi/)
- [Text converter function `tc`](https://docs.kustom.rocks/docs/reference/functions/tc/)
- [Condition function `if`](https://docs.kustom.rocks/docs/reference/functions/if/)
- [Date format function `df`](https://docs.kustom.rocks/docs/reference/functions/df/)
- [Math operators](https://docs.kustom.rocks/docs/more/math_operators/)

### First-hand run on 2026-10-05

KWGT 3.82 build 621115 (package `org.kustom.widget`),
downloaded from the publisher's own site,
file SHA-256 `481ee09b353028726ce08f59fec3c64d7e80b02e741b125b9c1dbfe3720a0071`.
It ran in a throwaway Android 17 Pixel 9 Pro Fold emulator,
created new for this look and deleted afterwards,
inside a container bounded to 6 GiB of memory and 2 CPUs.
The study emulator and its device image were not used.
A widget was placed on the home screen,
a `Text` item was added,
and its formula editor was opened and typed into.
Screenshots stay in private scratch storage:
the free version shows third-party advertisements,
so none is published.

## What KWGT's formula language does

### Text and formulas

A value is literal text with formulas enclosed in a pair of `$` signs.
Text outside the pair is shown as written.

```text
# typed into KWGT's formula editor
$mi(artist)$ - $tf(mi(len), mm:ss)$
```

With no music playing,
the editor's preview showed `Artist Name - 03:20` for that value.

### Functions

Functions have short two-letter names and take a mode word as their first argument.
`mi(title)`,
`mi(artist)`,
`mi(album)`,
`mi(len)` and `mi(pos)` read the playing track.
Calls nest:
`tf(mi(len), mm:ss)` formats the track length.
`tc` converts text:
lower case,
upper case,
capitalize,
cut to a length,
cut with an ellipsis,
pad,
split,
and regular-expression replace.
`df` formats dates.
Many other functions read weather,
battery,
network,
calendar,
notifications and web content;
those have no counterpart in a local music player.

### Operators and conditions

Inside a formula,
`+` adds numbers or joins text,
and `-`,
`*`,
`/` do arithmetic.
Comparisons are `=`,
`!=`,
`>`,
`>=`,
`<`,
`<=`,
and `~=` matches a regular expression.
`&` and `|` combine conditions.
`if(condition, then, [else])` chooses between values and accepts chained conditions.

### Named values and markup

The documentation lists a global-variable function (`gv`) for reusing a named value across formulas.
The editor's `BB code` example family is described as
`Adds simple text markup like color, bold, italic and so on`.
Neither was exercised in the first-hand run.

## What KWGT's editor does

Seen in the first-hand run.

### Layout

The editor is one screen.
A `Text Preview` line sits on top.
The `Formula Editor` field sits under it.
A row of buttons under the field shows a globe,
a palette,
a star,
and marks for bold,
size,
underline and italic.
None of them was pressed,
so what each does is not an observation here.
An `Examples` area fills the rest,
captioned `(click appends, long replaces)`.
The top bar holds buttons described as `Settings`,
`Star` and `Save` (a check mark).
Whether an edit takes effect before `Save` is pressed was not checked.

### Live preview

The preview changes with every keystroke.
With no music playing it still showed stand-in values:
`Artist Name`,
`Track Title`,
`03:20`.

### Help while typing

While the caret is inside a call,
the function's signature appears above the field with the current argument in bold,
for example `tf(date, [format])`,
and a sentence describing that argument appears under the field.
In one capture the opening `$` and the opening bracket of the outer call were tinted blue;
the rule behind the tint was not explored.

### Errors

A failed formula is reported in a red line under the field,
one line per failure,
naming the function and the problem.
Typing `$xx(1)$ and $mi(nope)$` produced `err: unknown` and
`mi: invalid music parameter: nope`.
The preview then showed only the surrounding literal text,
`and`.
A formula left without its closing `$` raised no error;
the preview showed it as literal text.

### Examples and favourites

Examples are grouped by function family,
such as `Music info`,
`Text converter` and `If conditions`.
Each entry shows a description,
a sample result and the formula,
for example `Current Track Duration (in mm:ss format)`,
`03:20`,
`$tf(mi(len), mm:ss)$`.
A tap appends the formula to the field;
appended formulas are joined with nothing between them.
The caption says a long press replaces the field's content;
that was not tried.
One family is `Fave Formulas`,
described as `Faves formulas, list or delete starred expressions`;
the `Star` button was not pressed.

## Where the player's need differs from KWGT

KWGT edits one free-standing text item.
The player needs,
and KWGT does not show:

- which row types have a template
  (track rows,
  folder rows,
  search results,
  the playing track);
- the player's own fields
  (such as duration,
  true peak,
  folder,
  file name and extension);
- a default per template and a way back to it;
- what a row shows when a field has no value;
- how a template interacts with the visible distinction D77 requires.

Those are design questions for later in this round,
not answered by KWGT.

## Answer: which KWGT features the player keeps and omits

The note first put each KWGT feature to the human with a recommendation.
On 2026-10-05 the human answered:
"I'll go with your recommendations with what to omit and keep."
D89 records it.
Where the recommendation was to keep or to omit,
that is now decided.
Where the note made no recommendation and said the choice was the human's,
the answer decides nothing;
those entries are listed under `Left to the agent and picked`.

### Kept

- Literal text with formulas between a pair of delimiters;
  text outside the pair is shown as written.
- Field functions that take a mode word,
  in the manner of `mi(title)`,
  with the player's own fields.
- Nested calls and formatting functions,
  in the manner of `tf(mi(len), mm:ss)`.
- Text conversion for case and for cutting to a length.
- `if` and comparisons,
  so a row can say what to show when a field has no value.
- A live preview that changes with every keystroke.
- Signature and argument help while the caret is inside a call.
- Error lines that name the function and the problem;
  their wording and marking follow the player's own error design.
- A field list that inserts at the caret.

### Omitted

- Arithmetic,
  unless a use appears.
- Global variables.
- Markup for colour,
  bold,
  italic and the like,
  because D35 gives the supporting line one neutral role.
- Functions for weather,
  battery,
  network,
  calendar,
  notifications and web content.
- Silence about a formula left open;
  the player reports it.
- Saved favourite formulas.
- The button row for markup,
  colour and the globe.

### Left to the agent and picked

The note recommended nothing here,
so the human's first answer did not settle these.
Asked whose they were,
the human answered on 2026-10-05:
you choose,
but for features extremely obscure in a music player,
such as regex,
lean on no.
D90 records the picks,
each open to veto:
text conversion stays at case and cutting to a length;
regular-expression match and combined conditions are omitted;
the preview shows real tracks from the open library,
with stand-in values only when none is open;
no list of ready-made examples;
an edit applies as typed while valid,
with a way back to the default and no separate save step.

The entries as first put:

- The rest of text conversion:
  ellipsis,
  padding,
  splitting and regular-expression replace.
- Regular-expression match (`~=`) and combining conditions with `&` and `|`.
- What the preview shows:
  stand-in sample values as in KWGT,
  or a real track from the open library.
- Whether ready-made examples are offered beside the field list.
- Whether an edit applies as typed with a way back to the default,
  or only on an explicit save.

## Not decided here

- The field inventory and field names.
- Which row types get templates,
  and whether titles are ever templated.
- Default templates.
- Fallback when a field is empty or a template is invalid.
- The editor's layout on the Fold's inner and cover panels.

The next step is a built Compose study of the editor on the Fold,
shown in a verified review form.
`Editor study: the agent's proposal` says what that study builds.

## Editor study: the agent's proposal

Nothing under this heading is decided.
It is what the agent builds so the human can look at it,
following the project's standard that visual and behavioural questions are
asked by building.

### What the app has today

Read from production source on 2026-10-05:

- `Track` holds a `uri` and a `displayPath`
  (`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/Track.kt`).
  Folder,
  file name and extension all come from that path.
- No tag is read.
  In the Android production source,
  artist and album occur only in comments and path examples
  (`Track.kt`,
  `core/Page.kt`,
  `core/RelPath.kt`,
  `BrainPlayer.kt`),
  so the language offers no artist or album field.
- Duration is known for the current track only (`PlaybackSnapshot.durationMs`).
  `MediaStoreSource.kt`,
  `SafTreeSource.kt` and `LibrarySource.kt` read no duration,
  so no row but the current one has it today.
- True peak is measured per file by the sweep and is absent until a file is analysed.
- The production track row (`trackRow` in `MainActivity.kt`) draws one text,
  `rowDisplay(rowLabel, item.name)`,
  and no supporting line.
  D35's duration and true-peak line exists in the design,
  not in the app.

The study therefore previews a line that production cannot fill yet for
every row.
That gap belongs to production,
not to the template language,
and the study states it.

### Spelling

KWGT's own spelling is adopted wherever KWGT has one,
because the human named KWGT as the precedent:
formulas between `$` signs,
`mi(...)` for a field of the track,
`tf(...)` for formatting a duration,
`tc(...)` for text conversion,
`if(condition, then, [else])`,
and `+` to join pieces of text inside a formula.
On 2026-10-06 D92 removed `if` and the comparisons;
the rest of this spelling stands.

Fields,
each a mode word of `mi`:

- `title`:
  the name the row shows as its title.
- `file`:
  the file name without its extension.
- `ext`:
  the extension.
- `folder`:
  the folder that holds the file.
- `path`:
  the path from the library root.
- `len`:
  the duration in seconds,
  as KWGT's documentation defines it;
  `tf(mi(len), m:ss)` formats it.
- `peak`:
  the true peak in dBTP with one decimal,
  and nothing when the file is not analysed yet.
  Since D93 (2026-10-06) the value carries its unit,
  as in `−1.2 dBTP`.

`mi(title)` and `mi(len)` mean what they mean in KWGT;
the other mode words are the player's own.
KWGT's documentation shows `tf(mi(len), mm:ss)`,
and the first-hand run gave `03:20` for it.
The player's rows show `4:35`,
without a leading zero,
so the default uses `m:ss`.
KWGT's documentation lists no table of format letters;
it uses a single `h` and `m` in one example.
Reading a single letter as unpadded is the agent's,
and the player's grammar defines it either way.

### Default and fallback

The default track-row template reproduces D35's line and drops the
true-peak part while a file is not analysed:

```text
# default template for a track row's supporting line
$tf(mi(len), m:ss)$$if(mi(peak) != "", " · " + mi(peak) + " dBTP")$
```

A field with no value yields nothing.
That is the agent's choice;
what KWGT does there was not observed.
While a template is invalid the rows keep the last valid template (D90).

This default needs `if`,
which D92 removed on 2026-10-06.
D93 replaces it with plain substitution:
text outside formulas is always shown,
the peak carries its unit,
and the default is `$tf(mi(len), m:ss)$ $mi(peak)$`,
which reads `4:35 −1.2 dBTP` and,
before analysis,
`5:12` followed by a space nobody sees.

### What the study shows

One template,
the track row's supporting line,
because that is the line D35 and D81 name.
Other row types and titles stay out until the human asks for them.

- The Settings page with the template as its one entry,
  showing the line it currently produces.
- The editor opened from that entry:
  preview rows on top,
  drawn as real track rows from the open library,
  one of them a file that is not analysed yet;
  the template field under them;
  the field list under the field;
  a way back to the default.
- The editor while typing inside a call,
  with the signature and argument help and the keyboard open.
- The editor with an unknown field and with a formula left open,
  showing the error lines while the preview keeps the last valid result.
- The editor with no library open,
  where the preview uses stand-in values.

Each is captured on both Fold panels,
in light and dark,
at 100% and 200% text.

### Baseline layout being built

`package/music-player/design/template-editor-scenes.mjs` holds the states:
`list`,
`default`,
`help`,
`unknown-field`,
`open-formula`,
`custom` and `no-library`.
`help`,
`unknown-field` and `open-formula` hold focus in the field with the keyboard open.

The Settings page lists the template under a `Templates` heading:
`Track row supporting line`,
with the line it currently produces as its supporting text.
Opening it shows the editor on the same page column,
titled `Supporting line`,
with a way back to Settings.
On the unfolded panel the page takes the right half and the folder browser
and deck stay on the left,
as the withdrawn Settings study placed its page (adopted from D50 and D51).

The editor,
top to bottom:

- `Preview`:
  two track rows,
  one analysed and one not,
  then a note saying where they come from.
- `Template`:
  an outlined field in a monospace face.
  Under it,
  error lines with an error icon when the template is invalid,
  otherwise the signature and argument description when the caret is inside a call.
- `Fields`:
  one row per field with its name,
  its value for the first preview row and what it inserts.
- `Reset to default`,
  offered only when the template differs from the default.

Typing is not connected in the study;
each state is authored.
What this baseline cannot show before it is captured is how much of it stays
visible above the keyboard,
most of all the preview while typing at 200% text.
That is the first thing to look at,
and a concern to answer with a built variant,
not a question in words.

The study is debug-only and authored:
each state's template text,
preview,
help and errors are fixture values,
not the output of a production parser.
A small reference evaluator beside the builder computes what each authored
template yields for the fixture tracks,
and the builder refuses a capture whose drawn text differs from it,
so no screenshot can show a result the grammar does not produce.
No production implementation is authorized.

### First look at the baseline

The first build of the study
(prototype commit `524b61c667e839fdcb3bba3871f0d392c5b0ded9`)
was captured on the Fold emulator on 2026-10-05.
The emulator ended in a segmentation fault during these visits too,
so the views were collected over several boots.
This reading used the views in hand at the time:
the whole inner panel and the cover panel at 100% text,
plus the first cover views at 200%.
These views are a first look only and are not published;
the build described under `Layouts being compared` replaces them.

What the hierarchy and the images show:

- At 100% text,
  on both panels,
  everything a typing state draws around the field is in view with the keyboard open:
  both preview rows,
  the note,
  the field and the lines under it.
  The page's scrolling window runs from pixel 314 to the keyboard at 1352 on the inner panel
  and from 330 to 1605 on the cover panel.
- At 200% text on the inner panel,
  typing inside a call scrolls the whole preview out of view.
  The field with its help is slightly taller than the window,
  so the field's label slides under the header and the help sentence ends flush against the keyboard.
- At 200% text on the inner panel,
  the two error states keep the preview rows,
  with the first row's title against the header or partly under it,
  and the error line ends flush against the keyboard with no gap.
- With the keyboard closed at 200% text,
  the inner panel shows the preview,
  the field and the first one or two fields of the list;
  the rest of the list needs scrolling.
- With the keyboard closed at 100% text the cover panel shows the whole page for the default template;
  with a changed template `Reset to default` falls just outside.
  The inner panel shows all but the last two fields.

Where the page rests with the keyboard open was not designed in that build.
It was whatever Compose does by default when a focused field's window shrinks.

Two problems of the capture itself,
not of the design:

- The first time the keyboard opens after the text size changes,
  Gboard draws its own notice above the keys,
  `Keyboard font size updated`,
  which makes the keyboard taller for that one view.
  The capture now opens the keyboard once per condition before capturing,
  and refuses a keyboard-open view whose keyboard edge differs from the panel's other views.
- The hierarchy dump writes an attribute in single quotes when its value holds a double quote.
  The first reader took only double-quoted attributes and so lost the text of the default template.
  The dump also prints a `visibleFrame` between the keyboard's frame and its visible flag,
  which the first pattern for the keyboard's edge did not allow for.
  Both are now read from the saved dumps.

### Layouts being compared

Superseded on 2026-10-06:
D91 took the scrolling preview and rejected both pinned layouts,
and D94 removed the authored scroll rule this section adds to every layout.
The section stays as the record of what was built and why.

The first look names one consequential concern the human did not raise:
at large text,
typing can hide the very preview that D89 keeps for feedback on every keystroke.
The project's standard is to answer such a concern with built variants.
Nothing here is decided.

One rule is added to every layout,
because without it the comparison would be about a default nobody chose:
while the field has focus,
the page scrolls just far enough that the lines under the field clear the keyboard by 8dp,
and never so far that the field's own upper edge leaves the window.

The layouts differ in where the preview is:

- `flow`:
  the baseline.
  Two preview rows at the top of the scrolling page,
  so they scroll away when the field and its lines need the room.
- `rows`:
  the two preview rows stay fixed under the header and the rest scrolls beneath them.
  KWGT also keeps its preview fixed over the formula field.
- `lines`:
  only the two lines the template yields stay fixed under the header,
  without the rows' titles,
  as KWGT's single `Text Preview` line does.
  It costs less height than `rows` and no longer looks like a track row.

Each pinned layout is built for the three typing states and for one state at rest.
The baseline also gains one view scrolled to the page's end,
because the first build never showed the end of the field list or `Reset to default`.

Considered and not built:

- Error and help lines docked directly over the keyboard,
  with or without a pinned preview.
  At 200% text on the inner panel the help for one argument is about 550 pixels tall
  (a signature line and six lines of description),
  the pinned rows about 400,
  and the window about 1040,
  which would leave the field less than one of its own lines.
  Docking only the error lines and the signature would fit,
  but then differs from `rows` only for a template taller than the window.
- A pinned preview that appears only while typing.
  The page would jump when the field takes focus;
  a layout with one rule is easier to predict.

A second concern is noted and not built yet:
the field list sits under the field,
so with the keyboard open most of it is out of view,
and inserting a field means scrolling away from the field.
What to do about it depends on which layout is taken,
because a pinned preview still shows the result while the field is scrolled away.

The first build's error for a formula left open read
`formula: the formula opened at character 1 is not closed`.
It now reads `formula: the $ at character 1 has no closing $`.

### What the second build measured

The second build
(prototype commit `82d2692b7471fcde27c86c9bb321a4d35557bbca`)
was captured in full on 2026-10-05 and published on 2026-10-06
(its images were removed from `questions/evidence/` later that day,
when D91 rejected the pinned layouts;
the measurements in this section stay as the record):
128 views,
every scene on both panels,
in light and dark,
at 100% and 200% text.
`package/music-player/design/evidence/template-editor-boundaries.md` says how a view's contents are read.
For the preview,
the field and the lines under it:

- At 100% text every layout keeps all of them in view in every typing state on both panels.
- `flow` at 200% text loses the preview while typing inside a call on both panels.
  On the cover panel it keeps the field and the whole help;
  on the inner panel the field and the help reach the window's edges.
- `rows` at 200% text keeps the preview everywhere.
  Typing inside a call leaves only the help's first line on the inner panel,
  cut at the keyboard,
  and cuts the help's sentence on the cover panel.
- `lines` at 200% text keeps the preview everywhere.
  Typing inside a call keeps the help's first line whole on the inner panel and the whole help on the cover panel.
- The error states fit in every layout at 200% text,
  except that in `flow` the first preview title sits against the header on the inner panel.

The agent's ranking,
as the review page states it:
`lines`,
then `rows`,
then `flow`.
`lines` over `rows` because both keep the result in view and `lines` leaves more room for the help at 200% text,
while what `rows` adds is the two titles,
which a template never changes.
`rows` over `flow` because `flow` loses the result while typing inside a call at 200% text,
and the rows lose help instead,
which can be scrolled to while the result stays in view.
The choice is the human's and is asked on the review page,
`package/music-player/design/questions/template-editor.html`.

### Decided on 2026-10-06

Asked through the question tool with the review page open,
the human rejected the agent's ranking and chose none of the offered options outright,
writing that least surprise against the base platform matters more,
and that the preview scrolls (D91).
Asked in the same way about the study's assumptions and the questions that followed:
the language has no conditional (D92),
an empty field is plain substitution with the unit inside the peak field (D93),
and the page rests where Android puts a focused field (D94).
Everything else the study assumed stands:
one template for the track row's supporting line,
KWGT's spelling without `if`,
the seven fields and the D90 picks.

What the agent learnt about its own ranking:
it weighed what stays in view at 200% text above how the page behaves against other apps,
and the human weighs it the other way.

The study is rebuilt for these decisions:
one layout,
no scroll of the study's own while typing,
the new default template,
and a changed-template scene whose second row ends with the separator an empty peak leaves.
Its review page shows the decided design and asks nothing.

Shown that page,
the human answered that a library is always open (D95),
that scrolling suffices to reach the field list (D96),
and that the agent builds one version of which other rows get templates for approval (D97).
That version is the next section.
The human then accepted the editor as shown (D98),
approved the version as built (D99),
and decided what D95 means for the first-run study (D100).

## Which rows get templates: the agent's version (2026-10-06)

D97 asks for one built version,
the agent's own design,
and the human's approval.
The human approved it as built (D99).

### Where supporting text appears

From the design decisions
(production source read on 2026-10-06 draws no deck subtitle;
the deck's `1 of 16 · −1.2 dBTP` is the prototype's authored deck):

- Track rows in the folder's track list:
  a title and a supporting line
  (C2,
  D35).
  This is the template the editor already studies.
- The playing track in the deck:
  a title and a subtitle that counts the track's place in its folder,
  as in `6 of 10` (D5),
  and adds the true peak once it is measured (D27).
- Search results:
  a title and the parent folder as supporting text,
  which D77 requires so that equal names stay apart.
- Folder cells of the picker:
  one name each,
  with no supporting line (D3).

### The version

- Two templates,
  listed under `Templates` in Settings:
  `Track rows` and `Playing track`.
- `Track rows` is the template studied so far,
  default `$tf(mi(len), m:ss)$ $mi(peak)$`.
- `Playing track` is the deck's subtitle,
  default `$mi(track)$ of $mi(total)$ $mi(peak)$`,
  which reads `1 of 16 −1.2 dBTP`,
  and `1 of 16` before analysis.
  Its fields are the seven track fields and two more:
  `track`,
  the track's place in its folder,
  and `total`,
  how many tracks the folder holds.
  `mi(track)` follows KWGT,
  whose documentation gives it as the current track in the playlist;
  `total` is the player's own word.
  `pos` is not used for the place,
  because KWGT gives `mi(pos)` as the playback position in seconds.
- Its preview draws the same two files as the track rows' preview,
  as rows of a title and the line.
  A deck is one playing track,
  so two decks would be a shape the app never shows,
  and one deck alone would hide the line before analysis.
  Rows keep both editors alike;
  the deck itself shows the line in the left pane on the inner panel.
- Search results are not templated:
  their supporting line is the parent folder D77 requires,
  and a template could remove it.
- Folder cells and every title stay as they are,
  as D81 implies no title customization.

Why two and not one:
a single template for both would either lose the deck's place in the folder
or put it on every track row.
foobar2000,
the music player most known for templates,
also keeps the now-playing surfaces apart from the playlist:
its Default User Interface preferences set
"how to display information about the currently played track in various places:
main window title,
status bar,
tooltip of the notification area icon"
([Hydrogenaudio knowledgebase](https://wiki.hydrogenaudio.org/index.php?title=Foobar2000:Preferences:Default_User_Interface)).

What this costs:
a second entry in Settings and two more fields to learn.
The deck's default drops the middle dot,
as the track rows' default does under D93.

### Built

The editor study gains the `Playing track` editor,
the Settings page lists both templates,
each editor is titled with its template's name
(so the track rows' editor,
titled `Supporting line` until now,
becomes `Track rows`),
the left pane's deck on the inner panel shows the new default,
and the state previously called `no-library` becomes `empty-library` under D95.
The field list is unchanged (D96).
