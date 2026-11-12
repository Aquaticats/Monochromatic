/**
 The rules every Pi brief carries: the file each brief names first, and the sentence every brief quotes about synthetic host
 load.

 The rules themselves live in `doc/rules-every-pi-brief-carries.md` beside this package, so a run reads them from the file rather
 than from a copy that can drift. This module holds only the path to that file, and the parts of a brief that the project asks
 every brief to carry.

 @module
 */

import { join, } from 'node:path';

/**
 The rules file's path from the top of the checkout: this package's own doc folder, which every worktree of this checkout holds.
 */
export const RULES_FILE_IN_THE_CHECKOUT: string = join(
  'package',
  'pi-shared',
  'subagent-driver',
  'doc',
  'rules-every-pi-brief-carries.md',
);

/**
 The sentence every brief quotes, which binds every run on this machine: synthetic host load is banned, and a test that needs
 load sizes its own container.
 */
export const NO_SYNTHETIC_HOST_LOAD: string =
  'Never generate synthetic host load in any session or subagent: no `stress-ng`, busy loops, `--stress-count`, or repeated '
    + 'full test runs; cap the test\'s own container (`--cpus` sized to the test) instead.';

/**
 Reads the rules file's path in one checkout.

 @param checkoutFolder - the top of the checkout the run works in

 @returns the rules file's path

 @example
 ```ts
 const rules = rulesFileIn({ checkoutFolder: '/repo', },);
 ```
 */
export function rulesFileIn({ checkoutFolder, }: { readonly checkoutFolder: string },): string {
  return join(
    checkoutFolder,
    RULES_FILE_IN_THE_CHECKOUT,
  );
}
