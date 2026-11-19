/**
 Prototype: native Pi virtual models whose routers send every request to this package's pool provider.

 This module answers one question and ships nothing: does routing through a native virtual model keep
 the package's in-request member fallback and regain Pi's context-overflow compact-and-retry? The
 findings page is `doc/hybrid-native-virtual-model-prototype.md`.

 Load it into one session and nothing else:

 ```sh
 pi --no-extensions --extension /path/to/src/hybrid-prototype/hybrid-extension.ts \
   --model rate-limit-fallback/hybrid-writer "Reply with one word."
 ```

 It registers, beside the package's own provider:
 - `rate-limit-fallback/hybrid-writer`, a router to the real `writer` pool;
 - `rate-limit-fallback/hybrid-probe`, a router to a probe pool whose members this run chooses;
 - `rate-limit-fallback/hybrid-native-probe`, a router straight to one stand-in member, with no pool;
 - `rate-limit-fallback-probe/probe`, the probe pool itself, over `PI_HYBRID_PROBE_MEMBERS`;
 - `hybrid-probe-member/*`, stand-in members with no network behind them.

 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ModelRoute,
} from '@earendil-works/pi-coding-agent';
import type {
  Api,
  Model,
  ModelThinkingLevel,
} from '@earendil-works/pi-ai';
import rateLimitFallbackExtension, { PROVIDER_ID, } from '../index.ts';
import type { Member } from '../model-pools.ts';
import {
  ANSWERING_MEMBER,
  STAND_IN_PROVIDER_ID,
  standInProvider,
} from './stand-in-member-provider.ts';
import {
  PROTOTYPE_POOL_ID,
  PROTOTYPE_POOL_PROVIDER_ID,
  registerPrototypePool,
} from './prototype-pool-provider.ts';
import {
  excerpt,
  note,
} from './prototype-evidence.ts';
import {
  noteContextUsage,
  registerAlwaysRouter,
  resolved,
  type TurnMessageView,
} from './prototype-routing-evidence.ts';

//region Catalog the prototype registers

/**
 Virtual model routing to the package's real writer pool.
 */
const HYBRID_WRITER_ID = 'hybrid-writer';

/**
 Virtual model routing to the probe pool.
 */
const HYBRID_PROBE_ID = 'hybrid-probe';

/**
 Virtual model routing straight to one stand-in member, the native-only control arm.
 */
const HYBRID_NATIVE_PROBE_ID = 'hybrid-native-probe';

/**
 Limits the routers declare, which Pi uses until a reply names the member that answered.

 Deliberately not a member's limits: item 4 of the findings page measures which pair Pi applies
 before the first reply and which after it.
 */
const DECLARED_CONTEXT_WINDOW = 200_000;

/**
 Output limit the routers declare beside their window.
 */
const DECLARED_MAX_TOKENS = 8_192;

/**
 Limits object every router declares.
 */
const DECLARED_LIMITS = {
  contextWindow: DECLARED_CONTEXT_WINDOW,
  maxTokens: DECLARED_MAX_TOKENS,
} as const;

/**
 Environment variable naming the probe pool's members, as JSON.

 The prototype keeps the member list out of the code so one session can dispatch to a refusing
 member and the next to a stand-in that overflows, without a rebuild between runs.
 */
const PROBE_MEMBERS_VARIABLE = 'PI_HYBRID_PROBE_MEMBERS';

/**
 Environment variable naming the member the native-only router sends every request to.
 */
const NATIVE_TARGET_VARIABLE = 'PI_HYBRID_NATIVE_TARGET';

/**
 Member the native-only router uses when the environment names none.
 */
const DEFAULT_NATIVE_TARGET: Member = {
  provider: STAND_IN_PROVIDER_ID,
  id: ANSWERING_MEMBER,
};

//endregion Catalog the prototype registers


//region Reading the session's own choices

/**
 Read one member out of an untrusted JSON entry.

 @param entry - parsed JSON value that must name a provider and a model id

 @returns member the probe pool dispatches to

 @throws Error when the entry does not name both as strings

 @example
 ```ts
 memberOf({ provider: 'hybrid-probe-member', id: 'answers' });
 ```
 */
function memberOf(entry: unknown,): Member {
  if ((typeof entry) !== 'object')
    throw new Error('A probe member must be an object',);
  if (entry === null)
    throw new Error('A probe member must be an object',);
  if ((!('provider' in entry)) || (!('id' in entry)))
    throw new Error('A probe member must name provider and id',);
  /**
   Fields the entry carries, unverified until each is read.
   */
  const candidate = entry as {
    provider: unknown;
    id: unknown
  };
  if ((typeof candidate.provider) !== 'string')
    throw new Error('A probe member must name its provider as a string',);
  if ((typeof candidate.id) !== 'string')
    throw new Error('A probe member must name its model id as a string',);
  return {
    provider: candidate.provider,
    id: candidate.id,
  };
}

/**
 Read the probe pool's member list from the environment, falling back when it names none.

 @param variable - environment variable holding a JSON array of members

 @param fallback - members to use when the variable names none

 @returns ordered members the probe pool dispatches to

 @example
 ```ts
 membersFromEnvironment({ variable: 'PI_HYBRID_PROBE_MEMBERS', fallback: [] });
 ```
 */
