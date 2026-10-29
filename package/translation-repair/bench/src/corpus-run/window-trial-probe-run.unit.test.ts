/**
 Tests for the window trial's walk end to end: a scripted corpus, a scripted
 client and a throwaway runs directory, so the walk buys arms, counts what it
 bought and refused, makes the one check only a live run can, and reports the
 ledger, with no call reaching a network.

 What the walk tells the log is read off a recording logger. The stages the
 real buyer runs write their own lines there too, each starting with the
 stage's name in brackets; the cases that run the real buyer keep the lines
 that do not.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RUN_ROSTER, } from '@monochromatic-dev/translation-repair-run';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  appendTrialRow,
  armOrderFor,
  type PickOutcome,
  protocolDigest,
  readTrialLedger,
  runPick,
  runWindowTrial,
  WindowEvidenceError,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  CAT_PIN,
  CLEAN_SOURCE,
  CLEAN_TARGET,
  DEATH_LINE,
  FLAGGED_IDENTITY,
  FLAGGED_SOURCE,
  FLAGGED_TARGET,
} from './window-trial-probe-draw.test-fixture.ts';
import { armRow, } from './window-trial-probe-rows.test-fixture.ts';
import { scriptedTrialClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/window-trial-scripted-client.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Head the protocol digest is of, in every case.
 */
const HEAD = 'a'.repeat(40,);

/**
 Models that translate and judge, the first four of the run's own roster so
 the votes carry the weight a real roster's would.
 */
const ROSTER = RUN_ROSTER.slice(
  0,
  4,
);

/**
 Digest every case buys under.
 */
const PROTOCOL = protocolDigest({
  headSha: HEAD,
  pin: CAT_PIN,
},);

/**
 Opening line of a run that holds no arm yet.
 */
const OPENING_FRESH = `protocol ${ PROTOCOL.slice(
  0,
  12,
) }; 0 arms already bought`;

/**
 Flagged entries the scripted corpus holds besides the entry that carries one
 side and the one the screen leaves alone.
 */
const FLAGGED_IDS: readonly string[] = [
  'Mittens',
  'Mittens2',
  'Mittens3',
];

/**
 Pages of the scripted corpus, by path: the flagged entries carry both sides,
 one entry carries only its original, and one the screen leaves alone carries
 both.
 */
const PAGES: ReadonlyMap<string, string> = new Map([
  ...FLAGGED_IDS.flatMap(function pagesOf(id,): readonly (readonly [string, string])[] {
    return [
      [
        `people/${id}/page.md`,
        FLAGGED_SOURCE,
      ],
      [
        `people/${id}/page.en.md`,
        FLAGGED_TARGET,
      ],
    ];
  },),
  [
    'people/Whiskers/page.md',
    CLEAN_SOURCE,
  ],
  [
    'people/Tabby/page.md',
    CLEAN_SOURCE,
  ],
  [
    'people/Tabby/page.en.md',
    CLEAN_TARGET,
  ],
  ...[...FLAGGED_IDS,
    'Tabby',].map(function infoOf(id,): readonly [string, string] {
      return [
        `people/${id}/info.yml`,
        'info:\n  born: \'2019-05-02\'\n',
      ];
    },),
  [
    'people/Mimi/page.md',
    FLAGGED_SOURCE,
  ],
  [
    'people/Mimi/page.en.md',
    FLAGGED_TARGET,
  ],
  [
    'people/Mimi/info.yml',
    'info:\n  died: 2025-02-26\n',
  ],
]);

/**
 What the ledger report says of a ledger holding the two slices of `Mittens`
 with every arm kept.
 */
const KEPT_REPORT: readonly string[] = [
  'untranslated: window moved replacement by 0.00 over 1 entry; narrow-a 0/1 narrow-b 0/1 wide 0/1; wide moved 0 down '
    + 'and 0 up, against a band of 0 down and 0 up; 0 incomplete, 0 dropped for a short panel, 0 slices left out for an '
    + 'arm the ledger holds twice',
  'control-unflagged: window moved replacement by 0.00 over 1 entry; narrow-a 0/1 narrow-b 0/1 wide 0/1; wide moved 0 '
    + 'down and 0 up, against a band of 0 down and 0 up; 0 incomplete, 0 dropped for a short panel, 0 slices left out '
    + 'for an arm the ledger holds twice',
];

