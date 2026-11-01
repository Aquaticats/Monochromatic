import {
  type EligibleEntries,
  selectEligible,
} from './artifact-eligible.ts';
import {
  type ArtifactFileName,
  entryIdOfArtifact,
} from './artifact-file-name.ts';
import {
  censusByGeneration,
  resolveCommit,
} from './artifact-generation.ts';
import {
  digestPipeline,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact pool
// One call for every reader that turns settled artifacts into a NUMBER.
//
// Readers split into two kinds and only one of them belongs here. A scheduler
// asking "which entries already have an artifact" must see all of them, or the
// pass re-runs settled work; `corpus-pass` reads the directory unfiltered for
// exactly that reason and is correct to. A reader producing a rate must see one
// generation, or the rate describes no pipeline that ever existed.
//
// The policy comes from the environment rather than from a flag on each script,
// because these are operational runners invoked by hand and the alternative is
// remembering to pass the same value to four of them. The ENTRY reads it
// (`readPoolPolicy`) and hands it down as a value, so no library caller of
// `resolvePool` pools under whatever the shell running it exported; until
// 2026-10-10 `resolvePool` read the environment itself.

/**
 Environment variable naming the commit an eligible pipeline must contain.
 */
const REQUIRED_COMMIT_VAR = 'TRANSLATION_REPAIR_REQUIRED_COMMIT';

/**
 Environment variable opting into a deliberately mixed pool.
 */
const POOL_ALL_VAR = 'TRANSLATION_REPAIR_POOL_ALL';

/**
 Value that opts into a mixed pool, spelled out so a stray `0` or empty string
 cannot silently disable the guard.
 */
const POOL_ALL_VALUE = 'yes';

/**
 Generation policy the invoker set, read by an entry file and handed to every
 reader that pools settled artifacts into a rate.

 @example
 ```ts
 const policy: PoolPolicy = { requiredCommit: '', poolAll: false, };
 ```
 */
export type PoolPolicy = {
  /**
   Commit an eligible entry's pipeline must contain, as the invoker wrote it;
   empty where none was set.
   */
  readonly requiredCommit: string;

  /**
   Whether the invoker asked for every generation at once.
   */
  readonly poolAll: boolean;
};

/**
 Reads the generation policy out of the environment an entry runs in.

 An exported-but-empty required commit is an ordinary shell accident, so it
 is folded together with absence rather than read as a requirement nobody
 can satisfy; the pool-all request is its one accepted value, spelled out so
 a stray `0` or empty string cannot silently disable the guard.

 @param env - environment of the process the entry runs in

 @returns The policy, read and not yet checked for a contradiction, which
 `resolvePool` refuses

 @example
 ```ts
 const policy = readPoolPolicy({ env: process.env, },);
 ```
 */
export function readPoolPolicy({ env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },): PoolPolicy {
  /**
   Both variables as the invoker set them.
   */
  const {
    [REQUIRED_COMMIT_VAR]: requiredCommit,
    [POOL_ALL_VAR]: poolAll,
  } = env;
  return {
    requiredCommit: requiredCommit ?? '',
    poolAll: poolAll === POOL_ALL_VALUE,
  };
}

/**
 Resolves which settled entries this reader may pool, and prints the census.

 Printing is not optional and not the caller's choice. A rate over a filtered
 pool is only readable beside the lines saying what was filtered, and leaving
 that to each call site is how one of them ends up silently omitting it.

 @param artifactsDir - directory holding one JSON per settled entry

 @param names - the caller's own listing of that directory, censused in
 place of a second listing where given

 @param policy - generation policy the entry read (`readPoolPolicy`)

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @returns Eligible entries, what was excluded, and the printed report

 @throws StatedRefusalError when the policy requires a commit and asks for
 every generation at once

 @throws MixedGenerationError when the directory spans pipeline generations
 and neither `TRANSLATION_REPAIR_REQUIRED_COMMIT` nor
 `TRANSLATION_REPAIR_POOL_ALL=yes` was set

 @example
 ```ts
 const pool = await resolvePool({ artifactsDir, policy: readPoolPolicy({ env: process.env, },), repository: import.meta.dirname, },);
 ```

 @internal
 */
export async function resolvePool(
  {
    artifactsDir,
    names,
    policy,
    repository,
  }: {
    readonly artifactsDir: string;
    readonly names?: readonly ArtifactFileName[];
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<EligibleEntries> {
  /**
   Required commit as the invoker wrote it, empty when none was set.
   */
  const required = policy.requiredCommit;

  // Two different pools asked for at once. Preferring either silently records a
  // policy nobody chose, and the report printed above the resulting number
  // would name that policy as though it had been requested. The operator's to
  // mend, so stated: as a plain `Error` it printed as a fault of the command,
  // its message dropped, until 2026-10-10.
  if ((required !== '') && policy.poolAll)
    throw new StatedRefusalError({
      says: `${REQUIRED_COMMIT_VAR} and ${POOL_ALL_VAR} are both set, which asks for `
        + 'a filtered pool and an unfiltered one at the same time.\n'
        + 'Unset whichever was not meant. A required commit selects entries '
        + 'whose pipeline contains it; pooling all takes every generation and '
        + 'says so above the number.',
    },);

  /**
   Settled entries partitioned by the built pipeline each recorded.

   Given the caller's own listing when it has one, so census and reader
   classify the same files rather than two views of a directory the
   accumulation is still writing into.
   */
  const census = await censusByGeneration({
    artifactsDir,
    ...((names === undefined) ? {} : { names, }),
  },);

  // Resolved once, here, rather than passed through as whatever the shell said.
  // Used raw it accepted `HEAD` and branch names, which resolve against the
  // READER's checkout at read time: two readers filtering the same directory
  // could cover different entries while both reporting the same requirement,
  // and a branch that moves changes a rate's population with nothing recording
  // that it did. Artifact tips are already held to canonical ids.
  /**
   Required commit as a full object id, empty when none was set.
   */
  const commit = (required === '')
    ? ''
    : await resolveCommit({
      revision: required,
      repository,
    },);

  if ((commit !== '') && (commit !== required))
    console.log(`POOL required commit ${required} resolves to ${commit}`,);

  /**
   Entries this reader may pool.
   */
  const eligible = await selectEligible({
    census,
    ...((commit === '') ? {} : { requiredCommit: commit, }),
    pooledDeliberately: policy.poolAll,
    repository,
  },);

  // The reader's OWN identity, which the artifacts cannot carry. Every artifact
  // records the pipeline that produced it, and a rate is a function of the
  // reader as well: change how a report counts and the number moves with no
  // artifact touched. Printed beside the pool rather than stored, because it
  // describes this invocation rather than the entries.
  /**
   Built pipeline this reader is running.
   */
  const reader = await digestPipeline({ dir: import.meta.dirname, },);

  console.log(`POOL read by pipeline ${reader.digest}`,);

  for (const line of eligible.report)
    console.log(line,);

  return eligible;
}

/**
 Keeps only the artifact file names an eligible entry owns.

 Exported through the barrel so the built bundle's tests can hand it a pool
 directly; the four readers are its callers.

 @internal

 @param names - artifact file names as read from disk

 @param eligible - resolved pool

 @returns Names belonging to eligible entries, order preserved

 @example
 ```ts
 const kept = keepEligible({ names, eligible, },);
 ```
 */
export function keepEligible(
  {
    names,
    eligible,
  }: {
    readonly names: readonly ArtifactFileName[];
    readonly eligible: EligibleEntries;
  },
): readonly ArtifactFileName[] {
  /**
   Ids this reader must still see: everything eligible, plus every artifact
   that would not parse.

   Malformed files are carried through rather than filtered because the reader
   downstream is the one that reports them, and dropping them here would make
   a corrupt artifact vanish from the failure list instead of appearing on it.
   */
  const allowed = new Set([
    ...eligible.entryIds,
    ...eligible.malformedIds,
  ],);

  return names.filter(function isEligible(name,): boolean {
    return allowed.has(entryIdOfArtifact({ name, },),);
  },);
}

//endregion Artifact pool
