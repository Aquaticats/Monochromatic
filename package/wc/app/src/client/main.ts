/**
 Client-side entry point for the wc text-stats tool.
 
 Debounces the input textarea, then recomputes and renders the stat
 tiles and the Frequency rows on every settled change. Also auto-grows
 the textarea to its content (`field-sizing: content` is missing from
 the Firefox ESR baseline, so the growth is scripted).
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

// Imported from page-stats directly, never from ../page.ts: page.ts
// imports the favicon generator, whose sharp/node imports must stay
// out of this browser bundle's module graph (a leaked `node:` import
// kills the whole inline module script on file://).
import { STAT_TILES, } from '../page-stats.ts';
import {
  analyzeText,
  computeFrequency,
  splitWords,
  type FrequencyEntry,
  type TextStats,
} from '../stat/index.ts';

/**
 Idle period after the last keystroke before stats/frequency recompute.
 */
const STATS_DEBOUNCE_MS = 150;

/**
 Figure space (U+2007): a digit-width space in tabular-numeral context,
 used to pad frequency numbers so columns align with no column-width
 CSS. Retained in the Inter subset via `src/subset-fonts.ts`.
 */
const FIGURE_SPACE = ' ';

/**
 Grouping formatter for tile headline values ("7,801"). Locale pinned
 to `en-US` so the group separator stays inside the Inter subset's
 charset (other locales separate with code points the subset lacks).
 */
const countFormat = new Intl.NumberFormat('en-US',);

/**
 Header label of the Frequency word column (the header row is
 visually hidden, screen-reader-only; see `FREQUENCY_COLUMN_LABELS`
 in `../page.ts`). Kept in the word-column measurement as a stable
 floor so the column never collapses below a label's width.
 */
const WORD_COLUMN_HEADER = 'Word';

/**
 Slack in rem added to the measured word column, absorbing the small
 differences between canvas text measurement and DOM rendering
 (hinting, `font-variant-numeric` on digit-bearing words).
 */
const WORD_COLUMN_SLACK_REM = 1 / (2 * 2);

/**
 Ceiling in rem on the word column, so one pathological token (a
 chemical name, a URL, a DNA string) cannot crush the bar tracks.
 Longer words truncate visually with an ellipsis while the full word
 rides in the cell's `title` and `aria-label`.
 */
const WORD_COLUMN_MAX_REM = ((2 * 2) * 2) + (2 + 2);

/**
 Detached canvas backing the word-width measuring context.
 */
const measureCanvas = document.createElement('canvas',);

/**
 2D canvas context for measuring rendered word widths without
 touching DOM layout. Nullable per the platform API; where 2D
 contexts are unavailable the word column falls back to per-cell
 content sizing.
 */
const measureContext = measureCanvas.getContext('2d',);

/**
 Writes every {@link STAT_TILES} entry's field into its DOM element,
 formatted with {@link countFormat}. No tile carries an `id`, so
 headline and sub-stat elements are matched to {@link STAT_TILES}
 positionally: `.tile-value` elements render in tile order (one per
 tile), and `.tile-sub-value` elements render in tile order too, but
 only for the subset of tiles carrying a {@link StatTile.sub}.
 
 @param stats - aggregate statistics to render
 */
function renderStats(stats: TextStats,): void {
  /**
   Headline value elements, in {@link STAT_TILES} order.
   */
  const valueElements = document.querySelectorAll<HTMLElement>('.tile-value',);

  for (const [index, tile,] of STAT_TILES.entries()) {
    /**
     Headline element for the current tile, or `undefined` when absent.
     */
    const element = valueElements[index];

    if (element !== undefined) {
      element.textContent = countFormat.format(stats[tile.key],);
    }
  }

  /**
   Tiles carrying a "longest" sub-stat, in the same relative order as
   their rendered `.tile-sub-value` elements.
   */
  const subTiles = STAT_TILES.filter(function hasSub(tile,): boolean {
    return tile.sub !== undefined;
  },);
  /**
   Sub-stat value elements, in {@link subTiles} order.
   */
  const subValueElements = document.querySelectorAll<HTMLElement>('.tile-sub-value',);

  for (const [index, tile,] of subTiles.entries()) {
    /**
     Sub-stat destructured once so the definedness check and the field
     read both narrow off the same value.
     */
    const { sub, } = tile;
    /**
     Sub-stat element for the current tile, or `undefined` when absent.
     */
    const element = subValueElements[index];

    if ((element !== undefined) && (sub !== undefined)) {
      element.textContent = countFormat.format(stats[sub.key],);
    }
  }
}

