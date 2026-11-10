/**
 Recursion guard for the chain of Claude sessions `spawn-claude` starts.
 
 AGENTS.md rule SPG requires automation that spawns agent sessions to carry an
 explicit recursion guard. `spawn-claude` is that automation: Every session it
 launches inherits the CLI on its PATH, so one runaway turn can open terminal
 windows generation after generation, each holding its own Claude process and
 spawn-state record, until something else stops it. Nothing did.
 
 The guard counts generations with {@link SPAWN_DEPTH_ENV}, which the CLI writes
 on every child it launches and the next `spawn-claude` call in that session
 reads back. The marker travels on the same channel `CLAUDE_SPAWN_ID` already
 uses: the CLI builds the child's environment and the launcher `execvp`s into
 the terminal emulator, so a session that can claim its own spawn record can
 also read its depth. The CLI decides the guard before it writes that record, so
 a refused attempt opens no window and leaves no state behind.
 
 @module
 */

import type { Environment, } from './paths.ts';

//region Chain depth marker

/**
 Environment variable carrying how many spawn generations sit above a session.
 
 The CLI sets it on every child, so a session at the top of a chain has no value
 and each generation below it adds one.
 */
const SPAWN_DEPTH_ENV = 'CLAUDE_SPAWN_DEPTH' as const;

/**
 Depth marker that is not a count of spawn generations.
 
 A unique symbol rather than a sentinel number, because the caller refuses for a
 different reason and with a different remedy than reaching the limit.
 */
const UNREADABLE_SPAWN_DEPTH: unique symbol = Symbol(
  'claude-spawn/spawn-depth-marker-unreadable',
);

/**
 Reads how many spawn generations deep the calling session already is.
 
 A blank value counts as the top of a chain, which is what an unset variable
 means; a value that is not a whole count is reported as unreadable rather than
 rounded, so a corrupt marker cannot silently reset the guard.
 
 @param rawSetting - value read from {@link SPAWN_DEPTH_ENV}, blank when unset
 
 @returns generation count, or {@link UNREADABLE_SPAWN_DEPTH}
 
 @example
 ```ts
 spawnDepth(''); // 0
 spawnDepth('2'); // 2
 ```
 */
function spawnDepth(rawSetting: string,): number | typeof UNREADABLE_SPAWN_DEPTH {
  /**
   Marker text without the surrounding whitespace a shell assignment may leave.
   */
  const trimmed = rawSetting.trim();

  if (trimmed === '')
    return 0;

  /**
   Parsed generation count; `Number` answers `NaN` for words and `Infinity`.
   */
  const parsed = Number(trimmed,);

  if ((!Number.isInteger(parsed,))
    || (parsed < 0))
    return UNREADABLE_SPAWN_DEPTH;

  return parsed;
}

//endregion Chain depth marker

//region Chain limit

/**
 Environment variable overriding {@link DEFAULT_MAX_SPAWN_DEPTH}.
 */
const MAX_SPAWN_DEPTH_ENV = 'MONOCHROMATIC_SPAWN_MAX_DEPTH' as const;

/**
 Spawn generations allowed below the first session before the guard refuses.
 
 Three, because the plugin exists to run work in parallel and each generation
 below the first is an agent delegating what it was asked to do itself. A loop
 that never stops costs one terminal window, one Claude process, and one spawn
 record per hop, so the cap sits far below the depth at which such a loop
 becomes expensive, while leaving room for a deliberate two-level delegation.
 Sessions that need a longer chain raise the limit with
 {@link MAX_SPAWN_DEPTH_ENV}.
 */
const DEFAULT_MAX_SPAWN_DEPTH = 3;

/**
 Resolves the configured spawn generation limit.
 
 An unset, blank, or unparsable setting falls back to
 {@link DEFAULT_MAX_SPAWN_DEPTH}: a typo must not remove the guard, and a limit
 below one would refuse even the first child.
 
 @param rawSetting - value read from {@link MAX_SPAWN_DEPTH_ENV}, blank when unset
 
 @returns configured limit, or {@link DEFAULT_MAX_SPAWN_DEPTH}
 
 @example
 ```ts
 maxSpawnDepth('6'); // 6
 maxSpawnDepth(''); // 3
 ```
 */
