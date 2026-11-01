/**
 Verifies the traffic block with the real `nft` inside a disposable user and network namespace,
 which needs no privilege:
 replacing a block that is in place never deletes its table,
 the old sequence does (the positive control),
 and the removal shows the table gone.
 A second scenario sends real packets through a block that lets the clearnet traffic through:
 a packet with the exempt mark and a packet to an update host pass,
 a packet with any other mark, an unmarked packet and nothing else leaving the machine are dropped,
 loopback stays allowed,
 and a process without `CAP_NET_RAW` and `CAP_NET_ADMIN` cannot give its packets the mark.

 @module
 */

import assert from 'node:assert/strict';
import { execFileSync, } from 'node:child_process';
import { constants, } from 'node:os';

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

/**
 What `nft` counted for a packet,
 as the clearnet scenario prints it.
 */
type Probed = {
  /**
   Kinds of rule whose counters grew while the packet was sent.
   */
  readonly counted: readonly string[];

  /**
   Exit status of the sender.
   */
  readonly status: number;

  /**
   Standard output of the sender.
   */
  readonly output: string;
};

/**
 Result of the clearnet scenario.
 */
type ObservedClearnet = {
  /**
   Kinds of the rules of the output chain,
   in order.
   */
  readonly outputOrder: readonly string[];

  /**
   Kinds of the rules of the forward chain,
   in order.
   */
  readonly forwardOrder: readonly string[];

  /**
   A packet with the exempt mark to an address that is not an exception.
   */
  readonly marked: Probed;

  /**
   A packet with a mark one above the exempt mark.
   */
  readonly markedPlusOne: Probed;

  /**
   A packet without a mark.
   */
  readonly unmarked: Probed;

  /**
   An unmarked packet to an update host.
   */
  readonly host: Probed;

  /**
   An echo request to loopback.
   */
  readonly loopback: Probed;

  /**
   A program that may set the mark.
   */
  readonly withCapabilities: Probed;

  /**
   The same program without the capabilities the kernel asks for.
   */
  readonly withoutCapabilities: Probed;
};

/**
 Output of the clearnet scenario in a namespace where the caller is mapped to root.
 */
const clearnetOutput = execFileSync(
  'unshare',
  [
    '--user',
    '--net',
    '--map-root-user',
    process.execPath,
    new URL('tunnel-traffic-block-clearnet.scenario.ts', import.meta.url,).pathname,
  ],
  { timeout: 120_000, encoding: 'utf8', },
);
/**
 What the clearnet scenario observed.
 */
const clearnet = JSON.parse(clearnetOutput.trim().split('\n',).at(-1,) ?? '{}',) as ObservedClearnet;
// The accept rules stand ahead of the drop, in the order the routing's rules are read: mark, then hosts.
assert.deepEqual(clearnet.outputOrder, ['mark', 'hosts4', 'hosts6', 'drop',],);
// The forward chain has the update-host accepts and no mark rule, ahead of its drop.
assert.deepEqual(clearnet.forwardOrder, ['hosts4', 'hosts6', 'replies4', 'replies6', 'drop',],);
assert.deepEqual(clearnet.marked.counted, ['mark',], 'a packet with the exempt mark must pass the block',);
assert.deepEqual(clearnet.markedPlusOne.counted, ['drop',], 'only the exact exempt mark passes',);
assert.deepEqual(clearnet.unmarked.counted, ['drop',], 'an unmarked packet must be dropped',);
assert.deepEqual(clearnet.host.counted, ['hosts4',], 'a packet to an update host must pass the block',);
// Loopback is not counted by any rule and is answered.
assert.deepEqual(clearnet.loopback.counted, [],);
assert.equal(clearnet.loopback.status, 0,);
assert.deepEqual(clearnet.withCapabilities.counted, ['mark',],);
assert.equal(clearnet.withCapabilities.output.trim(), 'marked',);
// Without the capabilities the kernel refuses the mark, and the packet goes out unmarked and is dropped.
assert.equal(clearnet.withoutCapabilities.output.trim(), `refused ${String(constants.errno.EPERM,)}`,);
assert.deepEqual(clearnet.withoutCapabilities.counted, ['drop',],);

/**
 Result of the forward scenario.
 */