/**
 Renders one Frequency row: count, percentage, word, and a
 proportional bar, with count and percentage figure-space padded to
 the widths of the top entry so tabular numerals align.
 
 @param entry - frequency row to render
 
 @param countWidth - character width counts are padded to
 
 @param pctWidth - character width percentage strings are padded to
 
 @param maxCount - top entry's count, the 100%-width bar reference
 
 @returns HTML string for the row
 */
function renderFrequencyRow(
  {
    entry,
    countWidth,
    pctWidth,
    maxCount,
  }: Readonly<{
    entry: FrequencyEntry;
    countWidth: number;
    pctWidth: number;
    maxCount: number;
  }>,
): string {
  return h(
    {
      tag: 'div',
      class: 'frequency-row',
      attrs: { role: 'row', },
      children: [
        h(
          {
            tag: 'span',
            class: 'freq-count',
            attrs: { role: 'cell', },
            text: String(entry.count,)
              .padStart(
              countWidth,
              FIGURE_SPACE,
              ),
          },
        ),
        h(
          {
            tag: 'span',
            class: 'freq-pct',
            attrs: { role: 'cell', },
            text: `${
              entry.percentage
                .toFixed(1,)
            }%`
              .padStart(
              pctWidth,
              FIGURE_SPACE,
              ),
          },
        ),
        h(
          {
            tag: 'span',
            class: 'freq-word',
            // Displayed text can truncate with an ellipsis once the
            // word column hits its cap; title (hover) and aria-label
            // (assistive tech) always carry the whole word.
            attrs: {
              role: 'cell',
              title: entry.word,
              'aria-label': entry.word,
            },
            text: entry.word,
          },
        ),
        h(
          {
            // Decorative: the whole bar cell is hidden from assistive
            // tech (no `role="cell"`), since `.freq-count`/`.freq-pct`
            // already carry the real data; screen readers thus see
            // exactly the columns the visually hidden header row
            // (`FREQUENCY_COLUMN_LABELS` in `../page.ts`) names.
            tag: 'span',
            class: 'freq-bar-track',
            attrs: { 'aria-hidden': 'true', },
            children: [
              // `value`/`max` are ordinary content attributes, so the
              // fill ratio needs no per-row `style` or `id`.
              h(
                {
                  tag: 'progress',
                  class: 'freq-bar',
                  attrs: {
                    value: String(entry.count,),
                    max: String(maxCount,),
                  },
                },
              ),
            ],
          },
        ),
      ],
    },
  );
}

/**
 Column count of the Frequency table as assistive tech sees it
 (count, percent, word; the decorative bar cell is `aria-hidden`),
 for the placeholder row's `aria-colspan`.
 */
const FREQUENCY_COLUMN_COUNT = 3;

/**
 Renders the Frequency placeholder row for when no word occurs more
 than once.
 
 @returns HTML string for the placeholder row
 */
function renderEmptyFrequencyRow(): string {
  return h(
    {
      tag: 'div',
      class: 'frequency-row',
      attrs: { role: 'row', },
      children: [
        h(
          {
            tag: 'span',
            class: 'frequency-empty',
            attrs: {
              role: 'cell',
              'aria-colspan': String(FREQUENCY_COLUMN_COUNT,),
            },
            text: 'No repeated words yet.',
          },
        ),
      ],
    },
  );
}

/**
 Measures the widest of words as rendered in reference's computed
 font, via canvas `measureText`, so unbounded row counts never force
 per-row DOM layout reads. The width sizes the shared word column:
 with the number columns already equal (figure-space padding) and the
 word column fixed, the flex-growing bar track comes out identical in
 every row, keeping bar lengths comparable.
 
 @param words - word column contents to measure
 
 @param reference - element whose computed font the words render in
 
 @param context - measuring canvas context, narrowed by caller
 
 @returns widest width in rem plus {@link WORD_COLUMN_SLACK_REM},
 capped at {@link WORD_COLUMN_MAX_REM}
 
 @mutates context - assignment updates `context.font` before observational `context.measureText` calls
 */
