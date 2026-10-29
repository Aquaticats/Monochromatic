/**
 Tests for the editor fidelity probe's run over a scripted reference, client and clock: what it refuses before anything
 is read or spent, what a zero cap says, the ceiling and the guard it prints, the arm it ends before a call that would
 pass `--max-usd`, and what a run writes to disk and prints, with the model ids it asked.

 No model is called and no corpus clone is read: the reference reader and the client builder are handed in. The real
 reviewed manifest supplies the request's specifications, and the invented cat texts of the fixture module stand in
 for what the pinned clone would hold.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { existsSync, } from 'node:fs';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  ClaudeAttemptAsk,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  promptPayloadStore,
  promptUniqueClient,
} from '@monochromatic-dev/translation-repair-stage';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  noteRunSpend,
  resetRunSpend,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  EDITOR_DAMAGE_KINDS,
  EDITOR_FIDELITY_POOL,
  editorDecisionRuleLine,
  editorFidelityTrials,
  type FidelityReferenceSpec,
  guardLine,
  RepliedWithoutCallError,
  REVIEWED_FIDELITY_REFERENCES,
  type ReviewedFidelityReference,
  runEditorFidelityProbe,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  HIGH_CARD,
  inventedReference,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Digest every run claims, ending in the eight characters a kept file's name carries.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(56,)}cafef00d`;

/**
 Closure every run records.
 */
const CLOSURE = {
  kind: 'read',
  entry: 'editor-fidelity-probe.mjs',
  chunks: ['run-config-abc.mjs',],
} as const;

/**
 Reviewed specification the cases select: the one entry whose manifest holds both families the probe builds, an
 insertion and an alteration.
 */
const SPEC: FidelityReferenceSpec = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES.find(function holdsBoth(spec,): boolean {
  return spec.damages.some(function isAlteration(damage,): boolean {
    return damage.kind === 'alteration';
  },);
},),);

/**
 What every scripted reply reports: two thousand prompt tokens and twenty thousand completion tokens, which cost
 (2000 x 2.5 + 20000 x 10) / 1e6 = 0.205 USD at the Sonnet prices, the prompt priced at the higher of the input and
 cache-write rates.
 */
const REPORTED_USAGE = {
  prompt_tokens: 2_000,
  completion_tokens: 20_000,
} as const;

/**
 What one scripted reply costs on the run meter, as the Claude client's SPEND line would put it there.
 */
const REPLY_USD = 0.205;

/**
 What a scripted attempt tells the admit it can cost when a case sets no figure: far under any amount a case allows.
 */
const SMALL_ATTEMPT_USD = 0.0001;

/**
 Instant the first clock reading gives.
 */
const STARTED_AT = '2026-10-23T06:00:00.000Z';

/**
 Instant the second clock reading gives.
 */
const FINISHED_AT = '2026-10-23T06:05:00.000Z';

/**
 The trials the invented reference makes, whose fixes the scripted client knows.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 What a run did, for the case to read.
 */
type ProbeRunSeen = {
  /**
   Every line the run logged, in order.
   */
  readonly lines: readonly string[];

  /**
   Everything written to the output sink.
   */
  readonly written: string;

  /**
   Model ids the scripted client was asked, in order.
   */
  readonly asked: readonly string[];

  /**
   Specifications the reference reader was handed, one entry per call.
   */
  readonly readFor: readonly (readonly FidelityReferenceSpec[])[];

  /**
   What each client build saw: the cards it was built for, and whether the plan was on disk by then.
   */
  readonly clientBuilds: readonly {
    readonly cardIds: readonly RosterModelId[];
    readonly promptPayloadDir: string;
    readonly planWasOnDisk: boolean;
  }[];

  /**
   Runs directory requests, one entry per call.
   */
  readonly runsAsked: readonly string[];

  /**
   What the run threw, empty when it completed.
   */
  readonly failures: readonly unknown[];
};

/**
 Reference reader answering with the invented reference for every specification it is handed.

 @param readFor - array each call's specifications are appended to

 @returns Reader the run takes

 @example
 ```ts
 const readReferences = inventedReader({ readFor: [], },);
 ```
 */
function inventedReader(
  { readFor, }: { readonly readFor: (readonly FidelityReferenceSpec[])[]; },
): Parameters<typeof runEditorFidelityProbe>[0]['readReferences'] {
  return function readInvented({ specs, },): Promise<readonly ReviewedFidelityReference[]> {
    readFor.push(specs,);
    return Promise.resolve(specs.map(function referenceOf(spec,): ReviewedFidelityReference {
      /**
       The invented reference, whose pages the pool specification's ranges must index into.
       */
      const invented = inventedReference();
      return {
        ...invented,
        spec: {
          ...spec,
          source: invented.spec.source,
          archive: invented.spec.archive,
        },
      };
    },),);
  };
}

/**
 Client that knows each trial's fix and applies it, unless told to say nothing readable.

 @param asked - array each call's model id is appended to

 @param silent - whether every reply is one nobody can read

 @param admit - admit the run handed the client builder, which the scripted client tells before its one attempt as
 the Claude client does; absent for a client that tells nobody

 @param attemptUsd - most the scripted attempt tells the admit it can cost

 @returns Client the run asks through

 @example
 ```ts
 const client = fixingClient({ asked: [], silent: false, },);
 ```
 */
