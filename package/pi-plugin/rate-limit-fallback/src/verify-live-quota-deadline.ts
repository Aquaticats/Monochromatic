/**
 Read every pool member's live quota endpoint under the shipped deadline, and show that the deadline can fire.
 @module

 The package README carried one precondition before writer rounds use the pool: a live check of the quota read's deadline.
 `doc/quota-read-latency.md` measured that deadline from a live endpoint measurement on 2026-11-19, and its last section records
 the check this file makes. Two arms, so a read that finished says something about the endpoint rather than about a probe that
 cannot hang:

 - `live`: one read per pool member per round, bounded by the shipped `QUOTA_READ_TIMEOUT_MS`, timed around the whole
   `readQuotaWithinDeadline` call. A member whose provider publishes no quota endpoint resolves with empty evidence and sends no
   request (`README.md`, "Provider sources").
 - `control`: the same call with a transport that never answers, which shows the bound can abandon a read.

 This is a Pi extension, not a command: it runs inside a started session, so the members' credentials come from the registry that
 session hands its extensions, which is the call the read itself makes. `run-the-live-quota-deadline-check.ts` starts it and reads
 what it writes. Every line names a member, a round, the milliseconds it took and how the read ended: no header, key or token is
 read here or written anywhere.

 Run it with `mise run //package/pi-plugin/rate-limit-fallback:verify:quota-deadline`.
 */

import { appendFile, } from 'node:fs/promises';
import { setTimeout as waitFor, } from 'node:timers/promises';
import type {
  Api,
  FetchFunction,
  Model,
} from '@earendil-works/pi-ai';
import { describeCaught, } from './error-class-name.ts';
import {
  QUOTA_READ_TIMEOUT_MS,
  readQuotaWithinDeadline,
} from './member-capacity.ts';
import { POOL_DEFINITIONS, } from './model-pools.ts';
import type { RoutingRegistry, } from './quota-endpoints.ts';

/**
 The variable naming the file this check writes its record to, which its starter sets.
 */
export const RECORD_PATH_VARIABLE = 'PI_QUOTA_DEADLINE_RECORD';

/**
 Rounds each member is read for: three reads, which is a spot check of one evening rather than a distribution.
 */
const ROUNDS = 3;

/**
 Milliseconds to wait before each read, from NeuralWatt's own published poll interval: its `/v1/rate_limits` is rate-limited to
 two requests a second per customer, and it asks clients to poll at most once a second.
 */
const READ_SPACING_MS = 1_100;

/**
 Milliseconds of grace after the deadline before a control read that never settles is recorded as one, so a transport that
 ignores its signal cannot hold the check.
 */
const CONTROL_GRACE_MS = 4_000;

/**
 The answer of the control arm's grace timer, meaning the stalled read had not settled by then.
 */
const NO_SETTLE = 'no settle within the grace after the deadline';

/**
 Providers whose member publishes a quota endpoint, which is the provider test `probeQuota` makes.
 */
const PROVIDERS_WITH_A_QUOTA_ENDPOINT: ReadonlySet<string> = new Set([
  'neuralwatt',
  'synthetic',
]);

/**
 A pool member as this check reads it: the provider and model id that name it, and nothing else of the SDK's record.
 */
type MemberIdentity = {
  readonly provider: string;
  readonly id: string;
};

/**
 One line of the record, which is JSON with no credential in it.
 */
type RecordLine = Readonly<Record<string, unknown>>;

/**
 Pi's extension API, as far as this check uses it.
 */
type CheckHost = {
  on: (
    event: 'session_start',
    handler: (
      event: unknown,
      context: CheckContext,
    ) => Promise<void>,
  ) => unknown;
};

/**
 What Pi hands an extension when a session starts.
 */
type CheckContext = {
  readonly modelRegistry: RoutingRegistry;
};

