/**
 Git version comparison and replay-plumbing applicability for scenarios.

 @module
 */

/**
 Oldest Git whose `git merge-tree` accepts `--merge-base`,
 the replay plumbing (`Documentation/RelNotes/2.40.0.adoc`).
 */
export const REPLAY_PLUMBING_GIT = '2.40.0';

/**
 How a scenario depends on replay plumbing:
 `required` needs it,
 `unused` never loses a landing race,
 and `absent` proves the fail-fast degradation without it.
 */
export type ReplayPlumbingDependency = 'absent' | 'required' | 'unused';

/**
 A scenario's replay-plumbing dependency lets it run on the Git under test.
 */
export const REPLAY_PLUMBING_FITS: unique symbol = Symbol('scenario fits Git replay plumbing',);

/**
 Compares dotted Git versions.

 @param left - version

 @param right - version

 @returns negative, zero, or positive

 @example
 ```ts
 compareVersions({ left: '2.40.0', right: '2.54.0' }) < 0; // => true
 ```
 */
export function compareVersions({
  left,
  right,
}: Readonly<{
  left: string;
  right: string;
}>,): number {
  /**
   Numeric parts of both versions.
   */
  const [leftParts, rightParts,] = [
    left,
    right,
  ].map(function parts(version,) {
    return version.split('.',)
      .map(Number,);
  },);
  return (leftParts ?? []).reduce(
    function firstDifference(
      difference,
      part,
      index,
    ) {
      return difference === 0 ? part - ((rightParts ?? [])[index] ?? 0) : difference;
    },
    0,
  );
}

/**
 Explains why a scenario's replay-plumbing dependency excludes a Git version.

 @param replayPlumbing - scenario dependency

 @param gitVersion - Git version under test

 @returns skip reason, or {@link REPLAY_PLUMBING_FITS} when the scenario runs

 @example
 ```ts
 replayPlumbingSkip({ replayPlumbing: 'required', gitVersion: '2.39.5' }); // => 'requires replay plumbing (Git 2.40.0)'
 ```
 */
export function replayPlumbingSkip({
  replayPlumbing,
  gitVersion,
}: Readonly<{
  replayPlumbing: ReplayPlumbingDependency;
  gitVersion: string;
}>,): string | typeof REPLAY_PLUMBING_FITS {
  /**
   Whether this Git has `git merge-tree --merge-base`.
   */
  const available = compareVersions({
    left: gitVersion,
    right: REPLAY_PLUMBING_GIT,
  },) >= 0;
  if ((replayPlumbing === 'required') && (!available))
    return `requires replay plumbing (Git ${REPLAY_PLUMBING_GIT})`;
  if ((replayPlumbing === 'absent') && available)
    return `proves degradation only before Git ${REPLAY_PLUMBING_GIT}`;
  return REPLAY_PLUMBING_FITS;
}
