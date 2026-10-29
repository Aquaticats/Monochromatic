/**
 Tests for wrapping a consolidation that ships.

 WHAT THESE PIN is the pair of properties `translate-wrap.unit.test.ts` and
 `repair-wrap.unit.test.ts` pin on the two lanes, now on the stage that had
 neither: only wording the consolidation PRODUCED is wrapped, and whether it
 still differs from what stands is re-derived from the wrapped text rather
 than taken from the gate's answer.

 The demotion case is the one with teeth. The gate compares an unwrapped
 consolidation against the standing text, so it can call a difference real
 when the only difference is where the lines break. Shipping that would
 report a change nobody decided on, and would put a slice through the whole
 delivery path to arrive at the wording it started with.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type ConsolidateGateOutcome,
  wrapConsolidation,
} from '../dist/final/node/index.mjs';

/**
 Logger these hand to the stage, whose output is not what is under test.
 */
const l = tagged({ tag: 'consolidate-wrap-test', },);

/**
 Builds a gate outcome that settled the way a case needs.

 @param ships - rendering the gate settled on

 @returns Outcome shaped as the gate produces one

 @example
 ```ts
 const outcome = gateSettling({ ships: 'consolidated', },);
 ```
 */
function gateSettling(
  { ships, }: { readonly ships: 'consolidated' | 'standing'; },
): ConsolidateGateOutcome {
  return {
    choice: (ships === 'consolidated') ? 'consolidated' : 'standing',
    ships,
    ballots: [],
    usable: 0,
    findings: [],
  };
}

/**
 One passage as a producer that ignored the rule would emit it.
 */
const ONE_LONG_LINE = 'The cat naps in the window. She wakes at four. She asks for nothing at all.';

/**
 One line-structured passage as it stands in a page nobody has wrapped.

 FLAT ON PURPOSE, and this is the ordinary case rather than a contrived one:
 50 of 64 archive incumbents already violate the line rule, and a retention
 keeps the archive's bytes.
 */
const VERSE_AS_IT_STANDS = [
  'The cat wakes. Sun is warm.',
  '',
  'She counts birds. Counts again.',
  '',
  'The kettle ticks. Time holds.',
  '',
  'A door swings. She stays.',
  '',
  'The sun moves. She follows.',
  '',
  'Dust settles. She sleeps.',
].join('\n',);

/**
 Same passage as a producer obeying the line-structure rule returns it, one
 line per unit.
 */