type ObservedForward = {
  /**
   Kinds of the rules of the router's forward chain,
   in order.
   */
  readonly forwardOrder: readonly string[];

  /**
   Kinds of the rules of the router's forward chain after the block was installed again.
   */
  readonly forwardOrderAfterReinstall: readonly string[];

  /**
   A forwarded IPv4 echo request to an update host and its reply.
   */
  readonly hostV4: Probed;

  /**
   A forwarded IPv4 echo request to another address.
   */
  readonly otherV4: Probed;

  /**
   How many rules for the replies were taken out to put the block back as it was before they existed.
   */
  readonly removedReplyRules: number;

  /**
   A TCP connect from the guest to an update host, with the rules for the replies taken out.
   */
  readonly connectWithoutReplyRules: Probed;

  /**
   The same connect with the block as it is installed.
   */
  readonly connectWithReplyRules: Probed;

  /**
   A TCP connect from the guest to another address of the outside, where the listener would answer.
   */
  readonly connectOther: Probed;

  /**
   A forwarded IPv6 packet to an update host.
   */
  readonly hostV6: Probed;

  /**
   A forwarded IPv6 packet to another address.
   */
  readonly otherV6: Probed;

  /**
   A packet of the router itself to an IPv6 update host.
   */
  readonly outputHostV6: Probed;
};

/**
 Output of the forward scenario in a namespace where the caller is mapped to root,
 which holds a guest namespace of its own.
 */
const forwardOutput = execFileSync(
  'unshare',
  [
    '--user',
    '--net',
    '--map-root-user',
    process.execPath,
    new URL('tunnel-traffic-block-forward.scenario.ts', import.meta.url,).pathname,
  ],
  { timeout: 120_000, encoding: 'utf8', },
);
/**
 What the forward scenario observed.
 */
const forwarded = JSON.parse(forwardOutput.trim().split('\n',).at(-1,) ?? '{}',) as ObservedForward;
assert.deepEqual(forwarded.forwardOrder, ['hosts4', 'hosts6', 'replies4', 'replies6', 'drop',],);
assert.deepEqual(forwarded.forwardOrderAfterReinstall, forwarded.forwardOrder, 'installing the block again must put back the rules that were taken out',);
// A guest's round trip to an update host passes in both directions, and any other forwarded packet is dropped.
assert.deepEqual(forwarded.hostV4.counted, ['hosts4', 'replies4',], 'a forwarded request to an update host and its reply must pass the block',);
assert.equal(forwarded.hostV4.status, 0, 'the guest must get its echo reply from the update host',);
assert.deepEqual(forwarded.otherV4.counted, ['drop',], 'any other forwarded packet must be dropped',);
assert.deepEqual(forwarded.hostV6.counted, ['hosts6', 'replies6',], 'a forwarded IPv6 request to an update host and its reply must pass the block',);
assert.equal(forwarded.hostV6.status, 0, 'the guest must get its IPv6 echo reply from the update host',);
assert.deepEqual(forwarded.otherV6.counted, ['drop',], 'any other forwarded IPv6 packet must be dropped',);
// The positive control: without the rules for the replies the request goes through, the answer is dropped, and the connect fails.
assert.equal(forwarded.removedReplyRules, 2,);
assert.equal(forwarded.connectWithoutReplyRules.output.split(' ',)[0]?.trim(), 'failed', 'a connect needs the replies: without their rules it must fail',);
assert.deepEqual(forwarded.connectWithoutReplyRules.counted, ['hosts4', 'drop',],);
// With the rules for the replies the same connect is made, and nothing is dropped.
assert.equal(forwarded.connectWithReplyRules.output.trim(), 'connected', 'a connect to an update host must be made through the block',);
assert.deepEqual(forwarded.connectWithReplyRules.counted, ['hosts4', 'replies4',],);
// Any other forwarded flow still fails, although the listener would answer it.
assert.equal(forwarded.connectOther.output.split(' ',)[0]?.trim(), 'failed', 'a connect to another address must still fail',);
assert.deepEqual(forwarded.connectOther.counted, ['drop',],);
// The router's own IPv6 traffic is noisy (neighbor discovery follows the packet), so only the accept is asserted.
assert.ok(forwarded.outputHostV6.counted.includes('hosts6',), 'a packet of the router to an IPv6 update host must pass the block',);
process.stdout
  .write('wg-quicker traffic block integration passed\n',);
