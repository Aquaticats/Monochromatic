/**
 Checks that the shapes the root `mise.toml` shares between task bodies are written once, in its `[vars]` section.

 The root task file already keeps its shared task-body scripts in `[vars]` (`fanout`, `fanout_packages`, `run_test_files`,
 `parse_usage_args`, `dispatch_workspace_node`, `run_playwright`). Two more shapes were hand-copied into the bodies that
 needed them: the predicate that skips deprecated and paused trees, and the raised stdout cap for reading the task list.
 Either copy can drift from the other with nothing to notice, so the checks below hold both to one definition and to an
 interpolation at every site that needs it.

 The counts are relations rather than numbers: the predicate has exactly as many readers as the file has test-tree
 walkers, and the cap has exactly as many readers as the file has task-list reads.

 The forwarded-arguments marker is a third shared shape. mise appends the words a task forwards to its inline script
 (the `-h`/`--help` case after `--`) past the TypeScript, so every body that reads `vars.parse_usage_args` ends with the
 marker comment those words land in. One check holds each body to that ending and one runs the parser against a
 synthetic paste, so the words are shown to arrive as arguments rather than as source.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { execFileSync, } from 'node:child_process';
import { readFile, } from 'node:fs/promises';

//region The file and its shapes

/**
 The root task file every package task inherits from.
 */
const taskFile = await readFile(new URL('../../../../mise.toml', import.meta.url,), 'utf8',);

/**
 Name of the `[vars]` table that owns the shared task bodies.
 */
const varsHeader = '[vars]';

/**
 The shared snippet that answers whether a test file sits in a deprecated or paused tree.
 */
const inactivitySnippet = '{{vars.is_inactive_test}}';

/**
 The shared snippet that declares the raised stdout cap for `mise tasks --json`.
 */
const bufferSnippet = '{{vars.mise_tasks_json_max_buffer}}';

/**
 The declaration each snippet must have exactly one of.
 */
const inactivityDeclaration = 'const isInactiveTest = ';

/**
 The declaration of the stdout cap.
 */
const bufferDeclaration = 'const MISE_TASKS_JSON_MAX_BUFFER = ';

/**
 Text that starts every task-list read whose output needs the raised cap.
 */
const taskListRead = "execFileSync('mise', ['tasks'";

/**
 Text that starts every walk of the test tree, all of which consult the inactivity predicate.
 */
const testTreeWalk = "for await (const file of glob('**/*.unit.test.ts'";

/**
 Text that starts every task body that parses post-`--` arguments with the shared parser.
 */
const parserSnippet = '{{vars.parse_usage_args}}';

/**
 The comment line every such body ends with, where mise appends the arguments it forwards.
 */
const forwardedMarkerSnippet = '{{vars.forwarded_arguments_marker}}';

/**
 The declaration of the marker's text.
 */
const forwardedMarkerDeclaration = 'forwarded_arguments_marker = "';

/**
 Text that a parser call ends with when it reads a named usage variable and the appended fallback.
 */
const namedArgumentRecovery = '?? forwardedArgumentWords()';

/**
 Text that a parser call ends with when it recovers `usage_args` or the appended words.
 */
const unnamedArgumentRecovery = 'parseUsageArgs(usageArgWords())';

//endregion

//region Reading the file

/**
 How many times a fixed text occurs.

 @param text - text to search

 @param needle - text to count

 @returns the count of non-overlapping occurrences

 @example
 ```ts
 countOf('aa', 'a') === 2;
 ```
 */
function countOf(text: string, needle: string,): number {
  return text.split(needle,).length - 1;
}

/**
 The text of the `[vars]` table: from its header to the next top-level table.

 @param text - contents of the root task file

 @returns the vars section, or an empty text when the file has none
 */
function varsSection(text: string,): string {
  const start = text.indexOf(`${varsHeader}\n`,);
  if (start === (-1)) return '';
  const rest = text.slice(start + varsHeader.length,);
  const end = rest.indexOf('\n[',);
  return end === (-1) ? rest : rest.slice(0, end,);
}

/**
 The vars section of the root task file.
 */
const sharedSnippets = varsSection(taskFile,);

/**
 The backslash character, which a TOML basic string escapes by doubling it.
 */