function fixingClient(
  {
    asked,
    silent,
    admit,
    attemptUsd = SMALL_ATTEMPT_USD,
  }: {
    readonly asked: string[];
    readonly silent: boolean;
    readonly admit?: (attempt: ClaudeAttemptAsk,) => void;
    readonly attemptUsd?: number;
  },
): ModelClient {
  /**
   Tells the admit, notes the ask and puts the reply's cost on the meter, as the Claude client does for each call it
   makes, and builds the report the scripted editor gives.

   @param request - exchange asked

   @returns Report the editor gives for the trial the sheet shows
   */
  function answerTo(request: ChatTextRequest,): unknown {
    // THE CLAUDE CLIENT TELLS ITS CALLER BEFORE THE ATTEMPT, so a refusal ends the call before it is asked.
    admit?.({
      label: request.modelId,
      bound: { promptTokens: 2_000, completionTokens: 32_000, usd: attemptUsd, },
    },);
    asked.push(request.modelId,);
    // THE CLAUDE CLIENT PUTS EACH PRICED REPLY ON THE RUN METER; the guard reads the meter.
    noteRunSpend({ provider: 'anthropic', costUsd: REPLY_USD, },);
    /**
     The sheet's user message, which holds the region's current text.
     */
    const sheet = request.messages.map(function wordsOf(message,): string {
      if ((typeof message.content) === 'string')
        return message.content;
      return JSON.stringify(message.content,);
    },)
      .join('\n',);
    /**
     The trial whose region the sheet shows.
     */
    const trial = TRIALS.find(function isShown(candidate,): boolean {
      return sheet.includes(`CURRENT TEXT: ${candidate.damagedText.slice(
        candidate.region.startOffset,
        candidate.region.endOffset,
      )}`,);
    },);
    // The report for it: the region with the sentence removed, or the number put back.
    return (silent || (trial === undefined))
      ? { unrelated: true, }
      : {
        edits: [
          {
            region: 1,
            newText: trial.damagedText.slice(
              trial.region.startOffset,
              trial.region.endOffset,
            )
              .replace(
              trial.damagedDefect,
              trial.cleanDefect,
              ),
          },
        ],
      };
  }

  return {
    // The text exchange is what the prompt reuse layer buys through, so a run over that layer reaches it.
    chatText: (request,): Promise<ChatTextReply> => Promise.resolve({
      text: JSON.stringify(answerTo(request,),),
      usage: REPORTED_USAGE,
    },),
    quotas: () => Promise.reject(new Error('quotas unused by the editor',),),
    chatJson: <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       The report the editor gave.
       */
      const report = answerTo(request,);
      if (!request.validate(report,)) {
        return Promise.resolve({
          kind: 'schema-mismatch',
          rawText: JSON.stringify(report,),
          detail: 'unreadable',
          usage: REPORTED_USAGE,
        },);
      }
      return Promise.resolve({
        kind: 'ok',
        value: report,
        rawText: JSON.stringify(report,),
        usage: REPORTED_USAGE,
      },);
    },
  };
}

/**
 Runs the probe once over scripted parts, keeping its files in the directory given.

 @param runsDir - runs directory the run keeps its files in

 @param typed - what the operator wrote after the script path

 @param silent - whether the scripted client says nothing readable

 @param attemptUsd - most each scripted attempt tells the admit the run handed the client builder it can cost

 @param exactly - whether the line is passed as typed, without the spend of the earlier arms stated as zero for it

 @param reusing - whether the scripted client sits under the prompt reuse layer the run client has, with its payload
 store in the directory the run hands the builder, as the real client builder does

 @param unscoped - whether the exchanges reach that layer without the reuse scope the row names, which is a reuse the
 pass scope did not prevent

 @returns What the run did, once it settles, including what it threw

 @example
 ```ts
 const seen = await probeIn({ runsDir: scratch.path, typed: ['--cap', '0',], silent: false, },);
 ```
 */
