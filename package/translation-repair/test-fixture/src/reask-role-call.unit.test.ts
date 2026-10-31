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
import {
  keysNamedInLiteralsOfType,
  keysNamedInLiteralsUnder,
  roleNamedInLiteral,
  rolesNamedAtCalls,
} from './reask-role-call.test-fixture.ts';

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
      name: 'REPORTS NONE FOR A CALL THAT NAMES NO ROLE, and skips an example written in a comment and a declaration',
      fn: async () => {
        const source = [
          '/**',
          ' const gather = await gatherStageVoices({ ..., role: \'checker\', },);',
          ' */',
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

    it({
      name: 'READS A CALL THAT HANDS ONLY A SPREAD AS \'only a spread\' AND NOT AS \'none\' (ledger B1150), since the role may ride '
        + 'in what it spreads; a conditional spread that names the role counts, and one that names other keys does not',
      fn: async () => {
        const source = [
          'await gatherStageVoices({ ...args, },);',
          'await gatherStageVoices({',
          '  client,',
          '  ...base,',
          '},);',
          'await gatherStageVoices({',
          '  ...(late ? { role: \'roster\', } : {}),',
          '},);',
          'await gatherStageVoices({',
          '  client,',
          '  ...((fanOut === undefined) ? {} : { fanOut, }),',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'only a spread',
          'only a spread',
          'only a spread',
          'none',
        ],);
      },
    },),

    it({
      name: 'COUNTS A SPREAD OF A CALL WITH AN OBJECT ARGUMENT as one that can replace the role, since a call can return '
        + 'anything however small its argument',
      fn: async () => {
        const source = [
          'await gatherStageVoices({',
          '  role: \'wide-judge\',',
          '  ...build({ a: 1, },),',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['wide-judge then a spread',],);
      },
    },),

    it({
      name: 'READS A ROLE WRITTEN AS A COMPUTED KEY, [\'role\']: ..., as the role named',
      fn: async () => {
        const source = [
          'await gatherStageVoices({',
          '  [\'role\']: \'checker\',',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['checker',],);
      },
    },),

    it({
      name: 'FINDS THE DECLARATION OF A LITERAL AND NOT AN EARLIER USE OF ITS NAME (the case be93348926 lacked): a name '
        + 'passed to a call before it is declared does not stand for the declaration',
      fn: async () => {
        const source = [
          'use(judging,);',
          'const judging = { role: \'late-judge\', };',
        ].join('\n',);
        expect(roleNamedInLiteral({ source, name: 'judging', },),).toBe('late-judge',);
      },
    },),

    it({
      name: 'REFUSES A CALLEE THE SOURCE NEVER NAMES, so a test that names the wrong function fails and does not read as '
        + 'a file with no call',
      fn: async () => {
        const source = [
          'await runWindowedRounds({',
          '  role: \'roster\',',
          '},);',
        ].join('\n',);
        expect(function readsTheWrongCallee() {
          rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },);
        },).toThrow('never names gatherStageVoices',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['roster',],);
      },
    },),

    it({
      name: 'SEES A SPREAD AFTER THE ROLE, which can replace it, and a role that comes after a spread as the role named',
      fn: async () => {
        const source = [
          'await gatherStageVoices({',
          '  role: \'wide-judge\',',
          '  ...base,',
          '},);',
          'await gatherStageVoices({',
          '  ...base,',
          '  role: \'wide-judge\',',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'wide-judge then a spread',
          'wide-judge',
        ],);
      },
    },),

    it({
      name: 'SEES A DUPLICATE ROLE KEY, naming both values, since the later one wins',
      fn: async () => {
        const source = [
          'await gatherStageVoices({',
          '  role: \'wide-judge\',',
          '  role: \'roster\',',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'duplicate role: wide-judge, roster',
        ],);
      },
    },),

    it({
      name: 'READS CALLS WITHOUT AWAIT: a returned call, a call kept in a variable and a call on one line',
      fn: async () => {
        const source = [
          'function a() {',
          '  return gatherStageVoices({',
          '    role: \'checker\',',
          '  },);',
          '}',
          'const pending = gatherStageVoices({',
          '  role: \'roster\',',
          '},);',
          'const quick = await gatherStageVoices({ client, role: \'writer\', },);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'checker',
          'roster',
          'writer',
        ],);
      },
    },),

    it({
      name: 'REPORTS A CALL WHOSE ARGUMENT IS NOT AN OBJECT LITERAL, which can carry any role',
      fn: async () => {
        const source = [
          'await gatherStageVoices(args,);',
          'const other = gatherStageVoices(build({ role: \'checker\', },),);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'not an object literal',
          'not an object literal',
        ],);
      },
    },),

    it({
      name: 'READS ANOTHER KEY THAN role, as a stage that takes the role of its judges by judgeRole shows it',
      fn: async () => {
        const source = [
          'await runRefineStage({',
          '  role: \'refiner\',',
          '  judgeRole: \'wide-judge\',',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'runRefineStage', key: 'judgeRole', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'READS A KEY INSIDE AN OBJECT THE CALL PASSES, by a dotted key, for a stage that hands its role in a nested '
        + 'argument',
      fn: async () => {
        const source = [
          'await judgeSlateWithRetry({',
          '  judging: {',
          '    client,',
          '    role: \'wide-judge\',',
          '  },',
          '  role: \'roster\',',
          '},);',
          'await judgeSlateWithRetry({',
          '  judging,',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'judgeSlateWithRetry', key: 'judging.role', },),).toEqual([
          'wide-judge',
          'none',
        ],);
      },
    },),

    it({
      name: 'READS THE KEY OF AN OBJECT LITERAL KEPT IN A VARIABLE, as a stage that builds one argument object for two '
        + 'calls does, with a type annotation before the equals sign',
      fn: async () => {
        const source = [
          'const judging: Parameters<typeof judgeTranslateSlate>[0] = {',
          '  client,',
          '  role: \'late-judge\',',
          '};',
          'const other = { role: \'roster\', };',
        ].join('\n',);
        expect(roleNamedInLiteral({ source, name: 'judging', },),).toBe('late-judge',);
        expect(roleNamedInLiteral({ source, name: 'other', },),).toBe('roster',);
        expect(roleNamedInLiteral({ source, name: 'missing', },),).toBe('no such literal',);
      },
    },),

    it({
      name: 'DOES NOT COUNT A SPREAD OF AN INLINE OBJECT THAT NAMES OTHER KEYS as one that can replace the role, and does '
        + 'count one that names the role itself',
      fn: async () => {
        const source = [
          'await gatherStageVoices({',
          '  role: \'wide-judge\',',
          '  ...((fanOut === undefined) ? {} : { fanOut, }),',
          '},);',
          'await gatherStageVoices({',
          '  role: \'wide-judge\',',
          '  ...(late ? { role: \'roster\', } : {}),',
          '},);',
        ].join('\n',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual([
          'wide-judge',
          'wide-judge then a spread',
        ],);
      },
    },),
  ],
},);

