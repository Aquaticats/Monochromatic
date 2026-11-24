/**
 Tests for the literal-`rm`-path detector.
 
 Every case names the branch it covers, so a passing run can be compared
 against the forms the detector claims to read.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findLiteralRmPaths,
  findLiteralRmPathsInScriptFile,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 The two lines of the script a subagent wrote to route around this rule.
 
 Kept as the first red fixture, so the observed bypass is a test rather than
 a claim.
 */
const OBSERVED_BYPASS_LINES: readonly string[] = [
  'rm -rf "$d"',
  'rm -f "$d/$n"',
];

/**
 Commands whose `rm` names a path the shell computes, with the offending word.
 */
const COMPUTED_PATH_COMMANDS: readonly { readonly source: string; readonly word: string; }[] = [
  { source: 'rm -rf "$d"', word: '"$d"', },
  { source: 'rm -f "$d/$n"', word: '"$d/$n"', },
  { source: 'rm -rf $d', word: '$d', },
  { source: `rm -rf "\${d}"`, word: `"\${d}"`, },
  { source: `rm -rf "\${HOME}/scratch"`, word: `"\${HOME}/scratch"`, },
  { source: 'rm -rf "$HOME/temp"', word: '"$HOME/temp"', },
  { source: 'rm -rf "$(pwd)/scratch"', word: '"$(pwd)/scratch"', },
  { source: 'rm -rf `pwd`/scratch', word: '`pwd`/scratch', },
  { source: 'rm -rf $((1 + 1))', word: '$((1 + 1))', },
  { source: 'rm -rf "$@"', word: '"$@"', },
  { source: 'rm --recursive "$dir"', word: '"$dir"', },
];

/**
 Spellings of one `rm` behind a program wrapper, each naming its path at run time.
 */
const WRAPPED_RM_COMMANDS: readonly string[] = [
  'sudo rm -rf "$d"',
  'sudo -u root rm -rf "$d"',
  'env rm -rf "$d"',
  'env X=1 rm -rf "$d"',
  'timeout 5 rm -rf "$d"',
  'systemd-run --user --scope rm -rf "$d"',
  'flock /repo.lock rm -rf "$d"',
  'nice -n 5 rm -rf "$d"',
  'command rm -rf "$d"',
  'builtin rm -rf "$d"',
  'exec rm -rf "$d"',
  'nohup rm -rf "$d"',
  'ionice rm -rf "$d"',
  'stdbuf -o 0 rm -rf "$d"',
  'doas rm -rf "$d"',
  'setsid rm -rf "$d"',
  'time rm -rf "$d"',
  'taskset 1 rm -rf "$d"',
  'chrt 5 rm -rf "$d"',
];

/**
 Spellings of the program word that name `rm`, each naming its path at run time.
 */
const RM_PROGRAM_SPELLINGS: readonly string[] = [
  '/usr/bin/rm -rf "$d"',
  './rm -rf "$d"',
  String.raw`\rm -rf "$d"`,
  'RM_DIR=1 rm -rf "$d"',
];

/**
 Commands that hand the `rm` to a shell or to `eval` as a script string.
 */
const SCRIPT_STRING_COMMANDS: readonly string[] = [
  'bash -c \'rm -rf "$d"\'',
  'sh -c \'rm -rf "$d"\'',
  'dash -c \'rm -rf "$d"\'',
  'zsh -c \'rm -rf "$d"\'',
  'ksh -c \'rm -rf "$d"\'',
  'busybox sh -c \'rm -rf "$d"\'',
  'env bash -lc \'rm -rf "$d"\'',
  'eval \'rm -rf "$d"\'',
  'ssh build-host \'rm -rf "$d"\'',
];

/**
 Commands whose `rm` paths come from outside the `rm` itself.
 */
const PATH_SUPPLIED_RM_COMMANDS: readonly string[] = [
  String.raw`printf '%s\n' build | xargs rm -rf`,
  String.raw`printf '%s\n' build | xargs -n 1 rm -f`,
  String.raw`find . -name build -exec rm -rf {} \;`,
  'find . -name build -execdir rm -rf {} +',
  String.raw`find . -name build -ok rm -rf {} \;`,
  String.raw`find . -name build -delete -o -exec rm -rf {} \;`,
  String.raw`env find . -exec rm -rf {} \;`,
];

/**
 Commands whose `rm` writes every path out, which must stay allowed.
 */
const LITERAL_PATH_COMMANDS: readonly string[] = [
  'rm --recursive /home/user/temp/agent/build-literal-rm-guard/scratch',
  'rm -rf /home/user/temp/agent/scratch',
  'rm -f ./build.log',
  'rm -rf ~/temp/agent/scratch',
  'rm -rf *.log',
  'rm -rf build/*',
  'rm -rf \'$d\'',
  'rm -rf \'$d/$n\'',
  'rm -rf "$"',
  String.raw`rm -rf file\ name.txt`,
  'rm',
  'rm --version',
];

/**
 Commands that name `rm` as another program's subcommand, which must stay allowed.
 */
