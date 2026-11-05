/**
 Classifies Git administrative entries that must not survive a finished run
 (`SPEC.md` "Transaction directory and journal"):
 lock files and lock directories,
 client scratch directories of the commit service route,
 published or staging transaction directories under `cli-git-transactions/`,
 the legacy single `cli-git-transaction` directory,
 shadow repositories under `monochromatic-git-policy-cli/shadow/` (or `cli-git/shadow/` from a program before the rename),
 and capture-store entries other than the worktree identity, the sequence, and the landed-record directory,
 including every landed-capture record (`SPEC.md` "Capture order"),
 which the last transaction to finish prunes.
 The persistent `cli-git-transactions/`, `monochromatic-git-policy-cli/shadow/`, and `cli-git-captures/` roots themselves are not leftovers.

 @module
 */

//region Classification

/**
 Capture-store entries that persist between transactions.
 */
const PERSISTENT_CAPTURE_ENTRIES: ReadonlySet<string> = new Set([
  'worktree-id',
  'sequence',
  'landed',
],);

/**
 Name prefix of the scratch directory a commit client prepares in (`cli-git-service-<id>`).
 A client removes it when it ends,
 so one that remains belongs to a client that died.
 */
const CLIENT_SCRATCH_PREFIX = 'monochromatic-git-policy-cli-service-';

/**
 Name prefix of the scratch directory a commit client of a program before the rename prepared in.
 */
const EARLIER_CLIENT_SCRATCH_PREFIX = 'cli-git-service-';

/**
 Returns the leftover root an entry belongs to.

 @param entry - path relative to the Git common directory, `/`-separated

 @returns the leftover root path, or nothing when the entry is not leftover state

 @example
 ```ts
 leftoverRoot('cli-git-transactions/0a1b/owner.json'); // => ['cli-git-transactions/0a1b']
 ```
 */
function leftoverRoot(entry: string,): readonly string[] {
  /**
   Path segments.
   */
  const segments = entry.split('/',);
  if (segments[0] === 'objects')
    return [];
  /**
   First segment that roots leftover state, with the segment count the root spans.
   */
  const rootEnd = segments.findIndex(function rootsLeftover(
    segment,
    index,
  ) {
    return segment.endsWith('.lock',)
      || segment.startsWith(CLIENT_SCRATCH_PREFIX,)
      || segment.startsWith(EARLIER_CLIENT_SCRATCH_PREFIX,)
      || (segment === 'cli-git-transaction')
      || (segments[index - 1] === 'cli-git-transactions')
      || ((segments[index - 1] === 'shadow') && ((segments[index - 2] === 'monochromatic-git-policy-cli') || (segments[index - 2] === 'cli-git')))
      || ((segments[index - 1] === 'cli-git-captures') && (!PERSISTENT_CAPTURE_ENTRIES.has(segment,)))
      || ((segments[index - 1] === 'landed') && (segments[index - 2] === 'cli-git-captures'));
  },);
  return rootEnd === (-1) ? [] : [segments.slice(
    0,
    rootEnd + 1,
  )
    .join('/',),];
}

/**
 Reduces a recursive listing of the Git common directory to distinct leftover roots.

 @param entries - paths relative to the common directory

 @returns sorted distinct leftover roots

 @example
 ```ts
 classifyLeftovers(['index.lock', 'cli-git-transactions', 'objects/pack/x.lock']); // => ['index.lock']
 ```
 */
export function classifyLeftovers(entries: readonly string[],): readonly string[] {
  return [
    ...new Set(entries.flatMap(leftoverRoot,),),
  ].toSorted(function byPath(
    left,
    right,
  ) {
    return left.localeCompare(right,);
  },);
}

//endregion Classification
