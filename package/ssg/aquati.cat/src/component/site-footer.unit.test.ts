/**
 Tests for the footer newsticker's self-sizing slot and animation timing.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  css,
  html,
} from './site-footer.ts';

//region Fixtures

/**
 Number of quotes the ticker cycles through.
 */
const QUOTE_COUNT = 9;

/**
 Per-quote `animation-delay` seconds, restated independently of the component.

 Every entry is negative on purpose. A non-negative delay would leave that quote
 at its unstarted transform, which is the visible slot, so it would sit on top of
 the running quote until its turn arrived.
 */
const EXPECTED_DELAYS = [
  -32,
  -28.5,
  -25,
  -21.5,
  -18,
  -14.5,
  -11,
  -7.5,
  -4,
] as const;

/**
 Seconds between one quote reaching the slot and the next reaching it.
 */
const QUOTE_INTERVAL = 3.5;

/**
 Seconds every delay is shifted back so even the last quote starts negative.
 */
const CYCLE_SHIFT = 32;

/**
 Opening of the paragraph rule carrying the per-quote delays.
 */
const DELAY_SELECTOR = 'site-footer .ticker-stack > p:nth-child(';

//endregion Fixtures

await describe({
  name: 'site footer newsticker',
  children: [
    it({
      name: 'renders one paragraph per quote inside the ticker stack',
      fn: async function rendersOneParagraphPerQuote(): Promise<void> {
        /**
         Rendered footer markup.
         */
        const markup = html();

        expect(markup.split('<p>',).length - 1,).toBe(QUOTE_COUNT,);
        expect(markup,).toContain('class="ticker-stack"',);
      },
    },),
    it({
      name: 'stacks every quote in one grid cell so the slot sizes itself',
      fn: async function stacksEveryQuoteInOneGridCell(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles,).toContain('site-footer .ticker-stack{display:grid}',);
        expect(styles,).toContain('grid-row-start:1;grid-column-start:1',);
      },
    },),
    it({
      name: 'declares no block-size, leaving the slot to the tallest quote',
      fn: async function declaresNoBlockSize(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles.includes('block-size:',),).toBe(false,);
        expect(styles,).toContain('overflow-block:clip',);
      },
    },),
    it({
      name: 'registers the seed so engines without random() start at the first quote',
      fn: async function registersTheSeedProperty(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles,).toContain(
          '@property --ticker-seed{syntax:"<integer>";inherits:true;initial-value:0}',
        );
      },
    },),
    it({
      name: 'draws the seed with the current random() grammar',
      fn: async function drawsTheSeedWithCurrentGrammar(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles,).toContain('--ticker-seed:random(--ticker-seed, 0, 8, 1)',);
        expect(styles.includes(', by ',),).toBe(false,);
      },
    },),
    it({
      name: 'gives every quote a negative delay offset by the seed',
      fn: async function givesEveryQuoteANegativeDelay(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        EXPECTED_DELAYS.forEach(function assertDelay(
          seconds: number,
          index: number,
        ): void {
          expect(seconds < 0,).toBe(true,);
          expect(styles,).toContain(
            `${DELAY_SELECTOR}${index + 1}){animation-delay:calc(${seconds}s - var(--ticker-seed) * ${QUOTE_INTERVAL}s)}`,
          );
        },);
      },
    },),
    it({
      name: 'spaces consecutive quote delays by exactly one interval',
      fn: async function spacesConsecutiveDelaysByOneInterval(): Promise<void> {
        EXPECTED_DELAYS.forEach(function assertSpacing(
          seconds: number,
          index: number,
        ): void {
          /**
           Delay this quote should carry if the sequence is evenly spaced, rounded
           past the binary floating point error repeated 3.5 addition introduces.
           */
          const evenlySpaced = Number(
            ((index * QUOTE_INTERVAL) - CYCLE_SHIFT).toFixed(2,),
          );

          expect(seconds,).toBe(evenlySpaced,);
        },);
      },
    },),
    it({
      name: 'jumps a departed quote back below the slot instead of sweeping it down',
      fn: async function jumpsDepartedQuoteBackBelow(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles,).toContain(
          '12.6984%{transform:translateY(-100%);animation-timing-function:step-end}',
        );
        expect(styles,).toContain('100%{transform:translateY(100%)}',);
      },
    },),
    it({
      name: 'pauses rather than animates under prefers-reduced-motion',
      fn: async function pausesUnderReducedMotion(): Promise<void> {
        /**
         Generated component stylesheet.
         */
        const styles = css();

        expect(styles,).toContain(
          '@media (prefers-reduced-motion: reduce){site-footer .ticker-stack > p{animation-play-state:paused}}',
        );
      },
    },),
  ],
},);
