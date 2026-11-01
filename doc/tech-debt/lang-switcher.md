# Tech debt: ssg-test language switcher dropdown

Created 2026-05-12.
 The language switcher shipped in commit `68c3930d` (rewritten same day to `<details>` / `<summary>`
  after the original Popover API attempt did not register clicks reliably) got the feature out the door
  under the same same-day deadline as the Catalan translations (see `TECH-DEBT.catalan-translations.md`).
 The visible shape is right and the build is clean,
 but the code,
 UI,
 and UX all cut corners that would not survive a real review.

## Code dirt

- **Generic layout types polluted with feature concerns**:
   `pageLayout()` in `src/templates/layout.ts` and `html()` in `src/components/site-header.ts` now take `currentName`
    and `availableInLangs` so the switcher can compute same-post links.
   These are lang-switcher concerns that have no business in a generic layout signature;
   every future page-context need will accrete here unless this is refactored
    into a single opaque `pageContext` object (or the switcher pulls from a context rather than receiving props).
- **`?: T | undefined` shortcut over conditional spread**:
   `currentName?: string | undefined` was the path of least resistance under `exactOptionalPropertyTypes`.
   The codebase's idiomatic alternative is conditional-spread at the call site,
   which preserves the stronger optional semantics.
   Three signatures (lang-switcher,
   site-header,
   layout) carry this shortcut.
- **`<lang-switcher data-is="">` marker is dead weight**:
   every other custom element under `src/components/` uses `data-is=""` to flag a client-registered element;
   the lang-switcher has no client-side class registering it.
   The marker should either be removed or paired with a registration even if the body is empty for forward-compat.
- **`availableLangsByName` precomputed for one consumer**:
   built once at the top of `generatePages` in `src/build/pages.ts`
    but only consumed inside the per-locale post-write map.
   Could be inlined as `byName[name].map(p => p.lang)` at the call site for a smaller diff and no surprise prefetch.
- **Magic value `font-weight: 600`** on the current-locale anchor
 in `lang-switcher.ts` should be a named constant alongside `MENU_MIN_INLINE_REM`.
- **Check-glyph hardcoded in CSS content**:
   `&::before { content: "\2713\00a0" }` puts a Unicode check plus nbsp directly in CSS.
   Inflexible,
   unstyleable separately,
   and the chosen glyph was not designer-vetted.
- **No keyboard navigation between menu items**:
   tab cycles but arrow keys do nothing.
   A roving-tabindex implementation would need client JS and was skipped under time pressure.
- **No JS means no viewport-overflow handling**:
   the menu is right-aligned via static `inset-inline-end: 0` absolute positioning.
   On narrow viewports or when the header lays out differently,
   the menu can clip without any reposition logic.

## UI dirt

- **Flat menu surface**:
   a 1px border and `--color-bg` fill.
   No shadow,
   no elevation cue.
   Looks pasted on top of content rather than floating.
- **Uneven indent between current and non-current items**:
   the `::before` check on the current-locale anchor pushes its text right by one glyph + nbsp,
   while other items start flush.
   A fixed indicator column (e.g. `padding-inline-start` reserved for everyone,
   glyph absolutely positioned in that gutter) would align baselines.
- **Weak hover affordance**:
   hover background is `var(--color-border)`,
   a low-contrast neutral.
   Easy to miss the hover state,
   especially in dark mode.
- **Trigger sizing diverges from siblings**:
   lang-switcher trigger sets `min-inline-size: 3rem; min-block-size: 3rem` (`TOUCH_TARGET`),
   but `<theme-toggle>` and `<site-search>` siblings do not enforce the same minimum.
   Heights may not align in the nav row.
- **Arbitrary menu width**:
   `min-inline-size: 8rem` was eyeballed for the three current autonyms (`English`,
   `Català`,
   `中文`).
   Not tested against locales with longer autonyms.
- **CJK vs Latin script baseline mismatch**:
   `中文` sits at a different optical baseline and weight than `English`/`Català`
    because the Material/Inter stack falls back to the system CJK font.
   Visual rhythm in the menu looks uneven.
- **No focus ring on the menu container**:
   only individual links have `:focus-visible` rings.
   When the popover opens,
   there is no perimeter cue.

## UX dirt

- **Icon-only trigger hides current language at rest**.
   This was the user's explicit choice during the design questions,
   but the trade-off is real:
   first-time visitors cannot see which locale they are reading until they open the menu.
   Worth A/B-considering "icon + current label" against the chosen icon-only later if discoverability data turns up bad.
- **Silent fallback to landing**:
   when a post does not exist in the target locale,
   the item links to `/{lang}` instead of `/{lang}/{name}` with no visual cue.
   A user clicking `中文` on a post they expect translated lands on the Chinese landing
    and may not realise the post itself is missing.
   A strikethrough,
   "(not translated)" suffix,
   or `aria-describedby` hint would surface this.
- **No focus management on open**:
   pressing the trigger opens the popover but focus stays on the button.
   The user must Tab to reach the first menu item;
   conventional menu pattern moves focus into the menu on activation.
- **No keyboard arrows between items**:
   tabbing works but is non-standard for menus.
   ARIA Authoring Practices expects ArrowUp/ArrowDown.
- **No open/close transition**:
   the menu snaps in and out.
   Acceptable for a utility menu but jarring next to the rest of the site.
- **No close button**:
   relies on click-outside (native popover light-dismiss) and Esc.
   Discoverable to power users;
   obscure to others.
- **No click-outside dismiss**:
   `<details>` does not close when the user clicks elsewhere on the page.
   The user has to click the summary again to collapse.
   The original Popover API attempt would have given this for free;
   the rewrite traded it away for click reliability.
- **No Esc dismiss**:
   same shape as click-outside;
   pressing Esc with the menu open does nothing.
   Would need a small bit of JS or the Invokers proposal.
- **Screen-reader trigger label is locale-agnostic**:
   `aria-label="Switch language"` does not announce the current locale.
   A reader has to walk the menu items and find `aria-current="page"` to discover the active locale.
   Better:
   `aria-label="Switch language, currently English"` composed at render time.
- **Check glyph in current-locale anchor reads aloud**:
   `\2713` is rendered as text content via `::before`,
   so screen readers may announce "check mark Català".
   Using `aria-current="page"` as the sole semantic signal and the glyph purely decorative (via `::marker`
    or `aria-hidden` sibling,
   both with caveats) would be cleaner.

## Out of scope for the look-back

- Adding more locales:
   stays at `ca / en / zh`.
- Replacing the Popover API with a JS-driven component:
   keep native;
   tighten what is there.
- Replacing Material Symbols / Inter font stack.
