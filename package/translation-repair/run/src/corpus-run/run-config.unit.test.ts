/**
 Tests for where run artifacts are written.

 `resolveRunsDir` had no test. Everything durable a run produces lands under
 the path it returns: artifacts, logs, the attempts map, and the grading
 sheets a human spends hours on. The sheet-path guard refuses to overwrite a
 final sheet, but that guard only protects paths under whatever this function
 resolved, so a wrong answer here relocates the entire protected area rather
 than defeating one check.

 The empty-string case is the one worth having. An exported-but-empty
 environment variable is a normal shell accident, and a bare truthiness check
 would treat it as an override, resolving every artifact path relative to the
 process working directory instead of the runs directory.

 The override is injected as a disposable so the variable is restored however
 a case ends, following the pattern in
 `package/pi-plugin/morph-compact/src/api-key.unit.test.ts`.

 The run client's cases build it with `runClientFrom` on an environment and
 a transport each case hands over (ledger M43, M68), so no case builds a
 client on a key the suite inherits or over the real Bedrock ledger; the one
 case of `createRunClient`, which reads the process's own, clears every key
 first and expects the refusal.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RUN_SEATS, } from '@monochromatic-dev/translation-repair-stage';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { HYPER_MESSAGES_URL, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
} from '@monochromatic-dev/translation-repair-provider';
import {
  createRunClient,
  OWNER_CULLED,
  readHeadSha,
  resolveRunsDir,
  runClientFrom,
  RunConfigError,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';

/**
 Environment variable that overrides the runs directory.
 */
const RUNS_DIR_VAR = 'TRANSLATION_REPAIR_RUNS_DIR';

/**
 Sets the override for the life of a scope and restores it on exit.

 @param value - override to install; the empty string is meaningful here

 @returns Disposable restoring the previous value, including its absence

 @example
 ```ts
 using _override = withRunsDir({ value: '/tmp/whiskers', },);
 ```
 */
function withRunsDir({ value, }: { readonly value: string; },): Disposable {
  /**
   Value before this scope; absent means the variable was unset.
   */
  const original = process.env[RUNS_DIR_VAR];
  process.env[RUNS_DIR_VAR] = value;
  return {
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(process.env, RUNS_DIR_VAR,);
      else
        process.env[RUNS_DIR_VAR] = original;
    },
  };
}

/**
 Removes the override for the life of a scope and restores it on exit.

 @returns Disposable restoring the previous value

 @example
 ```ts
 using _unset = withoutRunsDir();
 ```
 */
function withoutRunsDir(): Disposable {
  /**
   Value before this scope; absent means the variable was already unset.
   */
  const original = process.env[RUNS_DIR_VAR];
  Reflect.deleteProperty(process.env, RUNS_DIR_VAR,);
  return {
    [Symbol.dispose](): void {
      if (original !== undefined)
        process.env[RUNS_DIR_VAR] = original;
    },
  };
}

/**
 Environment variable carrying the Synthetic API key.

 Only its NAME appears in this file. No case asserts on the value, prints it,
 or compares against it, so a failure message can never carry a real key from
 a developer's environment into a log.
 */
const API_KEY_VAR = 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY';

/**
 Environment variable carrying the second provider's API key, named only.
 */
const HYPER_KEY_VAR = 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY';

/**
 Environment variable carrying the third provider's API key, named only.
 */
const OPENROUTER_KEY_VAR = 'TRANSLATION_REPAIR_OPENROUTER_API_KEY';

/**
 Environment variable carrying the fourth provider's API key, named only.
 */
const BEDROCK_KEY_VAR = 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY';

/**
 Environment variable carrying the fifth provider's API key, named only.
 */
const NEURALWATT_KEY_VAR = 'TRANSLATION_REPAIR_NEURALWATT_API_KEY';

/**
 Every provider key variable.
 */
const PROVIDER_KEY_VARS = [
  NEURALWATT_KEY_VAR,
  API_KEY_VAR,
  HYPER_KEY_VAR,
  OPENROUTER_KEY_VAR,
  BEDROCK_KEY_VAR,
] as const;

