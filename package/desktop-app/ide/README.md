# Read-only Slint IDE

Implementation in progress.
The accepted [0.x scope][scope] and [build approach][plan] define the completion criteria.
This package is not yet a completed application.
Use the [handover][handover] for current verification,
running-process details,
and the next implementation action.

## Implementation queue

- [ ] Complete the source view:
  shaped selection/copy,
  initial real syntax,
  font fidelity,
  and native external-change correspondence pass;
  the reading keys are listed under [Source view keys](#source-view-keys);
  hints and diagnostics render from injected snapshots,
  see [Inlay hints and diagnostics](#inlay-hints-and-diagnostics),
  and await the language-server wiring.
- [x] Native tree,
  asynchronous file switching,
  and recent-file reveal.
  The sidebar is resizable;
  see [Sidebar width](#sidebar-width).
  External changes reach the tree and the source through inotify notifications;
  see [Change watching](#change-watching).
- [x] Combined path/content search.
- [x] In-file find.
  Plain literal,
       case-insensitive matching with a transient find bar;
  see [In-file find](#in-file-find).
- [ ] Required language-intelligence feature paths.
- [ ] Measured Helix-supported language inventory and private server state.
- [ ] Native interaction tests and behavior-difference documentation.

## Project startup

Run `monochromatic-ide PROJECT` for one explicit local root,
or `monochromatic-ide PROJECT --file FILE` to initially display a file inside it.
Relative `FILE` paths start at `PROJECT`,
not the shell's working directory.
Without `PROJECT` the application opens the home folder that `HOME` names
(decided on 2026-10-06 for a launcher entry started without a folder);
without a usable `HOME` it reports a usage error with status 2.
[Home folder as the project](#home-folder-as-the-project) describes what changes then.
`--help` and `--version` exit before filesystem or native-display startup.
`--licenses` prints every license and notice text the executable carries,
each under a heading,
and exits with status 0 without a project,
a home folder,
or a display
(see [What the executable carries](#what-the-executable-carries));
beside `PROJECT` or `--file` it is a usage error.
The executable no longer substitutes example source when no file is selected.

Native tree and file switching pass callback tests and real nested Wayland input checks.
Long-distance reveal uses fixed-row windowing over native `ScrollView`;
see the [ListView investigation][listview].
Directory reads and later source opens run in bounded background workers.
Failed opens retain the displayed source and do not enter recent-file history.
Ctrl+0 through Ctrl+9 use session-local promotion and ancestor reveal.

[listview]: ../../../doc/troubleshooting/slint-listview-random-seek-offset.md
[find-matching]: ../../../doc/planning/slint-ide-find-matching.md

## Sidebar width

Drag the divider between the tree and the source to resize the tree.
The width starts at 256 px,
stays between 160 px and the window width minus the divider's 1 px line and a 288 px source column (the 240 px it had,
plus the 48 px the minimap takes at that size),
and lasts for the session only.
A window too narrow for the chosen width shows the tree narrower,
down to 160 px,
and restores the chosen width when the window widens again.
Double-clicking the divider does nothing.

The divider is reachable with Tab between the tree and the source.
Left and Right change the width by 16 px,
Home and End go to the narrowest and widest width,
and Ctrl+0 through Ctrl+9 still switch files.
Accessibility tools see a horizontal `slider` named `Sidebar width`
with its value,
bounds,
step,
and increment,
decrement,
and set-value actions;
Slint 1.18.1 has no splitter or separator role.
A pointer press never takes keyboard focus.

The divider is a 1 px line with no strip beside it.
Each state is marked by more than color:

- at rest,
  one faint column;
- under the pointer,
  the column-resize cursor and three columns in stronger ink;
- during a drag,
  the cursor and three columns in full ink;
- with keyboard focus,
  three columns in the accent color
  and a 5 px by 96 px handle in the same color in the middle of the line,
  never taller than the divider
  (the user chose the 96 px handle on 2026-10-06).

### Divider hit area

The divider's layout cell is its 1 px line,
so the tree and the source column meet at it.
Its pointer zone is 5 px wide over the whole window height:
the line's column and two columns on each side.
At sidebar width `W` those are the tree's last columns `W-2` and `W-1`,
the line at `W`,
and the source column's first columns `W+1` and `W+2`.
A press there starts a drag and does nothing else.
The zone takes presses only:
a wheel turn over it scrolls the tree or the source under it.

The zone is narrower than the 48 px minimum that every other interactive element of this application keeps.
The user decided this on 2026-10-05 for this one element:
the application runs on desktops only,
every desktop has a pointer and a keyboard,
dragging the divider is rare,
and the width is also adjustable by keyboard.
The exception does not extend to any other element.

What the zone covers,
measured in `ui/tree.slint`,
`ui/app.slint`,
and the toolkit's fluent scroll bar (Slint 1.18.1 `widgets/fluent/scrollview.slint`):

- Tree columns `W-2` and `W-1`.
  Rows span the whole tree width,
  and their text ends 12 px before the edge,
  so a row loses two columns of padding as a click target.
  When the tree overflows,
  its 14 px scroll bar lies over columns `W-14` to `W-1`.
  The thumb is drawn in columns `W-6` and `W-5`,
  or `W-10` to `W-5` under the pointer,
  and the arrow buttons take columns `W-11` to `W-4`,
  so the zone covers none of them.
  The bar scrolls by a drag that starts anywhere on its width;
  such a drag can no longer start on its last two columns.
- Source columns `W+1` and `W+2`.
  They are the first two of the 56 px line-number gutter.
  Line numbers are right-aligned and end 12 px before the text,
  and a gutter click puts the caret at the start of its line from any gutter column.
  Source text and selection rectangles start at `W+57`.
  Above and under the source view the zone lies in the 12 px padding of the file label and of the find bar.

### Pointer zone precedent

The zone's width follows desktop toolkits,
read from their sources on 2026-10-05:

- Qt gives a splitter handle narrower than 4 px a grab area of 4 or 5 px:
  `QSplitterHandle::resizeEvent` adds `(5 - handleWidth) / 2` px of margin on each side,
  5 px for a 1 px handle
  (`qt/qtbase` at `f127f11f`,
  `src/widgets/widgets/qsplitter.cpp` lines 208 to 221).
  KDE's Breeze style sets the handle width to 1 px
  (`KDE/breeze` at `fab6402a`,
  `kstyle/breezemetrics.h` line 169),
  so Breeze applications show a 1 px line with that 5 px grab area.
  Once the pointer is on a handle,
  Breeze also places a 24 px square proxy under it that keeps the drag reachable
  (`kstyle/breezesplitterproxy.cpp` line 312,
  `SplitterProxyWidth` 12 in `kstyle/breeze.kcfg`).
  Qt's Fusion style uses a 4 px handle
  (`src/widgets/styles/qfusionstyle.cpp` lines 2630 to 2632).
- Visual Studio Code's sash is 4 px wide and centered on the boundary
  (`microsoft/vscode` at `729f257f`,
  `workbench.sash.size` default 4 in `src/vs/workbench/contrib/sash/browser/sash.contribution.ts` lines 22 to 26,
  `src/vs/base/browser/ui/sash/sash.ts` lines 147 and 667).
- GTK 4 extends a paned separator's pointer area by 6 px on every side unless `wide-handle` is set
  (`GNOME/gtk` at `c2a232c4`,
  `gtk/gtkpaned.c` lines 131 and 297 to 310),
  and libadwaita draws the separator 1 px wide
  (`GNOME/libadwaita` at `19098711`,
  `src/stylesheet/widgets/_paned.scss` lines 2 to 4),
  which makes 13 px.

Qt's 5 px for a 1 px handle is used.
Visual Studio Code's 4 px cannot be centered on a 1 px line.
GTK's 6 px on the tree side would cover the scroll bar's thumb in columns `W-6` and `W-5`.
Three columns on each side would still clear the thumb and the arrow buttons;
four would cover the buttons' last column.

### Differences from editord

editord's `<file-tree>` starts at `16rem`,
which is the same 256 px default,
and is resized through the browser's `resize: inline` grip
(`package-paused/desktop-daemon/editord/src/client/file-tree/file-tree.styles.ts`).
Its styles declare no minimum or maximum width,
and its client has no double-click handling for the tree width.
This package keeps the default and the session-only width,
replaces the browser grip with a full-height divider,
and adds the 160 px minimum,
the 288 px source minimum (240 px of text beside the minimap),
and keyboard adjustment.

### Sidebar checks

`test:native` drives the divider with real pointer and key events:
a drag from each of the zone's five columns and from the first column on each side of it,
clicks on the zone and on the pixels beside it at the default,
narrowest,
and widest widths,
a tree scroll-bar drag from the last column left of the zone,
a wheel turn over the zone's tree columns and source columns,
and the rendered columns of every state.
`inspect:sidebar-guards` removes each width bound,
each edge of the zone,
and each keyboard-focus mark in a disposable copy
and checks that its named test fails.
The divider's accessible slider is tested under [Accessibility checks](#accessibility-checks).

## Entries that git ignores

The tree lists every entry of a folder,
hidden names and ignored ones included,
as `Workspace::list` reads them.
Before this change an ignored entry looked like any other
(measured on 2026-10-14 with the release bundle on a disposable repository:
`app.log`,
`build`,
`old.bak`,
`private.txt`,
and a nested `scratch.tmp` were all drawn in the full ink).
The name of an entry that git would ignore is now drawn at 0.8 opacity of its ink,
in both color schemes.
The user asked for the text only:
the disclosure arrow and the slot badge keep the full ink,
and the row's layout,
weight,
and size do not change.
IntelliJ colors the name text of such an entry and leaves its icon alone
(`NodeRenderer.customizeLegacyRenderer` in `platform/platform-api/src/com/intellij/ide/util/treeView/NodeRenderer.java`
takes the descriptor's color for the text attributes and the presentation's icon unchanged),
which is what the arrow and the badge do here.

A second signal does not depend on color or opacity:
the row's accessible label is the name followed by `, ignored by git`,
so a screen reader says it,
and the accessibility tree the nested compositor serves (`a11y`) lists `app.log, ignored by git`.
The row's description is unchanged (`Directory`,
or `Source file` with the shortcut of its slot badge).

### How an entry is judged

The `ignore` crate (ripgrep's,
0.4.33,
used for its `gitignore` module only) compiles each rule file;
`src/git_ignore.rs` applies them in git's order.
No git process runs and git's index is not read.
For a listed directory the reader thread (`DirectoryWorker`) decides,
beside the read,
which entries are ignored and sends the names with the listing,
so the tree installs the answer with the listing it describes
and a superseded reply is dropped with its answer by the same single-use request.
The window thread only compares and draws rows.

- The work tree is the nearest folder at or above the listed one that holds `.git`,
  a folder or a file.
  Outside any work tree nothing is dimmed,
  and the reason is logged once at info level.
  A `.git` file (linked work tree or submodule) names its git directory with a `gitdir:` line,
  and a `commondir` file there names the directory that holds `info/exclude`.
- `.gitignore` files apply from the top of the work tree down to the listed folder,
  the deepest first,
  then `info/exclude` of the common git directory,
  then the global excludes file (`core.excludesFile` of the user's git configuration,
  else `$XDG_CONFIG_HOME/git/ignore`);
  the first file that matches decides,
  a negation included,
  and inside one file the last matching line decides.
  A nested repository starts its own rules.
- An ignored directory ignores everything inside it,
  whatever a deeper rule says:
  `build/` with `!build/keep.txt` leaves `build/keep.txt` ignored.
  A directory pattern (`build/`) does not match a file or a symbolic link of that name.
  The directory row dims too.
- `.ignore` and `.rgignore` are not read:
  they are ripgrep's rules,
  and the tree follows git.

Compiled rule files are kept until their stamp (inode,
length,
modification time) changes,
and each listing stats the rule files that apply to it:
one `.gitignore` per level from the top of the work tree down,
`info/exclude`,
and the global file.
Matching costs about 3 microseconds per entry for four levels of twelve patterns
(debug build,
`test:ignored`:
60 ms for a directory at the 20,000 entry limit).
A directory with more entries than that shows nothing dimmed and the reason is logged,
so one huge folder cannot hold the single reader.

### Refresh

A folder's listing is read again when something in it changes name;
a `.gitignore` edited in place changes no name,
so the change watcher also reports a watched folder when a `.gitignore` in it,
or `info/exclude` inside a shown `.git` folder,
is written (`src/change_watch/record.rs`).
A listing whose own `.gitignore` stamp,
or whose work tree's `info/exclude` or global file stamp,
differs from the previous listing's marks the tree,
and the tick then reads every shown folder again,
because a changed rule can change the rows of any folder below it and nothing notifies those folders.
A `.gitignore` above the project folder,
`info/exclude` of a work tree whose `.git` folder is not shown,
and the global file are not watched:
the one-second safety sweep of [Change watching](#recovery-and-timers) finds them.
Measured on 2026-10-14 with the release bundle in the nested compositor,
from the write of a rule file to the label in the accessibility tree
(each polled every 20 ms,
so about 45 ms of every figure is the tree read itself;
six edits each way per source):
a `.gitignore` of a shown folder took 49 to 120 ms,
`.git/info/exclude` 48 to 973 ms,
and the global excludes file 53 to 945 ms,
the last two being the safety sweep's phase.
A `.gitignore` edit that changed a row of an open folder inside it took 20 ms more to show in the same pass.
Twelve expanded folders in a work tree cost the idle IDE 0.05 to 0.06 s of CPU in 30 s,
against 0.06 to 0.08 s for the build before this change (three runs each;
the clock ticks at 0.01 s).

### Limits

- Git's index is not read,
  so a tracked file that matches a pattern is dimmed,
  where `git check-ignore` reports it as not ignored.
  This is deliberate:
  telling tracked files needs the index or a git process,
  and the user chose the `ignore` crate.
  `a_file_that_matches_a_pattern_is_ignored_whether_or_not_git_tracks_it` pins the behavior.
- Only the rule sources named above are read.
  Case-insensitive matching (`core.ignorecase`),
  `core.excludesFile` set only in a repository's own configuration,
  and `include` and `includeIf` sections of the configuration are not followed;
  with `GIT_CONFIG_GLOBAL` set,
  the crate still falls back to the home configuration
  when the named file sets no excludes file,
  where git would not.
- A symbolic link to a directory is a file for directory patterns,
  as in git.
- Rules are judged from the files as they are on disk:
  an unreadable `.gitignore` counts as empty,
  and invalid lines are skipped with a warning in the log.

### Checks

`test:ignored` runs the matcher tests (patterns,
negation,
anchoring,
nested files,
ignored directories,
`info/exclude`,
the global file,
linked work trees,
the top of the work tree,
nested repositories,
the entry limit,
and the tracked file),
the tree tests (marks per directory,
the stale-answer drop,
the reread flag),
the reader tests,
and the rule-file notification test,
and prints the matching time.
`test:native` renders the rows in both schemes and measures the name's contrast,
checks the model flag and the accessible label through element handles,
and rewrites a `.gitignore` under a running window.
`inspect:ignored-guards` removes each guard in a disposable copy and requires the named test to fail.

Contrast of the name's strongest glyph pixel against its row fill,
measured on rendered frames in `ignored_names_are_drawn_at_eighty_percent_with_measured_contrast`
(plain name,
then the same name ignored):

- Dark scheme:
  17.04:1 to 11.16:1;
  with the pointer over the ignored row 17.04:1 to 9.38:1 (the fill becomes `#2D2D2D`);
  with keyboard focus on it the same 9.38:1.
- Light scheme:
  17.01:1 to 9.19:1;
  with the pointer over the ignored row 17.01:1 to 8.46:1;
  with keyboard focus the same 8.46:1.
- Selected row,
  both schemes:
  white on `#0078D4` 4.53:1 to 3.45:1;
  under the pointer or keyboard focus (fill `#006EC3`) 4.53:1 to 3.92:1.
  The ignored selected name stays above the 3:1 floor of the selected-ink rule
  and falls below the 4.5:1 of WCAG 2 for normal text,
  which a plain selected row reaches at 4.53:1.
  The user asked for 0.8 opacity,
  so it is kept on selected rows too;
  the row's label still says `ignored by git`.

## Combined search

Double-Shift opens transient search.
Filename matches precede content matches;
prefix the input with `%` to show only content results.
Up/Down wrap the selected result,
Enter or a click opens it,
and Escape or an outside click closes the overlay.
Content results navigate to their matching source line.
The scope is the last-focused tree directory or a focused file's parent,
falling back to the project root.

Input changes cancel old work immediately and debounce the next search by 150 ms.
The worker runs filename and content searches concurrently through ripgrep.
It retains at most 20 filename matches and 30 content matches,
with at most one matching line per file.
Filename matching is smart-case literal substring;
content matching is smart-case regular expression.
A content-search failure does not discard usable filename results.

Record and retained-diagnostic bounds limit memory,
not total bytes scanned or query duration.
The application kills and reaps cancelled search children;
inherited ripgrep configuration,
preprocessors,
and archive decompression are disabled.
Canonical scope validation is not an OS-enforced filesystem sandbox.
Native dark/light input and clipboard probes verify result opening and content-line navigation.
Headless tests cover late replies,
 close cancellation,
 pending-open focus,
 and result-model pointer lifetime.

## In-file find

Ctrl+F opens a find bar under the source view and focuses its input with the previous find text selected.
The bar exists only while finding;
it has no previous,
 next,
 or close buttons,
because Enter,
 Shift+Enter,
 and Escape cover those actions for a bar that only the keyboard can open.
Matching is incremental while typing.
Enter selects the next match and Shift+Enter the previous one,
wrapping at the ends of the file.
Escape closes the bar,
removes the highlights,
and returns focus to the source view.

The bar shows the active match and the total as `3/17`,
`0/17` when the selection is not on a match,
`No matches` in a heavier weight when nothing matches,
and a trailing `+` when the match list was cut off at its bound.
Accessibility tools get the same state as a sentence,
for example `Match 3 of 17`,
both as the find input's description and as a text element.
The bar uses the `search` role with the label `Find in file`.

### Find matching

Matching is plain:
the find text is compared literally,
ignoring case,
against the whole file.
Regular-expression punctuation has no special meaning,
and empty find text matches nothing.
Case folding is the Unicode simple case folding of the `regex` crate that Helix reexports;
no dependency was added.
Matching lives in one function,
`find_matches` in `src/find.rs`,
so its semantics can be replaced in one place.
Matches are ranges of source character positions,
the same unit as selection and copying.

### Deliberate differences from editord

editord delegates Ctrl+F to Chrome's find-in-page,
which matches through ICU collation search;
the measured semantics are in [the find matching plan][find-matching].
The user decided on 2026-10-05 not to reproduce that folding.
`tests/find_reference.rs` runs the production matcher against the captured Chrome 149.0.7827.54 corpus
in `tests/fixture/browser-find.json` and pins the exact set of differing cases,
so an accidental change of matching behavior fails the suite:

- `canonical-accent` and `plain-accent`:
  composed,
   decomposed,
   and unaccented letters do not match each other.
- `case-expansion`:
  `STRASSE` does not match `Straße`.
- `compatibility-ligature`:
  `office` does not match `oﬃce`.
- `dotted-i`:
  `i` does not match `İ`.
- `nbsp-as-space`:
  an ordinary space does not match a no-break space.
- `kana-script`,
   `kana-width`,
   and `kana-composed`:
  kana script,
   width,
   and voicing marks are not folded.
- `single-quote` and `double-quote`:
  straight quotes do not match curly quotes.
- `soft-hyphen`:
  a soft hyphen inside a word is not ignored.
- `combining-mark-only`:
  a lone combining mark matches wherever the file contains it.

The interaction also differs from the browser bar:
the bar sits under the source view and never covers source text,
it has no buttons,
and the active match is the reading selection while the bar is still open.

### Find behavior decisions

- The active match is the reading selection.
  Typing,
   Enter,
   and Shift+Enter set the selection to a match,
  so Ctrl+C in the source view copies its original text
  and the last active match stays selected after Escape.
  A selection that is not exactly one match leaves no active match.
- Typing selects the first match at or after the selection start,
  so extending the find text keeps the current match while it still matches.
  Enter continues after the selection end;
  Shift+Enter continues before the selection start.
- Only typing,
   Enter,
   and Shift+Enter move the selection.
  Reopening the bar,
  an external reload,
  and a file switch recompute the highlights and the count without moving it.
- Enter and Shift+Enter do nothing until matches for the displayed file,
  its current revision,
  and the current find text have arrived.
- After an external reload the selection follows Helix correspondence like any other selection.
  The active match stays active while its text survives;
  otherwise the count shows no active match,
  and the selection does not jump to another occurrence.
  Highlights for the old revision are hidden until matches for the new one arrive.
- Switching files while the bar is open,
  through the tree,
  a search result,
  or Ctrl+0 to Ctrl+9,
  keeps the bar open and recomputes matches for the new file.
  The find input keeps keyboard focus when it had it,
  and Ctrl+0 to Ctrl+9 work from the find input.
- Ctrl+F is ignored while the combined search overlay is open and while no file is displayed.
  Double-Shift opens combined search above an open find bar.
  Escape closes only the topmost surface:
  the overlay first,
  then the bar.
- Escape closes the bar from the tree and the source view as well as from the find input,
  and always moves focus to the source view.
- Empty find text clears the highlights and the count without reporting `No matches`.
- Revealing a match keeps the scroll offset when the match is already visible.
  Otherwise the visual row holding the match is centered,
  also when the match stands on a continuation row of a wrapped line.
  The offset is assigned directly,
   without easing.
- The find text is one line;
  the toolkit's `TextInput` replaces pasted line breaks with spaces.
- The find input is the application's own text box,
  the same one combined search uses;
  see [Find and search text box](#find-and-search-text-box).

### Find painting

Match rectangles come from the same shaped rows as selection rectangles (`ShapedView::range`),
so marking a match never reshapes text and ligatures stay intact.
Each state uses two visible channels:

- another match has a translucent fill in the foreground color and a 1 px boundary;
- the active match has the selection background and selection ink and a 2 px boundary;
- a selection that is not a match has the selection colors without a boundary.

Other matches are drawn over a selection,
so they stay visible inside a larger selection such as select-all.
Match ranges are part of the frame stamp:
a different match list repaints the visible source tile once,
even though only the native rectangles read the matches.

### Find bounds

- Find text is limited to 1,000 characters;
  longer text shows a diagnostic in the bar.
- Files up to 64 MiB are searched,
  because the worker copies the source into one contiguous string for the scan;
  a larger file shows a diagnostic.
- At most 10,000 matches are retained (160 kB per list);
  the count then ends in `+`,
  and navigation wraps within the retained matches.
- One background job runs at a time on the `ide-in-file-find` thread,
  with at most one waiting request.
  A newer request replaces the waiting one;
  a running scan is not interrupted,
  but its reply is discarded.
- Requests and replies carry the file-open generation,
  the content revision,
  and the query generation;
  a reply that differs in any of them is discarded.
- Only matches inside the materialized rows become native rectangles.

`test:find` covers the matcher,
 navigation,
 worker,
 painting,
 and the pinned browser differences.
`test:native` drives the bar through real window key events,
including reload,
 file switch,
 and the search overlay.
`inspect:find-guards` removes each guard in a disposable copy and checks that its named test fails.

## Find and search text box

The find text and the search query are edited in `QueryInput` (`ui/query-input.slint`),
a single-line box built on the toolkit's `TextInput` item.
It replaces the toolkit's fluent `LineEdit` because of that widget's clear control:
a cell 16 px wide whose width cannot be set from outside
(Slint 1.18.1 `widgets/fluent/lineedit.slint` lines 58 to 67,
`widgets/common/lineedit-base.slint` lines 189 to 196).
The user decided on 2026-10-05 to keep the control and give it a click target of at least 48 px by 48 px.

The box also differs in the ink of selected text.
The toolkit box draws the palette's accent ink,
which is black in the dark scheme and white in the light one.
Both boxes here take the ink of every other selection,
which native code chooses from the selection fill
(the user's choice of 2026-10-06);
see [Selected text ink](#selected-text-ink).

### Clear control

The x at the trailing end of the box empties the text.
Its click target is a 48 px by 48 px layout cell of the box:
the glyph is 16 px in the middle,
and the rest of the cell is padding that takes the click.
Being a layout cell,
it is never drawn over the text or over anything beside the box.
While the control is hidden,
the same cell is the box's 12 px trailing padding.

The control follows the toolkit's rule for when it exists:
the box has text,
is enabled,
and has keyboard focus.
A click empties the text,
reports the edit,
so the find count and highlights or the search results go with it,
and leaves keyboard focus in the box.
A press released outside the cell clears nothing.
As in the toolkit,
the control is not a Tab stop;
from the keyboard,
Ctrl+A and Delete empty the box.

Each pointer state has two marks:

- at rest,
  the glyph alone;
- under the pointer,
  a filled plate with a 1 px boundary;
- pressed,
  a stronger fill and a 2 px boundary.

The plate covers the whole 48 px cell,
so it shows the click target's real extent
(the user's choice of 2026-10-06).
Its fill and boundary are the foreground ink at reduced opacity:
the fill at 10 percent under the pointer and 24 percent pressed,
the boundary at 50 percent and 80 percent.
On the cell's top,
right,
and bottom edges the boundary lies on the box's border.
The border is composited under it but shifts its color by only 2 to 8 gray levels,
so while the plate shows,
the boundary takes the border's place there.
The focus line is drawn after the plate,
so it keeps its color in every state;
its contrast against the plate's fill is at least 3.5:1 in both schemes.
The measured colors are in `design/README.md`,
under "Applied frames of the 2026-10-06 UI batch 3b".

Accessibility tools see a `button` named `Clear find text` or `Clear search query`
whose default action clears.
The toolkit's control is not exposed to them at all.

### Behavior kept from the toolkit box

Read from `widgets/common/lineedit-base.slint` and `widgets/fluent/lineedit.slint` of Slint 1.18.1:

- The placeholder shows while the text and any input-method composition are both empty.
- Selected text has the palette's selection fill;
  its ink is not the toolkit's,
  as described under [Find and search text box](#find-and-search-text-box).
- A text wider than the box scrolls with the caret:
  while the caret moves through the text it stays 24 px inside the text area,
  and the end of the text reaches the area's edge.
- A right click opens a menu with Undo,
  Redo,
  Cut,
  Copy,
  Paste,
  and Select All;
  Copy and Select All are disabled while the box is empty.
- Editing keys,
  clipboard shortcuts,
  undo,
  redo,
  and input-method composition are `TextInput`'s own
  (`i-slint-core` 1.18.1 `items/text.rs`),
  so they are the same in both boxes.
- Focus is marked by a 2 px accent line along the bottom edge and a different fill.
- Accessibility tools see a `text-input` with its label,
  value,
  placeholder,
  and enabled state,
  and can set the value and the selection.

Not carried over,
because nothing here uses them:
the password and read-only modes,
the `accepted` callback,
and the key callbacks.

### Text box checks

`test:native` drives both boxes with real window events:

- a click on each corner pixel and on the center of the find box's clear cell clears,
  a click one pixel outside each edge does not,
  the cell's measured size is 48 px by 48 px,
  and clearing keeps focus and removes the find count and the highlights;
- the search box's clear cell has the same size,
  and clearing removes the results and keeps focus;
- editing keys,
  every entry of the context menu,
  and scrolling of a text wider than the box,
  which never reaches the clear cell;
- rendered pixels in both schemes:
  placeholder,
  focus marks,
  selection colors,
  and the clear control's three states.

`inspect:find-guards` and `inspect:search-guards` remove the cell's size,
its whole-cell click target,
its edit report,
and each half of its shown rule in a disposable copy
and check that the named tests fail.
They also put the toolkit's accent ink back on selected text
(`box-selected-ink`)
or drop it where the window and the panels pass the chosen ink on
(`find-box-ink-passed`,
`find-bar-ink-passed`,
`search-box-ink-passed`),
shrink the plate to 32 px
(`plate-whole-cell`),
and make its boundary opaque
(`plate-translucent`).
Input-method composition was not exercised:
the toolkit's public window events carry no composition event,
and the nested compositor provides no input method.
The role,
label,
value,
placeholder,
and actions of both boxes and both clear controls are tested under [Accessibility checks](#accessibility-checks).

## Accessibility checks

`test:native` reads what assistive tools are given through Slint's element handles
(`ElementHandle` of the test-only dependency `i-slint-backend-testing`),
the same accessible properties and actions the toolkit hands to the platform accessibility bridge.
The user allowed the dependency on 2026-10-06.
It is internal to Slint and has no semver guarantee,
so `Cargo.toml` requires exactly `=1.18.1`,
the resolved `slint` version;
both must be raised together.
It was already in `Cargo.lock` through `slint`,
so adding it added no package.

An element is found by its accessible label,
and exactly one element may carry it.
A row is found by its role and label inside its own list,
because the tree and the search results can list the same file name.
The texts inside tree,
search,
and location rows,
and the location list's title,
are not accessibility elements
(the user's choice of 2026-10-06):
the row or the list carries the name,
so a screen reader reads it once.
A tree row's slot badge is one of those texts,
so the row's description names its shortcut,
as in `Source file, Ctrl+3`.
A handle does not keep its element alive,
so a row is looked up again after the list changes.

- `src/native/accessible_box_tests.rs`:
  the find box and the search box are `text-input` elements with their label,
  placeholder,
  and value,
  and the find box's description is the match count (`Match 1 of 2`);
  setting the value runs find or search as typing does.
  The search box's description counts the results in the same words:
  empty without a query,
  `Searching` while a search runs,
  `Result 1 of 2` for the selected row,
  `No results` when nothing matches,
  the error text when the search failed,
  and `2 results, none selected` while no row is selected.
  Each clear control is a `button` of at least 48 px by 48 px that is offered only while the box has text,
  and its default action empties the box,
  removes the count or the results,
  and keeps keyboard focus in the box.
- `src/native/accessible_divider_tests.rs`:
  the divider is a horizontal `slider` named `Sidebar width`
  that reports the width,
  160 px and the widest width as its bounds,
  and a 16 px step.
  The increment,
  decrement,
  and set-value actions,
  and Left,
  Right,
  Home,
  and End with keyboard focus,
  change the value it reports;
  a set value above the widest width reports the widest.
- `src/native/accessible_list_tests.rs`:
  the tree,
  the search results,
  and the location list report their role,
  name,
  and row count.
  Every row is a selectable `list-item` with its name and position.
  In the tree the open file's row is the selected one,
  and opening another file through its row's default action moves the selection;
  a directory row is expandable,
  and its expand action expands it.
  In the search results and the location list the selected row follows Down and Up,
  and a location row's default action chooses it.
  In every list exactly one element carries a row's name,
  and only the location list carries its title;
  a tree row's description is `Directory`,
  or `Source file` followed by the shortcut of its slot badge,
  and the label of an entry that git ignores ends with `, ignored by git`.

Each test has a guard-removal control,
run in a disposable copy,
that was observed to fail with its guard removed:
`a11y-set-value-edits` and `a11y-clear-role` in `inspect:find-guards`,
`a11y-clear-default-action` and `a11y-result-selected` in `inspect:search-guards`,
`a11y-divider-increment` and `a11y-tree-row-selected` in `inspect:sidebar-guards`,
and `a11y-location-selected` in `inspect:language-navigation-guards`.
The hidden texts and the search count have their own:
`a11y-result-text-hidden` and `a11y-search-count` in `inspect:search-guards`,
`a11y-tree-row-text-hidden` and `a11y-tree-badge-shortcut` in `inspect:sidebar-guards`,
and `a11y-location-text-hidden` and `a11y-location-title-hidden` in `inspect:language-navigation-guards`.

What these checks do not cover:

- The platform bridge itself (AT-SPI on Linux) and a screen reader's speech:
  element handles read the toolkit's side of the bridge.
- Whether a screen reader announces the search box's new description as results arrive:
  element handles read the property,
  not the change events the bridge sends.

## Language module

The headless core in `src/language` drives Helix's language-server client on one worker thread.
[Language navigation](#language-navigation) wires it into the window.
The native layer owns one `LanguageWorker`:
it sends `open`,
 `reload`,
 `close`,
 `request`,
 and `request_hints` without waiting,
and polls `try_take_status`,
 `try_take_reply`,
 `try_take_diagnostics`,
 and `try_take_hints` from a timer.
Every result carries the file generation,
 content revision,
 and server process it answers;
the handle drops results for anything no longer displayed.
A command method returns `false` when the queue is full;
 send it again on the next poll.
`enter_project_directory` must run once at startup,
 before any thread or Helix call,
because Helix roots every server at the process working directory.
A server root outside the project is refused before anything starts.

Inlay hints and pull diagnostics are requests the worker makes on its own:
when a file is displayed,
 after a reload,
 and when `request_hints` reports the visible lines.
Nobody else would ask again,
so a request the server supersedes (`-32801` or `-32800`)
or leaves unanswered for its whole request timeout
is sent again up to 3 times,
and a hint request names the lines reported last.
After that the server owes the answer (`src/language/owed.rs`):
its next message of any kind makes the worker ask once more,
at most 3 times for one displayed text.
A failure the server states itself is final for that text.
A definition,
 references,
 or hover request that times out is reported to its caller as a timeout and is not sent again.
A hint answer for lines other than those reported last is dropped,
so a server that answers an older window after a newer one cannot replace the current hints.

Every request has the server's configured request timeout,
Helix's 20 seconds unless the definition names another.
`initialize` gets three request timeouts (`START_FACTOR` in `src/language/config.rs`):
helix-lsp never uses a client whose `initialize` timed out,
so a server that is only slow to start on a busy machine would otherwise stay failed
until the next file is displayed.
helix-lsp is given that start allowance,
and the worker bounds every request it sends by the request timeout itself.

After a reload,
a server's diagnostics pushed without a version are held back
until the server answers a request sent for the new text,
or for at most `HOLD_FALLBACK` (2 seconds),
because such a set may still describe the previous text.
The latest set pushed during the hold is shown when the hold ends,
unless a set with the current version arrived first;
a set naming a line the text does not have is discarded.

The TypeScript family uses the project's own TypeScript 7 server (`node_modules/typescript/bin/tsc --lsp --stdio`).
The launcher is looked up for the displayed file,
from the file's folder up to the project root,
and the nearest `node_modules/typescript` decides,
as Node and TypeScript resolve a package;
a pnpm monorepo opened at its root,
or a home folder opened as the project,
therefore uses the install at the repository root.
An install older than version 7 is reported as such and never skipped for a farther one.
An install above the project root is never used,
because the project root bounds what runs;
the reason names it and says to open the folder that holds it as the project.
The registry is built again when the launcher found for a file differs from the one it holds,
and `Languages::refresh` is given the file for that.
A project without any usable install shows the missing-executable state.
Server-initiated workspace edits are refused.
Every server launch passes through one launch policy in `src/language/launch.rs`.
The default,
 `src/language/confine.rs`,
 runs each server inside `/usr/bin/bwrap`
with the whole file system read-only,
 no network,
 and a cleared environment plus an allowlist,
following `doc/planning/slint-ide-write-confinement.md`.
Each server writes only its private state,
`$XDG_CACHE_HOME/monochromatic-ide/language/<project>-<hash>/<server>`,
which also holds its private `/tmp`,
 caches,
 and cargo output.
The state root is resolved through symbolic links first
and must not contain the project.
It may lie inside the project,
as it does when the home folder is the project (`~/.cache` is below it).
After the sandbox replaces `/tmp`,
 `/run`,
 and `/dev`,
the project is bound again read-only at its own path
and at Helix's working-directory spelling when that lies below one of them,
so projects below `/tmp` or `/run/media/<user>` work.
The writable state bind comes last,
after the read-only project bind,
because bubblewrap applies mounts in order and a later one covers an earlier one;
`state_inside_the_project_is_bound_writable_after_the_read_only_project` in
`src/language/confine/project_tests.rs` holds that order.
`PROJECT_MOUNT` in `src/language/confine/project.rs` is the one switch a later write mode changes.
Without bubblewrap or user namespaces,
for a project or state root below `/proc`,
or for state that contains the project,
the server shows the launch-refused state with the cause and remedy;
nothing falls back to an unconfined launch.
`LanguageSetup::unconfined()` exists only for tests and guard controls on disposable projects.

`test:language` runs the unit rules and sessions against the scripted server `ide-scripted-lsp`,
one child process per session.
Its request timeout is the product default of 20 seconds (`PRODUCT_TIMEOUT` in `tests/language/support.rs`)
wherever an assertion does not depend on it.
`IDE_SCRIPTED_HINT_STEPS` and `IDE_SCRIPTED_PULL_STEPS` script,
request by request,
which inlay-hint or pull-diagnostics request is answered,
left unanswered (`silent`),
held back until the next answer (`hold`),
or superseded (`modified`,
`modified-notify`),
so `tests/language/again.rs` and `tests/language/windows.rs` do not depend on how busy the machine is.
`IDE_SCRIPTED_INIT_GATE` names a file the server waits for before it answers `initialize`,
so a test can act while the server is still starting.
Every report line carries its wall-clock time in milliseconds (`at`),
and every message the server writes adds a `sent` line.
Each session keeps its project and report in `/dev/shm` when that exists:
the scripted server writes its report before it answers,
and on a disk the rest of the machine keeps busy one write blocked it for up to 10 seconds.
The file-opening,
reload,
directory,
and search worker tests take their fixtures from `tests/memory_fixture`,
and the native external-reload tests from `memory_project` in `src/native/find_tests.rs`,
both in `/dev/shm` when it exists:
on btrfs,
reading a fresh file or creating and removing files waits for filesystem transactions,
which took longer than these tests wait while the machine flushed.
The native external-reload tests also wait for the displayed file's watch before they change it,
and change it as an atomic save does (`replace_file`),
because an in-place write can be read after its truncation and before its text.
`IDE_LANGUAGE_TEST_LOG=1` makes a session print the worker's debug log
and the helix-lsp protocol log to standard error.
`inspect:language` runs all five feature paths,
 a reload,
 and the stale-reply case
against real confined TypeScript and Rust servers on disposable projects,
and checks that the servers left each project tree unchanged.
`inspect:language-confinement` runs the write-confinement acceptance tests:
write,
 escape,
 and delegation probes from inside the server trees,
the mount and environment audits,
 fail-closed cases,
and the same fixtures unconfined as the guard control.
`inspect:language-guards` removes the fencing,
 readiness,
 edit-refusal,
 and ask-again guards in a disposable copy
and checks that their named tests fail.

`inspect:language-lifecycle` opens a file,
sends one hover,
closes,
and lets the worker shut down against the same real confined servers.
It runs with every `helix-lsp` record visible (`RUST_LOG=ide_app=debug,helix_lsp=debug`)
and fails on any ERROR-level record,
on shutdown error text such as `context canceled` at WARN or ERROR or outside a record,
when the healthy-server records `helix-lsp` writes are missing or not below WARN,
and on any leftover process.
Its second part ends the application process with `SIGKILL`
and records which server and sandbox processes remain after 0,
1,
5,
and 10 seconds.
The servers still print what they printed before:
the TypeScript 7 server reports its own exit as `context canceled`
(`doc/troubleshooting/typescript-7-lsp-exit-context-canceled.md`),
and rust-analyzer warns about a user configuration file that does not exist
(`doc/troubleshooting/rust-analyzer-notify-missing-user-config.md`).
The application re-labels those lines,
as described in "Log",
instead of letting `helix-lsp` log them at ERROR
(`doc/troubleshooting/helix-lsp-transport-error-level-records.md`).
`tests/language/quiet.rs` is the scripted-server form in the container suite:
`quiet::clean_lifetime_logs_no_error_level_record` makes the scripted server produce all three re-labelled shapes
(`IDE_SCRIPTED_STDERR` writes one line to standard error at `shutdown`,
`IDE_SCRIPTED_HOVER=modified-once`)
and requires no ERROR record and each shape at its new level;
`quiet::unknown_helix_error_records_keep_their_level` requires a refused `initialize` to log two ERROR records.
`inspect:language-lifecycle-guards` observes both in a disposable copy,
with each re-labelling removed,
with the re-labelling widened to every transport error,
with a full log queue that waits for room,
and with the worker's shutdown request,
its wait for servers to end,
or its reaping removed.

When its handle is dropped,
the worker sends `shutdown` and `exit` to every server,
waits up to one second for the processes to end,
and drops its runtime,
which kills every server that is left.
It then waits,
for at most two more seconds,
until the kernel lists no child process of the worker thread (`src/language/reap.rs`),
so an ended server is not left in the process table as a zombie.
`lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns` covers a server
that ignores `exit` and the end of its input (`IDE_SCRIPTED_LINGER=1`).
A killed process that the kernel needs more than two seconds to end,
which was seen in measurements where processes stalled for seconds,
stays a zombie until the application exits.
`inspect:language-reap-rate` counts leftover processes by kind over many lifetimes,
with the reaping and with the fixed 50 ms pause it replaced;
`doc/troubleshooting/tokio-dropped-child-zombie-after-last-park.md` has the source trace and the measurements.

### Servers from mise

When the project folder has a mise configuration at or above it,
the language servers are looked up in the tool directories mise prints for that folder
and started from there;
a folder without one keeps the plain `PATH` lookup.
The desktop launcher's `PATH` is a snapshot of the login session:
it holds the mise shims directory but not the tool directories of a repository,
and a shim run inside the sandbox stops with "No version is set for shim".
Nothing of any repository is named in the code:
no path,
no tool list,
no server name.
`src/language/mise.rs` has the decision,
`src/language/mise/command.rs` the sandboxed run,
`src/language/mise/parse.rs` what mise prints,
`src/language/mise/search.rs` the lookup,
and `src/language/config/toolchain.rs` the registry's use of the answer.

#### What is asked, once per registry build

Mise itself is found as `mise` in the application's `PATH`,
else at `~/.local/bin/mise`;
without one the folder is not a mise project as far as the application can tell.
Three commands run in the folder (never with the folder as an argument,
so a name that starts with a dash cannot become an option):

- `mise config ls --json` from the project folder and again from `/`.
  The files the first lists and the second does not are the project's own configuration,
  whatever their names or layout (`mise.toml`,
  `.mise/config.toml`,
  `.tool-versions`,
  a parent's file);
  the user's global configuration appears in both and says nothing about this folder.
  No file name is built into the code.
- `mise bin-paths`,
  only when the project has a configuration of its own.
  It prints the directories of the tools the configuration lists,
  one per line.
  Only absolute entries that are existing directories are kept.

The answer is kept for the session.
It is asked again only when it may have changed:
two seconds or more after a failure,
or two seconds or more after the last answer when a server of the file's language is missing
(the user may have run `mise install`).
A folder mise does not serve stays that way until the application restarts.
All runs of one resolution share a limit of 10 seconds (`TIME_LIMIT`),
and they run on the language thread,
so a hung mise never blocks the window.

#### Why not `mise exec`

`mise exec -- SERVER` as the launched command would give every server the project's whole `[env]`.
`mise env --json` in this repository prints 42 variables,
26 of them with a name that contains `KEY` or `TOKEN` (names only were read,
never values):
decrypted secrets.
That bypasses the cleared environment and allowlist of `src/language/confine/recipe.rs`.
So mise is used only to say where the tools are,
and the servers start from those directories with the same environment as before plus a `PATH`.

#### Mise runs in the servers' sandbox

A trusted configuration's `[env]` templates run shell commands on every mise command,
including `mise config ls`
(probe:
a template that touches a marker file ran for `config ls`,
`bin-paths`,
`which`,
`env` and `exec`;
a `[hooks]` entry did not run for any of them).
Run on the bare host,
that is project code with network access and write access.
So mise runs through `confine_with`,
the function every server goes through:
bubblewrap,
no network,
the whole file system and the project read-only,
a cleared environment plus the allowlist,
a private `/tmp`,
and the mise state kept read-only (mise's attempt to link the configuration into its state directory fails;
that line is dropped from messages).
This is a deviation from "resolve outside the sandbox" and was measured:

- `mise bin-paths` in a two-tool project:
  235 ms plain (standard deviation 43),
  319 ms sandboxed (41),
  15 runs each.
- The same in this repository:
  413 ms plain (80),
  387 ms sandboxed (152);
  inside the spread.
- `mise config ls --json`:
  24 ms plain,
  `mise which taplo`:
  35 ms plain,
  `mise exec -- true`:
  114 ms plain.
- Production runs on a loaded host (load average about 60):
  179 to 639 ms for all three commands together
  over four `inspect:language-mise` runs,
  and 415 ms in a headless window run.

Variables set for every mise run (`SAFE_ENVIRONMENT` in `command.rs`;
an environment variable beats a project's `[settings]`,
measured with `mise settings get`):
`MISE_AUTO_INSTALL`,
`MISE_EXEC_AUTO_INSTALL`,
`MISE_NOT_FOUND_AUTO_INSTALL`,
`MISE_TASK_RUN_AUTO_INSTALL`,
`MISE_LOCKFILE` and `MISE_YES` are `false`,
`MISE_OFFLINE` is `true`,
and colors and the update notice are off.
Control:
in a disposable data directory,
`mise exec -- true` with auto install on exits 1 while trying to fetch versions,
and with it off exits 0.
Variables that only name places (`MISE_DATA_DIR`,
`MISE_CONFIG_DIR`,
`MISE_STATE_DIR`,
`MISE_GLOBAL_CONFIG_FILE`,
`MISE_ENV`,
`XDG_CONFIG_HOME`,
`XDG_DATA_HOME`,
`XDG_STATE_HOME` and a few others) are passed on when set;
every other variable of the application is cleared.
The application never runs `mise trust`,
`mise install` or `mise exec`.

#### Trust

Mise's own rule decides.
A configuration with only `[tools]` needs no trust (measured);
one with `[env]`,
`[settings]`,
hooks or templates is refused until the user runs `mise trust` on it.
The state is read from mise's state directory,
which the sandbox can read.
When mise refuses or fails,
the servers are looked up on `PATH` as before,
mise shims are skipped,
and every file with servers gets this in its language support notice:
that mise could not be used for the folder,
the remedy,
and mise's own message (without its version footer).
For an untrusted file the remedy names the file mise named and says the user runs `mise trust` on it
(the IDE never does),
then displays a file again.
For any other failure it points at `mise doctor`.
A mise that does not finish in 10 seconds is stopped and reported the same way.
A sandbox that cannot be set up is not a mise failure:
the servers are refused for that reason anyway.

#### What the servers get

Mise's directories lead the lookup,
then the application's `PATH`;
the first executable regular file that is not a mise shim wins.
A shim is a symbolic link whose final target is named `mise`.
The `PATH` inside the sandbox is the same list (mise's directories,
then the definition's or the application's `PATH`),
so a Node launcher's `#!/usr/bin/env node` finds the project's `node`.
Directories whose name holds a colon cannot stand in a `PATH` and are left out with a warning.
A tool directory below `/tmp`,
`/run` or `/dev` is bound again read-only after the sandbox replaced those locations,
before the state bind,
and never writable (`read_only_binds` in `LaunchRequest`,
`src/language/confine/project.rs`).
A tool that the configuration lists but that is not installed is reported with mise's own warning
(`mise WARN missing: taplo@0.9.0`) in the reason of the server that is missing.
Variables a tool would add to its environment (`GOROOT`,
`JAVA_HOME`) are not passed.

#### Checks

`test:language` holds the unit rules:
listings,
directories with spaces,
quotes,
newlines and leading dashes,
messages with escape sequences,
the sandbox request for mise (every safety variable,
read-only mounts,
the project never an argument),
the time limit,
the lookup order,
the shim refusal,
the retry rule and the notice text.
`inspect:language-mise` runs the production setup on disposable projects with the launcher's `PATH`:
a Node server (`bash-language-server`) answers a hover,
a binary server (`taplo`) and the Markdown server (`marksman`,
which also answers a hover on a wiki link) start,
an untrusted configuration gives the notice,
a folder without mise keeps "was not found on PATH" for a server nobody installed,
a fake `mise` that never answers is stopped at 10 seconds,
no process remains,
and every project tree is the same before and after (a control proves the comparison sees a created `mise.lock`).
The Markdown server roots its workspace at a folder with `.marksman.toml` or `.git`;
without one it answers no hover.

Details of the probes,
the failure causes and the tradeoffs are in
`doc/troubleshooting/mise-servers-for-the-confined-ide.md`.
Mise run outside the sandbox with `MISE_TRUSTED_CONFIG_PATHS` left trust records in
`~/.local/state/mise/trusted-configs`;
the sandboxed run cannot write there,
so the application never trusts anything.

## Language navigation

In the source view,
Ctrl+B goes to the definition of the symbol at the caret,
and at the definition itself it finds the symbol's references.
Ctrl+click goes to the definition of the character under the pointer.
Ctrl+Q shows hover information for the caret,
and a pointer resting 350 ms on one character shows it for that character.
Nothing of this is visible until it is used:
there is no status bar and no language indicator,
except the one entry that says why a file has no language support
([Why a feature is missing](#why-a-feature-is-missing)).

### Definitions and references

One target is opened directly.
In the displayed file the caret moves to the start of the target,
and the target's line is centered when it was out of view.
Another project file opens through the ordinary file open,
so it enters Ctrl+0 to Ctrl+9 history,
gets its tree badge,
and is revealed in the tree.
A file outside the project opens read-only,
with a bordered `Outside project` label before its path in the file label;
it gets no tree row,
no history slot,
and no project root of its own,
and Ctrl+0 returns to the last project file.

Several targets are listed beside the caret line,
below it when there is room and above it otherwise,
so the list never covers that line.
The list's title counts them,
for example `3 references` or `2 definitions`,
and each row shows the project-relative path and the one-based line,
or the server's address for a location that cannot be opened.
Outside-project rows say `Outside project`
and unopenable rows say `Cannot open`.
The list takes keyboard focus:
Up and Down move the selection and wrap,
Enter or a click opens the selected location,
Escape or a click outside closes the list,
and every way of closing returns focus to the source view.
Tab stays in the list and Ctrl+F waits until it is closed,
as with the search overlay.
Rows are 48 px tall;
the selected row has the selection fill,
a heavier weight,
and a boundary,
and its text is drawn in the ink described under [Selected rows](#selected-rows).
Accessibility tools see a `list` named by the title,
with `list-item` rows that report their selection and open on their default action.

A location the server names but the reader cannot open,
such as an `untitled:` or `jdt:` address or a missing file,
is explained instead of opened,
for example `Cannot open file:///gone.rs: the file does not exist.`

### Hover

Hover content is shown as plain text in JetBrains Mono in a popup beside its line:
below it when it fits,
above it otherwise,
and on the roomier side with a scrolling body when neither fits,
so the popup never covers the hovered line.
It is at most 640 px or 60% of the window wide and 320 px tall.
Markdown code-fence lines are dropped,
runs of blank lines collapse,
and content longer than 8,000 characters is cut with an ellipsis line.

The popup closes on Escape,
on any caret movement,
on scrolling,
on a reload of the file,
on a file switch,
and when the search overlay opens.
A pointer hover also closes when the pointer leaves both the source view and the popup,
so the pointer can move into the popup to scroll it,
and when the pointer rests over no character,
such as past the end of a line,
on a blank line,
or over the line numbers.
Resting over no character asks the server nothing.

### Notes

When an action the user asked for cannot be done,
a note appears in the same popup beside the line it was asked about.
A note has three channels that set it apart from hover content:
a heavier weight,
the interface typeface,
and a bar along its leading edge.
Notes are declared as an assertive live region and hover content as a polite one;
announcement by a screen reader was not tested.
A note closes like hover content,
and a new action replaces it.
A resting pointer shows a note only when the file's language support itself is missing or failing,
and then once for each displayed file ([Why a feature is missing](#why-a-feature-is-missing));
every other failure of a resting pointer is only logged.

Each note names the reason and the remedy:

- a server still starting:
  `scripted-ls is still starting (Indexing). Press Ctrl+B again in a moment.`,
  with the server's progress title when it reports one;
- a missing server program,
  with the reason the Language module reports,
  which for TypeScript names the project dependency to install,
  followed by `After installing it, press Ctrl+B again.`;
- a feature the server does not offer:
  `scripted-ls does not offer go to definition.`;
- a failed request,
  with the server's error code and message,
  a timeout,
  or a server that stopped while answering,
  each with the key to try again;
- a request the server set aside because its analysis kept changing;
- an empty answer:
  `No definition found.`,
  `No usages found.`,
  or `No hover information at this position.`,
  followed,
  while the server reports work in progress,
  by the advice to press the key again when it finishes;
- a server that was not started because it could not be confined:
  `typescript-native was not started, so go to definition is not available for this file:`
  followed by the cause and,
  last,
  the remedy the launch policy names;
- a server that cannot follow external changes,
  a failed start,
  a server root outside the project,
  and a file outside the project with no running server of its language;
- a file without a recognized language or without a configured server;
- an answer overtaken by a reload of the same file:
  `The file changed on disk before the hover information arrived. Press Ctrl+Q again.`;
- language support that could not start or stopped,
  with the reason and the need to restart the application.

The Language module re-resolves server programs and starts failed servers only when a file is displayed.
When the latest status shows a missing program,
a failed start,
an unsynchronized server,
or an outside-project file without a ready server,
and no server is ready,
an explicit action first displays the file to the worker again,
so the remedy "press Ctrl+B again" works without switching files.

### Why a feature is missing

Before 2026-10-14 a file whose server was missing,
failing,
or without hover or inlay hints looked like a file
where the features were broken:
a resting pointer logged every failure and nothing on screen mentioned hints at all.
Measured with the installed build on a disposable copy of the repository,
opening the TypeScript package folder,
or a folder above the repository,
as the project showed no hint
and no hover for `index.unit.test.ts`,
and left no `language` folder in the cache,
the same as the user's real `~/.cache/monochromatic-ide`,
while opening the repository root showed both.
The reasons are now on screen in two places.

The entry:
the Language module adds one `Information` entry named `language support` to the displayed file's diagnostics
(`src/language/notice.rs`,
joined to the snapshot in `Worker::publish`),
so the source view draws it as message rows over line 1 with the gutter letter `I`,
for example `Information (language support): Hover, inlay hints, and go to definition are not available for this file.`
Its range is empty at the first character,
so no text is underlined.
It is made from the status the window already polls and says what is lost,
the server,
the cause,
and the remedy:

- no server runs and none is starting:
  that hover,
  inlay hints,
  and go to definition are not available,
  then one line for each server naming its state:
  a program not found (`the program 'taplo' for taplo was not found on PATH`),
  the TypeScript install that was not found and where it was looked for,
  a launch the confinement refused with its cause,
  a start that failed,
  a server that stopped,
  one that cannot follow the file,
  or one that would be rooted outside the project;
  and,
  when displaying the file again can help,
  that servers are looked up again when a file is displayed;
- a server runs but offers no hover or no inlay hints:
  `Inlay hints are not offered by slint-lsp, so none are shown.`
  `slint-lsp` and `qmlls` both announce no inlay hints on this host;
- a server runs and a request failed:
  `The inlay hint request to typescript-native failed: the server answered with error -32603: ...`,
  or `the server did not answer in time` after the retries the worker makes on its own.
  The failure is kept for that server process until a later answer clears it.

Nothing is drawn while a server is still starting,
for a file without a recognized language or without a configured server,
for a file outside the project,
or when every feature works.
The entry stands on line 1 and leaves the view with it when the file is scrolled.

The resting pointer:
the first rest on a displayed file whose answer is about the support itself,
a failed request,
a server that cannot follow the file,
a feature the server does not offer,
or no running server for a file that has servers configured,
shows the same note Ctrl+Q shows,
drawn as a note and ended like hover content.
Each displayed file gets it once.
An empty answer,
a request the server set aside,
and a server that is still starting stay silent,
because they describe one position or one moment.
Decided on 2026-10-14:
the entry is the permanent surface and the pointer note is the pointer's answer,
so a user who scrolled away from line 1 still learns why.

A failed start states what the server wrote:
`the server process ended before it finished starting` was all the window said,
and the cause was in the log.
The last four standard-error lines,
each cut to 200 bytes,
now follow the reason on lines of their own,
once the server's standard error ended or the settle time of the log record passed (`Internal::EndCause`);
the start deadline carries the lines directly.
The log record of the ended server is unchanged because the hand-over reads a copy of the kept lines.
For `rust-analyzer` on this host the lines read `info: falling back to ".../mise/shims/rust-analyzer"`
and `error: infinite recursion detected`
(`doc/troubleshooting/rustup-proxy-mise-shim-rust-analyzer-recursion.md`).

Causes outside the IDE that the entry now names:
a language server that is not installed,
a project without TypeScript 7 in the project folder or below it,
and a server version that does not announce the feature.
Installing the server or the project's dependencies,
then displaying the file again,
ends the first two;
the third is the server's own capability.

Checks:
`src/language/notice_tests.rs` holds the wording of every state without a server,
`tests/language/notice.rs` the entry through the worker and the scripted server
(`IDE_SCRIPTED_FEATURES=hover` for a server without hints,
the hint step `error`,
`IDE_SCRIPTED_HOVER=silent`),
`tests/language/quiet.rs` the cause of a failed start,
`src/language/config_tests.rs` the TypeScript lookup,
and `native::language::hover_tests::resting_pointer_explains_a_missing_server_once_for_each_file` the pointer.
`inspect:language-guards` removes the lookup from the file's folder,
the nearest-install rule,
the rebuild when the launcher moves,
the publishing of the entry,
its wait for a starting server,
the kept hint failure,
the copy of the lines for a failed start,
and their arrival in the reason
(`typescript-document-folder`,
`typescript-nearest-install`,
`typescript-launcher-moved`,
`notice-published`,
`notice-waits-for-start`,
`hint-failure-kept`,
`cause-hand-over-copies`,
`cause-reaches-reason`),
and `inspect:language-navigation-guards` the pointer note and its once-per-file rule
(`pointer-explains-support-problem`,
`pointer-explains-once`),
each observed failing with the named test.

### Stale results

The handle's fence advances only when an open or reload command was queued,
so the window checks every reply again before applying it:
the reply must carry the number of the request the window waits for,
of the same kind,
and both the reply and the request must describe the displayed file generation and revision.
A request is sent only after the worker holds the displayed text,
because the worker drops a request for other text without replying;
a waiting request whose text is no longer displayed is dropped,
and on a reload of the same file the note asks to try again.
The popup and the list close when the displayed text changes,
and a language target waiting for its file is dropped by any later open.
Pointer hover and key actions use separate request slots,
so hovering never replaces a pending Ctrl+B,
and a resting pointer's late answer never replaces the location list or a note.

### Synchronization and annotations

The window tells the worker about each displayed file,
and about each accepted external reload with the copy taken before `Document::apply_reload` consumed it;
a command the queue cannot take is sent again on the next 20 ms tick.
The visible lines are reported for inlay hints once they stayed the same for 200 ms,
a chosen value,
and at once for newly displayed text.
Accepted hint and diagnostic snapshots are stored in `State::annotations`,
the store the section "Inlay hints and diagnostics" describes,
which hands a snapshot out only for the stamp of the text being drawn.
The tick that stores one repaints the source once (`src/native/language/poll.rs`),
so hints and diagnostics appear without another event.
The hover popup is anchored to the top of the hovered line's block of virtual rows
or to the bottom of its code row,
so it covers neither the line nor the hints and messages over it.
At window close the worker is dropped after the window is gone,
which waits up to about a second for servers to exit.

### Differences from editord

editord's language client is in `package-paused/desktop-daemon/editord/src/client/app/`.
Ctrl+B,
Ctrl+click,
the references fallback,
opening a single reference directly,
`path:line` rows,
wrapping Up and Down,
Enter,
Escape,
and the 350 ms pointer rest follow it.
Deliberate differences:

- Ctrl+Q shows hover information for the caret.
  editord has no keyboard hover;
  Ctrl+Q is the Quick Documentation key of the JetBrains keymap editord's other keys come from.
- Hover content drops Markdown fence lines;
  editord shows the raw text through `textContent`,
  fences included.
- The hover popup sits beside the hovered line at the hovered character,
  not 8 px below the pointer,
  and an empty answer for a new character closes it instead of leaving the previous content.
- Notes stay until dismissed instead of disappearing after 2 seconds,
  name a remedy,
  and are declared as a live region for assistive technologies.
- Several definitions are listed;
  editord's server keeps only the first.
- The list has a title,
  48 px rows,
  click to open,
  and modal focus;
  editord's popover has none of these.
- A reload or a file switch closes the popup and the list.
- A file outside the project gets no history slot.
- There is no Find Usages key besides Ctrl+B at the definition,
  no back navigation after a jump,
  and no indication while a request is pending.

### Language navigation checks

`test:native` builds the scripted server `ide-scripted-lsp`,
then drives every path with real window key and pointer events:
definitions in the same file,
another project file,
and outside the project,
the references list with its keys,
outside click,
and focus return,
unopenable targets,
hover by key and by the resting pointer with its placement and dismissals,
each server-state note,
answers overtaken by a reload and by a file switch,
a worker that could not start,
and closing while a request is pending.
`IDE_SCRIPTED_DEFINITION` and `IDE_SCRIPTED_REFERENCES` give the scripted server's answers as JSON.
`inspect:language-navigation-guards` removes each native guard in a disposable copy
and checks that its named test fails.
`inspect:native` takes `IDE_NATIVE_PROJECT` and `IDE_NATIVE_FILE`
to open a disposable project below the system temporary directory instead of its generated fixture.

## Inlay hints and diagnostics

The source view draws inlay hints and the displayed file's diagnostic messages on virtual rows
that stand over the line they belong to,
and underlines the characters each diagnostic marks.
The user decided this placement on 2026-10-05:
no hint inside or after a line,
hints and messages on rows of their own as editord draws them
(`package-paused/desktop-daemon/editord/src/client/inlay/`),
and spacing that makes a block of such rows read as part of the line under it.
An earlier build drew hints after the end of their line and showed messages in a card at the caret;
the design record `design/README.md` keeps its frames as history.

Snapshots come from the Language module's `HintsSnapshot` and `DiagnosticsSnapshot`.
The window state keeps them in `State::annotations`,
an `ide_app::annotation::Annotations` (`src/annotation.rs`).
A language poll stores each polled snapshot with `accept_hints(displayed, text, snapshot)`
or `accept_diagnostics(displayed, text, snapshot)`,
which refuse a snapshot of any other text,
and renders once when either accepted;
accepting draws nothing by itself.
`annotate::set_annotations(window, state, hints, diagnostics)` in `src/native/annotate.rs` replaces both and renders;
tests use it.

A snapshot is painted only while its stamp names the displayed text:
the file generation and the content revision.
Hints accumulate within one revision.
A server is asked about the lines around the view,
so each answer replaces the hints of the lines it was asked about and keeps those of every other line:
rows seen once are still there when the view returns to them.
A hint of an answer that stands on a line the answer did not ask about is left out,
because a server answers with the hints of every node that reaches into the asked lines,
and a statement that began above them answers with the hints of its first line:
TypeScript 7.1.0-beta answered a request for lines 100 to 200 of a 286-line test file with the four hints of line 96,
and keeping them added those four hints to line 96 again with each such answer,
so its block grew by a copy per answer in the code view and in the minimap
(`doc/troubleshooting/typescript-7-inlay-hints-outside-the-requested-range.md`;
`hints_outside_the_asked_lines_are_not_part_of_an_answer` and
`an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about`;
`inspect:source-guards` cases `hints-asked-lines` and `hints-asked-lines-window`).

### Placement

A line with hints or messages gets a block of virtual rows directly over its code row.
Hint rows come first and message rows after them,
so messages are nearest to the code.
These are the values at scale 1;
each is a logical length and is multiplied by the display scale:

- A code row is 24 px high (`row_map::CODE_ROW`).
- A virtual row is 16 px high (`virtual_row::ROW_HEIGHT`),
  set in Inter Variable at 13 px (`ROW_TEXT`).
- A block starts with 10 px of empty space (`BLOCK_GAP`).
  Nothing separates its rows from each other or from the code row they belong to.
- A line without hints and messages has no block and no gap.

Measured on `design/screenshots/2026-10-05-hint-rows/overview-dark-1x.png`,
the empty space between the ink of a hint row and the ink of its own line is 10 px,
and between that hint row and the ink of the line before it 22 px;
baseline to baseline the distances are 20 px and 30 px.
That difference is the only sign of which line a block belongs to:
there is no box,
rule,
or background.

#### Hint rows

A hint is drawn at the pixel x of the character it is placed before,
taken from the row the production shaper laid out,
so tabs,
CJK,
combining marks,
and ligatures put it where the character is.
Hints are packed in position order,
as editord packs them:
a hint goes on the current row when it starts at least 8 px (`virtual_row::HINT_GAP`)
after the end of the previous hint on that row,
otherwise it starts a new row,
and every later hint continues on that new row.
A hint at the end of a line is drawn on its row at the x of the line end;
nothing is drawn after a line.
Labels are shown as the server sent them,
without padding spaces,
in the source ink at 70 percent,
without a box.
On a wrapped line a hint is read at the x of the visual row its character is drawn on
(see [Rows fitted to the text area](#rows-fitted-to-the-text-area)).

The packing never looks back at an earlier row,
so a call with several short arguments can take one row per argument:
`resize(box(1, 2), 3, 4, 5)` takes five rows in `overview-dark-1x.png`.
Placing each hint on the first row with room is the alternative;
the design record compares the two as the question `hint-packing`.

#### Message rows

Every diagnostic that starts on a line gets rows of its own in that line's block,
starting at the pixel x of the diagnostic's first character.

- Text:
  `Error E0308 (rustc): mismatched types`,
  that is the severity in words,
  the code and the source when the server gave them,
  and the message.
- Ink:
  the severity's ink.
  The severity word is the second channel besides color,
  and the underline style is a third.
- Order:
  worst severity first,
  then source order.
- Line breaks:
  each line of a message starts a row.
  Blank lines are dropped;
  indentation is kept.
- Long lines:
  a line first wraps at a blank once it reaches 80 characters (`WRAP_COLUMNS`),
  wide characters counting as two;
  a longer word is cut.
  Every row after a message's first is indented by 16 px (`CONTINUATION_INDENT`).
  That first break is a count of characters,
  so the rows at the first break are known without shaping their text.
  A row wider than the text area is then broken again to fit it
  (see [Rows fitted to the text area](#rows-fitted-to-the-text-area)).
- Cap per message:
  12 rows (`MESSAGE_ROWS`).
  The last row of a longer message reads `… N more lines`.
- Cap per line:
  8 messages (`LINE_MESSAGES`).
  One more row then reads `N more on this line, the worst: Warning`,
  naming the worst severity left out.
- A range over several lines:
  its message stands over the first line of the range only;
  every line of the range is underlined.

A message cut by either cap is spelled out in full in the source view's accessible description
while the caret touches its diagnostic,
up to eight problems.
There is no pointer or keyboard way to read the rest on screen yet.

#### Rows fitted to the text area

The source view has no horizontal scrolling,
so no row of a block may pass the right edge of the text area.
The text area is the width the code wraps at (see [Soft wrap](#soft-wrap)),
and the blocks are made for that width and made again when it changes.
No IntelliJ source read for this work speaks for these rows:
none of it wraps a block inlay or draws a diagnostic message in the text.
They therefore wrap at the same width as the code.
`design/screenshots/2026-10-12-wrap-annotations-applied/` shows them at two window widths in both schemes.

- A block stands over the first visual row of its line,
  whatever row its hints and messages belong to.
  The user decided that virtual rows stand over the code line (2026-10-05,
  see [Placement](#placement)).
  IntelliJ places a block inlay over the visual row that holds its offset;
  see [Known differences from IntelliJ](#known-differences-from-intellij).
- A hint is packed against the line as it is broken at the wrap width,
  not against the unbroken line.
  Its x is the x of its character on the visual row the character is drawn on.
  A hint that would pass the right edge of the text area ends at the edge instead.
  A label wider than the text area breaks,
  with the shaper that paints it,
  into pieces on rows of their own;
  the later pieces start at the left edge.
- A message line with a row wider than the text area is joined and broken again at its width as a whole
  (`MessageRow::gap` records how a row follows the one before it:
  a blank,
  or nothing where a word was cut),
  so only its last row is short and a cut word stays whole.
  The first row of a message may use all of the width and every later row is 16 px in,
  so a row that is not itself continued breaks in two steps.
  The pieces after the first are continued rows.
- A message keeps its cap of 12 rows after this second break.
  The last row then reads `… N more lines`,
  where N counts the rows left out.
- A message whose widest row would pass the text area from the x of its diagnostic moves left,
  as a whole,
  until the widest row ends at the edge.
- The height of a block is the number of its rows after these breaks,
  so the vertical map and the scroll extent follow the width.
- The width of every message row is measured once for each display scale and kept,
  so a new width shapes again only the rows wider than it.
  `Annotations::work` counts the lines packed,
  the rows measured,
  and the rows broken by shaping;
  the structural tests compare the counts before and after a resize.
- A new wrap width makes the packing of a hinted line stale.
  The lines near the view (the requested tile and one tile on each side) are packed again at once;
  the others keep what they had until the view comes near them,
  which the row key records (`Annotations::stale_in`),
  so a resize costs the window thread one packing for each hinted line near the view
  and not for each line the view ever passed.
  Lines that wait keep the heights of the old width,
  which move the lines after them,
  never the lines in view.
- A hint or message that arrives while the worker builds the tile for a new width makes that tile stale.
  The window drops it and asks the worker for another at once (`tile::apply`),
  because the displayed tile stays on
  screen meanwhile and no later event is certain to ask.
- The block of a wrapped line and the hover,
  press,
  and Ctrl+click rules are those of any block:
  a point over the block belongs to its line and to no character,
  and a press on it acts on the first visual row of the line.
  Underlines stand on the visual rows they mark,
  with or without a block.

#### Virtual rows are not source text

- The caret never stands on a virtual row.
  Up,
  Down,
  and the page keys move between code rows
  (the visual rows of a wrapped line included).
- Selection fills cover code rows only,
  and copying yields the source text alone.
- Find matches source text only and paints its rectangles on code rows.
- Line numbers and gutter letters stand beside code rows.
- A press on a virtual row acts on the code line under it at the same x:
  the click,
  double click,
  triple click,
  Shift+click,
  or drag is the one a press on that line would be.
  A block belongs to its line for the pointer as it does for the eye.
- A resting pointer and Ctrl+click on a virtual row are over no character,
  so no hover and no definition request is sent.
- The hover popup is anchored to the top of the hovered line's block or to the bottom of its code row,
  so it covers neither the line nor its rows.
- Hints are not exposed to accessibility tools.
  The source view's accessible description starts with the problems at the caret.

Caret movement,
selection,
hit testing,
double-click words,
find rectangles,
tab stops,
copying,
and reload correspondence within a line are those of the line without annotations,
because nothing is inserted into a line.

### One vertical mapping

`RowMap` (`src/row_map.rs`) is the only code that converts between a vertical pixel and a source line.
It holds the height of every block,
sparsely with running sums,
and answers `block_top(line)`,
`code_top(line)`,
`code_bottom(line)`,
`height()`,
`locate(y)`,
`line_at(y)`,
and `page(offset, height, forward)` by binary search.
Painting,
hit testing,
caret and selection geometry,
find rectangles,
line numbers,
caret following,
the reveal of search results and jump targets,
the page keys,
scroll restore after a reload,
the line range reported for inlay hints,
and the language popup's anchor all ask it;
nothing else multiplies a row number by a row height.
A line takes one code row for each visual row it is wrapped to (see [Soft wrap](#soft-wrap)),
so the map also holds the visual row count of every line (`src/line_rows.rs`),
in shared chunks of 256 lines with the rows above each chunk summed,
and `locate`,
`page`,
and the extents count a wrapped line by its height.
Blocks come from `Annotations::assemble` (`src/annotation/blocks.rs`),
which packs a line's hints once per snapshot,
scale,
and wrap width,
and breaks message rows to the text area (see [Rows fitted to the text area](#rows-fitted-to-the-text-area)).

### Overscan tile

The materialized lines span three view heights:
one view height above the view,
the view,
and one view height below it.
`viewport::place` keeps that tile while the view stays inside it with `RECENTER_EDGE_VIEWS` (0.25 view heights)
of room at each edge,
and re-centers it only when that room runs out,
so a re-center comes after about 0.75 view heights of scrolling.
The first line of a re-centered tile starts one view height above the view's top line,
and the tile runs `TILE_SPARE_LINES` (2) lines past its bottom edge.
The window's scroll moves the image on its own:
a `viewport-changed` callback for a view inside the tile returns before any shaping,
painting,
or model update.
Syntax-highlight and text changes still render at once,
because their own callbacks call `render`.
The frame stamp keeps the tile's first line,
line count,
and wrap width,
so it does not change inside the tile.
A diagnostic or hint inside the tile repaints the tile once,
even when it is above the view.
The document position's viewport line follows the view's top line on every scroll step,
so reload anchoring and navigation see the line the reader is looking at.

Shaped rows are cached by line (`src/shaped_row_cache.rs`) for one text revision and display scale.
A re-centered tile shapes only the lines it did not hold,
and the cache keeps at most `ROW_CACHE_LINES` (512) lines,
evicting those farthest from the view;
the bound is above the lines one window-thread step for a new width may bring in (`REWRAP_LINES`),
so a step never evicts its own lines.
A change of syntax styles clears the cache (`TextShaper::forget_rows`).

A re-center caused by the reader's own scrolling is built on a worker thread
(`src/native/tile_build.rs`,
its thread loop in `src/native/tile_build/worker_thread.rs`);
so is the tile for a new wrap width when the window thread's allowance does not cover it
(see [A new width on the window thread](#a-new-width-on-the-window-thread)).
The window thread queues an owned job with a document snapshot,
the styles,
the row map,
and the paint inputs.
The worker keeps its own `TextShaper`,
and it builds only the newest queued job.
A `slint::Timer` polls the worker every 20 ms (`src/native/tile.rs`).
The displayed tile stays on screen until the result arrives,
so a scroll never shows a blank frame.
A result is swapped in only when it still holds the view and its paint inputs still match,
and the image,
its offset,
the line numbers,
and the rectangles arrive in one step.
A result that no longer matches is discarded,
and the next scroll queues a new job.
Every other tile change builds on the window thread as before:
edits,
reloads,
selections,
find,
theme,
scale,
annotation changes,
keyboard and pointer moves,
reveal or follow,
and a new wrap width within the allowance.

`overscan_tile_tests` counts renders,
painted tiles,
shaped rows,
and images on the window thread.
All four stay at zero while the view moves inside the tile and during a scroll re-center,
and the worker's own counts rise for each re-center.
A swapped-in tile matches a fresh render at the same offset,
and a result built before an edit is discarded.

### Soft wrap

The source view always wraps long lines to the width of its text area.
There is no setting and no horizontal scrolling:
the content is exactly as wide as the view,
the `Flickable` cannot move sideways,
a Shift+wheel turn scrolls nothing,
and no scroll range,
horizontal tile,
or caret-room widening exists.
Every open detail follows IntelliJ IDEA Community
(`doc/planning/slint-ide-soft-wrap.md` names the source file of each rule).

- The wrap width is the text area's width,
  which is the view minus the gutter,
  minus 2 px of caret room (`CARET_ROOM` in `src/native/viewport.rs`),
  so a caret after the last glyph of a full row stays inside the view.
  The minimap at the right edge of the view (see Minimap) is taken from the width as well,
  and the source view draws no scroll bar,
  so that is the only term for a scroll control.
  There is no minimum.
  A width of zero or less means the view has no layout yet:
  wrapping waits and the last width is kept.
  Until the first width arrives the view is not wrapped.
- Where a row breaks,
  and the continuation indent,
  are IntelliJ's rules (`src/wrap.rs`,
  `src/wrap_tests.rs`,
  `tests/wrap.rs`):
  a space or tab,
  or after `,` `;` `)` or before `(` and `.`,
  with the rightmost candidate winning,
  every code point from U+2F00 up is a break opportunity,
  and a token without a candidate breaks where the width is exceeded.
  A continuation row starts at the x of the line's first non-blank character plus one space for the mark.
- The gutter has one entry for each visual row.
  The first row of a line shows its number and,
  where a diagnostic starts,
  its severity letter.
  A continuation row shows the wrap indicator instead,
  in `c0c0c0` in the light and the dark scheme,
  the color IntelliJ gives `SOFT_WRAP_SIGN_COLOR` (the Darcula scheme inherits it).
  Shape and color are the two channels that tell it from a line number.
  The indicator is IntelliJ's own mark for a row that starts after a soft wrap:
  the second symbol of the first pair among U+2926 with U+2925,
  U+21B2 with U+21B3,
  U+2936 with U+2937,
  and U+21A9 with U+21AA whose two symbols the source font and its fallbacks display,
  and an arrow drawn by hand when no pair is displayable.
  The check shapes the symbols with the same shaper that sets the source text and looks for the missing-glyph box
  (`TextShaper::can_display` and `choose_sign` in `src/wrap_sign.rs`),
  and the image the gutter shows is set in that shaper and drawn by the same renderer,
  so the check is about what is drawn.
  Which pair a machine gets depends on its fonts;
  the choice is logged at debug level.
- A click picks the visual row under the pointer.
  A click right of the last character of a wrapped row puts the caret at the end of that row (upstream),
  and a click on the lower row's first character puts it at the start of the lower row (downstream).
  Hover and Ctrl+click resolve the visual row too.
  Selection,
  find matches,
  and underlines are painted per visual row.
- Home,
  End,
  Up,
  Down,
  PageUp,
  and PageDown move by visual rows
  (see [Source view keys](#source-view-keys)),
  and following the caret scrolls its visual row into view.
- A blank that overflows the last row of a line hangs on that row,
  as IntelliJ keeps it.
  Parley then adds one more line with no characters after it;
  `ShapedRow::visual_rows` does not count that line,
  so the gutter,
  the vertical map,
  and the keys never see an empty
  row,
  and a caret at the end of the text stands at the end of the last real row.

#### Measuring every line without waiting

The vertical map needs the height of every line,
but only the lines of the tile are laid out on the window thread.
The tile worker therefore also measures the whole text at the wrap width (`src/native/tile_pass.rs`),
in chunks of 128 lines from the view outward,
between the tile jobs it is given,
never before one:
a waiting tile job always goes first.
It keeps the laid-out lines in its own row cache (`KEPT_LAYOUTS` in `src/native/tile_build/worker_thread.rs`,
the 50,000 lines nearest the view;
measured on 2026-10-11 as about 158 MB,
3.2 KB a line).
A line that is not yet measured counts as one row,
or as its last known height after a resize.
What the worker has measured waits in a coalescing slot;
the window thread takes at most `TAKE_LIMIT` (4,096) lines per poll,
puts them into its table of visual rows,
and renders only when a count changed.
It never touches the lines of the displayed tile,
so a measurement cannot move the picture on screen.

The vertical map follows the table.
Rows that change height above the first visible row move the scroll offset with them,
so no visible pixel moves (`rows::settle`,
the keep-view-still machinery that annotation rows already use),
and the scroll extent settles as chunks arrive.
While the reader scrolls,
a change above the view waits until scrolling has stopped,
as before.
A finished tile is the exception:
its own lines have the heights it was painted with,
so it is swapped in at once,
moving the offset if it must,
which cuts a running wheel animation short.
A tile built on the worker measures its own lines first and returns their heights;
the window thread merges them into its table,
brings the map up to date,
and compares only the tile's own lines.
Anything else that moved is handled by `ShapedView::rebase`.
A tile built on the window thread (an edit,
a selection,
a jump) measures its lines first in the same way.

A new wrap width,
from a window or sidebar resize,
makes the worker break its kept rows again
(`TextShaper::rebreak`:
about a microsecond a line,
against about fifteen to shape one)
instead of shaping them.
A line whose tabs are not all on its first visual row is shaped again,
because a tab widened to its stop
depends on where the row broke.
The worker does this for the whole file after the window thread has taken the lines near the view
(see [A new width on the window thread](#a-new-width-on-the-window-thread)).
Measured on 2026-10-11,
breaking 50,000 kept layouts again took 40 to 80 ms.

Tab correction on a wrapped line repeats shaping until the corrections agree (`layout_wrapped_with_tabs`).
Row `n` of a line is right after `n + 1` passes,
because a row depends only on the rows before it,
so a line with tabs inside it is exact up to as many rows as it gets passes.
The first bound,
four passes,
was not enough:
`tab_correction_settles_at_every_width` swept widths from 40 to 700 px at two scales over two tab-heavy lines
and found tabs off their stops at 40 px.
A tab that ends its row can make two corrections repeat forever;
the loop then ends with the layout whose corrections are smaller,
the one in which that tab fits.

Whether the correction can settle at a bounded cost was measured on 2026-10-12 in the release profile,
on a host with a load of 40 to 90,
with the bound lifted:

- A line with a tab every few characters settled after 76 passes at 20,000 and 40,000 characters
  and after 74 at 100,000 characters (255 to 1,265 rows).
- A line of irregular words,
  blanks,
  and tabs settled after 4,
  7,
  and 54 passes
  at 5,000,
  20,000,
  and 100,000 characters.
- The count never came near the number of rows,
  but it is not known beforehand and it passed the former bound of 40.
- Every pass shapes the whole line.
  With 40 passes the 100,000 character lines took 2.3 s and 4.8 s on the window thread,
  where a first tile is built;
  the 20,000 character line of the first kind took 0.43 s.

A correction that settles at a bounded cost therefore cannot be promised,
and the bound stays,
now as a budget of characters shaped rather than a count of passes (`passes_for`,
`TAB_BUDGET`):
a line gets 40 passes (`TAB_PASSES`) up to 25,000 characters,
a longer line gets the budget of 1,000,000 characters divided by its length,
and never fewer than 4 (`TAB_MIN_PASSES`).
A line of 100,000 characters gets 10 passes;
after the budget the same lines took about 0.4 to 0.6 s
(shaping one such line without tabs takes 50 to 70 ms).
A line that wraps over more rows than it gets passes keeps the corrections of its last pass:
bounded and deterministic,
but its tabs on the later rows may stand off their stops.
That is the one place where the wrap trades exactness for a bound on the window thread's work.
`row_of`,
which finds the row of a tab,
is a binary search,
and the shaper counts its passes (`TextShaper::tab_passes`).

#### Lines taller than one raster

A tile is a stack of textures one below the other (`TileStack`,
`src/tile_scene.rs`),
and a window shows them as one picture:
the markup draws one `Image` for each texture at the tile's top plus the rows above it (`source-slices`),
so the textures abut on whole physical rows with no gap and no overlap.
A texture holds at most the device's texture side (8192 px unless the device says more)
and at most 256 bins of 256 by 256 pixels,
which is what Vello's coarse stage counts before it logs `Trying to paint too large image`
and is also exactly 64 MiB (`MAX_RASTER_BYTES`).
The raster is as wide as the text area plus 24 px,
so a texture's height is `256 * floor(256 / ceil(width / 256))` rows (`tile_texture_rows`):
16,384 rows at a width of 806 px,
10,752 at 1,358 px (the old byte rule allowed 12,288,
which is 288 bins and
made Vello log its warning at every render at scale 2),
2,048 at 7,259 px.
A tile that one texture holds is one texture,
so an ordinary window draws exactly what it drew before the stack.
A tile taller than a texture is cut at whole rows from its top into textures of `tile_texture_rows` and a rest.
Each texture has its own scene (`tile_items_in`):
the rows,
hints,
messages and underlines that can reach it,
placed from its first row by a whole number of pixels,
so a glyph that two textures share is drawn in each at the same fractional position,
and a clip band is cut to the texture (`encode`),
so a selected row at a seam is cut where the textures meet.
The stack holds at most four textures or the device's side plus 1,024 rows,
whichever is more (`stack_rows`),
so the view of the tallest window the device can show (never taller than its texture side) is always held.
A window is never larger than the device's texture side in either direction,
in physical pixels
(`max-width` and `max-height` of `AppWindow`,
set from the device's side):
the toolkit's surface configuration would panic past it.
`render::tile_cap` (`src/native/render/paint.rs`) is the largest tile height in logical pixels:
the larger of what one texture holds and three views plus six code rows,
within the stack's bound.
A tile of lines taller than that is clipped instead of refused (`Viewport::clip`,
`viewport::band_clip`):

- The clip is the three view heights around the view,
  cut to the cap,
  with its edges on whole code rows,
  as two distances below the top of the first materialized line's block
  (so a row that appears above the tile moves the clip with it,
  `ShapedView::rebase`).
- The tile's identity (`TileBounds`,
  the frame stamp) carries the clip,
  the view must stay a quarter of a view height clear of its edges (`viewport::holds`),
  and leaving them re-centers the tile on the worker as for any tile.
  Scrolling inside a clip does no work,
  as inside any built tile.
- The frame already skips the visual rows of a layout that lie outside it (`src/shaped_text/frame.rs`),
  so a line of thousands of rows costs only the rows in the clip.
  The gutter lists only the rows in the clip (`rows::measure`).
- A reveal (a search result,
  a jump target) measures the visual rows of its own line first (`viewport::reveal`),
  so a match on a late row of a wrapped last line that the background measurement has not reached is not cut off by
  the extent the map has for it;
  `native_find_paints_visible_matches_only_and_reveals_far_rows` failed 3 times in 16 runs
  on the tree before this slice for that reason.
- A view so tall that it and a quarter of its height on each side pass the cap can never be held,
  so such a tile is placed again at every scroll step;
  a window that tall has no better answer.
  The raster of a view the cap can hold is centered on the middle of the three view heights asked for,
  moved inside the lines,
  so the first and last lines of the text do not move it off the view.
- The window chooses the clip from the heights its map holds,
  and a line the background measurement has not reached counts as one row there,
  while the shaper sizes the raster from the rows it just shaped.
  `TextShaper::prepare_rows` therefore cuts every frame to the limit itself (`shaped_text::fit_clip`,
  `src/shaped_text/limit.rs`),
  around the middle of the lines it was given or of the clip it was asked for,
  and logs the cut at warn level.
  The built tile carries the clip it was really painted with (`TileBuilt::bounds`,
  and its stamp),
  while `TileBuilt::requested` names the job it answers,
  so the window shows the cut tile and re-places the view against the heights the tile measured.
  The window's map takes those heights in `tile::apply`,
  also when it discards the tile because the view left it,
  and the offset the map moved the view to is presented to the window on that path too;
  the next placement chooses its clip from the new heights.
  A raster ends with the text when the window reaches its last line,
  so the rounding of the top edge to a whole code row never costs the last row.
  A view within two code rows of the cap cannot be held by a raster on whole code rows,
  and is placed again at every scroll step like a taller one.
- The raster still refuses a tile past the limit with its diagnostic,
  which only a raster whose single code row is already past the limit can reach
  (a text area about 700,000 pixels wide at scale 1).
  A refused tile never blanks the source:
  the window thread keeps the picture last painted and names the failure,
  and the worker's failed job is freed so the next placement asks for the tile again.

The caret stops of a line were found in time that grew with the square of its length:
`project_line` walked the graphemes through a flat string view,
and every lookup scanned the string.
A line of 100,000 characters took longer than 90 s to open in a debug build.
Measured on 2026-10-12 in a debug build on a loaded host,
the projection alone took 0.37 s at 10,000 characters,
1.2 s at 20,000,
and 5.1 s at 40,000,
a factor of four for every doubling,
while shaping and breaking the same lines took 29,
65,
and 123 ms.
A line past 1 KiB (`FLAT_LINE_BYTES`) is now copied into a rope first,
whose chunks bound each lookup;
the projection of 100,000 characters took 0.5 s in the same build,
and in the release profile 13 to 20 ms
(the former walk took 8 ms at 20,000 and 190 ms at 100,000 characters there,
measured in isolation).
`text_projection_tests` counts the bytes the lookups scan and compares the stops with a flat walk.
Shaping and breaking a line of 100,000 characters without tabs takes 50 to 70 ms in the release profile,
about 1.2 s in a debug build,
and the window thread does it once,
when the tile that holds the line is first built.

#### A new width on the window thread

A resize must not show one frame of text wrapped for the old width.
Slint runs the change handlers before the frame at the new size is drawn:
inside the draw when the windowing system lets the redraw run at once,
and in the event loop pass that follows the size event when it holds the redraw back
(in the nested compositor's 16 ms drag the view reported its new width 0.4 ms after the command,
about 9 ms before the redraw request).
So the viewport-changed callback of a new window or sidebar size runs before the frame at that size is drawn,
and the window thread breaks the lines near the view again and paints the tile for the new width before that frame
(`src/native/rewrap.rs`,
`render::render`).
The width is the layout width of the `Flickable` that `ui/app.slint` reports through `changed width`
(`viewport-changed`,
`bind_viewport`,
`place_scrolled`),
and at every one of 715 recorded draws of five sessions the width the view had last reported was the window width
the toolkit drew minus 333 px (sidebar and gutter),
so the wrap width never arrived a frame late.

- `place_scrolled` marks `State::rewrap` when the wrap width differs from the width of the displayed tile.
  The next render asks `rewrap::begin` for an `Allowance`,
  which holds at most `REWRAP_LINES` (384) lines
  and `REWRAP_WORK` (24,000) units of work.
- A unit of work is a character shaped or broken,
  as `ide_app::shaped_text::line_work` counts it:
  `LINE_WORK` (32) for each line,
  plus its length for each shaping.
  A line with a tab counts one shaping and one more for each corrective pass of its tabs (`passes_for`),
  and a line over `WORK_SCAN_LIMIT` (65,536) characters counts as more than any allowance.
  A line the row cache already holds at the new width costs nothing.
  Every other line is charged before it is shaped or broken again (`TextShaper::measure_lines_within` and `Budget`),
  so a line that does not fit stops the step before any work is done on it.
- Within the allowance the window thread measures the lines of the requested tile at the new width:
  kept rows are broken again without shaping (`TextShaper::rebreak`),
  and the others are shaped.
  Their heights go into the table,
  `rows::settle` brings the map up to date with the view kept still,
  and the tile is painted and shown in the same render.
  Selection,
  find rectangles,
  the caret,
  the gutter rows,
  and the annotation blocks come from the same render,
  so they agree with the new width in the first frame.
- Where the bound would be exceeded,
  the change goes to the tile worker as before and the displayed tile stays on
  screen until the worker's tile arrives (`rewrap::defer`,
  which logs `left a new width to the tile worker`).
  That happens when the requested tile holds more than `REWRAP_LINES` lines
  (a window about 3,300 px tall over lines of one row each),
  when its lines need more than `REWRAP_WORK` units (two lines of 12,000 characters,
  one line of 200,000 characters,
  or wrapped lines with tabs),
  and when its raster would pass `REWRAP_RASTER_BYTES` (32 MiB,
  half of the limit of a
  tile the worker builds).
- The worker's whole-file measurement for the new width runs after the step as it did before,
  and ends at the heights of a fresh shaping.
  It never touches the lines of the displayed tile,
  so a tile that arrives later from the worker asks for the
  measurement again (`pass_posted` is cleared in `tile::apply`) and the lines only the old tile held are measured then.

`wrap_resize_tests` (debug counters,
no wall time) asserts the structure.
After one change of width the displayed tile holds the new width and has no row wider than the text area
except a row of one token that has no break opportunity,
and the gutter,
selection,
find rectangles,
caret,
and annotation blocks agree with a fresh shaping at that width.
The step is bounded in lines shaped or broken,
in work charged,
and in rasters painted for 2,000 lines of 600
characters,
one line of 200,000 characters,
a view taller than the clip cap (a window of `tile_cap` plus 400 px
at a text width of 3,600 px),
a window about 3,300 px tall,
and a file of 50,000 lines.
A change over any bound is left to the worker with nothing shaped on the window thread beyond the first refused line.
The guards `resize-on-window-thread`,
`drawn-frame-after-resize`,
`rewrap-lines-bound`,
`rewrap-work-budget`,
`rewrap-tab-passes`,
`rewrap-raster-bound`,
`rewrap-view-past-cap`,
`pass-again-after-new-width`,
and (annotations) `deferred-width-stays-on-worker` and `discarded-tile-asked-again`
remove each rule in a disposable copy and must make the named test fail.

Measured on 2026-10-13 with the release bundle (sha1 `2d8e01e9`,
built from commit `f851815582`;
the later commits
change only tests,
the guard script,
and a subtraction in `Allowance::lines` and `Allowance::work` that differs
only on overflow) in the nested compositor 0.4.0,
at 1100 by 660 pixels,
the frame recorder at 60 frames a second and synthetic input through the control socket.
The drag goes from 1100 to 600 pixels and back in steps of 25 pixels,
16 ms apart,
in two sweeps.
Six sessions of each of the two files,
none discarded (the recorder dropped nothing),
at a host load of 12 to 20.3
(one-minute average at the start of each session);
the 20 to 100 band was reached by 2 of 12 sessions,
at 20.3.
Run-to-run spread comes first,
because it is as large as most differences:
the longest gap between painted frames of one build and file varied from 17 to 83 ms (code50k,
median 33.3)
and from 17 to 43 ms (long2000,
median 25.7).
The earlier build (`final`,
the worker-only tile) varied from 17.7 to 50 ms and from 17.3 to 60 ms in the same protocol.
By load band:

- Below 20:
  `final` had 3 sessions per file,
  longest gap median 33.3 ms (17.7 to 33.3) on code50k and 17.3 ms
  (17.3 to 43.7) on long2000;
  the new build had 5 sessions per file,
  median 33.3 ms (17 to 83) and 33.3 ms (17.7 to 42.7).
- 20 to 100:
  `final` had 2 sessions per file (37.3 and 50 ms;
  33.3 and 59.7 ms);
  the new build had 1 session per file
  (35.7 ms and 17 ms).
  Too few sessions to compare in this band.
- No frame was blank or flat in any session,
  the last frame fit the width in every one,
  and the longest gap is no worse than that of the worker-only tile within the spread.
  No claim is made here that either build is smoother.
- Frames whose text does not fit the width (the detector of the earlier report:
  ink in the last two columns,
  ink
  stopping 120 px short of the edge,
  or a flat or empty frame):
  64 to 79 of about 100 frames per session on code50k and
  72 to 78 on long2000,
  against 72 to 81 and 73 to 79 for `final`.
  The longest stretch of such frames was 635 to 1,319 ms (new) against 384 to 1,371 ms (`final`).
  This measure did not change.
- The step itself,
  from the allowance to the last setter of the frame (log line
  `window thread re-wrapped for a new width`;
  42 to 79 steps per session,
  none left to the worker,
  no worker tile applied),
  in the five sessions of each file other than round 5:
  on code50k the median was 3.2 to 4.9 ms,
  the 95th percentile 8.0 to 20.5 ms,
  and the largest 16.9 to 60.0 ms;
  on long2000 the median was 6.0 to 6.9 ms,
  the 95th percentile 10.7 to 12.7 ms,
  and the largest 18.3 to 29.0 ms.
  The first step of a session took 8.1 to 21.2 ms (code50k) and 13.1 to 17.4 ms (long2000).
  The most one step charged was 70 lines and 5,702 units (code50k) and 13 lines and 8,229 units (long2000).
  One session of each file (round 5) ran with a slower window thread,
  45 and 42 steps,
  median 9.2 and 15.4 ms,
  largest 64 and 91 ms,
  recorder at 52.5 and 57.3 frames a second,
  while the load average read 14 to 16;
  the cause was not found.
  A step takes a few milliseconds typically and up to a few frame periods at the largest,
  so it can still make one frame late.

A second series on 2026-10-13 ran after the host's CPU was replaced (`nproc` reports 30;
one-minute load 5.7 to 10.8
at the start of every session,
so every session is in the band below 20) with the bundle of the final tree
(sha1 `9c60b963`,
from tree `1a7214f4f9`) and,
in the same series and order,
the earlier `final` bundle:
six sessions of each build and file in the 16 ms drag,
none excluded (the recorder dropped nothing).
Run-to-run spread is small here:
the longest gap was 17.1 to 17.7 ms in 11 of 12 `final` sessions (33.3 ms once)
and 17.0 to 18.0 ms in all 12 sessions of the new build.
Frames that do not fit were 76 to 82 per session for `final` and 74 to 77 for the new build;
the longest stretch of them was 650 to 767 ms for `final` on code50k and 650 to 1,302 ms on long2000,
and 633 to 650 ms
for the new build on both files;
the picture settled after the last resize step in 24.0 to 26.7 ms for the new build
against 23.2 to 109.1 ms for `final`.
No frame was blank or flat,
and the last frame fitted in every session.
The window-thread step took a median of 3.0 to 3.3 ms on code50k (largest 5.7 to 10.7 ms,
first step 5.5 to 10.7 ms)
and 5.9 to 6.8 ms on long2000 (largest 13.5 to 15.3 ms);
every step of every session was taken at once and none
was left to the worker.
In the 100 ms drag of this series (3 sessions each) frames that do not fit were 44,
42,
and 50 of about 265 for `final`
and 0,
8,
and 19 for the new build,
and the longest stretch of such frames was 33 to 50 ms against 0 to 17 ms.

The unchanged count of frames that do not fit has a cause that was looked at in the frames themselves.
In the 16 ms drag,
kept frames of one session show the window at the new size with the previous step's content:
at 1075 pixels the text is wrapped for 1100 and cut by the edge,
and on the way back a strip of another shade stands
at the right edge beside text wrapped for the previous,
narrower size.
These are the detector's `clipped` and `band` frames (38 and 39 in that session,
two sweeps of about 20 steps).
The strip of another shade stands where no tile of the application is,
so those frames were composed at the new size
from a buffer drawn for the previous one.
The `clipped` frames could come from either side,
so the first draw after a size event was checked in the headless
backend:
`a_frame_drawn_right_after_a_resize_event_is_wrapped_to_the_new_width` reports the new size,
takes a snapshot
at once with no timer or handler run between them (the draw runs the change handlers),
and finds no row of ink at the
right edge;
with the window-thread step turned off,
198 rows of that frame run to the edge.
So in the headless backend the first frame is already wrapped for the new width.
The compositor's recorded frames still show the previous step's content at each new size in the 16 ms drag,
and that delay was measured on 2026-10-13 (`doc/troubleshooting/nested-wayland-recording-previous-resize-step.md`):
it is the recorder's capture model and winit's redraw gate,
not the application.
The compositor changes the output mode,
which is the recorded frame size,
when the `resize` command arrives,
captures before it sends the frame callbacks,
and sends them only from the recorder tick,
and winit withholds the redraw of a Wayland window until the callback of its last swap arrives.
A size that reaches the application between two ticks is therefore drawn right after the next capture
and first captured one tick later,
on a canvas that already has the newest size.
In three 16 ms sessions 76 to 77 of 81 to 82 captures held a buffer of another width than the output mode,
all 77 recorded frames that did not fit in the first session were among them,
and no frame whose buffer had the
output's width failed to fit (7 of 7 in that session,
459 of 459 in two 100 ms sessions).
The debug lines at the target `monochromatic_ide::native::frame` (`src/native/frame_trace.rs`) place the winit size
event,
the redraw request,
the view report,
and the start and end of each draw on the log's clock.
Opening the gate at the command in a copy of the compositor cut the stale captures from 94 to 56 to 61 percent
and the commit from 13 to 8 ms after the command;
the rest is the re-wrap (4 ms) and the draw (3.5 ms) against a
16.7 ms tick.
So the requirement that no frame shows old-width text is met in the headless draw,
and in the compositor every frame
whose buffer has the output's width is wrapped for it.
A slower drag (100 ms between steps,
one sweep,
3 sessions each,
load 15 to 23) separates the two builds:
frames that do not fit were 14,
29,
and 46 of about 260 for the new build against 94,
47,
and 97 of about 250 for
`final`,
and the longest stretch was 33 to 50 ms against 200 to 319 ms.
The new build therefore leaves at most about three recorded frames of old-width content after a step and `final`
several more,
and the remaining ones are the capture delay above,
not wrapping by the application.

#### What the window thread pays

Measured on 2026-10-12 with the release bundle (`dist/monochromatic-ide`,
sha1 `13505a6f`,
built from commit
`9151379037`) in the nested compositor at 1100 by 660 pixels,
with the debug lines of `tile.rs` and `render.rs` timing
the window-thread functions.
The host's load was 37 to 43 throughout,
so the numbers are not those of an idle machine.
The later commit `ff16f9a3d9` changes only what the window does after it discards a stale tile (it renders again),
which none of these sessions did,
and the numbers were not measured again.
Only sessions in which the frame recorder dropped nothing count;
all 10 sessions were counted and none was discarded.
Each session opened the file,
waited for the pass to finish,
scrolled 60 wheel notches down and 60 up with a 16 ms wait
after each notch and the recorder running,
and resized the window to 900 and back to 1100 pixels.
The times are the time inside the window-thread functions,
never the time to the next frame:
the toolkit's image upload happens after them and is not in these numbers,
and the longest gap between painted frames was not measured.

A file of 50,000 lines of about 110 characters,
most of them wrapped to two rows (6 sessions):

- Taking in a finished tile (`tile::apply`,
  timed in `tile::poll`):
  while opening,
  19 tiles,
  median 0.14 ms,
  95th percentile 0.38 ms,
  largest 0.38 ms;
  while scrolling,
  123 re-centers,
  median 0.26 ms,
  95th percentile 1.0 ms,
  largest 16 ms;
  after the resizes,
  12 tiles,
  median 0.44 ms,
  95th percentile 0.54 ms,
  largest 0.54 ms.
- Taking in the output of the measurement pass (`tile::pass::apply_pass`,
  timed in `tile::poll`):
  while opening,
  1,649 outputs,
  median 0.07 ms,
  95th percentile 0.39 ms,
  largest 4.2 ms;
  after the resizes,
  172 outputs,
  median 0.11 ms,
  95th percentile 0.28 ms,
  largest 1.2 ms.
  The pass was done during the opening of each session;
  the scroll window held no output.
- Building a tile on the window thread (measuring its lines and painting it):
  18 builds while opening,
  median 1.6 ms,
  largest 23 ms (the first tile and the two that follow it);
  none while scrolling or resizing.

A file of one line of 200,000 characters (4 sessions):
the first build of a tile on the window thread took 164 to 228 ms in every session,
which is the time to shape that line once,
a second build took 10 to 13 ms,
and a third took 167 to 174 ms in two sessions and 4 to 6 ms in the others;
taking in a re-centered tile took a median of 0.11 ms,
a 95th percentile of 2.7 ms,
and at most 7.2 ms (88 tiles).
Shaping and breaking 100,000 characters without tabs takes 50 to 70 ms in the same profile when timed alone.
The window thread therefore still waits once,
about the time of one shaping of the line,
when such a file is opened,
and once more in two of the four sessions;
no scroll step entered a build,
which is what these counts show,
and not that no frame was late.

Structural tests,
independent of host load,
count what the window thread does.
`wrap_tests` opens 50,000 lines and asserts that the window thread shaped almost nothing
and that one poll applied at most `TAKE_LIMIT` lines.
A re-center during the pass is built on the worker with no row shaped on the window thread,
and it is not discarded.
A resize that the window thread takes shapes nothing again and re-breaks only the lines of its allowance.
A mixed file proves that no visible pixel moves when rows above the view change height.
`wrap_clip_tests` scrolls inside a built tile of a wrapped file and inside the clip of one tall line:
no render,
no shaped row,
no painted tile,
and no image on the window thread (guard `wrapped-tile-holds`);
it walks the view through a 100,000 character line and asserts the raster stays within the limit and holds the view,
that the gutter lists only the rows of the clip,
and that opening the file shapes each of its lines once.
`wrap_annotation_tests` shows that a resize with hints and messages on a wrapped line paints one tile on the window
thread within the allowance and packs only the hinted lines near the view,
and that a width the allowance does not cover paints none there.
`text_projection_tests` and `tab_layout_tests` count the work of the caret stops and of the tab passes.

#### Known differences from IntelliJ

Everything the user did not decide follows IntelliJ IDEA Community (`doc/planning/slint-ide-soft-wrap.md`).
These are the places where the source view differs,
and why:

- The wrap mark stands in the gutter,
  on every continuation row,
  because the user said so;
  IntelliJ paints it in the text area and,
  by default,
  only on the caret's line.
- A block of hints and messages stands over the first visual row of its line.
  IntelliJ puts a block inlay over the visual row that holds its offset (`VisualLinesIterator.setInlays`),
  and a block below a line under the visual row that holds it.
  The user decided that virtual rows stand over the code line (2026-10-05);
  no block stands under a line here.
- Hints and messages are broken at the width of the code.
  The IntelliJ source read for this work does not wrap a block inlay and draws no diagnostic message in the text.
- The caret cannot stand in the indent of a continuation row or in virtual space,
  so Home has one step fewer on a continuation row and on a line of blanks only.
- IntelliJ gives its identifier-symbol break rule a lower weight than the others.
  Parley keeps the rightmost opportunity whatever its weight,
  so the rule counts like the others.
- A line that wraps over more rows than its tab-correction passes keep their last corrections
  (see [Soft wrap](#soft-wrap));
  IntelliJ measures each tab on the row it stands on.
- A tile of a very tall line holds only the rows near the view;
  IntelliJ paints the rows in the view.
- Lines the background measurement has not reached count as one row,
  so the scroll extent grows while it runs.
- There is no switch for soft wrap,
  because the user asked for it to be always on.
- Thai,
  Lao,
  Khmer,
  and Myanmar get no dictionary-based breaks (Parley's `complex-scripts` feature is off).

### Stability

Rows arrive after the text is shown,
hints and diagnostics separately,
and they take space,
so the rules for what may move are part of the placement.

- Rows for lines before the first visible line move no visible pixel.
  The view keeps the distance between its scroll offset and the code row of its first line (`rows::settle`),
  and a view at the very top stays there.
- Rows for a visible line move only the lines after it,
  by the height of the block:
  26 px for one row and 16 px for each further row.
- While the reader scrolls,
  a change that would move the scroll offset waits until the offset has been still for 200 ms
  (`rows::quiet::SCROLL_QUIET`),
  so a wheel animation is never cut short.
  The rule reads the time through `rows::quiet_now`,
  which a test can hold still and move by exactly the time it means
  (`rows_above_the_view_wait_until_scrolling_has_stopped` does,
  so no host load can run the 200 ms out between a
  scroll and the rows that must wait for it;
  with the former real sleeps it failed whenever a stall passed 200 ms).
  A render that comes after a scroll step whose change handler has not run yet
  (the window's offset differs from the one last placed) counts as scrolling too;
  under host load a recording showed such a render moving the first visible line by 3 px before this rule.
  Underlines are drawn at once;
  they take no space.
- After an external change the rows of the replaced text are not painted,
  but their space is held over every line the change left in place (`rows::hold`).
  Hint space is given up when hints for those lines arrive.
  Message space is kept for 1 s (`rows::held::ROW_HOLD`) whatever arrives,
  because diagnostics can come in several answers,
  and then all held space is given up.
  A server that gives the same hints and messages within that time moves no line.
- A first open has nothing to hold:
  hints and diagnostics each move lines once when they arrive.
- The vertical scroll range is the sum of the visual rows and the blocks known so far.
  It grows as hints arrive for lines further on and as the background measurement reaches wrapped lines.
  The source view has no scroll bar;
  the minimap is its scroll control,
  and its box shrinks as the range grows.
- A caret line before the view is revealed together with its block.
  A search result or jump target is revealed from the top of its block.

Measured on frames recorded from the nested compositor
(commands and frame names are in `design/README.md`,
entry "2026-10-05 hint rows"):

- Injected hints for the first lines of the scene moved line 2 by 26 px,
  line 3 by 68 px,
  line 4 by 158 px,
  lines 5 to 13 by 184 px,
  and lines 14 to 20 by 210 px,
  in one frame;
  line 1 did not move.
  Diagnostics injected 3 s later moved lines 5 and 6 by 16 px,
  line 7 by 42 px,
  and line 8 by 164 px;
  lines 1 to 4 did not move.
- An external change with annotations returning 0.3 s and 0.7 s later changed three frames
  and moved no code row.
- With diagnostics returning 3 s later,
  message space was given up 1.01 s after the reload (lines moved up by 16,
  42,
  and 164 px)
  and taken again when the diagnostics arrived,
  3.16 s after it.
- With a TypeScript 7.0.2 server,
  hints and diagnostics arrived within 1 ms of each other at the first open
  and moved lines once;
  after an external change the rows were back within 0.04 s and no code row moved.
- 42 wheel notches sent while hints and diagnostics arrived
  gave 30 kept frames in which every code row found again moved by one common amount,
  never backwards;
  the rows appeared in one frame 0.32 s after the last render that deferred them,
  with the first visible lines unmoved and the lines after them moved by 26 to 256 px.
- Times come from the application's log:
  host load from other sessions dropped up to two thirds of the recorded frames.

The native tests `rows_arriving_in_view_move_only_lines_beneath_them`,
`rows_arriving_above_the_view_move_no_visible_pixel`,
`rows_above_the_view_wait_until_scrolling_has_stopped`,
`a_reload_holds_row_space_until_annotations_return`,
and `scrolling_across_annotated_lines_is_rigid` in `src/native/annotation_stability_tests.rs`
pin on rendered pixels which lines may move and by how much.

### Diagnostic marks

Each diagnostic is underlined under the characters it marks,
using the same range geometry as selection,
so tabs,
CJK,
combining marks,
and ligature halves are covered exactly.
Severity has three visible channels besides color:
the underline style,
the severity word that starts each message row,
and a letter in the gutter.

- Error:
  a wavy line and `E`.
- Warning:
  a dashed line and `W`.
- Information:
  a dotted line and `I`.
- Hint:
  sparse dots and `H`.

#### Gutter letters

The user decided on 2026-10-06 that severity is a plain letter in front of the line number,
not an icon or a boxed marker,
and then that the letter stands close to the number,
not a long space away.
A line where diagnostics start shows the letter of the worst of them in that severity's ink:
JetBrains Mono at the line numbers' 15 px,
bold,
in a cell one letter wide that ends 4 px before that line's own number,
whatever the number's digit count.
Measured on rendered frames
(`the_letter_stands_the_same_gap_before_one_two_and_three_digit_numbers`),
6 blank pixel columns lie between the letter's ink and the number's ink before `1`,
`10`,
and `100` alike
(the 4 px gap plus the side bearings of the two glyphs,
antialiased edges counted as ink),
where 1 blank column lies between the digits of `10`:
the letter reads as a mark before the number and not as one more digit,
and stands closer to it than one digit advance.

From its left edge the gutter holds 6 px,
the 9 px letter cell,
the 4 px gap,
the number column,
and the 12 px before the text,
unchanged.
The number column is as wide as the displayed file's widest line number,
with at least three digits,
which is Helix's default minimum
(`min_width: 3` in `helix-view/src/editor.rs`,
applied in `line_numbers_width` in `helix-view/src/gutter.rs`).
Both widths are measured from the font (`letter-probe` and `number-probe` in `ui/app.slint`)
and rounded to whole pixels:
9 px per digit and per letter,
so a file of fewer than 1000 lines gets a 58 px gutter at scale 1;
like every logical length it is multiplied by the display scale.
The letter's room is there on every line,
so neither the text nor a line number moves when diagnostics arrive or go.
Text moves right by one digit when the displayed file's line count gains a digit past 999,
as in Helix
(`the_gutter_gains_a_digit_past_999_lines_and_the_pointer_follows`).
The empty line after a final terminator has a number of its own and counts.
Every horizontal position in the source view,
hit testing,
find rectangles,
the caret,
selection,
underlines,
and the virtual rows,
is measured from the one `gutter-width` property of `ui/app.slint`,
so all of them follow the gutter;
native code is told the width of the view right of the gutter and never the gutter's width itself.

A range over several lines shows its letter on its first line only,
where its message rows stand.
The letter belongs to the diagnostics of the displayed text:
after an external change it goes with the message rows and returns with the new diagnostics.
`I` and `H` extend the user's `E` and `W` by the same rule;
that extension is open to the user's veto.
Native code hands the window one number per materialized line (`State::line_marks`,
set in `rows::window::present`),
taken from the first message row of the line's block,
which names the worst severity,
and the line count that sizes the number column.

Open to the user's veto:
the 4 px between letter and number,
the 6 px before the letter,
and the three-digit minimum.
Without the minimum,
text in a file of fewer than 10 lines would start 18 px further left than in a file of 100 lines or more,
and would move whenever the line count crosses 9 or 99.

A diagnostic without a severity is shown as a warning,
as Helix shows it.
A range over several lines underlines each of its rows and marks a crossed line end like a selected terminator,
so an empty line inside it stays visible.
A point range gets a mark one terminator wide:
after the text at a line end,
centered on its position elsewhere.
Where ranges overlap,
the mildest severity is drawn first and the worst on top.
Inside a selection an underline keeps its style but takes the selected-text ink,
as selected glyphs do:
the light-scheme severity inks reach only 1.16:1 to 1.39:1 against the selection fill `#0078D4`.

Severity inks have a light and a dark value each (`ui/annotation.slint`):
the WinUI system critical and caution fill colors for errors and warnings
(`SystemFillColorCritical` and `SystemFillColorCaution` in `microsoft/microsoft-ui-xaml`,
`controls/dev/CommonStyles/Common_themeresources_any.xaml`),
the accent pair of Slint's fluent style for information,
and a neutral gray for hints.
Measured on rendered frames (`rows_and_marks_render_in_both_schemes_with_measured_contrast`),
against the light background hint rows reach 8.36:1,
error rows 5.42:1 to 5.46:1,
warning rows 5.06:1,
information rows 6.06:1,
and hint-severity rows 5.93:1;
against the dark background 8.78:1,
8.39:1,
12.91:1,
9.47:1,
and 8.22:1.
Gutter letters,
measured the same way (`gutter_letters_show_the_worst_severity_in_front_of_the_line_number`),
reach 5.42:1 (`E`),
5.10:1 (`W`),
and 6.05:1 (`I`) against the light background,
and 8.39:1,
12.91:1,
and 9.47:1 against the dark one.

### Cost and invalidation

Accepting diagnostics groups their messages by line and wraps them once.
Accepting hints groups them by line;
a line's hints are shaped and packed the first time its block is assembled,
then kept until another answer covers the line or the display scale changes.
The vertical mapping is rebuilt only when the store,
the text,
or the scale changed (`rows::refresh`).
Each render takes the blocks and the diagnostics of the materialized rows with binary searches.
That visible part is a frame-stamp input:
a snapshot change outside the materialized rows repaints nothing,
installing the same snapshots again does nothing,
and rows that appear before the view only move the cached frame (`ShapedView::rebase`).
A repaint shapes one layout per visible hint and message row
and checks each visible diagnostic against each materialized row.
Gutter letters are read from the blocks of the materialized lines on every render,
and the window's list of them is changed only where a letter changed.

### Inspection

Debug builds started with `IDE_INSPECT_ANNOTATIONS` naming a JSON file show that file's hints and diagnostics
for the displayed text (`src/native/inspect.rs`),
for nested-compositor frames without a language server.
The file may give delays after which hints and diagnostics arrive,
separately,
and delays after which they arrive again following each external reload.
Such a window starts no language worker:
a worker without servers reports "no problems" for the displayed text,
which would replace the file's diagnostics.
Release builds do not contain this path,
and without the variable it does nothing.

`inspect:native dark annotations` and `inspect:native light annotations` write such a file for a fixture
whose first lines hold one case each:
several hints on one row,
hints that would touch,
a tab and CJK line with a hint and a message,
one message,
every severity on one line,
a long message,
a message with line breaks,
a range over several lines,
and hints together with messages;
further lines make the file long.
`IDE_NATIVE_ANNOTATION_DELAYS` and `IDE_NATIVE_ANNOTATION_RELOAD_DELAYS`,
each `HINTS,DIAGNOSTICS` in milliseconds,
set the delays,
and `IDE_NATIVE_SCALE` starts the nested output at another scale.
A parent compositor that cannot make a window of the scaled size keeps the nested window smaller;
the compositor's `scale` control command switches the scale of a running session without that limit.

### Differences from editord

editord draws the same rows over each line in Inter,
hints first and each diagnostic on a row of its own at its column.
Deliberate differences:

- Position:
  editord pads rows with spaces whose width it estimates from one measured ratio.
  Here a hint or message starts at the pixel x the shaper gives its character.
- Hint gap:
  editord lets a hint follow the previous one as soon as its column is reached,
  counted in characters.
  Here 8 px must stay free,
  measured in pixels.
- Hint labels:
  editord strips a type hint's leading `: ` and a parameter hint's trailing `:`.
  Here labels are shown as the server sent them.
- Message text:
  editord writes `error(rustc): message`.
  Here the row reads `Error E0308 (rustc): message`,
  with the code.
- Message ink:
  editord colors a line's whole block,
  hints included,
  in the worst severity's color,
  and shows severity by color alone.
  Here each message has its own severity's ink and word,
  hints keep the hint ink,
  underline styles differ,
  and the gutter shows the worst severity's letter in front of the line number.
- Order and caps:
  editord lists messages in the server's order and wraps them to the editor's width without a cap.
  Here they are listed worst first,
  wrap at 80 characters,
  and are capped per message and per line.
- Spacing:
  editord gives every row a line height of 1.5 and no extra gap.
  Here rows are 16 px with a 10 px gap before the block.
- Arrival:
  editord lets the page reflow when rows arrive.
  Here the view is held still for rows before it,
  waits for scrolling to stop,
  and holds row space across a reload.

`test:annotations` covers the vertical mapping,
the row rules,
the store,
layout,
painting,
the frame stamp,
and the measurements of inline placement kept in `tests/annotation_placement.rs`.
`test:annotations-native` drives injected snapshots through real key and pointer events in both schemes,
and `test:native` includes it.
`inspect:annotation-guards` removes each guard in a disposable copy and checks that its named test fails.

## Minimap

The source view has no scroll bar.
The minimap at the right edge of the view is its scroll control:
it shows the file as a picture and a box over it that marks what the view shows.
Pressing,
dragging,
and turning the wheel on it scroll the view,
and the keys that scroll the view work as before.
Every open detail follows CodeGlance Pro and,
where that is silent,
IntelliJ IDEA Community.
The sources read for this work are the `Nasller/CodeGlancePro` repository (the project that continues
`Vektah/CodeGlance`;
a repository named `Vektah/CodeGlancePro` does not exist) and the original `vektah/CodeGlance`.
Its defaults are in `core/.../CodeGlanceConfig.kt`;
the behavior is in `render/ScrollState.kt`,
`panel/scroll/ScrollBar.kt`,
`panel/GlancePanel.kt`,
`render/BaseMinimap.kt`,
and `render/MainMinimap.kt`.

### What it shows

- The picture is a zoomed-out reflection of the window's own rendering of the text:
  the window's glyphs,
  its syntax colors,
  the selection,
  the find matches and the active match's boundary,
  the diagnostics' underlines and the hints,
  the virtual rows above annotated lines,
  and the rows soft wrap makes.
  The picture is a Vello scene drawn on the GPU by the one renderer the code view uses (`src/native/gpu_tile.rs`):
  the shaped lines of the window (`TextShaper::prepare_rows`,
  the same shaping the tile uses),
  the match rectangles,
  the annotation layout,
  and the markup's fills are encoded as paint items
  (`src/minimap/scene.rs`,
  `src/minimap/shape.rs`,
  `src/minimap/fills.rs`),
  scaled to a sixth of the text by the scene's transform,
  and rendered into a texture that the markup shows as an image.
  Nothing is painted on the CPU,
  and nothing is averaged by hand:
  a pixel of the picture is the GPU's coverage of the glyph outlines it stands for,
  and the native test `src/native/minimap_reflection_tests.rs` compares the picture with the box average of a
  snapshot of the real window for plain text,
  a selection,
  find matches,
  wrapped lines,
  and diagnostics,
  at display scales one and two (the bounds are in Measured cost).
- The picture covers the whole width of the text area (the wrap width,
  or 1,300 px while the view has none) shrunk to the minimap's width,
  and the rows of the file in visual rows shrunk by a fixed scale of a sixth
  (`Geometry::scale`,
  so one code row of 24 px is four pixel rows,
  whatever the length of the file).
  The rows come from the one vertical mapping (`RowMap`),
  never from a second estimate of the height.
  Its pixels hold the straight-alpha color of the covered part of the pixel,
  so an average of ink and background is a fainter ink,
  as in the window's own frame;
  the caret and the line-number gutter are not part of the picture.
- A text that fits the view at that scale is drawn at that size and ends above the bottom of the view.
  A longer text keeps the scale,
  and its picture slides up past the track with the scroll:
  the picture's overhang past the track times the share of the scroll range the view has scrolled
  (`Geometry::slide`).
  This is CodeGlance Pro's "Proportional" mode;
  its "Fit" mode,
  which squeezes the whole file into the height of the view,
  is not used,
  because a file of fifty thousand lines then shares a pixel row among hundreds of code rows
  and the picture stops being the window's own rendering.
  The box is proportional too:
  its top is the share of the scroll range times the room it has in the track,
  which is exactly where the view's rows lie in the slid picture,
  and a press or a drag maps through the slide (`Geometry::content_y`,
  `Geometry::target`).
  The picture holds only the bands near the visible part of it:
  the bands the track shows and one more on each side (`Geometry::bands_for`),
  and `LEAD` more on the side a fling is heading toward (`Heading`,
  `Geometry::bands_toward`),
  so the work and the memory follow the view and not the length of the file,
  and a fling that moves the track by several bands in the time a picture takes
  does not run past the picture's end.
  Every line in the bands is drawn;
  nothing is sampled.
- The inks are the syntax classes and the palette of the source text in the dark and the light scheme
  (`CodeColors`,
  the same inks the tile uses),
  the selection's fill,
  and the foreground at 0.14 opacity with a boundary for the other matches.
  A change of scheme builds the picture again.
- The viewport box is the share of the text the view shows,
  never shorter than 20 px (CodeGlance Pro's
  `MIN_VIEWPORT_HEIGHT`),
  so a very long file keeps a box that can be grabbed.
  Its fill is the foreground ink at 0.15 opacity at rest,
  0.25 under the pointer,
  and 0.35 while it is dragged,
  as in
  CodeGlance Pro,
  and its border is 1 px at rest and 2 px under the pointer and while dragged.
  CodeGlance Pro's border thickness is zero;
  the border is added so that no state is told by one channel only.
  The pointer cursor is the third channel:
  a hand over the picture outside the box,
  where a press jumps,
  and the arrow
  over the box,
  while it is dragged,
  and below the end of a picture shorter than the view (CodeGlance Pro shows the hand
  only where `y` is above the end of the drawn picture).
  The fill and the border change only while the pointer is on the box (`isInRect` in `ScrollBar.updateAlpha`).
- The minimap is 110 px wide,
  or a sixth of the source column in a narrow window,
  down to 15 px
  (CodeGlance Pro uses a twelfth of the editor in a splitter,
  with the same minimum of 15 px).
  The text area,
  and so the wrap width,
  is the source column minus the gutter,
  the minimap,
  and the caret room.
  The source column's minimum is 288 px (the 240 px it had,
  plus the 48 px the minimap takes at that size),
  so the
  minimap
  never narrows the text below its old minimum.

### How it scrolls

- A press outside the box moves the caret to the row under the pointer and,
  in the same row,
  to the column under it
  (the position across the minimap is the same share of the text's width,
  `text_x` in `src/minimap/horizontal.rs`),
  and scrolls that row to the middle of the view,
  also near the ends of the file
  where the scroll range stops the view short of the middle.
  This is CodeGlance Pro's default (`jumpOnMouseDown = MOUSE_DOWN`,
  `clickType = CODE_POSITION`,
  `moveOnly = false`,
  `ScrollType.CENTER`).
  A press with Shift held keeps the anchor of the selection and extends it to that place.
  On a line too long to shape within a click's allowance the caret goes to the start of the line.
  A press in the virtual rows above a line jumps to that line's first row.
  The row under the pointer is the one of the picture as it has slid with the scroll when the pointer went down.
- A press on the box starts a drag without a jump.
  A press outside the box jumps and then continues as a drag of the box from where the jump left it
  (`ScrollBar.mousePressed`:
  `dragStart`,
  `dragStartDelta`).
  A drag moves the view by the pointer's distance over the room the box has,
  times the scroll range
  (`ScrollBar.mouseDragged`:
  `maxViewportStart / movableHeight`),
  so the box follows the pointer one to one;
  above and below the track the view stops at its ends.
  A drag never moves the caret.
- A wheel turn over the minimap scrolls the view exactly as over the text:
  the minimap is a child of the scrolling
  view,
  counter-translated so it stands still,
  and its touch area has no scroll handler,
  so the toolkit's own wheel
  physics get the event.
  CodeGlance Pro also hands the wheel to the editor (`MouseEventAdapter.redispatch`).
- The box and the slide of the picture follow the scroll live,
  from the scroll offset and the height of the text alone,
  in markup bindings (`minimap-box-top`,
  `minimap-slide` in `ui/app.slint`,
  which hold the same formulas as `Geometry`).
  The picture is placed at whole device pixels,
  so sliding it never resamples it.
  Native code notes the bands the track shows after each scroll step (a comparison of a few words)
  and asks the worker for a picture only when they change.
- The keys keep working:
  PageUp,
  PageDown,
  the arrows,
  Home,
  and End are unchanged,
  and a press on the minimap leaves
  the
  keyboard focus on the source view.
- A drag,
  a press,
  and a page step set the offset and paint the tile of the new place before the next frame,
  the way a
  caret
  key does,
  because a tile built by the worker for a place the drag has already left would arrive out of date and leave
  the text blank while the pointer moves over a long file.
- The minimap is a slider for assistive tools,
  labelled "Source minimap",
  with the offset as its value,
  the scroll
  range as
  its maximum,
  and a page up or down as its increment and decrement.
- The editor's own scroll bar:
  CodeGlance Pro keeps it by default (`hideOriginalScrollBar = false`).
  This view never had one,
  so the minimap is the only scroll control and nothing is hidden.

### Black surfaces in the dark scheme

- In the dark scheme the editor's background,
  the minimap's,
  the sidebar's (the file tree,
  the search results,
  the references list),
  the find box's,
  and the tile raster's background are `#000000`;
  the light scheme is unchanged.
  They share one binding,
  `Surface.background` in `ui/surface.slint`,
  which is `#000000` while the color scheme is dark and `Palette.background` otherwise.
  The raster paints no background of its own and `CodeColors` has no background field,
  so no second black exists to drift from the first.
- Checked on the real binary in the nested compositor
  (`inspect:native`,
  dark scheme,
  1500 by 900 px at scale 1):
  the pixels sampled in the editor,
  the sidebar,
  the header,
  and the minimap's blank columns all read 0,
  0, 0.
  The selection (a blue fill),
  the find matches (the active match in a blue box,
  the others with a boundary),
  the dim comment text,
  and the borders of the viewport box and of the find box were viewed in screenshots at scales one and two,
  and stay legible against black.
- `ui/hover.slint`'s `alternate-background` and the find box's own input fill (`#1E1E1EB3`) are small surfaces and
  were left as they were.

### Decisions where CodeGlance Pro is silent or differs

- CodeGlance Pro draws an empty picture for a file over 20,000 lines (`maxLinesCount` with `outLineEmpty`).
  The picture here has no such limit,
  because its cost follows the bands near the view and not the length of the text.
- CodeGlance Pro draws its picture at 0.8 opacity with a weight for each glyph.
  This picture is the GPU's own coverage of the window's glyphs at a sixth of their size,
  and no gain is applied to it.
  In the dark scheme the large surfaces are pure black (see Black surfaces in the dark scheme),
  so the average of ink over black is as bright as the window's own ink at that coverage.
- Its preview of the code under the pointer after 400 ms (`showEditorToolTip`) and its 7 px gutter for resizing the
  minimap are not implemented (see Limits and follow-ups).
- A press below the end of a short picture jumps to the last line (`fitLineToEditor` clamps),
  and only the left button
  presses.
- The box's rounded corners (radius 5) are kept for boxes taller than 20 px and square below,
  as in CodeGlance Pro.

### Performance bounds

- The picture is built on its own thread (`ide-minimap`,
  `src/native/minimap_worker.rs` and `src/native/minimap_worker/serve.rs`),
  never on the window thread.
  The worker returns paint items cut into chunks (`Built`),
  and the window thread hands them to the renderer with one call (`apply` in `src/native/minimap/picture.rs`).
  The rendering notifier renders the chunks with the one Vello renderer the code view owns
  (`src/native/gpu_tile/slice.rs`):
  there is no second renderer and no scratch memory of the minimap beyond a few chunk-sized textures.
  The window's side is split by what each part answers for in `src/native/minimap/`:
  `state.rs` holds the state and the drag,
  `pace.rs` decides when a request is posted,
  `request.rs` makes the key and posts the request,
  `picture.rs` takes in the finished pictures,
  and `pointer.rs` turns a press,
  a drag,
  and a page step into scroll offsets.
  The picture's own code is in `src/minimap/`:
  `plan.rs` cuts the text into bands at fixed distances from its top and the rows into chunks,
  `shape.rs` shapes the lines of a slice (or takes them from the row cache) and places the marks,
  `line_cache.rs` is the row cache,
  `fills.rs` makes the paint items of the selection and match marks,
  `scene.rs` runs one pass,
  and `chunks.rs` cuts the items into the chunks.
- The picture depends on the text,
  its syntax classes,
  the vertical mapping,
  the wrap width,
  the palette,
  the display scale,
  the size of the minimap,
  the selection,
  the matches,
  the annotations,
  the inks of the fills,
  and the numbers of the bands the track shows with the view scrolled where it is,
  and on nothing else
  (`Key` in `src/native/minimap_worker.rs`).
  A render that changes none of these costs a comparison of a few words:
  a scroll step that keeps the bands posts no request and sets no picture,
  and only the markup moves the picture and the box.
- A request is posted at once for the first picture of a file,
  a new palette,
  and a new scale;
  at most every 60 ms while only the bands the track shows change (`SLIDE`),
  which is how the picture follows a scroll through a long file;
  at most every 250 ms while the vertical mapping changes (`THROTTLE`),
  which is how a long file's measurement shows in the picture while it runs;
  and 120 ms after the last change for everything else,
  such as the size steps of a resize drag or an edit.
  The newest request wins:
  a build stops within 32 lines of the newer request,
  and an older request that was never started is dropped.
- The slice is the bands the track shows and one on each side.
  A band is 768 logical px of the text from a fixed distance from its top (`BAND`),
  so the cuts do not move with the scroll,
  and a slice holds every band the track shows,
  however tall that is.
  The GPU draws a slice into as many textures as the device's texture side asks for
  (`texture_runs` in `src/minimap/plan.rs`:
  whole chunks,
  each texture at most the device's side in rows
  and at most the source tile's byte rule),
  and the markup stacks them one below the other without a gap (`ui/minimap.slint`).
- The slice is rendered in chunks of at most 512 rows (`CHUNK_ROWS`).
  Vello 0.11 sizes the buffers of one render with fixed constants,
  and a scene that needs more renders blank with no error
  (`doc/planning/ide-vello-buffer-sizes.md`).
  The densest slice measured (the whole text selected and every line a match) renders completely at 1,024 rows and
  blank at 2,048 (`src/native/minimap_dense_tests.rs`),
  so a chunk is a factor of two inside what one render holds.
  Each chunk is rendered into a scratch texture of its size (at most four are kept)
  and copied into the slice texture on the GPU,
  so a chunk is a render and a copy and no pixel crosses to the CPU.
  An item that reaches into a chunk (a glyph run,
  a mark,
  a clip group) is kept in every chunk its ink can reach
  (`src/minimap/chunks.rs`),
  so no seam shows at a cut.
- The row cache (`src/minimap/line_cache.rs`) keeps the shaped lines of the picture by a key of the line's text,
  the digest of the syntax classes that reach it,
  the wrap width,
  and the display scale (4,096 lines at most;
  the lines the build did not use are evicted when it is over).
  An edit shapes the lines it changed and takes the others from the cache,
  an edit above the window that moves characters but no row shapes nothing,
  and an external edit of a file of 50,000 lines shapes only the line it touched
  (`an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches`).
  A line added or removed moves every row below it,
  so the slice is encoded and rendered again,
  with every line from the cache.
- The work is bounded by the view,
  not by the text:
  the picture holds the bands the track shows and one on each side,
  whatever the length of the file.
  A build may spend 5,000,000 units of `line_work` (`BUILD_WORK`).
  A line past `WORK_SCAN_LIMIT` characters,
  or one the budget cannot pay,
  is drawn blank and counted in `skipped` of the build's statistics,
  and a `tracing::warn!` gives the range of lines.
- The memory of a picture is the slice texture
  (`4 * width * rows` bytes at the display scale squared,
  for example 110 by 384 px at scale 1:
  165 KiB),
  the chunk scratch textures,
  the row cache (the shaped layouts of at most 4,096 lines),
  and the worker's job,
  which holds a rope snapshot,
  the row map,
  and the shared syntax classes,
  not copies of the text.

### Measured cost

The numbers are for the minimap drawn on the GPU.
The measurements of frame pacing,
memory,
and start-up against `origin/main` before the change are in `doc/planning/ide-gpu-snapshot-harness.md`,
section "Stages 5 and 6".

- A file of 50,000 lines of about ninety characters,
  release build in the nested compositor at 1100 by 660 px:
  the worker takes a median of 3.2 ms for a picture while the box is dragged and the view is scrolled
  (79 pictures,
  90th percentile 5.1 ms,
  longest 19.6 ms),
  and shapes a median of 37 of the 199 lines of the picture,
  the others coming from the row cache.
- A picture is compared with the box average of the window's own pixels by
  `src/native/minimap_reflection_tests.rs`.
  The worst pixel and the mean difference were measured at scale 1:
  plain 13 and 0.45,
  find matches 13 and 0.68,
  diagnostics 19 and 0.41,
  a selection 32 and 0.39;
  at scale 2:
  plain 41 and 0.53,
  find matches 41 and 0.78,
  diagnostics 49 and 0.57,
  a selection 54 and 0.47,
  and the bounds are 36 and 0.8 at scale 1 and 58 and 0.9 at scale 2.
  The runs are the same from one run to the next.
  A control that reads the window three pixels aside does not match.
- A window 12,000 px tall could not be shown in the nested compositor:
  it hosts a window of at most 3,704 px of height here,
  whatever size is asked.
  The tall windows are covered by `src/native/tall_window_minimap_tests.rs` at 3,500 and 12,000 px instead.


### Limits and follow-ups

- The caret and the line-number gutter are not part of the picture,
  and neither are CodeGlance Pro's version control marks or IntelliJ's error stripe as a separate column;
  the selection,
  the find matches,
  and the diagnostics appear in the picture as the window draws them.
- The code preview after 400 ms,
  the resizable width,
  and a drag with the middle button are not implemented.
- A picture is drawn at the size of the minimap,
  so a larger display scale makes a larger texture
  (`4 * width * rows` bytes at that scale squared),
  and the markup draws it at exactly that many device pixels without resampling.
- A line inserted or deleted moves every row below it,
  so the slice is encoded and rendered again for the window,
  but the shaped lines come from the row cache (see Performance bounds).
- A very fast drag of the box over a very long file may show blank rows at the edge of the picture for a moment:
  the worker builds the bands near the new place after the pause of the pace rule,
  and the picture keeps its last rows until then.
  This was not provoked on purpose in the nested compositor.
- There is no quick pass of a coarse picture:
  a picture of a new wrap width arrives as one reply,
  after the lines of the bands near the view are shaped.
- A window taller than the device's texture limit has a picture of several stacked textures that covers the
  whole track (owner decision of 2026-10-21).
  A selection that covers the whole slice makes the render cost grow with its rows until the tile code clips
  rectangles per row (see `doc/planning/ide-gpu-snapshot-harness.md`,
  "Stages 5 and 6").

### Minimap checks

`mise run //package/desktop-app/ide:test:minimap` runs the integration tests without a window:
the geometry (`tests/minimap.rs`)
and the scene (`tests/minimap_scene.rs`:
the plan of bands and chunks,
the most bands a texture holds at every scale and limit,
the glyphs of a slice in the palette's inks,
the selection and the matches as marks under the glyphs,
the row cache (an edit shapes only the lines it changes,
new syntax classes only the lines they reach,
a wrap width or a scale shapes every line again),
the window of a long text,
costly lines,
cancel,
and the cut of the items into chunks).
`test:native` runs the window tests:
`src/native/minimap_tests.rs`
(the box against the geometry at every offset,
a short text,
a press that jumps and the keys afterwards,
a press across the width,
a drag with both clamps and the release,
hover with two channels,
the wheel over the minimap against the wheel over the text,
a picture built on the worker within its bounds,
no request or picture for a scroll,
a worker held at its gate that never blocks the view,
a new wrap width,
a scroll that posts a request only when it names other bands,
the accept rule for stale pictures,
the palette inks in both schemes and the new highlights,
fifty thousand lines,
and the pace of requests for each kind of change),
`src/native/minimap_reflection_tests.rs`
(the picture against the box average of the window's own pixels:
plain wrapped text,
a selection,
find matches,
diagnostics and hints,
at display scales one and two,
with a control that a window read three pixels aside does not match),
and `src/native/tall_window_minimap_tests.rs`
(a window of 3,500 and of 12,000 px keeps a picture with the texture of glyphs,
rendered in chunks that are not
blank,
and a long file in a tall window keeps its picture within the device's texture limit;
a window of 12,000 px over the harness device of 8,192 px gets two textures that cover the track,
and a picture cut for a limit of 1,536 and of 1,024 rows is three and four textures whose texels,
seams,
and composited column equal the one texture's),
and `src/native/minimap_seam_tests.rs`
(the chunks stacked equal one render of the picture at scales 1 and 1.5),
and `src/native/minimap_dense_tests.rs`
(the densest chunks are not blank,
one render of four dense chunks is blank as the positive control,
and a frame scaled down keeps the coverage of its edges).
`inspect:source-guards` removes each rule in a disposable copy and checks that its named test fails:
`minimap-box-share`,
`minimap-slide-share`,
`minimap-drag-ratio`,
`minimap-position-clamp`,
`minimap-renders-glyphs`,
`minimap-glyph-inks`,
`minimap-fills-selection`,
`minimap-fills-joined`,
`minimap-fills-match`,
`minimap-fills-boundary`,
`minimap-frame-winding`,
`minimap-line-key-text`,
`minimap-line-key-text-reload`,
`minimap-line-key-styles`,
`minimap-line-key-wrap`,
`minimap-line-key-scale`,
`minimap-work-budget`,
`minimap-cancel`,
`minimap-bands-uncapped`,
`minimap-texture-limit-key`,
`minimap-texture-runs`,
`minimap-texture-top`,
`minimap-slices-in-place`,
`minimap-most-bands`,
`minimap-chunk-rows`,
`minimap-chunk-size`,
`minimap-chunk-cut`,
`minimap-chunk-copy-row`,
`minimap-slice-top`,
`minimap-window-key-selection`,
`minimap-key-has-bands`,
`minimap-slide-offset`,
`minimap-window-margin`,
`minimap-picture-origin`,
`minimap-origin-whole-rows`,
`minimap-text-x`,
`minimap-press-column`,
`minimap-press-centers`,
`minimap-press-caret`,
`minimap-shift-anchor`,
`minimap-drag-scrolls`,
`minimap-release-ends-drag`,
`minimap-wheel-reaches-view`,
`minimap-built-on-worker`,
`minimap-stale-text`,
`minimap-older-request`,
`minimap-follows-measurement`,
`minimap-resize-keeps-wait`,
`minimap-throttle-period`,
`minimap-slide-period`,
`minimap-scroll-notes-bands`,
and `overscan-early-return`.

Two native tests failed in full runs of the suite that run its tests in parallel (`mise run test:native` without a
filter;
a filtered run is serial,
and none of its 42 runs on this branch or on `main` failed).
In 23 such runs on this branch,
`the_view_follows_the_caret_through_every_visual_row` failed 3 times (for example
"after 161 Down presses the caret is on the drawn row 120 of 162",
with a row map of 121 rows) and
`find_box_context_menu_runs_each_entry` failed once (the menu's Select All did not select the whole find text).
In 10 such runs on `main` the first of these failed once with the same assertion (left 120,
right 147).
The row count the test reads before its Down presses is larger than the row map's count afterwards,
so the test reads
the map before the measurement has finished shrinking it;
the find box failure was found later (2026-10-21):
the right click landed where the clamped menu rests on its last entries,
and a release more than 300 ms after the menu opened runs the entry under the pointer
(see `doc/handover/ide-native-flakes.md`).
The layout tests were adjusted for the 288 px source column and the minimap's width in
`src/native/sidebar_tests.rs` (`minimap_beside`,
`text_area`).

## Fonts and appearance

Official variable roman and real italic fonts for JetBrains Mono and Inter are bundled under `asset/font`
with their original OFL notices.
The Slint compiler is configured to embed imported font bytes.
The UI binds to the system palette.
Native dark/light rendering and font isolation are verified.
Programming ligatures retain per-character caret,
selection,
and copy behavior.
Inter's default kerning,
continuous weights,
optical-size controls,
and real italic selection are covered by font-stack tests.
Native widget snapshots also verify intermediate weights and italic cache invalidation.
Inter's normal Slint UI request currently retains the optical-axis default,
not automatic optical sizing.
Idle DPI changes are covered by a native headless window-event regression.
Live system-theme change and physical-output scale migration remain to be verified.

### Selected rows

The selected row of the tree,
of the search results,
and of the location list draws its text and marks in the ink native code chooses from the selection fill.
It is the rule of [Selected text ink](#selected-text-ink) in `src/selection_ink.rs`,
applied in `src/native/render.rs`.
With the fluent palette that ink is white on `#0078D4` in both color schemes,
where the palette's own selection ink is black in the dark scheme.
The user chose this on 2026-10-05.

Measured on rendered frames in both schemes
by `selected_rows_use_the_ink_chosen_from_the_fill_with_measured_contrast`:

- White on the fill reaches 4.53:1.
  That holds for the tree's file name,
  its slot badge,
  the location list's label and detail,
  and both lines of a search result,
  whether or not the list has keyboard focus.
  No part of a selected row is dimmed,
  except the name of an ignored entry,
  which is drawn at 0.8 opacity (see [Entries that git ignores](#entries-that-git-ignores)):
  everything else of a selected row uses the one ink at full opacity.
- A selected tree row under the pointer or with keyboard focus is tinted.
  The tint is the opposite of the ink,
  black under white ink,
  so it moves the fill away from the text:
  the fill becomes `#006EC3` and the ratio 5.23:1.
  Tinted with the foreground ink,
  as unselected rows are,
  the dark scheme's fill became `#1482D7` and the ratio fell to 4.04:1.

`inspect:theme-guards` removes the ink choice and reverses the tint in a disposable copy
and checks that the test fails.

## Markdown and MDX highlighting

Files ending in `.md` and `.mdx` are read as the Helix language `markdown`,
because the pinned registry lists `mdx` as a Markdown file type.
The block grammar `markdown` paints headings,
quotes,
list markers,
and fences,
and injects three more grammars:
`markdown_inline` into every paragraph and table cell,
`yaml` into frontmatter between `---` lines
(`toml` between `+++` lines),
and the language a fence's info string names
(`rust`,
`js`,
`json`,
`sh`,
`toml` and every other bundled language).
An HTML block or inline tag is painted by the `html` grammar.

### What the palette maps

The palette (`syntax::SCOPES`) had no `markup` entry,
so every `markup.*` capture was dropped and only the frontmatter strings were colored.
On a copy of `.agents/skills/choosing-technology/SKILL.md` the language,
the grammars,
and the injections were all working:
357 spans,
2 of them colored before,
872 spans,
517 colored after.
The measured counts of the other samples are
33 spans and 9 colored before and 43 and 19 after for an `.mdx` file,
and 138 and 39 before and 170 and 71 after for a file with fences in five languages.

Each Markdown role takes one of the six existing hues and a second channel,
so color is never the only mark.
The glyph painter draws glyph runs only,
so underline and strikethrough are not available,
and the second channel is a weight or the real italic face of JetBrains Mono.

- `markup.heading` (marker included):
  blue,
  bold (the source weight plus 300).
- `markup.bold`:
  yellow,
  bold.
- `markup.italic`:
  purple,
  italic face.
- `markup.strikethrough`:
  gray,
  light (the source weight minus 100),
  because the line itself cannot be drawn.
- `markup.raw.inline`:
  green;
  the backticks,
  which stay in the default ink,
  mark the boundary.
- `markup.link` (text and reference labels):
  blue,
  semibold (plus 200).
- `markup.link.url`:
  orange,
  italic face,
  because underline is not available.
- `markup.list` (markers):
  orange,
  bold.
- `markup.quote`:
  gray,
  italic face.

The hues are the ones the other languages use,
so they keep the contrast already measured for them:
at least 6.6:1 in both schemes against an assumed near black and white background
(`markdown_roles_are_painted_in_their_hue_in_both_schemes` prints each ratio).

### What stays plain

- Delimiters (`*`,
  `**`,
  `~~`,
  backticks,
  `>`,
  table pipes,
  `---`) are `punctuation` captures,
  which paint in the default ink in every language.
- The text of a fenced block has no block color:
  a block color would show through every token its own language leaves unpainted.
  A fence in a language without a bundled grammar and an indented code block stay in the default ink.
- The info string after the opening fence is a `label` capture,
  which the palette leaves out because Rust and JavaScript use `label` for loop labels.
- `latex` blocks have no bundled grammar.
- Only the innermost capture paints:
  a code span inside a heading takes the code color and the regular weight,
  and bold inside a quote is upright.
- The other languages that emit `markup` captures change too:
  `<b>`,
  `<em>` and `<a>` in HTML,
  and Rust `///` documentation comments (the `markdown-rustdoc` language).
  A measured Rust file with documentation comments went from 41 to 45 spans and 21 to 25 colored.

### MDX

Helix has no JSX-aware MDX language and none is added,
so an `.mdx` file is Markdown.
A JSX element is painted by the `html` grammar only where CommonMark reads it as HTML:
a `<div className="box">` block shows its tag in purple,
its attribute name in yellow,
and its string in green (measured in the nested compositor in both color schemes),
and the text inside the block is not read as Markdown.
`<Chart data={[1, 2, 3]} />` is not valid HTML,
because an unquoted attribute value cannot hold spaces,
so it is a Markdown paragraph and `[1, 2, 3]` is painted as a link.
`import` and `export` lines are Markdown paragraphs too:
they show no JavaScript highlighting.
A fenced `js` block is painted as JavaScript.
No measured way to do better exists without a JSX-aware grammar,
which Helix does not have and this package does not add.

## Source view keys

The source view is read-only:
typing,
deleting,
and pasting are ignored.
Keys follow ordinary desktop text-view conventions.
Every movement key also has a Shift form that extends the selection from its anchor.

- Left and Right move by one grapheme,
  so a combining sequence,
  a joined emoji,
  and a CRLF pair are each one step.
  Without Shift they first collapse a selection to the side they point at.
- Ctrl+Left moves to the start of the current or previous word,
  Ctrl+Right to the end of the current or next word.
  A word is a run of letters of any script,
  digits,
  and underscores;
  a run of punctuation is a stop of its own.
- Home and End follow the visual rows of a wrapped line,
  as IntelliJ's smart Home and End do
  (`EditorActionUtil.moveCaretToLineStart` and `moveCaretToLineEnd`;
  `src/line_boundary.rs`).
  Home goes to the start of the visual row.
  At the start of a continuation row it goes to the first non-blank character of the line's first row.
  On the first row it alternates:
  from the line start to the first non-blank character,
  from a place after that character to it,
  and from it or from inside the indent to the line start.
  End goes to the end of the visual row without its trailing blanks,
  then to the end of the row,
  and from the end of a row that is not the last to the end of the line before its terminator,
  again without trailing blanks first.
  On the last row,
  End at the end of the line goes back before its trailing blanks,
  as IntelliJ's does.
  Ctrl+Home and Ctrl+End move to the start and the end of the text.
- Up and Down move by one visual row and aim for the horizontal pixel position
  where the run of vertical movements started (IntelliJ's `CaretImpl.myDesiredX`),
  so a short row or an empty line does not pull the caret to the left for the rows after it.
  A horizontal key or a click names a new position to aim for.
  Up on the first row moves to the start of the text,
  Down on the last row to its end.
  A caret that aims past the end of a shorter upper row stops at its end and stays drawn on that row.
- PageUp and PageDown move the caret and the view by the whole visual rows in view.
  A block of virtual rows counts with the first row of its line.
- Ctrl+Left and Ctrl+Right move by document offset,
  so a wrap offset is no stop and no obstacle.
- A caret at the offset where a visual row breaks stands on one of two sides:
  at the end of the upper row or at the start of the lower row.
  The side is part of the reading position (`ReadingPosition::affinity`).
  A click keeps the clicked side,
  End on the upper row leaves the caret at its end,
  and every other move puts it at the start of the lower row.
- Ctrl+A selects everything,
  and Ctrl+C copies the selection as original source text.
- Tab and Shift+Tab move keyboard focus between the tree,
  the sidebar divider,
  the source view,
  and the find input while the find bar is open,
  in that order.

Every caret key scrolls the view by the smallest amount that shows the caret's visual row.
The target of Up,
Down,
and the page keys is found by shaped pixel position,
not by counting characters,
so wide glyphs,
tabs,
combining marks,
and ligatures land where the eye expects.

### Pointer selection

A click places the caret at the nearest grapheme boundary of the visual row under the pointer.
A double click selects a word,
a punctuation run,
or a run of blanks,
the whole of it when it spans several visual rows;
a triple click selects the whole logical line with its terminator,
all its visual rows.
Two quick presses on different visual rows of one line are two single clicks,
as the distance between them is a row.
Shift+click extends from the anchor.
A drag extends by the unit of its press,
across visual rows and lines,
and scrolls when it passes an edge of the view.
A selected line terminator is shown as a mark after the line's text,
so a selected empty line stays visible.
Mouse drags never pan the view;
the wheel and touch scroll it.

### Tab stops

A tab ends at the next tab stop.
Stops lie at multiples of two space advances of the source font,
measured in pixels from the start of the line.
That is the CSS rule behind editord's `tab-size: 2`,
including its clause that a tab narrower than half a space runs on to the following stop.
Because stops are pixel positions,
a tab after a CJK glyph ends where a tab after Latin text ends,
although the fallback font's glyph is not two Latin cells wide.
Caret,
selection,
hit testing,
and find rectangles treat a tab as one character,
and copying yields the source tab.

### Line endings

LF and CRLF end a line.
End stops before a CRLF pair,
Right steps over it as one unit,
and a selection across it copies the original pair.
The last line needs no terminator.

### Selected text ink

Selected text is drawn in white while white reaches a contrast ratio of 3:1 against the selection fill,
and in black on a lighter fill.
The same ink is used in the source view,
on selected rows,
and for selected text in the find and search boxes.
The toolkit's fluent palette keeps the fill `#0078D4` in both color schemes
but pairs it with black ink in the dark scheme.
Black on that fill has a WCAG 2 ratio of 4.64 and white of 4.53,
so the ratio alone does not separate them;
the light ink is the readable one on a saturated mid-tone,
and it is what the light scheme always showed.

### Deliberate differences from editord's editor

editord leaves caret movement and selection to the browser's `contenteditable` handling.
This view implements the conventions of this section itself
and was not compared key by key with that engine.

- Words are runs of one character class,
  not dictionary segments:
  a run of CJK letters is one word.
- Ctrl+Up,
  Ctrl+Down,
  Ctrl+PageUp,
  Ctrl+PageDown,
  and Alt combinations are unbound.
- The page keys move by the whole visual rows in view without an overlapping row.
- The caret cannot stand in the indent of a continuation row or in virtual space after the end of a line.
  IntelliJ's Home therefore has one step fewer on a continuation row (the move into the indent),
  and on a line of blanks only it stays where it is instead of going to the indent of the line above.
- A drag past an edge of the view scrolls only while the pointer moves;
  no timer keeps scrolling under a resting pointer.
- Long lines wrap like editord's (see [Soft wrap](#soft-wrap)),
  and the keys move by visual rows as IntelliJ's do.
- Selected text is drawn in one ink instead of keeping its syntax colors.

`test`,
`test:native`,
and `inspect:source-guards` cover these rules:
library tests for movement,
tab stops,
and boundaries,
headless native tests that drive real window key and pointer events,
and guard-removal controls in a disposable copy.

## Current reader behavior

Copy selected source with Ctrl+C.
There is no read-only badge,
Copy button,
or permanent diagnostic footer.
Source positions,
painting,
and selection share native shaped-row geometry.

The displayed file is reread by a bounded background worker when a change notification names it;
see [Change watching](#change-watching).
External changes map the latest caret,
selection,
and viewport through Helix correspondence,
even while text remains selected.
Read failures retain the last readable source and show a diagnostic until recovery.
The concrete caret and replacement-selection cases in the accepted scope pass through the native GUI.

## Change watching

The tree and the displayed file follow external changes through Linux inotify,
using the `notify` crate 8.2.0 (`INotifyWatcher` by name,
default features off,
no polling backend).
The same inotify instance also watches the project's source folders for the language servers;
see [Watching for the language servers](#watching-for-the-language-servers).
Only what is shown is watched,
each directory non-recursively:
the project root,
every visible expanded folder,
and the displayed file's folder,
even when the tree does not show that folder.
A displayed file outside the project,
opened from a language target such as a standard-library source,
is not watched and is not a watch failure:
the safety sweep rereads it.
Collapsing a folder removes its watch;
folders inside a collapsed folder stay expanded in the tree but are not watched.
A folder is watched only at its own canonical path inside the project root,
so a symbolic link to a folder elsewhere,
or to another folder of the project,
is never watched.
The watcher only reads.

A notification carries no data.
It marks a directory listing or the displayed file as due,
and the existing bounded readers reread;
their request identity and stale-reply fencing still decide what is shown.
A burst of notifications for one folder becomes one pending reread,
with at most one more after a read already under way.
A notified folder or file is not reread sooner than 100 ms after its previous read started:
a single change is read at the next 20 ms timer tick,
and a folder or file that changes continuously,
such as build output or a busy log,
is reread at most 10 times per second.
The IDE's own opens and reads of watched paths are not changes and are ignored.
A new watch rereads its folder once more,
because a change can land between the first listing and the watch;
a newly displayed file is read once more for the same reason.
A permission change on a watched folder rereads that folder.

For the displayed file,
a closed write,
a rename into place,
a removal,
and a permission change are read at once.
A write that is still open,
including the truncation that starts an in-place save,
and a newly created file are read once 50 ms pass without another write,
and at most 100 ms after the first.
The user chose both values on 2026-10-05.
While such a write waits,
neither the safety sweep (the periodic reread in [Recovery and timers](#recovery-and-timers))
nor a highlighting request reads the file.
The latest notification decides,
so a save that deletes and rewrites the file waits for the rewrite.
A writer that leaves the file unfinished for longer than the quiet period is read mid-write,
and read again when it closes the file.

A read that no write notification asked for accepts the file only when it has been quiet:
the safety sweep,
the timer of an unwatched file,
a highlighting retry,
the first read,
and the reread after a new watch,
a newly displayed file,
or lost events.
The reader asks the open file for its modification time after reading;
when that time lies less than 50 ms before the read began,
or during the read,
or after the current time,
the bytes are dropped and the file waits like an unfinished write.
The check depends on the file alone,
so no order of timers and notifications lets such a read show a save in progress;
see [Measured write wait](#measured-write-wait).
A file system that keeps modification times in whole seconds can let a save through this check.

### Recovery and timers

A full reread of every shown folder and the displayed file follows:
an inotify queue overflow (`IN_Q_OVERFLOW`),
an error from the notification stream,
a watched folder that is removed or renamed,
a watch that cannot be added,
reaching the `fs.inotify.max_user_watches` limit,
and a watcher that cannot start or stops.
Each is logged once.
A renamed folder's watch is removed,
because inotify keeps following the moved directory under its old name.

A shown item without a live watch keeps the previous timers:
the displayed file every 250 ms,
and unwatched folders one at a time every 500 ms.
Failed watches are retried at each safety sweep,
while anything shown lacks a watch.
A failure is logged and followed by the full reread once,
and again only when its error text changes;
a watch that works again is logged once.

The watch limit counts the watches of every program the user runs,
so reaching it is one state rather than one failure per folder.
The first refused watch logs one warning that names the sysctl,
rereads everything shown once,
and stops adding watches in that pass.
The safety sweep then retries after 2 s,
doubling the wait after each refused retry up to 64 s.
Expanding,
collapsing,
switching files,
and scrolling the tree (at most once a second) retry at once
without lengthening the wait.
The displayed file's folder is tried first.
When every shown folder has a watch again,
one line says so.
Folders without a watch keep the timers and the sweep meanwhile.
See [Watch counts and the watch limit](#watch-counts-and-the-watch-limit).
Folders watched for the language servers come after these and have their own limit state.

Every second,
every shown folder and the displayed file are reread anyway,
after all notified work:
the safety sweep.
The user chose the 1 s interval on 2026-10-05.
inotify never reports some changes:
network and FUSE mounts,
writes through `mmap`,
and unmounts (`notify` does not map `IN_UNMOUNT`).
The sweep bounds how long those stay stale.
One directory read starts per 20 ms timer tick,
so one pass over more than about 50 shown folders outlasts the second.
The next pass then starts when the previous one has read every folder,
and each folder is reread once per pass.
The sweep costs one listing per shown folder and one source read per second,
where the previous polling did 2 listings and 4 source reads per second whatever was shown;
see [Measured idle cost](#measured-idle-cost).

With the sweep at 1 s,
the 500 ms timer for unwatched folders shortens the longest time a folder goes unread
only when one to three shown folders lack a watch:
500 ms for one,
900 ms for two,
980 ms for three,
and the sweep's 1 s from four
(`tests/refresh_intervals.rs` prints these from the shipped schedule).
The 250 ms timer for an unwatched displayed file stays four times as frequent as the sweep.
Both timers are kept.

### Watch counts and the watch limit

The IDE holds one watch per shown folder:
2 with one expanded folder,
13 with 12,
and 101 with 100,
counted from its inotify descriptors in `/proc` (`inspect:idle-cost`).
Watching only the folders in the tree's viewport was measured as the alternative
(`inspect:watch-scope`,
60 expanded folders,
three runs each):
it held 5 watches instead of 61,
but in 48 of 60 trials a folder changed while out of view showed its old listing
for a median of 61 to 85 ms per run,
at most 148 ms,
after it scrolled into view;
watching every expanded folder showed the change at once in 59 of 60 trials,
1 ms late in the other.
The IDE keeps watching every expanded folder.

The language servers count against the same limit;
see [Watching for the language servers](#watching-for-the-language-servers).

The IDE's own podman tasks run their containers with `--network=none`,
except `fetch` and `runtime`,
which download;
podman's default network starts a pasta helper per container,
which holds an inotify watch and warns when the limit is reached.

A positive control runs the IDE in a disposable user namespace whose own watch limit is 4
(`inspect:watch-limit`,
12 expanded folders,
the host limit untouched):
in each of two runs,
one warning,
no per-folder warning,
4 and 3 refused watch calls in 60 s with 59 and 60 sweeps,
a file created in an unwatched folder listed after 629 and 751 ms,
and one line 63.8 and 61.3 s after the namespace limit was raised,
the backoff's longest wait.
In the second run,
Home in the tree scrolled it and retried the watches 43 ms later,
with no watch call in the 1.5 s before.

### Watching for the language servers

Language servers learn about changes made outside the editor through `workspace/didChangeWatchedFiles`.
Helix watches no files for its servers,
so its built-in rust-analyzer definition sets `files.watcher = "server"`,
and rust-analyzer then asks notify for a recursive watch of each workspace package's folder (`vfs-notify`),
which watches every folder below it,
`node_modules` included,
whatever `files.excludeDirs` says.
The TypeScript 7 server watches nothing itself:
Helix declares client-side watching,
which TypeScript 7.0.2 then relies on (`internal/lsp/server.go`),
and before this change nothing told it about other files,
so the displayed file's diagnostics did not follow a change of a file it imports.

The IDE watches for the servers.
While some server has registered file watchers (`client/registerCapability`),
the change watcher watches every source folder of the project,
each non-recursively,
on its one inotify instance,
and sends every change in them to the language worker,
without passing through the interface thread.
When no server has watchers any more,
those watches are released.
Source folders are the folders ripgrep lists files from with the search's settings (`src/search_process.rs`),
so the search and the servers agree on `.gitignore`,
`.ignore`,
`.rgignore`,
and hidden names;
hidden folders such as `.cargo` are skipped as the search skips them.
`node_modules`,
`target`,
and `.git` are never watched,
even where no ignore file names them.
ripgrep lists a folder named on its command line even when an ignore rule names it,
but applies the ignore files of its parents below it (ripgrep 15.2.0),
so a new folder is classified by scanning its parent again.
An empty folder cannot be classified by a list of files:
it is watched until its first change,
which is held back until its parent has been scanned again;
the files that scan lists in it are then sent as created,
and a new ignored folder,
such as a build's `dist`,
never reaches a server.
Scans run on their own thread.

The language worker keeps each server's registrations itself.
helix-lsp's own handler (`helix-lsp/src/file_event.rs` at the pinned revision) keeps only string patterns,
ignores the kinds a watcher asks for,
and always sends "changed",
while both rust-analyzer and the TypeScript 7 server register relative patterns.
Glob patterns follow the protocol:
`*` and `?` stay inside one path segment,
`**` spans segments,
and `{}` and `[]` group.
A watcher's kind decides which of created,
changed,
and deleted it hears,
and a server that registered nothing gets nothing.
Changes are gathered per path with the kind of its final state on disk
(a deletion after a creation stays a deletion),
and a burst is sent once it pauses for 50 ms,
or 500 ms after it began,
as one notification per server.
Each server that heard about changes is then asked for the displayed file's diagnostics again,
since another file may have changed them.
A reload of the displayed file reaches the servers the same way.
rust-analyzer is given `files.watcher = "client"`,
set in code after the definitions are merged,
because a merged `files` table replaces the whole table;
it then registers watchers and takes no watches of its own.

The tree and the displayed file come first under the watch limit.
A folder both want keeps one watch,
and collapsing it in the tree keeps the servers' watch.
When the limit refuses a watch the tree wants,
a watch held only for the servers is given up for it,
and while the tree waits on the limit,
the servers add nothing.
The servers' limit is its own state:
one warning naming `fs.inotify.max_user_watches`,
one line when every source folder is watched again,
and retries after the same growing wait (2 s doubling to 64 s),
which the watch thread schedules itself.

`inspect:server-watches` measured both on disposable projects,
with the servers confined as in production,
on a quiet host,
two runs of each case in one session
(`~/temp/agent/ide-server-watches-5jAwxc`).
The Rust workspace has 40 crates,
1080 source folders,
a `target` of 6000 folders,
a `.git` of 257,
and a `node_modules` of 2001 folders inside its first crate,
9345 folders in all.
The displayed file calls a function another file defines,
and the other file is changed outside the IDE to give it a parameter,
which rust-analyzer's own analysis reports;
`cargo check` is turned off (`checkOnSave = false`) so it cannot report the change instead.
The TypeScript project has 1000 source files,
3000 dependency packages,
12328 folders in all,
and its displayed file imports a constant whose export is removed outside the IDE.

#### Before: the build at `45db45d02`

- rust-analyzer watched by itself:
  3081 watches in both runs,
  1080 for sources and 2001 for `node_modules`.
- TypeScript:
  no watches at all.

#### After: the IDE watches for the servers

- Rust:
  1082 watches in both runs,
  all held by the IDE and all on source folders;
  rust-analyzer held none.
  The displayed file's diagnostic appeared 2008 and 745 ms after the write.
- TypeScript:
  42 watches in both runs,
  all held by the IDE;
  the diagnostic appeared after 102 and 60 ms.

#### Positive control: the same build without forwarding

- Rust and TypeScript:
  no watches,
  and the diagnostics stayed unchanged for 60 s in both runs of each.

On this repository,
ripgrep lists files in 1549 folders,
the count the IDE would watch for a server with the repository as its project.

### Threading and shutdown

A watch thread owns the watcher and applies the set of shown folders,
because adding a watch blocks until notify's own event thread answers.
notify's event thread records notifications into state the UI timer drains every 20 ms.
Closing the window closes the watch thread and joins it;
dropping the watcher makes notify remove its watches and close its descriptor.
notify starts its event thread detached,
so that thread is not joined.
A watch call stuck on a hung filesystem delays window close,
as a stuck directory or file read already does.

### Measured refresh latency

`inspect:refresh-latency` times an external write until the tree row or the source text changes,
through the shipped bindings in the headless window,
16 trials per case,
three runs per build,
with the same pseudo-random write gaps and target folders in both builds.
The runs of the two builds alternated in one session on a busy host,
with a load average of 49 to 156 on 16 processors.
Polling (`48a1b5756`):
a new file in one of 8 expanded folders took a median of 1672 to 1789 ms across runs,
at most 4038 ms;
with one expanded folder,
671 to 789 ms,
at most 1524 ms;
a rewrite of the displayed file,
93 to 148 ms,
at most 288 ms.
Watching,
with the 1 s sweep and the 50 ms write wait:
a new file took a median of 28 to 33 ms with 8 folders and 35 to 51 ms with one,
at most 115 ms;
a rewrite took 38 to 75 ms,
at most 140 ms.
The medians of one build differed between its runs by at most 118 ms under polling and 27 ms under watching.
The schedule makes a change that lands within 100 ms after a sweep read of the same folder wait for the reread gap;
with the sweep at 1 s that is about one change in ten,
and the slowest trials near 100 ms fit it.

### Measured write wait

`inspect:refresh-latency` with the filter `write_wait_pause` plays an in-place save against the headless window:
truncate,
pause,
write,
close,
with `am a` selected,
8 trials per pause length in each of three runs.
With the 50 ms quiet period and the 100 ms limit,
no trial with a pause of 10 to 50 ms showed the truncated file,
2 of 24 did at 60 ms,
4 of 24 at 70 ms,
and all 96 at 80 ms and longer.
The wait is counted from the 20 ms timer tick that receives the notification,
so the boundary lies between 50 and 80 ms.
Every trial that showed the truncated file lost the selection;
every other trial kept it on `was a`.
A copy with the earlier 150 ms and 250 ms,
measured in the same session,
showed the truncated file in all 24 trials at 200 ms and in none at 100 and 150 ms.

Two trials showed the truncated file at a pause below the quiet period:
one of 200 trials with pauses of 10 to 50 ms across five runs at the shipped values,
and one of 240 trials with pauses of 10 to 150 ms in the copy with 150 ms and 250 ms.
Their cause is not established;
the sweep is ruled out for the second,
whose save began about 200 ms after a read had restarted the sweep clock.

The filter `write_wait_timer` starts the same save,
unfinished for 40 ms,
at a pseudo-random time within the 400 ms that contain the next sweep read,
until 120 trials have counted.
A trial counts only when the file stayed truncated for less than the quiet period in real time;
on a loaded host the test thread itself can stall mid-save,
which makes the save a long one that any read may show by design.
Before the quiet check (`45db45d02`) 4,
6,
and 8 of 120 trials showed the truncated file in three runs.
With it,
none of 120 did in any of three runs,
alternating with the earlier build in one session at a load average of 63 to 99 on 16 processors.
Of the trials the stalls stretched past the quiet period,
6 of 30 showed the truncated file before the check and 1 of 41 with it.

### Measured idle cost

`inspect:idle-cost` runs the release build in the nested compositor,
expands sibling folders of eight files each through key input,
and samples the idle IDE for 60 s:
CPU time from `/proc/<pid>/stat`,
read calls from `/proc/<pid>/io`,
listings from the IDE's log,
and every system call in one further session under `strace`.
The shipped 1 s build and a build that differs only in a 10 s `SAFETY_SWEEP` alternate,
three runs each;
ranges give the lowest and highest run.

#### One expanded folder

- 1 s sweep:
  3.0 to 3.2 ms of CPU per second,
  59 to 67 read calls,
  2 listings,
  301 system calls per second.
- 10 s sweep:
  2.5 to 2.8 ms of CPU per second,
  50 to 52 read calls,
  0.2 listings,
  256 system calls per second.

#### 12 expanded folders

- 1 s sweep:
  7.3 to 8.2 ms of CPU per second,
  84 to 90 read calls,
  12.9 listings,
  561 system calls per second.
- 10 s sweep:
  4.7 to 5.2 ms of CPU per second,
  54 to 57 read calls,
  1.3 listings,
  284 system calls per second.

#### 100 expanded folders

- 1 s sweep:
  45.0 to 45.5 ms of CPU per second,
  154 to 168 read calls,
  49.9 listings,
  1353 system calls per second.
- 10 s sweep:
  26.0 to 26.3 ms of CPU per second,
  73 to 90 read calls,
  10.1 listings,
  480 system calls per second.

#### Reading the numbers

Runs of one build differed by at most 0.84 ms of CPU and 17 read calls per second.
With 100 expanded folders one pass over the 101 shown directories took 2.02 s,
so the 1 s build reads without pause,
and every directory was listed once per pass.
There the 1 s build does 40 more listings per second than the 10 s build,
for about 19 ms more CPU and about 870 more system calls per second:
about 22 calls per listing,
of which 8 are `readlink` (path resolution) and 4 are `write` (log lines).
The 10 s build's cost also grows with the tree,
from 2.5 to 26 ms of CPU per second;
it sweeps too,
at 10 listings per second with 100 folders,
and what else grows with the tree was not separated.
The IDE logged at debug level during these measurements
(since 2026-10-06 the default is warnings,
and the measurement sets `RUST_LOG` for the records it reads),
four lines per listing:
9,
52,
and 200 log lines per second at the 1 s sweep,
against 0.9,
5.3,
and 40 at 10 s.
The log was written from the UI thread,
so a log destination that blocked stalled the window;
one sample with the log on a busy disk stood still for 10 s,
and the measurement keeps its live log in memory-backed storage for that reason.
Since 2026-10-06 a writer thread writes the log ("Log").
The displayed file was 640 bytes;
the sweep reads the whole file once per second,
so that part grows with the file,
and was not measured for large files.
The host was busy during the runs with one and 12 folders,
with a load average of 6 to 48 on 16 processors,
and nearly idle during the runs with 100;
the builds alternate within each run,
so each comparison shares its conditions.

### Deliberate differences from editord

editord watches each folder with chokidar when it is first expanded and keeps the watch after collapse;
this reader removes it on collapse.
editord refreshes the displayed file only through its folder's watch,
so a file opened from search in a collapsed folder is not refreshed there;
this reader always watches the displayed file's folder.
editord ignores events for `.git`,
`node_modules`,
editor swap,
and temporary file names;
this reader lists those names like any other,
so their changes appear.
editord waits for a file's size to stay unchanged for 150 ms (`awaitWriteFinish`) before reporting it;
this reader reads closed writes at once,
and for writes still open it waits 50 ms,
a shorter wait than editord's that the user chose on 2026-10-05.
editord drops a folder's watch on an error;
this reader rereads everything shown,
keeps that folder on a timer,
and retries the watch.
editord has no overflow handling or periodic reread.

### Change-watching checks

`tests/change_watch.rs` runs the watcher over disposable folders:
entry changes,
moves in and out,
the extra read after a new watch,
collapse removing the kernel watch,
finished and unfinished writes,
silence for the IDE's own reads,
a real queue overflow,
a notification error,
removed,
renamed,
failed,
and retried watches,
a repeated failure reported once,
refused outside and symbolic-link folders,
and no watch left after shutdown.
`tests/refresh_policy.rs` pins the intervals.
`tests/refresh_intervals.rs` simulates the shipped schedule in 20 ms ticks:
timers leave an unfinished write alone,
a pass over more folders than one interval can read still rereads every folder,
and the unwatched timer never makes a folder staler than the sweep alone.
`native::watch_tests` checks the shipped tree and source in the headless window.
`native::write_wait_tests` plays a slow writer against that window:
an in-place save and a delete-then-rewrite are shown only when finished.
`native::quiet_read_tests` does the same for a file outside the project,
whose reads no write notification ever asks for.
`tests/quiet_read.rs` checks the reader's quiet requirement with chosen modification times
and the reread classification.
The library tests in `change_watch/reconcile_tests.rs` drive the watch limit with a fake kernel:
one state,
no further adds,
the backoff and its cap,
immediate retries,
and the displayed file's folder first.
`change_watch/server_watch_tests.rs` drives the servers' watches with a fake kernel:
one watch for a folder both want,
a server watch given up for the tree,
the servers waiting while the tree waits,
and their own limit state.
`change_watch/server_scan_tests.rs` checks the scan's ignore rules,
pruned and empty folders,
and new folders classified by their parent.
`language/watched_files_tests.rs` checks the merge rule,
glob patterns per the protocol,
kinds,
bursts,
and servers that stopped.
`tests/language/watched.rs` drives the worker and a change watcher with the scripted server,
which registers watchers from `IDE_SCRIPTED_WATCHERS`:
changes by glob and kind,
the diagnostics pull after them,
ignored,
dependency,
and unmatched files left out,
a burst of 500 changes in a few notifications,
new folders followed,
a new ignored folder never sent,
and nothing for a server without watchers.
`native::language::watched_tests` checks the same through the shipped window's language tick.
`inspect:watch-guards` removes each guard in a disposable copy and requires its named test to fail.
`inspect:idle-cost`,
`inspect:refresh-latency`,
`inspect:watch-scope`,
`inspect:server-watches`,
and `inspect:watch-limit` produce the measurements in this section.
In the nested compositor,
dark and light,
external create,
rename,
and delete in an expanded folder,
and both correspondence examples through atomic replace,
show without polling.

## Log

The application writes its log to standard output.
By default it holds warnings and errors.
The standard `RUST_LOG` variable adds detail in `tracing-subscriber`'s directive syntax,
for example `RUST_LOG=ide_app=debug,monochromatic_ide=debug` for every record of the application,
which was the fixed level before 2026-10-06.
A directive in `RUST_LOG` replaces an earlier one for the same target,
so it can also reduce the log.
`helix-lsp` writes every protocol message in full at INFO,
so it stays at warnings whatever `RUST_LOG` says,
unless `RUST_LOG` names a `helix_lsp` target itself;
a bare level below warnings,
such as `RUST_LOG=error`,
applies to it too (`src/logging.rs`).

`helix-lsp` logs three kinds of record at ERROR for servers that work as intended.
The application lowers exactly those,
keeping their text,
target,
and source location
(`src/logging/relabel.rs`):

- a line the server wrote to its standard error,
  `{name} err <- "{line}"`,
  to INFO;
- the end of that stream,
  on every server exit,
  `{name} err: <- StreamClosed`,
  to DEBUG;
- an error answer with code `-32801` (content modified) or `-32800` (request cancelled),
  which the worker asks again for,
  to DEBUG.

Every other `helix-lsp` record keeps its level,
including error answers with any other code and failures to read or write a server's streams.
`RUST_LOG=helix_lsp=debug` shows the re-labelled records together with the protocol messages.

Each server's last standard-error lines are kept whatever the log shows (`src/logging/stderr_tail.rs`):
the newest 8 lines per server,
each cut to 512 bytes at a character boundary and then marked ` [cut]`.
`helix-lsp` passes every line on as an ERROR record,
so they are kept unless the log is off entirely.
When a server process ends without being asked to,
during its start or after it,
the worker's warning `language server process ended` lists them in its `stderr_tail` field,
oldest first.
That warning waits up to 1 s for the end of the server's standard error,
because `helix-lsp` can report the end of the process before its last lines;
the session state does not wait,
and a warning still waiting when the worker stops is written then.
When the worker stops a server that did not answer `initialize` in time,
the start-deadline error is followed by the warning
`language server wrote this to standard error before it was stopped`
with the lines so far,
and only when there are some.
A clean shutdown,
and a server the worker stops for any other reason,
log nothing more.
The log keeps all of them;
the window shows the last four of a failed start,
each cut to 200 bytes,
in the reason of that start
([Why a feature is missing](#why-a-feature-is-missing)).
`helix-lsp` names a line's server by its configured name only,
so two processes of one name share a tail;
lines a process writes after its lines were reported,
or after the worker stopped it,
are dropped until its standard error ends,
so they are never reported as a later process's words.

Records go through a queue to one writer thread (`src/logging/background.rs`),
so no thread of the application,
the interface thread included,
waits for the output.
A reader that stops reading,
or a disk that holds an append for seconds,
blocks only that writer thread.
The queue holds at most 8 MiB of formatted records.
A record that does not fit is dropped and counted;
the next record that fits is preceded by one warning line,
`N log records (B bytes) were not written because the log output did not accept them in time; the log has a gap here`,
which the writer thread writes itself,
without a level filter.
A clean exit waits up to one second for the queue to be written,
then exits anyway;
records still queued then are lost without a report,
because the only place to report them is the output that is not accepting.
The inspection tool `ide-language-inspect` logs the worker's debug records by default,
to standard error,
through the same filter and re-labelling,
and writes synchronously.

Checks:
`test:language logging` runs the filter,
re-labelling,
and queue tests,
among them `logging::background::tests::a_blocked_output_never_delays_the_logging_thread`,
which logs 5000 records through the real subscriber while the output stalls for 4 s;
`inspect:language-lifecycle-guards` makes the full queue wait for room and observes that test fail.
The kept lines have unit tests in `src/logging/stderr_tail_tests.rs` and `src/language/attach/report_tests.rs`,
and scripted-server tests in `tests/language/quiet.rs`:
a server that writes a line at its start and one right before it exits with status 7,
one that exits with status 3 when `initialize` arrives,
one that never answers `initialize`,
and the clean lifetime,
which must log neither the field nor an ended-server record.
The guard tool stops keeping lines,
stops waiting for the end of the stream,
and reports the lines on a clean shutdown,
and observes the named test fail each time.
`mise run //package/desktop-app/ide:inspect:log-stall <cache> [cargo] [seconds]`
runs the IDE in the nested compositor with every application record on
and its standard output on a FIFO that is never read,
moves the caret with key presses for 30 s,
and reads from the IDE's own records the longest pause between two keys the window handled.
It also photographs the compositor every 200 ms (`screenshot`,
which composites the latest frame the IDE committed)
and counts how often the picture changed and for how long it stood still:
a moving or blinking caret redraws the window on its event loop,
so a blocked event loop leaves the picture still.
A window that never appeared counts as a stall of both.
Until 2026-10-20 the check asked the window's Slint MCP server for its window list instead;
the build with that server is gone (see [Inspecting the running window](#inspecting-the-running-window)).
The same sources with the writer replaced by plain standard output are the control.
On 2026-10-06,
at a load average near 90 on 16 processors,
the shipped build handled 494 keys with a longest pause of 2.2 s and answered every MCP probe within 5 s,
and the control handled 8 keys,
then paused 31.8 s and left 6 probes unanswered
(`~/temp/agent/ide-log-stall-RMpmMv/results.json`).
On 2026-10-20,
at a load average near 11 on 24 processors,
the shipped build handled 573 keys with a longest pause of 81 ms,
and its picture changed in 143 of 144 photographs with a longest still time of 263 ms;
the control mapped no window in 120 s and handled no key,
because it blocked right after its first tile was built:
its last record before a gap that ends only when the script drains the FIFO two minutes later is 222 lines,
about 51,000 characters,
or an estimated 65,600 bytes with the colour codes (their average over the whole log),
which is the size of the 64 KiB pipe buffer.
The control therefore stalls before it maps a window,
and the picture check sees no window rather than a frozen one
(`~/temp/agent/ide-log-stall-jyzKKf/results.json`).
The picture probe itself was checked on a release build that was stopped with `SIGSTOP` and continued with `SIGCONT`
while the same key presses ran:
19 changes in 20 photographs while running,
0 in 20 while stopped,
and 20 in 21 after it continued.
A pause or a still picture of 5 s or more fails the shipped build;
single slower redraws occur on a busy host without any log involved.

A bare `RUST_LOG=debug` also turns on the debug records of every library:
the log up to the first window grew from about 60 KB to about 747 KB,
while the time to the first window stayed within the spread of three sessions each
(0.7 to 10.5 s with the application's records,
1.4 to 2.6 s with every record,
load average near 80).

## Build boundary

Application builds and tests run in the IDE container extending the repository's native-app image.
The container receives this package and a dedicated Cargo cache,
not the user's home or credentials.
The source documentation/line-budget task invokes the repository's Rust linter separately.

### Container bound

`bin/container-bounds.ts` is the one place that computes the bound of every container the tasks and the inspection
scripts start,
and of the `image` build.
With nothing set,
the bound is a computed share of the machine that runs the task:
90 percent of the CPUs the process may use (`os.availableParallelism()`) and 90 percent of the physical memory
(`os.totalmem()`),
each rounded down,
with at least one CPU and 2 GiB.
The process limit is 256 per CPU with a floor of 512,
and the descriptor limit is 4096.
No CPU count or memory size is written in the repository.
The owner decided on 2026-10-18 that the containers use 75 percent of the host,
and on 2026-10-22 raised it to 90 percent ("Use my system (almost) fully");
the share is the constant `hostShare` in the module.

A caller on a loaded host,
or one that wants another bound,
sets these variables for the task:

- `IDE_CONTAINER_CPUS`:
  CPUs as a positive number with at most two decimals,
  such as `4` or `1.5`.
- `IDE_CONTAINER_MEMORY`:
  a whole number with the unit `k`,
  `m`,
  or `g`,
  such as `512m` or `8g`.
- `IDE_CONTAINER_PIDS`:
  the process limit,
  a positive whole number.
- `IDE_CONTAINER_NOFILE`:
  the descriptor limit,
  a positive whole number.
- `IDE_CONTAINER_JOBS`:
  `CARGO_BUILD_JOBS`,
  which otherwise equals the whole CPUs of the bound.
- `IDE_CONTAINER_TEST_THREADS`:
  `NEXTEST_TEST_THREADS`,
  which otherwise equals the whole CPUs of the bound.

For example,
`IDE_CONTAINER_CPUS=2 IDE_CONTAINER_MEMORY=2g mise run //package/desktop-app/ide:test` reproduces the
fixed bound of 2 GiB and 2 CPUs that the tasks used before 2026-10-18.
A value that does not match its form stops the task with a message that names the variable and the accepted form.
The container tasks print the bound they use on standard error before they start the container.

The test runner is given the CPU limit explicitly through `NEXTEST_TEST_THREADS`.
Inside the container `nproc` prints the CPUs of the host (32 on the measuring host,
with a quota of 2),
but the installed cargo-nextest reads the control group quota through Rust's `available_parallelism`,
so on a quota of two CPUs it already ran two tests at once (measured with 48 sleeping tests).
The variable makes the count explicit and lets `IDE_CONTAINER_TEST_THREADS` override it;
see [the troubleshooting note](../../../doc/troubleshooting/container-nproc-ignores-cpu-quota.md).

### The IDE bin scripts are TypeScript

Every script of `bin/` and `design/` is a TypeScript file
(the compositor module,
the inspection,
guard,
and capture scripts,
`install.ts`,
`check-nextest-config.ts`,
and the test file `nested-compositor.test.ts`),
and the tasks run them with `node` directly (`node bin/inspect-cadence.ts`).
Node strips the types of a `.ts` file before it runs,
so nothing is built,
and the files use only syntax that stripping can erase:
no enums,
parameter properties,
or namespaces,
and an import of another TypeScript file names the `.ts` extension.
Functions of two or more parameters take one destructured object,
except where the caller dictates the signature (a callback of `map`,
`filter`,
`reduce`,
or `sort`,
or an event listener).

Until 2026-10-22 these files were plain `.mjs` files,
a habit with no rule behind it.
Two repository-wide style commits (`68415aeb53` and `3da31bdc3a`,
issue 1794) left 17 of those `.mjs` files unparseable,
while none of the TypeScript files they touched was broken,
so those 17 were converted instead of restored:

- `bin/nested-compositor.ts`:
  the compositor name and the check of its version,
  tested by `bin/nested-compositor.test.ts`.
- The inspections that measure or exercise a real IDE or server:
  `bin/inspect-bundle.ts`,
  `bin/inspect-cadence.ts`,
  `bin/inspect-find-reference.ts`,
  `bin/inspect-idle-cost.ts`,
  `bin/inspect-language.ts`,
  `bin/inspect-language-confinement.ts`,
  `bin/inspect-language-lifecycle.ts`,
  `bin/inspect-language-mise.ts`,
  `bin/inspect-language-reap-rate.ts`,
  `bin/inspect-log-stall.ts`,
  `bin/inspect-server-watches.ts`,
  `bin/inspect-watch-limit.ts`,
  and `bin/inspect-watch-scope.ts`.
- The control for the bundle checks:
  `bin/inspect-bundle-guards.ts`.
- The two capture scripts of the design notes:
  `design/soft-wrap-shots.ts` and `design/wrap-annotation-shots.ts`.

The 15 `.mjs` scripts that were still parseable were converted the same way on 2026-10-22,
by the owner's decision to convert all of them:
`bin/check-nextest-config.ts`,
`bin/install.ts`,
`bin/nested-compositor.test.ts`,
`bin/inspect-refresh-latency.ts`,
and the guard inspections
`bin/inspect-annotation-guards.ts`,
`bin/inspect-find-guards.ts`,
`bin/inspect-ignored-guards.ts`,
`bin/inspect-language-guards.ts`,
`bin/inspect-language-lifecycle-guards.ts`,
`bin/inspect-language-navigation-guards.ts`,
`bin/inspect-search-guards.ts`,
`bin/inspect-sidebar-guards.ts`,
`bin/inspect-source-guards.ts`,
`bin/inspect-theme-guards.ts`,
and `bin/inspect-watch-guards.ts`.
Only types,
the one-object call shape,
and the few narrowings that the strict options demand were added to them;
their layout and behavior are otherwise as before.
No `tsconfig.json` and no `lint:types` or `lint:oxlint` task covers this directory,
because the package is a Rust crate and not a workspace package,
so a type error shows only when a script runs.
The converted files were checked with the strict options of `package/config/typescript/tsconfig.options.json`
(`strict`,
`noUncheckedIndexedAccess`,
`exactOptionalPropertyTypes`,
`erasableSyntaxOnly`)
from a scratch `tsconfig.json`,
and `module.stripTypeScriptTypes` accepts each of them.

### Running the tests

`mise run //package/desktop-app/ide:test` runs every library and integration test.
`test:wrap` runs the soft-wrap tests,
which `test` already contains,
so running both repeats them.
Run only the tests a change can reach.
`test` takes an optional nextest filter expression to select or exclude tests;
`test:native` takes one as well:

```text
mise run //package/desktop-app/ide:test 'test(/wrap/)'
mise run //package/desktop-app/ide:test 'not test(/every_row_of_real_source/)'
```

With a filter the run does not stop at the first failure.
Without one the task runs what it always ran.

The real-source wrap check is six tests,
one per scale (1 and 2) and width (600,
900 and 1400),
named `every_row_of_real_source_fits_width_<width>_at_scale_<scale>`.
It shapes every line of the crate's own `src` at one pair and applies the rule checks to every row,
so its cost grows with the crate.
As one test it took 720 to 750 seconds on a single thread and alone set the wall time of `test` and `test:wrap`.
Most of that was one helper:
`unexcused_breaks` measured the advance of a space for every source line by building a new text shaper,
about 18,000 shapers per pair.
The advance depends only on the scale,
so a pair measures it once,
and a pair takes about 9 seconds with 24 CPUs.
The six tests also run side by side,
and a failure names its pair.
The coverage thresholds that keep the checks from passing without wrapping
(more than 100 wrapped lines and a token wider than a row) hold at width 600 only,
so the two width 600 tests assert them,
which implies the totals the single test asserted.

`.config/nextest.toml` gives the default profile a slow-timeout that kills a hung test after 1800 seconds.
`mise run //package/desktop-app/ide:test:nextest-config` shows in the container that nextest reads that file
from the package directory (the package is its own workspace root)
and that the timeout terminates a test that sleeps for a minute.

### Dependencies optimized in the dev profile

`Cargo.toml` sets `[profile.dev.package."*"] opt-level = 2`,
so every dependency (Slint,
Parley,
Swash,
tree-house,
Helix) is built optimized in a debug build,
and the `ide` crate itself stays at `opt-level = 0`.
The package declares its own `[workspace]`,
so no sibling package shares this profile.
Slint is about twelve times slower unoptimized (the GPU text study,
`doc/planning/ide-vello-text.md`),
which made a debug build show frozen frames that the optimized build does not have.

Measured on 2026-10-19 and 2026-10-20 in the bounded container (21 CPUs,
46 GiB) on a 28-CPU host,
with `bin/inspect-cadence.ts` for the cadence (see [Frame cadence check](#frame-cadence-check)).
Other sessions shared the host,
so the load average at the start of each run is given.

- Clean `build:debug` (empty `target/debug` and `target/runtime-config`,
  so the runtime,
  notices,
  and build steps all run):
  49.7 s at load 1.4 and 48.1 s at load 9.7 before;
  81.8 s at load 2.3 after (a first run at load 55 took 139.5 s and is not comparable).
  `opt-level = 1` took 72.8 s at load 2.7.
  A clean debug build is about 1.6 times slower.
- Incremental rebuild after `touch src/lib.rs` (`mise run --skip-deps`,
  wall time including mise):
  4.5 s,
  4.2 s,
  and 4.4 s before;
  8.9 s (the first build after the clean one),
  3.4 s,
  and 3.7 s after.
  An edit to the application crate costs the same as before.
- `test:native` (214 tests):
  the nextest run took 440.6 s before and 119.6 s and 166.4 s after
  (the second at a load of about 10 from other sessions),
  because the tests that build and shape large texts in Slint and Parley run in optimized code.
- Drag cadence of the debug build on a 489-line file (six runs of `inspect:cadence drag`,
  400 pixels):
  35 and 58 changed frames of 126 and 121 before;
  105 and 105 of 121 after;
  `opt-level = 1`:
  112 and 115 of 118;
  the release build:
  112 of 121 and 109 of 123.
  The debug build with optimized dependencies is not distinguishable from the release build within the spread of the
  release runs (three frames) and of the debug runs before the change (23 frames).
  Levels 1 and 2 are not distinguishable either;
  2 was kept because it is the setting the owner named,
  and `opt-level = 1` is the cheaper build if the clean build time becomes a cost.
- Scroll cadence of the debug build (scroll of 3000 pixels in 120 steps):
  106 and 109 changed frames of 118 and 119 before,
  115 of 116 after,
  115 of 117 in the release build.

The change exposed one test that passed by timing.
`rows_above_the_view_changing_height_move_no_visible_pixel` released the measurement pass and checked at once whether it
had settled;
the check saw an idle worker and returned before the pass had run (extent 10608 before and after,
24504 once measured).
It now waits for the extent to change before the settled check.
The one other test that calls `settled` right after releasing the gate,
`the_extent_settles_as_chunks_arrive`,
has the same shape and passed in every run here.

### Frame cadence check

`mise run //package/desktop-app/ide:inspect:cadence <scenario> <project> <file> [profile] [size] [steps]` records what
the window shows while an input stream runs.
It builds the application binary of the profile (`debug` by default,
or `release`) in the bounded container,
starts it in the repository's nested compositor over a project directory below the system temporary directory
(the same constraint as `inspect:native`,
so language servers never run against a real project),
waits for the window to settle,
starts `record DIR 60 bmp`,
and sends one input command 500 ms later:

- `scroll` sends `scroll X Y 0 3000 steps=120 step-ms=16`,
  a touchpad scroll of 3000 logical pixels over the source text.
- `drag` presses the minimap viewport box 55 pixels from the right edge at y = 60,
  and pulls it 400 pixels down with `drag X 60 X 460 left steps=120 step-ms=16`.

The recorder photographs the compositor at a steady 60 Hz whether or not the application drew,
so a recorded frame equal to the frame before it,
during the input,
means the picture did not move for that tick.
The script hashes the pixels of every frame and prints,
for the frames inside the input window only
(the quiet frames before and after are left out):
the frames compared,
how many changed and how many were identical,
the longest run of identical frames,
every run as `length@first slot`,
and the pattern of the whole recording (`X` changed,
`.` identical,
brackets around the window).
The line starting `CADENCE` repeats the numbers as JSON.
The frames are deleted after hashing;
`IDE_CADENCE_KEEP_FRAMES=1` keeps them.
The recording,
hashes,
screenshots,
and compositor log stay in `~/temp/agent/ide-cadence-*`.

The pure part,
the frame hash and the window and run counting,
is in `bin/cadence.ts`
and tested by `bin/cadence.test.ts` through `mise run //package/desktop-app/ide:test:bin`.
The script `bin/inspect-cadence.ts` only drives the compositor.

Tuning variables:
`IDE_CADENCE_SETTLE` (milliseconds to wait after startup,
default 10000),
`IDE_CADENCE_GRAB` (the drag's first y,
default 60),
`IDE_CADENCE_DISTANCE` (default 400),
`IDE_CADENCE_SCROLL` (default 3000),
and `IDE_CADENCE_STEP_MS` (default 16).

Reading the result:

- Run each build at least twice and call a difference only when it is larger than the difference between two runs of
  one build.
- A drag longer than the minimap box can travel ends in one long identical run at the end of the window,
  because the document cannot scroll further (at a distance of 600 pixels the 489-line file ended in a run of 13 or 14
  frames in the debug and in the release build alike).
  The start slot of each run shows whether a run is at the end.
  Choose a distance the box can travel for the file,
  or read only the runs before the last.
- The positive control is the debug build before the profile change against the release build.
  Before the change,
  `drag` at 400 pixels gave a debug build with 91 identical frames of 126 (runs of up to 9 frames)
  and 63 of 121 (up to 6),
  against 9 of 121 (up to 2) and 14 of 123 (up to 4) in the release build.

### Runtime grammars

The `runtime` task prepares pinned Helix grammars for the measured language inventory,
matching query assets,
and license notices beside built binaries.
It is a build-only operation,
not an application-triggered downloader.
The selection is the languages `tokei` measures as repository source or configuration
that Helix supports at the pinned revision,
plus the companion grammars their queries inject for content the repository contains.
[The runtime language plan][runtime-languages] records the measurement,
each grammar's revision and license,
and language-server presence on this host.
The published `manifest.json` lists the bundled grammars:
a file whose recognized language is not listed stays plain text,
while a listed grammar that fails to load is reported as a broken installation.
The application binary embeds `target/<profile>/runtime` when it is compiled
(see [What the executable carries](#what-the-executable-carries)),
so every task that builds it with the default `gui` feature runs `runtime` first,
and a build without a prepared runtime stops with a message naming that task.

[runtime-languages]: ../../../doc/planning/slint-ide-runtime-languages.md

Helix crates share a pinned upstream revision.
Helix code and runtime assets retain their own license obligations;
the application does not inherit Helix's modal commands or editing features.

### Lint gates

`mise run //package/desktop-app/ide:lint` runs these gates in order:

- Clippy over all targets with warnings denied,
  in the bounded container.
- `cargo fmt --check`,
  in the bounded container.
- The repository's Rust linter (`monochromatic-lint`):
  the line budget,
  the rustdoc requirement,
  Markdown line breaks,
  and the type-aware closure check (section Closures).
- `slint-viewer --check` over every UI file.

`lint:release` checks the release profile and is a separate task.

#### Closures

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) is part of `monochromatic-lint`,
which runs it over this package as `rust/avoidable-closure`.
The IDE task `lint:closures` was removed on 2026-10-19:
the linter now runs the lint for every Rust package,
and the IDE has no task of its own for it.
The linter checks the library and the binaries (`cargo check` without `--all-targets`);
tests,
fuzz crates and bench crates are exempt.
The configuration `monochromatic-lint.config.jsonc` holds the IDE at error severity (block `closure-check-ide`),
because it has no finding left,
while other packages report warnings.
The lint reports a closure unless it captures state
and is passed straight to a callee whose parameter requires `'static`.
Such a callee keeps the state past the call,
so no named function,
loop,
or `match` can replace the closure:
the Slint `on_*` setters,
`Timer::start`,
`thread::spawn`,
and `Builder::spawn` bodies are of this kind and pass.
Every other closure is a named function,
a `for` loop,
a `match`,
or a small named type.
That includes closures written inside macro arguments such as `assert!` and `json_value!`,
which the former syntax-only rule could not see.

A closure that its callee requires,
although the callee's parameter is not `'static`,
carries rustc's own suppression
on the closure argument,
or on the `let` statement that binds it.
Comment directives do not suppress this rule:

```text
// src/syntax_loader.rs
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(avoidable_closure, reason = "a true sentence about why the callee needs a closure")
)]
```

These are the two `catch_unwind` bodies of the minimap and tile workers,
the two `tree_house` callbacks in `src/syntax_loader.rs`,
and the caret callback in `src/native/language/actions.rs`.
`expect` makes rustc report a suppression that no longer suppresses anything (`unfulfilled_lint_expectations`),
and the linter reports that as a finding of the same rule,
at the same severity.
The `check-cfg` entry in `Cargo.toml` that declares the `dylint_lib` name is written by file-enforcer
(`file-enforcer.config.ts`,
`CARGO_DYLINT_DIRS`).

What the IDE needs on the host,
and what the linter does about it:

- The shared nightly of the repository with the `rustc-dev` component,
  and the mise tools `cargo-dylint` and
  `dylint-link`.
  Without them the linter skips every package with one note on standard error.
- The `runtime` and `notices` artifacts,
  `target/debug/runtime` and `target/crate-licenses.json`,
  because the IDE
  build embeds them.
  `lint:clippy` prepares both before the linter runs in `lint`.
  The prerequisite manifest `monochromatic-lint.prerequisites.jsonc` of this package names
  `ide-runtime-artifacts`:
  the linter checks the two artifacts,
  builds in `target/dylint`,
  and links both into it,
  because `build.rs` finds them relative to the build directory and the shared cargo configuration would otherwise
  place that directory under a path hashed from the checkout.
  A missing artifact skips the IDE with a note that names the two tasks.
- The cargo cache.
  The linter never fetches,
  so on a cold cache it skips the package and names `cargo fetch --locked`.
  The nightly cargo of the development host could not connect to crates.io when this check was written
  (`doc/troubleshooting/dylint-library-tests-and-shared-build-dir.md`),
  so fill the cache with `cargo +stable fetch --locked` in this directory.

The removed task `lint:closures` ran `cargo dylint --all-targets` and measured,
on 2026-10-18 with a load average
between 12 and 19 from other work and three runs each:
80 to 87 seconds (median 81.4) with `target/dylint` deleted,
15.0 to 16.9 seconds (median 16.0) with nothing changed,
and 16.5 to 21.6 seconds (median 17.6) after touching `src/lib.rs`.
Those figures cover all targets;
the linter's run covers the library and binaries only.

## Release build and single executable

`mise run //package/desktop-app/ide:build` builds the optimized `monochromatic-ide` binary
in the bounded container described under [Build boundary](#build-boundary).
`mise run //package/desktop-app/ide:bundle` runs that build and copies the binary,
unstripped,
to `dist/monochromatic-ide`:
one executable that carries its language runtime and its license texts
(decided on 2026-10-06:
"Ship as single file").
It runs from any location,
alone,
without the source tree and without `HELIX_RUNTIME` in the environment.
`dist/` is ignored by Git.

Run it as `dist/monochromatic-ide [PROJECT]`,
with `--file FILE` as under [Project startup](#project-startup);
without `PROJECT` it opens the home folder.
`mise run //package/desktop-app/ide:run:bundle [PROJECT]` does the same from the package directory.
As in the sibling applications,
`build` is the release build and `build:debug` the debug build;
`run` starts the debug binary that `build:debug`,
the tests,
and the inspection tasks use.
`mise run //package/desktop-app/ide:lint:release` checks the application binary under the release profile,
where code behind `cfg(debug_assertions)` is absent and unused items show only there.
The release build unsets `SLINT_EMIT_DEBUG_INFO`,
which the debug tasks set so the native tests can address interface elements by name,
and builds only the application binary,
not the runtime,
inspection,
and scripted-server helpers.

### Renderer and GPU

Nothing in the IDE paints on the CPU (owner decision of 2026-10-19),
so the application has one renderer:
Slint's FemtoVG drawing through wgpu 30 on the Vulkan backend (`renderer-femtovg-wgpu`,
design C of `doc/planning/ide-vello-text.md`).
There is no OpenGL renderer and no software renderer in the executable,
and no fallback to either:
`mise run //package/desktop-app/ide:inspect:no-software-renderer` fails if Slint's software renderer is linked again.
A host without a Vulkan driver for a hardware GPU therefore cannot start the IDE
(see below for the message it gets).

`src/launcher.rs` opens the GPU itself and gives it to Slint,
because Slint's own adapter choice cannot be trusted with that rule:
with no power preference wgpu does not sort its adapters,
so the first one the Vulkan loader lists wins,
and that may be the CPU adapter lavapipe (`llvmpipe`).
`src/gpu_device.rs` lists the Vulkan adapters,
keeps the discrete and integrated GPUs only,
refuses every other kind (CPU,
virtual,
unknown),
and requests a device with wgpu's full default limits raised to the adapter's image-size limits
(Vello's compute shaders need storage buffers,
which Slint's downlevel default limits forbid).
The native test harness (`src/native/gpu_snapshot.rs`) opens its device through the same function.

#### Which GPU

The IDE is a read-only editor,
so it does not rank a discrete GPU above an integrated one and does not wake
a second GPU on its own.
The hardware adapters stay in the order the Vulkan loader lists them,
and the first one that accepts the device wins.
That order is the system's choice:
Mesa documents that without `DRI_PRIME` the default GPU is the one the display server uses,
and that `DRI_PRIME` and `MESA_VK_DEVICE_SELECT` put the selected GPU first in the Vulkan physical device list.
Desktops set these per application through switcheroo-control,
so an entry such as "launch using discrete graphics card" reaches the IDE as a reordered list.

Two variables that wgpu documents are read as explicit choices and win over the loader's order:

- `WGPU_ADAPTER_NAME` keeps the hardware adapters whose name contains the value,
  ignoring letter case
  (`radv navi33` matches `AMD Radeon RX 7600 (RADV NAVI33)`).
  A value that matches no adapter,
  or only adapters of a refused kind such as `llvmpipe`,
  stops the IDE with a message
  that names the adapters seen.
  If the adapters it matches refuse the device,
  the IDE does not move on to an adapter the name did not select.
- `WGPU_POWER_PREF` is `low` (integrated GPUs first),
  `high` (discrete GPUs first),
  or `none` (the loader's order),
  ignoring letter case,
  as wgpu 30 accepts them.
  Any other value stops the IDE with a message that names the accepted values (wgpu itself ignores such a value).
  The other kind is not dropped;
  it stays behind as the next adapter to try.

When both are set,
the name filters and the power preference orders what is left.
An empty value counts as unset.
A CPU,
virtual,
or unknown adapter is never a candidate,
whatever the variables say.
These two variables are the whole interface:
the IDE has no command-line option for the GPU.

The launcher logs the chosen adapter and the reason at info level (`RUST_LOG=info`),
for example
`the window is drawn on this GPU adapter=AMD Radeon RX 7600 (RADV NAVI33) (DiscreteGpu, radv)`
`why=no variable reordered the adapters, so the system's choice applies: the Vulkan loader's order ...; candidate 1 of 2`,
with the device's storage-buffer and texture limits.
The reason is either the loader's position of the adapter or the variable that decided.

Measured on 2026-10-20 on a desktop with an RX 7600 (discrete),
the Ryzen 9 7950X's Raphael graphics (integrated),
and lavapipe,
with the IDE hosted in the nested compositor
(`DRI_PRIME` and `MESA_VK_DEVICE_SELECT` set for the IDE only,
not for the compositor):

- Nothing set:
  the loader lists the RX 7600 first,
  and the IDE draws on it.
- `DRI_PRIME=1` or `MESA_VK_DEVICE_SELECT=1002:164e`:
  the loader lists Raphael first,
  and the IDE draws on it.
- `WGPU_POWER_PREF=low`:
  Raphael,
  whatever the loader lists first;
  the window and the code view draw.
- `WGPU_POWER_PREF=high` with `DRI_PRIME=1`:
  the RX 7600,
  which the variable moves in front of the loader's first.
- `WGPU_ADAPTER_NAME="RADV NAVI33"`:
  the RX 7600 only.
- `VK_DRIVER_FILES` naming the lavapipe driver file:
  no hardware adapter,
  exit status 1 with the message of the section "When no GPU can be used".

A hybrid laptop and NVIDIA's proprietary driver were not available,
so their loader order is not measured here.

#### When no GPU can be used

A host with no hardware Vulkan adapter stops at startup with a message that names the adapters it saw,
in the loader's order,
and every remedy:
install the Vulkan driver of the GPU and the Vulkan loader,
give the process access to the render node (`/dev/dri/renderD*`),
check `vulkaninfo --summary`,
and check `VK_DRIVER_FILES` and `VK_ICD_FILENAMES`.
It also says that a CPU adapter is refused on purpose and that `SLINT_WGPU_CPU` does not allow one.
Every message about the choice (no hardware adapter,
an unmatched name,
a refused device) also gives the order used,
the two variables as read,
and how to choose a GPU:
`WGPU_ADAPTER_NAME`,
`WGPU_POWER_PREF`,
`DRI_PRIME`
(for example `DRI_PRIME=1`),
and `MESA_VK_DEVICE_SELECT` (`vendor_id:device_id`).
Both cases were run on 2026-10-20 with `VK_DRIVER_FILES` set to the lavapipe driver file (the message lists
`llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, llvmpipe)`) and to a missing file (the message says the loader found no driver).

`SLINT_BACKEND` is read as Slint reads it (lowercase,
split at the first dash):
unset,
`winit`,
`gl`,
and `winit-femtovg-wgpu` start the application as described above;
another backend (`qt`,
`linuxkms`) is left to Slint's own selector and gets no app id,
no adapter policy,
and,
for any backend that draws on the CPU,
no protection from this rule;
a winit renderer that is not in this build (`femtovg` is the OpenGL one,
and `skia`,
`software`,
`vello`)
is an error that names the accepted spellings,
because Slint would otherwise warn and draw with its default renderer on a device it chose.
A backend that is not winit with FemtoVG on wgpu does not read `WGPU_ADAPTER_NAME` or `WGPU_POWER_PREF` through
this program;
the adapter policy above applies only to the path the launcher opens.

The launcher keeps one extra handle of the wgpu instance,
adapter,
device,
and queue until the process ends.
Slint frees its handles while the main thread's thread-local values are destroyed,
and a debug build of wgpu-core reads a thread-local value when it frees a device,
which aborted the process at exit before this (see `doc/planning/ide-gpu-snapshot-harness.md`,
section "Stage 2").

### The code view on the GPU

The text area of a file is drawn by the GPU with no CPU painting:
no pixel buffer is filled and uploaded,
and no software renderer is involved.
The CPU lays out and shapes rows,
chooses inks,
and builds a list of paint commands;
the GPU turns outlines and paths into pixels.

- `src/tile_scene.rs` turns the shaped rows of a tile (`ShapedView`) into paint items (`tile_items`):
  one glyph run per Parley run in the ink of its syntax role,
  the selected part of a row as a second copy of its runs in the selection ink clipped to the selection
  (a ligature half is clipped by geometry,
  never recolored whole),
  the texts of hint and message rows in their severity ink,
  and each diagnostic underline as a stroked path
  (`src/tile_scene/underline.rs`:
  a triangle wave for errors,
  dashes for warnings,
  dots for information,
  sparse dots for hints).
  One shaper measures and draws:
  the glyph ids and positions are those of the layout that places the caret and answers hit tests.
- `src/tile_scene/encode.rs` writes the items to a Vello scene (`vello` 0.11).
  Glyphs are drawn unhinted at the positions the layout gave them.
  The scene is plain data and is built on whichever thread builds the tile:
  the tile worker (`src/native/tile_build/job.rs`) or the window thread.
- `src/native/gpu_tile.rs` owns the textures and the window's one rendering notifier.
  On the first frame (`RenderingSetup`) it takes Slint's wgpu device and makes Vello's renderer;
  before every frame (`BeforeRendering`) it renders a waiting scene into an `Rgba8Unorm` texture and,
  when the tile's size changed,
  shows that texture as the window's source image with `Image::try_from(texture)`.
  The texture holds straight alpha,
  which is how Slint samples an imported texture,
  so the scene is rendered over a transparent base and the selection fills and find boxes behind it show through.
  Scrolling inside the tile renders nothing:
  the image moves,
  and no scene is built or drawn.
  The wrap indicator of the gutter is a second,
  small texture drawn the same way.
- Selection fills and the boxes of find matches are Slint rectangles behind the tile image (`ui/app.slint`),
  drawn by FemtoVG on the GPU as before.
  Their edges and the clip edges of the selected ink lie on whole pixels,
  so no pixel is covered by two copies of a glyph.

The tile's texture is bounded by the device.
`src/shaped_text/limit.rs` holds the largest texture side (`texture_dimension`,
8192 until a device says otherwise),
which `src/gpu_device.rs` records when it opens the device and `src/native/gpu_tile.rs` checks at the first frame.
`tile_cap` (the tallest tile in logical pixels) is bound by the stack (`stack_rows`,
textures of `tile_texture_rows`),
one code row short for rounding,
and a tile of lines taller than that is clipped as described in "Lines taller than one raster".
`paint_tile` refuses a view past one texture and `paint_stack` a view past the stack with a diagnostic
instead of letting the GPU reject the texture.
The native test device is limited to 8192 px (wgpu's guaranteed minimum),
so every test run proves the rule on this host too,
whose own GPU offers 16384.
In a window 2,000 px tall over `mixed(6000)` lines at a text width of 1,100 px
(`a_2000_px_window_gets_a_tile_of_textures_within_the_devices_limit_at_both_scales`),
three views are 6,048 physical rows at scale 1,
a texture of 1113 by 6048 px (the side of 8192 holds it),
and 12,096 rows at scale 2,
two textures of 2226 px by 7168 and 4928 rows (7168 is 28 bins of 256 high,
the most that 9 bins across allow).

Stages 5 and 6 of `doc/planning/ide-vello-text.md` are done:
the minimap is drawn by the same renderer,
and `src/text_raster.rs`,
`src/glyph_cache.rs`,
`src/selection_paint.rs`,
the pixel code of `src/annotation_paint.rs`,
and the `swash` dependency are deleted.

The native tests read the GPU,
not the CPU:
`src/native/tile_reads.rs` renders a frame and reads the tile's texture back (`tile_pixels`,
`shown`,
`sign_pixels`),
or renders a scene without a window (`rendered_tile`),
and `tests/` read the paint items through `tile_scene::read` (`drawn_runs`,
`drawn_strokes`).
Two renders of one scene may differ by one level in a few texels on the GPU (`tile_reads::RENDER_NOISE`),
so tests that compare two renders accept one level and each has a control that differs by far more.

### Inspecting the running window

No build of the IDE contains Slint's MCP server (`--features slint/mcp`,
removed on 2026-10-20 on the owner's answer).
Its screenshot tool draws with Slint's software renderer,
so any build that carried it linked the CPU renderer that this package forbids.
Every check that looks at the window now does so through the repository's nested compositor
(`package/cli/nested-wayland-session`),
which composites the IDE's frames on the GPU and gives the checks pictures,
input,
an element tree,
and the log:

Pictures come from the control command `screenshot PATH` (a PNG of the latest committed frame)
and `record DIR FPS bmp` (a steady 60 Hz sequence,
as `inspect:cadence` uses).

Input comes from `click`,
`move` (a pointer move with no button,
which gives the window enter,
motion,
and leave events and so its hover state),
`button down|up`,
`drag`,
`scroll`,
`wheel`,
`key`,
and `type`.
The MCP tool `move_pointer` is `move`.

The element tree comes from the command `a11y`,
which prints one record per accessible element of the window
(`depth`,
`role`,
`name`,
`states`,
screen `bounds`),
read over AT-SPI from the tree Slint's `accessibility` feature publishes through AccessKit.
The compositor serves that bus only when started with `--accessibility`,
which `inspect:native` does;
the measuring tasks leave it off so they time the application as shipped.
On 2026-10-20 it returned 40 elements for a project with five tree rows:
the application,
the frame,
the `Project files` landmark,
the tree and each row by file name with its `selected`
and `focused` states and bounds,
the `Sidebar width` slider,
the file path label,
and the `Source code` entry.
After a click on a row,
a second `a11y` showed that row as `selected` and the screenshot showed its file.

State the window does not draw comes from the IDE's own log (`RUST_LOG=ide_app=debug,monochromatic_ide=debug`),
which `inspect:idle-cost`,
`inspect:watch-limit`,
`inspect:log-stall`,
and the bundle checks read.

What the server gave that these do not:
reading any property of any Slint element by its element id,
and invoking any callback by id.
`a11y` shows only what the toolkit publishes as accessible (role,
name,
states,
bounds),
and the compositor has no command that invokes an element's default action.
No script in this package used those two tools;
the native tests that need them use `ElementHandle` of `i-slint-backend-testing` in the test process.

`mise run //package/desktop-app/ide:inspect:no-software-renderer` runs `cargo tree` over every feature of the package
and fails if the software renderer is in the graph.
It also runs the same query with `--features slint/renderer-software` and with `--features slint/mcp`,
and fails unless both reach the renderer,
so an empty answer cannot come from a query that cannot see it.
Every query runs for both release targets,
`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`,
on whichever of the two hosts runs the task,
because platform-specific dependencies differ per target.

### What the executable carries

`build.rs` embeds these files of `target/<profile>/runtime` and of the package,
for every build with the `gui` feature,
the debug binary included,
and records each one's path,
bytes,
and FNV-1a digest (`src/content_digest.rs`) in a table sorted by path:

```txt
# Paths inside the embedded table of monochromatic-ide, built from target/release/runtime
runtime/manifest.json            pinned Helix revision and the bundled grammar libraries
runtime/grammars/<name>.so       one parser library per bundled grammar, only those the manifest lists
runtime/queries/<language>/      Helix query files of every language at the pinned revision
runtime/licenses/<grammar>/      license notice of each bundled grammar
runtime/Helix-LICENSE            MPL-2.0 text for the Helix query files and the Helix crates in the binary
LICENSES/                        LGPL-3.0-or-later and GPL-3.0-or-later texts of the application
LICENSES/font/                   SIL Open Font License notices of Inter and JetBrains Mono
LICENSES/crates.json             license texts of the Rust crates, collected by cargo-about
```

On 2026-10-06 that was 1,254 files and 34,659,293 bytes:
27 parser libraries (33,463,328 bytes,
from 11,098,288 for `sql.so` down to 20,464 for `json.so`),
1,193 query files of 341 languages (1,087,532 bytes),
28 grammar notices (39,593 bytes),
Helix's license,
the two application license texts,
the two font notices,
and the manifest.
Inter and JetBrains Mono themselves are compiled in through Slint,
as before.
`monochromatic-ide --licenses` prints the embedded license and notice texts
(decided on 2026-10-06:
"--licenses flag";
`src/runtime/notices.rs`):
every file below `LICENSES/` and `runtime/licenses/`,
and every other file whose name contains `LICENSE` or `LICENCE` or starts with `COPYING` or `NOTICE`,
which adds `runtime/Helix-LICENSE` and `runtime/queries/snakemake/LICENSE`;
34 files on 2026-10-06.
The two read-me files among the queries (`ecma/README.md`,
a description of query inheritance,
and `ripple/readme.md`,
a source link) are not license terms and are left out.
Each text comes in full under a framed heading that names its component and its embedded path,
for example `Language grammar rust: LICENSE` above `Embedded as runtime/licenses/rust/LICENSE`.
Every text is digest-checked before anything is printed,
so a damaged executable prints the damage message and exits with status 1 instead of a partial list;
a reader that closes the pipe early (`| head`) ends the listing with status 0.
After these files the listing prints the license texts of the Rust crates,
one entry per distinct text
(see [Rust crate license texts](#rust-crate-license-texts));
with them it counted 231 texts and 581,919 bytes on 2026-10-06.
The table's key (`c47e913b79bf6a42` for that runtime) is a digest of every path and file digest.

### Rust crate license texts

The user decided on 2026-10-06 to collect every Rust crate's license text with a notice generator,
and narrowed the choice the same day:
"No need to consider any alternatives.
Just it."
(cargo-about).

#### What was established before wiring it in

- Version:
  cargo-about 0.9.2,
  the latest release (2026-08-18).
  Its crates.io archive (sha256 `0cd19d99696eb83f0a2d6ab7a347b14968d2980416c8cca827ded220e6e9c4bb`)
  names commit `f7394d5c8f618623573072caadf6594821c789b6` in `.cargo_vcs_info.json`,
  and its files equal that tag's apart from Cargo's normalized manifest.
  The command needs the `cli` feature:
  `cargo install --locked --features cli --version 0.9.2 cargo-about`
  (without `--features cli` the install compiles for minutes and then installs nothing).
  Its book documents `-L, --log-level`;
  0.9.2 accepts only `-L`.
- License:
  cargo-about is MIT OR Apache-2.0 and runs only while building;
  none of its code is compiled into the executable.
  What the executable gains is each crate's own license file,
  or,
  when a crate ships none that cargo-about recognizes,
  the standard text of the chosen license from the SPDX License List data compiled into cargo-about
  (`spdx` 0.13.4).
- Offline:
  with `--frozen` (`--locked` plus `--offline`) cargo-about creates no HTTP client
  (`src/cargo-about/generate.rs`,
  the `client` binding after "gathered {} crates"),
  so license files that a configuration would fetch from a crate's git repository are not used,
  and Cargo resolves the graph from the local registry.
  Its license store is compiled in (`Store::load_inline` in `src/licenses.rs`).
  The crate sources come from the `ide-cargo` volume that the `fetch` task fills.
  Installing the tool is the one step that needs the network,
  so it belongs to the image build (`Containerfile`),
  as `rustfmt` already does.
  The trial ran with `--network=none`,
  the package and `ide-cargo` mounted read-only,
  and exited with status 0.
- Trial against the IDE at commit `4b8b03663`
  (target `x86_64-unknown-linux-gnu`,
  build and dev dependencies ignored,
  the unpublished package itself ignored):
  - 424 crates,
    6 of them Helix crates from git.
    `cargo tree --edges normal,no-proc-macro` names 369 crates besides the IDE linked into the executable,
    all among them;
    the other 55 are procedural-macro crates and their dependencies,
    which run only while compiling.
    The 187 registry crates whose source paths appear inside that release executable are all among them too.
  - 29 distinct license expressions.
    The most common:
    `MIT OR Apache-2.0` (188 crates),
    `MIT` (82),
    `Apache-2.0 OR MIT` (47),
    `Unicode-3.0` (25),
    `Apache-2.0` (15),
    Slint's `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` (10),
    and `MPL-2.0` (9).
    Unusual spellings parse as well:
    fnv's `Apache-2.0  OR  MIT`,
    smartstring's deprecated `MPL-2.0+`,
    and unicode-ident's `(MIT OR Apache-2.0) AND Unicode-3.0`.
  - No crate it could not resolve:
    `--fail` exited with status 0.
    As a positive control,
    the same run without `GPL-3.0-only` among the accepted licenses failed with status 1,
    naming all ten Slint crates.
  - Two runs gave byte-identical output;
    one took 55.7 s on 2 CPUs.
  - cargo-about reads no NOTICE files.
    In the fetched registry only `cfg_aliases` ships one (`NOTICES.md`),
    and it is a build dependency outside the graph.

#### How the build carries them

- `Containerfile` installs cargo-about 0.9.2 into the build image
  (`mise run //package/desktop-app/ide:image`).
  The `notices` task first checks `cargo-about --version` and,
  in an image built before this change,
  stops and names the `image` task.
- `notices` runs after `fetch`:
  cargo-about's `generate` with `--frozen`,
  `--fail`,
  `--config about.toml`,
  and `--output-file target/crate-licenses.json`,
  on the template `about.json.hbs`.
  It keeps the file while a SHA-256 over `Cargo.lock`,
  `Cargo.toml`,
  `about.toml`,
  `about.json.hbs`,
  and the cargo-about version is unchanged (`target/crate-licenses.inputs`).
  Every task that builds the application binary with the `gui` feature depends on it;
  `build.rs` embeds the file as `LICENSES/crates.json`
  and stops with a message naming the task when it is missing.
- `about.json.hbs` writes one entry per distinct license text:
  its SPDX identifier,
  its name,
  the crate file it came from (null for a standard text),
  the crates it covers,
  and the text.
  `src/runtime/crate_licenses.rs` reads the list,
  and `--licenses` prints every entry after the license files,
  under a framed heading such as `Rust crates under MIT License (MIT): 18 crates`,
  then the crates by name and version wrapped to 78 columns,
  then `Text from the crate file annotate-snippets-0.12.16/LICENSE-MIT`
  or `Text: the standard text of this license (no crate file was recognized)`.
- `about.toml` accepts,
  in this order of preference for crates offered under several licenses:
  MIT,
  Apache-2.0,
  ISC,
  BSD-2-Clause,
  BSD-3-Clause,
  Zlib,
  Unicode-3.0,
  MPL-2.0,
  CC0-1.0,
  Unlicense,
  GPL-3.0-only.
  A dependency under any other license stops the task until it is reviewed.
- On 2026-10-06 the list held 197 texts for 424 crates,
  193 of them read from the crates' own files,
  402,423 bytes of text in a 461,974-byte file.
  Standard texts cover 12 MIT crates,
  5 Apache-2.0 crates,
  the ten Slint crates (GPL-3.0-only;
  the `slint` crate keeps that text in a `LICENSES/` folder,
  which cargo-about did not pick),
  and five Helix crates (MPL-2.0;
  Helix keeps its license file only at its repository root,
  which `runtime/Helix-LICENSE` already carries).
  Preferring Apache-2.0 over MIT gave 117 texts of 621,736 bytes,
  66 crates on the standard Apache-2.0 text;
  preferring BSD-3-Clause over Apache-2.0 gave 224 texts,
  because cargo-about then took every BSD header among the source files of moxcms and pxfm as its own text.
  The task took 99 s with its `fetch` step.
  On the release executable of commit `9d476f838` (83,362,368 bytes),
  the `crate-licenses` bundle check found all 369 linked crates
  and all 291 registry crates whose source paths the executable contains among the crates printed.
- The `linked-crates` task writes `target/linked-crates.txt`
  from `cargo tree --frozen --edges normal,no-proc-macro` for the target the build container compiles natively
  (`x86_64-unknown-linux-gnu` on an x86_64 host,
  `aarch64-unknown-linux-gnu` on an arm64 host)
  for the bundle checks.
- `about.toml` lists both release targets,
  so both executables embed the same license list.
  Measured on 2026-10-22,
  the aarch64 graph is a subset of the x86_64 graph (448 crates against 449;
  `core_detect` is the one crate only x86_64 links),
  and the list `cargo-about` printed for both targets was byte for byte the one it printed for x86_64 alone.

#### Choices made by default and open to veto

- MIT ahead of Apache-2.0 in the accepted list,
  because the crates' MIT files carry the copyright lines that license asks to reproduce,
  and the list is smaller.
- Slint under `GPL-3.0-only`,
  whose text the executable already carries,
  rather than the royalty-free license,
  which asks for Slint's attribution (the `AboutSlint` widget in an About screen,
  or a badge on a public page).

### Where language files come from

Highlighting reads only the application's own runtime,
whatever Helix's loader would choose.
That loader (`prioritize_runtime_dirs` in `helix-loader/src/lib.rs`) searches,
for each file,
`runtime` beside `CARGO_MANIFEST_DIR`,
then `$XDG_CONFIG_HOME/helix/runtime`,
then `HELIX_RUNTIME`,
then a build-time directory,
then `runtime` beside the executable,
and takes the first that has it;
a file in the user's Helix configuration replaced a bundled query
(on 2026-10-05 an empty `queries/sql/highlights.scm` there drew SQL with 0 colored spans instead of 16).
So the application no longer uses Helix's `Loader` to compile languages:
`src/syntax_loader.rs` implements tree-house's `LanguageLoader` itself,
keeps Helix's built-in language table only for recognition,
and reads grammars and queries through `src/runtime.rs`:

- The application binary installs its embedded table at startup.
  Queries are read straight from it.
- A parser library must be a file for the dynamic loader,
  so the first time a language is needed its library is unpacked into
  `$XDG_CACHE_HOME/monochromatic-ide/runtime/<key>/grammars/<name>.so`
  (`~/.cache` when the variable is unset),
  directories mode 0700,
  files mode 0600.
  Only the languages of displayed files (and of their injections) are unpacked.
- Every load first checks the embedded bytes against their build-time digest,
  then compares the cached file with them byte for byte.
  A missing,
  shortened,
  or altered cached file is written again and logged as a warning;
  a damaged embedded part is reported under the source
  ("The file runtime/grammars/sql.so embedded in … is damaged …"),
  the file stays readable as plain text,
  and nothing is unpacked for it.
- When the highlighting engine cannot start at all,
  a file no language applies to,
  such as a `.txt` file,
  is plain text without a message or a warning;
  whether a language applies is decided by the compiled-in filename and shebang rules,
  which need no runtime.
- A write goes to a private file named after the process and a counter,
  then is renamed over the final name.
  The rename replaces the directory entry in one step,
  so concurrent first starts both write identical bytes,
  a reader sees either complete file,
  and a process that already loaded the old file keeps it.
- Folders of other builds go once they have been unused for 30 days
  (decided on 2026-10-06:
  "Remove after N days unused",
  with 30 days;
  `src/runtime/retention.rs`).
  At start,
  before any window,
  the application writes a fresh `<key>/last-used` into its own folder,
  then removes every other `<key>` folder whose last use lies more than 30 days back.
  Last use is the modification time of that marker,
  which a copy renews at every start and every parser load,
  so an older copy that keeps running keeps its folder;
  a folder without a marker,
  as builds before this rule left,
  is aged by its own modification time.
  A copy built before this rule renews nothing,
  so its folder can go while it runs:
  the libraries it already loaded stay mapped,
  and its next load unpacks the library again into a new folder.
  Only real folders named by 16 lowercase hexadecimal digits directly below `runtime/` are candidates;
  the current key,
  files,
  symbolic links,
  and other names stay,
  nothing is removed when `runtime/` itself is a symbolic link,
  and `remove_dir_all` removes links inside a folder without following them
  (its documentation in the Rust standard library).
  A folder is renamed to `.<key>.removing-<process>-<n>` before it is removed,
  so a removal cut short leaves only that name,
  which the next start removes.
  Failures are logged and skipped.

Test and development programs have no embedded table;
they read the directory `HELIX_RUNTIME` names,
which the package `runtime` task prepares,
again through `src/runtime.rs` and never through Helix's search.
Their messages name that task;
the executable's messages say to replace it with a fresh copy or build it again from source.
A build with the `gui` feature stops with a message naming the `runtime` task
when the prepared runtime lacks its manifest or a grammar library the manifest lists
(both observed in a disposable copy on 2026-10-06).

### What the host provides

- Linux on x86_64 or on aarch64 (ARM64) with glibc 2.39 or later
  (the release builds both;
  only the x86_64 build has been started on real hardware,
  see [Releases](#releases)),
  `libstdc++`,
  `libgcc_s`,
  and `libfontconfig`.
  The build container is Fedora 41.
- A Wayland session,
  with `libwayland-client`,
  `libwayland-egl`,
  `libxkbcommon`,
  and `libEGL`,
  which the binary opens when it starts.
  Only Wayland was checked.
- A writable private cache directory for the unpacked parser libraries,
  on a file system that allows running code (a `noexec` mount refuses them,
  and the message names `XDG_CACHE_HOME` as the way out).
- `rg` (ripgrep) on `PATH` for [Combined search](#combined-search).
- For language features:
  `/usr/bin/bwrap` with unprivileged user namespaces,
  and each language's server on `PATH`,
  such as `rust-analyzer`.
  TypeScript uses the project's own `node_modules/typescript` at version 7 or later,
  started through `node`.
  Without one of these the reader still works,
  and a language action explains what is missing;
  see [Language module](#language-module).
- No fonts:
  with the system copies of both families hidden,
  the packaged binary drew the interface and the source in its own faces.

The application writes language-server state and unpacked parser libraries below
`$XDG_CACHE_HOME/monochromatic-ide`,
and nothing below the configuration or data directories.
It logs to standard output,
warnings and errors by default and more with `RUST_LOG` (see "Log").

### Install and launcher entry

`mise run //package/desktop-app/ide:install` runs `bundle`,
then `bin/install.ts`,
which copies `dist/monochromatic-ide` to `~/.local/bin/monochromatic-ide` (mode 0755)
and `share/applications/monochromatic.ide.desktop` to `${XDG_DATA_HOME:-~/.local/share}/applications`,
each through a private file renamed into place,
then refreshes that folder's `mimeinfo.cache` with `update-desktop-database` when it is installed.
It was verified only with disposable `HOME` folders,
never against the real home.

The entry follows the sibling entries in `package/desktop-app/terminal/share/applications/`
and `package/music-player/desktop-app/share/applications/`:

- `Exec=monochromatic-ide %f`.
  The Desktop Entry Specification's `%f` is "a single file name (including the path)",
  and "if the application should not open any file the %f,
  %u,
  %F and %U field codes must be removed
  from the command line and ignored",
  so an application menu starts it with no argument (the home folder)
  and "open with" on a folder passes that folder's path.
  `%u` could pass a `file:` URL,
  which the command line does not accept.
- `MimeType=inode/directory;` offers it for folders.
  With the entry installed in a disposable home and `~/.local/bin` on `PATH`,
  `gio mime inode/directory` listed it among the registered and recommended applications
  while the default stayed `org.kde.dolphin.desktop` from the system `kde-mimeapps.list`.
  GLib lists the entry only when the command is found on `PATH`;
  this host's systemd user session `PATH` includes `~/.local/bin`.
- `StartupWMClass=monochromatic.ide`,
  the Wayland app id that `src/launcher.rs` stamps on the window
  through Slint's winit backend hook,
  as the sibling `launcher.rs` files do.
  Runs with a non-winit `SLINT_BACKEND` keep Slint's own backend and get no app id;
  `inspect:native` runs the same backend and gets the app id
  (it showed `set_app_id("monochromatic.ide")` in the Wayland log on 2026-10-20,
  and the compositor's `windows` command lists `app-id=monochromatic.ide`).
- `Icon=accessories-text-editor`,
  a stock name of the Icon Naming Specification present in this host's breeze,
  breeze-dark,
  and AdwaitaLegacy themes (Adwaita has only its symbolic variant);
  the application's own icon is still the user's decision.
- `desktop-file-validate` from desktop-file-utils 0.28 accepts the entry.

In the nested compositor with a disposable home,
opening a folder through GLib (`gio launch` with the entry and the folder)
and through KIO (`kioclient exec` with the folder URL and `inode/directory`,
the IDE made the default for folders in that home only)
both started the installed copy with the folder as its only argument and on that folder;
`gio launch` with the entry alone started it with no argument on the home folder.
The Wayland protocol log of the window showed `set_app_id("monochromatic.ide")`
and `set_title("Monochromatic IDE")`.

### Home folder as the project

Starting without a folder opens `HOME`.
The tree's header names the folder (the last path component),
the window title stays `Monochromatic IDE`,
and the source column is empty until a file is chosen.
What changes:

- Language-server state lies inside the project,
  below `~/.cache/monochromatic-ide/language`,
  which the confinement now allows,
  binding it writable after the read-only project
  (see [Language module](#language-module)).
  A real bubblewrap probe in a disposable home wrote into the state folder and was refused beside it;
  with the former order the state write was refused too.
- A loose file directly in the home folder,
  opened without a project folder argument,
  is a user error that gets no special handling (decided on 2026-10-06):
  like any file with no root marker of its language between itself and the project root,
  its server gets the project root,
  here the whole home folder,
  as workspace.
  With `scratch.rs` directly in the disposable home of 283,450 files,
  `rust-analyzer` started confined with that workspace,
  its state below the home folder's `.cache`,
  logged that it found no project there,
  held 47,092 KiB of resident memory 20 s later,
  and answered a hover request with "No hover information at this position.";
  the window closed cleanly with nothing left running.
- The tree reads only the folders shown,
  the change watcher watches only those (one inotify watch at start),
  and search runs `rg` over the home folder,
  which skips hidden folders and,
  inside git repositories,
  ignored ones.

### Bundle checks

`mise run //package/desktop-app/ide:inspect:bundle [file] [runtime] [only]` checks a single executable,
`dist/monochromatic-ide` by default,
against the runtime directory it was built from,
`target/release/runtime` by default,
without changing either:

- `inventory`:
  a regular ELF executable that contains every file `build.rs` embeds,
  byte for byte.
- `license-texts`:
  the application's license texts,
  both font notices,
  Helix's license,
  and a notice with a copyright line for every bundled grammar,
  each embedded byte for byte;
  and `--licenses`,
  run with an empty environment,
  exits with status 0,
  writes nothing to standard error,
  and prints exactly the files its selection rule names,
  each in full under its heading.
- `lone-copy-highlights`:
  a copy alone in its own folder reports its version with an empty environment,
  then highlights a SQL file in the nested compositor with empty XDG homes,
  unpacking exactly the runtime's `sql.so` and no partial file.
- `later-start-reuses-cache`:
  a second start on the same cache compares the cached parser and leaves the file untouched.
- `damaged-cache-rebuilt`:
  a cached parser with one changed byte is reported,
  written again,
  and highlighting works.
- `shadowing-query-ignored`:
  an empty `helix/runtime/queries/sql/highlights.scm` in the configuration home changes no span.
- `concurrent-first-starts`:
  two copies started at the same moment on one empty cache both highlight
  and leave one intact parser.
- `damaged-embedded-part-reported`:
  a copy with one changed byte inside its embedded `sql.so` keeps the source readable,
  names the damaged part,
  the executable,
  and the remedy,
  and unpacks nothing.
- `old-cache-folders-removed`:
  a start on a cache seeded with a key folder last used 31 days ago,
  one used a day ago,
  and the rest of a cut-short removal
  removes the first and the third,
  keeps the second,
  and renews its own folder's marker,
  before its display connection fails on purpose.
- `crate-licenses`:
  `--licenses` prints one section per entry of `target/crate-licenses.json`,
  with the same license,
  the same crates in the same order,
  and the text in full;
  every crate in `target/linked-crates.txt` (the `linked-crates` task,
  which both bundle tasks run first)
  and every registry crate whose source path the executable itself contains is among the crates printed.

The other startup checks need a Wayland session and the release build of `package/cli/nested-wayland-session`;
SQL is the sample because it has a bundled grammar and no configured language server.
Each requires a clean end:
the application exits with status 0 within the compositor's 2 s after the close request,
and the session leaves no application process,
no private bus daemon,
and no bus directory behind.
A frame is taken only once the window shows more than eight colors.
`RUST_LOG` is given to the application alone,
because the compositor reads it too.

`mise run //package/desktop-app/ide:inspect:bundle-guards [file] [runtime] [only]` damages one copy per case
and requires the matching checks to fail on it while the unrelated ones still pass:
a cleared executable bit,
and one changed byte inside the embedded SQL highlighting rules,
the SQL parser,
a grammar notice,
a font notice,
an application license,
Helix's license,
and the Rust crate license list.
`license-texts` fails on every one of these but the SQL cases:
a copy without its executable bit cannot run `--licenses`,
and a copy with a damaged text exits with status 1 and the damage message naming that text;
on the damaged crate list `crate-licenses` fails the same way.
A damaged file cannot show that `crate-licenses` notices a crate missing from an intact list,
so that was checked on an altered build (on 2026-10-06):
with `ignore-transitive-dependencies = true` added to `about.toml`,
the list held 21 crates,
and `crate-licenses` failed on the rebuilt debug executable
("348 of 369 linked crates have no license text").
In the same disposable copy,
a debug build without `target/crate-licenses.json` stopped with status 101
and the message naming the `notices` task.

Behavior cannot be removed from a finished file,
so the run-time checks were also run on an altered debug build (on 2026-10-06):
with the cache compared by length only,
`damaged-cache-rebuilt` failed ("the damaged cached parser was not reported and written again");
with a query file below `$XDG_CONFIG_HOME/helix/runtime` read first,
`shadowing-query-ignored` failed (0 spans with the user query);
with the digest check skipped,
`damaged-embedded-part-reported` failed ("the damaged copy reported nothing").
`lone-copy-highlights` passed on the same build.
The unit and integration tests behind these rules were observed failing in a copy with each guard removed:
`tests/syntax_shadowing.rs`,
the cache tests in `src/runtime/cache_tests.rs`
(`a_rewrite_replaces_the_file_instead_of_writing_into_it` fails with writes into the file in place,
which `concurrent_first_uses_all_get_the_complete_library` did not always detect),
the digest test in `src/runtime/embedded_tests.rs`,
the bind-order test,
the state-directory test,
and the home default in `tests/cli_args.rs`.
The cache cleanup's guards were removed the same way:
with the current key no longer skipped,
`the_current_key_is_never_removed_however_long_unused` failed (the current folder was removed);
with the age no longer compared,
`a_folder_used_within_30_days_is_kept` failed (all four recent folders removed),
and so did `a_start_removes_cache_folders_of_other_builds_unused_for_30_days` in `tests/cli_process.rs`
("the folder used a day ago was removed");
with the parser load no longer renewing the marker,
`a_marker_renewed_by_a_running_copy_keeps_its_folder` failed.

### Releases

GitHub pre-releases come from `.github/workflows/ide-release.yml`
(decided on 2026-10-22,
[decision][ide-releases];
steps in the [runbook][release-the-ide]).
A push to `main` that changes `version` in `Cargo.toml` starts a build and,
when the tag `ide-v<version>` is free,
a pre-release with four files:
`ide-<version>-linux-x64.tar.gz` and `ide-<version>-linux-arm64.tar.gz`
(each holds the executable from `bundle`,
the launcher entry,
the license texts,
and `INSTALL.txt`),
`ide-<version>-third-party-licenses.txt`,
and `SHA256SUMS`.
The release `ide-v0.1.0` predates the arm64 build:
it holds the x86_64 archive only,
and it is immutable,
so the arm64 archive first ships with the next version.
Only Linux on x86_64 and aarch64 is built,
and the requirements under [What the host provides](#what-the-host-provides) apply to both.

The workflow builds each architecture natively on its own runner
and runs the same tasks as a local build in the bounded container:
`bundle`,
then `inspect:no-software-renderer`,
and `inspect:bundle` with the checks `inventory`,
`license-texts`,
and `crate-licenses`,
which need no window.
It also requires that `--version` prints `monochromatic-ide <version>`,
that the dynamic section of the executable lists only the libraries it was measured to need
(on x86_64 the loader `ld-linux-x86-64.so.2` too,
on aarch64 no loader),
`libc`,
`libm`,
`libgcc_s`,
and `libfontconfig`,
that no symbol needs a glibc newer than 2.39,
and that the build leaves `Cargo.lock` as committed.
Raising the version therefore needs a refreshed `Cargo.lock`.
The other `inspect:bundle` checks and the test tasks need a Wayland session or a GPU and are not run by CI.
A manual dispatch with `dry-run` true builds and verifies everything without creating a tag or a release.

[ide-releases]: ../../../doc/decision/ide-releases.md
[release-the-ide]: ../../../doc/runbook/release-the-ide.md

### Measured on 2026-10-06

- Size:
  the executable is 81,743,984 bytes,
  34,887,184 more than the former directory's binary,
  close to the 34,659,293 embedded bytes.
- Build:
  one release build from an empty release directory took 12 min 45 s of Cargo time
  at host load averages between 45 and 78 when read during it;
  anonymous memory peaked at 938 MiB (sampled from the container's control group),
  the group reached its 2,048 MiB limit only through reclaimable file cache,
  used at most 45 MiB of swap,
  and recorded no out-of-memory kill.
- First start and later start,
  five runs each at host load averages of 90 to 99,
  for the 11,098,288-byte `sql.so`:
  unpacking took 7 to 586 ms (median 71 ms),
  the byte comparison of a later start 8 to 34 ms (median 10 ms);
  preparing the language as a whole (digest check,
  unpack or comparison,
  loading,
  and compiling its rules)
  took 218 to 776 ms on a first start (median 449 ms) and 130 to 204 ms on a later one (median 162 ms).
- Home folder:
  a disposable home of 283,450 files and 359,585,884 bytes
  (hidden caches and toolchains,
  documents,
  three 64 MiB binary downloads,
  ten git projects with ignored `node_modules` and `target`,
  one project without git)
  showed its tree 519 to 2,641 ms after the compositor started (three starts),
  with one inotify watch.
  Of its files,
  `rg --files` with the application's arguments lists 15,429.
  In the application,
  a query that matches nothing finished its file-name search in 266 ms
  and its content search in 4,618 ms;
  a common word reached the 30-result cap in 33 ms.
  Run directly with the same arguments,
  the content search for a query that matches nothing took 456 ms to 11,383 ms (three runs,
  cold page cache first).
- After the 30-day cleanup and `--licenses` (commit `0fe1d7ed8`):
  the executable is 81,766,496 bytes,
  22,512 more,
  with the same embedded table (1,254 files,
  key `c47e913b79bf6a42`);
  `--licenses` printed 34 texts in 117,354 bytes;
  the start-time sweep of the runtime cache folder took 36 to 116 µs in the ten bundle-check sessions
  that had nothing to remove,
  and 438 µs in `old-cache-folders-removed`,
  which removed a key folder and the rest of a cut-short removal
  (host load average 69.5 when the checks started).

### Measured on 2026-10-05 (former directory bundle)

Before the single executable,
`bundle` assembled a directory with the binary,
`runtime/`,
and `LICENSES/`:
81,516,093 bytes in 1,255 files,
the binary 46,856,800 bytes with its symbol table
(35,666,600 after `strip --strip-all`).
Two release builds from an empty release directory finished in 18 min 40 s and 15 min 22 s of Cargo time
at load averages between 54 and 97;
anonymous memory peaked at 1,116 MiB,
and both builds produced the same binary,
byte for byte.

[handover]: ../../../doc/handover/slint-ide-0x.md
[scope]: ../../../doc/decision/slint-ide-0x-scope.md
[plan]: ../../../doc/planning/slint-ide-implementation.md
