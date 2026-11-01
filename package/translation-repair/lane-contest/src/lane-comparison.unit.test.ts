/**
 Tests for the slice-by-slice comparison of the two lanes.

 The comparison exists to answer one question, whether repair and translate
 produce the same English where both touch a slice, and it has one hard
 requirement: it must read what each DOCUMENT carries rather than what each
 lane chose. A slice whose replacement the assembly guard withdrew chose one
 thing and shipped another, and a comparison that read the choice would report
 a rewrite no reader ever saw.

 It takes each lane's DELIVERY LEDGER rather than its wordings and an index
 set. The ledger has already refused a decided slice that is neither shipped,
 withdrawn, nor blocked, so what a document carries arrives as a stated fact;
 reading an index set here meant an omitted shipped index was indistinguisable
 from a lane that kept the archive.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type SliceDeliveryRecord,
  WordingCoherenceError,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  assertPreparationIdentity,
  type PreparationIdentity,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  compareDocumentLanes,
  DeliveryCoherenceError,
  type IdentifiedDeliveryLedger,
  LaneComparisonError,
} from '../dist/final/node/index.mjs';

/**
 Archive wording of the one slice most cases here use.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Original of that slice.
 */
const SOURCE_NAP = '猫猫在窗台上睡觉。';

/**
 Slicing every case here claims to describe.

 Built as a literal rather than from a preparation, because these cases are
 about the join and not about what names it; the validator is what makes it a
 real identity rather than a bare string.
 */
const NAMED_SLICING = `sha256-preparation-v1:${'a'.repeat(64,)}`;
assertPreparationIdentity(NAMED_SLICING,);

/**
 Same value with the narrowing written down, since an assertion at module
 scope does not reach inside a function declaration.
 */
const SLICING: PreparationIdentity = NAMED_SLICING;

/**
 Stamps a set of rows with the slicing every case here shares.

 @param records - rows of one lane's ledger

 @returns Those rows, under this file's slicing

 @example
 ```ts
 const ledger = ledgerOf({ records, },);
 ```
 */
function ledgerOf(
  { records, }: { readonly records: readonly SliceDeliveryRecord[]; },
): IdentifiedDeliveryLedger {
  return {
    preparationIdentity: SLICING,
    records,
  };
}

/**
 Builds one lane's ledger over a single decided slice.

 @param acceptedText - wording that lane decided on

 @param shipped - whether the returned document carries it

 @returns Ledger shaped as `buildSliceDelivery` returns one, under the
 slicing every case here shares

 @example
 ```ts
 const lane = laneOf({ acceptedText: 'The cat naps.', shipped: true, },);
 ```
 */
function laneOf(
  {
    acceptedText,
    shipped,
  }: {
    readonly acceptedText: string;
    readonly shipped: boolean;
  },
): IdentifiedDeliveryLedger {
  /**
   Whether the lane moved off the archive at all, which decides whether the
   unshipped case is a withdrawal or an ordinary keep.
   */
  const moved = acceptedText !== ARCHIVE_NAP;
  return ledgerOf({ records: [{
    sliceIndex: 0,
    sourceText: SOURCE_NAP,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    outcome: {
      kind: 'decided',
      acceptedText,
    },
    shippedText: shipped ? acceptedText : ARCHIVE_NAP,
    delivery: shipped
      ? { kind: 'replacement-shipped', }
      : (moved
        ? {
          kind: 'replacement-withdrawn',
          reason: 'assembly-integrity',
        }
        : { kind: 'incumbent-retained', }),
  },], },);
}

/**
 Builds one lane's ledger over a single slice it did not decide.

 @param outcome - what that lane did instead

 @param incumbentKind - whether the archive holds wording at this slice

 @returns Ledger carrying whatever the archive has there

 @example
 ```ts
 const lane = undecidedLaneOf({ outcome: { kind: 'not-evaluated', }, incumbentKind: 'present', },);
 ```
 */
