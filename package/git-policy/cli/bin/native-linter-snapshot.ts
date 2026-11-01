/**
 Snapshot inputs the native wrapper gained by linking the repository's Markdown linter library:
 the linter crate's manifest and sources, copied beside the wrapper like the scanner crate.
 The linter is a path dependency of the wrapper (without its default `semantic` feature), so the wrapper's own
 lockfile names every registry crate it pulls in and the wrapper's single vendoring step covers them. There is no
 linter executable, no separate build stage and no variable naming one: the wrapper's tests run the linked linter.
 */
import {
  join,
  resolve,
} from 'node:path';

/**
 One file or directory to copy from the repository into the build context.
 */
export type SnapshotEntry = {
  readonly from: string;
  readonly to: string;
};

/**
 The linter crate's directory under `package`.
 */
const linterCrate = 'linter/monochromatic-lint';

/**
 Copy entries for the linked linter, resolved from the wrapper package directory the gate runs in.

 @param context - build context directory receiving the repository-shaped snapshot

 @returns entries keeping the repository layout, so the wrapper's relative path dependency resolves;
   a library build of the linter reads only its manifest and `src`, and the dependency layer's stub builder copies
   every linked crate's `Cargo.lock` beside its manifest, so the lockfile is copied too

 @example
 ```ts
 const entries = linterSnapshotEntries({ context });
 ```
 */
export function linterSnapshotEntries({ context, }: {
  readonly context: string;
},): readonly SnapshotEntry[] {
  return [
    'Cargo.toml',
    'Cargo.lock',
    'src',
  ].map(function linterEntry(name) {
    return {
      from: resolve(
        '../..',
        linterCrate,
        name,
      ),
      to: join(
        context,
        'package',
        linterCrate,
        name,
      ),
    };
  });
}