async function probeIn(
  {
    runsDir,
    typed,
    silent = false,
    attemptUsd,
    exactly = false,
    reusing = false,
    unscoped = false,
  }: {
    readonly runsDir: string;
    readonly typed: readonly string[];
    readonly silent?: boolean;
    readonly attemptUsd?: number;
    readonly exactly?: boolean;
    readonly reusing?: boolean;
    readonly unscoped?: boolean;
  },
): Promise<ProbeRunSeen> {
  // The guard reads the process-wide run meter, so every run starts from a clean one.
  resetRunSpend();
  /**
   Lines the run logged.
   */
  const logged = capturingLoggerPair();
  /**
   Model ids asked.
   */
  const asked: string[] = [];
  /**
   Specifications the reader was handed.
   */
  const readFor: (readonly FidelityReferenceSpec[])[] = [];
  /**
   Client builds seen.
   */
  const clientBuilds: {
    cardIds: readonly RosterModelId[];
    promptPayloadDir: string;
    planWasOnDisk: boolean;
  }[] = [];
  /**
   Text written to the sink.
   */
  const chunks: string[] = [];
  /**
   Runs directory requests.
   */
  const runsAsked: string[] = [];
  /**
   What the run threw.
   */
  const failures: unknown[] = [];
  /**
   Clock readings still to give.
   */
  const instants = [STARTED_AT, FINISHED_AT,];
  try {
    await runEditorFidelityProbe({
      line: lineOf({
        command: 'editor-fidelity-probe',
        // The probe refuses a line that leaves out what the earlier arms spent, so a case that is not about that says zero.
        typed: (exactly || typed.includes('--bench-spent-usd',)) ? typed : ['--bench-spent-usd', '0', ...typed,],
      },),
      pin: {
        cloneDir: join(
          runsDir,
          'no-clone-is-read',
        ),
        commitSha: SPEC.corpusSha,
      },
      readReferences: inventedReader({ readFor, },),
      resolveRuns: function resolveScratch(): Promise<string> {
        runsAsked.push(runsDir,);
        return Promise.resolve(runsDir,);
      },
      newClient: function buildClient({ promptPayloadDir, cardIds, admitClaudeAttempt, },): ModelClient {
        clientBuilds.push({
          cardIds,
          promptPayloadDir,
          planWasOnDisk: existsSync(join(
            dirname(promptPayloadDir,),
            'plan.json',
          ),),
        },);
        /**
         The scripted editor, which tells the admit and writes the meter as the Claude client does.
         */
        const scripted = fixingClient({
          asked,
          silent,
          // Conditional spreads keep the members absent instead of undefined.
          ...((admitClaudeAttempt === undefined) ? {} : { admit: admitClaudeAttempt, }),
          ...((attemptUsd === undefined) ? {} : { attemptUsd, }),
        },);
        if (!reusing)
          return scripted;
        /**
         The reuse layer over the scripted editor, with its payload store.
         */
        const layer = promptUniqueClient({
          inner: scripted,
          store: promptPayloadStore({ dir: promptPayloadDir, },),
        },);
        return unscoped
          ? {
            ...layer,
            chatText: function withoutScopeText(request,) {
              const { reuseScope: _scope, ...rest } = request;
              return layer.chatText(rest,);
            },
            chatJson: function withoutScopeJson(request,) {
              const { reuseScope: _scope, ...rest } = request;
              return layer.chatJson(rest,);
            },
          }
          : layer;
      },
      now: function nextInstant(): string {
        return nonNullishOrThrow(instants.shift(),);
      },
      pipelineDigest: DIGEST,
      runnerClosure: CLOSURE,
      perCallTimeoutMs: HANG_STOP_MS,
      log: logged.logger,
      out: {
        write: function keep(text: string,): void {
          chunks.push(text,);
        },
      },
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }
  catch (error) {
    failures.push(error,);
  }
  return {
    lines: logged.lines,
    written: chunks.join('',),
    asked,
    readFor,
    clientBuilds,
    runsAsked,
    failures,
  };
}

/**
 The message the run's refusal carries.

 @param seen - what the run did

 @returns Stated message

 @throws {@link Error} when the run did not refuse with a stated refusal

 @example
 ```ts
 const said = refusedWith({ seen, },);
 ```
 */
function refusedWith({ seen, }: { readonly seen: ProbeRunSeen; },): string {
  const [failure,] = seen.failures;
  if (!(failure instanceof StatedRefusalError))
    throw new Error('the run was expected to refuse with a stated refusal',);
  return failure.message;
}

/**
 Flags every run case that asks something starts from: the entry the first reviewed reference holds, the one card alone.
 */
const ONE_CARD = [
  '--only',
  SPEC.entryId,
  '--candidates',
  LOW_CARD,
  '--candidates-alone',
];

await describe({
  name: runEditorFidelityProbe.name,
  // SEQUENTIAL: the spend guard reads the process-wide run meter, so runs must not overlap.
  concurrency: 1,
  children: [
    describe({
      name: 'refusals before anything is read',
      children: [
        it({
          name: 'REFUSES A CARD THE ROSTER DOES NOT KNOW, in the judge probe\'s wording, before reading or building '
            + 'anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ['--candidates', 'nobody/such-model', '--candidates-alone',],
            },);
            expect(refusedWith({ seen, },),).toContain(
              '--candidates names "nobody/such-model", which is not seatable; the roster knows ',
            );
            expect(seen.readFor,).toStrictEqual([],);
            expect(seen.clientBuilds,).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES --candidates-alone WITH NO CARD in the judge probe\'s wording, and a run that does not say '
            + 'alone, since this probe seats the named cards and nobody else and a roster beside them would spend '
            + 'on more than was asked',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            expect(refusedWith({
              seen: await probeIn({
                runsDir: scratch.path,
                typed: ['--candidates-alone',],
              },),
            },),).toBe(
              '--candidates-alone runs the candidates without the seated roster, and --candidates named none',
            );
            expect(refusedWith({
              seen: await probeIn({
                runsDir: scratch.path,
                typed: ['--candidates', LOW_CARD,],
              },),
            },),).toBe(
              'this probe seats only the cards --candidates names; write --candidates-alone to say so',
            );
            expect(refusedWith({
              seen: await probeIn({
                runsDir: scratch.path,
                typed: [],
              },),
            },),).toBe(
              'this probe seats only the cards --candidates names; write --candidates-alone to say so',
            );
          },
        },),

        it({
          name: 'REFUSES TO START AN ARM THAT WOULD PASS THE BENCH TOTAL, the spend the earlier arms measured plus '
            + 'this arm\'s --max-usd over sixty dollars, before reading or building anything, and starts one that '
            + 'reaches the total exactly',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const over = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--bench-spent-usd', '59', '--max-usd', '2',],
            },);
            expect(refusedWith({ seen: over, },),).toBe(
              'the bench would spend up to 61 USD: --bench-spent-usd 59 plus --max-usd 2 passes the 60 USD the owner '
                + 'ruled for all the arms of the editor fidelity bench together; write a --max-usd of at most 1, or '
                + 'stop the bench here',
            );
            expect(over.readFor,).toStrictEqual([],);
            expect(over.clientBuilds,).toStrictEqual([],);
            expect(over.runsAsked,).toStrictEqual([],);
            const exact = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--bench-spent-usd', '58', '--max-usd', '2', '--cap', '0',],
            },);
            expect(exact.failures,).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES A LINE THAT DOES NOT STATE --bench-spent-usd before reading or building anything, even with a '
            + 'cap of zero, since an arm told nothing of the earlier arms cannot hold the bench to its total',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--cap', '0',],
              exactly: true,
            },);
            expect(refusedWith({ seen, },),).toContain('--bench-spent-usd is required: write what the arms already run measured',);
            expect(seen.lines,).toStrictEqual([],);
            expect(seen.readFor,).toStrictEqual([],);
            expect(seen.clientBuilds,).toStrictEqual([],);
            expect(seen.runsAsked,).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES A CARD WITH NO PRICE TABLE before reading anything, since no ceiling could be printed for it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ['--candidates', 'hf:zai-org/GLM-5.3-Flash', '--candidates-alone',],
            },);
            expect(refusedWith({ seen, },),).toContain('has no price table in claude-card-sides.ts',);
            expect(seen.readFor,).toStrictEqual([],);
            expect(seen.clientBuilds,).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'a zero cap',
      children: [
        it({
          name: 'WRITES THE MANIFEST, POOL, CEILING, GUARD AND RULE LINES a run writes before its first call, then '
            + 'the preflight line, with the corpus read and no directory made, no client built and nothing asked',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--cap', '0',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines.length,).toBe(8,);
            expect(seen.lines[0],).toBe(`editors: ${LOW_CARD}`,);
            expect(seen.lines[1]?.startsWith('manifest: 1 specification, digest ',),).toBe(true,);
            expect(seen.lines[2],).toBe('pool: 2 trials over 1 entry (insertion 1, alteration 1)',);
            expect(seen.lines[3]?.startsWith(`ceiling ${LOW_CARD}: 2 units, one ask each, at most `,),).toBe(true,);
            expect(seen.lines[4],).toBe(guardLine({ maxUsd: 2, benchSpentUsd: 0, },),);
            expect(seen.lines[5],).toBe(editorDecisionRuleLine(),);
            expect(seen.lines[6],).toBe('planned: 2 of 2 trials',);
            expect(seen.lines[7],).toBe(
              `preflight only: 1 pinned reference specification selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
            expect(seen.readFor.length,).toBe(1,);
            expect(seen.runsAsked,).toStrictEqual([],);
            expect(seen.clientBuilds,).toStrictEqual([],);
            expect(seen.written,).toBe('',);
          },
        },),

        it({
          name: 'PRICES ONLY THE FIRST TRIALS when --trials is written beside it: a ceiling for that many, a planned line '
            + 'that says so, and a preflight line that does not claim to price the whole pool',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--trials', '1', '--cap', '0',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain('pool: 2 trials over 1 entry (insertion 1, alteration 1)',);
            expect(seen.lines.some(function isCeiling(line,): boolean {
              return line.startsWith(`ceiling ${LOW_CARD}: 1 unit, one ask each, at most `,);
            },),).toBe(true,);
            expect(seen.lines,).toContain('planned: 1 of 2 trials',);
            expect(seen.lines.at(-1,),).toBe(
              `preflight only: 1 pinned reference specification selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over the first 1 of 2 trials`,
            );
            expect(seen.clientBuilds,).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'the frozen pool',
      children: [
        it({
          name: 'SELECTS EVERY SPECIFICATION OF THE POOL when no entry is named, and a cap of zero says how many',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ['--candidates', LOW_CARD, '--candidates-alone', '--cap', '0',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain(
              `preflight only: ${String(EDITOR_FIDELITY_POOL.length,)} pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
          },
        },),

        it({
          name: 'READS A DRAWN ENTRY OF THE POOL by its id, the specifications of that entry and no others, in the pool\'s order',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const drawn = nonNullishOrThrow(EDITOR_FIDELITY_POOL.at(-1,),);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ['--only', drawn.entryId, '--candidates', LOW_CARD, '--candidates-alone', '--cap', '1',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.readFor,).toStrictEqual([
              EDITOR_FIDELITY_POOL.filter(function isEntry(spec,): boolean {
                return spec.entryId === drawn.entryId;
              },),
            ],);
          },
        },),

        it({
          name: 'SAYS WHAT IT DREW just before the ceiling, so the log of an arm shows the pool it saw',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            const ceilingAt = seen.lines.findIndex(function isCeiling(line,): boolean {
              return line.startsWith('ceiling ',);
            },);
            expect(ceilingAt,).toBeGreaterThan(0,);
            expect(seen.lines[ceilingAt - 1],).toBe('pool: 2 trials over 1 entry (insertion 1, alteration 1)',);
          },
        },),

        it({
          name: 'SAYS THE DIGEST OF THE MANIFEST IT ASKED just before the pool, so the logs of the arms can be checked '
            + 'to have asked the same specifications: the count and the sha256 of the specifications as JSON',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            const poolAt = seen.lines.findIndex(function isPool(line,): boolean {
              return line.startsWith('pool: ',);
            },);
            expect(poolAt,).toBeGreaterThan(0,);
            /**
             The specifications the run asked the reader for.
             */
            const asked = nonNullishOrThrow(seen.readFor[0],);
            expect(seen.lines[poolAt - 1],).toBe(
              `manifest: ${String(asked.length,)} ${asked.length === 1 ? 'specification' : 'specifications'}, digest ${
                hashContent({ content: JSON.stringify(asked,), },)
              }`,
            );
          },
        },),

        it({
          name: 'STATES THE DECISION RULE BEFORE THE FIRST CALL, so the log of every arm carries the rule the arms are read '
            + 'by and it cannot be chosen after the counts are seen',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            const ruleAt = seen.lines.indexOf(editorDecisionRuleLine(),);
            const firstRowAt = seen.lines.findIndex(function isRow(line,): boolean {
              return line.startsWith('row 1 of ',);
            },);
            expect(ruleAt,).toBeGreaterThan(0,);
            expect(ruleAt,).toBeLessThan(firstRowAt,);
          },
        },),
      ],
    },),

    describe({
      name: 'the ceiling',
      children: [
        it({
          name: 'PRINTS THE CEILING AND THE GUARD before any client exists, and does not refuse to start on the total: '
            + 'a --max-usd under the first-pass ceiling still builds the client, since the guard holds each call to it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--max-usd', '0.5', '--bench-spent-usd', '3.25',],
            },);
            const ceilingAt = seen.lines.findIndex(function isCeiling(line,): boolean {
              return line.startsWith(`ceiling ${LOW_CARD}: 2 units, one ask each, at most `,);
            },);
            expect(ceilingAt,).toBeGreaterThanOrEqual(0,);
            expect(seen.lines[ceilingAt + 1],).toBe(guardLine({ maxUsd: 0.5, benchSpentUsd: 3.25, },),);
            expect(seen.lines[ceilingAt + 1],).toContain('--max-usd 0.5000; the arms run before this one spent --bench-spent-usd 3.2500;',);
            expect(seen.clientBuilds.length,).toBe(1,);
            expect(seen.clientBuilds[0]?.planWasOnDisk,).toBe(true,);
          },
        },),

        it({
          name: 'CAPS THE ARM BEFORE THE CALL THAT WOULD PASS --max-usd, and ends it as a capped arm, not a failure: the '
            + 'first reply costs 0.2050 USD measured, the second call\'s ceiling would take the total past 0.5, so the '
            + 'second is never made, the run returns, a CAPPED line names the trial and the guard\'s three figures, the '
            + 'finished row stays in its file, and the tally, the kept line and the document of the finished row are '
            + 'written',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--max-usd', '0.5',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked,).toStrictEqual([LOW_CARD,],);
            /**
             The line that says where the arm was capped.
             */
            const capped = nonNullishOrThrow(seen.lines.find(function isCapped(line,): boolean {
              return line.startsWith('CAPPED at unit 2 of 2: 1 unit completed; ',);
            },),);
            expect(capped.includes(
              `the editor fidelity probe will not ask ${LOW_CARD}: the spend so far is 0.2050 USD, this call can cost 0.3`,
            ),).toBe(true,);
            expect(capped.endsWith(
              ' USD at most, and together they pass --max-usd 0.5000; the arm ends here, and the rows it finished stay in '
                + 'their files',
            ),).toBe(true,);
            expect(seen.lines,).toContain('TRIALS completed 1 of 2 units',);
            const [runName,] = (await readdir(scratch.path,)).filter(function isRun(name,): boolean {
              return name.startsWith('editor-fidelity-',);
            },);
            const runDir = join(
              scratch.path,
              nonNullishOrThrow(runName,),
            );
            expect(
              existsSync(join(
                runDir,
                'row-0.json',
              ),),
            ).toBe(true,);
            expect(
              existsSync(join(
                runDir,
                'row-1.json',
              ),),
            ).toBe(false,);
            expect(seen.lines.some(function isKept(line,): boolean {
              return line.startsWith('kept 1 editor row at ',);
            },),).toBe(true,);
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD}: clean fix 1 of 1 (exact 1, equivalent 0), defect only 0, collateral 0, missed 0, `
                + 'refused 0, unusable 0',
            );
            expect((JSON.parse(seen.written,) as { readonly rows: readonly unknown[]; }).rows.length,).toBe(1,);
          },
        },),

        it({
          name: 'ENDS AN ARM CAPPED BEFORE ITS FIRST TRIAL as a capped arm with nothing completed: a cap under the first '
            + 'call\'s ceiling asks nothing, says TRIALS completed 0, and returns',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--max-usd', '0.1',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked,).toStrictEqual([],);
            expect(seen.lines.some(function isCapped(line,): boolean {
              return line.startsWith('CAPPED at unit 1 of 2: 0 units completed; ',);
            },),).toBe(true,);
            expect(seen.lines,).toContain('TRIALS completed 0 of 2 units',);
          },
        },),

        it({
          name: 'HANDS THE CLIENT BUILDER THE GUARD\'S ADMIT, so an attempt the client tells it of is held to --max-usd '
            + 'as a call is: with 0.7 allowed, the first call fits and costs 0.2050 USD, the second call\'s ceiling '
            + 'still fits, but its attempt can cost 0.6000 USD, so it is refused as an attempt before the client is '
            + 'asked',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--max-usd', '0.7',],
              attemptUsd: 0.6,
            },);
            expect(seen.asked,).toStrictEqual([LOW_CARD,],);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain(
              'CAPPED at unit 2 of 2: 1 unit completed; '
                + `the editor fidelity probe will not ask ${LOW_CARD}: the spend so far is 0.2050 USD, this attempt can `
                + 'cost 0.6000 USD at most, and together they pass --max-usd 0.7000; the arm ends here, and the rows it '
                + 'finished stay in their files',
            );
          },
        },),

        it({
          name: 'SAYS WHAT THE GUARD COUNTED when the arm ends, finished or stopped: two replies of 0.2050 USD are '
            + 'GUARD total=0.410000 calls=2 atCeiling=0, and the stopped arm of one reply says '
            + 'GUARD total=0.205000 calls=1 atCeiling=0 although the run refuses',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const finished = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            expect(finished.failures,).toStrictEqual([],);
            expect(finished.lines.filter(function isGuardTotal(line,): boolean {
              return line.startsWith('GUARD total=',);
            },),).toStrictEqual(['GUARD total=0.410000 calls=2 atCeiling=0',],);
            const stopped = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--max-usd', '0.5',],
            },);
            expect(stopped.failures,).toStrictEqual([],);
            expect(stopped.lines.filter(function isGuardTotal(line,): boolean {
              return line.startsWith('GUARD total=',);
            },),).toStrictEqual(['GUARD total=0.205000 calls=1 atCeiling=0',],);
          },
        },),

        it({
          name: 'STARTS under the default two dollars for one card, which is the arm the lead runs',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.clientBuilds.length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'a run',
      children: [
        it({
          name: 'BUILDS ITS CLIENT ONCE, FOR THE NAMED CARD ONLY, after the plan is on disk, and asks that card and no '
            + 'other, once for each trial, with no critic, panel, judge or checker in the roster',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.clientBuilds.length,).toBe(1,);
            expect(seen.clientBuilds[0]?.cardIds,).toStrictEqual([LOW_CARD,],);
            expect(seen.clientBuilds[0]?.planWasOnDisk,).toBe(true,);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD,],);
            expect(seen.readFor.length,).toBe(1,);
          },
        },),

        it({
          name: 'BENCHES TWO CARDS EACH IN ITS OWN GATHER, one card at a time and the first card\'s trials before the '
            + 'second\'s, so two efforts of one wire model are never seated together',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [
                '--only',
                SPEC.entryId,
                '--candidates',
                `${LOW_CARD},${HIGH_CARD}`,
                '--candidates-alone',
              ],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.clientBuilds.length,).toBe(1,);
            expect(seen.clientBuilds[0]?.cardIds,).toStrictEqual([LOW_CARD, HIGH_CARD,],);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD, HIGH_CARD, HIGH_CARD,],);
          },
        },),

        it({
          name: 'KEEPS A PLAN, A ROW FOR EVERY CARD AND TRIAL AND THE RUN, and prints the tally of each card, the kept '
            + 'line and a document of the rows and tallies, none of it quoting the corpus',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
            },);
            expect(seen.failures,).toStrictEqual([],);
            const [runName,] = (await readdir(scratch.path,)).filter(function isRun(name,): boolean {
              return name.startsWith('editor-fidelity-',);
            },);
            const runDir = join(
              scratch.path,
              nonNullishOrThrow(runName,),
            );
            const plan = JSON.parse(await readFile(
              join(
                runDir,
                'plan.json',
              ),
              'utf8',
            ),) as { readonly status: string; readonly requestedCards: readonly string[]; readonly plannedRows: number; };
            expect(plan.status,).toBe('planned',);
            expect(plan.requestedCards,).toStrictEqual([LOW_CARD,],);
            expect(plan.plannedRows,).toBe(2,);
            expect(
              existsSync(join(
                runDir,
                'row-0.json',
              ),),
            ).toBe(true,);
            expect(
              existsSync(join(
                runDir,
                'row-1.json',
              ),),
            ).toBe(true,);
            /**
             The row lines, each with how the asks of its trial ended.
             */
            const rowLines = seen.lines.filter(function isRow(line,): boolean {
              return line.startsWith('row ',);
            },);
            expect(rowLines.length,).toBe(2,);
            expect(rowLines.every(function statesStop(line,): boolean {
              return line.endsWith('; stopped ok',);
            },),).toBe(true,);
            expect(rowLines.every(function statesEntry(line,): boolean {
              return line.includes(`; entry ${SPEC.entryId}; stopped `,);
            },),).toBe(true,);
            expect(seen.lines,).toContain('planned: 2 of 2 trials',);
            expect(seen.lines,).toContain('TRIALS completed 2 of 2 units',);
            expect(seen.lines.some(function isCapped(line,): boolean {
              return line.startsWith('CAPPED ',);
            },),).toBe(false,);
            /**
             The first kept row.
             */
            const keptRow = JSON.parse(await readFile(
              join(
                runDir,
                'row-0.json',
              ),
              'utf8',
            ),) as { readonly stops: readonly string[]; };
            expect(keptRow.stops,).toStrictEqual(['ok',],);
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD}: clean fix 2 of 2 (exact 2, equivalent 0), defect only 0, collateral 0, missed 0, `
                + 'refused 0, unusable 0',
            );
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD} insertion: clean fix 1 of 1 (exact 1, equivalent 0), defect only 0, collateral 0, `
                + 'missed 0, refused 0, unusable 0',
            );
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD} alteration: clean fix 1 of 1 (exact 1, equivalent 0), defect only 0, collateral 0, `
                + 'missed 0, refused 0, unusable 0',
            );
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD} defect only, not in the clean fix count: 0 of 2 (insertion 0, alteration 0)`,
            );
            expect(seen.lines.some(function isKept(line,): boolean {
              return line.startsWith('kept 2 editor rows at ',);
            },),).toBe(true,);
            const printed = JSON.parse(seen.written,) as {
              readonly rows: readonly { readonly cardId: string; }[];
              readonly tallies: readonly { readonly cardId: string; readonly fixed: number; }[];
            };
            expect(printed.rows.length,).toBe(2,);
            expect(printed.tallies.map(function pairOf(tally,): readonly [string, number] {
              return [tally.cardId, tally.fixed,];
            },),).toEqual([[LOW_CARD, 2,],],);
            for (const trial of TRIALS)
              expect(seen.written.includes(trial.damagedText,),).toBe(false,);
          },
        },),

        it({
          name: 'COUNTS A CARD THAT ANSWERS NOTHING READABLE as unusable on every trial, and still ends the run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: ONE_CARD,
              silent: true,
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD}: clean fix 0 of 2 (exact 0, equivalent 0), defect only 0, collateral 0, missed 0, `
                + 'refused 0, unusable 2',
            );
            /**
             The row lines, whose asks the guard of the reply rejected every time.
             */
            const rowLines = seen.lines.filter(function isRow(line,): boolean {
              return line.startsWith('row ',);
            },);
            expect(rowLines.length,).toBe(2,);
            expect(rowLines.every(function statesRejection(line,): boolean {
              return line.includes('; stopped rejected',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'WALKS ONLY THE FIRST TRIALS when --trials is written, says how many it planned and completed, and asks '
            + 'once for each, so a later arm walks the trials the first arm completed',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--trials', '1',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked,).toStrictEqual([LOW_CARD,],);
            expect(seen.lines,).toContain('pool: 2 trials over 1 entry (insertion 1, alteration 1)',);
            expect(seen.lines,).toContain('planned: 1 of 2 trials',);
            expect(seen.lines,).toContain('TRIALS completed 1 of 1 unit',);
            expect(seen.lines.filter(function isRow(line,): boolean {
              return line.startsWith('row ',);
            },).length,).toBe(1,);
          },
        },),

        it({
          name: 'PLANS THE WHOLE POOL when --trials names more trials than it holds, since the pool is all there is to walk',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--trials', '50',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain('planned: 2 of 2 trials',);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD,],);
          },
        },),

        it({
          name: 'WALKS THE POOL AGAIN when --passes names more than one: the planned line says the passes, every later '
            + 'pass asks every trial once more, its row lines name the pass, and the plan counts the passes as the '
            + 'matrix',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--passes', '2',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD, LOW_CARD, LOW_CARD,],);
            expect(seen.lines,).toContain('planned: 4 of 4 asks (2 passes over the 2 trials of the pool)',);
            expect(seen.lines,).toContain('TRIALS completed 4 of 4 units',);
            /**
             The row lines, the first pass's two and the second pass's two.
             */
            const rowLines = seen.lines.filter(function isRow(line,): boolean {
              return line.startsWith('row ',);
            },);
            expect(rowLines.length,).toBe(4,);
            expect(rowLines.map(function stopsAfterEntry(line,): string {
              return line.slice(line.indexOf('; entry ',),);
            },),).toStrictEqual([
              `; entry ${SPEC.entryId}; stopped ok`,
              `; entry ${SPEC.entryId}; stopped ok`,
              `; entry ${SPEC.entryId}; pass 2; stopped ok`,
              `; entry ${SPEC.entryId}; pass 2; stopped ok`,
            ],);
            expect(seen.lines,).toContain(
              `EDITOR ${LOW_CARD}: clean fix 4 of 4 (exact 4, equivalent 0), defect only 0, collateral 0, missed 0, `
                + 'refused 0, unusable 0',
            );
            const [runName,] = (await readdir(scratch.path,)).filter(function isRun(name,): boolean {
              return name.startsWith('editor-fidelity-',);
            },);
            const plan = JSON.parse(await readFile(
              join(
                scratch.path,
                nonNullishOrThrow(runName,),
                'plan.json',
              ),
              'utf8',
            ),) as {
              readonly plannedRows: number;
              readonly fullMatrixRows: number;
              readonly completeRequestedMatrix: boolean;
            };
            expect(plan.plannedRows,).toBe(4,);
            expect(plan.fullMatrixRows,).toBe(4,);
            expect(plan.completeRequestedMatrix,).toBe(true,);
          },
        },),

        it({
          name: 'ASKS THE MODEL AFRESH IN EVERY PASS THROUGH THE PROMPT REUSE LAYER THE RUN CLIENT HAS: two passes over '
            + 'two trials are four calls that reach the model, and not two calls and two replays of the first pass\'s '
            + 'replies, so a repeat measurement compares two answers and not an answer with itself',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--passes', '2',],
              reusing: true,
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD, LOW_CARD, LOW_CARD,],);
            expect(seen.lines,).toContain('TRIALS completed 4 of 4 units',);
            expect(seen.lines,).toContain('GUARD total=0.820000 calls=4 atCeiling=0',);
          },
        },),

        it({
          name: 'REFUSES TO SCORE A ROW THAT MADE NO CALL, and fails loud: a reply from the reuse layer reaches the arm in '
            + 'pass 2 although the pass scope should have prevented it, the run throws and does not count a trial, writes '
            + 'no row for it, books no SPEND line at the ceiling and prints no TRIALS line, while the rows that made a call '
            + 'stay and the guard line says what it counted',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--passes', '2',],
              reusing: true,
              unscoped: true,
            },);
            expect(seen.lines,).toContain('GUARD total=0.410000 calls=3 atCeiling=0',);
            expect(seen.lines.some(function isTrials(line,): boolean {
              return line.startsWith('TRIALS completed ',);
            },),).toBe(false,);
            expect(seen.failures.length,).toBe(1,);
            expect(seen.failures[0],).toBeInstanceOf(RepliedWithoutCallError,);
            expect(seen.asked,).toStrictEqual([LOW_CARD, LOW_CARD,],);
            expect(seen.lines.filter(function isRow(line,): boolean {
              return line.startsWith('row ',);
            },).length,).toBe(2,);
            const [runName,] = (await readdir(scratch.path,)).filter(function isRun(name,): boolean {
              return name.startsWith('editor-fidelity-',);
            },);
            const rowFiles = (await readdir(join(
              scratch.path,
              nonNullishOrThrow(runName,),
            ),)).filter(function isRow(name,): boolean {
              return name.startsWith('row-',);
            },);
            expect(rowFiles.length,).toBe(2,);
          },
        },),

        it({
          name: 'WALKS ONLY THE LEADING ASKS OF SEVERAL PASSES when --trials is written beside --passes, the first pass '
            + 'whole and then the start of the second, and warns that the matrix is partial',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--passes', '2', '--trials', '3',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.asked.length,).toBe(3,);
            expect(seen.lines,).toContain('planned: 3 of 4 asks (2 passes over the 2 trials of the pool)',);
            expect(seen.lines,).toContain('TRIALS completed 3 of 3 units',);
            expect(seen.lines,).toContain('partial reviewed matrix: 3 of 4 rows; not complete admission evidence',);
            expect(seen.lines.filter(function isSecondPass(line,): boolean {
              return line.startsWith('row ',) && line.includes('; pass 2;',);
            },).length,).toBe(1,);
          },
        },),

        it({
          name: 'PRICES SEVERAL PASSES under a cap of zero: a ceiling for every ask of the passes, and a preflight line '
            + 'that says the ceiling is priced over them',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--passes', '2', '--cap', '0',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines.some(function isCeiling(line,): boolean {
              return line.startsWith(`ceiling ${LOW_CARD}: 4 units, one ask each, at most `,);
            },),).toBe(true,);
            expect(seen.lines,).toContain('planned: 4 of 4 asks (2 passes over the 2 trials of the pool)',);
            expect(seen.lines.at(-1,),).toBe(
              `preflight only: 1 pinned reference specification selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every ask of the 2 passes over the pool`,
            );
            expect(seen.clientBuilds,).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS WHEN THE CAP LEAVES TRIALS OUT, in the judge probe\'s wording, and runs only the first',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-run-', },);
            const seen = await probeIn({
              runsDir: scratch.path,
              typed: [...ONE_CARD, '--cap', '1',],
            },);
            expect(seen.failures,).toStrictEqual([],);
            expect(seen.lines,).toContain('partial reviewed matrix: 1 of 2 rows; not complete admission evidence',);
            expect(seen.asked,).toStrictEqual([LOW_CARD,],);
          },
        },),
      ],
    },),
  ],
},);