function undecidedLaneOf(
  {
    outcome,
    incumbentKind,
  }: {
    readonly outcome: SliceDeliveryRecord['outcome'];
    readonly incumbentKind: 'present' | 'absent';
  },
): IdentifiedDeliveryLedger {
  /**
   Archive wording here, which an anchor does not have.
   */
  const incumbentText = (incumbentKind === 'absent') ? '' : ARCHIVE_NAP;
  return ledgerOf({ records: [{
    sliceIndex: 0,
    sourceText: SOURCE_NAP,
    incumbentKind,
    incumbentText,
    outcome,
    shippedText: incumbentText,
    delivery: (incumbentKind === 'absent')
      ? { kind: 'gap-remains', }
      : { kind: 'incumbent-retained', },
  },], },);
}

/**
 Builds one row where the lane examined the archive's wording and kept it, at
 a given slice, for the cases about which slices two ledgers name.

 @param sliceIndex - slice the row is for

 @returns Row a ledger could hold at any position

 @example
 ```ts
 const row = keptAt({ sliceIndex: 2, },);
 ```
 */
function keptAt({ sliceIndex, }: { readonly sliceIndex: number; },): SliceDeliveryRecord {
  return {
    sliceIndex,
    sourceText: SOURCE_NAP,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    outcome: {
      kind: 'decided',
      acceptedText: ARCHIVE_NAP,
    },
    shippedText: ARCHIVE_NAP,
    delivery: { kind: 'incumbent-retained', },
  };
}

