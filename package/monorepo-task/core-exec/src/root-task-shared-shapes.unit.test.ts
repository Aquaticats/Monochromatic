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
 marker comment those words land in. One check holds each body to that ending, and the rest run the parser against a
 paste, so the words are shown to arrive as arguments rather than as source. The recovery returns the words exactly as
 mise wrote them, a trailing space, tab, U+00A0, U+FEFF, U+3000, U+2003, vertical tab or form feed included; it
 refuses a word holding a line feed, a carriage return, U+2028 or U+2029 and names that word, the last one included;
 and it refuses a body that continues past the marker with a message of its own. The catalog through real mise covers
 every one of those shapes, so a recovery that trims the tail or miscounts the broken word fails it. Package task files
 interpolate the same root variables, so the marker checks cover every package task file's
 parser bodies as well as the root file's, and the marker itself must stay declared in the root file alone.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { spawnSync, } from 'node:child_process';
import {
  glob,
  mkdtempDisposable,
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  fixtureEnvironment,
  resolveMise,
} from './build-and-test-fixture.ts';

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

/**
 The repository root, which holds the root task file and the package task files.
 */
const repositoryRoot = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 One package task file: its repository-relative path and its contents.
 */
type PackageTaskFile = {
  readonly path: string;
  readonly text: string;
};

/**
 Every package task file, in path order, so each is read once.
 */
const packageTaskFiles: PackageTaskFile[] = [];
for await (const path of glob('package/*/*/mise.toml', { cwd: repositoryRoot, },)) {
  packageTaskFiles.push({ path, text: await readFile(`${repositoryRoot}${path}`, 'utf8',), },);
}
packageTaskFiles.sort((left, right) => (left.path < right.path ? (-1) : 1),);

/**
 The package task files that interpolate the shared parser, which are the ones whose bodies receive forwarded words.
 */
const parserPackageFiles = packageTaskFiles.filter((file) => file.text.includes(parserSnippet,),);

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
 The raw TOML declaration of one `[vars]` entry, as a task file would carry it.

 A scratch configuration must interpolate exactly what mise renders, so it copies the declaration rather than the
 unescaped body `variableBody` returns.

 @param name - variable whose declaration is wanted

 @returns the declaration including any closing delimiter, or an empty text when it is absent

 @example
 ```ts
 variableDeclaration('parse_usage_args').startsWith('parse_usage_args = """');
 ```
 */
