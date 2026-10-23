/**
 Tests for the translate probe's run: which section it demonstrates on, what
 it prints for each slice it asks about, how it reports a slice that failed
 and what it refuses, over a throwaway corpus and a scripted roster.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { armedDeadlinesDuring, } from
  '@monochromatic-dev/translation-repair-provider/ts/armed-deadlines.test-fixture.ts';
import { transportLandingAfter, } from
  '@monochromatic-dev/translation-repair-provider/ts/delayed-transport.test-fixture.ts';
import {
  claudeModelClientOver,
  claudeStreamOf,
  PLAIN_USAGE,
} from '@monochromatic-dev/translation-repair-provider/ts/claude-client.test-fixture.ts';
import { scripted, } from '@monochromatic-dev/translation-repair-provider/ts/direct-client.test-fixture.ts';
import { probeTranslate, } from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type FakeClockWalk,
  walkOnFakeClock,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/fake-clock-walk.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  makeProbeCorpus,
  refusalOf,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Translators asked: two seats.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 What the translators were scripted to say, by model id; a model absent from
 the script stays silent.
 */
type Script = Readonly<Record<string, string>>;

/**
 Client answering each translator from a script.

 @param script - rendered English each model returns

 @returns Client honoring that script, which never reaches a network

 @example
 ```ts
 const client = scriptedClient({ script: { 'hf:cat/Cat-A': 'The cat naps.', }, },);
 ```
 */
function scriptedClient({ script, }: { readonly script: Script; },): ModelClient {
  return {
    chatText: function noText(): never {
      throw new Error('chatText unused by the translate probe',);
    },
    quotas: function noQuotas(): never {
      throw new Error('quotas unused by the translate probe',);
    },
    chatJson: function answer<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Rendering this model was scripted to give.
       */
      const rendering = script[request.modelId];
      if (rendering === undefined)
        return Promise.resolve({
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        },);

      /**
       Wire value carrying it.
       */
      const value: unknown = { translation: rendering, };
      if (!request.validate(value,))
        return Promise.reject(new Error('scripted reply failed the wire guard',),);
      return Promise.resolve({
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      },);
    },
  };
}

/**
 What the probe printed and how many clients it built.
 */
type Probed = {
  /**
   Clients built, one for each slice asked about.
   */
  readonly built: number;

  /**
   Lines printed.
   */
  readonly lines: readonly string[];

  /**
   Whether the probe was refused, and with what, empty when it ran.
   */
  readonly refusal: string;
};

/**
 Runs the probe over the given pages of the entry it is fixed on.

 @param files - corpus files by path

 @param newClient - what building the client for one slice does

 @param roster - translators asked

 @param sinon - the case's own sandbox

 @param perCallTimeoutMs - per-call figure the probe is handed

 @param drive - how the case runs the walk once the corpus stands; it calls the walk and awaits it, and a case that
 fakes the timers installs them here, after the corpus is made and before the walk starts

 @returns Clients built, lines printed and the refusal if any

 @example
 ```ts
 const probed = await probeOver({ files, newClient, sinon, },);
 ```
 */
async function probeOver(
  {
    files,
    newClient,
    roster,
    sinon,
    perCallTimeoutMs = HANG_STOP_MS,
    drive = async function plainly(walkOnce,): Promise<void> {
      await walkOnce();
    },
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly newClient: () => ModelClient;
    readonly roster: readonly RosterModelId[];
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
    readonly perCallTimeoutMs?: number;
    readonly drive?: (walkOnce: () => Promise<void>,) => Promise<void>;
  },
): Promise<Probed> {
  await using corpus = await makeProbeCorpus({ files, },);
  using printed = divertingConsoleLog({ sinon, },);
  const { logger, } = capturingLoggerPair();

  /**
   Clients built, one entry each.
   */
  const built: ModelClient[] = [];

  /**
   Builds the client for one slice and counts it.

   @returns The client the case scripted

   @example
   ```ts
   const client = counting();
   ```
   */
  function counting(): ModelClient {
    /**
     Client the case scripted for this slice.
     */
    const client = newClient();
    built.push(client,);
    return client;
  }

  /**
   Runs the probe, as the case under test asks.
   */
  async function walk(): Promise<void> {
    await drive(async function walkOnce(): Promise<void> {
      await probeTranslate({
        entryId: 'Mittens',
        pin: corpus.pin,
        newClient: counting,
        editorModelIds: roster,
        perCallTimeoutMs,
        log: logger,
        stageSettings: SCRIPTED_STAGE_SETTINGS,
      },);
    },);
  }

  /**
   What the probe refused with, empty when it ran.
   */
  const refusal = await refusalOf({ run: walk, },);
  return {
    built: built.length,
    lines: printed.lines,
    refusal,
  };
}