/**
 Stand-in keys for the first two providers and nothing else.

 EVERY CLIENT HERE IS BUILT ON AN ENVIRONMENT THE CASE HANDS OVER (ledger
 M43, M68). Until 2026-09-29 these cases wrote one or two keys into
 `process.env` and built the client on the rest of it, so every case ran on
 the third and fourth keys the suite inherits from `mise` and the real
 Bedrock ledger's path; the refusal cases had to clear each new key by hand,
 and two landings (2026-09-03, 2026-09-07) turned four of them into builds
 until they did.
 */
const TWO_KEYS = {
  [API_KEY_VAR]: 'whiskers-not-a-real-key',
  [HYPER_KEY_VAR]: 'mittens-not-a-real-key',
};

/**
 Transport for cases that build a client and ask nothing.

 @returns Never

 @throws Error always
 */
async function unaskedTransport(): Promise<{ readonly status: number; readonly bodyText: string; }> {
  throw new Error('a case that only builds a client asked a provider',);
}

/**
 Clears every provider key from the process for the life of a scope and
 restores each on exit, for the one case that asks what `createRunClient`
 reads.

 @returns Disposable restoring every key the process held

 @example
 ```ts
 using _cleared = withoutProviderKeys();
 ```
 */
function withoutProviderKeys(): Disposable {
  /**
   Each key the process held, by variable.
   */
  const held = new Map<string, string>();
  for (const name of PROVIDER_KEY_VARS) {
    /**
     Value the variable holds now, if any.
     */
    const value = process.env[name];
    if (value !== undefined)
      held.set(name, value,);
    Reflect.deleteProperty(process.env, name,);
  }
  return {
    [Symbol.dispose](): void {
      for (const [name, value,] of held)
        process.env[name] = value;
    },
  };
}



/**
 Streamed reply the first provider's chat endpoint answers with in the wiring
 cases: one content delta and the terminator, the way the provider ends a
 stream. Cat-themed, like every fixture here.
 */
const FIRST_PROVIDER_REPLY = [
  `data: ${JSON.stringify({ choices: [{ delta: { content: '喵。', }, },], },)}`,
  'data: [DONE]',
  '',
].join('\n\n',);

/**
 Status the wiring transport answers where a call must fail at once: not a
 budget status, so the router does not re-ask the other provider, and not a
 transient one, so no retry ladder waits on it.
 */
const REFUSED_OUTRIGHT = 400;

/**
 Status a provider answers with when it rejects the credential it was sent.
 */
const UNAUTHORIZED = 401;

/**
 Status of the one endpoint that answers.
 */
const ANSWERED = 200;

/**
 Whether a URL is the first provider's chat endpoint.

 @param url - URL the transport was asked

 @returns Whether a chat exchange went to the first provider

 @example
 ```ts
 const askedFirst = urls.some(isFirstProviderChat,);
 ```
 */
function isFirstProviderChat(url: string,): boolean {
  return url.endsWith('/chat/completions',);
}

/**
 Builds a transport that records every URL asked and answers by endpoint:
 the first provider's chat endpoint streams `FIRST_PROVIDER_REPLY`, the
 second provider's messages endpoint refuses outright, and every meter
 (quotas, credits) refuses too. AN UNREADABLE METER READS AS SPENDABLE, which
 is the documented failover in `provider-budget.ts`, so the routing these
 cases observe is decided on serving capability alone, never on budget.

 @returns Transport plus the URLs it was asked, in call order

 @example
 ```ts
 const { transport, urls, } = recordingTransport();
 ```
 */
function recordingTransport(): {
  readonly transport: (exchange: { readonly url: string; },) => Promise<{
    readonly status: number;
    readonly bodyText: string;
  }>;
  readonly urls: string[];
} {
  /**
   URLs asked so far, pushed as each exchange arrives.
   */
  const urls: string[] = [];
  return {
    async transport(exchange: { readonly url: string; },): Promise<{
      readonly status: number;
      readonly bodyText: string;
    }> {
      urls.push(exchange.url,);
      if (isFirstProviderChat(exchange.url,))
        return { status: ANSWERED, bodyText: FIRST_PROVIDER_REPLY, };
      return { status: REFUSED_OUTRIGHT, bodyText: '{}', };
    },
    urls,
  };
}