const tomlBackslash = String.fromCodePoint(0x5C,);

/**
 The text of one triple-quoted `[vars]` declaration, as TOML hands it to mise.

 The escapes this file's bodies use are a newline and a doubled backslash; one pass of pair-to-single backslash turns
 the raw file's text into the text mise stores, which is what node evaluates.

 @param name - variable whose text is wanted

 @returns the declaration's body, or an empty text when it is absent

 @example
 ```ts
 variableBody('parse_usage_args').includes('parseUsageArgs');
 ```
 */
function variableBody(name: string,): string {
  const opening = `${name} = """\n`;
  const start = sharedSnippets.indexOf(opening,);
  if (start === (-1)) return '';
  const body = sharedSnippets.slice(start + opening.length,);
  const end = body.indexOf('\n"""',);
  return end === (-1) ? '' : body.slice(0, end,).replaceAll(`${tomlBackslash}${tomlBackslash}`, tomlBackslash,);
}

/**
 The value of the `forwarded_arguments_marker` declaration.

 @returns the marker comment, or an empty text when the declaration is absent
 */
function forwardedMarker(): string {
  const line = sharedSnippets.split('\n',).find((entry) => entry.startsWith(forwardedMarkerDeclaration,),);
  if (line === undefined) return '';
  const quoted = line.slice(forwardedMarkerDeclaration.length,);
  return quoted.endsWith('"',) ? quoted.slice(0, (-1),) : quoted;
}

/**
 The last line of every triple-quoted `run` block that interpolates the shared parser.

 A body that ended anywhere else would leave the words mise appends after it inside the TypeScript.

 @param text - contents of the root task file

 @returns the final body line of each interpolating `run` block, in file order

 @example
 ```ts
 parserBodyEndings(source).length === 22;
 ```
 */
function parserBodyEndings(text: string,): string[] {
  const endings: string[] = [];
  const delimiters = ['"""', "'''",];
  let delimiter = '';
  let insideParserBody = false;
  let lastLine = '';
  for (const line of text.split('\n',)) {
    const open = delimiters.find((candidate) => line === `run = ${candidate}`,);
    if (open !== undefined) {
      delimiter = open;
      insideParserBody = false;
      lastLine = '';
      continue;
    }
    if (delimiter === '') continue;
    if (line === delimiter) {
      if (insideParserBody) endings.push(lastLine,);
      delimiter = '';
      continue;
    }
    if (line === parserSnippet) insideParserBody = true;
    if (line.trim() !== '') lastLine = line;
  }
  return endings;
}

/**
 The lines that call the shared parser with an argument, so each can be checked for a recovery.

 @param text - contents of the root task file

 @returns calling lines, excluding the declaration and comments about it
 */
function parserCalls(text: string,): string[] {
  const calls: string[] = [];
  for (const line of text.split('\n',)) {
    if (!line.includes('parseUsageArgs(',)) continue;
    if (line.includes('parseUsageArgs = ',)) continue;
    const trimmed = line.trimStart();
    if (trimmed.startsWith('#',)) continue;
    if (trimmed.startsWith('//',)) continue;
    calls.push(line,);
  }
  return calls;
}

/**
 A synthetic task body: the shared parser, a read of its answer, then the marker and the words mise appends there.

 @param options - marker text, appended words, and the line a body carries after its marker

 @returns TypeScript for `node --input-type=module-typescript -e`

 @example
 ```ts
 parserRun({ marker, pasted: ' --help', afterMarker: '' });
 ```
 */
function parserRun(
  options: {
    readonly marker: string;
    readonly pasted: string;
    readonly afterMarker: string;
  },
): string {
  const read = 'process.stdout.write(JSON.stringify(parseUsageArgs(usageArgWords())))';
  const parser = variableBody('parse_usage_args',).replaceAll(forwardedMarkerSnippet, options.marker,);
  return `${parser}\n${read}\n${options.marker}${options.pasted}${options.afterMarker}`;
}

/**
 Arguments the shared parser recovers from a synthetic body.

 The child's environment never carries a `usage_args` of its own, because this test's own task was given one.

 @param options - marker text, appended words, the line after the marker, and any `usage_args` value

 @returns the parsed argument vector the body printed
 */
