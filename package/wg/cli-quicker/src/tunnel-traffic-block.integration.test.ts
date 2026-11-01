/**
 Verifies the traffic block with the real `nft` inside a disposable user and network namespace,
 which needs no privilege:
 replacing a block that is in place never deletes its table,
 the old sequence does (the positive control),
 and the removal shows the table gone.

 @module
 */

import assert from 'node:assert/strict';
import { execFileSync, } from 'node:child_process';

/**
 Result of the scenario,
 as the scenario prints it.
 */
type Observed = {
  /**
   What `nft monitor` printed while a block in place was replaced.
   */
  readonly replacement: string;

  /**
   What `nft monitor` printed while the old sequence ran.
   */
  readonly control: string;

  /**
   State of the block after the installs.
   */
  readonly installed: string;

  /**
   State of the block after the removal.
   */
  readonly removed: string;
};

/**
 Counts the transactions in the output of `nft monitor`:
 every committed change ends with one line that starts a new generation of the rule set.

 @param monitorOutput - Output of `nft monitor`.

 @returns The number of generations the change took.

 @example
 ```ts
 generations({ monitorOutput: '# new generation 3 by process 1 (nft)\n' }); // 1
 ```
 */
function generations({ monitorOutput, }: { readonly monitorOutput: string; },): number {
  return monitorOutput.split('# new generation',).length - 1;
}

/**
 Output of the scenario run in a namespace where the caller is mapped to root.
 */
const stdout = execFileSync(
  'unshare',
  [
    '--user',
    '--net',
    '--map-root-user',
    process.execPath,
    new URL('tunnel-traffic-block.scenario.ts', import.meta.url,).pathname,
  ],
  { timeout: 120_000, encoding: 'utf8', },
);
/**
 What the scenario observed.
 */
const observed = JSON.parse(stdout.trim().split('\n',).at(-1,) ?? '{}',) as Observed;
assert.ok(
  observed.control.includes('delete table inet wg-quicker-wgscenario-blocked',),
  `the monitor did not show the old sequence deleting the table, so it cannot show its absence: ${observed.control}`,
);
assert.ok(
  observed.replacement.length > 0,
  'the monitor saw nothing while the block was replaced, so it cannot show its absence',
);
assert.equal(
  generations({ monitorOutput: observed.control, },),
  2,
  `the old sequence is expected to take two transactions, with the table gone between them: ${observed.control}`,
);
assert.equal(
  generations({ monitorOutput: observed.replacement, },),
  1,
  `replacing a block in place is expected to be one transaction: ${observed.replacement}`,
);
assert.equal(
  observed.replacement.split('delete rule',).length - 1,
  2,
  `the old rules are expected to be emptied out by the same transaction that adds the new ones: ${observed.replacement}`,
);
assert.ok(
  !observed.replacement.includes('delete table',),
  `replacing a block in place deleted its table: ${observed.replacement}`,
);
assert.ok(
  !observed.replacement.includes('delete chain',),
  `replacing a block in place deleted a chain: ${observed.replacement}`,
);
assert.equal(observed.installed, 'present',);
assert.equal(observed.removed, 'absent',);
process.stdout
  .write('wg-quicker traffic block integration passed\n',);
