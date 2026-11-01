import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { runAllowingFailure, } from '../dist/final/node/runner.mjs';
import { shellQuote, } from '../dist/final/node/shell-quote.mjs';
import {
  blockStatements,
  manualFinishSteps,
  readManualFacts,
} from '../dist/final/node/tunnel-teardown-manual.mjs';
import {
  trafficBlockScript,
  trafficBlockTable,
} from '../dist/final/node/tunnel-traffic-block.mjs';

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
 What the routing sends around the tunnel when it could be read.
 */
const CLEARNET = {
  mark: 100,
  hosts: [{ family: '-4', address: '203.0.113.7', },],
  interfaces: [{ family: '-4', name: 'eth0', },],
} as const;

/**
 The command that puts the traffic block in place,
 for an interface without exceptions.
 */
const BLOCK_LINE = `  sudo nft ${shellQuote({ value: blockStatements({ interfaceName: INTERFACE_NAME, },), },)}`;

/**
 Process ID the record of the route watcher names.
 */
const WATCHER_PID = 4_242;

/**
 Lines of the steps when the record of the route watcher gave its process ID.
 */
const WITH_PID = manualFinishSteps({
  interfaceName: INTERFACE_NAME,
  facts: {
    statePath: STATE_PATH,
    companionPath: COMPANION_PATH,
    policyTable: POLICY_TABLE,
    routeWatcherPid: WATCHER_PID,
  },
},);

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
  return lines.filter((line,) => line.startsWith('  sudo ',) || line.startsWith('  ps ',),);
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
    args: ['-c', `sudo() { printf '<%s>' "$@"; printf '\\n'; }\nps() { printf '<%s>' "$@"; printf '\\n'; }\n${line}`,],
  },);
  return result.exitCode === 0 ? result.stdout : `shell failed with ${String(result.exitCode,)}: ${result.stderr}`;
}

/**
 A runtime directory of the ownership state that is empty,
 which stays set for the interface name of the scope.
 */
