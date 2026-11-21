/**
 The one list of crate sources every container snapshot of the native wrapper copies: the wrapper itself and each
 path dependency it links, transitively.

 The gate and the end-to-end suite each build an image from a repository-shaped
 snapshot. Each used to carry its own copy of this list, so a new path dependency reached only the snapshots
 somebody remembered to edit. Now they all call `copyWrapperSnapshot`, and each adds only what is specific to it.
 `native-wrapper-snapshot.unit.test.ts` reads the manifests and fails when this list misses a linked crate.
 */
import { cp, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { NativeVerificationError, } from './native-verification-process.ts';

/**
 One file or directory to copy from the repository into the build context.
 */
export type SnapshotEntry = {
  readonly from: string;
  readonly to: string;
};

/**
 Crate the harnesses test, as a directory below `package`.
 */
export const subjectCrate = 'git-policy/cli';

/**
 Path crates the subject links, as directories below `package`: the JSONC crate, the catalog specifier crate, the
 scanner with its engine, the Markdown linter library and the ZFS binding the root-run clone-mount handler makes its
 `/dev/zfs` requests through.
 `native-wrapper-snapshot.unit.test.ts` checks this list against the path dependencies the manifests declare.
 */
export const pathCrates = [
  'jsonc-edit/core-rust',
  'catalog-specifier/core-rust',
  'forbidden-strings/cli',
  'forbidden-strings/core',
  'linter/monochromatic-lint',
  'zfs/request-to-openzfs-kernel-device',
] as const;

/**
 Every crate whose manifest the dependency layer reads.
 */
export const wrapperCrates: readonly string[] = [
  subjectCrate,
  ...pathCrates,
];

/**
 Entries of each crate a library build reads, keyed by crate so that a path crate added to `pathCrates` without
 entries here fails type-checking.
 The scanner's `build.rs` and `data` hold the baseline its library compiles in, and the dependency layer's stub builder
 copies every linked crate's `Cargo.lock` beside its manifest.
 A file a test reads at run time is part of this list too, or the control that reads it fails in the image while the
 path is there on a checkout: `README.md` is included by a test, and `SPEC.md` is read by the control that holds the
 longest runtime directory the prose states to the number the function returns.
 */
const crateEntryNames = {
  'git-policy/cli': [
    'Cargo.toml',
    'Cargo.lock',
    'README.md',
    'SPEC.md',
    'src/native',
  ],
  'jsonc-edit/core-rust': [
    'Cargo.toml',
    'Cargo.lock',
    'src',
    'fixtures',
  ],
  'catalog-specifier/core-rust': [
    'Cargo.toml',
    'Cargo.lock',
    'src',
    'fixtures',
  ],
  'forbidden-strings/cli': [
    'Cargo.toml',
    'Cargo.lock',
    'build.rs',
    'src',
    'data',
  ],
  'forbidden-strings/core': [
    'Cargo.toml',
    'Cargo.lock',
    'src',
  ],
  'linter/monochromatic-lint': [
    'Cargo.toml',
    'Cargo.lock',
    'src',
  ],
  'zfs/request-to-openzfs-kernel-device': [
    'Cargo.toml',
    'Cargo.lock',
    'src',
  ],
} as const satisfies Readonly<Record<typeof pathCrates[number] | typeof subjectCrate, readonly string[]>>;

/**
 The same table looked up by a crate directory read from `wrapperCrates`.
 */
const namesByCrate: ReadonlyMap<string, readonly string[]> = new Map(Object.entries(crateEntryNames));

/**
 Directory holding every package of the repository, found from this file so the entries never depend on the
 directory the script runs in.
 */
const packageRoot = resolve(
  import.meta.dirname,
  '../../..',
);

/**
 Copy entries for the wrapper and every path crate it links.

 @param context - build context directory receiving the repository-shaped snapshot

 @returns entries keeping the repository layout, so the manifests' relative path dependencies resolve

 @example
 ```ts
 const entries = wrapperSnapshotEntries({ context });
 ```
 */
export function wrapperSnapshotEntries({ context, }: {
  readonly context: string;
},): readonly SnapshotEntry[] {
  return wrapperCrates.flatMap(function entriesOf(crate) {
    // A crate outside the table is a type error for the two exported names and a thrown error for a widened list.
    /**
     Entries listed for the crate.
     */
    const names = namesByCrate.get(crate);
    if (names === undefined)
      throw new NativeVerificationError(`No snapshot entries are listed for crate ${crate}.`);
    return names.map(function entryOf(name) {
      return {
        from: resolve(
          packageRoot,
          crate,
          name,
        ),
        to: join(
          context,
          'package',
          crate,
          name,
        ),
      };
    });
  });
}

/**
 Copy entries into the build context.
 Copies run concurrently, and every copy settles before this returns or throws,
 so removing the context afterwards never races an in-flight copy.

 @param context - build context directory the entries land in, named in the error

 @param entries - files and directories to copy, directories recursively

 @throws NativeVerificationError when any copy failed, with every failure as its cause

 @example
 ```ts
 await copySnapshotEntries({ context, entries });
 ```
 */
export async function copySnapshotEntries({
  context,
  entries,
}: {
  readonly context: string;
  readonly entries: readonly SnapshotEntry[];
},): Promise<void> {
  /**
   Result of every copy; a failed copy never stops the others, so none is still in flight afterwards.
   */
  const copies = await Promise.allSettled(entries.map(function copyEntry(entry) {
    return cp(
      entry.from,
      entry.to,
      {
        recursive: true,
        // A link copied verbatim keeps its own relative target, so it resolves inside the snapshot it lands in. Node's
        // default rewrites a link to the absolute path it resolved to on this host, and a `uses/` link then names a
        // folder the image does not hold (`AGENTS.md`, rule LYD: every source folder carries one).
        verbatimSymlinks: true,
      },
    );
  }));
  /**
   Reasons of the copies that failed.
   */
  const failures: unknown[] = [];
  for (const copy of copies) {
    if (copy.status === 'rejected')
      failures.push(copy.reason);
  }
  if (failures.length > 0)
    throw new NativeVerificationError(
      `Copying ${String(failures.length)} snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}

/**
 Copy the wrapper, its path crates and the entries one harness adds into the build context.

 @param context - build context directory receiving the repository-shaped snapshot

 @param additionalEntries - entries only the calling harness needs, such as a configuration file or its drivers

 @example
 ```ts
 await copyWrapperSnapshot({ context, additionalEntries: [driverEntry] });
 ```
 */
export async function copyWrapperSnapshot({
  context,
  additionalEntries = [],
}: {
  readonly context: string;
  readonly additionalEntries?: readonly SnapshotEntry[];
},): Promise<void> {
  await copySnapshotEntries({
    context,
    entries: [
      ...wrapperSnapshotEntries({ context }),
      ...additionalEntries,
    ],
  });
}