/**
 Original whose headings carry romanised names, the shape the aligner can
 prove a missing section in.
 */
const ANCHORED_SOURCE = `开场白：猫猫登场。

## 其一：Mittens

猫猫喜欢晒太阳，尾巴一摇一摇。

## 其二：Boots

猫猫每天下午都在窗台上打盹，直到太阳落下。

## 其三：Paws

猫猫和邻居家的黑猫是好朋友，它们常常一起追蝴蝶。
`;

/**
 Translation of that original missing its middle section.
 */
const ANCHORED_TARGET = `Prologue: the cat arrives.

## Mittens

The cat loves sunbathing, tail swishing.

## Paws

The cat and the black cat next door are friends.
`;

/**
 Original of the fixture entry: a heading and two blocks.
 */
const SOURCE_PAGE = '# 猫\n\n猫睡觉。\n\n猫吃鱼。\n';

/**
 Translation of the fixture original, lacking a block.
 */
const TARGET_PAGE = '# Cat\n\nThe cat naps.\n';

/**
 The deadline a Claude seat is armed with: 32,000 tokens at the floor rate of 80 a second, and a minute more.
 */
const CLAUDE_DEADLINE_MS = 460_000;

/**
 The per-call figure the run hands every call, which a caller-side cut would use.
 */
const RUN_PER_CALL_MS = 360_000;

/**
 Longest the fake clock is ticked for a reply that is meant to be heard: past the Claude deadline and a minute more.
 */
const LONGEST_TICKING_MS = 520_000;

/**
 What a probe over one Claude seat whose reply lands late did.
 */
type LateClaudeProbe = {
  /**
   Lines the probe printed.
   */
  readonly lines: readonly string[];

  /**
   What the probe was refused with, empty when it ran.
   */
  readonly refusal: string;

  /**
   Reasons the transport saw exchanges cut for by an abort signal, in order.
   */
  readonly cut: readonly unknown[];

  /**
   How many exchanges had been cut after each tick, in the order of the ticks.
   */
  readonly cutsAfterEachTick: readonly number[];

  /**
   Whether the probe had ended on its own when the ticking stopped, so the voice was heard or lost by the clock and not
   by the case's release.
   */
  readonly endedByItself: boolean;
};

/**
 Runs the probe over one Claude seat whose reply lands after a delay, on a fake clock, and reports what it did.

 EVERY CALLER-SIDE CUT IS DRIVEN BY THE SAME CLOCK: `setTimeout` and `clearTimeout` are fake, and the signal factory
 `AbortSignal.timeout` is replaced by one built on the fake `setTimeout`, so a signal made from a timer, from
 `AbortSignal.timeout` or from `AbortSignal.any` over either aborts at its figure on the fake clock and the cut shows.
 The clock stands still until the probe has made its call, since the probe reads its corpus over real I/O first, and
 the transport is released when the ticking stops, so the walk always ends.

 @param landMs - delay before the scripted reply lands

 @param ticks - fake milliseconds to advance, one step after another

 @param waitForEnd - whether to wait for the probe to end once the ticking is done, which a probe meant to outlast the
 ticking does not

 @param sinon - case's own sandbox

 @returns What the probe printed and what the transport saw

 @example
 ```ts
 const { lines, cut, } = await probeClaudeLandingAfter({
   landMs: 400_000,
   ticks: [LONGEST_TICKING_MS,],
   waitForEnd: true,
   sinon,
 },);
 ```
 */
