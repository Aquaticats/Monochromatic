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
 Files a real `up` would edit when OpenSnitch is installed on the machine,
 which this test must not find.
 */
const OSNITCH_FILES: readonly string[] = [
  '/etc/opensnitchd/default-config.json',
  '/etc/opensnitchd/system-fw.json',
];

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
   One `up` of an interface held at its `ip link add`, and a second start of the same interface beside it.
   */
  readonly raced: {
    readonly secondExit: number;
    readonly secondOutput: string;
    readonly whileHeld: InterfaceState;
    readonly heldExit: number;
    readonly afterHeld: InterfaceState;
    readonly killSwitchAfterHeld: boolean;
    readonly rulesAfterHeld: string;
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
  /**
   Two tunnels whose main-table suppressions must stay apart.
   */
  readonly suppression: {
    readonly ups: readonly number[];
    readonly rulesBothUp: string;
    readonly lanRouteBothUp: string;
    readonly down: number;
    readonly rulesAfterDown: string;
    readonly lanRouteAfterDown: string;
    readonly internetRouteAfterDown: string;
    readonly failedReclaim: number;
    readonly rulesAfterFailedUp: string;
    readonly lanRouteAfterFailedUp: string;
  };
  /**
   A tunnel whose start is killed between its fwmark and its first rule.
   */
  readonly killed: {
    readonly whileHeld: InterfaceState;
    readonly afterKillExit: number;
    readonly afterKillOutput: string;
    readonly states: {
      readonly killed: InterfaceState;
      readonly after: InterfaceState;
    };
    readonly internetAfterKill: string;
    readonly downExit: number;
    readonly afterDownState: InterfaceState;
    readonly rulesAfterDown: string;
    readonly internetAfterDown: string;
  };
};

if (OSNITCH_FILES.some(function present(path,): boolean {
  return existsSync(path,);
},)) {
  throw new Error(
    'This test brings real interfaces up, whose start edits the OpenSnitch firewall when its configuration is present; run it where neither /etc/opensnitchd/default-config.json nor /etc/opensnitchd/system-fw.json exists.',
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

// The second start of the same interface must refuse at the whole-lifecycle lock,
// and the first one, released afterwards, must finish with everything it made still there.
assert.notEqual(
  observed.raced.secondExit,
  0,
  'a second start of one interface ran beside the first instead of refusing',
);
assert.ok(
  observed.raced.secondOutput.includes('Another wg-quicker lifecycle is running for',),
  `the second start of one interface refused for another reason: ${observed.raced.secondOutput}`,
);
assert.equal(
  observed.raced.heldExit,
  0,
  `the held start did not finish: ${observed.raced.afterHeld.link}`,
);
assert.equal(observed.raced.afterHeld.link, true, 'the held start left no link.');
assert.equal(
  observed.raced.killSwitchAfterHeld,
  true,
  'the held start installed no kill switch.',
);
assert.ok(
  observed.raced.rulesAfterHeld.includes('not from all fwmark',),
  `the held start installed no policy rule to check: ${observed.raced.rulesAfterHeld}`,
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
// Both tunnels allow 10.0.0.1, so which of the two carries it is decided by rule priority, as the page says;
// what matters is that it leaves through one of their tables and not by the physical default.
assert.ok(
  ['dev wgla', 'dev wglb',].some((name,) => observed.concurrent.routeGets.ten.includes(name,))
    && observed.concurrent.routeGets.ten.includes('table 518',),
  `10.0.0.1 does not leave through either tunnel of the pair: ${observed.concurrent.routeGets.ten}`,
);
assert.ok(
  observed.concurrent.routeGets.eight.includes('dev wgla',),
  `8.8.8.8 does not leave through the tunnel that allows everything: ${observed.concurrent.routeGets.eight}`,
);

assert.deepEqual(observed.suppression.ups, [0, 0,],);
assert.ok(
  observed.suppression.lanRouteBothUp.includes('dev lan0',),
  `192.168.77.9 does not stay on its own interface with both tunnels up: ${observed.suppression.lanRouteBothUp}`,
);
assert.equal(observed.suppression.down, 0,);
const suppressionsLeft = observed.suppression.rulesAfterDown
  .split('\n',)
  .filter((line,) => line.includes('suppress_prefixlength 0',));
assert.equal(
  suppressionsLeft.length,
  1,
  `the down of one tunnel left ${String(suppressionsLeft.length,)} main-table suppressions instead of the other tunnel's one: ${observed.suppression.rulesAfterDown}`,
);
assert.ok(
  observed.suppression.lanRouteAfterDown.includes('dev lan0',),
  `the down of one tunnel sent the other tunnel's local-network traffic into the tunnel: ${observed.suppression.lanRouteAfterDown}`,
);
assert.ok(
  observed.suppression.internetRouteAfterDown.includes('dev wglt',),
  `the surviving tunnel stopped carrying the internet: ${observed.suppression.internetRouteAfterDown}`,
);
assert.notEqual(observed.suppression.failedReclaim, 0,);
const suppressionsAfterFailure = observed.suppression.rulesAfterFailedUp
  .split('\n',)
  .filter((line,) => line.includes('suppress_prefixlength 0',));
assert.equal(
  suppressionsAfterFailure.length,
  1,
  `a failed up's rollback left ${String(suppressionsAfterFailure.length,)} main-table suppressions instead of the other tunnel's one: ${observed.suppression.rulesAfterFailedUp}`,
);
assert.ok(
  observed.suppression.lanRouteAfterFailedUp.includes('dev lan0',),
  `a failed up's rollback sent the other tunnel's local-network traffic into the tunnel: ${observed.suppression.lanRouteAfterFailedUp}`,
);

// The killed start had already set its fwmark and named no rule yet: that is the window.
assert.notEqual(observed.killed.whileHeld.table, undefined,);
assert.deepEqual(observed.killed.whileHeld.routes, [],);
assert.equal(observed.killed.afterKillExit, 0,);
assert.notEqual(observed.killed.states.killed.table, observed.killed.states.after.table,);
assert.equal(
  observed.killed.states.after.link,
  true,
  `the tunnel started after the kill has no link: ${observed.killed.afterKillOutput}`,
);
assert.ok(
  observed.killed.internetAfterKill.includes('dev wgld',),
  `the tunnel started after the kill does not carry the internet: ${observed.killed.internetAfterKill}`,
);
assert.equal(observed.killed.downExit, 0,);
assert.equal(
  observed.killed.afterDownState.link,
  true,
  'the down of the killed tunnel removed the live tunnel\'s link',
);
const liveRulesAfterDown = observed.killed.rulesAfterDown
  .split('\n',)
  .filter((line,) => line.includes('not from all fwmark',));
assert.equal(
  liveRulesAfterDown.length,
  1,
  `the down of the killed tunnel removed the live tunnel's policy rules: ${observed.killed.rulesAfterDown}`,
);
assert.ok(
  observed.killed.internetAfterDown.includes('dev wgld',),
  `after the dead tunnel's down the live tunnel's traffic left outside it: ${observed.killed.internetAfterDown}`,
);