function membersFromEnvironment({
  variable,
  fallback,
}: {
  readonly variable: string;
  readonly fallback: readonly Member[];
},): readonly Member[] {
  /**
   Raw value the session set, or undefined when it set none.
   */
  const raw = process.env[variable];
  if (raw === undefined)
    return fallback;
  /**
   Parsed value, which must be a JSON array of members.
   */
  const parsed: unknown = JSON.parse(raw,);
  if (!Array.isArray(parsed,))
    throw new Error(`${variable} must hold a JSON array of members`,);
  return parsed.map(memberOf,);
}

/**
 Read the native-only router's target from the environment, falling back when it names none.

 @param variable - environment variable holding `<provider>/<id>`

 @param fallback - member to route to when the variable names none

 @returns member the native-only router sends every request to

 @example
 ```ts
 nativeTargetFromEnvironment({ variable: 'PI_HYBRID_NATIVE_TARGET', fallback: DEFAULT_NATIVE_TARGET });
 ```
 */
function nativeTargetFromEnvironment({
  variable,
  fallback,
}: {
  readonly variable: string;
  readonly fallback: Member;
},): Member {
  /**
   Raw value the session set, or undefined when it set none.
   */
  const raw = process.env[variable];
  if (raw === undefined)
    return fallback;
  /**
   First slash, which separates provider from model id.
   */
  const split = raw.indexOf('/',);
  if (split <= 0)
    throw new Error(`${variable} must hold <provider>/<id>`,);
  if (split === (raw.length
    - 1))
    throw new Error(`${variable} must hold <provider>/<id>`,);
  return {
    provider: raw.slice(
      0,
      split,
    ),
    id: raw.slice(split + 1,),
  };
}

//endregion Reading the session's own choices


//region Extension entry

/**
 Register the package's pool provider, the probe pool, the stand-in members and the virtual models.

 @param pi - Pi's extension API for this session

 @example
 ```ts
 pi -e src/hybrid-prototype/hybrid-extension.ts --model rate-limit-fallback/hybrid-writer
 ```
 */
export default function hybridVirtualModelExtension(pi: ExtensionAPI,): void {
  /**
   Members the probe pool dispatches to, in preference order.
   */
  const probeMembers = membersFromEnvironment({
    variable: PROBE_MEMBERS_VARIABLE,
    fallback: [
      {
        provider: STAND_IN_PROVIDER_ID,
        id: ANSWERING_MEMBER,
      },
    ],
  },);
  /**
   Member the native-only router sends every request to, with no pool in between.
   */
  const nativeTarget = nativeTargetFromEnvironment({
    variable: NATIVE_TARGET_VARIABLE,
    fallback: DEFAULT_NATIVE_TARGET,
  },);
  note({
    event: 'extension-loaded',
    packageProvider: PROVIDER_ID,
    probePool: `${PROTOTYPE_POOL_PROVIDER_ID}/${PROTOTYPE_POOL_ID}`,
    probeMembers,
    nativeTarget,
  },);
  // The package's own extension registers `writer` and `reviewer` beside the prototype's models.
  rateLimitFallbackExtension(pi,);
  pi.registerProvider(standInProvider(),);
  registerPrototypePool({
    host: pi,
    members: probeMembers,
  },);
  registerAlwaysRouter({
    pi,
    id: HYBRID_WRITER_ID,
    name: 'Hybrid writer pool',
    declaredLimits: DECLARED_LIMITS,
    pick(ctx: ExtensionContext,): Model<Api> {
      return resolved({
        registry: ctx.modelRegistry,
        member: {
          provider: PROVIDER_ID,
          id: 'writer',
        },
      },);
    },
  },);
  registerAlwaysRouter({
    pi,
    id: HYBRID_PROBE_ID,
    name: 'Hybrid probe pool',
    declaredLimits: DECLARED_LIMITS,
    pick(ctx: ExtensionContext,): Model<Api> {
      return resolved({
        registry: ctx.modelRegistry,
        member: {
          provider: PROTOTYPE_POOL_PROVIDER_ID,
          id: PROTOTYPE_POOL_ID,
        },
      },);
    },
  },);
  registerAlwaysRouter({
    pi,
    id: HYBRID_NATIVE_PROBE_ID,
    name: 'Hybrid native-only probe',
    declaredLimits: DECLARED_LIMITS,
    pick(ctx: ExtensionContext,): Model<Api> {
      return resolved({
        registry: ctx.modelRegistry,
        member: nativeTarget,
      },);
    },
  },);
  pi.on(
    'session_start',
    function noteStart(
      _event: unknown,
      ctx: ExtensionContext,
    ): void {
      noteContextUsage({
        label: 'session_start',
        ctx,
      },);
    },
  );
  pi.on(
    'turn_end',
    function noteTurn(
      event: TurnMessageView,
      ctx: ExtensionContext,
    ): void {
      /**
       Message Pi finished this turn with.
       */
      const { message, } = event;
      note({
        event: 'turn-end',
        turnIndex: event.turnIndex,
        role: message.role,
        answeredBy: (message.provider === undefined) || (message.model === undefined)
          ? undefined
          : `${message.provider}/${message.model}`,
        stopReason: message.stopReason,
        errorMessage: excerpt(message.errorMessage ?? '',),
        usageInput: message.usage
          ?.input,
        usageOutput: message.usage
          ?.output,
      },);
      noteContextUsage({
        label: `turn_end_${event.turnIndex}`,
        ctx,
      },);
    },
  );
  pi.on(
    'agent_settled',
    function noteSettled(
      _event: unknown,
      ctx: ExtensionContext,
    ): void {
      noteContextUsage({
        label: 'agent_settled',
        ctx,
      },);
    },
  );
}

//endregion Extension entry