function maxSpawnDepth(rawSetting: string,): number {
  /**
   Parsed limit; `Number` rejects blanks and words as `NaN`.
   */
  const parsed = Number(rawSetting.trim(),);

  if ((!Number.isInteger(parsed,))
    || (parsed < 1))
    return DEFAULT_MAX_SPAWN_DEPTH;

  return parsed;
}

//endregion Chain limit

//region Spawn decision

/**
 Attempt the guard allows.
 */
type SpawnAllowed = {
  /**
   Spawn generation the child occupies, counted from the first session.
   */
  readonly childDepth: number;
};

/**
 Attempt the guard refuses.
 */
type SpawnRefused = {
  /**
   Text explaining the refusal and every way out of it.
   */
  readonly diagnostic: string;
};

/**
 Outcome of the recursion guard for one spawn attempt.
 
 The two shapes carry different values on purpose: an allowed attempt only needs
 the generation it creates, and a refused one only needs the text the person
 reads. Callers narrow on {@link SpawnRefused.diagnostic}, the field only a
 refusal has.
 */
type SpawnChainDecision = SpawnAllowed | SpawnRefused;

/**
 Decides whether a spawn attempt may proceed, and what generation it creates.
 
 A session at or past the limit is refused. So is a session whose depth marker
 does not parse, because the chain length is then unknown, and reading that as
 the top of a chain would let one corrupt variable reset the guard for every
 generation below it.
 
 @param env - environment of the calling session
 
 @returns the child's generation, or the diagnostic to refuse with
 
 @example
 ```ts
 spawnChainDecision({}); // { childDepth: 1 }
 ```
 */
function spawnChainDecision(env: Environment,): SpawnChainDecision {
  /**
   Marker text this session inherited from the CLI that launched it.
   */
  const rawDepth = env[SPAWN_DEPTH_ENV]
    ?? '';
  /**
   Generation count the marker carries, or the unreadable sentinel.
   */
  const depth = spawnDepth(rawDepth,);
  /**
   Limit this chain may reach, from the override or the default.
   */
  const limit = maxSpawnDepth(env[MAX_SPAWN_DEPTH_ENV]
    ?? '',);

  if (depth === UNREADABLE_SPAWN_DEPTH) {
    return {
      diagnostic: [
        `spawn-claude: refusing to spawn: ${SPAWN_DEPTH_ENV} holds ${JSON.stringify(rawDepth,)},`,
        'which is not a count of spawn generations, so the recursion guard cannot',
        'tell how deep this chain is. No terminal was opened.',
        `Unset ${SPAWN_DEPTH_ENV} in the shell that runs spawn-claude; the CLI sets it`,
        'itself on the sessions it launches.',
      ]
        .join(' ',),
    };
  }

  if (depth >= limit) {
    return {
      diagnostic: [
        `spawn-claude: refusing to spawn: this session is ${String(depth,)} spawn generations deep`,
        `and the limit is ${String(limit,)}. No terminal was opened.`,
        'The guard bounds how deep a chain of spawned sessions may go, not how wide.',
        `Raise the limit with ${MAX_SPAWN_DEPTH_ENV}=<generations> in the environment`,
        'of the session that spawns, or do the work in this session.',
      ]
        .join(' ',),
    };
  }

  return { childDepth: depth + 1, };
}

//endregion Spawn decision

export {
  DEFAULT_MAX_SPAWN_DEPTH,
  MAX_SPAWN_DEPTH_ENV,
  maxSpawnDepth,
  SPAWN_DEPTH_ENV,
  spawnChainDecision,
  spawnDepth,
};

export type {
  SpawnAllowed,
  SpawnChainDecision,
  SpawnRefused,
};
