/**
 A scene the tests of carrying the chosen rendering to one receiving place
 share: a repeated sentence whose chosen place is worded one way and whose
 receiving place is worded another, a scripted bench for both the writers and
 the judges, and a cache that keeps what it is handed. Cat-themed invention; no
 corpus content appears here.

 @module
 */

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  CarryContext,
  CarryReceiver,
  CarryUnit,
  CarryWriters,
  ConsistencyCacheValue,
  LocatedPlace,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  scriptedJudgeClient,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

//region Carry scene

/**
 The repeated sentence.
 */
export const CARRY_REPEAT = {
  kind: 'maximal',
  wording: '小猫打盹',
} as const;

/**
 The bench both the writers and the judges are drawn from, in roster order.
 */
const CARRY_BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_ONLY,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_VISION,
];

/**
 The English of the chosen place's slice.
 */
const CHOSEN_ENGLISH = 'The cat naps. It is warm.';

/**
 The English of the receiving place's slice, which is also its one paragraph.
 */
export const RECEIVING_ENGLISH = 'Later the kitten naps in the sun.';

/**
 The receiving paragraph with the chosen words moved in by code.
 */
export const SPLICED_PARAGRAPH = 'Later the cat naps in the sun.';

/**
 The original of the chosen place's slice.
 */
const CHOSEN_SOURCE = '小猫打盹。';

/**
 The original of the receiving place's slice.
 */
const RECEIVING_SOURCE = '傍晚的时候，窗台上小猫打盹，阳光暖暖的。';

/**
 The whole original: the chosen slice, a blank line, the receiving slice.
 */
const WHOLE_ORIGINAL = `${CHOSEN_SOURCE}\n\n${RECEIVING_SOURCE}`;

/**
 Where the receiving slice starts in the whole original.
 */
const RECEIVING_START = CHOSEN_SOURCE.length + 2;

/**
 Where the receiving place's words start in the whole original.
 */
const RECEIVING_WORDS_START = RECEIVING_START + RECEIVING_SOURCE.indexOf('小猫打盹',);

/**
 Everything a test of one carry reads and the sinks it writes to.

 @example
 ```ts
 const scene = carryScene({ behave, },);
 ```
 */
export type CarryScene = {
  /**
   What the carry reads.
   */
  readonly context: CarryContext;

  /**
   The chosen place with its located rendering.
   */
  readonly chosen: LocatedPlace;

  /**
   The receiving place, whose chosen words can be moved in by code.
   */
  readonly receiver: CarryReceiver;

  /**
   The receiving place's block as a unit of its own: its one paragraph, which is
   its slice's whole English, holding nothing to keep.
   */
  readonly unit: CarryUnit;

  /**
   Seat of every structured call, in order.
   */
  readonly asked: RosterModelId[];

  /**
   Messages of every structured call, in order.
   */
  readonly sheets: string[];

  /**
   Level and text of every line logged.
   */
  readonly lines: string[];

  /**
   Every question and answer handed to the store.
   */
  readonly persisted: {
    readonly key: string;
    readonly serialized: string
  }[];
};

/**
 Builds the scene.

 @param behave - what each scripted seat does

 @param resumed - what an earlier run settled

 @param writers - whether the writers are asked, the whole bench by default

 @returns The scene

 @example
 ```ts
 const scene = carryScene({ behave, },);
 ```
 */
export function carryScene(
  {
    behave,
    resumed = new Map<string, ConsistencyCacheValue>(),
    writers = {
      kind: 'asked',
      modelIds: CARRY_BENCH,
    },
  }: {
    readonly behave: SeatScript;
    readonly resumed?: ReadonlyMap<string, ConsistencyCacheValue>;
    readonly writers?: CarryWriters;
  },
): CarryScene {
  /**
   Seat of every structured call, in order.
   */
  const asked: RosterModelId[] = [];
  /**
   Messages of every structured call, in order.
   */
  const sheets: string[] = [];
  /**
   Level and text of every line logged.
   */
  const lines: string[] = [];
  /**
   Every question and answer handed to the store.
   */
  const persisted: {
    key: string;
    serialized: string
  }[] = [];
  /**
   The chosen place, worded as the chosen sentence is.
   */
  const chosen: LocatedPlace = {
    place: {
      startOffset: 0,
      endOffset: CHOSEN_SOURCE.length - 1,
      line: 1,
      slice: {
        sliceIndex: 0,
        sourceIndex: 0,
        sourceText: CHOSEN_SOURCE,
        sourceStart: 0,
        nodes: [],
        current: {
          kind: 'present',
          text: CHOSEN_ENGLISH,
        },
        lineStructured: false,
        disputedWordings: [],
      },
    },
    rendering: {
      kind: 'located',
      quote: 'The cat naps',
      core: 'The cat naps',
      start: 0,
    },
  };
  /**
   The receiving place, worded another way, before the plan to move the chosen words in is added.
   */
  const located: LocatedPlace = {
    place: {
      startOffset: RECEIVING_WORDS_START,
      endOffset: RECEIVING_WORDS_START + '小猫打盹'.length,
      line: 3,
      slice: {
        sliceIndex: 1,
        sourceIndex: 1,
        sourceText: RECEIVING_SOURCE,
        sourceStart: RECEIVING_START,
        nodes: [],
        current: {
          kind: 'present',
          text: RECEIVING_ENGLISH,
        },
        lineStructured: false,
        disputedWordings: [],
      },
    },
    rendering: {
      kind: 'located',
      quote: 'the kitten naps',
      core: 'the kitten naps',
      start: 6,
    },
  };
  /**
   The receiving place with the plan to move the chosen words in.
   */
  const receiver: CarryReceiver = {
    ...located,
    position: 1,
    label: 'OCCURRENCE 2',
    plan: {
      kind: 'replace',
      start: 6,
      end: 21,
      text: 'the cat naps',
    },
  };
  return {
    context: {
      client: scriptedJudgeClient({
        behave,
        asked,
        sheets,
      },),
      writers,
      judgeModelIds: CARRY_BENCH,
      cache: {
        resumed,
        persist: function persist(entry,): Promise<void> {
          persisted.push(entry,);
          return Promise.resolve();
        },
      },
      text: WHOLE_ORIGINAL,
      declared: [],
      declaredNames: [],
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      l: levelCapturingLogger({ lines, },),
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },
    chosen,
    receiver,
    unit: {
      block: {
        start: 0,
        end: RECEIVING_ENGLISH.length,
        text: RECEIVING_ENGLISH,
      },
      writable: true,
      spliceOnlyIn: [],
      receivers: [receiver,],
      keeping: [],
      crossing: [],
    },
    asked,
    sheets,
    lines,
    persisted,
  };
}

/**
 A second receiving place of the scene's block, worded differently, whose words stand after the first's.

 @param receiver - the scene's receiving place

 @returns Another place of the same slice, at position 2, whose plan moves in the warm sun

 @example
 ```ts
 const second = secondReceiverOf({ receiver: scene.receiver, },);
 ```
 */
export function secondReceiverOf({ receiver, }: { readonly receiver: CarryReceiver; },): CarryReceiver {
  return {
    ...receiver,
    position: 2,
    label: 'OCCURRENCE 3',
    rendering: {
      kind: 'located',
      quote: 'the sun',
      core: 'the sun',
      start: 25,
    },
    plan: {
      kind: 'replace',
      start: 25,
      end: 32,
      text: 'the warm sun',
    },
  };
}

//endregion Carry scene