function variableDeclaration(name: string,): string {
  const lines = sharedSnippets.split('\n',);
  const start = lines.findIndex((line) => line.startsWith(`${name} = `,),);
  if (start === (-1)) return '';
  const opening = lines[start];
  if (opening === undefined) return '';
  if (!opening.endsWith('"""',)) return opening;
  const end = lines.findIndex((line, index) => (index > start) && (line === '"""'),);
  return end === (-1) ? '' : lines.slice(start, end + 1,).join('\n',);
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
 The line the shared parser's declaration opens on.
 */
const parserDeclarationStart = 'parse_usage_args = """';

/**
 The line range of the shared parser's own declaration.

 The parser parses text it has already recovered, so its own calls are not readers of mise's variables and the
 recovery check below must not count them. Package task files interpolate the declaration instead of carrying it, so
 they have no such range and every call in them is a reader.

 @param lines - the task file's lines

 @returns the first and last line index of the declaration, both -1 when the file has none

 @example
 ```ts
 parserDeclarationRange(text.split('\n')).first >= 0;
 ```
 */
function parserDeclarationRange(lines: readonly string[],): {
  readonly first: number;
  readonly last: number;
} {
  const first = lines.findIndex((line) => line.startsWith(parserDeclarationStart,),);
  if (first === (-1)) return { first: (-1), last: (-1), };
  const last = lines.findIndex((line, index) => (index > first) && (line === '"""'),);
  return { first, last, };
}

/**
 The lines that call the shared parser with an argument, so each can be checked for a recovery.

 @param text - contents of the root task file

 @returns calling lines outside the parser's own declaration, excluding comments about the declaration
 */
function parserCalls(text: string,): string[] {
  const lines = text.split('\n',);
  const { first, last, } = parserDeclarationRange(lines,);
  const calls: string[] = [];
  for (const [index, line,] of lines.entries()) {
    if ((first !== (-1)) && (index >= first) && (index <= last)) continue;
    if (!line.includes('parseUsageArgs(',)) continue;
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
 An exit status as a failure message reads it, naming a signal death instead of an absent status.

 @param status - the child's exit status, absent when a signal ended it

 @returns the status as digits, or the signal case in words

 @example
 ```ts
 exitText(1) === '1';
 ```
 */
function exitText(status?: number,): string {
  return status === undefined ? 'a signal rather than an exit status' : String(status,);
}

/**
 One parser run: its exit status and both streams.
 */
type ParserOutcome = {
  /**
   Whether node ended on its own with status 0.
   */
  readonly succeeded: boolean;
  /**
   The exit status as a message reads it, naming a signal death when there is no status.
   */
  readonly exit: string;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run a synthetic body through node, with no `usage_args` inherited from this test's own task.

 @param options - marker text, appended words, the line after the marker, and any `usage_args` value

 @returns the child's exit status and both streams

 @example
 ```ts
 runParserBody({ marker, pasted: ' --help', afterMarker: '' });
 ```
 */
function runParserBody(
  options: {
    readonly marker: string;
    readonly pasted: string;
    readonly afterMarker: string;
    readonly usageArgs?: string;
  },
): ParserOutcome {
  const inherited = Object.entries(process.env,).filter(([name]) => name !== 'usage_args',);
  const environment = Object.fromEntries(inherited,);
  if (options.usageArgs !== undefined) environment.usage_args = options.usageArgs;
  const result = spawnSync(
    'node',
    ['--input-type=module-typescript', '-e', parserRun(options,),],
    { encoding: 'utf8', env: environment, },
  );
  return {
    succeeded: result.status === 0,
    exit: exitText(result.status ?? undefined,),
    stdout: (result.stdout ?? '').trim(),
    stderr: result.stderr ?? '',
  };
}

/**
 Arguments the shared parser recovers from a synthetic body.

 @param options - marker text, appended words, the line after the marker, and any `usage_args` value

 @returns the parsed argument vector the body printed

 @throws Error when the body exits non-zero, carrying the child's diagnostics
 */
function recoveredArguments(
  options: {
    readonly marker: string;
    readonly pasted: string;
    readonly afterMarker: string;
    readonly usageArgs?: string;
  },
): string[] {
  const outcome = runParserBody(options,);
  if (!outcome.succeeded)
    throw new Error(`the parser body exited ${outcome.exit}: ${outcome.stderr}`,);
  return JSON.parse(outcome.stdout,) as string[];
}

//endregion

//region The paste boundary mise actually writes, through real mise

/**
 One paste case: the words a caller puts after `--`, and a name a failure message can use.
 */
type PasteCase = {
  readonly name: string;
  readonly words: readonly string[];
  /**
   Whether the words go before `--`, where mise's own usage parser reads them instead of pasting them.
   */
  readonly bare?: boolean;
};

/**
 Every shell-word shape mise's paste can write into the script without a line terminator.

 Each was taken from a real mise run, not hand-written, so the quotes, backslashes, empty words, marker text,
 delimiters and traversal tokens are the ones mise produces. The recovery must return each list unchanged.
 */
const exactPasteCases: readonly PasteCase[] = [
  { name: 'no arguments', words: [], },
  { name: 'no arguments and no separator', bare: true, words: [], },
  { name: 'ordinary arguments, one with two words', words: ['alpha', 'two words',], },
  { name: 'ordinary arguments without a separator', bare: true, words: ['alpha', 'two words',], },
  { name: 'a flag that is not help', words: ['--foo',], },
  { name: 'a short flag', words: ['-x',], },
  { name: 'help alone', words: ['--help',], },
  { name: 'short help alone', words: ['-h',], },
  { name: 'help not first', words: ['a', '--help', 'b',], },
  { name: 'help then a two-word argument', words: ['--help', 'two words',], },
  { name: 'help then an empty argument', words: ['--help', '',], },
  { name: 'help then only empty arguments', words: ['--help', '', '',], },
  { name: 'a single quote inside a word', words: ['--help', "it's",], },
  { name: 'a double quote inside a word', words: ['--help', 'say "hi"',], },
  { name: 'a backslash inside a word', words: ['--help', String.raw`back\slash`,], },
  { name: 'a trailing backslash', words: ['--help', `trail${tomlBackslash}`,], },
  { name: 'a lone quote word', words: ['--help', "'",], },
  { name: 'two quote characters only', words: ['--help', '"\'"',], },
  { name: 'shell metacharacters', words: ['--help', ';', '$(echo hi)', '`x`', '*', '&&', '|', '>',], },
  { name: 'comment openers and closers', words: ['--help', '*/', '/*', '//', '#',], },
  { name: 'a tab and unicode words', words: ['--help', 'tab\there', 'caf\u00E9 \u{1F600}',], },
  { name: 'a last word ending with U+00A0', words: ['--help', 'a\u00A0',], },
  { name: 'a last word that is only U+00A0', words: ['--help', '\u00A0',], },
  { name: 'a last word ending with U+FEFF', words: ['--help', 'a\uFEFF',], },
  { name: 'a last word ending with U+3000', words: ['--help', 'a\u3000',], },
  { name: 'a last word ending with U+2003', words: ['--help', 'a\u2003',], },
  { name: 'a last word ending with a vertical tab', words: ['--help', 'a\u000B',], },
  { name: 'a last word ending with a form feed', words: ['--help', 'a\u000C',], },
  { name: 'the marker text itself', words: ['--help', '// mise appends forwarded arguments after this comment',], },
  { name: 'a word that begins with the marker', words: ['--help', '// mise appends forwarded arguments after this comment tail',], },
  { name: 'traversal tokens', words: ['--help', '../../etc/passwd', String.raw`..\..\x`,], },
  { name: 'a very long word', words: ['--help', 'x'.repeat(100_000,),], },
  { name: 'a second separator', words: ['--help', '--', 'x',], },
];

/**
 Every paste case whose word carries a line terminator and whose pasted lines still parse as source.

 The break ends the marker comment, so the words after it are source lines; the recovery must refuse and name the word
 that carries the break rather than return a shorter or a different list.
 */
const refusedPasteCases: readonly PasteCase[] = [
  { name: 'a line feed then a line comment', words: ['--help', 'a\n// b',], },
  { name: 'a line feed then the marker text', words: ['--help', 'a\n// mise appends forwarded arguments after this comment',], },
  { name: 'a carriage return inside a word', words: ['--help', 'a\rb',], },
  { name: 'a carriage return then a line comment', words: ['--help', 'a\r// b',], },
  { name: 'a U+2028 line separator inside a word', words: ['--help', 'a\u2028b',], },
  { name: 'a U+2028 line separator then a line comment', words: ['--help', 'a\u2028// b',], },
  { name: 'a U+2029 paragraph separator then a line comment', words: ['--help', 'a\u2029// b',], },
  { name: 'short help then a line feed and a line comment', words: ['-h', 'x\n// y',], },
  { name: 'short help then a line feed, the marker and more', words: ['--help', 'x\n// mise appends forwarded arguments after this comment y',], },
  { name: 'a break in the fourth word only', words: ['--help', 'one', 'two words', 'x\r//y',], },
  { name: 'a last word ending with a carriage return', words: ['--help', 'a\r',], },
  { name: 'a last word that is only a carriage return', words: ['--help', '\r',], },
  // mise quotes this one for its space, so the break lands inside a quote and the count must still name it.
  { name: 'a last word ending with a space and a carriage return then a comment', words: ['--help', 'a \r// b',], },
  { name: 'a last word that is only a U+2028 separator', words: ['--help', '\u2028',], },
  { name: 'a last word ending with U+2028', words: ['--help', 'a\u2028',], },
  { name: 'a last word ending with U+2029', words: ['--help', 'a\u2029',], },
];

/**
 Paste cases whose pasted lines do not parse at all, so node refuses the whole script before the body runs.

 These are the shapes no recovery code can see: the syntax error is the failure, and it is the one `main` gives.
 */
const unparsablePasteCases: readonly PasteCase[] = [
  { name: 'a line feed then ordinary text', words: ['--help', 'a\nb',], },
  { name: 'a line feed then a block comment opener', words: ['--help', 'a\n/*',], },
  // mise quotes this word for its space, and the quote it opens is still open when the carriage return ends the
  // marker comment, so node reads a raw carriage return inside a string literal.
  { name: 'a quoted word ending with a carriage return', words: ['--help', 'a \r',], },
];

/**
 The start of the message the recovery gives when a line terminator breaks the pasted words.
 */
const refusedPastePrefix = 'Error: cannot read the arguments mise pasted into this script:';

/**
 The text that precedes the word number in the refusal message.
 */
const refusedPositionPrefix = 'argument ';

/**
 Every JavaScript line terminator, each of which ends the marker comment on node's side of the paste.
 */
const lineTerminators: readonly string[] = ['\n', '\r', '\u2028', '\u2029',];

/**
 The word number a refusal message names.

 @param message - the refusal line

 @returns the digits after `argument `, or nothing when the message carries none

 @example
 ```ts
 namedWord('argument 3 contains a line break') === '3';
 ```
 */
function namedWord(message: string,): string {
  const from = message.indexOf(refusedPositionPrefix,);
  if (from === (-1)) return '';
  const rest = message.slice(from + refusedPositionPrefix.length,);
  return rest.slice(0, rest.search(' ',),);
}

/**
 Whether one word carries a line terminator.

 @param word - one forwarded word as the caller typed it

 @returns true when the recovery must refuse it

 @example
 ```ts
 carriesLineTerminator('a\nb') === true;
 ```
 */
function carriesLineTerminator(word: string,): boolean {
  return lineTerminators.some(function appears(terminator,): boolean {
    return word.includes(terminator,);
  },);
}

/**
 A scratch configuration whose probe task prints what the root's own parser recovers.

 @returns TOML for a disposable repository root
 */
function probeConfiguration(): string {
  return `[vars]
${variableDeclaration('forwarded_arguments_marker',)}
${variableDeclaration('parse_usage_args',)}

[tasks.probe]
usage = 'arg "[args]" var=#true help="probe the forwarded-argument recovery"'
shell = "node --input-type=module-typescript -e"
run = """
{{vars.parse_usage_args}}
process.stdout.write(JSON.stringify(parseUsageArgs(usageArgWords())))
{{vars.forwarded_arguments_marker}}
"""
`;
}

/**
 What one nested mise run did with the words it was given.
 */
type ProbeOutcome = {
  /**
   Whether mise ended on its own with status 0.
   */
  readonly succeeded: boolean;
  /**
   The exit status as a message reads it, naming a signal death when there is no status.
   */
  readonly exit: string;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run each case's words through real mise in one disposable configuration.

 The configuration interpolates the root file's own marker and parser declarations into a task body that ends with
 the marker, so each case exercises mise's paste and this file's recovery together rather than a hand-written copy
 of the paste. mise's state, home and trusted configuration path are the fixture root, so no outer configuration or
 credential is read.

 @param cases - words to put after `--`, one run each

 @returns each case with the exit status and streams of its run
 */
async function probeThroughMise(
  cases: readonly PasteCase[],
): Promise<readonly { readonly pasteCase: PasteCase; readonly outcome: ProbeOutcome }[]> {
  await using directory = await mkdtempDisposable(join(scratchRoot(), 'mise-forwarded-arguments-',),);
  const root = await realpath(directory.path,);
  const mise = await resolveMise();
  await writeFile(join(root, 'mise.toml',), probeConfiguration(),);
  const environment = fixtureEnvironment({ root, mise, },);
  return cases.map((pasteCase) => {
    const separators = pasteCase.bare === true ? [] : ['--',];
    const result = spawnSync(mise, ['run', 'probe', ...separators, ...pasteCase.words,], {
      cwd: root,
      env: environment,
      encoding: 'utf8',
      timeout: 60_000,
      maxBuffer: 8_000_000,
    },);
    return {
      pasteCase,
      outcome: {
        succeeded: result.status === 0,
        exit: exitText(result.status ?? undefined,),
        stdout: (result.stdout ?? '').trimEnd(),
        stderr: result.stderr ?? '',
      },
    };
  },);
}

/**
 Why a run of the exact cases is not what its cases expect, or nothing when it is.

 @param options - the case and its outcome

 @returns one diagnostic per failed expectation
 */
function exactProblems({
  pasteCase,
  outcome,
}: {
  readonly pasteCase: PasteCase;
  readonly outcome: ProbeOutcome;
}): readonly string[] {
  const problems: string[] = [];
  if (!outcome.succeeded)
    problems.push(`${pasteCase.name}: exited ${outcome.exit} and said ${outcome.stderr}`,);
  else if (outcome.stdout !== JSON.stringify(pasteCase.words,))
    problems.push(`${pasteCase.name}: printed ${outcome.stdout === '' ? 'nothing' : outcome.stdout}`,);
  return problems;
}

/**
 Why a refused run is not what its case expects, or nothing when it is.

 The message must name the word that carries the line terminator, counted from the case's own words so the expectation
 does not restate the implementation, including when the break starts a word or lands inside a word mise quotes.

 @param options - the case and its outcome

 @returns one diagnostic per failed expectation
 */
function refusalProblems({
  pasteCase,
  outcome,
}: {
  readonly pasteCase: PasteCase;
  readonly outcome: ProbeOutcome;
}): readonly string[] {
  const problems: string[] = [];
  if (outcome.succeeded) problems.push(`${pasteCase.name}: exited 0 and printed ${outcome.stdout}`,);
  const carrying = pasteCase.words.findIndex(carriesLineTerminator,) + 1;
  const message = outcome.stderr.split('\n',).find((line) => line.startsWith(refusedPastePrefix,),);
  if (message === undefined) problems.push(`${pasteCase.name}: no refusal naming the broken word in ${outcome.stderr}`,);
  else if (namedWord(message,) !== String(carrying,))
    problems.push(`${pasteCase.name}: named a different word than the ${String(carrying,)}th, which carries the break`,);
  return problems;
}

/**
 Why an unparsable run is not what its case expects, or nothing when it is.

 No body code may run, so nothing may be printed, and the failure must be node's own TypeScript syntax error.

 @param options - the case and its outcome

 @returns one diagnostic per failed expectation
 */
function unparsableProblems({
  pasteCase,
  outcome,
}: {
  readonly pasteCase: PasteCase;
  readonly outcome: ProbeOutcome;
}): readonly string[] {
  const problems: string[] = [];
  if (outcome.succeeded) problems.push(`${pasteCase.name}: exited 0 and printed ${outcome.stdout}`,);
  if (outcome.stdout !== '') problems.push(`${pasteCase.name}: printed ${outcome.stdout} from a body that cannot parse`,);
  if (!outcome.stderr.includes('ERR_INVALID_TYPESCRIPT_SYNTAX',))
    problems.push(`${pasteCase.name}: not node's TypeScript syntax failure: ${outcome.stderr}`,);
  return problems;
}

/**
 Windows passes an inline task's forwarded words as separate argv, so no paste exists to check there.
 */
const notWindows = process.platform === 'win32'
  ? 'mise on Windows passes a non-POSIX shell its words as a separate argv, so it writes no paste into the script'
  : false;

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
      name: 'ends every package parser body with the marker comment too',
      fn: async function everyPackageParserBodyIsMarked() {
        expect(parserPackageFiles.length,).toBeGreaterThan(0,);
        const endingsByFile = parserPackageFiles.map((file) => ({
          file,
          endings: parserBodyEndings(file.text,),
        }),);
        for (const { file, endings } of endingsByFile) {
          expect(endings.length,).toBe(countOf(file.text, `\n${parserSnippet}\n`,),);
        }
        const bodies = endingsByFile.flatMap(({ endings }) => endings);
        expect(bodies.length,).toBeGreaterThan(0,);
        const unmarked = endingsByFile.flatMap(({ file, endings }) => endings.filter((ending) => ending !== forwardedMarkerSnippet).map((ending) => `${file.path}: ${ending}`,),);
        expect(unmarked,).toEqual([],);
      },
    },),
    it({
      name: 'recovers the appended words at every package parser call',
      fn: async function everyPackageParserCallRecovers() {
        const calls = parserPackageFiles.flatMap((file) => parserCalls(file.text,),);
        expect(calls.length,).toBeGreaterThan(0,);
        const unrecovered = calls.filter((line) => {
          if (line.includes(unnamedArgumentRecovery,)) return false;
          return !line.includes(namedArgumentRecovery,);
        },);
        expect(unrecovered,).toEqual([],);
      },
    },),
    it({
      name: 'declares the forwarded-arguments marker in the root file only',
      fn: async function markerHasOneHome() {
        const declarations = packageTaskFiles.filter((file) => file.text.includes(forwardedMarkerDeclaration,),).map((file) => file.path);
        expect(declarations,).toEqual([],);
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
      name: 'refuses when the script continues past the marker line, where mise appends nothing',
      fn: async function refusesTextAfterTheMarker() {
        const marker = forwardedMarker();
        const outcome = runParserBody({ marker, pasted: '', afterMarker: '\nconst after = 1;\n', },);
        expect(outcome.succeeded,).toBe(false,);
        expect(outcome.stderr,).toContain(`${refusedPastePrefix} the body continues past the forwarded-arguments marker comment`,);
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
    it({
      skip: notWindows,
      name: 'recovers every shell-word shape mise pastes or parses, through real mise',
      fn: async function recoversEveryShellWordShape() {
        const results = await probeThroughMise(exactPasteCases,);
        const failures = results.flatMap(exactProblems,);
        expect(results.length,).toBe(exactPasteCases.length,);
        expect(failures,).toEqual([],);
      },
    },),
    it({
      skip: notWindows,
      name: 'refuses a word carrying a line terminator, naming that word, through real mise',
      fn: async function refusesLineTerminators() {
        const results = await probeThroughMise(refusedPasteCases,);
        const failures = results.flatMap(refusalProblems,);
        expect(results.length,).toBe(refusedPasteCases.length,);
        expect(failures,).toEqual([],);
      },
    },),
    it({
      skip: notWindows,
      name: 'fails before any body code runs when the pasted lines cannot parse, through real mise',
      fn: async function failsBeforeTheBodyRuns() {
        const results = await probeThroughMise(unparsablePasteCases,);
        const failures = results.flatMap(unparsableProblems,);
        expect(results.length,).toBe(unparsablePasteCases.length,);
        expect(failures,).toEqual([],);
      },
    },),
  ],
},);
