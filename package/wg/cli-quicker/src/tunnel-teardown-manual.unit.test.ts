import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runAllowingFailure, } from '../dist/final/node/runner.mjs';
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
 State file path that a hostile or careless environment could produce:
 spaces, a quote, a semicolon, a command substitution, backticks and a newline.
 */
const HOSTILE_STATE_PATH = '/run/wg quicker/it\'s; $(touch injected) `touch injected`\nnext.json';

/**
 Program path with a space, a semicolon and a substitution.
 */
const HOSTILE_COMPANION_PATH = '/opt/my tools/wg-quicker-exempt; touch injected $(touch injected)';

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

/**
 Lines of the steps for hostile paths.
 */
const HOSTILE = manualFinishSteps({
  interfaceName: INTERFACE_NAME,
  facts: {
    statePath: HOSTILE_STATE_PATH,
    companionPath: HOSTILE_COMPANION_PATH,
    policyTable: POLICY_TABLE,
  },
},);

/**
 The commands among the lines:
 those that start with `sudo`.

 @param lines - Lines of the steps.

 @returns The command lines in order.

 @example
 ```ts
 commandsOf({ lines: ['to look:', '  sudo cat x'] }); // ['  sudo cat x']
 ```
 */
function commandsOf({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter((line,) => line.startsWith('  sudo ',),);
}

/**
 Runs a printed command in bash with `sudo` replaced by a function that prints each word it is given in angle brackets,
 so what the shell made of the quoting can be read back.

 @param line - A printed command line.

 @returns What the stand-in printed,
 or the failure of the shell.

 @example
 ```ts
 await wordsOf({ line: '  sudo cat \'a b\'' }); // '<cat><a b>\n'
 ```
 */
async function wordsOf({ line, }: { readonly line: string; },): Promise<string> {
  /**
   What the shell ran.
   */
  const result = await runAllowingFailure({
    command: 'bash',
    args: ['-c', `sudo() { printf '<%s>' "$@"; printf '\\n'; }\n${line}`,],
  },);
  return result.exitCode === 0 ? result.stdout : `shell failed with ${String(result.exitCode,)}: ${result.stderr}`;
}

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
        const commands = commandsOf({ lines: KNOWN, },);
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
    it({
      name: 'keeps every command a command that can be pasted whole: remarks stand on lines of their own, beginning with a hash',
      fn: async () => {
        for (const line of commandsOf({ lines: KNOWN, },))
          expect(line.includes('(',) || line.includes('    ',),).toBe(false,);
        expect(KNOWN.filter((line,) => line.startsWith('  # ',),).length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'quotes the state path and the program path, so that a shell reads each as one word whatever they hold',
      fn: async () => {
        expect(await wordsOf({ line: `  sudo cat x`, },),).toBe('<cat><x>\n',);
        /**
         The command lines for the hostile paths.
         */
        const commands = commandsOf({ lines: HOSTILE, },);
        for (const command of commands) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each command starts its own shell.
          const printed = await wordsOf({ line: command, },);
          expect(printed.startsWith('<',),).toBe(true,);
          expect(printed.endsWith('>\n',),).toBe(true,);
        }
        expect(await wordsOf({ line: commands.find((line,) => line.startsWith('  sudo mv ',),) ?? '', },),).toBe(`<mv><${HOSTILE_STATE_PATH}><${HOSTILE_STATE_PATH}.damaged>\n`,);
        expect(await wordsOf({ line: commands.find((line,) => line.includes(' watch-stop ',),) ?? '', },),).toBe(`<${HOSTILE_COMPANION_PATH}><watch-stop><${INTERFACE_NAME}>\n`,);
        expect(await wordsOf({ line: commands.find((line,) => line.startsWith('  sudo cat ',) && line.includes('watcher',),) ?? '', },),).toBe(`<cat><${HOSTILE_STATE_PATH}.watcher.json>\n`,);
        expect(await wordsOf({ line: commands.find((line,) => line.startsWith('  sudo rm ',),) ?? '', },),).toBe(`<rm><--force><${HOSTILE_STATE_PATH}><${HOSTILE_STATE_PATH}.watcher.json>\n`,);
      },
    },),
    it({
      name: 'prints no control character and no line break inside any line, so each line of the message is one line and cannot rewrite a terminal',
      fn: async () => {
        for (const line of HOSTILE)
          expect(Array.from(line,).some((character,) => ((character.codePointAt(0,) ?? 0) < 0x20) || (((character.codePointAt(0,) ?? 0) >= 0x7F) && ((character.codePointAt(0,) ?? 0) <= 0x9F)),),).toBe(false,);
      },
    },),
  ],
},);
