/**
 The source snapshot the native wrapper gate builds its test image from.
 */
import { cp, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { linterSnapshotEntries, } from './native-linter-snapshot.ts';
import {
  scannerSnapshotEntries,
  type SnapshotEntry,
} from './native-scanner-snapshot.ts';
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 Copy the native crate and its path dependency into the build context.
 Copies run concurrently, and every copy settles before this returns or throws,
 so removing the context afterwards never races an in-flight copy.

 @param context - build context directory receiving the repository-shaped snapshot

 @example
 ```ts
 await copySourceSnapshot({ context: context.path });
 ```
 */
export async function copySourceSnapshot({ context, }: {
  readonly context: string;
},): Promise<void> {
  /**
   Where the wrapper crate lands in the repository-shaped snapshot.
   */
  const subject = join(
    context,
    'package/git-policy/cli',
  );
  /**
   Where the JSONC crate the wrapper links lands in the snapshot.
   */
  const dependency = join(
    context,
    'package/jsonc-edit/core-rust',
  );
  /**
   Where the catalog specifier crate the wrapper links lands in the snapshot.
   */
  const catalogDependency = join(
    context,
    'package/catalog-specifier/core-rust',
  );
  /**
   Everything the gate image builds from: the wrapper, its path dependencies and the repository files its tests read.
   */
  const entries: readonly SnapshotEntry[] = [
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'README.md',
      'src/native',
    ].map(function crateEntry(name) {
      return {
        from: resolve(name),
        to: join(
          subject,
          name,
        ),
      };
    }),
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'src',
      'fixtures',
    ].map(function dependencyEntry(name) {
      return {
        from: resolve(
          '../../jsonc-edit/core-rust',
          name,
        ),
        to: join(
          dependency,
          name,
        ),
      };
    }),
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'src',
      'fixtures',
    ].map(function catalogDependencyEntry(name) {
      return {
        from: resolve(
          '../../catalog-specifier/core-rust',
          name,
        ),
        to: join(
          catalogDependency,
          name,
        ),
      };
    }),
    ...scannerSnapshotEntries({ context }),
    ...linterSnapshotEntries({ context }),
    {
      from: resolve('../../../clippy.toml'),
      to: join(
        context,
        'clippy.toml',
      ),
    },
    // The configuration control in `src/native/config_repository_tests.rs` reads the repository's own file.
    {
      from: resolve('../../../cli-git.config.jsonc'),
      to: join(
        context,
        'cli-git.config.jsonc',
      ),
    },
  ];
  /**
   Result of every copy; a failed copy never stops the others, so none is still in flight afterwards.
   */
  const copies = await Promise.allSettled(entries.map(function copyEntry(entry) {
    return cp(
      entry.from,
      entry.to,
      { recursive: true },
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
      `Copying ${String(failures.length)} source snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}
