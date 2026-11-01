import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { manualFinishSteps, } from '../dist/final/node/tunnel-teardown-manual.mjs';
import { trafficBlockTable, } from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Interface the steps are written for.
 */
const INTERFACE_NAME = 'wgtest';

/**
 State file of that interface.
 */
const STATE_PATH = '/run/wg-quicker/interface-0123456789abcdef.json';

/**
 Path of the program that stops the application watcher.
 */
const COMPANION_PATH = '/opt/wg/target/release/wg-quicker-exempt';

/**
 Policy table of the interface.
 */
const POLICY_TABLE = 51_820;

/**
 Lines of the steps when the policy table is known.
 */
const KNOWN = manualFinishSteps({
  interfaceName: INTERFACE_NAME,
  facts: {
    statePath: STATE_PATH,
    companionPath: COMPANION_PATH,
    policyTable: POLICY_TABLE,
  },
},);

await describe({
  name: manualFinishSteps.name,
  children: [
    it({
      name: 'names the state file to look at and to move away, and the rules to look at',
      fn: async () => {
        expect(KNOWN,).toContain(`  sudo cat ${STATE_PATH}`,);
        expect(KNOWN,).toContain('  sudo ip -4 rule show',);
        expect(KNOWN,).toContain('  sudo ip -6 rule show',);
        expect(KNOWN,).toContain(`  sudo mv ${STATE_PATH} ${STATE_PATH}.damaged`,);
      },
    },),
    it({
      name: 'lists the commands that take the tunnel down by hand, in the order wg-quicker runs them, with the paths and the policy table of this machine',
      fn: async () => {
        /**
         The by-hand commands in the order they are listed.
         */
        const commands = KNOWN.filter((line,) => line.startsWith('  sudo ',),).map((line,) => line.split('    (',)[0] ?? line,);
        const start = commands.indexOf(`  sudo ${COMPANION_PATH} watch-stop ${INTERFACE_NAME}`,);
        expect(start,).toBeGreaterThan(-1,);
        expect(commands.slice(start,),).toEqual([
          `  sudo ${COMPANION_PATH} watch-stop ${INTERFACE_NAME}`,
          `  sudo cat ${STATE_PATH}.watcher.json`,
          `  sudo ip -4 rule delete table ${String(POLICY_TABLE,)}`,
          `  sudo ip -6 rule delete table ${String(POLICY_TABLE,)}`,
          '  sudo ip -4 rule delete table main suppress_prefixlength 0',
          '  sudo ip -6 rule delete table main suppress_prefixlength 0',
          `  sudo nft delete table inet wg-quicker-${INTERFACE_NAME}`,
          `  sudo ip link delete dev ${INTERFACE_NAME}`,
          `  sudo resolvectl revert ${INTERFACE_NAME}`,
          `  sudo nft delete table inet ${trafficBlockTable({ interfaceName: INTERFACE_NAME, },)}`,
          `  sudo rm --force ${STATE_PATH} ${STATE_PATH}.watcher.json`,
        ],);
      },
    },),
    it({
      name: 'stops the route watcher before it removes any rule, and removes the interface after the rules',
      fn: async () => {
        /**
         Position of a line that starts with the given text.
         */
        const at = (text: string,): number => KNOWN.findIndex((line,) => line.startsWith(text,),);
        expect(at(`  sudo cat ${STATE_PATH}.watcher.json`,),).toBeLessThan(at('  sudo ip -4 rule delete table 5',),);
        expect(at('  sudo ip -6 rule delete table main',),).toBeLessThan(at(`  sudo ip link delete dev ${INTERFACE_NAME}`,),);
        expect(at(`  sudo nft delete table inet wg-quicker-${INTERFACE_NAME}`,),).toBeLessThan(at(`  sudo ip link delete dev ${INTERFACE_NAME}`,),);
      },
    },),
    it({
      name: 'says where to read the policy table, and does not invent one, when the live interface gave none',
      fn: async () => {
        /**
         Lines when the table is not known.
         */
        const unknown = manualFinishSteps({ interfaceName: INTERFACE_NAME, facts: { statePath: STATE_PATH, companionPath: COMPANION_PATH, }, },);
        expect(unknown.some((line,) => line.startsWith(`  sudo wg show ${INTERFACE_NAME} fwmark`,),),).toBe(true,);
        expect(unknown.some((line,) => line.startsWith('  sudo ip -4 rule delete table ',) && (!line.includes('main',)),),).toBe(false,);
      },
    },),
    it({
      name: 'explains how to remove the bypass rules and routes, which carry protocol 201 and are not removed by the other commands',
      fn: async () => {
        /**
         The line about protocol 201.
         */
        const text = KNOWN.join('\n',);
        expect(text,).toContain('proto 201',);
        expect(text,).toContain('sudo ip -4 rule delete pref <pref> protocol 201',);
        expect(text,).toContain('sudo ip -4 route flush table <table> protocol 201',);
      },
    },),
  ],
},);
