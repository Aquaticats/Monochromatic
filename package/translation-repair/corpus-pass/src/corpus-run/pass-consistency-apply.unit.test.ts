/**
 Tests for applying every group's replacements slice by slice and putting each
 changed slice through the floors: where two groups meet in one slice, where
 their words overlap, what a floor's refusal keeps, what each place and each
 changed slice is recorded as, and the invariants a settled group must hold.

 The groups are settled from scripted ballots over the cat page, so the
 replacements are the ones the pass would plan.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { repeatGroupsOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { ConsistencyInvariantError, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ArtifactConsistencyGroup,
  parseConsistency,
} from '@monochromatic-dev/translation-repair-artifact';
import {
  applyConsistency,
  consistencyFloorVerdict,
  type ConsistencySliceFacts,
  consistencySlicesOf,
  type DecidedGroup,
  type PendingEdit,
  placesOf,
  repeatOf,
  settleAsked,
  settleCarried,
  type SettledGroup,
  type UnitCarried,
  unplacedOf,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import {
  archiveEnglishOf,
  preparedCatPage,
  projectedCatPage,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import {
  REFRAIN_QUOTES,
  SENTENCE_QUOTES,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-quotes.test-fixture.ts';

/**
 What the pass made of each slice of the cat page when the archive stands everywhere.
 */
const ENGLISH = archiveEnglishOf({ prepared: preparedCatPage(), },);

/**
 The refrain's quotes with the third place located over words that run on into the sentence's second place.
 */
const WIDENED_REFRAIN_QUOTES = REFRAIN_QUOTES.map(function widened(quote,): Record<string, string> {
  return (quote.label === 'OCCURRENCE 3')
    ? {
      label: 'OCCURRENCE 3',
      quote: 'the fish snacks smiled too. So this cat has to be a sweet little darling.',
      slot: 'sweet',
    }
    : quote;
},);

/**
 The ballots of three judges who all choose one place and keep none.

 @param quotes - quotes every judge gives

 @param choice - label every judge chooses

 @returns Voices, in seat order

 @example
 ```ts
 const voices = alikeVoices({ quotes: SENTENCE_QUOTES, choice: 'OCCURRENCE 1', },);
 ```
 */
function alikeVoices(
  {
    quotes,
    choice,
  }: {
    readonly quotes: readonly Record<string, string>[];
    readonly choice: string;
  },
) {
  return ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function voiceFor(seat,) {
    return {
      seat,
      reason: 'The first place words it best.',
      quotes: quotes.map(function quoted(quote,) {
        return {
          label: quote.label ?? '',
          quote: quote.quote ?? '',
          ...((quote.slot === undefined) ? {} : { slot: quote.slot, }),
        };
      },),
      choice,
      keep: [] as string[],
    };
  },);
}

/**
 The carry of every receiving place when the bench chooses the chosen words moved in by code, as the
 pass would record it where the writers wrote nothing; a place the code cannot reach keeps its English.

 @param decided - group with its receivers

 @returns One carried place per receiver, in the order of the receivers

 @example
 ```ts
 const carried = carriedBySplice({ decided, },);
 ```
 */
function carriedBySplice({ decided, }: { readonly decided: DecidedGroup; },): readonly UnitCarried[] {
  return decided.receivers
    .map(function carryOf(receiver,): UnitCarried {
      /**
       The paragraph the receiving place has now.
       */
      const current = receiver.place.slice.current.kind === 'present' ? receiver.place.slice.current.text : '';
      if (receiver.plan.kind !== 'replace') {
        return {
          carries: [
            {
              label: receiver.label,
              candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: current, },],
              ballots: [],
              usable: 0,
              quorum: 0,
              decision: { kind: 'none', reason: 'no-alternative', },
            },
          ],
          edits: [],
          findings: [],
        };
      }
      return {
        carries: [
          {
            label: receiver.label,
            candidates: [
              { label: 'CANDIDATE 1', kind: 'current', writers: [], text: current, },
              { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: receiver.plan.text, },
            ],
            ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotOf(seat,) {
              return { seat, reason: 'The chosen words belong here.', choice: 'CANDIDATE 2', };
            },),
            usable: 3,
            quorum: 2,
            decision: { kind: 'chosen', label: 'CANDIDATE 2', },
          },
        ],
        edits: [
          {
            kind: 'splice',
            placePositions: [receiver.position,],
            sliceIndex: receiver.place.slice.sliceIndex,
            start: receiver.plan.start,
            end: receiver.plan.end,
            text: receiver.plan.text,
          },
        ],
        findings: [],
      };
    },);
}

/**
 The carry of every receiving place when the bench keeps the current English everywhere, as the pass
 records a carry that reached no decision: no place of the group is replaced.

 @param decided - group with its receivers

 @returns One carried place per receiver, in the order of the receivers

 @example
 ```ts
 const carried = carriedKeepingCurrent({ decided, },);
 ```
 */
function carriedKeepingCurrent({ decided, }: { readonly decided: DecidedGroup; },): readonly UnitCarried[] {
  return decided.receivers
    .map(function carryOf(receiver,): UnitCarried {
      return {
        carries: [
          {
            label: receiver.label,
            candidates: [
              {
                label: 'CANDIDATE 1',
                kind: 'current',
                writers: [],
                text: receiver.place.slice.current.kind === 'present' ? receiver.place.slice.current.text : '',
              },
            ],
            ballots: [],
            usable: 0,
            quorum: 0,
            decision: { kind: 'none', reason: 'no-alternative', },
          },
        ],
        edits: [],
        findings: [],
      };
    },);
}

/**
 The cat page's two groups settled from scripted ballots, beside the facts of its slices.

 @param prepared - preparation of the page, the cat page by default

 @param english - what each slice would ship, the archive by default

 @param sentenceQuotes - quotes the judges give for the repeated sentence

 @param frameQuotes - quotes the judges give for the refrain

 @param choice - label the judges choose, first place by default

 @param carryOf - how the chosen words are carried to a group's receiving places, by code by default

 @returns Settled groups in page order and facts of every slice

 @example
 ```ts
 const { settled, slices, } = settledCatPage({},);
 ```
 */
