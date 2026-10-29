/**
 Tests for the coverage probe's run: which entries and passages it asks the
 roster about, the rows and progress lines each leaves, the file it keeps and
 what it prints, over a throwaway corpus, a throwaway runs directory and a
 scripted roster.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { readTextOrEmptyIfMissing, } from '@monochromatic-dev/translation-repair-typography';
import type {
  CoverageAnswer,
  runCoverageStage,
} from '@monochromatic-dev/translation-repair-archive';
import { runCoverageProbe, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  keptInertClient,
  makeProbeCorpus,
  refusalOf,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original of three sections against a translation of one: three section
 passages.
 */
const THREE_SECTIONS = '## 一\n\n猫一。\n\n## 二\n\n猫二。\n\n## 三\n\n猫三。\n';

/**
 Original of one section holding two blocks against a translation holding
 one: one block passage.
 */
const TWO_BLOCKS = '## 一\n\n猫一。\n\n猫二。\n';

/**
 Translation of one section.
 */
const ONE_SECTION = '## One\n\nCat one.\n';

/**
 Roster handed to every scripted stage.
 */
const ROSTER = [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, 'hf:Qwen/Qwen3.8-27B',] as const;

/**
 Digest every run claims, ending in the eight characters a file name keeps.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 What the closure of the run says.
 */
const CLOSURE = {
  kind: 'unavailable',
  reason: 'a cat sat on the entry',
} as const;

/**
 Answer a scripted stage gives for a passage.

 @param kind - verdict kind

 @returns Roster answer whose counts follow the kind

 @example
 ```ts
 const answer = answerOf({ kind: 'carried', },);
 ```
 */
function answerOf({ kind, }: { readonly kind: 'carried' | 'absent'; },): CoverageAnswer {
  return {
    verdict: {
      kind,
      anchoredFull: (kind === 'carried') ? 2 : 0,
      anchoredPartial: 0,
      absent: (kind === 'absent') ? 2 : 0,
      unanchored: 1,
      misattributed: 0,
      heard: 3,
      asked: 3,
      evidence: (kind === 'carried') ? ['Cat one.',] : [],
      unanchoredQuotes: ['a cat',],
      misattributedQuotes: [],
    },
    findings: [],
  };
}

/**
 The file a run keeps, as the probe store writes it.

 @param commitSha - corpus commit the run read

 @param walked - entries the run walked

 @param requested - entries the operator asked for

 @param rows - rows the run kept

 @returns Text of the file

 @example
 ```ts
 const text = keptFileOf({ commitSha, walked: ['Mittens',], requested: [], rows: [], },);
 ```
 */
function keptFileOf(
  {
    commitSha,
    walked,
    requested,
    rows,
  }: {
    readonly commitSha: string;
    readonly walked: readonly string[];
    readonly requested: readonly string[];
    readonly rows: readonly unknown[];
  },
): string {
  return JSON.stringify(
    {
      startedAt: '2026-10-06T01:00:00.000Z',
      finishedAt: '2026-10-06T01:05:00.000Z',
      pipelineDigest: DIGEST,
      runnerClosure: CLOSURE,
      roster: ROSTER,
      subject: {
        corpusPin: commitSha,
        entriesWalked: walked,
        entriesRequested: requested,
        candidateCap: 12,
      },
      rows,
    },
    undefined,
    2,
  );
}

/**
 What the run did.
 */
type Ran = {
  /**
   Clients built.
   */
  readonly built: number;

  /**
   Passages the stage was asked about, in order.
   */
  readonly passages: readonly string[];

  /**
   Progress lines, each behind its level.
   */
  readonly logged: readonly string[];

  /**
   Lines printed on standard output.
   */
  readonly printed: readonly string[];

  /**
   File the run kept, absent when it kept none.
   */
  readonly kept: string;

  /**
   Commit the corpus was read at.
   */
  readonly commitSha: string;

  /**
   What the run refused with, empty when it ran.
   */
  readonly refusal: string;
};