/**
 Single user message reused across the wiring exchanges.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '猫猫的翻译对吗？',
  },
];

/**
 Seat the first provider serves under its own catalog name. Not the
 every-provider fixture seat, which is the model the owner culled on
 2026-09-24 and which the run client refuses (ledger P4).
 */
const SHARED_SEAT = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Seat the owner culled from every role, which every provider still serves.
 */
const CULLED_SEAT = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Seat only the second provider serves: a Charm Hyper endpoint label.
 */
const SECOND_ONLY_SEAT = SEAT_HYPER_VISION;

/**
 Asks one seat through the client and hands back whatever came of it, the
 reply or the failure, because half of the wiring cases expect the call to
 fail and care only about where it went and how it was counted.

 @param client - client under test

 @param modelId - seat to ask

 @returns Reply when the call answered, otherwise what it threw

 @example
 ```ts
 const came = await askSeat({ client, modelId: SECOND_ONLY_SEAT, },);
 ```
 */
async function askSeat(
  {
    client,
    modelId,
  }: {
    readonly client: ReturnType<typeof runClientFrom>;
    readonly modelId: typeof CULLED_SEAT | typeof SHARED_SEAT | typeof SECOND_ONLY_SEAT;
  },
): Promise<unknown> {
  try {
    return await client.chatText({
      modelId,
      messages: MESSAGES,
      signal: new AbortController().signal,
    },);
  }
  catch (error) {
    return error;
  }
}

/**
 Moves the process working directory for the life of a scope and restores it
 on exit.

 @param path - directory to move to

 @returns Disposable restoring the previous working directory

 @example
 ```ts
 using _elsewhere = inDirectory({ path: tmpdir(), },);
 ```
 */