/**
 Lines of a log that the walk itself wrote, without the stages' own.

 @param lines - every line the logger recorded

 @returns Lines that do not start with a stage's bracketed name

 @example
 ```ts
 const own = walkLines({ lines, },);
 ```
 */
function walkLines({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isOwn(line,): boolean {
    return !line.startsWith('[',);
  },);
}

/**
 What the walk says of one slice of `Mittens` whose arms were all kept, arms
 in the order the slice buys them.

 @param sliceIndex - slice the line is about

 @returns Arms, each kept, as the line names them

 @example
 ```ts
 const arms = armsOf({ sliceIndex: 1, },);
 ```
 */
function armsOf({ sliceIndex, }: { readonly sliceIndex: number; },): string {
  return armOrderFor({
    protocol: PROTOCOL,
    entryId: 'Mittens',
    sliceIndex,
  },)
    .map(function said(arm,): string {
      return `${arm}=kept`;
    },)
    .join(' ',);
}

/**
 Reads a page of the scripted corpus, refusing as git does when it holds none.

 @param relPath - path asked for

 @returns Page

 @example
 ```ts
 const text = await readScripted({ relPath: 'people/Mittens/page.md', },);
 ```
 */
async function readScripted({ relPath, }: { readonly relPath: string; },): Promise<string> {
  /**
   Text of the page, when the corpus holds it.
   */
  const held = PAGES.get(relPath,);
  if (held === undefined)
    throw new CorpusReadError({
      detail: `${relPath} at ${CAT_PIN.commitSha}`,
      cause: { stderr: `fatal: path '${relPath}' does not exist in '${CAT_PIN.commitSha}'`, },
      commit: 'held',
    },);
  return held;
}

/**
 Lists the people of the scripted corpus.

 @returns Their ids, in listing order

 @example
 ```ts
 const people = await listScripted();
 ```
 */
async function listScripted(): Promise<readonly string[]> {
  return [
    'Mittens',
    'Whiskers',
    'Tabby',
  ];
}

/**
 Lists a given set of people.

 @param ids - people the corpus lists

 @returns Lister answering them

 @example
 ```ts
 const listPeople = listing({ ids: ['Mittens',], },);
 ```
 */
function listing({ ids, }: { readonly ids: readonly string[]; },): () => Promise<readonly string[]> {
  return async function listIds(): Promise<readonly string[]> {
    return ids;
  };
}

/**
 Buyer that answers each slice from a script, whatever it is asked.

 @param outcomes - what each successive slice yields

 @param clients - array each call's client lands in

 @returns Buyer standing in for the real one

 @example
 ```ts
 const pickSlice = scriptedPick({ outcomes: [{ kind: 'refused', },], clients: [], },);
 ```
 */
function scriptedPick(
  {
    outcomes,
    clients,
  }: {
    readonly outcomes: readonly PickOutcome[];
    readonly clients: ModelClient[];
  },
): typeof runPick {
  return async function pickFromScript(
    { client, }: { readonly client: ModelClient; },
  ): Promise<PickOutcome> {
    /**
     Outcome this call answers with.
     */
    const outcome = outcomes[clients.length];
    clients.push(client,);
    if (outcome === undefined)
      throw new Error('the script holds no outcome for this slice',);
    return outcome;
  };
}