/**
 Runs the probe over the given files with a scripted stage.

 @param files - corpus files by path

 @param typed - arguments the operator wrote

 @param stageFor - what the stage does for the nth passage asked, from zero

 @param sinon - case's own sandbox

 @returns What the run did

 @example
 ```ts
 const ran = await runOver({ files, typed: [], stageFor, sinon, },);
 ```
 */
async function runOver(
  {
    files,
    typed,
    stageFor,
    sinon,
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly typed: readonly string[];
    readonly stageFor: (nth: number,) => CoverageAnswer;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<Ran> {
  await using corpus = await makeProbeCorpus({ files, },);
  await using runs = await scratchDir({ prefix: 'coverage-probe-runs-', },);
  using printed = divertingConsoleLog({ sinon, },);

  /**
   Progress lines.
   */
  const logged: string[] = [];

  /**
   Passages the stage was asked about.
   */
  const passages: string[] = [];

  /**
   Clients built, one entry each.
   */
  const built: ModelClient[] = [];

  /**
   Instants still to hand out.
   */
  const instants = ['2026-10-06T01:00:00.000Z', '2026-10-06T01:05:00.000Z',];

  /**
   Scripted stage: keeps the passage it was asked about and answers for it.

   @param input - what the run handed the stage

   @returns Scripted answer, or a throw where the script says so

   @example
   ```ts
   const answer = await scripted(input,);
   ```
   */
  function scripted(input: Parameters<typeof runCoverageStage>[0],): Promise<CoverageAnswer> {
    passages.push(input.sourcePassage,);
    return Promise.resolve(stageFor(passages.length - 1,),);
  }

  /**
   Runs the probe, as the case under test asks.
   */
  async function walk(): Promise<void> {
    await runCoverageProbe({
      line: lineOf({
        command: 'coverage-probe',
        typed,
      },),
      pin: corpus.pin,
      newClient: function newClient(): ModelClient {
        return keptInertClient({ clients: built, },);
      },
      stage: scripted,
      roster: ROSTER,
      exchangeTimeoutMs: HANG_STOP_MS,
      readDigest: function digest(): Promise<string> {
        return Promise.resolve(DIGEST,);
      },
      readClosure: function closure() {
        return Promise.resolve(CLOSURE,);
      },
      resolveRunsDir: function runsDir(): Promise<string> {
        return Promise.resolve(runs.path,);
      },
      now: function nextInstant(): string {
        return instants.shift() ?? '';
      },
      log: levelCapturingLogger({ lines: logged, },),
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }

  /**
   What the run refused with, empty when it ran.
   */
  const refusal = await refusalOf({ run: walk, },);

  /**
   File the run kept.
   */
  const keptPath = join(
    runs.path,
    'coverage-probe',
    '2026-10-06T01-00-00.000Z-cccccccc.json',
  );
  return {
    built: built.length,
    passages,
    logged: logged.map(function withoutPath(text,): string {
      return text.replace(
        keptPath,
        '<kept file>',
      );
    },),
    printed: printed.lines,
    kept: await readTextOrEmptyIfMissing({ path: keptPath, },),
    commitSha: corpus.commitSha,
    refusal,
  };
}

await describe({
  name: runCoverageProbe.name,
  concurrency: 1,
  children: [
    it({
      name: 'ASKS about every passage of an entry, keeps a row for each answer and for the call that failed, and prints the rows',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Mittens/page.md': THREE_SECTIONS,
            'people/Mittens/page.en.md': ONE_SECTION,
          },
          typed: [],
          stageFor: function answers(nth,): CoverageAnswer {
            if (nth === 1)
              throw new RangeError('secret whisker text',);
            return answerOf({ kind: (nth === 0) ? 'carried' : 'absent', },);
          },
          sinon: ctx.sinon,
        },);

        /**
         Rows both the file and the standard output carry.
         */
        const rows = [
          {
            entryId: 'Mittens',
            scale: 'section',
            where: 'section 0',
            sourceChars: 9,
            kind: 'carried',
            anchoredFull: 2,
            anchoredPartial: 0,
            absent: 0,
            unanchored: 1,
            heard: 3,
            asked: 3,
            unanchoredQuotes: ['a cat',],
            evidence: ['Cat one.',],
            findings: [],
          },
          {
            entryId: 'Mittens',
            scale: 'section',
            where: 'section 1',
            sourceChars: 9,
            kind: 'failed',
            anchoredFull: 0,
            anchoredPartial: 0,
            absent: 0,
            unanchored: 0,
            heard: 0,
            asked: 2,
            evidence: [],
            unanchoredQuotes: [],
            findings: ['refused by RangeError',],
          },
          {
            entryId: 'Mittens',
            scale: 'section',
            where: 'section 2',
            sourceChars: 9,
            kind: 'absent',
            anchoredFull: 0,
            anchoredPartial: 0,
            absent: 2,
            unanchored: 1,
            heard: 3,
            asked: 3,
            unanchoredQuotes: ['a cat',],
            evidence: [],
            findings: [],
          },
        ];
        expect(ran,).toEqual({
          built: 1,
          passages: ['## 一\n\n猫一。', '## 二\n\n猫二。', '## 三\n\n猫三。',],
          logged: [
            'info Mittens: 3 unpaired passages',
            'info Mittens section 0: carried (full 2, partial 0, absent 0, unanchored 1, heard 3 of 3)',
            'info Mittens section 1: FAILED refused by RangeError',
            'info Mittens section 2: absent (full 0, partial 0, absent 2, unanchored 1, heard 3 of 3)',
            'info kept 3 rows at <kept file>',
          ],
          printed: [JSON.stringify(
            { rows, },
            undefined,
            2,
          ),],
          kept: keptFileOf({
            commitSha: ran.commitSha,
            walked: ['Mittens',],
            requested: [],
            rows,
          },),
          commitSha: ran.commitSha,
          refusal: '',
        },);
      },
    },),
    it({
      name: 'SAYS passage in the singular for an entry with one unpaired passage, and keeps one row',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Tabby/page.md': TWO_BLOCKS,
            'people/Tabby/page.en.md': ONE_SECTION,
          },
          typed: [],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran.logged,).toEqual([
          'info Tabby: 1 unpaired passage',
          'info Tabby pair 0 block 2: carried (full 2, partial 0, absent 0, unanchored 1, heard 3 of 3)',
          'info kept 1 row at <kept file>',
        ],);
      },
    },),
    it({
      name: 'NAMES A PASSAGE THE PROVIDER REFUSED by the status and the provider\'s words on the progress line, '
        + 'the key its refusal echoed masked, and keeps the status alone in the row',
      fn: async (ctx) => {
        /**
         Refusal the real client raised over the real transport.
         */
        const failure = await statusFailureOf({
          sinon: ctx.sinon,
          status: 401,
        },);
        const ran = await runOver({
          files: {
            'people/Tabby/page.md': TWO_BLOCKS,
            'people/Tabby/page.en.md': ONE_SECTION,
          },
          typed: [],
          stageFor: function refused(): CoverageAnswer {
            throw failure;
          },
          sinon: ctx.sinon,
        },);

        expect(ran.logged,).toEqual([
          'info Tabby: 1 unpaired passage',
          `info Tabby pair 0 block 2: FAILED ${statusFailureLogText({ provider: 'synthetic', status: 401, },)}`,
          'info kept 1 row at <kept file>',
        ],);
        expect(ran.printed,).toEqual([JSON.stringify(
          {
            rows: [
              {
                entryId: 'Tabby',
                scale: 'block',
                where: 'pair 0 block 2',
                sourceChars: 3,
                kind: 'failed',
                anchoredFull: 0,
                anchoredPartial: 0,
                absent: 0,
                unanchored: 0,
                heard: 0,
                asked: 2,
                evidence: [],
                unanchoredQuotes: [],
                findings: ['refused by ProviderHttpError from synthetic with HTTP 401',],
              },
            ],
          },
          undefined,
          2,
        ),],);
      },
    },),
    it({
      name: 'STOPS at the cap, leaving the rest of an entry unasked and every later entry unread',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Cat1/page.md': THREE_SECTIONS,
            'people/Cat1/page.en.md': ONE_SECTION,
            'people/Cat2/page.md': THREE_SECTIONS,
          },
          typed: ['--cap', '2',],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran.passages,).toEqual(['## 一\n\n猫一。', '## 二\n\n猫二。',],);
        expect(ran.logged,).toEqual([
          'info Cat1: 3 unpaired passages',
          'info Cat1 section 0: carried (full 2, partial 0, absent 0, unanchored 1, heard 3 of 3)',
          'info Cat1 section 1: carried (full 2, partial 0, absent 0, unanchored 1, heard 3 of 3)',
          'info kept 2 rows at <kept file>',
        ],);
      },
    },),
    it({
      name: 'ASKS NOBODY under a cap of zero, keeping an empty file and printing empty rows, without reading a page',
      fn: async (ctx) => {
        const ran = await runOver({
          files: { 'people/Cat1/page.md': THREE_SECTIONS, },
          typed: ['--cap', '0',],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran.passages,).toEqual([],);
        expect(ran.logged,).toEqual(['info kept 0 rows at <kept file>',],);
        expect(ran.printed,).toEqual(['{\n  "rows": []\n}',],);
      },
    },),
    it({
      name: 'SKIPS an entry that lacks a page and an entry whose passages are all paired, and goes on to the next',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Cat1/page.md': THREE_SECTIONS,
            'people/Cat2/page.md': '## 一\n\n猫一。\n',
            'people/Cat2/page.en.md': ONE_SECTION,
            'people/Cat3/page.md': TWO_BLOCKS,
            'people/Cat3/page.en.md': ONE_SECTION,
          },
          typed: [],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran.logged.slice(0, 1,),).toEqual([
          'info Cat1: skipped, CorpusReadError: corpus read failed for '
            + `${ran.commitSha}:people/Cat1/page.en.md (missing-object); `
            + 'the commit has no such path: check the path, or pin a commit that has it.',
        ],);
        expect(ran.passages,).toEqual(['猫二。',],);
      },
    },),
    it({
      name: 'WALKS only the entries named after --only, recording what was asked for beside what was walked',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Cat1/page.md': TWO_BLOCKS,
            'people/Cat1/page.en.md': ONE_SECTION,
            'people/Cat2/page.md': TWO_BLOCKS,
            'people/Cat2/page.en.md': ONE_SECTION,
          },
          typed: ['--only', 'Cat2',],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran.logged.slice(0, 1,),).toEqual(['info Cat2: 1 unpaired passage',],);
        expect(ran.kept,).toBe(keptFileOf({
          commitSha: ran.commitSha,
          walked: ['Cat2',],
          requested: ['Cat2',],
          rows: [{
            entryId: 'Cat2',
            scale: 'block',
            where: 'pair 0 block 2',
            sourceChars: 3,
            kind: 'carried',
            anchoredFull: 2,
            anchoredPartial: 0,
            absent: 0,
            unanchored: 1,
            heard: 3,
            asked: 3,
            unanchoredQuotes: ['a cat',],
            evidence: ['Cat one.',],
            findings: [],
          },],
        },),);
      },
    },),
    it({
      name: 'REFUSES an entry the corpus lacks after building the client, before asking anyone or keeping a file',
      fn: async (ctx) => {
        const ran = await runOver({
          files: { 'people/Cat1/page.md': TWO_BLOCKS, },
          typed: ['--only', 'Nobody',],
          stageFor: function answers(): CoverageAnswer {
            return answerOf({ kind: 'carried', },);
          },
          sinon: ctx.sinon,
        },);

        expect(ran,).toEqual({
          built: 1,
          passages: [],
          logged: [],
          printed: [],
          kept: '',
          commitSha: ran.commitSha,
          refusal: 'StatedRefusalError: --only asks for "Nobody", which the corpus at the pin does not hold',
        },);
      },
    },),
  ],
},);