type RuntimeDirectory = {
  /**
   Path of the directory.
   */
  readonly path: string;

  /**
   Puts the environment back and removes the directory.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Points the runtime directory of the ownership state at a new empty directory until the scope ends.

 @returns The directory,
 disposed by `await using`.

 @example
 ```ts
 await using runtime = await emptyRuntimeDirectory();
 ```
 */
async function emptyRuntimeDirectory(): Promise<RuntimeDirectory> {
  /**
   Empty directory standing for the runtime directory.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-quicker-manual-',),);
  /**
   Value to put back.
   */
  const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = path;
  return {
    path,
    [Symbol.asyncDispose]: async function restore(): Promise<void> {
      if (previous === undefined)
        delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
      else
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: readManualFacts.name,
  children: [
    it({
      name: 'reads the process ID from the record of the route watcher next to the state file, and leaves it out, without throwing, when the record is absent or damaged',
      fn: async () => {
        // One scope for all cases: the runtime directory is a variable of the whole process, so cases must not overlap.
        await using runtime = await emptyRuntimeDirectory();
        expect((await readManualFacts({ interfaceName: 'wgtest', },)).routeWatcherPid,).toBe(undefined,);
        const { statePath, } = await readManualFacts({ interfaceName: 'wgtest', },);
        // Nothing is written unless the path is inside the scratch directory.
        expect(statePath.startsWith(`${runtime.path}/`,),).toBe(true,);
        await writeFile(`${statePath}.watcher.json`, JSON.stringify({ ownerId: 'owner', pid: WATCHER_PID, startTime: '12345', },),);
        expect((await readManualFacts({ interfaceName: 'wgtest', },)).routeWatcherPid,).toBe(WATCHER_PID,);
        for (const text of ['not json', '{}', '{"ownerId":"o","pid":-1,"startTime":"s"}', '{"ownerId":"o","pid":"4242","startTime":"s"}',]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case writes and reads its own record.
          await writeFile(`${statePath}.watcher.json`, text,);
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case writes and reads its own record.
          expect((await readManualFacts({ interfaceName: 'wgtest', },)).routeWatcherPid,).toBe(undefined,);
        }
      },
    },),
  ],
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
        const commands = commandsOf({ lines: KNOWN, },);
        const start = commands.indexOf(BLOCK_LINE,);
        expect(start,).toBeGreaterThan(-1,);
        expect(commands.slice(start,),).toEqual([
          BLOCK_LINE,
          `  sudo ${COMPANION_PATH} watch-stop ${INTERFACE_NAME}`,
          `  sudo ip -4 rule delete table ${String(POLICY_TABLE,)}`,
          `  sudo ip -6 rule delete table ${String(POLICY_TABLE,)}`,
          '  sudo ip -4 rule delete table main suppress_prefixlength 0',
          '  sudo ip -6 rule delete table main suppress_prefixlength 0',
          `  sudo nft delete table inet wg-quicker-${INTERFACE_NAME}`,
          `  sudo ip link delete dev ${INTERFACE_NAME}`,
          `  sudo resolvectl revert ${INTERFACE_NAME}`,
          `  sudo rm --force ${STATE_PATH} ${STATE_PATH}.watcher.json`,
          `  sudo nft delete table inet ${trafficBlockTable({ interfaceName: INTERFACE_NAME, },)}`,
        ],);
      },
    },),
    it({
      name: 'opens the by-hand list with the traffic block, in one nft argument, and lifts it as the last command, so no routing is removed with no block in place',
      fn: async () => {
        /**
         The by-hand commands in the order they are listed.
         */
        const commands = commandsOf({ lines: KNOWN, },);
        /**
         Where the block goes in and where it comes out.
         */
        const blockIn = commands.indexOf(BLOCK_LINE,);
        const blockOut = commands.indexOf(`  sudo nft delete table inet ${trafficBlockTable({ interfaceName: INTERFACE_NAME, },)}`,);
        expect(blockIn,).toBeGreaterThan(-1,);
        expect(blockOut,).toBe(commands.length - 1,);
        // Nothing that removes routing stands ahead of the block.
        for (const [index, command,] of commands.entries()) {
          if (command.includes(' delete ',) || command.includes(' revert ',) || command.includes(' watch-stop ',) || command.includes(' rm ',))
            expect(index,).toBeGreaterThan(blockIn,);
        }
        // The block is built from the same script that wg-quicker applies.
        expect(blockStatements({ interfaceName: INTERFACE_NAME, },),).toBe(trafficBlockScript({ interfaceName: INTERFACE_NAME, },).split('\n',).filter((line,) => line !== '',).join('; ',),);
      },
    },),
    it({
      name: 'puts the exceptions that could be read into the block, and says in a remark when none could',
      fn: async () => {
        /**
         Lines of the steps with and without the exceptions.
         */
        const withExceptions = manualFinishSteps({ interfaceName: INTERFACE_NAME, facts: { statePath: STATE_PATH, companionPath: COMPANION_PATH, clearnet: CLEARNET, }, },);
        const statements = blockStatements({ interfaceName: INTERFACE_NAME, clearnet: CLEARNET, },);
        expect(withExceptions,).toContain(`  sudo nft ${shellQuote({ value: statements, },)}`,);
        expect(statements,).toContain('meta mark 0x64 counter accept',);
        expect(statements,).toContain('ip daddr { 203.0.113.7 } counter accept',);
        expect(statements,).toContain('ip saddr { 203.0.113.7 } counter accept',);
        expect(withExceptions.some((line,) => line.startsWith('  # ',) && line.includes('read just now',),),).toBe(true,);
        // Without them the block has no accept rule beyond the policy of its chains.
        expect(blockStatements({ interfaceName: INTERFACE_NAME, },).includes('counter accept',),).toBe(false,);
        expect(KNOWN.some((line,) => line.startsWith('  # ',) && line.includes('no exceptions could be read',),),).toBe(true,);
      },
    },),
    it({
      name: 'asks for the route watcher to be stopped first, before the state file is moved away and before the by-hand commands, because without the state file wg-quicker no longer looks for it',
      fn: async () => {
        /**
         Position of a line that starts with the given text.
         */
        const at = (text: string,): number => KNOWN.findIndex((line,) => line.startsWith(text,),);
        /**
         The command that shows the record of the route watcher.
         */
        const record = at(`  sudo cat ${STATE_PATH}.watcher.json`,);
        expect(record,).toBeGreaterThan(-1,);
        expect(record,).toBeLessThan(at(`  sudo mv ${STATE_PATH} `,),);
        expect(record,).toBeLessThan(at(BLOCK_LINE,),);
        // The record is shown once: the by-hand list does not repeat it.
        expect(KNOWN.filter((line,) => line === `  sudo cat ${STATE_PATH}.watcher.json`,).length,).toBe(1,);
        expect(KNOWN.some((line,) => line.startsWith('first stop the route watcher, whichever way you go on',) && line.includes('adds rules again',),),).toBe(true,);
      },
    },),
    it({
      name: 'checks that the process ID is still the route watcher before it kills it: it prints the process list line first, says what it must show, and kills only after',
      fn: async () => {
        /**
         Position of a line that starts with the given text.
         */
        const at = (text: string,): number => WITH_PID.findIndex((line,) => line.startsWith(text,),);
        /**
         Where the check and the kill stand.
         */
        const record = at(`  sudo cat ${STATE_PATH}.watcher.json`,);
        const check = at(`  ps -o args= -p ${String(WATCHER_PID,)}`,);
        const kill = at(`  sudo kill ${String(WATCHER_PID,)}`,);
        expect(record,).toBeGreaterThan(-1,);
        expect(check,).toBeGreaterThan(record,);
        expect(kill,).toBe(check + 1,);
        expect(kill,).toBeLessThan(at(`  sudo mv ${STATE_PATH} `,),);
        // The remark between the record and the check says what the check must show and what to do otherwise.
        /**
         The remark above the check.
         */
        const remark = WITH_PID[check - 1] ?? '';
        expect(remark.startsWith('  # ',),).toBe(true,);
        expect(remark,).toContain(`ends with ${STATE_PATH}`,);
        expect(remark,).toContain('do not run the kill after it',);
        // No kill stands without the check before it.
        for (const [index, line,] of WITH_PID.entries()) {
          if (line.includes('kill',) && line.startsWith('  sudo ',))
            expect(WITH_PID.slice(0, index,).some((earlier,) => earlier.startsWith('  ps -o args= -p ',),),).toBe(true,);
        }
      },
    },),
    it({
      name: 'prints no command with a placeholder for the process ID: when the record gave none, the check and the kill are in a remark',
      fn: async () => {
        expect(KNOWN.some((line,) => line.startsWith('  ps ',) || line.startsWith('  sudo kill',),),).toBe(false,);
        expect(KNOWN.some((line,) => line.startsWith('  # ',) && line.includes('ps -o args= -p <pid>',) && line.includes('sudo kill <pid>',) && line.includes('do not kill it',),),).toBe(true,);
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
        expect(await wordsOf({ line: commands.find((line,) => line.startsWith('  sudo nft ',) && (!line.includes(' delete ',)),) ?? '', },),).toBe(`<nft><${blockStatements({ interfaceName: INTERFACE_NAME, },)}>\n`,);
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