function recoveredArguments(
  options: {
    readonly marker: string;
    readonly pasted: string;
    readonly afterMarker: string;
    readonly usageArgs?: string;
  },
): string[] {
  const inherited = Object.entries(process.env,).filter(([name]) => name !== 'usage_args',);
  const environment = Object.fromEntries(inherited,);
  if (options.usageArgs !== undefined) environment.usage_args = options.usageArgs;
  const stdout = execFileSync(
    'node',
    ['--input-type=module-typescript', '-e', parserRun(options,),],
    { encoding: 'utf8', env: environment, },
  );
  return JSON.parse(stdout,) as string[];
}

//endregion

await describe({
  name: 'root mise.toml shared task-body shapes',
  children: [
    it({
      name: 'finds the sections it counts, so a renamed table cannot make the checks below vacuous',
      fn: async function findsItsSubjects() {
        expect(sharedSnippets,).not.toBe('',);
        expect(countOf(taskFile, testTreeWalk,),).toBeGreaterThan(0,);
        expect(countOf(taskFile, taskListRead,),).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'declares the deprecated-and-paused predicate once, in the vars section, and reads it from every test-tree walk',
      fn: async function onePredicate() {
        expect(countOf(sharedSnippets, inactivityDeclaration,),).toBe(1,);
        expect(countOf(taskFile, inactivityDeclaration,),).toBe(1,);
        expect(countOf(taskFile, inactivitySnippet,),).toBe(countOf(taskFile, testTreeWalk,),);
      },
    },),
    it({
      name: 'declares the raised task-list stdout cap once, in the vars section, and reads it at every task-list read',
      fn: async function oneBufferCap() {
        expect(countOf(sharedSnippets, bufferDeclaration,),).toBe(1,);
        expect(countOf(taskFile, bufferDeclaration,),).toBe(1,);
        expect(countOf(taskFile, bufferSnippet,),).toBe(countOf(taskFile, taskListRead,),);
      },
    },),
    it({
      name: 'ends every parser body with the marker comment mise appends forwarded arguments after',
      fn: async function everyParserBodyIsMarked() {
        const endings = parserBodyEndings(taskFile,);
        expect(endings.length,).toBe(countOf(taskFile, `\n${parserSnippet}\n`,),);
        expect(endings.filter((line) => line !== forwardedMarkerSnippet),).toEqual([],);
        expect(forwardedMarker().startsWith('//',),).toBe(true,);
      },
    },),
    it({
      name: 'declares the forwarded-arguments marker once and recovers the appended words at every parser call',
      fn: async function oneMarker() {
        expect(countOf(sharedSnippets, forwardedMarkerDeclaration,),).toBe(1,);
        const unrecovered = parserCalls(taskFile,).filter((line) => {
          if (line.includes(unnamedArgumentRecovery,)) return false;
          return !line.includes(namedArgumentRecovery,);
        },);
        expect(parserCalls(taskFile,).length,).toBeGreaterThan(0,);
        expect(unrecovered,).toEqual([],);
      },
    },),
    it({
      name: 'recovers the words mise appends after the marker as the parsed arguments',
      fn: async function recoversAppendedWords() {
        const marker = forwardedMarker();
        expect(
          recoveredArguments({ marker, pasted: " --help 'two words'", afterMarker: '', },),
        ).toEqual(['--help', 'two words',],);
      },
    },),
    it({
      name: 'parses no arguments when mise appended none',
      fn: async function parsesNothingWithoutPaste() {
        const marker = forwardedMarker();
        expect(recoveredArguments({ marker, pasted: '', afterMarker: '', },),).toEqual([],);
      },
    },),
    it({
      name: 'parses no arguments when the marker is not on the last line of the script',
      fn: async function markerNotLastMeansNoArguments() {
        const marker = forwardedMarker();
        expect(
          recoveredArguments({ marker, pasted: '', afterMarker: '\nconst after = 1;\n', },),
        ).toEqual([],);
      },
    },),
    it({
      name: 'prefers the usage_args mise parsed over the appended words',
      fn: async function theParserWins() {
        const marker = forwardedMarker();
        expect(
          recoveredArguments({ marker, pasted: ' --appended', afterMarker: '', usageArgs: '--parsed', },),
        ).toEqual(['--parsed',],);
      },
    },),
  ],
},);
