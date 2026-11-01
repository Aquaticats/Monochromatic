/**
 Tests for the Bedrock ledger, the meter this provider does not have: the
 durable file every priced call appends to, the credit line it is read
 against, and the environment that names both.

 Every case writes under a disposable directory and removes it after.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BEDROCK_CREDIT_USD,
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
  BedrockCreditOverrideError,
  bedrockCreditUsdFrom,
  BEDROCK_DRY_MARGIN_USD,
  bedrockIsDry,
  bedrockLedgerPathFrom,
  BedrockLedgerShapeError,
  bedrockMeterLevel,
  createBedrockLedger,
  defaultBedrockLedgerPath,
} from '../dist/final/node/index.mjs';
import { SEAT_BEDROCK_ONLY_TEXT, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 One priced call, cat-themed.

 @param usd - what it cost

 @returns Entry to note

 @example
 ```ts
 await ledger.note(callCosting({ usd: 0.5, },),);
 ```
 */
function callCosting({ usd, }: { readonly usd: number; },) {
  return {
    at: '2026-09-07T20:00:00.000Z',
    model: SEAT_BEDROCK_ONLY_TEXT,
    usd,
    promptTokens: 90,
    completionTokens: 10,
  };
}

/**
 Runs one case inside a disposable directory, removing it after.

 @param fn - case body, given the directory

 @example
 ```ts
 await inScratch(async function body(dir,) { ... },);
 ```
 */
