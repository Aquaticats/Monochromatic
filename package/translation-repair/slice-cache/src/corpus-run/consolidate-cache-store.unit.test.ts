/**
 Tests for the store that resumes a settlement an earlier run bought.

 WHY THE SHAPE IS CHECKED AT ALL, and why more strictly than the contest
 store's: a settlement carries `text` that SHIPS. The record built from one
 hands that text to the assembly whenever the terminal says a consolidation
 won, so this store is the single path on which bytes read off disk become
 corpus text in an artifact. A file that was truncated, hand-edited, or
 written by a different schema would carry them there with nothing else in the
 way. Refusing it costs one re-asked slice.

 THE ABSENT GATE IS THE CASE MOST LIKELY TO BE BROKEN BY A STRICTER GUARD, so
 it is pinned here: a slice the validity floor stopped never reached the gate,
 and a store that required the key would refuse every floored slice and
 re-buy it every run.

 Fixtures are cat-themed invention written into throwaway directories.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { openConsolidateCache, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Built pipeline the fixtures are filled under.
 */
const TEST_GENERATION = `sha256-tree-v1:${'a'.repeat(64,)}`;

/**
 Key every case writes and reads under.
 */
const CAT_KEY = 'c'.repeat(64,);

/**
 One gate ballot, carrying every field the loader checks.
 */
const CAT_BALLOT = {
  choice: 'consolidated',
  unsupported: [],
  unsupportedRaw: [],
  dropped: ['standing',],
  droppedRaw: ['the standing text drops the hour she wakes',],
  reason: 'the consolidation keeps both',
};

/**
 A settlement that shipped a consolidation, as the stage writes one.
 */
const CAT_SETTLEMENT = {
  terminal: 'consolidated',
  text: 'The cat naps in the window.\nShe wakes at four.',
  floor: {
    kind: 'proposals',
    validModelIds: ['hf:cat/Cat-A',],
  },
  verdicts: [
    {
      modelId: 'hf:cat/Cat-A',
      kind: 'valid',
      findings: [],
    },
  ],
  gate: {
    choice: 'consolidated',
    ships: 'consolidated',
    ballots: [
      CAT_BALLOT,
      CAT_BALLOT,
    ],
    usable: 2,
    findings: [],
  },
  rewrapped: true,
  demoted: false,
  findings: ['cat-gather-lost-a-voice'],
};

/**
 One gate ballot preferring what already stood.
 */
const STANDING_BALLOT = {
  choice: 'standing',
  unsupported: ['consolidated',],
  unsupportedRaw: ['the consolidation adds a second nap the original never mentions',],
  dropped: [],
  droppedRaw: [],
  reason: 'the standing text says no more than the original does',
};

/**
 What the slate judges settled where a slate reached them. The store checks
 no more of it than that it is a record, since its shape is the translate
 stage's own contract.
 */
const CAT_DECIDED = {
  text: 'The cat naps in the window.',
  decision: 'judged',
  findings: [],
};

/**
 A settlement whose gate kept the standing text, as the stage writes one: the
 gate ships standing, so nothing was wrapped and nothing demoted.
 */
const KEPT_SETTLEMENT = {
  terminal: 'gate-kept-standing',
  text: 'The cat naps in the window.',
  floor: CAT_SETTLEMENT.floor,
  verdicts: CAT_SETTLEMENT.verdicts,
  decided: CAT_DECIDED,
  gate: {
    choice: 'standing',
    ships: 'standing',
    ballots: [
      STANDING_BALLOT,
      STANDING_BALLOT,
    ],
    usable: 2,
    findings: [],
  },
  rewrapped: false,
  demoted: false,
  findings: [],
};

/**
 A settlement whose gate shipped a consolidation the wrap then found to be
 the standing text in all but layout, as the stage writes one.
 */
