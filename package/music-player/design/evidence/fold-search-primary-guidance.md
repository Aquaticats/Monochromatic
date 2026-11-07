# Baseline Material 3 Search and foldable precedent

## Source boundary

The locally archived Google Material 3 pages are under
`/var/home/user/Downloads/m3.material.io/`.
 This note uses their extracted
readable text in `/var/home/user/temp/agent/md3-archive-text/` and names the
corresponding archived page paths,
 not a visual reconstruction from memory.
The Google foldable-app article is an older first-party example,
 not a
Pixel 9 Pro Fold Search capture.
 The direct current AVD observations are in
[`fold-search-emulator-precedents.md`](fold-search-emulator-precedents.md).

## Search anatomy and placement

The archived `components/search/guidelines/index.html` says the search bar
should **stay close to searchable content** and,
 in most cases,
 **stay in
its pane and scale in width accordingly**.
 Focused suggestions or results
appear in a **list below** the search bar.
 Its two focused layout options
are **docked** for medium/expanded windows and **full-screen** by default
for compact windows.
 The search entry point may be an icon button when
Search is a secondary action.
 A focused standalone Search destination
is possible when Search is primary;
 no passage demands that such a
Destination always fill the unfolded panel.

The archived `components/search/specs/index.html` lists the **divided**
style as available in baseline M3 and explains that a divider separates
Search input from suggestions/results.
 It also distinguishes the
contained,
expressive style;
 D47/D48 did not opt into Expressive.
The baseline anatomy is a Search bar and a container for suggestions or
results,
 empty until populated.
 This directly contradicts a layout in
which the header visually governs an empty pane while all results appear
in a detached pane.
 It does **not** choose between a docked one-half
Search region and a full-width page with safely placed text.

The archived `foundations/layout/scaffold/panes/index.html` says single
flexible panes can appear at any breakpoint,
 although two panes are
recommended at expanded sizes.
 A fixed pane may be temporary;
 its
example is a list or side sheet with light information density.
 Visible
panes may be co-planar,
 floating or docked according to task and
breakpoint.
 This allows a bounded Search pane alongside useful context;
there is no requirement to fill the other half with empty space.

The archived
`foundations/layout/canonical-examples/list-detail/index.html`
pairs a **list of items** with a **selected item's detail**,
 including
messages,
 files and music artist/album examples.
 It does not describe
splitting a Search query from its results into independent panes.
The archived
`foundations/layout/breakpoints/expanded/index.html` calls for 24dp
margins and a 24dp **pane spacer** in an expanded split layout.
 That
spacing recommendation is not a measurement of the user's physical
crease or a requirement for a blank painted stripe.

## First-party app article

Google's [foldable-app article][fold-apps] describes LINE conversations on
the left with the **selected conversation's detail** on the right;
Weather's ten-day forecast with **selected-day detail**;
 and Keep's note
list with an editor plus a toggle to expand the list.
 Its Deezer example
puts playback alongside lyrics and sharing.
 The images do **not** show
a Search query with populated and empty states across Fold postures,
 so
these are related context-preservation patterns,
 not evidence that Search
must use two panes.
 Our live Maps,
 Messages and DocumentsUI probes provide
the Search-specific comparisons.

## Project-specific constraint beyond those sources

The user supplied a visible physical dent width of **about 7.5mm** on the
inner display,
 roughly **110 panel px** at the 2076px physical resolution.
`max(min_padding, crease_width)` separates **informational material** in
a common physical coordinate system;
 borders,
 padding,
 background
surfaces and hit regions can cross.
 Neither baseline M3's breakpoint
spacing nor the AVD's zero-width occlusion sensor may overwrite that
user rule.
 Android dp varies with display scaling,
 so convert the
physical width only for the current rendering configuration.
 No source
here settles the numeric `min_padding` or a final Search layout.

[fold-apps]: https://blog.google/products-and-platforms/platforms/android/app-redesign-foldables/
