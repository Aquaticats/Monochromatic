/**
 The material of a review and the brief it writes, against real Git on a disposable repository: every byte a path can carry
 survives, and two changes whose status and names joined by spaces are equal are still told apart.

 A stand-in Git cannot prove either, because one string answers both the question about a range's paths and the question about the
 paths a commit holds: the two answers only disagree when Git quotes a name, which a stand-in building both from the same strings
 never does. These tests therefore build the repository with real Git, and read it with the writer's own questions.

 The fixture's commits are made with `git fast-import`, which is plumbing and needs no commit service, so this holds on a machine
 whose Git wrapper refuses to commit without a user service manager. No file of the checkout is touched: every repository here is
 one the test made under the harness's scratch root and throws away.

 @module
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  FOCUS_OF_A_CHANGE_REVIEW,
  type ReviewMaterial,
  briefFileOfAReview,
  gitRunnerOf,
  materialOfAReview,
  runCommandWithChildProcess,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 A backslash, which a lone one cannot stand in a raw template, so one is taken from two.
 */
const BACKSLASH = String.raw`\\`.slice(
  0,
  1,
);

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 How the writer itself asks Git a question, which these tests use to read the fixtures.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 The change a review reads.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-name-real-git-quotes';
}

/**
 The folder whose changes are reviewed: the whole checkout, so a path that begins with a dash is one of them.

 @returns the folder's path, from the top of the checkout

 @example
 ```ts
 const source = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return '.';
}

/**
 The rules file the brief names first.

 @returns the file's path, inside this package

 @example
 ```ts
 const rules = aRulesFile();
 ```
 */
function aRulesFile(): string {
  return new URL('../doc/rules-every-pi-brief-carries.md', import.meta.url,).pathname;
}

/**
 The names these tests change, each of which Git quotes in a line or a shell would misread: a space, a double quote, a backslash,
 a line break, a tab, a byte outside ASCII, and a name that begins with a dash.

 @returns the paths, from the top of the checkout

 @example
 ```ts
 const names = awkwardPaths();
 ```
 */
