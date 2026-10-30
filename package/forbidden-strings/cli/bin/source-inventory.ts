/**
 Content hashes of a copied source tree.
 */
import { createHash } from 'node:crypto';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { ScannerVerificationError } from './scanner-run.ts';

/**
 One copied file and the digest of its bytes.
 */
export type InventoryRecord = {
  readonly path: string;
  readonly sha256: string;
};

/**
 Hash the actual copied bytes, not a possibly dirty Git tree identifier alone.

 @param root - directory holding the copied tree

 @param relative - directory below the root to list, or the root itself when empty

 @returns every file below the directory, in name order within each directory, depth first

 @throws ScannerVerificationError when an entry is neither a file nor a directory

 @example
 ```ts
 const records = await inventory({ root: context });
 ```
 */
export async function inventory({
  root,
  relative = '',
}: {
  readonly root: string;
  readonly relative?: string;
}): Promise<readonly InventoryRecord[]> {
  /**
   Entries of the directory, in name order.
   */
  const entries = (await readdir(
    join(
      root,
      relative,
    ),
    { withFileTypes: true },
  ))
    .toSorted(function byName(
      first,
      second,
    ) {
      return first.name
        .localeCompare(second.name);
    });
  /**
   Records of each entry, in entry order.
   */
  const perEntry = await mapInSequence({
    items: entries,
    async run(entry): Promise<readonly InventoryRecord[]> {
      /**
       Path of the entry below the root.
       */
      const name = join(
        relative,
        entry.name,
      );
      if (entry.isDirectory())
        return await inventory({
          root,
          relative: name,
        });
      if (!entry.isFile())
        throw new ScannerVerificationError(`Unexpected snapshot entry: ${name}`);
      return [
        {
          path: name,
          sha256: createHash('sha256')
            .update(await readFile(join(
              root,
              name,
            )))
            .digest('hex'),
        },
      ];
    },
  });
  return perEntry.flat();
}