await describe({
  name: keysNamedInLiteralsUnder.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS WHAT EVERY LITERAL WRITTEN UNDER A PROPERTY NAME NAMES UNDER A KEY (ledger B1163): the key, none where it '
        + 'is absent, a spread alone, and no entry for a property that holds a variable',
      fn: async () => {
        const source = [
          'const a = { reaskSeat: { role, bench, reask: \'other-model\', }, };',
          'const b = { reaskSeat: { role, bench, }, };',
          'const c = { reaskSeat: { ...reaskSeat, otherModelMessages, }, };',
          'const d = { reaskSeat, };',
          'const e = { reaskSeat: seat, };',
          '// reaskSeat: { reask: \'measured-model\', } in a comment is no literal',
        ].join('\n',);
        expect(keysNamedInLiteralsUnder({ source, holder: 'reaskSeat', key: 'reask', },),).toEqual([
          'other-model',
          'none',
          'only a spread',
        ],);
      },
    },),
  ],
},);

await describe({
  name: keysNamedInLiteralsOfType.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS WHAT A LITERAL ASSIGNED TO A VARIABLE OF A TYPE NAMES UNDER A KEY (ledger B1163), and no other variable',
      fn: async () => {
        const source = [
          'const a: ReaskSeat = { role: \'reader\', bench, };',
          'const b: ReaskSeat = { role, bench, reask, };',
          'const c: Other = { role, };',
          'const d: ReaskSeat = build();',
        ].join('\n',);
        expect(keysNamedInLiteralsOfType({ source, type: 'ReaskSeat', key: 'reask', },),).toEqual(['none', 'reask',],);
      },
    },),
  ],
},);

//endregion Re-ask role call tests