function awkwardPaths(): readonly string[] {
  return [
    'plain.ts',
    'a b.ts',
    'a"b.ts',
    String.raw`a\b.ts`,
    'a\nb.ts',
    'a\tb.ts',
    'naïve.ts',
    '-dash.ts',
  ];
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'review-material-real-git-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Reads the environment every Git of one fixture runs in: its own home beside the repository, no session bus, no configuration of
 this machine, and no prompt.

 @param repo - the fixture's repository, whose folder is beside the home

 @returns the environment

 @example
 ```ts
 const environment = fixtureEnvironmentOf(repo);
 ```
 */
function fixtureEnvironmentOf(repo: string,): Record<string, string> {
  /**
   The test's folder, which holds the repository and the home.
   */
  const fixture = dirname(repo,);
  /**
   The environment this test's Git runs in, which begins as this process's own without its absent variables.
   */
  const environment: Record<string, string> = {};
  for (const [name, value,] of Object.entries(process.env,)) {
    if (value !== undefined) {
      environment[name] = value;
    }
  }
  environment.HOME = join(fixture, 'home',);
  environment.XDG_RUNTIME_DIR = join(fixture, 'runtime',);
  environment.XDG_CACHE_HOME = join(fixture, 'home', '.cache',);
  environment.GIT_CONFIG_GLOBAL = '/dev/null';
  environment.GIT_CONFIG_SYSTEM = '/dev/null';
  environment.GIT_TERMINAL_PROMPT = '0';
  environment.GIT_ASKPASS = '';
  // A fixture's own home does not keep a wrapper off this machine's bus, and no fixture here needs a service manager.
  delete environment.DBUS_SESSION_BUS_ADDRESS;
  return environment;
}

/**
 Runs one Git command in a fixture.

 @param folder - the fixture's folder

 @param words - the command's words

 @param input - what to write to its standard input, when it reads one

 @returns what it printed

 @example
 ```ts
 const answer = await gitIn({ folder: repo, words: ['rev-parse', 'HEAD'] },);
 ```
 */
async function gitIn({
  folder,
  words,
  input,
}: {
  readonly folder: string;
  readonly words: readonly string[];
  readonly input?: string;
}): Promise<string> {
  /**
   The environment every Git of this fixture runs in.
   */
  const environment = fixtureEnvironmentOf(folder,);
  if (input !== undefined) {
    // `git fast-import` reads its stream from standard input, which `execFile` cannot feed, so this one command is spawned and its
    // input written whole and closed.
    /**
     The command's process.
     */
    const child = spawn(
      'git',
      [...words,],
      {
        cwd: folder,
        env: environment,
      },
    );
    /**
     What it said on standard error, for a refusal.
     */
    let said = '';
    child.stderr.setEncoding('utf8',);
    child.stderr.on('data', function remember(chunk: string,) {
      said += chunk;
    },);
    child.stdin.end(input,);
    /**
     What the process reported when it closed.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    if (closed[0] !== 0) {
      throw new Error(`git ${words.join(' ',)} answered ${String(closed[0],)}: ${said}`,);
    }
    return '';
  }
  /**
   What the command answered.
   */
  const answered = await execFileAsync(
    'git',
    [...words,],
    {
      cwd: folder,
      env: environment,
      encoding: 'utf8',
    },
  );
  return answered.stdout;
}

/**
 Reads a path the way `git fast-import` takes one on a command line: as itself, or, when it holds a line break or a carriage
 return or begins with a double quote, inside double quotes with C-style escapes.

 @param path - a path, from the top of the checkout

 @returns the path as the stream line carries it

 @example
 ```ts
 const field = fastImportPath('src/a\nb.ts');
 ```
 */
function fastImportPath(path: string,): string {
  /**
   Whether the path holds a line break or a carriage return, which a stream line cannot carry as itself.
   */
  const holdsALineBreak = path.includes('\n',) || path.includes('\r',);
  if ((!holdsALineBreak) && (!path.startsWith('"',))) {
    return path;
  }
  /**
   The path with the characters a quoted field escapes.
   */
  const escaped = path
    .replaceAll(BACKSLASH, String.raw`\\`,)
    .replaceAll('"', String.raw`\"`,)
    .replaceAll('\n', String.raw`\n`,)
    .replaceAll('\r', String.raw`\r`,)
    .replaceAll('\t', String.raw`\t`,);
  return `"${escaped}"`;
}

/**
 Adds one commit to a fixture through `git fast-import`, which needs no commit service.

 @param folder - the fixture's folder

 @param message - the commit's message

 @param files - the paths to write, each with its content

 @param remove - the paths to delete

 @param from - the commit this one follows, or nothing for the first commit of a branch

 @returns the commit's id

 @example
 ```ts
 const commit = await fastImportCommit({ folder: repo, message: 'base', files: new Map([['src/gate.ts', 'a']]), remove: [], },);
 ```
 */
async function fastImportCommit({
  folder,
  message,
  files,
  remove,
  from,
}: {
  readonly folder: string;
  readonly message: string;
  readonly files: ReadonlyMap<string, string>;
  readonly remove: readonly string[];
  readonly from?: string;
}): Promise<string> {
  /**
   The lines of the stream fast-import reads.
   */
  const lines: string[] = [
    'commit refs/heads/main',
    'committer probe <probe@example.invalid> 1700000000 +0000',
    `data ${String(Buffer.byteLength(message, 'utf8',),)}`,
    message,
  ];
  if (from !== undefined) {
    lines.push(`from ${from}`,);
  }
  for (const [path, content,] of files) {
    lines.push(
      `M 100644 inline ${fastImportPath(path,)}`,
      `data ${String(Buffer.byteLength(content, 'utf8',),)}`,
      content,
    );
  }
  for (const path of remove) {
    lines.push(`D ${fastImportPath(path,)}`,);
  }
  lines.push('',);
  await gitIn({
    folder,
    words: ['fast-import', '--quiet',],
    input: `${lines.join('\n',)}\n`,
  },);
  return (await gitIn({
    folder,
    words: ['rev-parse', 'refs/heads/main',],
  },)).trim();
}

/**
 Makes a disposable repository, and the home its Git runs in.

 @param folder - the test's folder, which holds every part of the fixture

 @returns the repository's folder

 @example
 ```ts
 const repo = await makeRepository(folder.path);
 ```
 */
async function makeRepository(folder: string,): Promise<string> {
  /**
   The repository's folder.
   */
  const repo = join(folder, 'repo',);
  await mkdir(repo, { recursive: true, },);
  await mkdir(join(folder, 'home',), { recursive: true, },);
  await mkdir(join(folder, 'runtime',), { recursive: true, },);
  await gitIn({
    folder: repo,
    words: ['init', '--quiet', '--initial-branch=main',],
  },);
  return repo;
}

/**
 Reads what a range of a fixture reads.

 @param repo - the fixture's folder

 @param range - the range, as `git diff` takes one

 @param source - the folder whose changes are reviewed

 @param commit - the commit under review

 @returns what the writer read

 @example
 ```ts
 const material = await materialOf({ repo, range: `${base}...${tip}`, source: '.', commit: tip },);
 ```
 */
async function materialOf({
  repo,
  range,
  source,
  commit,
}: {
  readonly repo: string;
  readonly range: string;
  readonly source: string;
  readonly commit: string;
}): Promise<ReviewMaterial> {
  return await materialOfAReview({
    git,
    checkoutFolder: repo,
    ranges: [range,],
    source,
    commit,
  },);
}

/**
 Reads the files a material gives its reviewer, refusing a material that is not a change of code.

 @param material - what the writer read

 @returns the files to read

 @throws Error when the change answered as something other than code

 @example
 ```ts
 const files = filesOf(material);
 ```
 */
function filesOf(material: ReviewMaterial,): readonly string[] {
  if (material.kind !== 'code-and-build-decisions') {
    throw new Error(`a change of code answered as ${material.kind}`,);
  }
  return material.files;
}

/**
 Reads the paths a material says the change removed or renamed, refusing a material that is not a change of code.

 @param material - what the writer read

 @returns the removals and renames

 @throws Error when the change answered as something other than code

 @example
 ```ts
 const removed = removalsOf(material);
 ```
 */
function removalsOf(material: ReviewMaterial,) {
  if (material.kind !== 'code-and-build-decisions') {
    throw new Error(`a change of code answered as ${material.kind}`,);
  }
  return material.removedOrRenamed;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a review read from real Git',
      children: [
        it({
          name: 'names every changed file the commit holds, whatever byte its name carries, because both questions about paths are asked for raw names',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The fixture's repository.
             */
            const repo = await makeRepository(folder.path,);
            /**
             The names every commit of this test holds.
             */
            const names = awkwardPaths();
            /**
             The first commit, which writes every name.
             */
            const base = await fastImportCommit({
              folder: repo,
              message: 'base',
              files: new Map(names.map(function withContent(path,) {
                return [path, 'a',];
              },),),
              remove: [],
            },);
            /**
             The commit under review, which changes every one of them.
             */
            const tip = await fastImportCommit({
              folder: repo,
              message: 'change every name',
              files: new Map(names.map(function withContent(path,) {
                return [path, 'b',];
              },),),
              remove: [],
              from: base,
            },);
            /**
             What the writer read.
             */
            const material = await materialOf({
              repo,
              range: `${base}...${tip}`,
              source: aSourceFolder(),
              commit: tip,
            },);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(filesOf(material,).toSorted(),).toEqual(names.toSorted(),);
          },
          timeout: 60_000,
        },),
        it({
          name: 'writes a brief that names a file whose name holds a line break as a JSON string, says what that form is, and never claims the commit holds no file',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The fixture's repository.
             */
            const repo = await makeRepository(folder.path,);
            /**
             The range's first commit.
             */
            const base = await fastImportCommit({
              folder: repo,
              message: 'base',
              files: new Map([['plain.ts', 'a',],],),
              remove: [],
            },);
            /**
             The commit under review, whose only changed file holds a line break in its name.
             */
            const tip = await fastImportCommit({
              folder: repo,
              message: 'a name with a line break',
              files: new Map([['a\nb.ts', 'b',],],),
              remove: [],
              from: base,
            },);
            /**
             The folder the review keeps its files in.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            await writeReviewBrief({
              git,
              checkoutFolder: repo,
              review: aReview(),
              commit: tip,
              ranges: [`${base}...${tip}`,],
              source: aSourceFolder(),
              focus: FOCUS_OF_A_CHANGE_REVIEW,
              reviewsFolder,
              runsFolder: join(folder.path, 'runs',),
              rulesFile: aRulesFile(),
            },);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(briefFileOfAReview({
              reviewsFolder,
              review: aReview(),
            },), 'utf8',);
            expect(brief,).toContain('- `"a\\nb.ts"`',);
            expect(brief,).toContain('A path written in double quotes is shown as a JSON string',);
            expect(brief,).not.toContain('The reviewed commit holds no file at a path this change touches',);
          },
          timeout: 60_000,
        },),
        it({
          name: 'keeps both of two renames whose status and names joined by spaces are equal, because one change is told from another by its parts',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The fixture's repository.
             */
            const repo = await makeRepository(folder.path,);
            /**
             The first commit, which holds both paths before they are renamed.
             */
            const base = await fastImportCommit({
              folder: repo,
              message: 'base',
              files: new Map([
                ['x', 'P',],
                ['x y.ts', 'Q',],
              ],),
              remove: [],
            },);
            /**
             The commit under review, which renames both: their statuses and names joined by spaces are the same text.
             */
            const tip = await fastImportCommit({
              folder: repo,
              message: 'rename both',
              files: new Map([
                ['y.ts z.ts', 'P',],
                ['z.ts', 'Q',],
              ],),
              remove: ['x', 'x y.ts',],
              from: base,
            },);
            /**
             What the writer read.
             */
            const material = await materialOf({
              repo,
              range: `${base}...${tip}`,
              source: aSourceFolder(),
              commit: tip,
            },);
            expect(material.kind,).toBe('code-and-build-decisions',);
            expect(filesOf(material,).toSorted(),).toEqual(['y.ts z.ts', 'z.ts',],);
            expect(removalsOf(material,),).toEqual([
              {
                status: 'renamed',
                was: 'x',
                now: 'y.ts z.ts',
              },
              {
                status: 'renamed',
                was: 'x y.ts',
                now: 'z.ts',
              },
            ],);
          },
          timeout: 60_000,
        },),
      ],
    },),
  ],
},);