await describe({
  name: runWindowTrial.name,
  concurrency: 1,
  children: [
    it({
      name: 'BUYS the flagged slice and its control arm by arm through the real buyer, names what it bought, and reports '
        + 'the ledger, stepping past the entry that carries one side and the entry the screen leaves alone',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-', },);
        const rig = scriptedTrialClient({
          narrowBallot: 0,
          wideBallot: 0,
        },);
        const { logger, lines, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return rig.client;
          },
          pin: CAT_PIN,
          listPeople: listScripted,
          readPage: readScripted,
          pickSlice: runPick,
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(walkLines({ lines, },),).toEqual([
          OPENING_FRESH,
          'the window reached every judge of the first wide arm',
          `Mittens/1 (untranslated): ${ armsOf({ sliceIndex: 1, },) }`,
          `Mittens/2 (control-unflagged): ${ armsOf({ sliceIndex: 2, },) }`,
          'Whiskers: skipped, CorpusReadError: corpus read failed for people/Whiskers/page.en.md at '
            + `${CAT_PIN.commitSha} (missing-object); the commit has no such path: check the path, or pin a `
            + 'commit that has it.',
          'bought 2 slices this run; 0 refused',
          'ledger read for this report: 6 rows under every protocol; 0 whole lines left out as no trial row of this build; '
            + 'last line whole',
          ...KEPT_REPORT,
        ],);
        expect((await readTrialLedger({
          path: join(
            scratch.path,
            'window-trial',
            'arms.jsonl',
          ),
        },)).length,).toBe(6,);
      },
    },),
    it({
      name: 'SKIPS every slice the ledger already holds, buying nothing and counting none, and reports the ledger it holds',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-held-', },);
        /**
         Where the ledger lives under the runs directory.
         */
        const path = join(
          scratch.path,
          'window-trial',
          'arms.jsonl',
        );
        for (const [sliceIndex, sliceClass,] of [
          [
            1,
            'untranslated',
          ],
          [
            2,
            'control-unflagged',
          ],
        ] as const) {
          for (const arm of [
            'narrow-a',
            'narrow-b',
            'wide',
          ] as const) {
            /* oxlint-disable-next-line no-await-in-loop -- rows are appended one at a time, as a run appends them */
            await appendTrialRow({
              path,
              row: armRow({
                arm,
                shipped: false,
                sliceIndex,
                protocol: PROTOCOL,
                sliceClass,
                entryId: 'Mittens',
              },),
            },);
          }
        }
        const rig = scriptedTrialClient({
          narrowBallot: 0,
          wideBallot: 0,
        },);
        const { logger, lines, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return rig.client;
          },
          pin: CAT_PIN,
          listPeople: listing({ ids: ['Mittens',], },),
          readPage: readScripted,
          pickSlice: runPick,
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(walkLines({ lines, },),).toEqual([
          `protocol ${ PROTOCOL.slice(
            0,
            12,
          ) }; 6 arms already bought`,
          'bought 0 slices this run; 0 refused',
          'ledger read for this report: 6 rows under every protocol; 0 whole lines left out as no trial row of this build; '
            + 'last line whole',
          ...KEPT_REPORT,
        ],);
        expect(rig.served
          .count,).toBe(0,);
      },
    },),
    it({
      name: 'HANDS the first slices the recording client and every slice after the check passed the client itself',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-swap-', },);
        const rig = scriptedTrialClient({
          narrowBallot: 0,
          wideBallot: 0,
        },);
        /**
         Client each slice was bought through.
         */
        const clients: ModelClient[] = [];
        const { logger, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return rig.client;
          },
          pin: CAT_PIN,
          listPeople: listing({ ids: ['Mittens',], },),
          readPage: readScripted,
          pickSlice: async function recordingPick(input,): Promise<PickOutcome> {
            clients.push(input.client,);
            return runPick(input,);
          },
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(clients.length,).toBe(2,);
        expect(clients[0] === rig.client,).toBe(false,);
        expect(clients[1] === rig.client,).toBe(true,);
      },
    },),
    it({
      name: 'HANDS the buyer of each slice the page facts of its entry\'s preparation, so an entry that records a death is '
        + 'bought on sheets that tell the judges so',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-facts-', },);
        const rig = scriptedTrialClient({
          narrowBallot: 0,
          wideBallot: 0,
        },);
        /**
         Page facts each slice was bought with.
         */
        const facts: unknown[] = [];
        const { logger, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return rig.client;
          },
          pin: CAT_PIN,
          listPeople: listing({ ids: ['Mimi',], },),
          readPage: readScripted,
          pickSlice: async function recordingFacts({ sheetFacts, },): Promise<PickOutcome> {
            facts.push(sheetFacts,);
            return {
              kind: 'bought',
              rows: [],
            };
          },
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         Facts of the entry that records a death.
         */
        const recorded = {
          identityContext: `${DEATH_LINE}\n${FLAGGED_IDENTITY}`,
          lifeEnded: true,
        };
        expect(facts,).toEqual([recorded, recorded,],);
      },
    },),
    it({
      name: 'STOPS with a stated refusal at the fifth refusal in a row, before the rest of the draw, saying nothing of '
        + 'the slices it never reached',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-refused-', },);
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the walk rejected with.
         */
        const refusal = await rejectionOf({
          promise: runWindowTrial({
            runsDir: scratch.path,
            headSha: HEAD,
            makeClient: function makeClient(): ModelClient {
              return scriptedTrialClient({
                narrowBallot: 0,
                wideBallot: 0,
              },).client;
            },
            pin: CAT_PIN,
            listPeople: listing({ ids: FLAGGED_IDS, },),
            readPage: readScripted,
            pickSlice: scriptedPick({
              outcomes: Array.from(
                { length: 6, },
                function refused(): PickOutcome {
                  return { kind: 'refused', };
                },
              ),
              clients: [],
            },),
            roster: ROSTER,
            perCallTimeoutMs: HANG_STOP_MS,
            l: logger,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: 5 slices refused in a row, which is a fault in the run rather than in the slices; '
            + 'stopping before the rest of the draw is spent producing slates nobody judges',
        );
        expect(walkLines({ lines, },),).toEqual([OPENING_FRESH,],);
      },
    },),
    it({
      name: 'COUNTS four refusals and the slice that bought after them, and walks past a slice the ledger held',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-streak-', },);
        const { logger, lines, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return scriptedTrialClient({
              narrowBallot: 0,
              wideBallot: 0,
            },).client;
          },
          pin: CAT_PIN,
          listPeople: listing({ ids: FLAGGED_IDS, },),
          readPage: readScripted,
          pickSlice: scriptedPick({
            outcomes: [
              { kind: 'refused', },
              { kind: 'refused', },
              { kind: 'refused', },
              { kind: 'refused', },
              {
                kind: 'bought',
                rows: [armRow({ arm: 'narrow-a', shipped: true, },),],
              },
              {
                kind: 'bought',
                rows: [],
              },
            ],
            clients: [],
          },),
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(walkLines({ lines, },),).toEqual([
          OPENING_FRESH,
          'Mittens3/1 (untranslated): narrow-a=replaced',
          'bought 1 slice this run; 4 refused',
          'ledger read for this report: 0 rows under every protocol; 0 whole lines left out as no trial row of this build; '
            + 'last line whole',
        ],);
      },
    },),
    it({
      name: 'LEAVES the streak where it was for a slice the ledger held, so a refusal after it is the fifth',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-held-streak-', },);
        const { logger, } = capturingLoggerPair();

        /**
         What the walk rejected with.
         */
        const refusal = await rejectionOf({
          promise: runWindowTrial({
            runsDir: scratch.path,
            headSha: HEAD,
            makeClient: function makeClient(): ModelClient {
              return scriptedTrialClient({
                narrowBallot: 0,
                wideBallot: 0,
              },).client;
            },
            pin: CAT_PIN,
            listPeople: listing({ ids: FLAGGED_IDS, },),
            readPage: readScripted,
            pickSlice: scriptedPick({
              outcomes: [
                { kind: 'refused', },
                { kind: 'refused', },
                { kind: 'refused', },
                { kind: 'refused', },
                {
                  kind: 'bought',
                  rows: [],
                },
                { kind: 'refused', },
              ],
              clients: [],
            },),
            roster: ROSTER,
            perCallTimeoutMs: HANG_STOP_MS,
            l: logger,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
      },
    },),
    it({
      name: 'REFUSES a first wide arm no judge was shown the window in, naming the count found and the count expected',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-unseen-', },);
        const { logger, } = capturingLoggerPair();

        /**
         What the walk rejected with.
         */
        const refusal = await rejectionOf({
          promise: runWindowTrial({
            runsDir: scratch.path,
            headSha: HEAD,
            makeClient: function makeClient(): ModelClient {
              return scriptedTrialClient({
                narrowBallot: 0,
                wideBallot: 0,
              },).client;
            },
            pin: CAT_PIN,
            listPeople: listing({ ids: ['Mittens',], },),
            readPage: readScripted,
            pickSlice: scriptedPick({
              outcomes: [
                {
                  kind: 'bought',
                  rows: [armRow({ arm: 'wide', shipped: false, },),],
                },
              ],
              clients: [],
            },),
            roster: ROSTER,
            perCallTimeoutMs: HANG_STOP_MS,
            l: logger,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        },);

        expect(refusal,).toBeInstanceOf(WindowEvidenceError,);
        expect(String(refusal,),).toBe(
          'WindowEvidenceError: 0 of the judge sheets carried SURROUNDING ORIGINAL where 4 should have. The wide arm is '
            + 'the only thing this trial varies, so if it did not carry the window then all three arms saw the same evidence '
            + 'and every row bought after this point would report a false null. Nothing has been spent beyond the first slice.',
        );
      },
    },),
    it({
      name: 'BUILDS NO CLIENT and lists no corpus when the ledger will not read, naming its line and none of its text',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-torn-', },);
        /**
         Where the ledger lives under the runs directory.
         */
        const path = join(
          scratch.path,
          'window-trial',
          'arms.jsonl',
        );
        await mkdir(dirname(path,),);
        await writeFile(
          path,
          'not json\n',
        );
        /**
         What the walk touched before it stopped.
         */
        const touched: string[] = [];
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the walk rejected with.
         */
        const refusal = await rejectionOf({
          promise: runWindowTrial({
            runsDir: scratch.path,
            headSha: HEAD,
            makeClient: function makeClient(): ModelClient {
              touched.push('client',);
              return scriptedTrialClient({
                narrowBallot: 0,
                wideBallot: 0,
              },).client;
            },
            pin: CAT_PIN,
            listPeople: async function listTouched(): Promise<readonly string[]> {
              touched.push('corpus',);
              return [];
            },
            readPage: readScripted,
            pickSlice: runPick,
            roster: ROSTER,
            perCallTimeoutMs: HANG_STOP_MS,
            l: logger,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        },);

        expect(String(refusal,),).toBe(
          `TrialLedgerLineError: window trial ledger ${path}: line 1 ends in a newline and does not parse as JSON `
            + '(SyntaxError). A kill mid-append leaves only an unterminated last line, which the next append removes, '
            + 'so this line was written by something else: a build that appended a row onto such a fragment, or two '
            + 'runs appending at once. What it bought cannot be read from here. Remove that line by hand to buy its '
            + 'arms again, or move the ledger aside to start a fresh one.',
        );
        expect(touched,).toEqual([]);
        expect(lines,).toEqual([]);
      },
    },),
    it({
      name: 'ASKS for the client after the opening line and before the corpus is listed, and passes its refusal on',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-keyless-', },);
        /**
         What the walk touched before it stopped.
         */
        const touched: string[] = [];
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the walk rejected with.
         */
        const refusal = await rejectionOf({
          promise: runWindowTrial({
            runsDir: scratch.path,
            headSha: HEAD,
            makeClient: function makeClient(): ModelClient {
              throw new StatedRefusalError({ says: 'no key is set', },);
            },
            pin: CAT_PIN,
            listPeople: async function listTouched(): Promise<readonly string[]> {
              touched.push('corpus',);
              return [];
            },
            readPage: readScripted,
            pickSlice: runPick,
            roster: ROSTER,
            perCallTimeoutMs: HANG_STOP_MS,
            l: logger,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        },);

        expect(String(refusal,),).toBe('StatedRefusalError: no key is set',);
        expect(touched,).toEqual([],);
        expect(lines,).toEqual([OPENING_FRESH,],);
      },
    },),
    it({
      name: 'REPORTS an empty corpus as nothing bought and no row, the one and the other in the plural',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-run-empty-', },);
        const { logger, lines, } = capturingLoggerPair();

        await runWindowTrial({
          runsDir: scratch.path,
          headSha: HEAD,
          makeClient: function makeClient(): ModelClient {
            return scriptedTrialClient({
              narrowBallot: 0,
              wideBallot: 0,
            },).client;
          },
          pin: CAT_PIN,
          listPeople: listing({ ids: [], },),
          readPage: readScripted,
          pickSlice: runPick,
          roster: ROSTER,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(lines,).toEqual([
          OPENING_FRESH,
          'bought 0 slices this run; 0 refused',
          'ledger read for this report: 0 rows under every protocol; 0 whole lines left out as no trial row of this build; '
            + 'last line whole',
        ],);
      },
    },),
  ],
},);
