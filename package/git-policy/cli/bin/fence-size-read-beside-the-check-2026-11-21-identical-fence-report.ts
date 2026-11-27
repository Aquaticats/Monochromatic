/**
 What the 2026-11-21 control on the overlap's answer reads: the fence decision each arm printed, and whether the two arms
 made the same one.

 The control adds one small fixture twice with the same binary, once with the size read started beside the source check and
 once without, and the fault it looks for is a fence decision that differs between the two. This module is the judgement a
 unit test can hold; building the fixture and driving the adds stays in the script.
 */

/**
 The words the add prints in front of the fence it chose, which is where the decision begins in a verbose line.
 */
const fenceMarker = 'the fence carries';

/**
 What one arm's add left behind, as its own line and its destination read it.
 */
export type FenceReading = {
  /**
   The line of the add's standard error that carries the fence, or the words saying it printed none.
   */
  readonly fence: string;
  /**
   The destination's `git status --porcelain`, which says which worktree the add wrote.
   */
  readonly status: string;
  /**
   The route the add's own record names, or nothing when its record names none.
   */
  readonly route?: string;
  /**
   The fence lines of the add's verbose output, which name how it reached the fence.
   */
  readonly verbose: readonly string[];
  /**
   The steps the add's own record carries, which say what it spent.
   */
  readonly carried: Readonly<Record<string, unknown>>;
};

/**
 One entry of a destination: its path inside the destination and its bytes, base64-encoded.
 */
export type FenceTreeEntry = readonly [
  string,
  string
];

/**
 The fence decision a verbose line of an add carries.

 @param line - the add's line, or the words saying it printed no fence line

 @returns the decision from the marker onwards, without the space around it

 @example
 ```ts
 fenceOf({ line: '… the fence carries 40 paths itself' }); // 'the fence carries 40 paths itself'
 ```
 */
export function fenceOf({ line, }: { readonly line: string; }): string {
  /**
   Where the decision begins, or nothing when the line does not carry one.
   */
  const at = line.indexOf(fenceMarker,);
  return (at === (-1) ? line : line.slice(at,)).trim();
}

/**
 What the control printed: each arm's fence and destination, and whether the two arms agreed.

 @param serial - the arm whose size read runs before it reaches the fence

 @param overlap - the arm whose size read starts beside the source check

 @param serialTree - the files the serial arm's destination holds

 @param overlapTree - the files the overlap arm's destination holds

 @returns the report, one entry per arm and the two comparisons

 @example
 ```ts
 identicalFenceReport({ serial, overlap, serialTree, overlapTree });
 ```
 */
export function identicalFenceReport({
  serial,
  overlap,
  serialTree,
  overlapTree,
}: {
  readonly serial: FenceReading;
  readonly overlap: FenceReading;
  readonly serialTree: readonly FenceTreeEntry[];
  readonly overlapTree: readonly FenceTreeEntry[];
}): Record<string, unknown> {
  /**
   The decision the serial arm made.
   */
  const serialDecision = fenceOf({ line: serial.fence, },);
  /**
   The decision the overlap arm made.
   */
  const overlapDecision = fenceOf({ line: overlap.fence, },);
  return {
    serial: {
      fence: serialDecision,
      status: serial.status
        .trim(),
      route: serial.route ?? null,
      verbose: serial.verbose,
    },
    overlap: {
      fence: overlapDecision,
      status: overlap.status
        .trim(),
      route: overlap.route ?? null,
      verbose: overlap.verbose,
    },
    sameTrees: JSON.stringify(serialTree,) === JSON.stringify(overlapTree,),
    files: serialTree.length,
    sameDecision: serialDecision === overlapDecision,
  };
}