function inDirectory({ path, }: { readonly path: string; },): Disposable {
  /**
   Working directory before this scope.
   */
  const original = process.cwd();
  process.chdir(path,);
  return {
    [Symbol.dispose](): void {
      process.chdir(original,);
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveRunsDir.name,
      // ONE AT A TIME: cases set and clear the one process-wide runs directory,
      // and a restore finishing out of order could leave a sibling's value set
      // for the suites after this one (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'honors an explicit override exactly, so a run can be pointed at a '
            + 'throwaway directory without touching the real one',
          fn: async () => {
            using _override = withRunsDir({ value: '/tmp/whiskers-runs', },);

            expect(await resolveRunsDir(),).toBe('/tmp/whiskers-runs',);
          },
        },),

        it({
          name: 'IGNORES an empty override and falls back to the default. An '
            + 'exported-but-empty variable is an ordinary shell accident, and '
            + 'treating it as an override would resolve every artifact path '
            + 'relative to the process working directory instead of the runs '
            + 'directory, scattering a run and moving the sheet guard\'s protected '
            + 'area with it',
          fn: async () => {
            using _empty = withRunsDir({ value: '', },);

            /**
             Resolved directory under an empty override.
             */
            const resolved = await resolveRunsDir();

            expect(resolved,).not.toBe('',);
            expect(resolved,).toContain(join(
              'node_modules',
              '.monochromatic',
              'translation-repair-runs',
            ),);
          },
        },),

        it({
          name: 'defaults under the worktree\'s gitignored node_modules when no '
            + 'override is set, so artifacts are durable across runs yet can never '
            + 'be committed: the corpus they derive from is unlicensed',
          fn: async () => {
            using _unset = withoutRunsDir();

            /**
             Resolved directory with no override present.
             */
            const resolved = await resolveRunsDir();

            expect(resolved,).toContain(join(
              'node_modules',
              '.monochromatic',
              'translation-repair-runs',
            ),);
            expect(resolved.startsWith('/',),).toBe(true,);
          },
        },),

        it({
          name: 'returns an ABSOLUTE path in both branches, since callers join '
            + 'sheet and artifact names onto it from working directories they do '
            + 'not control',
          fn: async () => {
            {
              using _override = withRunsDir({ value: '/tmp/whiskers-runs', },);

              expect((await resolveRunsDir()).startsWith('/',),).toBe(true,);
            }

            using _unset = withoutRunsDir();

            expect((await resolveRunsDir()).startsWith('/',),).toBe(true,);
          },
        },),

        it({
          name: 'restores the environment after each case, so one case cannot '
            + 'silently decide where a later one writes',
          fn: async () => {
            /**
             Value outside any override scope.
             */
            const outside = process.env[RUNS_DIR_VAR];

            {
              using _override = withRunsDir({ value: '/tmp/whiskers-runs', },);

              expect(process.env[RUNS_DIR_VAR],).toBe('/tmp/whiskers-runs',);
            }

            expect(process.env[RUNS_DIR_VAR],).toBe(outside,);
          },
        },),
      ],
    },),

    describe({
      name: runClientFrom.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'builds a client with the chat and meter surface from the keys handed over, so every existing caller '
            + 'and the bench recorder are untouched by routing',
          fn: async () => {
            /**
             Client built from the stand-in keys.
             */
            const client = runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport: unaskedTransport, },);

            // `quotas` is the first provider's meter and nothing else; the routing
            // client does not offer one, and this wiring layer supplies it.
            expect(typeof client.chatJson,).toBe('function',);
            expect(typeof client.chatText,).toBe('function',);
            expect(typeof client.quotas,).toBe('function',);
          },
        },),

        it({
          name: 'BUILDS with only Synthetic configured because one wet provider is normal mode',
          fn: async () => {
            const client = runClientFrom({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { [API_KEY_VAR]: 'whiskers-not-a-real-key', },
              transport: unaskedTransport,
            },);
            expect(typeof client.chatText,).toBe('function',);
          },
        },),

        it({
          name: 'BUILDS with only Hyper configured because no provider family is mandatory, and answers the '
            + 'first provider\'s meter as exhausted rather than asking anyone (ledger T8)',
          fn: async () => {
            const client = runClientFrom({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { [HYPER_KEY_VAR]: 'mittens-not-a-real-key', },
              transport: unaskedTransport,
            },);
            expect(await client.quotas({ signal: new AbortController().signal, },),).toEqual({
              fiveHour: {
                remaining: 0,
                max: 0,
                limited: true,
                nextTickAt: '',
              },
              weekly: {
                percentRemaining: 0,
                nextRegenAt: '',
              },
            },);
          },
        },),

        it({
          name: 'BUILDS ON EVERY KEY, the Bedrock client reading the ledger and credit the handed environment names, '
            + 'wet under its credit and dry at none, while the meters it cannot read stay spendable (ledger T8)',
          fn: async () => {
            /**
             Directory holding this case's empty ledger.
             */
            await using scratch = await scratchDir({ prefix: 'run-client-ledger-', },);
            const dir = scratch.path;
            /**
             What each provider's meter read as, on a client keyed for all four.

             @param creditUsd - Bedrock credit the environment grants

             @returns Dryness per provider
             */
            async function drynessAt(creditUsd: string,): Promise<unknown> {
              return await runClientFrom({
                maxHoldMs: TEST_MAX_HOLD_MS,
                env: {
                  ...TWO_KEYS,
                  [OPENROUTER_KEY_VAR]: 'tabby-not-a-real-key',
                  [BEDROCK_KEY_VAR]: 'calico-not-a-real-key',
                  [BEDROCK_LEDGER_PATH_VAR]: join(dir, 'bedrock-spend.jsonl',),
                  [BEDROCK_CREDIT_USD_VAR]: creditUsd,
                },
                // BEDROCK IS LEFT OFF BY DEFAULT (`providers-off.ts`); this case is about the client it builds when on.
                off: [],
                transport: async function refuseEveryMeter() {
                  return { status: 400, bodyText: '{}', };
                },
              },).providerDryness({ signal: new AbortController().signal, },);
            }
            expect(await drynessAt('40',),).toEqual({
              neuralwatt: true,
              synthetic: false,
              hyper: false,
              bedrock: false,
              deepseek: true,
              xiaomi: true,
              anthropic: true,
              openrouter: false,
            },);
            expect(await drynessAt('0',),).toEqual({
              neuralwatt: true,
              synthetic: false,
              hyper: false,
              bedrock: true,
              deepseek: true,
              xiaomi: true,
              anthropic: true,
              openrouter: false,
            },);
          },
        },),

        it({
          name: 'refuses with no key at all as a STATED refusal, so the CLI boundary repeats the variable name and '
            + 'exits 6 instead of printing a fault with frames: a RunConfigError naming the variable and mise, '
            + 'never content',
          fn: async () => {
            /**
             What building with no key raised, read for its marker, class and wording.
             */
            const refusal = caught(function buildWithoutKey() {
              runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: {}, transport: unaskedTransport, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as StatedRefusalError).messageNamesOnly,).toBe(true,);
            expect(refusal,).toBeInstanceOf(RunConfigError,);
            expect((refusal as Error).message,).toContain(API_KEY_VAR,);
            expect((refusal as Error).message,).toContain('mise',);
          },
        },),

        it({
          name: 'REFUSES when every provider key is empty, since an exported empty variable is no key',
          fn: async () => {
            /**
             What building with four empty keys raised.
             */
            const refusal = caught(function buildWithEmptyKeys() {
              runClientFrom({
                maxHoldMs: TEST_MAX_HOLD_MS,
                env: Object.fromEntries(PROVIDER_KEY_VARS.map(function empty(name,) {
                  return [name, '',];
                },),),
                transport: unaskedTransport,
              },);
            },);

            expect(refusal,).toBeInstanceOf(RunConfigError,);
            expect((refusal as Error).message,).toContain(API_KEY_VAR,);
          },
        },),
      ],
    },),

    describe({
      name: createRunClient.name,
      children: [
        it({
          name: 'READS THE PROCESS ENVIRONMENT, the one place a runner hands it over, and refuses when it holds no '
            + 'provider key (ledger T8)',
          fn: async () => {
            using _cleared = withoutProviderKeys();

            /**
             What building on a process holding no key raised.
             */
            const refusal = caught(function buildOnProcess() {
              createRunClient({ maxHoldMs: TEST_MAX_HOLD_MS, },);
            },);

            expect(refusal,).toBeInstanceOf(RunConfigError,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: `${runClientFrom.name} wiring`,
      children: [
        it({
          name: 'ROUTES a Charm Hyper endpoint label to the second provider and '
            + 'never to the first, with the first provider live: serving '
            + 'capability is a property of the pair, not of a provider\'s health',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport recording where the call went.
             */
            const { transport, urls, } = recordingTransport();

            await askSeat({
              client: runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },),
              modelId: SECOND_ONLY_SEAT,
            },);

            expect(urls.includes(HYPER_MESSAGES_URL,),).toBe(true,);
            expect(urls.some(isFirstProviderChat,),).toBe(false,);
          },
        },),

        it({
          name: 'READS A PROVIDER WHOSE CREDENTIAL WAS REFUSED AS DRY in the view the seat readers and benches are '
            + 'derived from, though its meter reads wet: the money is there and the key is not accepted (zheermao19)',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport refusing the first provider's credential and every other endpoint outright.
             */
            async function refuseFirstProviderCredential(exchange: { readonly url: string; },): Promise<{
              readonly status: number;
              readonly bodyText: string;
            }> {
              if (isFirstProviderChat(exchange.url,))
                return { status: UNAUTHORIZED, bodyText: '{"error":{"code":"invalid_api_key"}}', };
              return { status: REFUSED_OUTRIGHT, bodyText: '{}', };
            }

            /**
             Client under test, whose first provider rejects its credential.
             */
            const client = runClientFrom({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: TWO_KEYS,
              transport: refuseFirstProviderCredential,
            },);
            const { signal, } = new AbortController();

            /**
             Dryness before any call met the refusal.
             */
            const before = await client.providerDryness({ signal, },);
            await askSeat({ client, modelId: SHARED_SEAT, },);
            /**
             Dryness once a call met it.
             */
            const after = await client.providerDryness({ signal, },);

            expect({ before: before.synthetic, after: after.synthetic, hyper: after.hyper, },).toEqual({
              before: false,
              after: true,
              hyper: false,
            },);
          },
        },),

        it({
          name: 'SENDS a seat the first provider serves to the first provider, so '
            + 'the routing does not push the whole roster onto the second',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport recording where the call went.
             */
            const { transport, urls, } = recordingTransport();

            /**
             What the shared seat answered.
             */
            const came = await askSeat({
              client: runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },),
              modelId: SHARED_SEAT,
            },);

            expect(Error.isError(came,),).toBe(false,);
            expect(urls.some(isFirstProviderChat,),).toBe(true,);
            expect(urls.includes(HYPER_MESSAGES_URL,),).toBe(false,);
          },
        },),

        it({
          name: 'WRAPS ROUTED CLIENT with model-prompt payload reuse',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport recording provider calls.
             */
            const { transport, urls, } = recordingTransport();
            /**
             One configured client preserving prompt claims across calls.
             */
            const client = runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },);
            await askSeat({ client, modelId: SHARED_SEAT, },);
            const duplicate = await askSeat({ client, modelId: SHARED_SEAT, },);

            expect(Error.isError(duplicate,),).toBe(false,);
            expect(urls.filter(isFirstProviderChat,),).toHaveLength(1,);
          },
        },),

        it({
          name: 'COUNTS every call against its seat on the run-wide tally, so the '
            + 'closing report can say which seat never answered',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport answering the first provider and refusing the second.
             */
            const { transport, } = recordingTransport();

            /**
             Client under test, built once for both seats.
             */
            const client = runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },);

            await askSeat({ client, modelId: SHARED_SEAT, },);
            await askSeat({ client, modelId: SECOND_ONLY_SEAT, },);

            /**
             Counts for the seat that answered.
             */
            const shared = RUN_SEATS.counts().find(function isShared(count,): boolean {
              return count.modelId === SHARED_SEAT;
            },);

            /**
             Counts for the seat that was refused.
             */
            const secondOnly = RUN_SEATS.counts().find(function isSecondOnly(count,): boolean {
              return count.modelId === SECOND_ONLY_SEAT;
            },);

            expect(shared?.asked,).toBe(1,);
            expect(shared?.usable,).toBe(1,);
            expect(secondOnly?.asked,).toBe(1,);
            expect(secondOnly?.threw,).toBe(1,);
            expect(RUN_SEATS.dark().map(function toId(count,): string {
              return count.modelId;
            },),).toStrictEqual([SECOND_ONLY_SEAT,],);
          },
        },),

        it({
          name: 'COUNTS AN UNREADABLE ANSWER AS UNUSABLE on the run-wide tally (ledger P6): the tally sat inside the '
            + 'prompt-uniqueness wrapper, which parses the reply itself, so every reply that arrived read as usable '
            + '("SEAT inception/mercury-2.5 asked=1007 usable=1007 unusable=0" beside 40 schema losses)',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport answering the first provider with text no schema reads.
             */
            const { transport, } = recordingTransport();
            await runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },).chatJson({
              modelId: SHARED_SEAT,
              messages: MESSAGES,
              signal: new AbortController().signal,
              responseFormat: {
                type: 'json_schema',
                json_schema: {
                  name: 'purr',
                  strict: true,
                  schema: {
                    type: 'object',
                    properties: { purr: { type: 'string', }, },
                    required: ['purr',],
                    additionalProperties: false,
                  },
                },
              },
              validate: function isPurr(value: unknown,): value is { readonly purr: string; } {
                return ((typeof value) === 'object') && (value !== null) && ('purr' in value);
              },
            },);

            /**
             Counts for the seat.
             */
            const counts = RUN_SEATS.counts().find(function isShared(count,): boolean {
              return count.modelId === SHARED_SEAT;
            },);
            expect({
              asked: counts?.asked,
              usable: counts?.usable,
              unusable: counts?.unusable,
            },).toEqual({
              asked: 1,
              usable: 0,
              unusable: 1,
            },);
          },
        },),

        it({
          name: 'REFUSES A SEAT THE OWNER CULLED without calling any provider (ledger P4): the cull held on every '
            + 'bench the run derives, and a path that named the seat itself, the recall benchmark\'s default '
            + 'judges, would have bought it; the refusal is the one a round reads as an unreachable seat',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport recording where any call went.
             */
            const { transport, urls, } = recordingTransport();

            /**
             What the culled seat's call came to.
             */
            const came = await askSeat({
              client: runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, },),
              modelId: CULLED_SEAT,
            },);

            expect({
              culled: OWNER_CULLED.has(CULLED_SEAT,),
              refused: came instanceof NoProviderForModelError,
              chatted: urls.some(isFirstProviderChat,) || urls.includes(HYPER_MESSAGES_URL,),
            },).toEqual({
              culled: true,
              refused: true,
              chatted: false,
            },);
          },
        },),
        it({
          name: 'REFUSES A SEAT WHOSE ONLY PROVIDER HAS NO KEY without asking any provider (ledger T8)',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Transport recording where any call went.
             */
            const { transport, urls, } = recordingTransport();

            /**
             What the second provider's seat came to on a client keyed for the first alone.
             */
            const came = await askSeat({
              client: runClientFrom({
                maxHoldMs: TEST_MAX_HOLD_MS,
                env: { [API_KEY_VAR]: 'whiskers-not-a-real-key', },
                transport,
              },),
              modelId: SECOND_ONLY_SEAT,
            },);

            expect(came,).toBeInstanceOf(NoProviderForModelError,);
            expect(urls.some(isFirstProviderChat,) || urls.includes(HYPER_MESSAGES_URL,),).toBe(false,);
          },
        },),

        it({
          name: 'KEEPS PROMPT PAYLOADS UNDER THE DIRECTORY HANDED OVER, so a second client over it answers the same '
            + 'prompt without asking (ledger T8)',
          fn: async () => {
            using _fresh = withFreshRunSeats();

            /**
             Payload directory this case owns.
             */
            await using scratch = await scratchDir({ prefix: 'run-client-payloads-', },);
            const dir = scratch.path;

            /**
             Transport recording provider calls.
             */
            const { transport, urls, } = recordingTransport();
            await askSeat({
              client: runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, promptPayloadDir: dir, },),
              modelId: SHARED_SEAT,
            },);

            /**
             What a fresh client over the same payloads answered.
             */
            const replayed = await askSeat({
              client: runClientFrom({ maxHoldMs: TEST_MAX_HOLD_MS, env: TWO_KEYS, transport, promptPayloadDir: dir, },),
              modelId: SHARED_SEAT,
            },);

            expect(Error.isError(replayed,),).toBe(false,);
            expect(urls.filter(isFirstProviderChat,),).toHaveLength(1,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: readHeadSha.name,
      // ONE AT A TIME: a case moves the process's working directory across an
      // await, which every case beside it would run in (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'reads the sha of THIS repository regardless of the working '
            + 'directory the task was invoked from. The pin is what says which '
            + 'pipeline version produced an artifact, so resolving it against the '
            + 'process cwd would stamp another repository\'s sha onto a run, or '
            + 'fail outright when a task ran from a directory git does not track',
          fn: async () => {
            /**
             Sha read from the ordinary working directory.
             */
            const fromHere = await readHeadSha();

            using _elsewhere = inDirectory({ path: tmpdir(), },);

            expect(await readHeadSha(),).toBe(fromHere,);
          },
        },),

        it({
          name: 'returns a bare 40-character sha with no trailing newline, since '
            + 'it is written into artifacts and a stray newline there would travel '
            + 'into every file that records the pin',
          fn: async () => {
            /**
             Sha under test.
             */
            const sha = await readHeadSha();

            expect(sha.length,).toBe(40,);
            expect(sha.includes('\n',),).toBe(false,);
            expect(sha,).toBe(sha.trim(),);
          },
        },),
      ],
    },),
  ],
},);