async function probeClaudeLandingAfter(
  {
    landMs,
    ticks,
    waitForEnd,
    sinon,
  }: {
    readonly landMs: number;
    readonly ticks: readonly number[];
    readonly waitForEnd: boolean;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<LateClaudeProbe> {
  const { transport, firstCall, cut, release, } = transportLandingAfter({
    landMs,
    reply: {
      status: 200,
      bodyText: claudeStreamOf({
        text: '{"translation":"The cat naps."}',
        stopReason: 'end_turn',
        usage: PLAIN_USAGE,
      },),
    },
  },);

  /**
   What the driver reported, in a list of none or one.
   */
  const walked: FakeClockWalk[] = [];
  const probed = await probeOver({
    files: {
      'people/Mittens/page.md': SOURCE_PAGE,
      'people/Mittens/page.en.md': TARGET_PAGE,
    },
    newClient: function claudeVoice(): ModelClient {
      return claudeModelClientOver({ transport, },);
    },
    roster: ['claude-sonnet-5-5',],
    sinon,
    perCallTimeoutMs: RUN_PER_CALL_MS,
    drive: async function onFakeClock(walkOnce,): Promise<void> {
      walked.push(await walkOnFakeClock({
        sinon,
        walkOnce,
        firstCall,
        release,
        ticks,
        cutCount: () => cut.length,
        waitForEnd,
      },),);
    },
  },);
  return {
    lines: probed.lines,
    refusal: probed.refusal,
    cut,
    cutsAfterEachTick: walked.flatMap(function cutsOf(walk,): readonly number[] { return walk.cutsAfterEachTick; },),
    endedByItself: walked.some(function endedOf(walk,): boolean { return walk.endedByItself; },),
  };
}

await describe({
  name: probeTranslate.name,
  // SEQUENTIAL: one case wraps the process-wide timer function and one spies on the process-wide signal factory.
  concurrency: 1,
  children: [
    it({
      name: 'ARMS NO CALLER-SIDE TIMEOUT SIGNAL (F1 of the re-check of the second fix round): a signal that aborts at '
        + 'the per-call figure cuts a Claude call at 360,000 ms whatever deadline the client arms inside',
      fn: async (ctx) => {
        const timeoutSignal = ctx.sinon.spy(
          AbortSignal,
          'timeout',
        );
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': SOURCE_PAGE,
            'people/Mittens/page.en.md': TARGET_PAGE,
          },
          newClient: function oneVoice(): ModelClient {
            return scriptedClient({ script: { [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'The cat dozes.', }, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed.refusal,).toBe('',);
        expect(timeoutSignal.callCount,).toBe(0,);
      },
    },),
    it({
      name: 'ARMS A CLAUDE SEAT AT THE CLAUDE DEADLINE, 460,000 MS, WHICH THE PROBE\'S OWN PER-CALL FIGURE DOES NOT '
        + 'REACH',
      fn: async (ctx) => {
        const { transport, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({
              text: '{"translation":"The cat naps."}',
              stopReason: 'end_turn',
              usage: PLAIN_USAGE,
            },),
          },],
        },);
        const { armed, result, } = await armedDeadlinesDuring({
          run: async () =>
            await probeOver({
              files: {
                'people/Mittens/page.md': SOURCE_PAGE,
                'people/Mittens/page.en.md': TARGET_PAGE,
              },
              newClient: function claudeVoice(): ModelClient {
                return claudeModelClientOver({ transport, },);
              },
              roster: ['claude-sonnet-5-5',],
              sinon: ctx.sinon,
            },),
        },);

        expect(result.refusal,).toBe('',);
        expect(result.lines.includes('HEARD 1/1',),).toBe(true,);
        expect(armed.includes(CLAUDE_DEADLINE_MS,),).toBe(true,);
      },
    },),
    it({
      name: 'HEARS A CLAUDE REPLY THAT LANDS AT 400,000 MS: no signal the '
        + 'caller hands the client aborts before the Claude deadline of 460,000 ms, whether it is built from '
        + 'AbortSignal.timeout, from setTimeout or from AbortSignal.any',
      fn: async (ctx) => {
        const probed = await probeClaudeLandingAfter({
          landMs: 400_000,
          ticks: [LONGEST_TICKING_MS,],
          waitForEnd: true,
          sinon: ctx.sinon,
        },);

        expect(probed.refusal,).toBe('',);
        expect(probed.cut,).toEqual([],);
        expect(probed.endedByItself,).toBe(true,);
        expect(probed.lines.includes('HEARD 1/1',),).toBe(true,);
      },
    },),
    it({
      name: 'CUTS A CLAUDE REPLY THAT LANDS AT 500,000 MS AT 460,000 MS, not before (the positive control): the '
        + 'rig shows the client\'s own deadline firing, so the heard reply at 400,000 ms is a measurement and not a '
        + 'rig that cannot cut',
      fn: async (ctx) => {
        const probed = await probeClaudeLandingAfter({ landMs: 500_000,
          ticks: [CLAUDE_DEADLINE_MS - 1, 1,],
          waitForEnd: false,
          sinon: ctx.sinon,
        },);

        expect(probed.cutsAfterEachTick,).toEqual([0, 1,],);
        expect(probed.endedByItself,).toBe(false,);
      },
    },),
    it({
      name: 'PRINTS the section, the slice and what each heard voice rendered, with the finding for the seat that stayed silent',
      fn: async (ctx) => {
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': SOURCE_PAGE,
            'people/Mittens/page.en.md': TARGET_PAGE,
          },
          newClient: function oneVoice(): ModelClient {
            return scriptedClient({ script: { [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'The cat naps and eats fish.', }, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed,).toEqual({
          built: 1,
          lines: [
            'TRANSLATE Mittens: source 3 blocks / 15 chars, target 2 blocks / 20 chars, coverage 0.667',
            'TRANSLATE section subdivides into 1 slice; probing the first 1',
            '\n--- slice: 15 source chars, 20 target chars ---',
            'SOURCE: # 猫\n\n猫睡觉。\n\n猫吃鱼。',
            'HEARD 1/2',
            `  ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: The cat naps and eats fish.`,
            `  finding: stage-voice-lost (translate-probe ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
            '  finding: stage-roster-incomplete (translate-probe 1/2)',
          ],
          refusal: '',
        },);
      },
    },),
    it({
      name: 'PROBES only the first three slices of a section that cuts into more, building a client for each and printing every voice heard',
      fn: async (ctx) => {
        /**
         Paragraphs each long enough to be a slice of their own.
         */
        const paragraphs = ['猫', '鱼', '窗', '碗',].map(function paragraphOf(character,): string {
          return character.repeat(450,);
        },);
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': `# 猫\n\n${paragraphs.join('\n\n',)}\n`,
            'people/Mittens/page.en.md': `# Cat\n\n${['a', 'b', 'c', 'd',].map(function englishOf(letter,): string {
              return letter.repeat(450,);
            },).join('\n\n',)}\n`,
          },
          newClient: function twoVoices(): ModelClient {
            return scriptedClient({ script: {
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'The cat naps.',
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat sleeps.',
            }, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed.built,).toBe(3,);
        expect(probed.refusal,).toBe('',);
        expect(probed.lines.filter(function isHeading(line,): boolean {
          return line.startsWith('\n--- slice: ',);
        },),).toEqual([
          '\n--- slice: 3 source chars, 5 target chars ---',
          '\n--- slice: 450 source chars, 450 target chars ---',
          '\n--- slice: 450 source chars, 450 target chars ---',
        ],);
        expect(probed.lines.filter(function isCount(line,): boolean {
          return line.startsWith('TRANSLATE section',) || line.startsWith('HEARD',);
        },),).toEqual([
          'TRANSLATE section subdivides into 5 slices; probing the first 3',
          'HEARD 2/2',
          'HEARD 2/2',
          'HEARD 2/2',
        ],);
      },
    },),
    it({
      name: 'REPORTS a slice whose round threw as failed, in the error\'s own text, and goes on to the next',
      fn: async (ctx) => {
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': SOURCE_PAGE,
            'people/Mittens/page.en.md': TARGET_PAGE,
          },
          newClient: function silent(): ModelClient {
            return scriptedClient({ script: {}, },);
          },
          roster: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          sinon: ctx.sinon,
        },);

        expect(probed.refusal,).toBe('',);
        expect(probed.lines.slice(4,),).toEqual([
          `  SLICE FAILED: StageRosterRepeatError: the translate-probe roster seats these models more than once: [${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}]; `
            + 'a repeated id is one model counted as several voices, which meets quorum on fewer independent voices than '
            + 'the roster size promises, and the ballots keyed by model id would keep only one of its replies',
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated when the client cannot be built, after printing the section and the slice\'s source and before any call, '
        + 'where it once reported the slice as failed and ran on',
      fn: async (ctx) => {
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': SOURCE_PAGE,
            'people/Mittens/page.en.md': TARGET_PAGE,
          },
          newClient: function noKey(): ModelClient {
            throw new StatedRefusalError({ says: 'the cat has no key', },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed,).toEqual({
          built: 0,
          lines: [
            'TRANSLATE Mittens: source 3 blocks / 15 chars, target 2 blocks / 20 chars, coverage 0.667',
            'TRANSLATE section subdivides into 1 slice; probing the first 1',
            '\n--- slice: 15 source chars, 20 target chars ---',
            'SOURCE: # 猫\n\n猫睡觉。\n\n猫吃鱼。',
          ],
          refusal: 'StatedRefusalError: the cat has no key',
        },);
      },
    },),
    it({
      name: 'PROBES a section the translation lacks altogether as one the translators write from nothing, showing it at coverage zero',
      fn: async (ctx) => {
        const probed = await probeOver({
          files: {
            'people/Mittens/page.md': ANCHORED_SOURCE,
            'people/Mittens/page.en.md': ANCHORED_TARGET,
          },
          newClient: function oneVoice(): ModelClient {
            return scriptedClient({ script: {
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'The cat naps on the sill.',
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 'The cat dozes on the sill.',
            }, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed.refusal,).toBe('',);
        // The renderings print in the order of a bench the prompt rotates (it moved when the house tense rule changed the
        // prompt), so the lines of the two voices are compared sorted.
        expect(probed.lines.slice(0, 5,),).toEqual([
          'TRANSLATE Mittens: source 2 blocks / 34 chars, target 0 blocks / 0 chars, coverage 0.000 (barely translated)',
          'TRANSLATE section subdivides into 1 slice; probing the first 1',
          '\n--- slice: 34 source chars, 0 target chars ---',
          'SOURCE: ## 其二：Boots\n\n猫猫每天下午都在窗台上打盹，直到太阳落下。',
          'HEARD 2/2',
        ],);
        expect(probed.lines.slice(5,).toSorted(),).toEqual([
          `  ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: The cat naps on the sill.`,
          `  ${SEAT_SYNTHETIC_VISION_WITHHELD}: The cat dozes on the sill.`,
        ].toSorted(),);
      },
    },),
    it({
      name: 'SAYS no section carries source blocks for entries with nothing to carve, building no client',
      fn: async (ctx) => {
        expect(await probeOver({
          files: {
            'people/Mittens/page.md': '',
            'people/Mittens/page.en.md': TARGET_PAGE,
          },
          newClient: function unused(): ModelClient {
            return scriptedClient({ script: {}, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },),).toEqual({
          built: 0,
          lines: ['TRANSLATE no aligned section carries source blocks',],
          refusal: '',
        },);
      },
    },),
    it({
      name: 'REFUSES an entry whose translation is absent, naming the page, before printing anything',
      fn: async (ctx) => {
        const probed = await probeOver({
          files: { 'people/Mittens/page.md': SOURCE_PAGE, },
          newClient: function unused(): ModelClient {
            return scriptedClient({ script: {}, },);
          },
          roster: ROSTER,
          sinon: ctx.sinon,
        },);

        expect(probed.built,).toBe(0,);
        expect(probed.lines,).toEqual([],);
        expect(probed.refusal.startsWith('CorpusReadError: corpus read failed for ',),).toBe(true,);
        expect(probed.refusal.endsWith(':people/Mittens/page.en.md (missing-object); the commit has no such path: check the path, or pin a commit that has it.',),).toBe(true,);
      },
    },),
  ],
},);
