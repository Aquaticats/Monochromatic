/**
 Guards that the family's living Markdown passes the repository's Markdown
 linter (ledger D27 and D35). The repository's `lint:monochromatic-lint` task
 walks the whole tree and reports tens of thousands of findings outside this
 family, and no task of a family package ran it, so 90 `semantic-line-breaks`
 findings stood in nine docs of the package and the living repository-level docs
 on `main` with every gate green: the linter's finding count was read by no one.

 WHAT IS READ is what the other doc scans read: the package's and the family's
 docs and READMEs (`readPackageDocPaths`), the living repository-level docs
 (`readLivingRepositoryDocs`), the package split plan, which every package
 README links and no split audit pins, and the older repository-level docs
 whose names do not start with `translation-repair` (`OTHER_STEMS`, the list
 `doc-line-limit.unit.test.ts` reads too, so a new file under a listed stem is
 read the day it is added; ledger D35). The dated snapshots, history segments,
 run-continuity parts and interface-candidate files are the archives that
 ledger D27 keeps byte for byte, and they are not read.

 HOW IT RUNS: the `monochromatic-lint` binary, which the repository's `mise`
 puts on the path of its tasks, is started from the repository root through
 the keyless child fixture, so it finds the root `monochromatic-lint.config.jsonc`
 and judges the docs by the rules the root task applies. It prints one JSON
 finding per line on stdout and exits 1 when it found any and 2 when it could
 not finish; any other exit, or a line that is not a finding, fails the case
 rather than passing it as clean. The binary is a `cargo:monochromatic-lint`
 tool the root `mise.toml` does not pin, so a newer linter may find more.

 THE FIXTURE CASE COMES FIRST, so the docs case is read against a run shown
 able to find a finding and to pass a clean file (ledger M21). Fixtures are
 cat-themed.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  OTHER_STEMS,
  readLivingRepositoryDocs,
  readPackageDocPaths,
  readStemDocs,
  REPOSITORY_ROOT,
  staleStemFindings,
} from './living-docs.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 The linter's command name, as the repository's tasks run it.
 */
const LINTER = 'monochromatic-lint';

/**
 Exit code of a finished run that found something.
 */
const EXIT_FINDINGS = 1;

/**
 Repository-level docs the guard reads beyond the living set, each with why.
 The package split plan is linked from every package README and no split audit
 pins its bytes, so it is a living doc the living fixture does not list.
 */
const EXTRA_DOCS: readonly string[] = [
  join(
    'doc',
    'planning',
    'translation-repair-package-split.md',
  ),
];

/**
 One linter finding as a line, read off the JSON it printed.

 @param record - printed JSON, parsed

 @returns `path:line:column code`

 @throws Error when the record lacks the name, the code or the first label's
 position, since the run would then be read as clean

 @example
 ```ts
 const line = findingLine({ record: { filename: 'cat.md', code: 'markdown/semantic-line-breaks', labels: [], }, },);
 ```
 */
function findingLine({ record, }: { readonly record: Readonly<Record<string, unknown>>; },): string {
  /**
   The name, rule and labels the linter printed.
   */
  const { filename, code, labels, } = record;
  if (((typeof filename) !== 'string') || ((typeof code) !== 'string') || (!isJsonArray(labels,)))
    throw new Error(`the linter printed a finding without a file, a rule or labels: ${JSON.stringify(record,)}`,);
  /**
   First label, which holds the place.
   */
  const [first,] = labels;
  if (!isJsonRecord(first,))
    throw new Error(`the linter printed a finding with no label: ${JSON.stringify(record,)}`,);
  /**
   Where the finding stands.
   */
  const { span, } = first;
  if (!isJsonRecord(span,))
    throw new Error(`the linter printed a finding with no place: ${JSON.stringify(record,)}`,);
  /**
   Line and column of the place.
   */
  const { line, column, } = span;
  return `${filename}:${String(line,)}:${String(column,)} ${code}`;
}

/**
 Every finding the linter makes over some Markdown files.

 @param paths - files to lint, relative to `cwd`

 @param cwd - directory the linter runs in, which decides the configuration it finds

 @param configuration - configuration file to use instead of the one found from `cwd`

 @returns One `path:line:column code` line per finding, in the linter's order

 @throws Error when the linter could not start, exited other than 0 or 1, or
 printed a line that is not a JSON finding, since each would pass as clean

 @example
 ```ts
 const findings = await lintFindings({ paths: ['nap.md',], cwd: REPOSITORY_ROOT, },);
 ```
 */