const SUBCOMMAND_RM_COMMANDS: readonly string[] = [
  'git rm -rf "$d"',
  'git rm --cached package.json',
  'cargo rm "$crate"',
  'npm rm "$package"',
  'pnpm rm "$package"',
  'podman rm -f "$container"',
  'docker rm -f "$container"',
  'gh release delete "$tag"',
];

/**
 Deleting programs this guard does not read yet, kept allowed until the owner decides.
 */
const NOT_READ_DELETING_COMMANDS: readonly string[] = [
  'rmdir "$d"',
  'unlink "$d"',
  'shred -u "$d"',
  'find . -name build -delete',
  'printf \'%s\' "$d" | xargs rmdir',
];

/**
 Prose that mentions an `rm` line without running one.
 */
const PROSE_COMMANDS: readonly string[] = [
  'echo \'rm -rf "$d"\'',
  String.raw`echo "rm -rf \$d"`,
  String.raw`printf '%s\n' 'rm -rf "$d"' > notes.txt`,
  'cat <<\'EOF\'\n# never run this\necho rm -rf "$d"\nEOF',
];

//endregion Fixtures

//region Expectations

/**
 Reads the offending words of one command, in order.
 
 @param source - shell source to read
 
 @returns offending words the detector reported
 
 @example
 ```ts
 offendingWords('rm -rf "$d"'); // ['"$d"']
 ```
 */
function offendingWords(source: string,): readonly string[] {
  return findLiteralRmPaths({ source, },)
    .map(function wordOf(finding,): string {
      return finding.word;
    },);
}

//endregion Expectations

