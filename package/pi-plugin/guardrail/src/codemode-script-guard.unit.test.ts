/**
 Tests for the code-mode script guard, red and green.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  evaluateCodemodeScriptGuard,
  findRmCommandsBuiltAtRunTime,
  GUARDRAIL_NOT_BLOCKED,
  scriptLiterals,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Scripts that build an `rm` command at run time, with the word that gave them away.
 */
const BUILDING_SCRIPTS: readonly { readonly code: string; readonly word: string; }[] = [
  {
    code: `const d = "x";
const command = \`rm -rf \${d}\`;
const run = await tools.bash({ command });
text(run.output);
`,
    word: 'rm',
  },
  {
    code: `const d = "x";
const command = "/usr/bin/rm -rf " + d;
const run = await tools.bash({ command });
`,
    word: '/usr/bin/rm',
  },
  {
    code: `const command = \`rm\` + " -rf x";
const run = await tools.bash({ command });
`,
    word: 'rm',
  },
  {
    code: `const d = "x";
const command = \`rmdir -p \${d}\`;
const run = await tools.bash({ command });
`,
    word: 'rmdir',
  },
  {
    code: `const f = "x";
const command = "unlink " + f;
const run = await tools.bash({ command });
`,
    word: 'unlink',
  },
  {
    code: `const f = "x";
const command = \`shred -u \${f}\`;
const run = await tools.bash({ command });
`,
    word: 'shred',
  },
  {
    code: `const root = "x";
const command = \`find \${root} -delete\`;
const run = await tools.bash({ command });
`,
    word: 'find',
  },
];

/**
 Scripts this guard refuses although they hand no command to a shell.
 
 The guard reads literals, so it cannot prove which of them reaches `tools.bash`:
 a template that prints an `rm` line is refused like one that runs it.
 */
const OVER_REFUSED_SCRIPTS: readonly string[] = [
];

/**
 Scripts whose `rm` text the nested Bash call has to refuse instead.
 
 A literal that puts `sh` or another shell in command position names no `rm`
 of its own, so this guard reads no violation in it.
 */
const NESTED_SHELL_SCRIPTS: readonly string[] = [
  `const command = \`sh -c \\\`rm -rf \${d}\\\`\`;\nconst run = await tools.bash({ command });\n`,
];

/**
 Scripts whose text builds no `rm` command at run time.
 */
const INNOCENT_SCRIPTS: readonly string[] = [
  "const run = await tools.bash({ command: 'rm --recursive /home/user/temp/agent/scratch' });\n",
  'const run = await tools.bash({ command: "rm -rf input" });\n',
  `// rm -rf \${d} in a comment is text, not a command\nconst run = await tools.read({ path: "a.txt" });\n`,
  `const name = \`file-\${index}.txt\`;\nconst run = await tools.read({ path: name });\n`,
  "const run = await tools.bash({ command: 'find . -name build -delete' });\n",
  "const run = await tools.bash({ command: 'git rm --cached a.txt' });\n",
  "const run = await tools.bash({ command: 'rmdir --parents /home/user/temp/agent/x' });\n",
  "const run = await tools.bash({ command: 'find /home/user/temp/agent/x -print' });\n",
  `const text = 1;\nconst file = \`\${text}.txt\`;\n`,
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: scriptLiterals.name,
      children: [
        it({
          name: 'reads a quoted literal and a template literal in order',
          fn: async function testLiteralOrder() {
            expect(
              scriptLiterals('const a = \'one\'; const b = `two`;',)
                .map(function textOf(literal,): string {
                  return literal.text;
                },),
            ).toEqual(['one', 'two',],);
          },
        },),
        it({
          name: 'reads a template substitution as a space and marks it',
          fn: async function testSubstitution() {
            /**
             Literals of a script whose template holds one substitution.
             */
            const [literal,] = scriptLiterals(
              `const c = \`rm -rf \${d}\`;`,
            );
            expect(literal?.text,).toBe('rm -rf  ',);
            expect(literal?.holdsSubstitution,).toBe(true,);
          },
        },),
        it({
          name: 'reads no literal from a comment',
          fn: async function testComments() {
            expect(scriptLiterals('// \'a\'\n/* \'b\' */\n',),).toEqual([],);
          },
        },),
        it({
          name: 'marks a literal that stands beside a plus',
          fn: async function testPlusJoin() {
            /**
             Literals of a script that joins a quoted command to a value.
             */
            const literals = scriptLiterals('const c = "rm -rf " + d;',);
            expect(literals[0]?.joinedWithPlus,).toBe(true,);
          },
        },),
        it({
          name: 'marks a literal alone in its statement as no join',
          fn: async function testNoJoin() {
            expect(
              scriptLiterals('const c = "rm -rf /x";',)[0]?.joinedWithPlus,
            ).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: findRmCommandsBuiltAtRunTime.name,
      children: [
        ...BUILDING_SCRIPTS.map(function buildingScript(fixture,) {
          return it({
            name: `finds the command \`${fixture.code.split('\n',)[1]}\` builds`,
            fn: async function testBuildingScript() {
              /**
               Findings of one script.
               */
              const findings = findRmCommandsBuiltAtRunTime({ source: fixture.code, },);
              expect(findings.length,).toBeGreaterThan(0,);
              expect(findings[0]?.word,).toBe(fixture.word,);
            },
          },);
        },),
        ...OVER_REFUSED_SCRIPTS.map(function overRefusedScript(code,) {
          return it({
            name: `finds one in the over-refused \`${code.split('\n',)[0]}\``,
            fn: async function testOverRefusedScript() {
              expect(
                findRmCommandsBuiltAtRunTime({ source: code, },).length,
              ).toBeGreaterThan(0,);
            },
          },);
        },),
        ...NESTED_SHELL_SCRIPTS.map(function nestedShellScript(code,) {
          return it({
            name: `finds none in a script that hands a whole command to a shell`,
            fn: async function testNestedShellScript() {
              expect(findRmCommandsBuiltAtRunTime({ source: code, },),).toEqual([],);
            },
          },);
        },),
        ...INNOCENT_SCRIPTS.map(function innocentScript(code,) {
          return it({
            name: `finds none in \`${code.split('\n',)[0]}\``,
            fn: async function testInnocentScript() {
              expect(findRmCommandsBuiltAtRunTime({ source: code, },),).toEqual([],);
            },
          },);
        },),
      ],
    },),
    describe({
      name: evaluateCodemodeScriptGuard.name,
      children: [
        it({
          name: 'blocks a script that builds a computed deletion',
          fn: async function testBlocks() {
            /**
             Decision the guard gave for one building script.
             */
            const decision = evaluateCodemodeScriptGuard({
              input: { code: BUILDING_SCRIPTS[0]?.code, },
            },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('expected the script to be blocked',);
            expect(decision.reason,).toContain('builds a deletion command at run time',);
            expect(decision.reason,).toContain('`rm --recursive /home/user/temp/agent/<folder>/scratch`',);
          },
        },),
        it({
          name: 'allows a script that only runs a literal-path command',
          fn: async function testAllowsLiteral() {
            expect(
              evaluateCodemodeScriptGuard({
                input: { code: "const run = await tools.bash({ command: 'rm -rf /x' });\n", },
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows input that carries no script',
          fn: async function testNoScript() {
            expect(
              evaluateCodemodeScriptGuard({ input: { code: 1, }, },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(
              evaluateCodemodeScriptGuard({ input: 'text', },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
  ],
},);