async function lintFindings(
  {
    paths,
    cwd,
    configuration,
  }: {
    readonly paths: readonly string[];
    readonly cwd: string;
    readonly configuration?: string;
  },
): Promise<readonly string[]> {
  /**
   How the linter ended and what it printed.
   */
  const run = await runKeyless({
    file: LINTER,
    args: [
      ...((configuration === undefined) ? [] : ['--config', configuration,]),
      ...paths,
    ],
    cwd,
  },);
  if ((run.code !== 0) && (run.code !== EXIT_FINDINGS))
    throw new Error(`${LINTER} exited ${String(run.code,)}, so its run is not read as clean: ${run.stderr}${run.stdout}`,);
  return run.stdout
    .split('\n',)
    .filter(function isPrinted(line,): boolean {
      return line !== '';
    },)
    .map(function toLine(line,): string {
      /**
       The printed finding.
       */
      const record: unknown = JSON.parse(line,);
      if (!isJsonRecord(record,))
        throw new Error(`${LINTER} printed a line that is not a finding: ${line}`,);
      return findingLine({ record, },);
    },);
}

/**
 Every Markdown doc the guard reads, from the repository root.

 @returns Package docs, then the living repository-level docs, then the extras, then the older docs found by stem

 @throws {@link Error} when the living docs cannot be located

 @example
 ```ts
 const paths = await readLintedDocs();
 ```
 */
async function readLintedDocs(): Promise<readonly string[]> {
  /**
   Living repository-level docs.
   */
  const { decisionRecords, handover, currentPlanning, operations, } = await readLivingRepositoryDocs();
  return [
    ...await readPackageDocPaths(),
    ...decisionRecords,
    ...handover,
    ...currentPlanning,
    ...operations,
    ...EXTRA_DOCS,
    ...await readStemDocs({ stems: OTHER_STEMS, },),
  ];
}

await describe({
  name: 'the Markdown linter over the family\'s living docs',
  children: [
    it({
      name: 'FINDS a clause that runs on past a comma, and PASSES a file that breaks after it, under the root '
        + 'configuration',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cat-lint-', },);
        await mkdir(
          join(
            scratch.path,
            'doc',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            scratch.path,
            'doc',
            'runs-on.md',
          ),
          'The cat naps, and the kitten purrs.\n',
          'utf8',
        );
        await writeFile(
          join(
            scratch.path,
            'doc',
            'broken.md',
          ),
          'The cat naps,\nand the kitten purrs.\n',
          'utf8',
        );
        expectFindingsAsListed({
          findings: await lintFindings({
            paths: [
              join(
                'doc',
                'runs-on.md',
              ),
              join(
                'doc',
                'broken.md',
              ),
            ],
            cwd: scratch.path,
            configuration: join(
              REPOSITORY_ROOT,
              'monochromatic-lint.config.jsonc',
            ),
          },),
          listed: [`${join('doc', 'runs-on.md',)}:1:14 markdown/semantic-line-breaks`,],
        },);
      },
    },),
    it({
      name: 'REFUSES a run that cannot finish rather than reading it as clean',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cat-lint-', },);
        await expect(lintFindings({
          paths: ['no-such-nap.md',],
          cwd: scratch.path,
          configuration: join(
            REPOSITORY_ROOT,
            'monochromatic-lint.config.jsonc',
          ),
        },),).rejects.toThrow('exited 2',);
      },
    },),
    it({
      name: 'FINDS a stem no doc\'s name starts with, and passes a stem a doc and a split file of it both open',
      fn: async () => {
        expect(staleStemFindings({
          stems: ['cat-nap-', 'dog-walk', 'cat-bowl',],
          paths: [
            join('doc', 'planning', 'cat-nap-on-the-sofa.md',),
            join('doc', 'planning', 'cat-bowl.md',),
            join('doc', 'planning', 'cat-bowl-and-water.md',),
          ],
        },),).toEqual([
          'no doc\'s name starts with the stem dog-walk; drop it from OTHER_STEMS or rename the doc back',
        ],);
      },
    },),
    it({
      name: 'FINDS NO MARKDOWN LINT FINDING in the package docs, their READMEs, the living repository-level docs, '
        + 'the package split plan or the older docs OTHER_STEMS names',
      fn: async () => {
        /**
         Every doc the guard reads.
         */
        const paths = await readLintedDocs();
        expect(paths.some(function isOlderDoc(path,): boolean {
          return path.endsWith('wire-the-heading-aligner.md',);
        },),).toBe(true,);
        expect(paths.some(function isAuditDoc(path,): boolean {
          return path.endsWith(join('doc', 'audit', 'where-a-round-spends-its-wall-clock.md',),);
        },),).toBe(true,);
        expect(paths.some(function isTroubleshootingDoc(path,): boolean {
          return path.endsWith(join('doc', 'troubleshooting', 'synthetic-hf-prefix-misroute.md',),);
        },),).toBe(true,);
        expectNoFindings({ findings: staleStemFindings({ stems: OTHER_STEMS, paths, },), },);
        expect(paths.some(function isPassLog(path,): boolean {
          return path.includes('translation-repair-openrouter-2026-09-03-',);
        },),).toBe(true,);
        expect(paths.some(function isLedger(path,): boolean {
          return path.endsWith('audit-ledger.md',);
        },),).toBe(true,);
        expectNoFindings({
          findings: await lintFindings({
            paths,
            cwd: REPOSITORY_ROOT,
          },),
        },);
      },
    },),
  ],
},);