async function inScratch(fn: (dir: string,) => Promise<void>,): Promise<void> {
  /**
   Directory this case owns, removed when this function returns.
   */
  await using scratch = await scratchDir({ prefix: 'bedrock-ledger-', },);
  await fn(scratch.path,);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createBedrockLedger.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A FILE THAT DOES NOT EXIST YET AS NOTHING SPENT, since the first run has no history',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger over a file nobody has written.
               */
              const ledger = createBedrockLedger({
                path: join(
                  dir,
                  'nested',
                  'bedrock-spend.jsonl',
                ),
                creditUsd: 200,
              },);
              expect(await ledger.read(),).toEqual({
                creditUsd: 200,
                spentUsd: 0,
                reckonedUsd: 0,
                remainingUsd: 200,
                calls: 0,
              },);
            },);
          },
        },),

        it({
          name: 'APPENDS ONE LINE PER CALL, creating the directory, and SUMS THEM on the next read',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger over a file in a directory that does not exist yet.
               */
              const ledger = createBedrockLedger({
                path: join(
                  dir,
                  'state',
                  'bedrock-spend.jsonl',
                ),
                creditUsd: 1,
              },);
              await ledger.note(callCosting({ usd: 0.25, },),);
              await ledger.note(callCosting({ usd: 0.5, },),);
              expect(await ledger.read(),).toEqual({
                creditUsd: 1,
                spentUsd: 0.75,
                reckonedUsd: 0,
                remainingUsd: 0.25,
                calls: 2,
              },);

              /**
               The file as written: one JSON object per line.
               */
              const lines = (await readFile(
                ledger.path,
                'utf8',
              ))
                .trim()
                .split('\n',);
              expect(lines.length,).toBe(2,);
              expect(JSON.parse(lines[0] ?? '',),).toEqual(callCosting({ usd: 0.25, },),);
            },);
          },
        },),

        it({
          name: 'GOES DRY WITH THE MARGIN STILL LEFT (ledger P1): the meter is read once a freshness window, and '
            + 'calls started inside one land after it, so a balance at the margin is already spent',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger one call above the margin.
               */
              const ledger = createBedrockLedger({
                path: join(
                  dir,
                  'bedrock-spend.jsonl',
                ),
                creditUsd: BEDROCK_DRY_MARGIN_USD + 0.25,
              },);
              /**
               Reading before the call.
               */
              const before = bedrockIsDry({ credits: await ledger.read(), },);
              await ledger.note(callCosting({ usd: 0.25, },),);
              expect({
                before,
                after: bedrockIsDry({ credits: await ledger.read(), },),
              },).toEqual({
                before: false,
                after: true,
              },);
            },);
          },
        },),

        it({
          name: 'READS BELOW ZERO rather than clamped, so the meter says how far past the line the calls in '
            + 'flight went',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger with a credit two calls exceed.
               */
              const ledger = createBedrockLedger({
                path: join(
                  dir,
                  'bedrock-spend.jsonl',
                ),
                creditUsd: 0.3,
              },);
              await ledger.note(callCosting({ usd: 0.25, },),);
              await ledger.note(callCosting({ usd: 0.25, },),);

              /**
               Reading past the line.
               */
              const credits = await ledger.read();
              expect(bedrockIsDry({ credits, },),).toBe(true,);
              expect(credits.remainingUsd,).toBeCloseTo(
                -0.2,
                9,
              );
              expect(bedrockMeterLevel({ credits, },),).toEqual(['bedrockUsd=-0.20', 'bedrockReckonedUsd=0.00',],);
            },);
          },
        },),

        it({
          name: 'REFUSES A LINE THAT WILL NOT READ, naming the ledger file and the line\'s number, rather than summing '
            + 'the file as if the money were unspent, where the refusal named the line and not the file a variable '
            + 'can move',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger file with a torn second line.
               */
              const path = join(
                dir,
                'bedrock-spend.jsonl',
              );
              await writeFile(
                path,
                `${JSON.stringify(callCosting({ usd: 0.1, },),)}\n{"at":"2026-09-07T20:00:00.000Z","model":"goo\n`,
                'utf8',
              );

              /**
               Ledger over the torn file.
               */
              const ledger = createBedrockLedger({
                path,
                creditUsd: 200,
              },);

              /**
               What the read threw.
               */
              const thrown = await ledger
                .read()
                .then(
                function unexpected(): unknown {
                  return undefined;
                },
                function caught(error: unknown,): unknown {
                  return error;
                },
                );
              expect(thrown,).toBeInstanceOf(BedrockLedgerShapeError,);
              expect(String(thrown,),).toBe(
                `BedrockLedgerShapeError: Bedrock spend ledger ${path} line 2 violated expectations: not valid JSON `
                  + '(SyntaxError)',
              );
            },);
          },
        },),

        it({
          name: 'REFUSES A LINE THAT IS JSON BUT NOT AN OBJECT, since a bare number or list is not a call',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger file whose one line is a bare number.
               */
              const path = join(
                dir,
                'bedrock-spend.jsonl',
              );
              await writeFile(
                path,
                '42\n',
                'utf8',
              );

              /**
               What the read threw.
               */
              const thrown = await createBedrockLedger({
                path,
                creditUsd: 200,
              },)
                .read()
                .then(
                function unexpected(): unknown {
                  return undefined;
                },
                function caught(error: unknown,): unknown {
                  return error;
                },
                );
              expect(thrown,).toBeInstanceOf(BedrockLedgerShapeError,);
              expect((thrown as Error).message,).toContain('not a JSON object',);
            },);
          },
        },),

        it({
          name: 'REFUSES A LINE WHOSE COST IS NOT A NON-NEGATIVE NUMBER, since a negative or absent cost '
            + 'would refund the credit',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger file whose one line carries a negative cost.
               */
              const path = join(
                dir,
                'bedrock-spend.jsonl',
              );
              await writeFile(
                path,
                `${JSON.stringify(callCosting({ usd: -1, },),)}\n`,
                'utf8',
              );

              /**
               What the read threw.
               */
              const thrown = await createBedrockLedger({
                path,
                creditUsd: 200,
              },)
                .read()
                .then(
                function unexpected(): unknown {
                  return undefined;
                },
                function caught(error: unknown,): unknown {
                  return error;
                },
                );
              expect(thrown,).toBeInstanceOf(BedrockLedgerShapeError,);
              expect((thrown as Error).message,).toContain('usd is not a non-negative number',);
            },);
          },
        },),

        it({
          name: 'REFUSES A LINE WHOSE TIME, MODEL OR TOKEN COUNTS ARE NOT WHAT A CALL WRITES, naming the line and '
            + 'the field, and SUMS the same call written whole',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Each broken line, beside the refusal its read gives.
               */
              const broken = [
                {
                  line: {
                    ...callCosting({ usd: 0.1, },),
                    at: 1_788_811_674,
                  },
                  detail: 'at is not a string',
                },
                {
                  line: {
                    ...callCosting({ usd: 0.1, },),
                    model: 7,
                  },
                  detail: 'model is not a string',
                },
                {
                  line: {
                    ...callCosting({ usd: 0.1, },),
                    completionTokens: '10',
                  },
                  detail: 'token counts are not numbers',
                },
              ];
              /**
               Ledger file holding only one broken line, by that line's place in the list.

               @param index - place of the line in the list

               @returns Path of its ledger file
               */
              function ledgerPathOf(index: number,): string {
                return join(
                  dir,
                  `bedrock-spend-${String(index,)}.jsonl`,
                );
              }
              /**
               Refusal each broken line's read gives, by its message.
               */
              const refusals = await Promise.all(broken.map(async function readBroken(
                { line, },
                index,
              ): Promise<string> {
                /**
                 Ledger file holding only this line.
                 */
                const path = ledgerPathOf(index,);
                await writeFile(
                  path,
                  `${JSON.stringify(line,)}\n`,
                  'utf8',
                );
                /**
                 What the read threw.
                 */
                const thrown = await createBedrockLedger({
                  path,
                  creditUsd: 200,
                },)
                  .read()
                  .then(
                  function unexpected(): unknown {
                    return undefined;
                  },
                  function caught(error: unknown,): unknown {
                    return error;
                  },
                  );
                return (thrown instanceof BedrockLedgerShapeError) ? thrown.message : 'not a ledger shape refusal';
              },),);
              expect(refusals,).toEqual(broken.map(function expected(
                { detail, },
                index,
              ): string {
                return `Bedrock spend ledger ${ledgerPathOf(index,)} line 1 violated expectations: ${detail}`;
              },),);

              /**
               The same call written whole, which the broken lines each change one field of.
               */
              const wholePath = join(
                dir,
                'bedrock-spend-whole.jsonl',
              );
              await writeFile(
                wholePath,
                `${JSON.stringify(callCosting({ usd: 0.1, },),)}\n`,
                'utf8',
              );
              expect(await createBedrockLedger({
                path: wholePath,
                creditUsd: 200,
              },)
                .read(),).toEqual({
                  creditUsd: 200,
                  spentUsd: 0.1,
                  reckonedUsd: 0,
                  remainingUsd: 199.9,
                  calls: 1,
                },);
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'ledger settings from the environment',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE OWNER\'S CREDIT by default and an override where one is set',
          fn: async () => {
            expect(bedrockCreditUsdFrom({ env: {}, },),).toBe(BEDROCK_CREDIT_USD,);
            expect(bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: '', }, },),).toBe(BEDROCK_CREDIT_USD,);
            expect(bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: '150.5', }, },),).toBe(150.5,);
            expect(bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: '0', }, },),).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES AN OVERRIDE THAT IS NOT AN AMOUNT, naming the variable and quoting the value, '
            + 'since an operator who set it believes the credit is bounded as they asked',
          fn: async () => {
            expect(function plenty(): number {
              return bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: 'plenty', }, },);
            },).toThrow(BedrockCreditOverrideError,);
            expect(function negative(): number {
              return bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: '-5', }, },);
            },).toThrow(BedrockCreditOverrideError,);
          },
        },),

        it({
          name: 'REFUSES AN OVERRIDE NOT WRITTEN AS A PLAIN DECIMAL, which `Number` read as a credit nobody typed: '
            + 'a hexadecimal, an exponent, a sign, a space either side and a point missing digits on one side; '
            + 'and a digit run past the range a double holds (ledger B73)',
          fn: async () => {
            /**
             Spellings `Number` reads as an amount that no operator writes as one.
             */
            const spellings = [
              '0x10',
              '1e1',
              '+15',
              ' 15',
              '15 ',
              '15.',
              '.5',
              `1${'0'.repeat(400,)}`,
            ];
            for (const raw of spellings) {
              expect(function readSpelling(): number {
                return bedrockCreditUsdFrom({ env: { [BEDROCK_CREDIT_USD_VAR]: raw, }, },);
              },).toThrow(BedrockCreditOverrideError,);
            }
          },
        },),

        it({
          name: 'PUTS THE FILE UNDER THE HOME IT IS GIVEN by default, never a spelled-out username, and '
            + 'where the variable points otherwise',
          fn: async () => {
            expect(defaultBedrockLedgerPath({ home: '/srv/cats', },),).toBe(
              '/srv/cats/.local/state/translation-repair/bedrock-spend.jsonl',
            );
            expect(bedrockLedgerPathFrom({
              env: {},
              home: '/srv/cats',
            },),).toBe('/srv/cats/.local/state/translation-repair/bedrock-spend.jsonl',);
            expect(bedrockLedgerPathFrom({
              env: { [BEDROCK_LEDGER_PATH_VAR]: '/var/lib/cats/spend.jsonl', },
              home: '/srv/cats',
            },),).toBe('/var/lib/cats/spend.jsonl',);
          },
        },),
      ],
    },),

    describe({
      name: 'the ledger says how much of its spend is reckoned (ledger P1)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SUMS THE RECKONED LINES APART, in the reading and on the meter line, so a reader can see how much of '
            + 'what is left rests on bounds and estimates rather than reported usage',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger holding one reported call and one reckoned attempt.
               */
              const ledger = createBedrockLedger({
                path: join(
                  dir,
                  'bedrock-spend.jsonl',
                ),
                creditUsd: 200,
              },);
              await ledger.note(callCosting({ usd: 0.25, },),);
              await ledger.note({
                ...callCosting({ usd: 0.1, },),
                estimated: 'abandoned-bound',
              },);
              /**
               Reading of both.
               */
              const credits = await ledger.read();
              expect({
                spent: credits.spentUsd,
                reckoned: credits.reckonedUsd,
                meter: bedrockMeterLevel({ credits, },),
              },).toEqual({
                spent: 0.35,
                reckoned: 0.1,
                meter: ['bedrockUsd=199.65', 'bedrockReckonedUsd=0.10',],
              },);
            },);
          },
        },),
        it({
          name: 'REFUSES A RECKONING MARK IT DOES NOT WRITE, naming its line, rather than summing a line whose '
            + 'provenance nothing here can vouch for',
          fn: async () => {
            await inScratch(async function body(dir,) {
              /**
               Ledger file whose one line carries a mark nothing writes.
               */
              const path = join(
                dir,
                'bedrock-spend.jsonl',
              );
              await writeFile(
                path,
                `${JSON.stringify({ ...callCosting({ usd: 0.1, },), estimated: 'guessed', },)}\n`,
                'utf8',
              );
              /**
               What the read threw.
               */
              const thrown = await (async function readOrError(): Promise<unknown> {
                try {
                  return await createBedrockLedger({
                    path,
                    creditUsd: 200,
                  },).read();
                }
                catch (error) {
                  return error;
                }
              })();
              expect(thrown,).toBeInstanceOf(BedrockLedgerShapeError,);
              expect((thrown as Error).message,).toContain('line 1',);
            },);
          },
        },),
      ],
    },),
  ],
},);
