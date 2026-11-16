/**
 Verifies the concurrent-lifecycle properties of the built CLI in a disposable user and network
 namespace, which needs no privilege:
 one `up` that loses the race for an interface's link must not remove the link, the policy rules
 or the kill switch of the `up` that won it,
 and two tunnels never share one policy table, so no tunnel's traffic can leave through the other.

 The scenario runs in a container without OpenSnitch,
 because a real `up` looks for that firewall's configuration and would edit it when it finds one,
 and this test changes nothing on the machine it runs on.

 @module
 */

import assert from 'node:assert/strict';
import { execFileSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Bound of the whole scenario, in milliseconds.
 */
const SCENARIO_BOUND_MILLISECONDS = 300_000;

/**
 Policy table and route state the scenario reported for one interface.
 */
type InterfaceState = {
  /**
   Whether the link is present.
   */
  readonly link: boolean;
  /**
   Whether the link is a WireGuard interface.
   */
  readonly wireguard?: boolean;
  /**
   Policy table carried in the fwmark, absent when none.
   */
  readonly table?: number;
  /**
   Routes of that table.
   */
  readonly routes: readonly string[];
};

/**
 What the scenario observed, as it printed it.
 */
type Observed = {
  /**
   The two `up`s of one interface that raced for its link.
   */
  readonly raced: {
    readonly winnerExit: number;
    readonly loserExit: number;
    readonly loserOutput: string;
    readonly afterWinner: InterfaceState;
    readonly afterLoser: InterfaceState;
    readonly killSwitchAfterWinner: boolean;
    readonly killSwitchAfterLoser: boolean;
    readonly rulesAfterWinner: string;
    readonly rulesAfterLoser: string;
  };
  /**
   The positive control: one `up` on its own.
   */
  readonly solo: {
    readonly exit: number;
    readonly state: InterfaceState;
  };
  /**
   The two `up`s of two interfaces that raced for a policy table.
   */
  readonly concurrent: {
    readonly firstExit: number;
    readonly secondExit: number;
    readonly firstOutput: string;
    readonly secondOutput: string;
    readonly first: InterfaceState;
    readonly second: InterfaceState;
    readonly routeGets: {
      readonly ten: string;
      readonly eight: string;
    };
  };
};

if (existsSync('/etc/opensnitchd')) {
  throw new Error(
    'This test brings real interfaces up, whose start edits the OpenSnitch firewall when its configuration is present; run it in a container without /etc/opensnitchd.',
  );
}

/**
Scratch directory of this test, which the scenario writes its fixtures into.
 */
const directory = await mkdtemp(join(scratchRoot(), 'wg-lifecycle-',),);
await using scratch = {
  async [Symbol.asyncDispose](): Promise<void> {
    await rm(directory, { force: true, recursive: true, },);
  },
};
/**
 What the scenario printed on its last line.
 */
const stdout = execFileSync(
  'unshare',
  [
    '--user',
    '--net',
    '--map-root-user',
    process.execPath,
    new URL('tunnel-lifecycle.scenario.ts', import.meta.url,).pathname,
    directory,
  ],
  {
    timeout: SCENARIO_BOUND_MILLISECONDS,
    encoding: 'utf8',
  },
);
/**
 The observations, parsed from the scenario's verdict line.
 */
const observed = JSON.parse(
  stdout.trim()
    .split('\n',)
    .at(-1,) ?? '{}',
) as Observed;

assert.equal(
  observed.raced.winnerExit,
  0,
  `the first up of the raced interface did not succeed: ${observed.raced.loserOutput}`,
);
assert.equal(observed.raced.afterWinner.link, true, 'the winning up left no link.');
assert.equal(
  observed.raced.killSwitchAfterWinner,
  true,
  'the winning up installed no kill switch.',
);
assert.notEqual(
  observed.raced.loserExit,
  0,
  'the up that lost the race for the link reported success.',
);
assert.equal(
  observed.raced.afterLoser.link,
  true,
  `the rollback of the up that lost the race removed the winning up's link: ${observed.raced.loserOutput}`,
);
assert.equal(
  observed.raced.killSwitchAfterLoser,
  true,
  "the rollback of the up that lost the race removed the winning up's kill switch, so traffic leaves outside the tunnel",
);
assert.ok(
  observed.raced.rulesAfterWinner.includes('not from all fwmark',),
  `the winning up installed no policy rule to check: ${observed.raced.rulesAfterWinner}`,
);
assert.ok(
  observed.raced.rulesAfterLoser.includes('not from all fwmark',),
  `the rollback of the up that lost the race removed the winning up's policy rules, so traffic leaves outside the tunnel: ${observed.raced.rulesAfterLoser}`,
);

assert.equal(
  observed.solo.exit,
  0,
  `the solo up that is the positive control did not succeed: ${JSON.stringify(observed.solo.state,)}`,
);
assert.notEqual(observed.solo.state.table, undefined, 'the solo up installed no policy table.');

assert.equal(observed.concurrent.first.link, true, 'the first tunnel is not up after the retry.');
assert.equal(observed.concurrent.second.link, true, 'the second tunnel is not up after the retry.');
assert.notEqual(
  observed.concurrent.first.table,
  observed.concurrent.second.table,
  `both tunnels share one policy table, so each one's traffic can leave through the other: ${JSON.stringify(observed.concurrent,)}`,
);
assert.deepEqual(
  observed.concurrent.first.routes.filter((line,) => line.includes('wglb',)),
  [],
  `the first tunnel's policy table holds the second tunnel's route: ${JSON.stringify(observed.concurrent.first.routes,)}`,
);
assert.deepEqual(
  observed.concurrent.second.routes.filter((line,) => line.includes('wgla',)),
  [],
  `the second tunnel's policy table holds the first tunnel's route: ${JSON.stringify(observed.concurrent.second.routes,)}`,
);
assert.ok(
  observed.concurrent.first.routes.some((line,) => line.startsWith('default ',),),
  `the tunnel with the default route has no default in its own table: ${JSON.stringify(observed.concurrent.first.routes,)}`,
);
assert.ok(
  observed.concurrent.second.routes.some((line,) => line.startsWith('10.0.0.0/8 ',),),
  `the tunnel that allows 10/8 has no route for it in its own table: ${JSON.stringify(observed.concurrent.second.routes,)}`,
);
assert.ok(
  observed.concurrent.routeGets.ten.includes('dev wglb',),
  `10.0.0.1 does not leave through the tunnel that allows 10/8: ${observed.concurrent.routeGets.ten}`,
);
assert.ok(
  observed.concurrent.routeGets.eight.includes('dev wgla',),
  `8.8.8.8 does not leave through the tunnel that allows everything: ${observed.concurrent.routeGets.eight}`,
);