const VERSE_UNMERGED = [
  'The cat wakes.\nSun is warm.',
  '',
  'She counts birds.\nCounts again.',
  '',
  'The kettle ticks.\nTime holds.',
  '',
  'A door swings.\nShe stays.',
  '',
  'The sun moves.\nShe follows.',
  '',
  'Dust settles.\nShe sleeps.',
].join('\n',);
await describe({
  name: wrapConsolidation.name,
  children: [
    it({
      name: 'LEAVES THE STANDING TEXT BYTE FOR BYTE when the gate settled on it, because wrapping a '
        + 'slice that decided to change nothing turns that decision into a change, which is the one '
        + 'edit no reader can usefully judge and the delivery check refuses outright',
      fn: async () => {
        /**
         Wording already in place, written the way a producer would not.
         */
        const standingText = ONE_LONG_LINE;

        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'standing', },),
          consolidatedText: 'The cat naps.\nShe wakes at four.',
          standingText,
          lineStructured: false,
          l,
        },);

        expect(shipped.ships,).toBe('standing',);
        expect(shipped.text,).toBe(standingText,);
        expect(shipped.rewrapped,).toBe(false,);
        expect(shipped.demoted,).toBe(false,);
      },
    },),

    it({
      name: 'WRAPS A CONSOLIDATION THE PRODUCER EMITTED AS ONE LINE, which is the whole defect: both '
        + 'lanes wrap at their assembly step and this stage has none, so seven of eleven and seven of '
        + 'twelve shipped consolidations across the band pair carried the wrapping of whichever model '
        + 'happened to answer',
      fn: async () => {
        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: ONE_LONG_LINE,
          standingText: 'The cat sleeps somewhere else entirely.',
          lineStructured: false,
          l,
        },);

        expect(shipped.ships,).toBe('consolidated',);
        expect(shipped.rewrapped,).toBe(true,);
        expect(shipped.demoted,).toBe(false,);
        expect(shipped.text.split('\n',).length,).toBeGreaterThan(1,);
        expect(shipped.text.includes('She asks for nothing at all.',),).toBe(true,);
      },
    },),

    it({
      name: 'REPORTS NO REWRAP for a producer that already wrote it correctly, so a reader can tell a '
        + 'roster that honours the rule from one this stage is silently correcting',
      fn: async () => {
        /**
         The same passage, already written the way the rule would have it.
         */
        const alreadyWrapped = 'The cat naps in the window.\nShe wakes at four.\nShe asks for nothing at all.';

        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: alreadyWrapped,
          standingText: 'The cat sleeps somewhere else entirely.',
          lineStructured: false,
          l,
        },);

        expect(shipped.text,).toBe(alreadyWrapped,);
        expect(shipped.rewrapped,).toBe(false,);
        expect(shipped.demoted,).toBe(false,);
      },
    },),

    it({
      name: 'SENDS THE SLICE BACK TO ITS STANDING TEXT when wrapping erases the difference, because '
        + 'the gate compared an unwrapped consolidation and can call a line break a change. Without '
        + 'this a slice travels the whole delivery path to arrive at the wording it started with, and '
        + 'is recorded as a consolidation that shipped',
      fn: async () => {
        /**
         What stands, written the way the rule would have it.
         */
        const standingText = 'The cat naps in the window.\nShe wakes at four.\nShe asks for nothing at all.';

        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: ONE_LONG_LINE,
          standingText,
          lineStructured: false,
          l,
        },);

        expect(shipped.ships,).toBe('standing',);
        expect(shipped.text,).toBe(standingText,);
        expect(shipped.demoted,).toBe(true,);
      },
    },),

    it({
      name: 'DEMOTES AGAINST AN UNWRAPPED STANDING TEXT TOO, which is the case the standing text '
        + 'itself creates: where a lane contest settled on the incumbent, what stands is the archive '
        + 'wording, and nothing has ever wrapped that. A consolidation that is a pure re-wrapping of '
        + 'the archive matches no key this stage holds unless the standing text is wrapped to compare',
      fn: async () => {
        /**
         What stands, as the archive carries it: one line, never wrapped.
         */
        const standingText = ONE_LONG_LINE;

        /**
         The same wording, wrapped, which is what a producer handed back.
         */
        const consolidatedText = 'The cat naps in the window.\nShe wakes at four.\nShe asks for nothing at all.';

        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText,
          standingText,
          lineStructured: false,
          l,
        },);

        expect(shipped.ships,).toBe('standing',);
        expect(shipped.text,).toBe(standingText,);
        expect(shipped.demoted,).toBe(true,);
        expect(shipped.rewrapped,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES A CONSOLIDATION WHOSE WORDING IS THE STANDING TEXT WITH ITS SOFT LINE BREAKS ELSEWHERE, '
        + 'which no wrap of either text makes equal, since the wrap only adds breaks: the site renders a soft '
        + 'break as a space, so shipping it would report a change the page does not show (ledger B26)',
      fn: async () => {
        /**
         What stands, with a soft break the wrap would never put there.
         */
        const standingText = 'The cat naps on the mat\nall afternoon.';

        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: 'The cat naps on the mat all afternoon.',
          standingText,
          lineStructured: false,
          l,
        },);

        expect(shipped.ships,).toBe('standing',);
        expect(shipped.text,).toBe(standingText,);
        expect(shipped.demoted,).toBe(true,);
      },
    },),

    it({
      name: 'DEMOTES A GOVERNED CONSOLIDATION THAT IS THE STANDING TEXT BUT FOR A TRAILING NEWLINE, which the '
        + 'slate already reads as one wording; the producer\'s own lines still ship, as "SHIPS THE '
        + 'PRODUCER\'S OWN LINES ON A LINE-STRUCTURED SLICE" pins (ledger B26)',
      fn: async () => {
        const trailing = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: `${VERSE_UNMERGED}\n`,
          standingText: VERSE_UNMERGED,
          lineStructured: true,
          l,
        },);

        expect(trailing.ships,).toBe('standing',);
        expect(trailing.text,).toBe(VERSE_UNMERGED,);
        expect(trailing.demoted,).toBe(true,);
      },
    },),

    it({
      name: 'STILL REPORTS THE REWRAP ON A DEMOTED SLICE, since the two facts answer different '
        + 'questions: whether the rule had to correct the producer, and whether anything survived it',
      fn: async () => {
        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: ONE_LONG_LINE,
          standingText: 'The cat naps in the window.\nShe wakes at four.\nShe asks for nothing at all.',
          lineStructured: false,
          l,
        },);

        expect(shipped.rewrapped,).toBe(true,);
        expect(shipped.demoted,).toBe(true,);
      },
    },),
    it({
      name:
        'SHIPS THE PRODUCER\'S OWN LINES ON A LINE-STRUCTURED SLICE, which is the fixture the '
          + 'audit owed and could only read off the code. The standing text here is FLAT, so wrapping '
          + 'it for comparison reproduces exactly the lines a producer obeying the rule emits; the '
          + 'wrapped path then demotes on that collision and ships the flat text, discarding the exact '
          + 'repair the rule demands. Measured against the code before the skip, not inferred from it',
      fn: async () => {
        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: VERSE_UNMERGED,
          standingText: VERSE_AS_IT_STANDS,
          lineStructured: true,
          l,
        },);

        expect(shipped.ships,).toBe('consolidated',);
        expect(shipped.text,).toBe(VERSE_UNMERGED,);
        expect(shipped.demoted,).toBe(false,);
        expect(shipped.rewrapped,).toBe(false,);
      },
    },),

    it({
      name:
        'STILL DEMOTES A GOVERNED PROPOSAL THAT IS THE STANDING TEXT, comparing the two raw. '
          + 'Skipping the wrap removes every way a re-wrapping could collide, but not the plain case '
          + 'of a proposal that changes nothing: a replacement identical to its incumbent survives the '
          + 'footnote guard and lands in the shipped set beside a document nobody changed',
      fn: async () => {
        const shipped = wrapConsolidation({
          outcome: gateSettling({ ships: 'consolidated', },),
          consolidatedText: VERSE_UNMERGED,
          standingText: VERSE_UNMERGED,
          lineStructured: true,
          l,
        },);

        expect(shipped.ships,).toBe('standing',);
        expect(shipped.text,).toBe(VERSE_UNMERGED,);
        expect(shipped.demoted,).toBe(true,);
      },
    },),
  ],
},);