function measureWordColumnRem(
  {
    words,
    reference,
    context,
  }: ForeignBorrowed<Readonly<{
    words: readonly string[];
    reference: HTMLElement;
    context: CanvasRenderingContext2D;
  }>>,
): number {
  /**
   Computed font parts of the element the words render inside.
   */
  const {
    fontWeight,
    fontSize,
    fontFamily,
  } = getComputedStyle(reference,);

  context.font = `${fontWeight} ${fontSize} ${fontFamily}`;

  /**
   Widest measured word width in CSS pixels.
   */
  const widestPx = (function measureWidestPx(): number {
    /**
     Widest width accumulated inside isolated mutation scope.
     */
    let widest = 0;
    for (const word of words) {
      /**
       Measured metrics of word in the reference font.
       */
      const metrics = context.measureText(word,);
      widest = Math.max(
        widest,
        metrics.width,
      );
    }
    return widest;
  })();

  /**
   Root computed font-size string, always serialized in `px`.
   */
  const { fontSize: rootFontSize, } = getComputedStyle(
    document.documentElement,
  );

  /**
   Index where the `px` unit starts inside the root font-size string.
   */
  const unitStart = rootFontSize.indexOf('px',);

  /**
   Root font size in CSS pixels, for the px-to-rem conversion (rem
   keeps the column proportional if the root size changes).
   */
  const rootPx = Number(rootFontSize.slice(
    0,
    unitStart,
  ),);

  return Math.min(
    (widestPx / rootPx) + WORD_COLUMN_SLACK_REM,
    WORD_COLUMN_MAX_REM,
  );
}

/**
 Replaces the Frequency body rowgroup with rows for entries, via
 {@link renderFrequencyRow}, or {@link renderEmptyFrequencyRow} when
 entries is empty, and pins the shared `--word-col` custom property
 to the measured widest word so every row's bar track is equal. This
 is the one deliberate inline-style holdout in the module: the
 measured width is a continuous, per-render value with no static-HTML
 attribute that could carry it, unlike the bar fill (an ordinary
 `value`/`max` pair on a native `<progress>`, see
 {@link renderFrequencyRow}).
 
 @param entries - frequency rows to render, sorted by count descending
 */
function renderFrequency(entries: readonly FrequencyEntry[],): void {
  /**
   Frequency body rowgroup element, or `null` when absent.
   */
  const body = document.querySelector<HTMLElement>('.frequency-body',);

  if (body === null) {
    return;
  }

  /**
   Frequency table container carrying the shared word-column custom
   property (header and body word cells both read it).
   */
  const frequency = body.closest<HTMLElement>('.frequency',);

  if (entries.length === 0) {
    body.innerHTML = renderEmptyFrequencyRow();

    if (frequency !== null) {
      /**
       Container style, destructured so member access stays flat.
       */
      const { style, } = frequency;

      style.removeProperty('--word-col',);
    }

    return;
  }

  if ((frequency !== null) && (measureContext !== null)) {
    /**
     Word column inline size in rem, from the widest rendered word.
     */
    const wordColumnRem = measureWordColumnRem(
      {
        words: [
          ...entries.map(function pickWord(entry,): string {
            return entry.word;
          },),
          WORD_COLUMN_HEADER,
        ],
        reference: frequency,
        context: measureContext,
      },
    );

    /**
     Container style, destructured so member access stays flat.
     */
    const { style, } = frequency;

    style.setProperty(
      '--word-col',
      `${wordColumnRem}rem`,
    );
  }

  /**
   Top entry; entries are sorted by count descending, so it defines
   the widest count string, the widest percentage string, and the
   100%-width bar reference.
   */
  const [top,] = entries;

  if (top === undefined) {
    return;
  }

  /**
   Character width counts are padded to.
   */
  const countWidth = String(top.count,)
    .length;

  /**
   Character width percentage strings are padded to.
   */
  const pctWidth = `${
    top.percentage
      .toFixed(1,)
  }%`.length;

  body.innerHTML = entries
    .map(function renderRow(entry,): string {
      return renderFrequencyRow(
        {
          entry,
          countWidth,
          pctWidth,
          maxCount: top.count,
        },
      );
    },)
    .join('',);
}

