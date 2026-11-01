/**
 Tests for the introduced-defect probe stage itself: what it asks, what it
 skips, and how it accounts for probers it could not hear.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  EMPTY_INTRODUCED_DEFECT_REPORT,
  type RepairRegion,
  runIntroducedDefectProbe,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the probes under test.
 */
const l = tagged({ tag: 'introduced-defect-probe-test', },);

/**
 Probers the fixtures configure.
 */
const PROBERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Region the editors replaced, dropping the second clause.
 */
const REGION: RepairRegion = {
  envelopeId: 'envelope/nap',
  issueIds: ['adjudicated/nap',],
  before: 'The cat is doing the sleeping, and she wakes at dusk.',
  editorAfter: 'The cat sleeps.',
};

/**
 Client answering with one scripted check per region, or refusing.

 @param verdict - verdict every prober casts on every region

 @param evidence - added-damage quote every prober offers

 @param silentModelIds - probers whose voice is always lost

 @param twiceModelIds - probers that answer the region twice

 @param prompts - shared log of every user sheet the stage sent

 @returns Client the stage calls

 @example
 ```ts
 const client = catClient({ verdict: 'uncertain', },);
 ```
 */
function catClient(
  {
    verdict = 'no-introduced-defect-found',
    evidence = '',
    silentModelIds = [],
    twiceModelIds = [],
    prompts = [],
  }: {
    readonly verdict?: string;
    readonly evidence?: string;
    readonly silentModelIds?: readonly string[];
    readonly twiceModelIds?: readonly string[];
    readonly prompts?: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the probe',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Last message, whose text the probe records.
       */
      const asked = request.messages.at(-1,);
      prompts.push((asked === undefined) ? '' : messageText({ message: asked, },),);
      if (silentModelIds.includes(request.modelId,)) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        };
      }

      /**
       One check for the single fixture region.
       */
      const check = {
        region: 1,
        verdict,
        category: '',
        severity: '',
        evidence,
        omittedText: '',
        verdictReason: '',
      };

      /**
       The reply, with the check cast twice by a prober that answers twice.
       */
      const scripted: unknown = { checks: twiceModelIds.includes(request.modelId,) ? [check, check,] : [check,], };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the probe',);
    },
  };
}

