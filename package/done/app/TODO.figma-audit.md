# Figma audit: implementation vs design

Comparing the current implementation against the Figma wireframes:

- **Draft 3 tablet** (node 514-5446)
- **Draft 2 phone** (node 205-2193)

Figma file:
 `wSe7z97geCFtkFS0LZ4biq` (Hifi wireframes | todo app)

## Layout and navigation

### Side drawer

- **Figma (tablet)**:
   the navigation drawer shows a user profile row at the top with an avatar icon,
   "Firstname" label and a dropdown caret,
   followed by Inbox / In Progress / Settings / Contact links.
   Each link has a Material-style icon to the left (inbox,
   sprint,
   settings,
   contact_support).
   A thin divider separates the profile row from the nav links.
- **Figma (phone)**:
   same drawer content,
   but as a full-screen overlay with a close (X) button in the top-right corner of the header row.
   Status bar visible at the top.
- **Implementation**:
   the drawer renders "Firstname" as plain text with no avatar icon or dropdown caret.
   Nav links are text-only (no icons).
   The structure is otherwise correct (inline sidebar on tablet,
   popover on phone),
   but the visual richness is missing.

**Suggestions**:

- Add the profile avatar icon (the filled-person Material icon) before the name
- Add the dropdown caret after the name
- Add Material icons before each nav link (Inbox,
   In Progress,
   Settings,
   Contact)

### Top nav

- **Figma (tablet)**:
   the top nav shows the page title on the left (left-aligned),
   a search icon (magnifying glass) on the right,
   and a thin bottom border.
   No hamburger menu visible in tablet mode.
   The title text appears to use normal weight,
   roughly 1.5rem.
- **Figma (phone)**:
   adds a hamburger icon on the left,
   page title centered,
   search icon on the right.
- **Implementation**:
   matches the two-mode behavior correctly (hamburger hidden on tablet,
   shown on phone).
   The search icon is a custom CSS shape.
   Title centers on mobile but already aligns left on tablet.
   The bottom border on tablet mode is present.

**Suggestions**:

- The implementation is close.
   Consider using an actual SVG magnifying-glass icon to match Figma more precisely,
   though the CSS-drawn version is acceptable.

## Inbox page

### Suggested section

- **Figma**:
   the "Suggested" heading has a star/wand icon,
   the text "Suggested",
   and a small up/down toggle caret.
   Below it,
   "My location" and "My focus" controls are shown.
   On tablet they sit side by side;
   on phone they stack vertically.
   The autodetect toggle is a pill-shaped toggle-switch + location chip (e.g. "Walmart" with a pin icon).
   The "My focus" control is a dropdown with an icon.
- **Implementation**:
   uses a `<section-heading>` component with a text emoji icon and Unicode triangles for the toggle indicator.
   "My location" and "My focus" controls are stacked vertically in all viewports (never side-by-side).
   The autodetect toggle is a text button with a `<toggle-switch>`.
   Location chips exist but are basic text.

**Suggestions**:

- On tablet,
   lay out "My location" and "My focus" side by side (the implementation currently stacks them at all widths)
- Replace the Unicode triangle toggle (up/down) with an SVG chevron to match Figma
- Add a pin/location icon to the location chip ("Walmart")

### Task cards (li)

- **Figma**:
   each task row has a square checkbox (unfilled),
   the task title,
   and below that a row of small metadata chips.
   The chips show:
   `# shopping` (red/muted red text),
   `tracked: 0s` (with clock icon),
   `where: Walmart` (with pin icon),
   `priority: medium` (with exclamation icon),
   `due: today 9pm` (with calendar icon),
   `complexity: low` (with lightbulb icon).
   Each chip has a small icon prefix.
   The metadata row scrolls horizontally and does not wrap.
- **Implementation**:
   checkbox and title row matches.
   Metadata chips are rendered as plain text spans with no icons (just text like `# shopping`,
   `tracked: 0s`,
   `where: Walmart`).
   The tags chip uses a muted red color.
   No icons before chip text.

**Suggestions**:

- Add small inline icons (SVG or Material symbol) before each metadata chip to match Figma
- The `# tag` chip color (muted red) already matches

### All section

- **Figma**:
   identical card layout to Suggested,
   preceded by a collapsible "All" heading with an infinity icon and toggle caret.
- **Implementation**:
   matches structurally.
   Same icon/toggle suggestions apply.

### Divider between sections

- **Figma**:
   a 1px horizontal rule separates Suggested from All.
- **Implementation**:
   present and correct (`.divider` class).

### FAB (floating action button)

- **Figma (tablet)**:
   a circular FAB in the bottom-right corner of the main content area,
   with a "+" icon.
- **Figma (phone)**:
   same,
   positioned at bottom-right.
- **Implementation**:
   matches.
   The FAB is a circular button with `+` text,
   fixed bottom-right.