await describe({
  name: '',
  children: [
    describe({
      name: findLiteralRmPaths.name,
      children: [
        //region The observed bypass
        describe({
          name: 'the script a subagent wrote to route around this rule',
          children: [
            ...OBSERVED_BYPASS_LINES.map(function observedLine(line,) {
              return it({
                name: `refuses \`${line}\``,
                fn: async function testObservedBypassLine() {
                  expect(offendingWords(line,),).toHaveLength(1,);
                },
              },);
            },),
            it({
              name: 'refuses both lines of the script, each on its own line',
              fn: async function testObservedScript() {
                /**
                 The observed script's two deletions, joined as the runner wrote them.
                 */
                const script = OBSERVED_BYPASS_LINES
                  .join('\n',);
                /**
                 Findings the script produced.
                 */
                const findings = findLiteralRmPathsInScriptFile({
                  path: '/tmp/observed.sh',
                  source: script,
                },);
                expect(findings,).toHaveLength(2,);
                expect(findings[0]?.line,).toBe(1,);
                expect(findings[1]?.line,).toBe(2,);
                expect(findings[1]?.file,).toBe('/tmp/observed.sh',);
              },
            },),
            it({
              name: 'refuses the first deletion when a second command follows it',
              fn: async function testObservedSemicolon() {
                expect(
                  offendingWords('rm -rf "$d"; mkdir -p "$d"',),
                ).toEqual(['"$d"',],);
              },
            },),
          ],
        },),
        //endregion The observed bypass

        //region Computed paths
        describe({
          name: 'an rm whose operands the shell computes',
          children: [
            ...COMPUTED_PATH_COMMANDS.map(function computedPathCommand(fixture,) {
              return it({
                name: `refuses \`${fixture.source}\` naming \`${fixture.word}\``,
                fn: async function testComputedPathCommand() {
                  expect(offendingWords(fixture.source,),).toContain(fixture.word,);
                },
              },);
            },),
            it({
              name: 'refuses every operand holding an expansion, not only the first',
              fn: async function testSeveralOperands() {
                expect(offendingWords('rm -rf "$a" "$b"',),).toEqual(['"$a"', '"$b"',],);
              },
            },),
            it({
              name: 'refuses an rm whose path a nested command substitution builds',
              fn: async function testNestedSubstitution() {
                expect(
                  offendingWords('rm -rf "$(dirname "$0")/build"',),
                ).toContain('"$(dirname "$0")/build"',);
              },
            },),
          ],
        },),
        //endregion Computed paths

        //region Wrappers and program spellings
        describe({
          name: 'an rm behind a program wrapper',
          children: [
            ...WRAPPED_RM_COMMANDS.map(function wrappedCommand(source,) {
              return it({
                name: `refuses \`${source}\``,
                fn: async function testWrappedCommand() {
                  expect(offendingWords(source,),).toEqual(['"$d"',],);
                },
              },);
            },),
            ...RM_PROGRAM_SPELLINGS.map(function programSpelling(source,) {
              return it({
                name: `refuses \`${source}\``,
                fn: async function testProgramSpelling() {
                  expect(offendingWords(source,),).toHaveLength(1,);
                },
              },);
            },),
            it({
              name: 'refuses an rm a function body stores',
              fn: async function testFunctionBody() {
                expect(offendingWords('cleanup(){ rm -rf "$d"; }; cleanup',),).toEqual(['"$d"',],);
              },
            },),
          ],
        },),
        //endregion Wrappers and program spellings

        //region Script strings
        describe({
          name: 'an rm a shell or eval runs from a word',
          children: SCRIPT_STRING_COMMANDS
            .map(function scriptStringCommand(source,) {
              return it({
                name: `refuses \`${source}\``,
                fn: async function testScriptStringCommand() {
                  expect(offendingWords(source,),).toEqual(['"$d"',],);
                },
              },);
            },),
        },),
        //endregion Script strings

        //region Paths supplied from outside
        describe({
          name: 'an rm whose paths another program supplies',
          children: PATH_SUPPLIED_RM_COMMANDS
            .map(function suppliedPathCommand(source,) {
              return it({
                name: `refuses \`${source}\``,
                fn: async function testSuppliedPathCommand() {
                  expect(offendingWords(source,),).toEqual(['rm',],);
                },
              },);
            },),
        },),
        //endregion Paths supplied from outside

        //region Here-document bodies
        describe({
          name: 'an rm written inside a here-document body',
          children: [
            it({
              name: 'refuses a body a shell reads with its delimiter quoted',
              fn: async function testQuotedDelimiterBody() {
                expect(
                  offendingWords('bash <<\'EOF\'\nrm -rf "$d"\nEOF',),
                ).toEqual(['"$d"',],);
              },
            },),
            it({
              name: 'refuses a body written to a file and run afterwards',
              fn: async function testWrittenBody() {
                expect(
                  offendingWords('{ cat <<EOF\nrm -rf "$d"\nEOF\n} > /tmp/x.sh',),
                ).toEqual(['"$d"',],);
              },
            },),
            it({
              name: 'refuses a body fed to the shell by a tab-stripping redirect',
              fn: async function testStrippedBody() {
                expect(
                  offendingWords('bash <<-EOF\n\trm -rf "$d"\n\tEOF',),
                ).toEqual(['"$d"',],);
              },
            },),
            it({
              name: 'allows a body whose own text holds no rm',
              fn: async function testInnocentBody() {
                expect(offendingWords('cat <<\'EOF\'\nhello\nEOF',),).toEqual([],);
              },
            },),
          ],
        },),
        //endregion Here-document bodies

        //region A command the parser rejects
        describe({
          name: 'a command the parser rejects',
          children: [
            it({
              name: 'refuses an rm the visible text shows, unterminated quote and all',
              fn: async function testUnparsableCommand() {
                expect(offendingWords('rm -rf "$d" "unterminated',),).toEqual(['$d',],);
              },
            },),
            it({
              name: 'refuses an rm behind a wrapper in a command the parser rejects',
              fn: async function testUnparsableWrappedCommand() {
                expect(offendingWords('cd x && sudo rm -rf "$d" "unterminated',),).toEqual(['$d',],);
              },
            },),
            it({
              name: 'refuses a raw word carrying only a backtick',
              fn: async function testUnparsableBacktick() {
                expect(offendingWords('rm -rf `pwd`/x "unterminated',),).toEqual(['`pwd`/x',],);
              },
            },),
            it({
              name: 'allows prose an unterminated quote leaves in no command position',
              fn: async function testUnparsableProse() {
                expect(offendingWords('echo "rm -rf $d',),).toEqual([],);
              },
            },),
          ],
        },),
        //endregion A command the parser rejects

        //region Literal forms
        describe({
          name: 'a form that writes every path out',
          children: LITERAL_PATH_COMMANDS
            .map(function literalCommand(source,) {
              return it({
                name: `allows \`${source}\``,
                fn: async function testLiteralCommand() {
                  expect(offendingWords(source,),).toEqual([],);
                },
              },);
            },),
        },),
        //endregion Literal forms

        //region Other programs
        describe({
          name: 'a program that is not rm',
          children: [
            ...SUBCOMMAND_RM_COMMANDS.map(function subcommandCommand(source,) {
              return it({
                name: `allows \`${source}\``,
                fn: async function testSubcommandCommand() {
                  expect(offendingWords(source,),).toEqual([],);
                },
              },);
            },),
            ...NOT_READ_DELETING_COMMANDS.map(function notReadCommand(source,) {
              return it({
                name: `allows \`${source}\`, which this guard does not read yet`,
                fn: async function testNotReadCommand() {
                  expect(offendingWords(source,),).toEqual([],);
                },
              },);
            },),
            ...PROSE_COMMANDS.map(function proseCommand(source,) {
              return it({
                name: `allows prose \`${source.split('\n')
                  .join(String.raw`\n`)}\``,
                fn: async function testProseCommand() {
                  expect(offendingWords(source,),).toEqual([],);
                },
              },);
            },),
          ],
        },),
        //endregion Other programs

        //region Nesting
        describe({
          name: 'a script that launches another script',
          children: [
            it({
              name: 'refuses a second shell layer',
              fn: async function testSecondShellLayer() {
                expect(
                  offendingWords(String.raw`bash -c "bash -c 'rm -rf \$d'"`,),
                ).toHaveLength(1,);
              },
            },),
            it({
              name: 'terminates on a command that runs its own text',
              fn: async function testSelfLaunchingCommand() {
                expect(
                  findLiteralRmPaths({ source: 'eval "$0"', },),
                ).toEqual([],);
              },
            },),
          ],
        },),
        //endregion Nesting
      ],
    },),
  ],
},);