await describe({
  name: compareDocumentLanes.name,
  children: [
    it({
      name:
        'names the four ways two documents can differ on a slice: neither moved, one moved, '
          + 'both moved to the same wording, and both moved apart, which is the only one a human has to read',
      fn: async () => {
        /**
         Both lanes left the archive wording standing.
         */
        const kept = compareDocumentLanes({
          repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
          translate: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
        },);
        expect(kept.slices[0]?.verdict,).toBe('archive-stands',);

        /**
         Only repair changed the slice.
         */
        const repairOnly = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: true, },),
          translate: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
        },);
        expect(repairOnly.slices[0]?.verdict,).toBe('repair-only',);

        /**
         Only translate changed it.
         */
        const translateOnly = compareDocumentLanes({
          repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
          translate: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
        },);
        expect(translateOnly.slices[0]?.verdict,).toBe('translate-only',);

        /**
         Both changed it the same way, character for character.
         */
        const agreed = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
          translate: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
        },);
        expect(agreed.slices[0]?.verdict,).toBe('both-agree',);

        /**
         Both changed it, differently.
         */
        const apart = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: true, },),
          translate: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
        },);
        expect(apart.slices[0]?.verdict,).toBe('both-differ',);
        expect(apart.slices[0]?.repairText,).toBe('The cat is asleep on the windowsill.',);
        expect(apart.slices[0]?.translateText,).toBe('A cat dozes in the window.',);
      },
    },),
    it({
      name:
        'carries the slicing into the result, so no writer can persist rows without the thing that '
          + 'numbers them: rows joined on a slice index mean nothing without the preparation, and one '
          + 'persisted alone reads against a later preparation of the same entry with every row still '
          + 'looking well formed',
      fn: async () => {
        /**
         Comparison over one unchanged slice.
         */
        const comparison = compareDocumentLanes({
          repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
          translate: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
        },);
        expect(comparison.preparationIdentity,).toBe(SLICING,);
      },
    },),
    it({
      name:
        'reads what the DOCUMENT carries rather than what the lane chose: a slice whose replacement '
          + 'the assembly guard withdrew compares as the archive wording, even though its record names a rewrite, '
          + 'because reporting the rewrite would credit a lane with English no reader ever saw',
      fn: async () => {
        /**
         Repair chose a rewrite the guard took back; translate shipped one.
         */
        const comparison = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: false, },),
          translate: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
        },);
        expect(comparison.slices[0]?.verdict,).toBe('translate-only',);

        // The withdrawn wording is nowhere in the carried text: what repair
        // CARRIES is the archive text, and that is the only repair-side text a
        // comparison may state. Its own decision is still on the row, and so is
        // the route by which the document came to lack it.
        expect(comparison.slices[0]?.repairText,).toBe(ARCHIVE_NAP,);
        expect(comparison.slices[0]?.repairDelivery,).toEqual({
          kind: 'replacement-withdrawn',
          reason: 'assembly-integrity',
        },);
        expect(comparison.slices[0]?.repairOutcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The cat is asleep on the windowsill.',
        },);
      },
    },),
    it({
      name:
        'REFUSES two ledgers whose slice counts differ, since a shorter one means the lanes ran over '
          + 'different preparations and every row after the first gap compares two different passages',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
            translate: ledgerOf({ records: [], },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe('lanes report 1 and 0 slices, so they ran over different preparations',);
      },
    },),
    it({
      name:
        'REFUSES two ledgers that disagree about a slice`s archive wording, which is the same defect '
          + 'arriving with matching counts and is otherwise undetectable downstream',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
            translate: ledgerOf({ records: [{
              sliceIndex: 0,
              sourceText: SOURCE_NAP,
              incumbentKind: 'present',
              incumbentText: 'A different archive sentence entirely.',
              outcome: {
                kind: 'decided',
                acceptedText: 'A cat dozes in the window.',
              },
              shippedText: 'A cat dozes in the window.',
              delivery: { kind: 'replacement-shipped', },
            },], },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,)
          .toBe('slice 0 carries a different incumbent in each lane, so the two results describe different preparations',);
      },
    },),
    it({
      name:
        'REFUSES two lanes that disagree about whether the archive translates a slice at all, EVEN WHEN '
          + 'their incumbent text matches, which is the only case that matters: a blank content slice and a '
          + 'place the archive never translated both carry the empty string, and every row took its kind '
          + 'from the repair lane',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({ records: [{
              sliceIndex: 0,
              sourceText: SOURCE_NAP,
              incumbentKind: 'present',
              incumbentText: '',
              outcome: { kind: 'not-evaluated', },
              shippedText: '',
              delivery: { kind: 'incumbent-retained', },
            },], },),
            translate: undecidedLaneOf({
              outcome: { kind: 'unfilled', },
              incumbentKind: 'absent',
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe(
          'the archive\'s wording at slice 0 is present in the repair lane and absent in the translate lane, so '
            + 'the two disagree about whether the archive translates it',
        );
      },
    },),
    it({
      name:
        'asserts each lane`s row against itself before joining them, since the two structural boundaries '
          + 'that take rows from a caller are this and the delivery ledger, and a row that contradicts '
          + 'itself would otherwise be compared as though it did not',
      fn: async () => {
        /**
         Failure raised by a lane falling back on wording the archive lacks.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: undecidedLaneOf({
              // Nothing to fall back on: the archive holds no wording here.
              outcome: { kind: 'incumbent-fallback', },
              incumbentKind: 'absent',
            },),
            translate: undecidedLaneOf({
              outcome: { kind: 'unfilled', },
              incumbentKind: 'absent',
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(WordingCoherenceError,);
        expect((caught as Error).message,)
          .toBe('slice 0 reports the archive\'s wording standing by default, and the archive holds none',);
      },
    },),
    it({
      name:
        'separates a lane that LOOKED and kept the archive wording from one that never reached the slice, '
          + 'which the repair lane`s whole-document block produces: both documents carry the archive text '
          + 'either way, and only one of them means anybody examined it',
      fn: async () => {
        /**
         Repair stopped before this slice; translate looked and kept it.
         */
        const comparison = compareDocumentLanes({
          repair: undecidedLaneOf({
            outcome: { kind: 'not-evaluated', },
            incumbentKind: 'present',
          },),
          translate: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
        },);
        expect(comparison.slices[0]?.verdict,).toBe('archive-stands',);
        expect(comparison.slices[0]?.repairOutcome
          .kind,).toBe('not-evaluated',);
        expect(comparison.slices[0]?.translateOutcome
          .kind,).toBe('decided',);
        expect(comparison.slices[0]?.repairText,).toBe(ARCHIVE_NAP,);
        // The two lanes did different things here, so their DECISIONS are not
        // comparable however alike the two documents read.
        expect(comparison.slices[0]?.decisionComparison
          .kind,).toBe('not-comparable',);
      },
    },),
    it({
      name:
        'reports a passage NEITHER lane filled as a gap that still remains rather than as the archive '
          + 'standing, since the archive has never translated it: the older verdict told a grader a '
          + 'translation was being kept where none has ever existed',
      fn: async () => {
        /**
         Both lanes reached the anchor and neither filled it.
         */
        const comparison = compareDocumentLanes({
          repair: undecidedLaneOf({
            outcome: { kind: 'unfilled', },
            incumbentKind: 'absent',
          },),
          translate: undecidedLaneOf({
            outcome: { kind: 'unfilled', },
            incumbentKind: 'absent',
          },),
        },);
        expect(comparison.slices[0]?.verdict,).toBe('gap-remains',);
        expect(comparison.slices[0]?.incumbentKind,).toBe('absent',);
        expect(comparison.slices[0]?.repairDelivery
          .kind,).toBe('gap-remains',);
        // Neither lane decided anything, so there is nothing to compare either.
        expect(comparison.slices[0]?.decisionComparison
          .kind,).toBe('not-comparable',);
      },
    },),
    it({
      name:
        'does not call it disagreement when one lane had no work to do. The repair lane mends existing '
          + 'English, so at a passage the archive never translated it never had an opinion; reporting its '
          + 'silence as a decision made every anchor the translate lane filled read as the two lanes '
          + 'choosing different wordings',
      fn: async () => {
        /**
         Anchor the translate lane filled and the repair lane cannot touch.
         */
        const comparison = compareDocumentLanes({
          repair: undecidedLaneOf({
            outcome: { kind: 'not-applicable', },
            incumbentKind: 'absent',
          },),
          translate: ledgerOf({ records: [{
            sliceIndex: 0,
            sourceText: SOURCE_NAP,
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: {
              kind: 'decided',
              acceptedText: 'The cat has a bowl of its own.',
            },
            shippedText: 'The cat has a bowl of its own.',
            delivery: { kind: 'replacement-shipped', },
          },], },),
        },);

        // ONE lane decided, so there is nothing to compare, and the row names
        // which one rather than implying both fell short.
        expect(comparison.slices[0]?.decisionComparison,).toEqual({
          kind: 'not-comparable',
          undecidedLanes: ['repair',],
        },);
        expect(comparison.slices[0]?.verdict,).toBe('translate-only',);
        expect(comparison.slices[0]?.repairOutcome
          .kind,).toBe('not-applicable',);
      },
    },),
    it({
      name:
        'compares what the two lanes DECIDED beside what the two documents carry, because both lanes '
          + 'choosing the same wording where only one shipped it is agreement between the lanes and a '
          + 'difference between the documents, and one verdict cannot state both',
      fn: async () => {
        /**
         Both lanes chose the same replacement; the guard withdrew translate`s.
         */
        const agreed = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: true, },),
          translate: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: false, },),
        },);
        // The DOCUMENTS differ, since only one carries the replacement.
        expect(agreed.slices[0]?.verdict,).toBe('repair-only',);
        // The LANES agree, which the delivery verdict cannot say.
        expect(agreed.slices[0]?.decisionComparison,).toEqual({
          kind: 'comparable',
          verdict: 'same',
        },);

        /**
         Both lanes chose differently, and both shipped.
         */
        const apart = compareDocumentLanes({
          repair: laneOf({ acceptedText: 'The cat is asleep on the windowsill.', shipped: true, },),
          translate: laneOf({ acceptedText: 'A cat dozes in the window.', shipped: true, },),
        },);
        expect(apart.slices[0]?.decisionComparison,).toEqual({
          kind: 'comparable',
          verdict: 'different',
        },);
      },
    },),
    it({
      name:
        'REFUSES a ledger that reports one slice twice, since equal lengths are not equal coverage: two '
          + 'rows for slice 1 against rows for 1 and 2 both count two, and the join would emit slice 1 '
          + 'twice while dropping slice 2 without a symptom',
      fn: async () => {
        /**
         Failure the comparison raised. The fixture is the one the name
         describes: repair rows for slice 1 twice against translate rows for 1
         and 2. A translate ledger repeating a slice too would be refused by the
         translate check first, and this case once used exactly that pair, so
         the repair check it names never ran (ledger T8).
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 1, },),
              ],
            },),
            translate: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 2, },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe('repair lane reports 2 rows over 1 distinct slice',);
      },
    },),
    it({
      name:
        'REFUSES a translate ledger that reports one slice twice, the same check from the other end, since the '
          + 'join looks translate rows up by slice and a repeat there drops one of them just as silently',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 2, },),
              ],
            },),
            translate: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 1, },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe('translate lane reports 2 rows over 1 distinct slice',);
      },
    },),
    it({
      name:
        'REFUSES a repair slice the translate ledger does not name, which two ledgers of equal length and '
          + 'no repeats can still hold when they cover different slices',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 3, },),
              ],
            },),
            translate: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 2, },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe('slice 3 is missing from the translate lane',);
      },
    },),
    it({
      name:
        'REFUSES two ledgers naming the same slices in a different order, since a ledger built over one '
          + 'preparation states them in document order, and two orders mean two preparations',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({
              records: [
                keptAt({ sliceIndex: 1, },),
                keptAt({ sliceIndex: 2, },),
              ],
            },),
            translate: ledgerOf({
              records: [
                keptAt({ sliceIndex: 2, },),
                keptAt({ sliceIndex: 1, },),
              ],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,).toBe(
          'slice 1 sits at position 0 in one ledger and elsewhere in the other, so the two are not in one '
            + 'document order',
        );
      },
    },),
    it({
      name:
        'REFUSES two ledgers that name different slicings, which is the pair no other check can catch: '
          + 'ledgers loaded from two artifacts of one entry line up perfectly, and their slice indices '
          + 'number different passages',
      fn: async () => {
        /**
         A slicing that is not this file`s.
         */
        const otherSlicing = `sha256-preparation-v1:${'b'.repeat(64,)}`;
        assertPreparationIdentity(otherSlicing,);

        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
            translate: {
              preparationIdentity: otherSlicing,
              records: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },).records,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,)
          .toBe('the two ledgers name different slicings, so their slice indices number different passages',);
      },
    },),
    it({
      name:
        'REFUSES two ledgers that disagree about a slice`s ORIGINAL, since two preparations can pair the '
          + 'same archive wording against different source passages and every other field would still match',
      fn: async () => {
        /**
         Failure the comparison raised.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
            translate: ledgerOf({
              records: [{
                sliceIndex: 0,
                sourceText: '猫猫在吃饭。',
                incumbentKind: 'present',
                incumbentText: ARCHIVE_NAP,
                outcome: {
                  kind: 'decided',
                  acceptedText: ARCHIVE_NAP,
                },
                shippedText: ARCHIVE_NAP,
                delivery: { kind: 'incumbent-retained', },
              },],
            },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneComparisonError,);
        expect((caught as Error).message,)
          .toBe('slice 0 covers a different original in each lane, so the two results describe different preparations',);
      },
    },),
    it({
      name:
        'REFUSES a row whose delivery contradicts the wording beside it, because a record reaching here '
          + 'is a structural type rather than proof that the ledger builder made it: a row shipping wording '
          + 'it never decided is well formed in every field on its own',
      fn: async () => {
        /**
         Failure raised by a row shipping a decision it does not have.
         */
        let caught: unknown;
        try {
          compareDocumentLanes({
            repair: ledgerOf({
              records: [{
                sliceIndex: 0,
                sourceText: SOURCE_NAP,
                incumbentKind: 'present',
                incumbentText: ARCHIVE_NAP,
                outcome: { kind: 'not-evaluated', },
                shippedText: 'The cat is asleep on the windowsill.',
                delivery: { kind: 'replacement-shipped', },
              },],
            },),
            translate: laneOf({ acceptedText: ARCHIVE_NAP, shipped: false, },),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(DeliveryCoherenceError,);
        expect((caught as Error).message,).toBe(
          'slice 0 reports a replacement and an outcome of not-evaluated, so there is no decision for the '
            + 'delivery to describe',
        );
      },
    },),
    it({
      name: 'WORDS its refusal from the fault alone, as a marked class the boundary may print',
      fn: async () => {
        const error = new LaneComparisonError({
          fault: {
            kind: 'position-differs',
            sliceIndex: 3,
            position: 1,
          },
        },);
        expect(error.message,).toBe(
          'slice 3 sits at position 1 in one ledger and elsewhere in the other, so the two are not in one '
            + 'document order',
        );
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