const ERASED_SETTLEMENT = {
  terminal: 'wrap-erased-difference',
  text: 'The cat naps in the window.',
  floor: CAT_SETTLEMENT.floor,
  verdicts: CAT_SETTLEMENT.verdicts,
  decided: CAT_DECIDED,
  gate: CAT_SETTLEMENT.gate,
  rewrapped: true,
  demoted: true,
  findings: [],
};

/**
 A settlement the validity floor stopped before either round, as the stage
 writes one.
 */
const FLOORED_SETTLEMENT = {
  terminal: 'incumbent-only',
  text: 'A cat sleeps by the window.',
  floor: {
    kind: 'incumbent-only',
    refusedModelIds: ['hf:cat/Cat-A',],
  },
  verdicts: [
    {
      modelId: 'hf:cat/Cat-A',
      kind: 'invalid',
      findings: ['the page is 2 blocks and this is 1',],
    },
  ],
  rewrapped: false,
  demoted: false,
  findings: [],
};

/**
 One honest settlement per way out that ends before the gate: the floored
 slice, a slice with no standing text and no slate bought, and the three
 ways a judged slate keeps what stood.
 */
const GATELESS_SETTLEMENTS = [
  FLOORED_SETTLEMENT,
  {
    terminal: 'no-standing-text',
    text: '',
    floor: {
      kind: 'incumbent-only',
      refusedModelIds: [],
    },
    verdicts: [],
    rewrapped: false,
    demoted: false,
    findings: [],
  },
  ...['slate-endorsed-standing', 'slate-unjudged-standing', 'slate-declined-standing',].map(
    function toSlateSettlement(terminal,) {
      return {
        terminal,
        text: 'The cat naps in the window.',
        floor: CAT_SETTLEMENT.floor,
        verdicts: CAT_SETTLEMENT.verdicts,
        decided: CAT_DECIDED,
        rewrapped: false,
        demoted: false,
        findings: [],
      };
    },
  ),
];

/**
 Writes one settlement and reads the directory back through a fresh store.

 @param settlement - value to persist, valid or not

 @returns Whether a second store resumed it

 @example
 ```ts
 const resumed = await roundTrip({ settlement, },);
 ```
 */
async function roundTripValue(
  { settlement, }: { readonly settlement: unknown; },
): Promise<unknown> {
  await using scratch = await scratchDir({ prefix: 'whiskers-consolidate-', },);

  /**
   Store this run persists through.
   */
  const writing = await openConsolidateCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },);
  await writing.persist({
    key: CAT_KEY,
    serialized: JSON.stringify(settlement,),
  },);

  /**
   Store a later run would resume through.
   */
  const reading = await openConsolidateCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },);
  return reading.resumed
    .get(CAT_KEY,);
}

/**
 Writes one settlement and reports whether fresh store resumes it.

 @param settlement - value to persist, valid or not

 @returns Whether second store resumed key

 @example
 ```ts
 const resumed = await roundTrip({ settlement, });
 ```
 */
async function roundTrip(
  { settlement, }: { readonly settlement: unknown; },
): Promise<boolean> {
  return (await roundTripValue({ settlement, },)) !== undefined;
}

/**
 Writes each settlement into a store of its own and reports which a fresh
 store resumes.

 @param settlements - values to persist, each naming its terminal beside
 whatever other fields the case gives it

 @returns Whether each was resumed, keyed by its terminal

 @example
 ```ts
 const resumed = await resumedByTerminal({ settlements: GATELESS_SETTLEMENTS, },);
 ```
 */
async function resumedByTerminal(
  {
    settlements,
  }: {
    readonly settlements: readonly {
      readonly terminal: string;
      readonly [field: string]: unknown;
    }[];
  },
): Promise<Record<string, boolean>> {
  return Object.fromEntries(
    await Promise.all(settlements.map(async function toEntry(settlement,): Promise<[string, boolean,]> {
      return [
        settlement.terminal,
        await roundTrip({ settlement, },),
      ];
    },),),
  );
}

