/**
 Site footer with rolling newsticker.

 Colocates the CSS animation, keyframe generation, ticker quotes,
 and HTML template in a single `<site-footer>` custom element.

 All quotes share one CSS grid cell, so the footer's block size is the
 tallest quote's wrapped height at whatever inline size the footer happens
 to have. That makes the visible slot self-sizing: no breakpoint declares
 how many lines a quote takes, and a quote that wraps to three lines on a
 narrow screen is shown in full rather than clipped after its first line.

 Each quote animates through that one slot on a shared keyframe timeline,
 offset by a per-quote `animation-delay`. A `random()` seed shifts every
 delay by a whole number of quotes, so a different quote greets each visit.
 */
import {
  cssCalc,
  cssInt,
  cssPercent,
  cssRandom,
  cssRem,
  cssS,
  cssTranslateY,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

//region Ticker quotes

/**
 Ticker quotes displayed one at a time in the footer.

 @example
 ```ts
 TICKER_QUOTES.length // 9
 ```
 */
const TICKER_QUOTES = [
  // Cookie Clicker: the game's newsticker has this news
  'flavor text flavored flavorless',
  // AI slop: "slop" as slang for low-effort AI-generated content
  'sloppiest sloppy slop',
  // SpongeBob, the Krusty Krab training video: "the finest dining establishment ever established for dining"
  'programmable blogging program programmed for blogging',
  // Recursive error message: an error handler that errors while handling errors
  'ErrorError: ErrorError in erroring Error to ErrorError',
  // TC39 pipeline operator proposal: stuck in the standardization pipeline since 2017
  'pipeline operator stuck in pipeline',
  // Mafumafu: "すーぱーぬこになれんかった" (Super Nuko ni Narenkatta, 2019)
  `I wasn't able to become a super cat after all.`,
  // Francis Bacon: "Of Studies" (1597), with the literal food reading swapped in
  'Bacon: Some ... are to be tasted, others to be swallowed, and some few to be chewed and digested.',
  // Viral video: Exotic Black TV cleaning a Himalayan marmot with a paint roller (2025)
  'Marmots can be cleaned with paint rollers.',
  // Phil Karlton: "There are only two hard things in Computer Science: cache invalidation and naming things."
  'There are only 10 hard problems in computer science: cache invalidation, naming things, and off by one.',
] as const;

//endregion Ticker quotes

//region Animation constants

/**
 Number of quotes in the ticker.
 */
const QUOTE_COUNT = TICKER_QUOTES.length;

/**
 Seconds each quote is held visible before scrolling.

 @example
 ```ts
 HOLD_DURATION // 3
 ```
 */
const HOLD_DURATION = 3;

/**
 Seconds for the scroll transition between quotes.

 @example
 ```ts
 SCROLL_DURATION // 0.5
 ```
 */
const SCROLL_DURATION = 0.5;

/**
 Seconds between one quote reaching the slot and the next reaching it.

 @example
 ```ts
 QUOTE_INTERVAL // 3.5
 ```
 */
const QUOTE_INTERVAL = HOLD_DURATION + SCROLL_DURATION;

/**
 Total animation cycle duration in seconds.

 @example
 ```ts
 TOTAL_DURATION // 31.5
 ```
 */
const TOTAL_DURATION = QUOTE_COUNT * QUOTE_INTERVAL;

/**
 Seconds into a quote's pass at which it has finished sliding into the slot.
 */
const ENTER_END = SCROLL_DURATION;

/**
 Seconds into a quote's pass at which it begins sliding back out of the slot.
 */
const HOLD_END = ENTER_END + HOLD_DURATION;

/**
 Seconds into a quote's pass at which it has fully left the slot.
 */
const EXIT_END = HOLD_END + SCROLL_DURATION;

/**
 Seconds subtracted from every quote's delay so the last one still starts negative.

 Seeking backwards by a whole cycle plus one scroll leaves every quote mid-animation
 at load, which is what lets a paused animation freeze each one where its delay put it.
 */
const CYCLE_SHIFT = SCROLL_DURATION + TOTAL_DURATION;

/**
 Translate distance that parks a quote exactly one slot outside the visible slot.

 Resolved against each quote's own border box, which is one slot tall because
 every quote shares the same grid cell, so this is one slot regardless of how
 many lines the quote wrapped to.

 @example
 ```ts
 SLOT_OFFSET // 100
 ```
 */
const SLOT_OFFSET = 100;

/**
 Multiplier converting a fraction of the cycle into a keyframe stop percentage.
 */
const PERCENT_SCALE = 100;

/**
 Decimal places kept when rounding keyframe stop percentages.

 Four places keep every stop distinct at this cycle length while staying
 short enough to read in the built stylesheet.
 */
const STOP_PRECISION = 4;

/**
 Name of the animation carrying a quote through the visible slot.
 */
const TICKER_ANIMATION = 'ticker-scroll';

/**
 Custom property name, without leading dashes, holding the per-load start offset.
 */
const SEED_NAME = 'ticker-seed';

/**
 Registered custom property holding the per-load start offset, in whole quotes.
 */
const SEED_PROPERTY = `--${SEED_NAME}`;

/**
 Inline gutter in `rem`, matching the padding on `site-header` and `page-content`.
 */
const FOOTER_GUTTER = 1;

//endregion Animation constants

//region Keyframe generation

/**
 Converts an elapsed second count into its keyframe stop percentage.

 @param seconds - elapsed seconds measured from the start of the cycle

 @returns keyframe selector such as `'11.1111%'`

 @example
 ```ts
 stopPercent(3.5) // '11.1111%'
 ```
 */
function stopPercent(seconds: number,): string {
  /**
   Stop position rounded to `STOP_PRECISION`, with trailing zeroes dropped.
   */
  const rounded = Number(
    ((seconds / TOTAL_DURATION) * PERCENT_SCALE).toFixed(STOP_PRECISION,),
  );

  return `${rounded}%`;
}

/**
 Generates the `@keyframes ticker-scroll` stops shared by every quote.

 One quote's pass through the slot is: waiting one slot below, sliding up
 into the slot, holding, then sliding up one slot above. The stop that lands
 it above carries `step-end`, so the return to the waiting position below
 happens as an instant jump at the end of the cycle rather than as a visible
 downward sweep back through the slot.

 @returns array of CSS keyframe rule strings

 @example
 ```ts
 const stops = tickerKeyframeStops();
 // ['0%{transform:translateY(100%)}', '1.5873%{transform:translateY(0%)}', ...]
 ```
 */
function tickerKeyframeStops(): string[] {
  /**
   Transform parking a quote one slot below the visible slot.
   */
  const below = cssTranslateY(cssPercent(SLOT_OFFSET,),);
  /**
   Transform placing a quote in the visible slot.
   */
  const inSlot = cssTranslateY(cssPercent(0,),);
  /**
   Transform parking a quote one slot above the visible slot.
   */
  const above = cssTranslateY(cssPercent(-SLOT_OFFSET,),);

  return [
    $({
      rule: stopPercent(0,),
      decls: { transform: below, },
    },),
    $({
      rule: stopPercent(ENTER_END,),
      decls: { transform: inSlot, },
    },),
    $({
      rule: stopPercent(HOLD_END,),
      decls: { transform: inSlot, },
    },),
    $({
      rule: stopPercent(EXIT_END,),
      decls: {
        transform: above,
        'animation-timing-function': 'step-end',
      },
    },),
    $({
      rule: stopPercent(TOTAL_DURATION,),
      decls: { transform: below, },
    },),
  ];
}

/**
 Generates the per-quote `animation-delay` rules that stagger the shared timeline.

 Every delay is negative, which seeks each quote to its position partway through
 an already-running cycle. That matters for two reasons: no quote sits at its
 unstarted default transform while waiting its first turn, and pausing the
 animation freezes each quote exactly where its delay placed it.

 Subtracting the seed shifts every quote by the same whole number of intervals,
 which rotates the starting point of the sequence without disturbing its order.

 @returns array of CSS rule strings, one per quote

 @example
 ```ts
 const rules = tickerDelayRules();
 // rules.length === 9
 ```
 */
function tickerDelayRules(): string[] {
  return TICKER_QUOTES.map(function delayRule(
    _quote: string,
    index: number,
  ): string {
    /**
     Seconds this quote trails the first one in the sequence.
     */
    const sequenceOffset = index * QUOTE_INTERVAL;
    /**
     Seed-independent delay placing this quote's hold at its slot in the sequence.
     */
    const base = sequenceOffset - CYCLE_SHIFT;

    return $({
      rule: `site-footer .ticker-stack > p:nth-child(${index + 1})`,
      decls: {
        'animation-delay': cssCalc(
          `${cssS(base,)} - ${cssVar(SEED_NAME,)} * ${cssS(QUOTE_INTERVAL,)}`,
        ),
      },
    },);
  },);
}

//endregion Keyframe generation

//region CSS

/**
 Footer and ticker styles with the scroll animation.

 Registering the seed via `@property` is what makes the degraded path work:
 browsers without `random()` drop the declaration and fall back to the
 registered `initial-value`, starting the sequence at its first quote rather
 than collapsing every quote onto the same delay.

 @returns CSS string for the `<site-footer>` element

 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      at: 'property',
      params: SEED_PROPERTY,
      decls: {
        syntax: '"<integer>"',
        inherits: 'true',
        'initial-value': '0',
      },
    },),
    $({
      rule: 'site-footer footer',
      decls: {
        'overflow-block': 'clip',
        'text-align': 'center',
        'padding-inline': cssRem(FOOTER_GUTTER,),
        [SEED_PROPERTY]: cssRandom({
          key: SEED_PROPERTY,
          min: 0,
          max: QUOTE_COUNT - 1,
          step: 1,
        },),
      },
      children: [
        $({
          rule: '& p',
          decls: { 'margin-block': 0, },
        },),
      ],
    },),
    $({
      rule: 'site-footer .ticker-stack',
      decls: { display: 'grid', },
    },),
    $({
      rule: 'site-footer .ticker-stack > p',
      decls: {
        'grid-row-start': cssInt(1,),
        'grid-column-start': cssInt(1,),
        'animation-name': TICKER_ANIMATION,
        'animation-duration': cssS(TOTAL_DURATION,),
        'animation-iteration-count': 'infinite',
        'animation-timing-function': 'ease-in-out',
      },
    },),
    ...tickerDelayRules(),
    $({
      at: 'keyframes',
      params: TICKER_ANIMATION,
      children: tickerKeyframeStops(),
    },),
    $({
      at: 'media',
      params: '(prefers-reduced-motion: reduce)',
      children: [
        $({
          rule: 'site-footer .ticker-stack > p',
          decls: { 'animation-play-state': 'paused', },
        },),
      ],
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Renders the site footer with a rolling newsticker.

 @returns HTML string for the `<site-footer>` element

 @example
 ```ts
 const markup = html();
 ```
 */
export function html(): string {
  return h({
    tag: 'site-footer',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'footer',
        children: [
          h({
            tag: 'div',
            class: 'ticker-stack',
            children: TICKER_QUOTES.map(function tickerLine(quote,) {
              return h({
                tag: 'p',
                text: quote,
              },);
            },),
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
