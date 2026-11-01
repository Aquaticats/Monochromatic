import { rm, } from 'node:fs/promises';
import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Draw outputs
// The lifetime of the files one draw writes.
//
// A draw produces three outputs that must exist together to mean anything: the
// detection sheet, the repair sheet, and the manifest joining sheet positions
// back to issue ids. They are written one after another, and a FINAL path is
// refused once it exists, which is what protects graded work from a repeated
// draw.
//
// Those two facts combine badly on failure. A fault between writes leaves a
// partial set, and the refusal then blocks the next draw on files nobody ever
// graded. Removing what the failed invocation created is what returns the
// directory to a state a draw can be retried from, and it can never remove
// graded work, because a graded sheet is one an earlier invocation committed.
//
// TRACKING IS FOR FINAL DRAWS ONLY, and that is a correctness requirement
// rather than an optimization. A final output is created with `wx`, so its path
// did not exist a moment earlier and removing it can only remove what this
// invocation made. A preliminary output is written with `w`, which REPLACES
// whatever was there, so removing one on failure would delete a file this
// invocation never created. Preliminary draws need no protection anyway: they
// carry no overwrite guard, so a partial one is replaced by the next draw
// rather than blocking it.
//
// Paths are recorded BEFORE their write, not after. A write can create or
// truncate a file and then fail, and a path recorded only on success would
// leave exactly that file behind. The one failure that created nothing is the
// exclusive create finding the path taken (two draws racing, or a link left
// at the path): that path is released, so the file standing there, which no
// draw of this invocation made, is not removed with the rest.

/**
 Files a draw has written, and whether it finished writing all of them.

 @example
 ```ts
 await using outputs: DrawOutputs = trackDrawOutputs();
 ```
 */
export type DrawOutputs = AsyncDisposable & {
  /**
   Notes a file this draw created.
   */
  readonly record: ({ path, }: { readonly path: string; },) => void;

  /**
   Forgets a file this draw recorded but did not create, so it is not removed
   on the way out: an exclusive create that found the path already there
   created nothing, and the file standing there is someone else's.
   */
  readonly release: ({ path, }: { readonly path: string; },) => void;

  /**
   Marks the set complete, so nothing is removed on the way out.
   */
  readonly commit: () => void;
};

/**
 Tracks the files a draw writes and removes them unless it finished.

 @param enabled - whether this draw's outputs are exclusively created, which
 is true of a final draw and false of a preliminary one; a disabled tracker
 records nothing and removes nothing

 @returns Tracker to be held with `await using`

 @example
 ```ts
 await using outputs = trackDrawOutputs({ enabled: isFinal, },);
 outputs.record({ path: sheetPath, },);
 outputs.commit();
 ```
 */
export function trackDrawOutputs(
  { enabled, }: { readonly enabled: boolean; },
): DrawOutputs {
  /**
   Paths written so far, and whether the set completed.
   */
  const state: {
    paths: string[];
    committed: boolean;
  } = {
    paths: [],
    committed: false,
  };
  return {
    record({ path, }: { readonly path: string; },): void {
      if (!enabled)
        return;
      state.paths
        .push(path,);
    },
    release({ path, }: { readonly path: string; },): void {
      state.paths = state.paths
        .filter(function isOtherPath(recorded,): boolean {
          return recorded !== path;
        },);
    },
    commit(): void {
      state.committed = true;
    },
    async [Symbol.asyncDispose](): Promise<void> {
      if (state.committed)
        return;
      // `force` so removing a path the failing write never created is not
      // itself a failure, which is the ordinary case: the write that threw is
      // the one whose file may be absent.
      await allInInputOrder({
        members: state.paths
          .map(function removeOne(path,) {
            return rm(
              path,
              { force: true, },
            );
          },),
      },);
    },
  };
}

//endregion Draw outputs
