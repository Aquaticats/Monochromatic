# Footer newsticker sizes its slot from layout, not from breakpoints

Decision record for issue #477,
"Personal site footer ticker is sometimes cut off on smaller screens".
Decisions below were put to the user question by question and accepted explicitly.

## Problem

`package/ssg/aquati.cat/src/component/site-footer.ts` pinned
`site-footer footer` to `block-size: 1lh` with `overflow-block: clip`,
and stepped an inner track by `translateY(-N * 100/9%)`.
Both assume every quote occupies exactly one line.

Two quotes are 97 and 103 characters.
Below roughly 779px of footer width they wrap,
and two failures compound:
the extra lines are clipped away,
and the track grows past `9lh` so each `100/9%` step stops landing on a line
boundary and every following quote drifts out of position.
Short quotes stay on one line,
which is why the report said "sometimes".

## Decisions

-   **Long quotes get a uniform multi-line slot.**
    Not truncation,
    not a font shrink,
    not a horizontal marquee,
    and not rewriting the quotes.
-   **Quote text is fixed.**
    The layout absorbs whatever length it is given.
    A layout that only works for the current nine strings would break the next
    time a quote is added,
    and `TODO.generalize-ssg.md:34-36` plans to make
    `tickerQuotes` site-configurable.
-   **320px is the supported floor.**
-   **`prefers-reduced-motion: reduce` is handled**,
    which the ticker never did.
    The animation pauses on a single quote rather than swapping quotes instantly,
    because an instant swap is still auto-updating content the visitor cannot
    pause,
    so it would not discharge WCAG 2.2.2.
-   **No media query and no container query.**
    All nine quotes share one CSS grid cell,
    so the slot is the tallest quote's
    wrapped height at the current width,
    computed by layout.
    No breakpoint declares a line count,
    and the result holds for any font,
    any width,
    and any quote set.
-   **One shared timeline,
    per-quote `animation-delay`.**
    Every delay is negative,
    which seeks each quote into an already-running cycle.
    That is what lets `animation-play-state: paused` freeze each quote exactly
    where its delay placed it,
    so reduced motion and the random start offset fall
    out of the same arithmetic.
-   **A `random()` seed rotates the starting quote per load**,
    registered through `@property` so engines without `random()` fall back to its
    `initial-value` and start at the first quote.
-   **Graceful degradation,
    no script.**
    No JS fallback for the randomness.
    `shuffle-children` earned its script because quiz question order is functional;
    a footer joke starting in the same place is not a defect.
-   **The footer takes the site's `1rem` inline gutter**,
    matching `site-header.ts:50` and `page-content.ts:41`.

## Rejected alternatives

-   **Hard one line with `text-overflow: ellipsis`.**
    The two longest quotes are the two with the best payload.
-   **Natural per-quote height.**
    Needs runtime measurement or a jumping footer height.
-   **Horizontal marquee.**
    A full redesign,
    slow to read,
    and an accessibility liability.
-   **Shortening the quotes.**
    Moves the threshold instead of solving the general case.
-   **A hidden sizer element** holding a copy of the longest quote,
    which would have let the existing translate-the-track mechanism stay.
    Rejected because it pins correctness to a build-time guess about which
    string wraps tallest,
    which is not the same as longest by character count,
    and because generalize-ssg will hand those strings to site config.
-   **Container query with `ch` breakpoints.**
    Measured as workable:
    `ch` inside `@container` resolves against the
    container's own font,
    so a 300px container at `font-size: 32px` did not match
    `@container (min-inline-size: 20ch)` because `1ch` measured 19.63px there.
    Rejected anyway once the grid-cell approach removed the need to count lines.

## Measured evidence

Chrome 149,
Inter loaded (`document.fonts.status: "loaded"`),
root 16px,
`line-height: 1.6`,
`1lh` = 25.59px,
`1ch` = 10.06px.

Sweeping the footer's inline size and reading back the maximum wrapped line count
across all nine quotes,
without padding:
1 line at 779px and up,
2 from 405 to 778,
3 from 283 to 404,
4 from 228 to 282.
With `padding-inline: 1rem` every threshold shifts by exactly +32px.

At the 320px floor the answer is 3 lines either way,
so the gutter costs no extra line.
Padding moves only the Marmots quote from 1 line to 2,
and the slot follows the
maximum,
which does not move.

Font size does not change the outcome.
At 320px with the gutter,
the maximum stays 3 lines at every size from 16px down
to 12px,
so a narrow-viewport font shrink buys no slot at all.

Verification after the fix,
zero console and page errors:
the slot auto-sizes to 3 lines at 320,
375 and 414px,
2 at 768px,
1 at 1024 and
1440px,
with all nine quotes uniform and none clipped.
Sampling over 11 seconds showed quotes advancing 0,
1,
2,
3 with exactly one in
the slot throughout.
Under reduced motion the animation reports `paused` and does not move after 5 seconds.
Forcing the seed to 8 selects the 103-character quote and renders all three lines
unclipped at 320px.

## Corrections made during this session

Recorded so a future session does not re-derive them.

-   **A bare `random()` is per-element,
    not shared.**
    The omitted key means `auto`,
    which the specification defines as
    `element-scoped property-index-scoped`.
    `shuffle-children`'s per-child intent was right all along.
-   **`cssRandom` emitted a stale grammar.**
    See `doc/troubleshooting/css-random.md`.
    That was a prerequisite fix,
    not scope creep,
    because the accepted design needs a working `random()` helper.
-   **The `1rem` gutter does not cost a line at 320px.**
    An earlier claim that it "will likely push an extra line" was arithmetic,
    not measurement,
    and measurement contradicted it.
-   **The media-versus-container question should never have been asked.**
    Neither is needed.

## Commits

-   `8decc5d92` `fix(module-hyperscript): emit current random() grammar`
-   `d4ba30caa` `fix(ssg-aquati.cat): stop clipping wrapped ticker quotes on narrow screens`
-   `6ce012e3f` `test(ssg-aquati.cat): cover the ticker slot sizing and delay timing`

## Open questions

-   Issue #488:
    the `shuffle-children` CSS path has never executed in a browser,
    because the `by` grammar was invalid everywhere.
    It goes live in Safari 26.2 and later and in Chrome 155.
    Needs confirmation that children receive distinct `order` values and that the
    CSS path and `src/client/shuffle-children.ts` do not both apply.
-   `site-footer.unit.test.ts` imports package source,
    which trips
    `test-import(require-eventual-artifact)`.
    That is the documented repo-wide baseline of 383 files pending migration in
    `doc/planning/oxlint-test-import-eventual-artifact.md`;
    this package ships no entry that would resolve its components.