/**
 Writes each named settlement into a store of its own and reports which a
 fresh store resumes.

 @param settlements - values to persist, keyed by what each one breaks or keeps

 @returns Whether each was resumed, under the same names

 @example
 ```ts
 const resumed = await roundTripEach({ settlements: { 'standing choice over consolidated ballots': settlement, }, },);
 ```
 */
async function roundTripEach(
  { settlements, }: { readonly settlements: Readonly<Record<string, unknown>>; },
): Promise<Record<string, boolean>> {
  return Object.fromEntries(
    await Promise.all(Object.entries(settlements,)
      .map(async function toEntry([name, settlement,],): Promise<[string, boolean,]> {
        return [
          name,
          await roundTrip({ settlement, },),
        ];
      },),),
  );
}

await describe({
  name: openConsolidateCache.name,
  children: [
    it({
      name: 'RETAINS wider quorum provenance in decisive and confirmation review records',
      fn: async () => {
        /** Valid candidate-bound round with a wider three-seat basis. */
        const review = {
          quorumOver: 3,
          candidateText: CAT_SETTLEMENT.text,
          candidateDigest: hashContent({ content: CAT_SETTLEMENT.text, },),
          paragraphCount: 1,
          paragraphDigests: [hashContent({ content: CAT_SETTLEMENT.text, },),],
          seats: ['hf:cat/Cat-A', 'hf:cat/Cat-B',].map(function accepting(modelId,) {
            return { modelId, status: 'acceptable', findings: [], reason: 'Natural cat sentence.', };
          },),
          usable: 2,
          verdict: 'acceptable',
          findings: [],
        };
        /** Exact settlement bytes carried through the cache, without schema projection. */
        const settlement = {
          ...CAT_SETTLEMENT,
          polish: {
            kind: 'settled', baseText: CAT_SETTLEMENT.text, proposedText: CAT_SETTLEMENT.text,
            text: CAT_SETTLEMENT.text, changed: false, refinersHeard: [], contributors: [], rounds: [],
            review: { correctionCount: 0, corrections: [], rounds: [review,], confirmations: [review,], },
            findings: [],
          },
        };
        expect(await roundTripValue({ settlement, },),).toEqual(settlement,);
      },
    },),
    it({
      name: 'ROUND-TRIPS a settled consolidation, which is the half that fails silently: persist and '
        + 'resume disagreeing about a file name costs a re-bought slate and gate per slice per run '
        + 'and errors nowhere',
      fn: async () => {
        expect(await roundTrip({ settlement: CAT_SETTLEMENT, },),).toBe(true,);
      },
    },),

    it({
      name: 'ROUND-TRIPS SCHEMA-NINE TWO-CORRECTION AUDIT without dropping transition evidence',
      fn: async () => {
        /** Digest fixture with valid lowercase SHA-256 shape. */
        const digest = 'd'.repeat(64,);
        /** Settlement carrying exact bounded correction audit. */
        const settlement = {
          ...CAT_SETTLEMENT,
          polish: {
            kind: 'settled',
            baseText: 'The cat conducted a nap.',
            proposedText: 'The cat napped peacefully.',
            text: 'The cat napped peacefully.',
            changed: true,
            refinersHeard: ['hf:cat/Cat-A',],
            contributors: ['hf:cat/Cat-A',],
            rounds: [],
            review: {
              correctionCount: 2,
              corrections: [
                {
                  inputDigest: digest,
                  findingsDigest: digest,
                  gatedTextDigest: digest,
                },
                {
                  inputDigest: digest,
                  findingsDigest: digest,
                  gatedTextDigest: digest,
                },
              ],
              rounds: [],
            },
            findings: [],
          },
        } as const;
        expect(await roundTripValue({ settlement, },),).toEqual(settlement,);
      },
    },),

    it({
      name: 'RESUMES A SLICE THAT NEVER REACHED THE GATE, since the validity floor stopping a slate '
        + 'is the ordinary outcome where every proposal was structurally refused. A store requiring '
        + 'the gate key would re-buy a full roster every run to be told the same thing',
      fn: async () => {
        expect(await roundTrip({ settlement: FLOORED_SETTLEMENT, },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A TERMINAL THIS SCHEMA DOES NOT NAME, rather than resuming a settlement whose '
        + 'own account of how it ended nothing can read. The terminal is what decides whether the '
        + 'text ships, so an unreadable one is a slice that would ship on a coin toss',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            terminal: 'shipped-because-i-said-so',
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT WHOSE TEXT IS NOT A STRING, which is the whole reason this guard is '
        + 'stricter than the contest store: that text is written into the document when the terminal '
        + 'says a consolidation won, so a null or a number reaches the assembly as the passage',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            text: null,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A GATE WHOSE USABLE COUNT DISAGREES WITH ITS BALLOTS, mirroring the contest '
        + 'store: the count is what the quorum and the resume rule both read, so a file claiming six '
        + 'voices behind one ballot would settle a slice on a panel that never existed',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: {
              ...CAT_SETTLEMENT.gate,
              usable: 6,
            },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A BALLOT MISSING A FIELD, rather than resuming an outcome the artifact reader '
        + 'will refuse after a whole document has been paid for',
      fn: async () => {
        /**
         Ballot without the raw findings the reader requires.
         */
        const { droppedRaw: _dropped, ...partial } = CAT_BALLOT;
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: {
              ...CAT_SETTLEMENT.gate,
              ballots: [partial,],
              usable: 1,
            },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A BALLOT NAMING A RENDERING THAT DOES NOT EXIST, because the evidence fields are '
        + 'read as choices rather than as prose: a name outside consolidated, standing and neither '
        + 'would be counted as nothing and silently weaken the very evidence the gate settles on',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: {
              ...CAT_SETTLEMENT.gate,
              ballots: [
                {
                  ...CAT_BALLOT,
                  dropped: ['the other one',],
                },
              ],
              usable: 1,
            },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A FLOOR IT CANNOT READ, since the floor is what says whether any proposal '
        + 'survived validation at all, and a settlement carrying an unreadable one cannot be audited '
        + 'for the case the band pair actually hit',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            floor: { kind: 'everyone-passed', },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A decided FIELD THAT IS AN ARRAY, which this guard otherwise lets through '
        + 'unvalidated: `decided.decision` would then read undefined, never equal to any entry in '
        + 'UNSETTLED_DECISIONS, and an unsettled or corrupted round would settle silently '
        + '(ledger B92)',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            decided: ['Whiskers',],
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A GATE BALLOT THAT IS NO RECORD, since nothing there says who judged what',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: {
              ...CAT_SETTLEMENT.gate,
              ballots: [null,],
              usable: 1,
            },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A GATE BALLOT whose unsupported rendering names no choice the gate knows',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: {
              ...CAT_SETTLEMENT.gate,
              ballots: [
                {
                  ...CAT_BALLOT,
                  unsupported: ['the other one',],
                },
              ],
              usable: 1,
            },
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A SLATE FLOOR THAT IS NO RECORD, since the floor is what says the slate was stopped',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            floor: null,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A GATE OUTCOME THAT IS NO RECORD, since nothing there says what the gate settled',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            gate: null,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'RESUMES A SETTLEMENT WHOSE GATE KEPT THE STANDING TEXT, and one whose wrap erased the '
        + 'difference, each beside the gate the stage writes with it',
      fn: async () => {
        expect(await roundTrip({ settlement: KEPT_SETTLEMENT, },),).toBe(true,);
        expect(await roundTrip({ settlement: ERASED_SETTLEMENT, },),).toBe(true,);
      },
    },),

    it({
      name: 'RESUMES EVERY TERMINAL THAT ENDS BEFORE THE GATE with no gate beside it',
      fn: async () => {
        expect(await resumedByTerminal({ settlements: GATELESS_SETTLEMENTS, },),).toEqual({
          'incumbent-only': true,
          'no-standing-text': true,
          'slate-endorsed-standing': true,
          'slate-unjudged-standing': true,
          'slate-declined-standing': true,
        },);
      },
    },),

    it({
      name: 'REFUSES A TERMINAL ONLY THE GATE WRITES when no gate stands beside it, since a consolidation '
        + 'ships through the gate alone and this record says none was asked',
      fn: async () => {
        expect(
          await resumedByTerminal({
            settlements: [CAT_SETTLEMENT, ERASED_SETTLEMENT, KEPT_SETTLEMENT,].map(function withoutGate(settlement,) {
              /**
               The settlement with its gate taken out, every other field as the
               stage wrote it.
               */
              const { gate: _gate, ...gateless } = settlement;
              return gateless;
            },),
          },),
        ).toEqual({
          'consolidated': false,
          'wrap-erased-difference': false,
          'gate-kept-standing': false,
        },);
      },
    },),

    it({
      name: 'REFUSES A GATE BESIDE A TERMINAL THAT ENDS BEFORE IT, since the stage asks no gate on those '
        + 'five ways out',
      fn: async () => {
        expect(
          await resumedByTerminal({
            settlements: GATELESS_SETTLEMENTS.map(function withGate(settlement,) {
              return {
                ...settlement,
                gate: CAT_SETTLEMENT.gate,
              };
            },),
          },),
        ).toEqual({
          'incumbent-only': false,
          'no-standing-text': false,
          'slate-endorsed-standing': false,
          'slate-unjudged-standing': false,
          'slate-declined-standing': false,
        },);
      },
    },),

    it({
      name: 'REFUSES A GATE SHIPPING THE RENDERING ITS TERMINAL DOES NOT NAME: standing under a '
        + 'consolidation or an erased difference, consolidated under a kept standing',
      fn: async () => {
        expect(
          await resumedByTerminal({
            settlements: [
              {
                ...CAT_SETTLEMENT,
                gate: KEPT_SETTLEMENT.gate,
              },
              {
                ...ERASED_SETTLEMENT,
                gate: KEPT_SETTLEMENT.gate,
              },
              {
                ...KEPT_SETTLEMENT,
                gate: CAT_SETTLEMENT.gate,
              },
            ],
          },),
        ).toEqual({
          'consolidated': false,
          'wrap-erased-difference': false,
          'gate-kept-standing': false,
        },);
      },
    },),

    it({
      name: 'REFUSES A GATE WHOSE CHOICE ITS OWN BALLOTS DO NOT GIVE, since the stage settles the choice from '
        + 'the ballots it stores and nothing rewrites it afterwards: a file naming a choice no count of its '
        + 'ballots reaches is a verdict nobody can recompute',
      fn: async () => {
        expect(await roundTripEach({
          settlements: {
            'standing choice over two consolidated ballots': {
              ...CAT_SETTLEMENT,
              gate: {
                ...CAT_SETTLEMENT.gate,
                choice: 'standing',
              },
            },
            'consolidated choice over two standing ballots': {
              ...KEPT_SETTLEMENT,
              gate: {
                ...KEPT_SETTLEMENT.gate,
                choice: 'consolidated',
              },
            },
            'neither choice over two consolidated ballots': {
              ...CAT_SETTLEMENT,
              gate: {
                ...CAT_SETTLEMENT.gate,
                choice: 'neither',
              },
            },
            'consolidated choice over a split pair of ballots': {
              ...CAT_SETTLEMENT,
              gate: {
                ...CAT_SETTLEMENT.gate,
                ballots: [CAT_BALLOT, STANDING_BALLOT,],
              },
            },
          },
        },),).toEqual({
          'standing choice over two consolidated ballots': false,
          'consolidated choice over two standing ballots': false,
          'neither choice over two consolidated ballots': false,
          'consolidated choice over a split pair of ballots': false,
        },);
      },
    },),

    it({
      name: 'RESUMES A GATE WHOSE CHOICE ITS BALLOTS GIVE EVEN WHERE THE CHOICE AND WHAT SHIPS DIFFER, which '
        + 'is a split pair settling on neither beside a kept standing text, and a standing choice shipping '
        + 'the consolidation because the standing was forfeit',
      fn: async () => {
        expect(await roundTripEach({
          settlements: {
            'split pair settling on neither': {
              ...KEPT_SETTLEMENT,
              gate: {
                ...KEPT_SETTLEMENT.gate,
                choice: 'neither',
                ballots: [CAT_BALLOT, STANDING_BALLOT,],
              },
            },
            'standing choice over a forfeit standing': {
              ...CAT_SETTLEMENT,
              gate: {
                ...CAT_SETTLEMENT.gate,
                choice: 'standing',
                ballots: [STANDING_BALLOT, STANDING_BALLOT,],
              },
            },
          },
        },),).toEqual({
          'split pair settling on neither': true,
          'standing choice over a forfeit standing': true,
        },);
      },
    },),

    it({
      name: 'REFUSES A DEMOTED FLAG THAT DISAGREES WITH ITS TERMINAL, since the stage records a demotion only '
        + 'as the terminal wrap-erased-difference and the artifact reports the flag: demoted beside any other '
        + 'terminal, or the erased difference with no demotion, is a record the stage did not write',
      fn: async () => {
        expect(await roundTripEach({
          settlements: {
            'consolidated demoted': {
              ...CAT_SETTLEMENT,
              demoted: true,
            },
            'kept standing demoted': {
              ...KEPT_SETTLEMENT,
              demoted: true,
            },
            'floored demoted': {
              ...FLOORED_SETTLEMENT,
              demoted: true,
            },
            'erased difference not demoted': {
              ...ERASED_SETTLEMENT,
              demoted: false,
            },
          },
        },),).toEqual({
          'consolidated demoted': false,
          'kept standing demoted': false,
          'floored demoted': false,
          'erased difference not demoted': false,
        },);
      },
    },),

    it({
      name: 'REFUSES A REWRAP BESIDE A TERMINAL THAT NEVER WRAPS, and resumes a consolidation that was not '
        + 'rewrapped, since the stage wraps only a consolidation it ships or erases and the artifact '
        + 'reports the flag',
      fn: async () => {
        expect(await roundTripEach({
          settlements: {
            'kept standing rewrapped': {
              ...KEPT_SETTLEMENT,
              rewrapped: true,
            },
            'floored rewrapped': {
              ...FLOORED_SETTLEMENT,
              rewrapped: true,
            },
            'consolidated not rewrapped': {
              ...CAT_SETTLEMENT,
              rewrapped: false,
            },
          },
        },),).toEqual({
          'kept standing rewrapped': false,
          'floored rewrapped': false,
          'consolidated not rewrapped': true,
        },);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT MARKED ARCHIVE-KEPT, since the stage never persists one: the mark is what '
        + 'makes the archive ship, so a copy found on disk was written by something else',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...CAT_SETTLEMENT,
            archiveKept: true,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT MARKED BENCH-BELOW-FLOOR, since the stage never persists one: no judge was asked '
        + 'for it, so a copy found on disk was written by something else and a later pass over an unsettled '
        + 'entry would resume a standing nobody judged',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...KEPT_SETTLEMENT,
            benchBelowFloor: true,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT THAT SHIPPED THE TRANSLATE LANE WINNER, since the stage never persists one: the final '
        + 'round could not seat two judges, so a copy found on disk was written by something else and a later pass '
        + 'would ship the winner without asking a bench that could seat two',
      fn: async () => {
        expect(await roundTrip({
          settlement: {
            ...KEPT_SETTLEMENT,
            translateWinnerShipped: true,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A SETTLEMENT THAT IS NO RECORD AT ALL',
      fn: async () => {
        expect(await roundTrip({ settlement: null, },),).toBe(false,);
      },
    },),
  ],
},);