**Suggestions**:

- None;
   the FAB matches well.

## In-progress page

- **Figma**:
   shows the same task card layout as inbox.
   The top nav says the current page name.
   Only tasks with active timers appear.
- **Implementation**:
   matches the Figma intent.
   Same icon suggestions for task cards apply here.

## Settings page

- **Figma**:
   three setting rows visible:
  1. "System calendar":
      a connect/disconnect button-style row with a sync icon and descriptive text
  2. "Data privacy":
      a toggle switch beside the title,
      with description text below
  3. "Dark theme":
      a toggle switch beside the title,
      with description text below
- **Implementation**:
   uses `<setting-group>` components.
   The labels and descriptions match.
   Calendar connect is rendered with `mode="button"`.
   Toggle switches are present for the other two.

**Suggestions**:

- Add the sync icon to the calendar connect row
- Verify the toggle-switch visual (rounded pill shape) matches Figma precisely

## Search page

### Search bar

- **Figma (phone)**:
   status bar at top,
   then a search bar with a back arrow on the left and text input.
   Placeholder text is visible.
- **Figma (tablet)**:
   same search bar but the back arrow is hidden (same as how the hamburger disappears on tablet).
- **Implementation**:
   matches this behavior.
   Back button hides on tablet via media query.
   Input has placeholder text.

### Empty search state

- **Figma**:
   shows "Type something...or select one of the categories.
  " as hint text,
   followed by a row of tag chips (pill-shaped,
   outlined,
   with `#` prefix).
   Tags wrap to multiple lines on phone.
- **Implementation**:
   matches.
   Tag chips are rendered as outlined pill buttons that navigate to `/search?q=#tag`.

### Search results

- **Figma**:
   shows a heading like `#shopping` at the top of the results,
   then task cards in the same format as inbox.
- **Implementation**:
   does not show a results heading.
   Task cards appear directly.
   The heading showing the search term is missing.

**Suggestions**:

- Add a heading at the top of search results showing the current query (e.g. `#shopping`)

## Task details panel

- **Figma**:
   a panel with:
  - Header:
     X close button (left),
     "Task details" text (center),
     Save button (right,
     outlined)
  - Title input:
     underlined text field
  - Description:
     bordered textarea
  - Action buttons:
     "Attach file" and "Take photo" (outlined,
     with icons)
  - Metadata pills:
     outlined rounded pills with icon prefixes (`# ?`,
     `tracked: 0s`,
     `where: ?`,
     `priority: ?`,
     `due: ?`,
     `complexity: ?`,
     `reminders: None`,
     `blockedBy: none`).
     The pills wrap to a second row.
- **Implementation**:
   structurally matches.
   The close button uses an SVG X.
   The header layout matches.
   Action buttons say "Attach file" and "Take photo" but lack the icons.
   Metadata pills are present but without icon prefixes (same as task cards).

**Suggestions**:

- Add icons to the "Attach file" (paperclip) and "Take photo" (camera) buttons
- Add icon prefixes to the metadata pills (same as task card chip icons)
- In create mode,
   the header says "New task" and the save button says "Create";
   this matches Figma's "Task details" heading but the Figma design always shows "Task details" regardless of mode.
   Decide whether to unify or keep the distinction.

## Typography and spacing

- **Figma**:
   uses what appears to be a system/sans-serif font.
   Task titles are roughly 1.25rem,
   metadata chips are smaller (~1rem).
   Section headings are ~1.25rem.
   Page headings in top-nav are ~1.5rem.
- **Implementation**:
   uses `Inter, system-ui, sans-serif`.
   Font sizes match the Figma proportions (title 1.25rem,
   heading 1.5rem,
   chips 1rem).

**Suggestions**:

- Typography is already aligned.
   No changes needed.

## Color and theming

- **Figma**:
   light gray background,
   near-black text,
   muted red for tag text.
   Simple,
   monochromatic palette.
- **Implementation**:
   uses CSS custom properties (`--fg`,
   `--bg`,
   `--red-fg` etc.) with dark mode support via `prefers-color-scheme`.
   The light-mode colors visually match Figma.

**Suggestions**:

- Colors are well-matched.
   No changes needed.

## Summary of highest-impact gaps

Listed in rough priority order (biggest visual delta first):

1. **Missing icons throughout**:
    nav links,
    metadata chips,
    action buttons all lack the icons shown in Figma.
    This is the single biggest visual difference.
2. **Drawer profile area**:
    missing avatar icon and dropdown caret
3. **Suggested section layout on tablet**:
    "My location" and "My focus" should sit side by side,
    not stack
4. **Section heading toggle**:
    Unicode triangles instead of SVG chevrons
5. **Search results heading**:
    no heading showing the current query term above results
6. **"Attach file" / "Take photo" buttons**:
    missing paperclip and camera icons
