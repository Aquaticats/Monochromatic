import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
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

import {
  removePolicyRules,
  untaggedSuppressionRules,
  untaggedSuppressionWarning,
} from '../dist/final/node/tunnel-route.mjs';

/**
 Protocol tag this program writes on the main-table suppression it installs.
 */
const POLICY_RULE_PROTOCOL = '201';

/**
 Policy table an interface with automatic routing carries in its fwmark.
 */
const POLICY_TABLE = '52001';

/**
 Text of a program standing in for `wg` that reports the fwmark given.
 It is also asked for allowed-ips, which this file does not exercise.

 @param fwmark - What `wg show <interface> fwmark` must print.

 @returns The program text.
 */
function wgStandIn(fwmark: string,): string {
  return `#!/usr/bin/env node
const words = process.argv.slice(2);
if (words[0] === 'show' && words[2] === 'fwmark') {
  process.stdout.write(${JSON.stringify(`${fwmark}\n`,)});
  process.exit(0);
}
process.exit(1);
`;
}

/**
 Text of a program standing in for `ip` that records every call, answers a delete
 of a rule with the exact absence diagnostic when the call names the ownership protocol,
 and lists the given rules.

 @param calls - File the program appends each call to.

 @param taggedDeleteStatus - Exit status of a delete that names the protocol tag.

 @param listedRules - What `rule show` prints.

 @returns The program text.
 */
function ipStandIn(calls: string, taggedDeleteStatus: number, listedRules = '',): string {
  return `#!/usr/bin/env node
const fs = require('node:fs');
const words = process.argv.slice(2);
fs.appendFileSync(${JSON.stringify(calls,)}, words.join(' ') + '\\n',);
if (words.includes('show')) {
  process.stdout.write(${JSON.stringify(listedRules,)});
  process.exit(0);
}
if (!words.includes('delete'))
  process.exit(0);
if (words.includes('protocol') && !words.includes(${JSON.stringify(POLICY_RULE_PROTOCOL,)}))
  process.exit(0);
if (words.includes('protocol') && (${String(taggedDeleteStatus,)} !== 0)) {
  process.stderr.write('RTNETLINK answers: No such file or directory\\n');
  process.exit(${String(taggedDeleteStatus,)});
}
process.exit(0);
`;
}

/**
 Runs `removePolicyRules` for one interface over stand-in tools.

 @param fwmark - What the stand-in `wg` reports for the interface's fwmark.

 @param taggedDeleteStatus - Exit status the stand-in `ip` gives a delete naming the tag.

 @param listedRules - What the stand-in `ip` prints for `rule show`.

 @returns Every `ip` call the removal made, in the order they were recorded.
 */
async function callsOf(
  {
    fwmark,
    taggedDeleteStatus = 0,
    listedRules = '',
  }: {
    readonly fwmark: string;
    readonly taggedDeleteStatus?: number;
    readonly listedRules?: string;
  },
): Promise<readonly string[]> {
  /**
   Scratch directory of this call.
   */
  const scratch = await mkdtemp(join(scratchRoot(), 'wg-quicker-route-',),);
  /**
   Programs this call stands in for.
   */
  const bin = join(scratch, 'bin',);
  /**
   File the stand-in `ip` records its calls in.
   */
  const calls = join(scratch, 'calls.txt',);
  await mkdir(bin,);
  await writeFile(calls, '',);
  await Promise.all(
    ([
      ['wg', wgStandIn(fwmark,),],
      ['ip', ipStandIn(calls, taggedDeleteStatus, listedRules,),],
    ] as const).map(async ([name, text,]) => {
      const path = join(bin, name,);
      await writeFile(path, text,);
      await chmod(path, 0o700,);
    }),
  );
  /**
   The search path the removal sees,
   restored and the scratch folder removed however this call ends.
   */
  const previousPath = process.env.PATH;
  process.env.PATH = `${bin}:${previousPath ?? ''}`;
  await using restore = {
    async [Symbol.asyncDispose](): Promise<void> {
      if (previousPath === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = previousPath;
      await rm(scratch, { recursive: true, force: true, },);
    },
  };
  await removePolicyRules({ interfaceName: 'wgtest', },);
  return (await readFile(calls, 'utf8',)).trim()
    .split('\n',)
    .filter((line,) => line !== '');
}

await describe({
  name: removePolicyRules.name,
  // One at a time: each case owns the search path of the process.
  concurrency: 1,
  children: [
    it({
      name: 'removes no policy rule at all for an interface without a policy fwmark, so a config with a numeric Table never deletes a main-table suppression it did not create',
      fn: async () => {
        const calls = await callsOf({ fwmark: 'off', },);
        expect(calls.filter((call,) => call.includes('delete',),),).toEqual([],);
      },
    },),
    it({
      name: 'removes the tagged main-table suppression and the interface policy rule for an interface with automatic routing, and names the ownership protocol',
      fn: async () => {
        const calls = await callsOf({ fwmark: POLICY_TABLE, },);
        expect(calls.toSorted(),).toEqual([
          `-4 rule delete table ${POLICY_TABLE}`,
          `-4 rule delete table main suppress_prefixlength 0 protocol ${POLICY_RULE_PROTOCOL}`,
          `-6 rule delete table ${POLICY_TABLE}`,
          `-6 rule delete table main suppress_prefixlength 0 protocol ${POLICY_RULE_PROTOCOL}`,
        ],);
      },
    },),
    it({
      name: 'leaves an untagged main-table suppression in place, because it cannot be told apart from one an administrator or another tunnel installed, and says how to remove it',
      fn: async () => {
        const calls = await callsOf({
          fwmark: POLICY_TABLE,
          taggedDeleteStatus: 2,
          listedRules: '32765:\tfrom all lookup main suppress_prefixlength 0\n',
        },);
        expect(calls.filter((call,) => (call.includes('suppress_prefixlength',) && (!(call.includes('protocol',)))),),).toEqual([],);
        expect(calls.filter((call,) => call.endsWith('rule show',),).toSorted(),).toEqual([
          '-4 rule show',
          '-6 rule show',
        ],);
      },
    },),
    it({
      name: 'names the untagged suppression it leaves, the tag this program writes, and the command that removes the earlier build leftover',
      fn: async () => {
        const rule = '32765:\tfrom all lookup main suppress_prefixlength 0';
        expect(untaggedSuppressionRules({ listed: `${rule}\n32766:\tfrom all lookup main\n` },),).toEqual([rule],);
        expect(untaggedSuppressionRules({ listed: `32765: from all lookup main suppress_prefixlength 0 proto ${POLICY_RULE_PROTOCOL}\n` },),).toEqual([],);
        const warning = untaggedSuppressionWarning({
          proto: '-4',
          rule,
        },);
        expect(warning,).toContain(rule,);
        expect(warning,).toContain(POLICY_RULE_PROTOCOL,);
        expect(warning,).toContain('sudo ip -4 rule delete table main suppress_prefixlength 0',);
      },
    },),
  ],
},);