/**
 The transport the control arm needs: one that never answers.

 @returns a promise that never settles

 @example
 ```ts
 const stalled = neverAnswers();
 ```
 */
function neverAnswers(): Promise<Response> {
  return Promise.withResolvers<Response>()
    .promise;
}

/**
 Reads every pool member's endpoints, and the control arm, then writes the record.

 @param registry - the started session's own model registry

 @param recordPath - the file the record's lines go to

 @example
 ```ts
 await measure({ registry: context.modelRegistry, recordPath: '/tmp/record.jsonl' });
 ```
 */
async function measure({
  registry,
  recordPath,
}: {
  readonly registry: RoutingRegistry;
  readonly recordPath: string;
},): Promise<void> {
  /**
   Appends one line to the record, so a check that ends early still leaves what it read.

   @param line - the line to append
   */
  async function note(line: RecordLine,): Promise<void> {
    await appendFile(
      recordPath,
      `${JSON.stringify(line,)}\n`,
    );
  }
  /**
   The session registry's own lookup, bound to the registry it came from: Pi's registry reads its runtime through `this`, so a
   lookup taken off the object and called bare would find nothing.
   */
  const modelOf = registry.find
    .bind(registry,);
  await note({
    kind: 'deadline',
    deadlineMs: QUOTA_READ_TIMEOUT_MS,
    rounds: ROUNDS,
  },);
  /**
   Every distinct member of both pools, in the order the pools ask them.
   */
  const members: MemberIdentity[] = [];
  for (const pool of POOL_DEFINITIONS) {
    for (const member of pool.members) {
      if (!members.some(function isSeen(seen: MemberIdentity,) {
        return (seen.provider === member.provider) && (seen.id === member.id);
      },)) {
        members.push(member,);
      }
    }
  }
  /**
   The members this session's registry holds, which are the ones this check reads.
   */
  const present: MemberIdentity[] = [];
  for (const member of members) {
    /**
     The catalog's model for this member, or nothing when this session holds none.
     */
    const model = modelOf(
      member.provider,
      member.id,
    );
    if (model === undefined) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one line per absent member, written before the reads begin, so the record names every member it could not read; see doc/lint/sequential-awaits.md
      await note({
        kind: 'missing',
        member: `${member.provider}/${member.id}`,
      },);
      continue;
    }
    present.push(member,);
  }
  for (const member of present) {
    /**
     The name a line reports this member under.
     */
    const label = `${member.provider}/${member.id}`;
    /**
     Whether this member's provider publishes an endpoint, which is what says a read of 0 ms means no request rather than an
     answer.
     */
    const endpointExpected = PROVIDERS_WITH_A_QUOTA_ENDPOINT.has(member.provider,);
    /**
     The catalog's model for this member, which the shipped read asks for its credentials.
     */
    const model = modelOf(
      member.provider,
      member.id,
    );
    if (model === undefined) {
      continue;
    }
    for (let round = 1; round <= ROUNDS; round += 1) {
      // NeuralWatt asks clients to poll its rate-limit endpoint at most once a second, so no two reads come closer than that,
      // whichever member's round this is.
      // oxlint-disable-next-line eslint/no-await-in-loop -- one read after another, each a second apart at the provider's own asking; see doc/lint/sequential-awaits.md
      await waitFor(READ_SPACING_MS,);
      /**
       When this read began.
       */
      const beganAt = performance.now();
      /**
       How this read ended, as the record says it.
       */
      let outcome = 'evidence';
      try {
        /**
         What the shipped read answered: parsed facts, empty facts for a provider with no endpoint, or its sentinel for no
         evidence.
         */
        // oxlint-disable-next-line eslint/no-await-in-loop -- the rounds are read one at a time, so each line names the transport's own moment; see doc/lint/sequential-awaits.md
        const read = await readQuotaWithinDeadline({
          registry,
          model,
          fetchQuota: globalThis.fetch,
          signal: AbortSignal.timeout(QUOTA_READ_TIMEOUT_MS,),
        },);
        /**
         Whether the read answered the shipped sentinel rather than facts.
         */
        const unavailable = ((typeof read) === 'symbol');
        /**
         How many facts the read brought back, which is none for a provider that publishes no endpoint.
         */
        const facts = unavailable ? 0 : Object.keys(read,)
          .length;
        if (unavailable) {
          outcome = 'unavailable';
        }
        else if (facts === 0) {
          outcome = 'no-endpoint';
        }
      }
      catch (error) {
        outcome = `threw ${describeCaught(error,)}`;
      }
      // oxlint-disable-next-line eslint/no-await-in-loop -- each round's line is written before the next round, so a run that ends early keeps what it read; see doc/lint/sequential-awaits.md
      await note({
        kind: 'live',
        member: label,
        round,
        elapsedMs: Math.round(performance.now() - beganAt,),
        outcome,
        endpointExpected,
      },);
    }
  }
  // The control arm: the same shipped call with a transport that never answers, which is what shows the bound can abandon a read.
  // A member with an endpoint is used, so the read reaches the transport instead of returning before it.
  /**
   The member the control reads, which is the first pool member whose provider publishes an endpoint.
   */
  const controlMember = members.find(function hasAnEndpoint(member: MemberIdentity,) {
    return PROVIDERS_WITH_A_QUOTA_ENDPOINT.has(member.provider,);
  },);
  /**
   The control member's model, as the session's registry holds it.
   */
  const controlModel = (controlMember === undefined)
    ? undefined
    : modelOf(
      controlMember.provider,
      controlMember.id,
    );
  await note({
    kind: 'control-start',
    found: (controlModel !== undefined) && (controlMember !== undefined),
  },);
  if ((controlModel !== undefined) && (controlMember !== undefined)) {
    /**
     When the control read began.
     */
    const beganAt = performance.now();
    // `AbortSignal.timeout`'s timer is unref'd, so a read whose transport ignores its signal settles only while something else
    // keeps the event loop alive. A referenced timer holds it here, as the in-flight model request holds it on the dispatch path.
    /**
     What the control read answered, or the grace timer's own sentinel.
     */
    const settled: object | symbol | string = await Promise.race([
      readQuotaWithinDeadline({
        registry,
        model: controlModel,
        fetchQuota: neverAnswers,
        signal: AbortSignal.timeout(QUOTA_READ_TIMEOUT_MS,),
      },),
      waitFor(CONTROL_GRACE_MS,)
        .then(function noAnswerAfterTheGrace() {
          return NO_SETTLE;
        },),
    ],);
    await note({
      kind: 'control',
      member: `${controlMember.provider}/${controlMember.id}`,
      elapsedMs: Math.round(performance.now() - beganAt,),
      settled: ((typeof settled) === 'symbol') || (settled === NO_SETTLE),
      abandoned: (typeof settled) === 'symbol',
    },);
  }
  await note({ kind: 'done', },);
}

/**
 Registers the check on a started session, which is where it reads the registry and writes the record.

 @param pi - Pi's extension API for this session

 @example
 ```ts
 pi -e src/verify-live-quota-deadline.ts --no-extensions --no-session --mode rpc
 ```
 */
export default function liveQuotaDeadlineCheck(pi: CheckHost,): void {
  pi.on(
    'session_start',
    async function onSessionStart(
      _event: unknown,
      context: CheckContext,
    ) {
    /**
     The file this run's record goes to, which its starter names: a check that wrote nowhere would pass by writing nothing.
     */
      const recordPath = process.env[RECORD_PATH_VARIABLE] ?? '';
      if (recordPath === '') {
        throw new Error(`${RECORD_PATH_VARIABLE} names no file, so this check has nowhere to record what it read`,);
      }
      await measure({
        registry: context.modelRegistry,
        recordPath,
      },);
    },
  );
}
