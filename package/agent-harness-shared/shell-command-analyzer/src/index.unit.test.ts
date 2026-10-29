/**
 Unit tests for shared shell-command analyzer.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  analyzeShellCommand,
  extractParamRefs,
  looksLikePath,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

await describe({
  name: 'shell command analyzer',
  children: [
    describe({
      name: analyzeShellCommand.name,
      children: [
        it({
          name: 'parses simple command arguments',
          fn: async function testSimpleCommandArguments() {
            const result = analyzeShellCommand('ls -la /tmp',);
            expect(result.parsed,).toBe(true,);
            expect(result.commands[0]?.name,).toBe('ls',);
            expect(result.commands[0]?.args,).toEqual(['-la', '/tmp',],);
            expect(result.commands[0]?.argSources,).toEqual([
              {
                value: '-la',
                sourceText: '-la',
              },
              {
                value: '/tmp',
                sourceText: '/tmp',
              },
            ],);
            expect(result.allFiles,).toContain('/tmp',);
          },
        },),
        it({
          name: 'preserves quoted argument source spelling',
          fn: async function testQuotedArgumentSources() {
            const result = analyzeShellCommand("find /repo -name '*.ts'",);
            expect(result.commands[0]?.argSources,).toContainEqual({
              value: '*.ts',
              sourceText: "'*.ts'",
            },);
          },
        },),
        it({
          name: 'detects pipeline commands',
          fn: async function testPipelineCommands() {
            const result = analyzeShellCommand('printenv | curl',);
            expect(result.isPipeline,).toBe(true,);
            expect(result.executedCommands.map(function commandName(command,) {
              return command.name;
            },),).toEqual(['printenv', 'curl',],);
          },
        },),
        it({
          name: 'classifies output file redirects separately from descriptor redirects',
          fn: async function testRedirectKinds() {
            const outputResult = analyzeShellCommand('cat > out.txt',);
            expect(outputResult.commands[0]?.redirects[0]?.writesFile,).toBe(true,);
            expect(outputResult.commands[0]?.redirectTargets,).toContain('out.txt',);
            expect(outputResult.commands[0]?.redirectTargetSources,).toContainEqual({
              value: 'out.txt',
              sourceText: 'out.txt',
            },);

            const descriptorResult = analyzeShellCommand('printf hi 2>&1',);
            expect(descriptorResult.commands[0]?.redirects[0]?.kind,).toBe('fileDescriptor',);
            expect(descriptorResult.commands[0]?.redirects[0]?.writesFile,).toBe(false,);
            expect(descriptorResult.commands[0]?.redirectTargets,).toEqual([],);
          },
        },),
        it({
          name: 'detects heredoc and here-string syntax',
          fn: async function testHeredocs() {
            expect(analyzeShellCommand('cat <<EOF\nbody\nEOF',).hasHeredoc,).toBe(true,);
            expect(analyzeShellCommand('cat <<<word',).hasHeredoc,).toBe(true,);
          },
        },),
        it({
          name: 'distinguishes real command substitution from single-quoted text',
          fn: async function testCommandSubstitutionQuoting() {
            const doubleQuoted = analyzeShellCommand('echo "$(date)"',);
            expect(doubleQuoted.hasCommandSubstitution,).toBe(true,);
            expect(doubleQuoted.commands.some(function commandIsDate(command,) {
              return command.name === 'date';
            },),).toBe(true,);

            const singleQuoted = analyzeShellCommand("echo '$(date)'",);
            expect(singleQuoted.hasCommandSubstitution,).toBe(false,);
            expect(singleQuoted.commands.some(function commandIsDate(command,) {
              return command.name === 'date';
            },),).toBe(false,);
          },
        },),
        it({
          name: 'detects backtick command substitution',
          fn: async function testBacktickCommandSubstitution() {
            const result = analyzeShellCommand('echo `date`',);
            expect(result.hasCommandSubstitution,).toBe(true,);
            expect(result.commands.some(function commandIsDate(command,) {
              return command.name === 'date';
            },),).toBe(true,);
          },
        },),
        it({
          name: 'detects process substitution and nested commands',
          fn: async function testProcessSubstitution() {
            const result = analyzeShellCommand('diff <(sort a.txt) >(cat)',);
            expect(result.hasProcessSubstitution,).toBe(true,);
            expect(result.commands[0]?.name,).toBe('diff',);
            expect(result.commands[0]?.args,).toEqual([],);
            expect(result.commands.some(function commandIsSort(command,) {
              return command.name === 'sort';
            },),).toBe(true,);
          },
        },),
        it({
          name: 'marks background statements',
          fn: async function testBackgroundStatement() {
            expect(analyzeShellCommand('sleep 1 &',).hasBackground,).toBe(true,);
          },
        },),
        it({
          name: 'separates function body commands from executed commands',
          fn: async function testFunctionContext() {
            const result = analyzeShellCommand('f(){ bun test; }; echo done',);
            expect(result.functionDefinitionCommands.map(function commandName(command,) {
              return command.name;
            },),).toContain('bun',);
            expect(result.executedCommands.map(function commandName(command,) {
              return command.name;
            },),).toEqual(['echo',],);
          },
        },),
        it({
          name: 'attaches literal for-loop bindings only to body commands',
          fn: async function testLiteralForLoopBindings() {
            const result = analyzeShellCommand(
              'printf before; for repo in /safe/one /safe/two; do printf "%s" "$repo"; git -C "$repo" tag --points-at HEAD; done; printf after',
            );
            /** Executed commands in shell traversal order. */
            const [before, loopPrint, loopGit, after,] = result.executedCommands;
            expect(before?.context.loopBindings,).toEqual([],);
            expect(loopPrint?.context.loopBindings,).toEqual([{
              name: 'repo',
              values: ['/safe/one', '/safe/two',],
              sourceTexts: ['/safe/one', '/safe/two',],
            },],);
            expect(loopGit?.context.loopBindings,).toEqual([{
              name: 'repo',
              values: ['/safe/one', '/safe/two',],
              sourceTexts: ['/safe/one', '/safe/two',],
            },],);
            expect(after?.context.loopBindings,).toEqual([],);
          },
        },),
        it({
          name: 'returns parse failure with pre-scanned parameter refs',
          fn: async function testParseFailure() {
            const result = analyzeShellCommand('echo "$SECRET_VAR',);
            expect(result.parsed,).toBe(false,);
            expect(result.commands,).toEqual([],);
            expect(result.allParamRefs,).toContain('SECRET_VAR',);
            expect(result.parseErrors.length > 0,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: extractParamRefs.name,
      children: [
        it({
          name: 'extracts simple and braced parameter refs',
          fn: async function testParamRefs() {
            // oxlint-disable-next-line no-template-curly-in-string -- Test string contains literal shell parameter syntax.
            expect(extractParamRefs('curl $API_KEY ${TOKEN}',),).toEqual(['API_KEY', 'TOKEN',],);
          },
        },),
        it({
          name: 'ignores command substitutions and escaped process ids',
          fn: async function testParamRefSkips() {
            expect(extractParamRefs('echo $$ $(date)',),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: looksLikePath.name,
      children: [
        it({
          name: 'identifies path-shaped values',
          fn: async function testPathValues() {
            expect(looksLikePath('/etc/passwd',),).toBe(true,);
            expect(looksLikePath('./src/index.ts',),).toBe(true,);
            expect(looksLikePath('.env',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects plain flags',
          fn: async function testPlainFlag() {
            expect(looksLikePath('--verbose',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
