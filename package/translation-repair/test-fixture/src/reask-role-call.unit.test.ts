/**
 Tests for reading the role a call names from the source text (ledger B1079).

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from './reask-role-call.test-fixture.ts';

//region Re-ask role call tests

/**
 Source of a caller that names a role beside a prompt whose message has a role of its own.
 */
const NAMED = [
  'const gather = await gatherStageVoices({',
  '  client,',
  '  messages: [{ role: \'user\', content: "meow", },],',
  '  // role: \'late-judge\' in a comment is no role',
  '  stage: `panel`,',
  '  role: \'wide-judge\',',
  '},);',
].join('\n',);

await describe({
  name: rolesNamedAtCalls.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS THE ROLE A CALL NAMES, not the role of a message in its prompt or of a comment',
      fn: async () => {
        expect(rolesNamedAtCalls({ source: NAMED, callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'READS A ROLE PASSED BY NAME and the shorthand, so a stage that takes its role from its caller shows it',
      fn: async () => {
        const source = [
          'await gatherStageVoices<Wire>({',
          '  role: bench.role,',
          '},);',
          'await gatherStageVoices({',
          '  client,',
          '  role,',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['bench.role', 'role',],);
      },
    },),

    it({
      name: 'REPORTS NONE FOR A CALL THAT NAMES NO ROLE, and skips an example written on one line and a declaration',
      fn: async () => {
        const source = [
          ' const gather = await gatherStageVoices({ ..., role: \'checker\', },);',
          'export async function gatherStageVoices({',
          '  role,',
          '},) {}',
          'await gatherStageVoices({',
          '  messages: [{ role: \'user\', },],',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['none',],);
      },
    },),
  ],
},);

//endregion Re-ask role call tests
