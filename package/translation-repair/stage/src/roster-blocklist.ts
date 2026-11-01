//region Roster blocklist
// MODELS THE OWNER BLOCKLISTED ON 2026-09-01, verbatim authority in
// `doc/decision/translation-repair-roster-blocklist.md`. The candidate set for
// any calibration is the live provider catalogs minus this list, so the list
// is code rather than prose: the catalog drift check labels a blocked model
// BLOCKED instead of suggesting it, and a guard test refuses a compiled
// catalog that seats one.
//
// EXACT IDS FIRST, FAMILY PREDICATES SECOND. The exact ids are the owner's
// entries mapped to every spelling the two providers serve today, aliases
// included, because an alias left unmapped resurfaces the blocked model under
// its other name at the next refresh. The predicates cover only the entries
// the owner phrased as a series or family (Llama, Qwen3.6, Qwen3.7, bare
// Qwen3), so a name the providers add later inside one of those families is
// caught without an edit; entries the owner phrased as single models (Kimi
// K2.5 through K2.7 Code, GLM5, GLM5.1) stay exact, because extending them
// would convert agent inference into owner instruction.

/**
 One blocked model spelling beside the owner's reason for it.

 @example
 ```ts
 const entry: RosterBlocklistEntry = { id: 'qwen3.8-max', reason: 'absurd cost in money', };
 ```
 */
export type RosterBlocklistEntry = {
  /**
   Spelling a provider serves the blocked model under.
   */
  readonly id: string;

  /**
   Owner's stated reason, verbatim from the decision.
   */
  readonly reason: string;
};

/**
 Owner's reason attached to the whole "too outdated" group.
 */
const TOO_OUTDATED = 'too outdated';

/**
 Every blocked spelling either provider serves as of 2026-09-01.

 @example
 ```ts
 const everyEntry = ROSTER_BLOCKLIST;
 ```
 */
export const ROSTER_BLOCKLIST: readonly RosterBlocklistEntry[] = [
  {
    id: 'hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4',
    reason: "can't stick to its own viewpoint",
  },
  {
    id: 'hf:zai-org/GLM-4.7-Flash',
    reason: TOO_OUTDATED,
  },
  {
    // Synthetic serves GLM-4.7-Flash under this alias too.
    id: 'syn:small:text',
    reason: TOO_OUTDATED,
  },
  {
    id: 'hf:zai-org/GLM-5.2',
    reason: TOO_OUTDATED,
  },
  {
    // Hyper's spelling of GLM-5.2.
    id: 'glm-5.2',
    reason: TOO_OUTDATED,
  },
  {
    // Owner's bare "GLM5" names this model, not the series; the reading is
    // recorded in the decision document.
    id: 'glm-5',
    reason: TOO_OUTDATED,
  },
  {
    id: 'glm-5.1',
    reason: TOO_OUTDATED,
  },
  {
    id: 'kimi-k2.5',
    reason: TOO_OUTDATED,
  },
  {
    id: 'kimi-k2.6',
    reason: TOO_OUTDATED,
  },
  {
    id: 'kimi-k2.7-code',
    reason: TOO_OUTDATED,
  },
  {
    id: 'minimax-m2.7',
    reason: TOO_OUTDATED,
  },
  {
    id: 'deepseek-v4-flash',
    reason: 'the undated versions',
  },
  {
    id: 'deepseek-v4-pro',
    reason: 'the undated versions',
  },
  // ADDED 2026-09-16 at the owner's instruction, both providers' spellings:
  // these two had been seated since 2026-08-24 and are superseded by
  // `deepseek-v4.1-flash`.
  {
    id: 'deepseek-v4-pro-0813',
    reason: 'DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash',
  },
  {
    id: 'deepseek-v4-flash-0731',
    reason: 'DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash',
  },
  {
    id: 'deepseek/deepseek-v4-pro-0813',
    reason: 'DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash',
  },
  {
    id: 'deepseek/deepseek-v4-flash-0731',
    reason: 'DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash',
  },
  {
    id: 'qwen3.8-max',
    reason: 'absurd cost in money',
  },
  // ADDED 2026-09-03 with the OpenRouter provider, on the owner's words about
  // this model in the same message that named the OpenRouter allowlist. The
  // `:batch` variant is a second spelling of the same model, so it is listed
  // rather than left to a family rule (doc/decision/translation-repair-openrouter-fallback.md).
  {
    id: 'google/gemini-3.8-flash',
    reason: 'a wildly misaligned model',
  },
  {
    id: 'google/gemini-3.8-flash:batch',
    reason: 'a wildly misaligned model',
  },
];

/**
 Trailing provider-path segment, lowercased, for family matching.

 Both providers embed the model name last: Synthetic as
 `hf:vendor/Name`, Hyper as the bare name.

 @param id - spelling as a provider serves it

 @returns Name segment in lowercase

 @example
 ```ts
 const name = modelNameOf({ id: 'hf:Qwen/Qwen3.6-Plus', },);
 ```
 */
function modelNameOf(
  { id, }: { readonly id: string; },
): string {
  /**
   Where the last path segment starts, which is the model name under both
   spellings: after the last slash, or at the start where there is none.
   */
  const nameStart = id.lastIndexOf('/',) + 1;
  return id.slice(nameStart,)
    .toLowerCase();
}

/**
 Whether the owner blocklist bars one spelling, beside the stated reason.

 @example
 ```ts
 const verdict: BlocklistVerdict = { blocked: true, reason: 'too outdated', };
 ```
 */
export type BlocklistVerdict =
  | {
    /**
     Discriminator marking a spelling the owner barred.
     */
    readonly blocked: true;

    /**
     Owner's stated reason, verbatim from the decision.
     */
    readonly reason: string;
  }
    | {
    /**
     Discriminator marking an eligible spelling.
     */
      readonly blocked: false;
    };

/**
 Owner's verdict on one spelling.

 Family predicates cover the entries the owner phrased as families:
 Llama as a whole, and the Qwen3 line through 3.7 while every Qwen3.8
 variant except the exact-listed Max stays eligible.

 @param id - spelling a provider serves or could serve

 @returns Blocked with the verbatim reason, or eligible

 @example
 ```ts
 const verdict = blocklistVerdictFor({ id: 'llama-5-800b', },);
 ```
 */
export function blocklistVerdictFor(
  { id, }: { readonly id: string; },
): BlocklistVerdict {
  /**
   Exact-spelling entry when the owner named this model singly.
   */
  const exact = ROSTER_BLOCKLIST.find(function sameId(entry,): boolean {
    return entry.id === id;
  },);
  if (exact !== undefined)
    return {
      blocked: true,
      reason: exact.reason,
    };

  /**
   Name half of the spelling, provider prefix removed.
   */
  const name = modelNameOf({ id, },);
  /**
   Whole spelling lowercased, for the vendor-prefixed Llama form.
   */
  const lowered = id.toLowerCase();
  if (name.startsWith('llama',) || lowered.includes('meta-llama/',))
    return {
      blocked: true,
      reason: TOO_OUTDATED,
    };
  if (name.startsWith('qwen3.6',) || name.startsWith('qwen3.7',))
    return {
      blocked: true,
      reason: TOO_OUTDATED,
    };
  // Bare Qwen3-series names (qwen3-coder, qwen3-next) are blocked while
  // qwen3.8 variants stay eligible; the dot after "qwen3" is the boundary.
  if (name.startsWith('qwen3-',))
    return {
      blocked: true,
      reason: TOO_OUTDATED,
    };
  return { blocked: false, };
}

//endregion Roster blocklist