function settledCatPage(
  {
    prepared = preparedCatPage(),
    english = archiveEnglishOf({ prepared, },),
    sentenceQuotes = SENTENCE_QUOTES,
    frameQuotes = REFRAIN_QUOTES,
    choice = 'OCCURRENCE 1',
    carryOf = carriedBySplice,
  }: {
    readonly prepared?: ReturnType<typeof preparedCatPage>;
    readonly english?: readonly string[];
    readonly sentenceQuotes?: readonly Record<string, string>[];
    readonly frameQuotes?: readonly Record<string, string>[];
    readonly choice?: string;
    readonly carryOf?: (arguments_: { readonly decided: DecidedGroup; },) => readonly UnitCarried[];
  },
): {
  readonly settled: readonly SettledGroup[];
  readonly slices: readonly ConsistencySliceFacts[];
} {
  const slices = consistencySlicesOf({
    prepared,
    projected: projectedCatPage({ prepared, english, },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes: new Map(),
  },);
  const text = prepared.sourceText;
  return {
    slices,
    settled: repeatGroupsOf({
      text,
      slices: prepared.slices
        .map(function sourceOf(slice,) {
          return slice.source;
        },),
    },)
      .map(function settle(group,): SettledGroup {
        const places = placesOf({ group, slices, },);
        const decided = settleAsked({
          repeat: repeatOf({ group, text, },),
          places,
          unplaced: unplacedOf({ group, },),
          asked: {
            quorum: 2,
            voices: alikeVoices({ quotes: (group.kind === 'refrain') ? frameQuotes : sentenceQuotes, choice, },),
            findings: [],
            resumed: false,
          },
        },);
        return settleCarried({ decided, carried: carryOf({ decided, },), },);
      },),
  };
}

/**
 Outcomes of every place of every group, in page order.

 @param groups - groups with final outcomes

 @returns One list of outcomes per group

 @example
 ```ts
 const outcomes = outcomesOf({ groups, },);
 ```
 */
function outcomesOf({ groups, }: { readonly groups: readonly ArtifactConsistencyGroup[]; },): readonly (readonly string[])[] {
  return groups.map(function placesOutcomes(group,): readonly string[] {
    return Array.from(
      group.occurrences,
      function outcomeOf(occurrence,): string {
        return occurrence.outcome;
      },
    );
  },);
}

/**
 The text the third slice ships once the replacements are applied, as a list of zero or one.

 @param applied - what applying came to

 @returns Text of the third slice when it changed, none when it did not

 @example
 ```ts
 const third = thirdSliceOf({ applied, },);
 ```
 */
function thirdSliceOf({ applied, }: { readonly applied: ReturnType<typeof applyConsistency>; },): readonly string[] {
  return applied.slices
    .filter(function isThird({ sliceIndex, },): boolean {
      return sliceIndex === 2;
    },)
    .map(function textOf({ text, },): string {
      return text;
    },);
}

/**
 What the artifact reader makes of the consistency section applying came to, written to JSON and read as the
 generation the pass writes.

 @param applied - what applying came to

 @param page - settled groups and slice facts the section was applied to

 @returns Section as the reader gives it back, which is the section itself when the reader accepts it

 @throws {@link ArtifactParseError} when the reader refuses the section

 @example
 ```ts
 const readBack = readBackOf({ applied, page, },);
 ```
 */
function readBackOf(
  {
    applied,
    page,
  }: {
    readonly applied: ReturnType<typeof applyConsistency>;
    readonly page: ReturnType<typeof settledCatPage>;
  },
): unknown {
  return parseConsistency({
    value: structuredClone({ groups: applied.groups, slices: applied.slices, findings: applied.findings, },),
    path: 'cat.consistency',
    generation: 17,
    sliceIndexes: new Set(page.slices.map(function indexOf({ sliceIndex, },): number {
      return sliceIndex;
    },),),
  },);
}

/**
 The section applying came to, as the reader returns it from JSON.

 @param applied - what applying came to

 @returns Section as it is written

 @example
 ```ts
 const written = writtenOf({ applied, },);
 ```
 */
function writtenOf({ applied, }: { readonly applied: ReturnType<typeof applyConsistency>; },): unknown {
  return structuredClone({ groups: applied.groups, slices: applied.slices, findings: applied.findings, },);
}

/**
 The place each overlapped place of a section says it gave way to, in page order, for every group.

 @param groups - groups with final outcomes

 @returns One list per group, naming for each place the group and label it gave way to, or none

 @example
 ```ts
 const gaveWay = gaveWayOf({ groups: applied.groups, },);
 ```
 */
function gaveWayOf({ groups, }: { readonly groups: readonly ArtifactConsistencyGroup[]; },): readonly (readonly unknown[])[] {
  return groups.map(function placesGaveWay(group,): readonly unknown[] {
    return Array.from(
      group.occurrences,
      function gaveWayTo(occurrence,): unknown {
        return occurrence.gaveWayTo ?? 'none';
      },
    );
  },);
}

/**
 Applies the replacements of a settled cat page with no declared names.

 @param page - settled groups and slice facts

 @param lines - sink for the level and text of every line the pass logs

 @returns What applying came to

 @example
 ```ts
 const applied = applyCat({ page: settledCatPage({},), lines: [], },);
 ```
 */
function applyCat(
  {
    page,
    lines,
  }: {
    readonly page: ReturnType<typeof settledCatPage>;
    readonly lines: string[];
  },
): ReturnType<typeof applyConsistency> {
  return applyConsistency({
    settled: page.settled,
    slices: page.slices,
    declared: [],
    declaredNames: [],
    l: levelCapturingLogger({ lines, },),
  },);
}

/**
 The English of the slice the three kitten groups meet in.
 */
const KITTEN_SLICE = 'Later the kitten naps in the sun.';

/**
 A group of the kitten page with a code splice over its place in the kitten slice.

 @param quote - words the judges locate for the group's place in the kitten slice

 @param splice - where the splice starts and ends, and the words it puts in

 @returns Group, whose first place stands in another slice and whose second place the splice replaces

 @example
 ```ts
 const group = kittenGroupSpliced({ quote: 'naps', splice: { start: 17, end: 21, text: 'sleeps', }, },);
 ```
 */
function kittenGroupSpliced(
  {
    quote,
    splice,
  }: {
    readonly quote: string;
    readonly splice: { readonly start: number; readonly end: number; readonly text: string; };
  },
): SettledGroup {
  return withEdit({
    group: groupIn({
      places: [
        { slice: 3, english: 'Dusk: the kitten purrs in the sun.', quote: 'in the sun', },
        { slice: 1, english: KITTEN_SLICE, quote, },
      ],
    },),
    edit: { kind: 'splice', placePositions: [1,], sliceIndex: 1, ...splice, },
  },);
}

/**
 The facts of the kitten page's two slices, whose English is what each ships now.

 @returns Facts of the kitten slice and of the slice after it

 @example
 ```ts
 const slices = kittenSlices();
 ```
 */
function kittenSlices(): readonly ConsistencySliceFacts[] {
  return [
    { sliceIndex: 1, english: KITTEN_SLICE, },
    { sliceIndex: 3, english: 'Dusk: the kitten purrs in the sun.', },
  ].map(function factsOf({ sliceIndex, english, },): ConsistencySliceFacts {
    return {
      sliceIndex,
      sourceIndex: sliceIndex,
      sourceText: '小猫打盹。',
      sourceStart: 0,
      nodes: [],
      current: { kind: 'present', text: english, },
      lineStructured: false,
      disputedWordings: [],
    };
  },);
}

//region Arrangement model
// EVERY SMALL ARRANGEMENT OF GROUPS OVER SHORT SLICES, built through `applyConsistency` itself. A group has one or
// two receiving places in each slice it stands in; each place carries no replacement, a code splice over exactly its
// located words, or a writer's block whose span holds the located words of every place it carries and starts and
// ends at a word, at the gap before a word or at the gap after one, so that spans are disjoint, touching, nested
// and partly overlapping. A block either keeps the words it replaces or drops them. Groups are chosen in every
// order, because each group's choice is drawn independently of the others'.

/**
 A short slice of the arrangement model: its English and where each word starts and ends.

 @example
 ```ts
 const slice: ArrangementSlice = { index: 1, text: 'Tiny kittens.', words: [[0, 4,], [5, 12,],], };
 ```
 */
type ArrangementSlice = {
  /**
   Index of the slice on the page.
   */
  readonly index: number;

  /**
   English of the slice.
   */
  readonly text: string;

  /**
   Start and exclusive end of each word.
   */
  readonly words: readonly (readonly [number, number])[];
};

/**
 The located words of a place: a run of the slice's words.

 @example
 ```ts
 const core: ArrangementCore = { start: 0, end: 12, };
 ```
 */
type ArrangementCore = {
  /**
   Offset where the words start.
   */
  readonly start: number;

  /**
   Exclusive offset where the words end.
   */
  readonly end: number;
};

/**
 A receiving place of an arrangement's group.

 @example
 ```ts
 const place: ArrangementPlace = { slice, core: { start: 0, end: 4, }, };
 ```
 */
type ArrangementPlace = {
  /**
   The slice the place stands in.
   */
  readonly slice: ArrangementSlice;

  /**
   The place's located words.
   */
  readonly core: ArrangementCore;
};

/**
 A replacement of an arrangement's group.

 @example
 ```ts
 const edit: ArrangementEdit = { carries: [0,], slice, kind: 'splice', start: 0, end: 4, keeps: false, };
 ```
 */
type ArrangementEdit = {
  /**
   Positions, among the group's receiving places, of the places it carries.
   */
  readonly carries: readonly number[];

  /**
   The slice it is made in.
   */
  readonly slice: ArrangementSlice;

  /**
   A code splice over one place's words, or a writer's block.
   */
  readonly kind: 'splice' | 'block';

  /**
   Offset where the replaced text starts.
   */
  readonly start: number;

  /**
   Exclusive offset where the replaced text ends.
   */
  readonly end: number;

  /**
   Whether the text that goes in keeps every word it replaces.
   */
  readonly keeps: boolean;
};

/**
 One group of an arrangement: its receiving places and the replacements it plans.

 @example
 ```ts
 const group: ArrangementGroup = { places: [], edits: [], };
 ```
 */
type ArrangementGroup = {
  /**
   Receiving places, in page order.
   */
  readonly places: readonly ArrangementPlace[];

  /**
   Replacements, none of which meets another of the group.
   */
  readonly edits: readonly ArrangementEdit[];
};

/**
 What a small model is built from: the slices and how many receiving places each group has in each.

 @example
 ```ts
 const model: ArrangementModel = { slices: [{ slice, places: 1, },], groupCounts: [1, 2,], };
 ```
 */
type ArrangementModel = {
  /**
   The slices, with the number of receiving places each group has in them.
   */
  readonly slices: readonly {
    readonly slice: ArrangementSlice;
    readonly places: 1 | 2;
  }[];

  /**
   The numbers of groups to arrange.
   */
  readonly groupCounts: readonly number[];
};

/**
 What one arrangement came to, judged against the invariant.
 */
type ArrangementVerdict = {
  /**
   Each way the arrangement broke the invariant, in words.
   */
  readonly breaches: readonly string[];

  /**
   Places of other groups whose located words a made replacement changed.
   */
  readonly changed: number;

  /**
   Of those, places recorded by the left-out branch of the arbitration.
   */
  readonly leftOut: number;

  /**
   Of those, places recorded by the branch that overlaps a later replacement with an earlier one.
   */
  readonly outlasted: number;
};

/**
 What applying every arrangement of a model came to.

 @example
 ```ts
 const outcome: ArrangementOutcome = { arrangements: 0, changed: 0, leftOut: 0, outlasted: 0, breaches: [], };
 ```
 */
type ArrangementOutcome = {
  /**
   How many arrangements were built and applied.
   */
  readonly arrangements: number;

  /**
   How many places of other groups a made replacement changed, over all of them.
   */
  readonly changed: number;

  /**
   Of those, how many the left-out branch of the arbitration recorded.
   */
  readonly leftOut: number;

  /**
   Of those, how many the branch that overlaps a later replacement with an earlier one recorded.
   */
  readonly outlasted: number;

  /**
   The first breaches of the invariant, in words.
   */
  readonly breaches: readonly string[];
};

/**
 The English of the slice every arrangement's chosen places stand in.
 */
const ARRANGEMENT_CHOSEN_ENGLISH = 'Dusk: the kitten purrs in the sun.';

/**
 The start of the shapes a group's places can take: no places yet.
 */
const NO_SHAPES: readonly (readonly ArrangementPlace[])[] = [[],];

/**
 A slice of the model made of the given words.

 @param index - index of the slice on the page

 @param words - words, which the slice's English joins with spaces and ends with a full stop

 @returns Slice with the span of each word

 @example
 ```ts
 const slice = sliceOfWords({ index: 1, words: ['Tiny', 'kittens',], });
 ```
 */
function sliceOfWords(
  {
    index,
    words,
  }: {
    readonly index: number;
    readonly words: readonly string[];
  },
): ArrangementSlice {
  return {
    index,
    text: `${words.join(' ',)}.`,
    words: words.map(function spanOf(
      word,
      position,
    ): readonly [number, number] {
      /**
       Offset where the word starts.
       */
      const start = words.slice(
        0,
        position,
      )
        .reduce(function past(
          offset,
          earlier,
        ): number {
          return offset + earlier.length + 1;
        }, 0,);
      return [start, start + word.length,];
    },),
  };
}

/**
 The sets of located words a group's places can have in a slice: runs of words that do not meet each other.

 @param slice - slice

 @param count - how many places the group has in the slice

 @returns Every set, its runs in page order

 @example
 ```ts
 const sets = coreSetsOf({ slice, count: 2, });
 ```
 */
function coreSetsOf(
  {
    slice,
    count,
  }: {
    readonly slice: ArrangementSlice;
    readonly count: 1 | 2;
  },
): readonly (readonly ArrangementCore[])[] {
  /**
   Every run of words.
   */
  const runs = slice.words.flatMap(function fromWord(
    [start,],
    first,
  ): readonly ArrangementCore[] {
    return slice.words
      .slice(first,)
      .map(function toWord([, end,],): ArrangementCore {
        return { start, end, };
      },);
  },);
  if (count === 1) {
    return runs.map(function alone(run,): readonly ArrangementCore[] {
      return [run,];
    },);
  }
  return runs.flatMap(function withLater(earlier,): readonly (readonly ArrangementCore[])[] {
    return runs
      .filter(function follows(later,): boolean {
        return earlier.end < later.start;
      },)
      .map(function pair(later,): readonly ArrangementCore[] {
        return [earlier, later,];
      },);
  },);
}

/**
 Every replacement a group could plan over its places: a splice over each place's words, and every block span
 that holds the words of the places it carries.

 @param places - group's receiving places

 @returns Replacements, each carrying places of one slice

 @example
 ```ts
 const candidates = editCandidatesOf({ places, });
 ```
 */
function editCandidatesOf({ places, }: { readonly places: readonly ArrangementPlace[]; },): readonly ArrangementEdit[] {
  return Array.from(
    { length: (2 ** places.length) - 1, },
    function carrySet(
      _mask,
      position,
    ): readonly number[] {
      return places.flatMap(function carriedBy(
        _place,
        carried,
      ): readonly number[] {
        return (Math.trunc((position + 1) / (2 ** carried)) % 2) === 1 ? [carried,] : [];
      },);
    },
  )
    .flatMap(function editsCarrying(carries,): readonly ArrangementEdit[] {
      /**
       The places the replacement carries.
       */
      const carried = carries.map(function placeAt(carry,): ArrangementPlace {
        return nonNullishOrThrow(places[carry],);
      },);
      const [first,] = carried;
      if ((first === undefined) || carried.some(function elsewhere({ slice, },): boolean {
        return slice !== first.slice;
      },))
        return [];
      /**
       Where the carried words start, and where they end.
       */
      const lowest = Math.min(...carried.map(function startOf({ core, },): number {
        return core.start;
      },),);
      const highest = Math.max(...carried.map(function endOf({ core, },): number {
        return core.end;
      },),);
      /**
       Offsets where a word starts or ends.
       */
      const edges = first.slice
        .words
        .flat()
        .toSorted(function ascending(
          left,
          right,
        ): number {
          return left - right;
        },);
      /**
       The writer's blocks, each keeping or dropping the words it replaces.
       */
      const blocks = edges.flatMap(function fromEdge(start,): readonly ArrangementEdit[] {
        return edges
          .filter(function holdsWords(end,): boolean {
            return (start <= lowest) && (highest <= end);
          },)
          .flatMap(function toEdge(end,): readonly ArrangementEdit[] {
            return [false, true,].map(function blockOf(keeps,): ArrangementEdit {
              return {
                carries,
                slice: first.slice,
                kind: 'block',
                start,
                end,
                keeps,
              };
            },);
          },);
      },);
      return (carries.length === 1)
        ? [
          {
            carries,
            slice: first.slice,
            kind: 'splice',
            start: lowest,
            end: highest,
            keeps: false,
          },
          ...blocks,
        ]
        : blocks;
    },);
}

/**
 Every set of replacements a group can plan, none carrying a place another carries and none meeting another.

 @param candidates - replacements to choose from

 @param chosen - replacements already chosen

 @returns Chosen replacements, and each way to add later candidates to them

 @example
 ```ts
 const sets = editSetsFrom({ candidates, chosen: [], });
 ```
 */
function editSetsFrom(
  {
    candidates,
    chosen,
  }: {
    readonly candidates: readonly ArrangementEdit[];
    readonly chosen: readonly ArrangementEdit[];
  },
): readonly (readonly ArrangementEdit[])[] {
  return [
    chosen,
    ...candidates.flatMap(function adding(
      candidate,
      position,
    ): readonly (readonly ArrangementEdit[])[] {
      /**
       Whether the candidate carries a place a chosen one carries, or meets a chosen one.
       */
      const clashes = chosen.some(function clashesWith(other,): boolean {
        return other.carries.some(function sharedPlace(carry,): boolean {
          return candidate.carries
            .includes(carry,);
        },) || ((other.slice === candidate.slice) && (candidate.start < other.end) && (other.start < candidate.end));
      },);
      return clashes
        ? []
        : editSetsFrom({
          candidates: candidates.slice(position + 1,),
          chosen: [...chosen, candidate,],
        },);
    },),
  ];
}

/**
 Every group a model allows.

 @param model - slices and how many places a group has in each

 @returns Each choice of located words for the places, with each set of replacements over them

 @example
 ```ts
 const groups = groupsOf({ model, });
 ```
 */
function groupsOf({ model, }: { readonly model: ArrangementModel; },): readonly ArrangementGroup[] {
  return model.slices
    .reduce(
    function withSlice(
      shapes,
      {
        slice,
        places,
      },
    ): readonly (readonly ArrangementPlace[])[] {
      return shapes.flatMap(function extended(shape,): readonly (readonly ArrangementPlace[])[] {
        return coreSetsOf({
          slice,
          count: places,
        },)
          .map(function placed(cores,): readonly ArrangementPlace[] {
            return [
              ...shape,
              ...cores.map(function placeOf(core,): ArrangementPlace {
                return { slice, core, };
              },),
            ];
          },);
      },);
    },
    NO_SHAPES,
    )
    .flatMap(function setsOver(places,): readonly ArrangementGroup[] {
      return editSetsFrom({
        candidates: editCandidatesOf({ places, },),
        chosen: [],
      },)
        .map(function groupOf(edits,): ArrangementGroup {
          return { places, edits, };
        },);
    },);
}

/**
 The settled group an arrangement's group stands for: a chosen place in another slice, then its receiving places,
 those a replacement carries to be replaced and the rest kept for their context.

 @param group - arrangement's group

 @returns Settled group with its pending replacements

 @example
 ```ts
 const settled = settledArrangementGroup({ group, });
 ```
 */
function settledArrangementGroup({ group, }: { readonly group: ArrangementGroup; },): SettledGroup {
  /**
   Positions of the receiving places a replacement carries.
   */
  const carried = new Set(group.edits.flatMap(function carriesOf({ carries, },): readonly number[] {
    return carries;
  },),);
  /**
   The provisional outcome of each place, the chosen one first.
   */
  const outcomes: SettledGroup['outcomes'] = [
    'chosen',
    ...group.places.map(function outcomeOf(
      _place,
      position,
    ): 'replaced' | 'kept-context' {
      return carried.has(position,) ? 'replaced' : 'kept-context';
    },),
  ];
  /**
   The pending replacements, naming places by their position among the group's places.
   */
  const edits = group.edits.map(function pendingOf(edit,): PendingEdit {
    return {
      kind: edit.kind,
      placePositions: edit.carries
        .map(function positionOf(carry,): number {
          return carry + 1;
        },),
      sliceIndex: edit.slice
        .index,
      start: edit.start,
      end: edit.end,
      text: edit.keeps
        ? edit.slice
          .text
          .slice(
          edit.start,
          edit.end,
          )
        : 'mew',
    };
  },);
  return {
    ...groupIn({
      places: [
        { slice: 3, english: ARRANGEMENT_CHOSEN_ENGLISH, quote: 'in the sun', },
        ...group.places.map(function placeIn(
          { slice, core, },
        ): { readonly slice: number; readonly english: string; readonly quote: string; } {
          return {
            slice: slice.index,
            english: slice.text,
            quote: slice.text
              .slice(
              core.start,
              core.end,
              ),
          };
        },),
      ],
    },),
    outcomes,
    edits,
  };
}

/**
 The facts of the slices an arrangement is applied over, whose English is what each ships now.

 @param model - slices and how many places a group has in each

 @returns Facts of each slice of the model and of the slice the chosen places stand in

 @example
 ```ts
 const slices = arrangementSliceFacts({ model, });
 ```
 */
function arrangementSliceFacts({ model, }: { readonly model: ArrangementModel; },): readonly ConsistencySliceFacts[] {
  return [
    ...model.slices
      .map(function textOf({ slice, },): { readonly index: number; readonly text: string; } {
        return slice;
      },),
    { index: 3, text: ARRANGEMENT_CHOSEN_ENGLISH, },
  ].map(function factsOf({ index, text, },): ConsistencySliceFacts {
    return {
      sliceIndex: index,
      sourceIndex: index,
      sourceText: '小猫打盹。',
      sourceStart: 0,
      nodes: [],
      current: { kind: 'present', text, },
      lineStructured: false,
      disputedWordings: [],
    };
  },);
}

/**
 Whether a made replacement changes the located words of a place: it meets them and either does not hold all of
 them or drops words it holds.

 @param edit - replacement

 @param place - place of another group

 @returns Whether the place's words are changed

 @example
 ```ts
 const changes = changesPlace({ edit, place, });
 ```
 */
function changesPlace(
  {
    edit,
    place,
  }: {
    readonly edit: ArrangementEdit;
    readonly place: ArrangementPlace;
  },
): boolean {
  if ((edit.slice !== place.slice) || (edit.end <= place.core.start) || (place.core.end <= edit.start))
    return false;
  return !((edit.start <= place.core.start) && (place.core.end <= edit.end) && edit.keeps);
}

/**
 The label the pass gives a group's receiving place.

 @param position - position of the place among the group's receiving places, from zero

 @returns Label, counting the chosen place first

 @example
 ```ts
 const label = receivingLabel({ position: 0, });
 ```
 */
function receivingLabel({ position, }: { readonly position: number; },): string {
  return `OCCURRENCE ${String(position + 2,)}`;
}

/**
 Judges one applied arrangement: every place of another group whose located words a made replacement changed must
 be recorded overlapped, name the place it gave way to, and have been recorded by the arbitration, either as a
 replacement left out for changing the words of the place it names, or as a replacement that lost the words to a
 made one whose first carried place it names.

 @param groups - arrangement's groups, in the order they were applied

 @param applied - what applying them came to

 @returns Breaches found and how many places each branch of the arbitration recorded

 @example
 ```ts
 const verdict = judgedArrangement({ groups, applied, });
 ```
 */
function judgedArrangement(
  {
    groups,
    applied,
  }: {
    readonly groups: readonly ArrangementGroup[];
    readonly applied: ReturnType<typeof applyConsistency>;
  },
): ArrangementVerdict {
  /**
   The recorded place of a group.
   */
  function occurrenceOf({ group, position, }: { readonly group: number; readonly position: number; },): ArtifactConsistencyGroup['occurrences'][number] {
    return nonNullishOrThrow(applied.groups[group]?.occurrences[position + 1],);
  }
  /**
   The replacements that were made: every place they carry ended replaced.
   */
  const made = groups.flatMap(function madeOf(
    { edits, },
    group,
  ): readonly { readonly group: number; readonly edit: ArrangementEdit; }[] {
    return edits
      .filter(function wasMade({ carries, },): boolean {
        return carries.every(function replaced(position,): boolean {
          return occurrenceOf({ group, position, },).outcome === 'replaced';
        },);
      },)
      .map(function named(edit,): { readonly group: number; readonly edit: ArrangementEdit; } {
        return { group, edit, };
      },);
  },);
  /**
   The places of other groups whose located words a made replacement changed.
   */
  const changed = made.flatMap(function changesOf({ group, edit, },): readonly { readonly group: number; readonly position: number; }[] {
    return groups.flatMap(function placesOfGroup(
      other,
      otherGroup,
    ): readonly { readonly group: number; readonly position: number; }[] {
      return (otherGroup === group)
        ? []
        : other.places.flatMap(function changedPlace(
          place,
          position,
        ): readonly { readonly group: number; readonly position: number; }[] {
          return changesPlace({ edit, place, },) ? [{ group: otherGroup, position, },] : [];
        },);
    },);
  },);
  /**
   Each change judged: the breaches it shows, and the branch that recorded it.
   */
  const judged = changed.map(function judge(
    { group, position, },
  ): { readonly breaches: readonly string[]; readonly branch: 'left-out' | 'outlasted' | 'neither'; } {
    /**
     The place's recorded outcome, and the place it gave way to.
     */
    const { outcome, gaveWayTo, } = occurrenceOf({ group, position, },);
    /**
     How the place is named in a breach.
     */
    const called = `group ${String(group + 1,)} ${receivingLabel({ position, },)}`;
    if (outcome !== 'overlapped')
      return { breaches: [`${called} was changed by a made replacement and ended ${outcome}`,], branch: 'neither', };
    if (gaveWayTo === undefined)
      return { breaches: [`${called} was changed by a made replacement and names no place it gave way to`,], branch: 'neither', };
    /**
     The replacement of the place's group that carries it.
     */
    const pending = groups[group]?.edits
      .find(function carriesIt({ carries, },): boolean {
        return carries.includes(position,);
      },);
    if (pending === undefined)
      return { breaches: [`${called} was changed by a made replacement and has no replacement of its own`,], branch: 'neither', };
    /**
     What is reported of a replacement left out against the place it names.
     */
    const report = `slice ${String(pending.slice.index,)}: the consistency replacement of group ${String(group + 1,)} ${
      pending.carries
        .map(function labelOf(carry,): string {
          return receivingLabel({ position: carry, },);
        },)
        .join(' and ',)
    } was left out because it changes the words that group ${String(gaveWayTo.group + 1,)} ${gaveWayTo.label} located`;
    if (applied.findings.includes(report,))
      return { breaches: [], branch: 'left-out', };
    /**
     The places the made replacements that meet the place's replacement are named by.
     */
    const winners = made
      .filter(function meets({ edit, },): boolean {
        return (edit.slice === pending.slice) && (edit.start < pending.end) && (pending.start < edit.end);
      },)
      .map(function nameOf({ group: winnerGroup, edit, },): string {
        return `${String(winnerGroup,)} ${receivingLabel({ position: nonNullishOrThrow(edit.carries[0],), },)}`;
      },);
    return winners.includes(`${String(gaveWayTo.group,)} ${gaveWayTo.label}`,)
      ? { breaches: [], branch: 'outlasted', }
      : {
        breaches: [`${called} gave way to ${String(gaveWayTo.group + 1,)} ${gaveWayTo.label}, which neither a left-out finding nor a made replacement over it explains`,],
        branch: 'neither',
      };
  },);
  /**
   Places the floors refused, which would make a replacement that was not made look made.
   */
  const refused = applied.groups.flatMap(function refusedIn(group,): readonly string[] {
    return group.occurrences
      .filter(function wasRefused({ outcome, },): boolean {
        return outcome === 'kept-floor';
      },)
      .map(function named({ label, },): string {
        return `${label} was refused by a floor`;
      },);
  },);
  return {
    breaches: [
      ...refused,
      ...judged.flatMap(function breachesOf({ breaches, },): readonly string[] {
        return breaches;
      },),
    ],
    changed: judged.length,
    leftOut: judged.filter(function byLeftOut({ branch, },): boolean {
      return branch === 'left-out';
    },).length,
    outlasted: judged.filter(function byLoser({ branch, },): boolean {
      return branch === 'outlasted';
    },).length,
  };
}

/**
 Applies every arrangement of a model and judges each.

 @param model - slices and how many places a group has in each

 @returns How many arrangements were built, how many changes were judged, how each branch recorded them, and the first breaches

 @example
 ```ts
 const outcome = arrangementOutcome({ model, });
 ```
 */
function arrangementOutcome(
  { model, }: { readonly model: ArrangementModel; },
): ArrangementOutcome {
  /**
   Every group the model allows, with the settled group each stands for.
   */
  const allowed = groupsOf({ model, },);
  const settledAllowed = allowed.map(function settledOf(group,): SettledGroup {
    return settledArrangementGroup({ group, },);
  },);
  /**
   Facts of the slices every arrangement is applied over.
   */
  const slices = arrangementSliceFacts({ model, },);
  /**
   Every arrangement: one group chosen for each of the groups on the page, in page order.
   */
  const arrangements = model.groupCounts.flatMap(function ofCount(count,): readonly (readonly number[])[] {
    return Array.from(
      { length: allowed.length ** count, },
      function choice(
        _arrangement,
        number,
      ): readonly number[] {
        return Array.from(
          { length: count, },
          function groupAt(
            _group,
            position,
          ): number {
            return Math.trunc(number / (allowed.length ** position)) % allowed.length;
          },
        );
      },
    );
  },);
  /**
   Each arrangement applied and judged.
   */
  const verdicts = arrangements.map(function judge(chosen,): ArrangementVerdict {
    /**
     The arrangement's groups, and the settled group of each.
     */
    const groups = chosen.map(function groupAt(position,): ArrangementGroup {
      return nonNullishOrThrow(allowed[position],);
    },);
    return judgedArrangement({
      groups,
      applied: applyConsistency({
        settled: chosen.map(function settledAt(position,): SettledGroup {
          return nonNullishOrThrow(settledAllowed[position],);
        },),
        slices,
        declared: [],
        declaredNames: [],
        l: levelCapturingLogger({ lines: [], },),
      },),
    },);
  },);
  return {
    arrangements: verdicts.length,
    changed: verdicts.reduce(function total(sum, { changed: count, },): number {
      return sum + count;
    }, 0,),
    leftOut: verdicts.reduce(function total(sum, { leftOut: count, },): number {
      return sum + count;
    }, 0,),
    outlasted: verdicts.reduce(function total(sum, { outlasted: count, },): number {
      return sum + count;
    }, 0,),
    breaches: verdicts
      .flatMap(function breachesOf({ breaches, },): readonly string[] {
        return breaches;
      },)
      .slice(0, 5,),
  };
}


/**
 The two-word slice of the models.
 */
const TWO_WORD_SLICE = sliceOfWords({ index: 1, words: ['Tiny', 'kittens',], },);

/**
 The three-word slice of the models, whose runs of words include ones that partly overlap.
 */
const THREE_WORD_SLICE = sliceOfWords({ index: 1, words: ['Tiny', 'kittens', 'nap',], },);

/**
 The one-word slice after the two-word one.
 */
const ONE_WORD_SLICE = sliceOfWords({ index: 2, words: ['Purrs',], },);

/**
 A model with what every arrangement of it comes to.

 @example
 ```ts
 const arrangementCase: ArrangementCase = { name: 'two groups', model, expected, };
 ```
 */
type ArrangementCase = {
  /**
   What the model is, for the case's name.
   */
  readonly name: string;

  /**
   The slices and how many places a group has in each.
   */
  readonly model: ArrangementModel;

  /**
   What applying every arrangement of the model must come to.
   */
  readonly expected: ArrangementOutcome;
};

/**
 The models every arrangement is built from: one place per group over a two-word slice with up to three groups,
 over a three-word slice whose runs of words partly overlap, over two slices, and two places per group over one
 slice. Each pins the number of arrangements built and the number of changed places the arbitration recorded by
 each branch, so a model that shrinks, or one that never reaches a branch, fails.
 */
const ARRANGEMENT_CASES: readonly ArrangementCase[] = [
  {
    name: 'one place per group in a two-word slice, up to three groups',
    model: { slices: [{ slice: TWO_WORD_SLICE, places: 1, },], groupCounts: [1, 2, 3,], },
    expected: { arrangements: 8_420, changed: 2_574, leftOut: 1_730, outlasted: 844, breaches: [], },
  },
  {
    name: 'one place per group in a three-word slice, up to two groups',
    model: { slices: [{ slice: THREE_WORD_SLICE, places: 1, },], groupCounts: [1, 2,], },
    expected: { arrangements: 4_160, changed: 372, leftOut: 284, outlasted: 88, breaches: [], },
  },
  {
    name: 'one place per group in each of two slices, up to two groups',
    model: { slices: [{ slice: TWO_WORD_SLICE, places: 1, }, { slice: ONE_WORD_SLICE, places: 1, },], groupCounts: [1, 2,], },
    expected: { arrangements: 6_480, changed: 2_496, leftOut: 1_408, outlasted: 1_088, breaches: [], },
  },
  {
    name: 'two places per group in a two-word slice, up to two groups',
    model: { slices: [{ slice: TWO_WORD_SLICE, places: 2, },], groupCounts: [1, 2,], },
    expected: { arrangements: 1_482, changed: 504, leftOut: 288, outlasted: 216, breaches: [], },
  },
];

//endregion Arrangement model

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applyConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES EVERY GROUP\'S REPLACEMENTS IN ONE SLICE TOGETHER, the sentence and the refrain both reaching the third slice',
          fn: async () => {
            const lines: string[] = [];
            const applied = applyCat({ page: settledCatPage({},), lines, },);
            expect(applied.slices.map(function textOf(slice,): readonly [number, string] {
              return [slice.sliceIndex, slice.text,];
            },),).toEqual([
              [1, '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat must be a lazy little treasure.',],
              [
                2,
                '## Three\n\nIn the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. '
                  + 'So this cat must be a sweet little treasure.',
              ],
              [3, '## Four\n\nThe rain in the yard sent the kitten running home. So this cat must be a warm little treasure.',],
            ],);
            expect(outcomesOf({ groups: applied.groups, },),).toEqual([
              ['chosen', 'replaced',],
              ['chosen', 'replaced', 'replaced', 'replaced',],
            ],);
            expect([applied.findings, lines,],).toEqual([[], [],],);
          },
        },),
        it({
          name: 'MAKES THE REPLACEMENTS OF TWO PLACES OF ONE GROUP IN ONE SLICE TOGETHER, neither being left out as '
            + 'overlapping its own group, whether the carries are one for each place or one for the block',
          fn: async () => {
            const prepared = prepareDocumentPair({
              sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。厨房里很暖和。窗台上的阳光让小猫睡着了。\n\n## 二\n\n窗台上的阳光让小猫睡着了。\n',
              targetText: '## One\n\nThe sunny window put the kitten to sleep. The kitchen was warm. The sun on the sill sent the '
                + 'kitten off to sleep.\n\n## Two\n\nThe window sun made the kitten fall asleep.\n',
            },);
            const page = settledCatPage({
              prepared,
              sentenceQuotes: [
                { label: 'OCCURRENCE 1', quote: 'The sunny window put the kitten to sleep', },
                { label: 'OCCURRENCE 2', quote: 'The sun on the sill sent the kitten off to sleep', },
                { label: 'OCCURRENCE 3', quote: 'The window sun made the kitten fall asleep', },
              ],
              choice: 'OCCURRENCE 3',
            },);
            const lines: string[] = [];
            const applied = applyCat({ page, lines, },);
            /**
             The same edits as one edit naming both places, as a writer's block would make.
             */
            const [group,] = page.settled;
            const [first, second,] = group?.edits ?? [];
            const together = applyCat({
              page: {
                ...page,
                settled: page.settled.map(function asOneBlock(settled,): SettledGroup {
                  return {
                    ...settled,
                    edits: ((first === undefined) || (second === undefined))
                      ? settled.edits
                      : [
                        {
                          kind: 'block',
                          placePositions: [...first.placePositions, ...second.placePositions,],
                          sliceIndex: first.sliceIndex,
                          start: first.start,
                          end: second.end,
                          text: 'The window sun made the kitten fall asleep. The kitchen was warm. The window sun made the kitten fall asleep',
                        },
                      ],
                  };
                },),
              },
              lines,
            },);
            expect({
              outcomes: outcomesOf({ groups: applied.groups, },),
              changed: applied.slices.map(function textOf(slice,): readonly [number, string] {
                return [slice.sliceIndex, slice.text,];
              },),
              editsPlanned: group?.edits.map(function positionsOf({ placePositions, },): readonly number[] {
                return placePositions;
              },),
              findings: applied.findings,
              lines,
              togetherOutcomes: outcomesOf({ groups: together.groups, },),
              togetherChanged: together.slices.map(function textOf(slice,): readonly [number, string] {
                return [slice.sliceIndex, slice.text,];
              },),
            },).toEqual({
              outcomes: [['replaced', 'replaced', 'chosen',],],
              changed: [
                [
                  0,
                  '## One\n\nThe window sun made the kitten fall asleep. The kitchen was warm. '
                    + 'The window sun made the kitten fall asleep.',
                ],
              ],
              editsPlanned: [[0,], [1,],],
              findings: [],
              lines: [],
              togetherOutcomes: [['replaced', 'replaced', 'chosen',],],
              togetherChanged: [
                [
                  0,
                  '## One\n\nThe window sun made the kitten fall asleep. The kitchen was warm. '
                    + 'The window sun made the kitten fall asleep.',
                ],
              ],
            },);
          },
        },),
        it({
          name: 'RECORDS EACH PLACE WITH ITS LABEL, SLICE, OFFSETS, LINE, SLOT OFFSETS AND RENDERING, and each group with its '
            + 'repeat, unplaced places and verdict as they were settled',
          fn: async () => {
            const page = settledCatPage({},);
            const applied = applyCat({ page, lines: [], },);
            const [sentence, frame,] = applied.groups;
            const [sentenceSettled, frameSettled,] = page.settled;
            expect(nonNullishOrThrow(frame,).occurrences[1],).toEqual({
              label: 'OCCURRENCE 2',
              sliceIndex: 1,
              startOffset: 56,
              endOffset: 70,
              line: 7,
              slotStartOffset: 64,
              slotEndOffset: 66,
              rendering: { kind: 'located', quote: 'So this cat is surely a lazy little treasure', start: 58, slot: 'lazy', },
              outcome: 'replaced',
            },);
            expect(nonNullishOrThrow(sentence,).occurrences[1],).toEqual({
              label: 'OCCURRENCE 2',
              sliceIndex: 2,
              startOffset: 85,
              endOffset: 97,
              line: 11,
              rendering: { kind: 'located', quote: 'the sunlight on the sill lulled the kitten to sleep', start: 26, },
              outcome: 'replaced',
            },);
            expect([sentence, frame,].map(function carried(group,): unknown {
              return [group?.repeat, group?.unplaced, group?.verdict,];
            },),).toEqual([sentenceSettled, frameSettled,].map(function settledOf(group,): unknown {
              return [group?.repeat, group?.unplaced, group?.verdict,];
            },),);
          },
        },),
        it({
          name: 'RECORDS EACH GROUP\'S CARRIES AS THEY WERE SETTLED, one for each receiving place, beside its places',
          fn: async () => {
            const page = settledCatPage({},);
            const applied = applyCat({ page, lines: [], },);
            expect(applied.groups.map(function carriesOf(group,): unknown {
              return group.carries?.map(function labelOf({ label, decision, },): unknown {
                return [label, decision,];
              },);
            },),).toEqual([
              [['OCCURRENCE 2', { kind: 'chosen', label: 'CANDIDATE 2', },],],
              [
                ['OCCURRENCE 2', { kind: 'chosen', label: 'CANDIDATE 2', },],
                ['OCCURRENCE 3', { kind: 'chosen', label: 'CANDIDATE 2', },],
                ['OCCURRENCE 4', { kind: 'chosen', label: 'CANDIDATE 2', },],
              ],
            ],);
            expect(applied.groups.map(function carriesOf(group,): unknown {
              return group.carries;
            },),).toEqual(page.settled.map(function carriesOf(group,): unknown {
              return group.carries;
            },),);
          },
        },),
        it({
          name: 'RECORDS A PLACE WHOSE QUOTE WAS NOT LOCATED AS UNLOCATED WITH ITS REASON, and replaces the others',
          fn: async () => {
            const applied = applyCat({
              page: settledCatPage({
                sentenceQuotes: [
                  nonNullishOrThrow(SENTENCE_QUOTES[0],),
                  { label: 'OCCURRENCE 2', quote: 'the lamp on the sill lulled the kitten to sleep', },
                ],
              },),
              lines: [],
            },);
            expect(nonNullishOrThrow(applied.groups[0],).occurrences[1],).toMatchObject({
              rendering: { kind: 'unlocated', reason: 'not-found', },
              outcome: 'unlocated',
            },);
            expect(applied.slices.map(function indexOf(slice,): number {
              return slice.sliceIndex;
            },),).toEqual([1, 2, 3,],);
          },
        },),
        it({
          name: 'CHANGES NO SLICE AND KEEPS EVERY OUTCOME WHERE NO GROUP PLANNED A REPLACEMENT',
          fn: async () => {
            const page = settledCatPage({},);
            const applied = applyConsistency({
              settled: page.settled
                .map(function withoutEdits(group,): SettledGroup {
                  return {
                    ...group,
                    edits: [],
                    outcomes: group.outcomes
                      .map(function untouched(): 'untouched' {
                        return 'untouched';
                      },),
                  };
                },),
              slices: page.slices,
              declared: [],
              declaredNames: [],
              l: levelCapturingLogger({ lines: [], },),
            },);
            expect([applied.slices, applied.findings, outcomesOf({ groups: applied.groups, },),],).toEqual([
              [],
              [],
              [['untouched', 'untouched',], ['untouched', 'untouched', 'untouched', 'untouched',],],
            ],);
          },
        },),
        it({
          name: 'LEAVES THE LATER OF TWO GROUPS\' REPLACEMENTS OUT WHERE THEIR WORDS OVERLAP, recording overlapped and making '
            + 'the earlier group\'s replacement',
          fn: async () => {
            const applied = applyCat({
              page: settledCatPage({
                sentenceQuotes: [
                  nonNullishOrThrow(SENTENCE_QUOTES[0],),
                  {
                    label: 'OCCURRENCE 2',
                    quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. '
                      + 'So this cat has to be a sweet little darling',
                  },
                ],
              },),
              lines: [],
            },);
            expect(outcomesOf({ groups: applied.groups, },),).toEqual([
              ['chosen', 'replaced',],
              ['chosen', 'replaced', 'overlapped', 'replaced',],
            ],);
            expect(applied.slices.find(function third(slice,): boolean {
              return slice.sliceIndex === 2;
            },),).toEqual({ sliceIndex: 2, text: '## Three\n\nIn the evening, the sunny window put the kitten to sleep.', },);
            expect({
              gaveWay: gaveWayOf({ groups: applied.groups, },),
              findings: applied.findings,
            },).toEqual({
              gaveWay: [['none', 'none',], ['none', 'none', { group: 0, label: 'OCCURRENCE 2', }, 'none',],],
              findings: [],
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A BLOCK REPLACEMENT THAT CHANGES THE WORDS ANOTHER GROUP LOCATED IN IT, whichever order the groups '
            + 'come in, recording overlapped for the places it carried and keeping a finding that names both groups',
          fn: async () => {
            const page = settledCatPage({},);
            const third = nonNullishOrThrow(page.slices[2],);
            const english = third.current.kind === 'present' ? third.current.text : '';
            const blockStart = english.indexOf('In the evening',);
            /**
             The page with the sentence group's replacement in the third slice made over the whole paragraph, as a
             writer's block would be.
             */
            function withBlockText({ text, }: { readonly text: string; },): ReturnType<typeof settledCatPage> {
              return {
                ...page,
                settled: page.settled.map(function rewritten(group, at,): SettledGroup {
                  return (at === 0)
                    ? {
                      ...group,
                      edits: [
                        {
                          kind: 'block',
                          placePositions: [1,],
                          sliceIndex: 2,
                          start: blockStart,
                          end: english.length,
                          text,
                        },
                      ],
                    }
                    : group;
                },),
              };
            }
            const rewritten = withBlockText({
              text: 'In the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. '
                + 'So this cat is a darling.',
            },);
            const lines: string[] = [];
            const forward = applyCat({ page: rewritten, lines, },);
            const backwardLines: string[] = [];
            const backward = applyCat({
              page: { ...rewritten, settled: rewritten.settled.toReversed(), },
              lines: backwardLines,
            },);
            const kept = '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks '
              + 'smiled too. So this cat must be a sweet little treasure.';
            expect({
              forward: outcomesOf({ groups: forward.groups, },),
              backward: outcomesOf({ groups: backward.groups, },),
              forwardThird: thirdSliceOf({ applied: forward, },),
              backwardThird: thirdSliceOf({ applied: backward, },),
              forwardGaveWay: gaveWayOf({ groups: forward.groups, },),
              backwardGaveWay: gaveWayOf({ groups: backward.groups, },),
              forwardFindings: forward.findings,
              backwardFindings: backward.findings,
              forwardWarned: lines,
              backwardWarned: backwardLines,
            },).toEqual({
              forward: [['chosen', 'overlapped',], ['chosen', 'replaced', 'replaced', 'replaced',],],
              backward: [['chosen', 'replaced', 'replaced', 'replaced',], ['chosen', 'overlapped',],],
              forwardThird: [kept,],
              backwardThird: [kept,],
              forwardGaveWay: [['none', { group: 1, label: 'OCCURRENCE 3', },], ['none', 'none', 'none', 'none',],],
              backwardGaveWay: [['none', 'none', 'none', 'none',], ['none', { group: 0, label: 'OCCURRENCE 3', },],],
              forwardFindings: [
                'slice 2: the consistency replacement of group 1 OCCURRENCE 2 was left out because it changes the words that '
                  + 'group 2 OCCURRENCE 3 located',
              ],
              backwardFindings: [
                'slice 2: the consistency replacement of group 2 OCCURRENCE 2 was left out because it changes the words that '
                  + 'group 1 OCCURRENCE 3 located',
              ],
              forwardWarned: [
                'warn [applyConsistency] slice 2: the consistency replacement of group 1 OCCURRENCE 2 was left out because it '
                  + 'changes the words that group 2 OCCURRENCE 3 located',
              ],
              backwardWarned: [
                'warn [applyConsistency] slice 2: the consistency replacement of group 2 OCCURRENCE 2 was left out because it '
                  + 'changes the words that group 1 OCCURRENCE 3 located',
              ],
            },);
          },
        },),
        it({
          name: 'MAKES A BLOCK REPLACEMENT THAT STILL HOLDS THE WORDS ANOTHER GROUP LOCATED, the later group\'s replacement '
            + 'there being left out as overlapping it, and leaves out one that cuts those words in half',
          fn: async () => {
            const page = settledCatPage({},);
            const third = nonNullishOrThrow(page.slices[2],);
            const english = third.current.kind === 'present' ? third.current.text : '';
            const blockStart = english.indexOf('In the evening',);
            /**
             What applying the page comes to with the sentence group's replacement in the third slice changed.
             */
            function appliedWith(
              { start, text, }: { readonly start: number; readonly text: string; },
            ): ReturnType<typeof applyConsistency> {
              return applyCat({
                page: {
                  ...page,
                  settled: page.settled.map(function rewritten(group, at,): SettledGroup {
                    return (at === 0)
                      ? {
                        ...group,
                        edits: [
                          { kind: 'block', placePositions: [1,], sliceIndex: 2, start, end: english.length, text, },
                        ],
                      }
                      : group;
                  },),
                },
                lines: [],
              },);
            }
            const holding = appliedWith({
              start: blockStart,
              text: 'In the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. '
                + 'So this cat has to be a sweet little darling.',
            },);
            const cutting = appliedWith({
              start: english.indexOf('this cat',),
              text: 'cat is a darling.',
            },);
            expect({
              holding: outcomesOf({ groups: holding.groups, },),
              holdingFindings: holding.findings,
              cutting: outcomesOf({ groups: cutting.groups, },),
              cuttingFindings: cutting.findings,
            },).toEqual({
              holding: [['chosen', 'replaced',], ['chosen', 'replaced', 'overlapped', 'replaced',],],
              holdingFindings: [],
              cutting: [['chosen', 'overlapped',], ['chosen', 'replaced', 'replaced', 'replaced',],],
              cuttingFindings: [
                'slice 2: the consistency replacement of group 1 OCCURRENCE 2 was left out because it changes the words that '
                  + 'group 2 OCCURRENCE 3 located',
              ],
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A CODE SPLICE THAT CHANGES THE WORDS OF A PLACE ANOTHER GROUP LOCATED AND DOES NOT REPLACE, whichever '
            + 'order the groups come in, recording overlapped for the place it carried with the place it gave way to, keeping a '
            + 'finding that names both and leaving the slice\'s English as it stood',
          fn: async () => {
            /**
             The sentence group keeps its current English in its second place, whose words reach into the refrain's third
             place, and the refrain's third place is located over words that run into the sentence's second.
             */
            const page = settledCatPage({
              sentenceQuotes: [
                nonNullishOrThrow(SENTENCE_QUOTES[0],),
                {
                  label: 'OCCURRENCE 2',
                  quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too',
                },
              ],
              frameQuotes: WIDENED_REFRAIN_QUOTES,
              carryOf: function sentenceKeepsCurrent({ decided, },): readonly UnitCarried[] {
                return (decided.repeat.kind === 'refrain')
                  ? carriedBySplice({ decided, },)
                  : carriedKeepingCurrent({ decided, },);
              },
            },);
            const forwardLines: string[] = [];
            const forward = applyCat({ page, lines: forwardLines, },);
            const backwardLines: string[] = [];
            const backward = applyCat({
              page: { ...page, settled: page.settled.toReversed(), },
              lines: backwardLines,
            },);
            /**
             The slices the refrain's other two places change, which is every change there is.
             */
            const changed = [
              [1, '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat must be a lazy little treasure.',],
              [3, '## Four\n\nThe rain in the yard sent the kitten running home. So this cat must be a warm little treasure.',],
            ];
            const forwardFinding = 'slice 2: the consistency replacement of group 2 OCCURRENCE 3 was left out because it '
              + 'changes the words that group 1 OCCURRENCE 2 located';
            const backwardFinding = 'slice 2: the consistency replacement of group 1 OCCURRENCE 3 was left out because it '
              + 'changes the words that group 2 OCCURRENCE 2 located';
            expect({
              forwardOutcomes: outcomesOf({ groups: forward.groups, },),
              backwardOutcomes: outcomesOf({ groups: backward.groups, },),
              forwardGaveWay: gaveWayOf({ groups: forward.groups, },),
              backwardGaveWay: gaveWayOf({ groups: backward.groups, },),
              forwardChanged: forward.slices.map(function textOf(slice,): readonly [number, string] {
                return [slice.sliceIndex, slice.text,];
              },),
              backwardChanged: backward.slices.map(function textOf(slice,): readonly [number, string] {
                return [slice.sliceIndex, slice.text,];
              },),
              forwardFindings: forward.findings,
              backwardFindings: backward.findings,
              forwardWarned: forwardLines,
              backwardWarned: backwardLines,
            },).toEqual({
              forwardOutcomes: [['chosen', 'kept-current',], ['chosen', 'replaced', 'overlapped', 'replaced',],],
              backwardOutcomes: [['chosen', 'replaced', 'overlapped', 'replaced',], ['chosen', 'kept-current',],],
              forwardGaveWay: [['none', 'none',], ['none', 'none', { group: 0, label: 'OCCURRENCE 2', }, 'none',],],
              backwardGaveWay: [['none', 'none', { group: 1, label: 'OCCURRENCE 2', }, 'none',], ['none', 'none',],],
              forwardChanged: changed,
              backwardChanged: changed,
              forwardFindings: [forwardFinding,],
              backwardFindings: [backwardFinding,],
              forwardWarned: [`warn [applyConsistency] ${forwardFinding}`,],
              backwardWarned: [`warn [applyConsistency] ${backwardFinding}`,],
            },);
            expect(readBackOf({ applied: forward, page, },),).toEqual(writtenOf({ applied: forward, },),);
            expect(readBackOf({ applied: backward, page, },),).toEqual(writtenOf({ applied: backward, },),);
          },
        },),
        it({
          name: 'RECORDS THE LATER OF TWO GROUPS\' PLACES AS OVERLAPPED WITH THE EARLIER GROUP\'S PLACE IT GAVE WAY TO, in either '
            + 'order, with no finding for the winning replacement that changed the later group\'s words partway',
          fn: async () => {
            /**
             The sentence group replaces its second place with words that reach into the refrain's third place, whose
             located words run into the sentence's second.
             */
            const page = settledCatPage({
              sentenceQuotes: [
                nonNullishOrThrow(SENTENCE_QUOTES[0],),
                {
                  label: 'OCCURRENCE 2',
                  quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too',
                },
              ],
              frameQuotes: WIDENED_REFRAIN_QUOTES,
            },);
            const forwardLines: string[] = [];
            const forward = applyCat({ page, lines: forwardLines, },);
            const backwardLines: string[] = [];
            const backward = applyCat({
              page: { ...page, settled: page.settled.toReversed(), },
              lines: backwardLines,
            },);
            expect({
              forwardOutcomes: outcomesOf({ groups: forward.groups, },),
              backwardOutcomes: outcomesOf({ groups: backward.groups, },),
              forwardGaveWay: gaveWayOf({ groups: forward.groups, },),
              backwardGaveWay: gaveWayOf({ groups: backward.groups, },),
              forwardThird: thirdSliceOf({ applied: forward, },),
              backwardThird: thirdSliceOf({ applied: backward, },),
              forwardFindings: forward.findings,
              backwardFindings: backward.findings,
              forwardWarned: forwardLines,
              backwardWarned: backwardLines,
            },).toEqual({
              forwardOutcomes: [['chosen', 'replaced',], ['chosen', 'replaced', 'overlapped', 'replaced',],],
              backwardOutcomes: [['chosen', 'replaced', 'replaced', 'replaced',], ['chosen', 'overlapped',],],
              forwardGaveWay: [['none', 'none',], ['none', 'none', { group: 0, label: 'OCCURRENCE 2', }, 'none',],],
              backwardGaveWay: [['none', 'none', 'none', 'none',], ['none', { group: 0, label: 'OCCURRENCE 3', },],],
              forwardThird: [
                '## Three\n\nIn the evening, the sunny window put the kitten to sleep. So this cat has to be a sweet little darling.',
              ],
              backwardThird: [
                '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and So this cat must be a '
                  + 'sweet little treasure.',
              ],
              forwardFindings: [],
              backwardFindings: [],
              forwardWarned: [],
              backwardWarned: [],
            },);
            expect(readBackOf({ applied: forward, page, },),).toEqual(writtenOf({ applied: forward, },),);
            expect(readBackOf({ applied: backward, page, },),).toEqual(writtenOf({ applied: backward, },),);
          },
        },),
        it({
          name: 'KEEPS THE PLACE AN OVERLAPPED PLACE GAVE WAY TO WHEN A LATER MADE REPLACEMENT ALSO CHANGES ITS WORDS, in either '
            + 'order, the place named being the earlier winner and no finding being kept for the made replacements',
          fn: async () => {
            /**
             Three groups splice the kitten slice: the middle group's splice overlaps the first group's, so it loses to
             it, and the last group's splice, which does not overlap the first, also changes the middle group's words.
             */
            const settled = [
              kittenGroupSpliced({ quote: 'the kitten', splice: { start: 6, end: 16, text: 'the cat', }, },),
              kittenGroupSpliced({ quote: 'the kitten naps', splice: { start: 6, end: 21, text: 'the kitten sleeps', }, },),
              kittenGroupSpliced({ quote: 'naps', splice: { start: 17, end: 21, text: 'sleeps', }, },),
            ];
            /**
             What applying the groups in the given order comes to.
             */
            function appliedIn({ groups, }: { readonly groups: readonly SettledGroup[]; },): ReturnType<typeof applyConsistency> {
              return applyConsistency({
                settled: groups,
                slices: kittenSlices(),
                declared: [],
                declaredNames: [],
                l: levelCapturingLogger({ lines: [], },),
              },);
            }
            const forward = appliedIn({ groups: settled, },);
            const backward = appliedIn({ groups: settled.toReversed(), },);
            /**
             What either order comes to: the middle group's splice lost to the group that came first, whichever it was.
             */
            const expected = {
              outcomes: [['chosen', 'replaced',], ['chosen', 'overlapped',], ['chosen', 'replaced',],],
              gaveWay: [['none', 'none',], ['none', { group: 0, label: 'OCCURRENCE 2', },], ['none', 'none',],],
              slices: [{ sliceIndex: 1, text: 'Later the cat sleeps in the sun.', },],
              findings: [],
            };
            expect({
              forward: {
                outcomes: outcomesOf({ groups: forward.groups, },),
                gaveWay: gaveWayOf({ groups: forward.groups, },),
                slices: forward.slices,
                findings: forward.findings,
              },
              backward: {
                outcomes: outcomesOf({ groups: backward.groups, },),
                gaveWay: gaveWayOf({ groups: backward.groups, },),
                slices: backward.slices,
                findings: backward.findings,
              },
            },).toEqual({ forward: expected, backward: expected, },);
          },
        },),
        it({
          name: 'LEAVES BOTH OUT WHERE TWO GROUPS\' BLOCK REPLACEMENTS EACH CHANGE THE WORDS THE OTHER LOCATED, each place recorded '
            + 'overlapped with the other\'s place, and the section still reads back',
          fn: async () => {
            const page = settledCatPage({},);
            const third = nonNullishOrThrow(page.slices[2],);
            const english = third.current.kind === 'present' ? third.current.text : '';
            const blockStart = english.indexOf('In the evening',);
            /**
             A writer's block over the third slice's paragraph for the places of one group.
             */
            function blockOver(
              { placePositions, text, }: { readonly placePositions: readonly number[]; readonly text: string; },
            ): PendingEdit {
              return { kind: 'block', placePositions, sliceIndex: 2, start: blockStart, end: english.length, text, };
            }
            const lines: string[] = [];
            const applied = applyCat({
              page: {
                ...page,
                settled: page.settled.map(function rewritten(group, at,): SettledGroup {
                  return (at === 0)
                    ? {
                      ...group,
                      edits: [
                        blockOver({
                          placePositions: [1,],
                          text: 'In the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. '
                            + 'So this cat is a darling.',
                        },),
                      ],
                    }
                    : {
                      ...group,
                      edits: group.edits.map(function blockInSecond(edit,) {
                        return (edit.sliceIndex === 2)
                          ? blockOver({
                            placePositions: edit.placePositions,
                            text: 'In the evening, the kitten slept in the sun. So this cat has to be a sweet little darling.',
                          },)
                          : edit;
                      },),
                    };
                },),
              },
              lines,
            },);
            expect({
              outcomes: outcomesOf({ groups: applied.groups, },),
              gaveWay: gaveWayOf({ groups: applied.groups, },),
              changed: applied.slices.map(function indexOf(slice,): number {
                return slice.sliceIndex;
              },),
              findings: applied.findings,
            },).toEqual({
              outcomes: [['chosen', 'overlapped',], ['chosen', 'replaced', 'overlapped', 'replaced',],],
              gaveWay: [
                ['none', { group: 1, label: 'OCCURRENCE 3', },],
                ['none', 'none', { group: 0, label: 'OCCURRENCE 2', }, 'none',],
              ],
              changed: [1, 3,],
              findings: [
                'slice 2: the consistency replacement of group 1 OCCURRENCE 2 was left out because it changes the words that '
                  + 'group 2 OCCURRENCE 3 located',
                'slice 2: the consistency replacement of group 2 OCCURRENCE 3 was left out because it changes the words that '
                  + 'group 1 OCCURRENCE 2 located',
              ],
            },);
            expect(readBackOf({ applied, page, },),).toEqual(writtenOf({ applied, },),);
          },
        },),
        it({
          name: 'KEEPS A SLICE\'S WORDING WHEN A FLOOR REFUSES IT, recording kept-floor for the replacement meant for it, the '
            + 'finding and a warning, and leaves the other slices changed',
          fn: async () => {
            const page = settledCatPage({
              english: [
                nonNullishOrThrow(ENGLISH[0],),
                '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat is surely Mittens, a lazy little treasure.',
                nonNullishOrThrow(ENGLISH[2],),
                nonNullishOrThrow(ENGLISH[3],),
              ],
              frameQuotes: [
                nonNullishOrThrow(REFRAIN_QUOTES[0],),
                { label: 'OCCURRENCE 2', quote: 'So this cat is surely Mittens, a lazy little treasure.', slot: 'lazy', },
                nonNullishOrThrow(REFRAIN_QUOTES[2],),
                nonNullishOrThrow(REFRAIN_QUOTES[3],),
              ],
            },);
            const lines: string[] = [];
            const applied = applyConsistency({
              settled: page.settled,
              slices: page.slices,
              declared: [],
              declaredNames: ['Mittens',],
              l: levelCapturingLogger({ lines, },),
            },);
            const slice = nonNullishOrThrow(page.slices[1],);
            const refused = consistencyFloorVerdict({
              sliceIndex: 1,
              sourceText: slice.sourceText,
              currentText: slice.current.kind === 'present' ? slice.current.text : '',
              candidateText: '## Two\n\nThe smell of fish in the kitchen woke the kitten. So this cat must be a lazy little treasure.',
              lineStructured: false,
              declared: [],
              declaredNames: ['Mittens',],
              disputedWordings: [],
            },);
            if (refused.kind !== 'refused')
              throw new Error('Expected the floors to refuse the slice without its declared name',);
            expect(outcomesOf({ groups: applied.groups, },)[1],).toEqual(['chosen', 'kept-floor', 'replaced', 'replaced',],);
            expect(applied.slices.map(function indexOf(changed,): number {
              return changed.sliceIndex;
            },),).toEqual([2, 3,],);
            expect(applied.findings,).toEqual([
              `slice 1: the floors refused the consistency replacement: ${refused.findings.join(' | ',)}`,
            ],);
            expect(lines,).toEqual([
              `warn [applyConsistency] slice 1: the floors refused the consistency replacement, so the slice keeps its wording: ${
                refused.findings.join(' | ',)
              }`,
            ],);
          },
        },),
        it({
          name: 'KEEPS OVERLAPPED AS THE OUTCOME OF A REPLACEMENT LEFT OUT EVEN WHEN A FLOOR THEN REFUSES THE SLICE, and records '
            + 'kept-floor for the replacement that was made in it',
          fn: async () => {
            const page = settledCatPage({
              sentenceQuotes: [
                nonNullishOrThrow(SENTENCE_QUOTES[0],),
                {
                  label: 'OCCURRENCE 2',
                  quote: 'the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. '
                    + 'So this cat has to be a sweet little darling',
                },
              ],
            },);
            const applied = applyConsistency({
              settled: page.settled,
              slices: page.slices,
              declared: [],
              declaredNames: ['sunlight',],
              l: levelCapturingLogger({ lines: [], },),
            },);
            expect(outcomesOf({ groups: applied.groups, },),).toEqual([
              ['chosen', 'kept-floor',],
              ['chosen', 'replaced', 'overlapped', 'replaced',],
            ],);
            expect(gaveWayOf({ groups: applied.groups, },),).toEqual([
              ['none', 'none',],
              ['none', 'none', { group: 0, label: 'OCCURRENCE 2', }, 'none',],
            ],);
            expect(applied.slices.map(function indexOf(slice,): number {
              return slice.sliceIndex;
            },),).toEqual([1, 3,],);
          },
        },),
        it({
          name: 'REFUSES A SETTLED GROUP WHOSE RENDERINGS OR OUTCOMES ARE FEWER THAN ITS PLACES, naming the place and the group',
          fn: async () => {
            const page = settledCatPage({},);
            const [first,] = page.settled;
            const group = nonNullishOrThrow(first,);
            /**
             The text of the refusal applying a changed group raises.
             */
            function refusalOf({ changed, }: { readonly changed: SettledGroup; },): string {
              return String(caught(function applies(): unknown {
                return applyConsistency({
                  settled: [changed,],
                  slices: page.slices,
                  declared: [],
                  declaredNames: [],
                  l: levelCapturingLogger({ lines: [], },),
                },);
              },),);
            }
            expect(caught(function applies(): unknown {
              return applyConsistency({
                settled: [{ ...group, outcomes: [], },],
                slices: page.slices,
                declared: [],
                declaredNames: [],
                l: levelCapturingLogger({ lines: [], },),
              },);
            },),).toBeInstanceOf(ConsistencyInvariantError,);
            expect([
              refusalOf({ changed: { ...group, renderings: group.renderings.slice(0, 1,), }, }),
              refusalOf({ changed: { ...group, outcomes: [], }, }),
            ],).toEqual([
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 of group 1 has no rendering or outcome, '
                + 'and both are made for every place',
              'ConsistencyInvariantError: unreachable: OCCURRENCE 1 of group 1 has no rendering or outcome, '
                + 'and both are made for every place',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE TO BE REPLACED WHOSE REPLACEMENT NEVER REACHED A SLICE THE PASS READS, naming the place and the group',
          fn: async () => {
            const page = settledCatPage({},);
            const absent = page.slices
              .map(function withoutSecond(facts,): ConsistencySliceFacts {
                return (facts.sliceIndex === 2) ? { ...facts, current: { kind: 'absent', }, } : facts;
              },);
            const refusal = caught(function applies(): unknown {
              return applyConsistency({
                settled: page.settled,
                slices: absent,
                declared: [],
                declaredNames: [],
                l: levelCapturingLogger({ lines: [], },),
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 of group 1 was to be replaced and no replacement was applied, '
                + 'left out or refused, and every pending replacement is one of the three',
            );
          },
        },),
        it({
          name: 'REFUSES TWO REPLACEMENTS OF ONE GROUP WHOSE WORDS OVERLAP BEFORE ANY OUTCOME IS RECORDED, naming the group and '
            + 'both places, and accepts two that only touch',
          fn: async () => {
            /**
             The group of the kitten page with a replacement over each of its two places in the kitten slice.

             @param splice - where the splice over the first place starts and ends
             */
            function groupWithTwoEdits({ splice, }: { readonly splice: { readonly start: number; readonly end: number; }; },): SettledGroup {
              return {
                ...groupIn({
                  places: [
                    { slice: 3, english: 'Dusk: the kitten purrs in the sun.', quote: 'in the sun', },
                    { slice: 1, english: KITTEN_SLICE, quote: 'Later the kitten', },
                    { slice: 1, english: KITTEN_SLICE, quote: 'naps in the sun', },
                  ],
                },),
                outcomes: ['chosen', 'replaced', 'replaced',],
                edits: [
                  { kind: 'splice', placePositions: [1,], sliceIndex: 1, ...splice, text: 'Soon the kitten', },
                  { kind: 'block', placePositions: [2,], sliceIndex: 1, start: 16, end: 33, text: ' naps in the sun!', },
                ],
              };
            }
            /**
             Applies the group's replacements to the kitten page.

             @param group - group
             */
            function appliedTo({ group, }: { readonly group: SettledGroup; },): ReturnType<typeof applyConsistency> {
              return applyConsistency({
                settled: [group,],
                slices: kittenSlices(),
                declared: [],
                declaredNames: [],
                l: levelCapturingLogger({ lines: [], },),
              },);
            }
            const overlapping = caught(function appliesOverlapping(): unknown {
              return appliedTo({ group: groupWithTwoEdits({ splice: { start: 0, end: 20, }, }), },);
            },);
            expect(overlapping,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(overlapping,),).toBe(
              'ConsistencyInvariantError: unreachable: the replacements group 1 OCCURRENCE 2 and group 1 OCCURRENCE 3 of slice 1 '
                + 'overlap, and no two replacements of one group address the same words',
            );
            expect(outcomesOf({ groups: appliedTo({ group: groupWithTwoEdits({ splice: { start: 0, end: 16, }, }), },).groups, },),)
              .toEqual([['chosen', 'replaced', 'replaced',],],);
          },
        },),
        ...ARRANGEMENT_CASES.map(function arrangementIt({ name, model, expected, },) {
          return it({
            name: `RECORDS EVERY PLACE A MADE REPLACEMENT CHANGED AS OVERLAPPED IN THE ARBITRATION ITSELF, with a name, in every `
              + `arrangement of ${name}, so that no pass after the replacements are made has a place left to record`,
            fn: async () => {
              expect(arrangementOutcome({ model, },),).toEqual(expected,);
            },
          },);
        },),
      ],
    },),
  ],
},);