await describe({
  name: runIntroducedDefectProbe.name,
  children: [
    it({
      name: 'asks nobody when no region was replaced, since a chunk that '
        + 'changed nothing cannot have introduced anything',
      fn: async () => {
        /** Sheets the stage sent, which must stay empty. */
        const prompts: string[] = [];

        const report = await runIntroducedDefectProbe({
          client: catClient({ prompts, },),
          proberModelIds: PROBERS,
          sourceText: '猫在睡觉。',
          baselineText: 'The cat naps.',
          regions: [],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(prompts,).toHaveLength(0,);
        expect(report,).toEqual(EMPTY_INTRODUCED_DEFECT_REPORT,);
      },
    },),

    it({
      name: 'keeps the configured roster size beside the heard count, because '
        + 'two of three heard and two of six configured are different evidence '
        + 'and only the pair tells them apart',
      fn: async () => {
        const report = await runIntroducedDefectProbe({
          client: catClient({ silentModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], },),
          proberModelIds: PROBERS,
          sourceText: '猫在睡觉。',
          baselineText: REGION.before,
          regions: [REGION,],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(report.heardProbers,).toBe(2,);
        expect(report.configuredProbers,).toBe(3,);
        expect(report.regions,).toHaveLength(1,);
        expect(report.regions[0]
          ?.noneFound,).toBe(2,);
      },
    },),

    it({
      name: 'REPORTS A PROBER THAT ANSWERS ONE REGION TWICE as a duplicate-check finding under its model id, '
        + 'and counts its first check once',
      fn: async () => {
        const report = await runIntroducedDefectProbe({
          client: catClient({ twiceModelIds: [SEAT_HYPER_OPENROUTER_UNMEASURED,], },),
          proberModelIds: PROBERS,
          sourceText: '猫在睡觉。',
          baselineText: REGION.before,
          regions: [REGION,],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(report.findings,).toEqual([`${SEAT_HYPER_OPENROUTER_UNMEASURED}: duplicate-check (1)`,],);
        expect(report.regions[0]
          ?.noneFound,).toBe(PROBERS.length,);
      },
    },),

    it({
      name: 'screens each heard prober\'s claim rather than trusting it, so a '
        + 'quote every prober lifted from the baseline lands as contradicted',
      fn: async () => {
        const report = await runIntroducedDefectProbe({
          client: catClient({
            verdict: 'introduced-defect',
            evidence: 'The cat',
          },),
          proberModelIds: PROBERS,
          sourceText: '猫在睡觉。',
          baselineText: REGION.before,
          regions: [REGION,],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(report.regions[0]
          ?.contradicted,).toBe(PROBERS.length,);
        expect(report.regions[0]
          ?.corroborated,).toBe(0,);
        expect(report.regions[0]
          ?.claims,).toHaveLength(PROBERS.length,);
      },
    },),

    it({
      name: 'corroborates a quote the probers copied across a note between translators in the replacement, '
        + 'because the sheet shows the replacement without the note and the screen reads the quote there',
      fn: async () => {
        /** Region whose replacement keeps a note that ends its first line. */
        const noted: RepairRegion = {
          envelopeId: 'envelope/nap',
          issueIds: ['adjudicated/nap',],
          before: 'The cat slept.\nShe woke at dusk.',
          editorAfter: 'The cat sleeps.{/* 这句不太对劲 */}\nShe wakes at noon.',
        };
        /** Sheets the stage sent. */
        const prompts: string[] = [];

        const report = await runIntroducedDefectProbe({
          client: catClient({
            verdict: 'introduced-defect',
            evidence: 'The cat sleeps.\nShe wakes at noon.',
            prompts,
          },),
          proberModelIds: PROBERS,
          sourceText: '猫睡了。她黄昏醒来。',
          baselineText: noted.before,
          regions: [noted,],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(prompts.some(function showsNote(prompt,): boolean {
          return prompt.includes('这句不太对劲',);
        },),).toBe(false,);
        expect(report.regions[0]
          ?.corroborated,).toBe(PROBERS.length,);
        expect(report.regions[0]
          ?.unanchored,).toBe(0,);
      },
    },),

    it({
      name: 'sends no pre-existing defect list at all by default, which is what '
        + 'stopped the stage answering the same way whether or not damage was '
        + 'present, while still sending both texts the prober has to compare',
      fn: async () => {
        /** Sheets the stage sent. */
        const prompts: string[] = [];

        await runIntroducedDefectProbe({
          client: catClient({ prompts, },),
          proberModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          sourceText: '猫在睡觉。',
          baselineText: REGION.before,
          regions: [REGION,],
          issues: [],
          identityContext: '',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /** Single sheet the one prober received. */
        const sheet = prompts[0] ?? '';
        expect(sheet.includes('PRE-EXISTING',),).toBe(false,);
        expect(sheet.includes('(none recorded)',),).toBe(false,);
        expect(sheet.includes(REGION.editorAfter,),).toBe(true,);
        expect(sheet.includes(REGION.before,),).toBe(true,);
      },
    },),

    it({
      name: 'still renders the list on request, and still shows a prober only '
        + 'the accepted issues of the region it is being asked about, which is '
        + 'what keeps one region\'s prior defects from reading as another\'s',
      fn: async () => {
        /** Sheets the stage sent. */
        const prompts: string[] = [];

        await runIntroducedDefectProbe({
          client: catClient({ prompts, },),
          proberModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          sourceText: '猫在睡觉。',
          baselineText: REGION.before,
          regions: [REGION,],
          issues: [],
          identityContext: '',
          disclosure: 'rendered',
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /** Single sheet the one prober received. */
        const sheet = prompts[0] ?? '';
        expect(sheet.includes('PRE-EXISTING',),).toBe(true,);
        expect(sheet.includes('(none recorded)',),).toBe(true,);
      },
    },),
  ],
},);