/**
 Recomputes and renders every result section for text, via
 {@link renderStats} and {@link renderFrequency}.
 
 @param text - current textarea value
 */
function updateResults(text: string,): void {
  renderStats(analyzeText(text,),);
  renderFrequency(
    computeFrequency(splitWords(text,),),
  );
}

/**
 Grows a textarea to fit its content: resets the scripted minimum so
 the flex layout can reclaim space after deletions, then raises it to
 the content's scroll height. The flex stretch keeps the
 viewport-filling floor, so short content never shrinks the box below
 the visible page remainder. The grown height is a continuous,
 per-render pixel value with no static-HTML attribute to carry it, so
 (like {@link renderFrequency}'s word-column width) it deliberately
 keeps using `style` rather than a class.
 
 @param input - textarea to grow
 
 @mutates input - assignments update `input.style.minBlockSize` to fit current content
 */
function autoGrow(
  { input, }: ForeignBorrowed<Readonly<{ input: HTMLTextAreaElement; }>>,
): void {
  /**
   Style declaration destructured once so member access stays flat.
   */
  const { style, } = input;

  style.minBlockSize = '';

  /**
   Block-axis border total: `min-block-size` spans borders under
   `border-box` sizing while `scrollHeight` does not, so growing to
   bare `scrollHeight` leaves a border-height sliver of internal
   scroll.
   */
  const borderCompensation = input.offsetHeight - input.clientHeight;

  style.minBlockSize = `${input.scrollHeight + borderCompensation}px`;
}

/**
 Renders every result section from input's current value and re-grows
 input to fit it. Runs at startup and again on `pageshow` because
 browsers restore textarea values across reloads (F5) and
 back/forward navigations without firing `input`, at timings that
 vary by browser (often after inline scripts have run); re-reading
 the live value once the page has fully shown is the reliable hook.
 
 @param input - textarea whose current value drives the page
 
 @mutates input - `autoGrow` updates `input.style.minBlockSize` before results render
 */
function syncFromInput(
  { input, }: ForeignBorrowed<Readonly<{ input: HTMLTextAreaElement; }>>,
): void {
  autoGrow({ input, },);
  updateResults(input.value,);
}

/**
 Input textarea the user types or pastes text into.
 */
const textarea = document.querySelector<HTMLTextAreaElement>('.wc-input',);

if (textarea !== null) {
  // `.scripted` (styled in `styles-layout.ts`) hides the textarea's own
  // scrollbar now that growth tracks content; added here (not present
  // in the static markup) so content stays reachable via the native
  // scrollbar if scripting is unavailable.
  /**
   Class list destructured once so member access stays flat.
   */
  const { classList, } = textarea;

  classList.add('scripted',);
  syncFromInput({ input: textarea, },);

  window.addEventListener(
    'pageshow',
    function handlePageShow(): void {
      syncFromInput({ input: textarea, },);
    },
  );

  /**
   Document font set, destructured so member access stays flat.
   */
  const { fonts, } = document;

  // Word-column measurement runs against the fallback font until the
  // embedded Inter finishes decoding; re-sync once font loading
  // settles so the measured column matches the real rendering.
  fonts.addEventListener(
    'loadingdone',
    function handleFontsLoaded(): void {
      syncFromInput({ input: textarea, },);
    },
  );

  /**
   Container for the shared debounce timer handle, so the binding stays
   `const` while the handle is reassigned on every keystroke.
   */
  const timer: { handle?: ReturnType<typeof setTimeout>; } = {};

  textarea.addEventListener(
    'input',
    function handleInput(): void {
      autoGrow({ input: textarea, },);
      clearTimeout(timer.handle,);
      timer.handle = setTimeout(
        function updateAfterDebounce(): void {
          updateResults(textarea.value,);
        },
        STATS_DEBOUNCE_MS,
      );
    },
  );
}
