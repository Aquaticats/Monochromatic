/**
 Tests for what the daily update reports about the packages it breaks: which checks exist, which failures are new,
 what the issue says, how it is deduplicated, and that none of it can fail or undo the update.

 Every effect is a scripted stand-in, so no test touches the network, git, the desktop or the real `gh`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cargoLockMoves,
  causeOf,
  checksFromTaskNames,
  compareChecks,
  dailyUpdateGroups,
  firstErrorOf,
  type CheckResult,
  issueMarker,
  MAXIMUM_MOVE_LINES,
  type PackageCheck,
  pnpmImporterMoves,
  publishIssues,
  renderIssue,
  runChecks,
  runUpdateWithChecks,
  toolchainLockMoves,
  CHECK_PASS_DEADLINE_MS,
  RUST_CHECK_PARALLEL,
  TYPESCRIPT_CHECK_PARALLEL,
} from '../dist/final/node/testing.mjs';
import {
  answerLikeAnEmptyTracker,
  fixture,
  FIXTURE_GLOBAL_LOCK,
  PASSES,
} from './daily-update-checks-fixture.ts';

//region Fixtures: texts the tests read

/**
 A Rust output whose first error has a code and a location, followed by the usual closing error.
 */
const ZBUS_BREAK_OUTPUT = [
  '[//package/a/b:lint:clippy] $ cargo clippy --locked',
  'warning: unused import: `x`',
  ' --> src/lib.rs:1:5',
  '',
  'error[E0433]: failed to resolve: could not find `blocking` in `zbus`',
  ' --> src/session.rs:12:11',
  '  |',
  '12 | use zbus::blocking::Connection;',
  '  |           ^^^^^^^^ could not find `blocking` in `zbus`',
  '',
  'error: could not compile `a` (lib) due to 1 previous error',
].join('\n',);

/**
 A Cargo lock text with a given zbus version and one unrelated crate.

 @param zbusVersion - version recorded for zbus

 @returns lock text
 */
function cargoLock(zbusVersion: string,): string {
  return [
    '[[package]]',
    'name = "zbus"',
    `version = "${zbusVersion}"`,
    'source = "registry+https://github.com/rust-lang/crates.io-index"',
    '',
    '[[package]]',
    'name = "serde"',
    'version = "1.0.200"',
    '',
  ].join('\n',);
}

/**
 A pnpm lock text with the importers of two packages.

 @param version - version of `typescript` the first importer locks

 @param otherVersion - version of `typescript` the second importer locks

 @returns lock text
 */
function pnpmLock({
  version,
  otherVersion,
}: {
  readonly version: string;
  readonly otherVersion: string;
},): string {
  return [
    'lockfileVersion: 9',
    '',
    'importers:',
    '',
    '  package/a/b:',
    '    devDependencies:',
    "      '@types/node':",
    "        specifier: 'catalog:'",
    '        version: 22.1.0',
    '      typescript:',
    "        specifier: 'catalog:'",
    `        version: ${version}`,
    '',
    '  package/a/bc:',
    '    devDependencies:',
    '      typescript:',
    "        specifier: 'catalog:'",
    `        version: ${otherVersion}`,
    '',
    'packages:',
    '',
  ].join('\n',);
}

/**
 A toolchain lock text with the given rust and zig versions.

 @param rust - rust version

 @param zig - zig version

 @returns lock text
 */
function toolchainLock({
  rust,
  zig,
}: {
  readonly rust: string;
  readonly zig: string;
},): string {
  return JSON.stringify({ tools: { rust: { version: rust, }, zig: { version: zig, }, bin: { version: '0.29.4', }, }, },);
}

/**
 A failed result of a Rust check with the zbus error.

 @param pass - package path

 @returns the result
 */
function redResult(pass: string,): CheckResult {
  const check: PackageCheck = checksFromTaskNames({ taskNames: [`//${pass}:lint:clippy`,], },)[0]!;
  return {
    check,
    status: 'failed',
    seconds: 3,
    firstError: firstErrorOf(ZBUS_BREAK_OUTPUT,),
  };
}

//endregion Fixtures: texts the tests read

await describe({
  name: 'what the daily update reports about the packages it breaks',
  children: [
    describe({
      name: checksFromTaskNames.name,
      children: [
        it({
          name: 'gives a Rust crate its Clippy task and a TypeScript package its types task, and ignores other tasks',
          fn: async () => {
            const checks = checksFromTaskNames({
              taskNames: [
                '//package/a/b:lint:clippy',
                '//package/a/b:build',
                '//package/c/d:lint:types',
                '//package/c/d:lint:oxlint',
                '//package/e/f:build',
                '//:lint:types',
              ],
            },);
            expect(checks.map((check,) => [check.package, check.kind, check.task,],),).toEqual([
              ['package/a/b', 'rust', 'lint:clippy',],
              ['package/c/d', 'typescript', 'lint:types',],
            ],);
          },
        },),
        it({
          name: 'checks both kinds of a package that has both tasks, in a stable order, with a finite time bound each',
          fn: async () => {
            const checks = checksFromTaskNames({
              taskNames: [
                '//package/z/z:lint:types',
                '//package/a/b:lint:types',
                '//package/a/b:lint:clippy',
              ],
            },);
            expect(checks.map((check,) => `${check.package}:${check.task}`,),).toEqual([
              'package/a/b:lint:clippy',
              'package/a/b:lint:types',
              'package/z/z:lint:types',
            ],);
            for (const check of checks) {
              expect(Number.isFinite(check.timeoutMs,),).toBe(true,);
              expect(check.timeoutMs,).toBeGreaterThan(0,);
            }
          },
        },),
      ],
    },),
    describe({
      name: firstErrorOf.name,
      children: [
        it({
          name: 'finds the first Rust error with its code, skipping warnings and the closing summary error',
          fn: async () => {
            const error = firstErrorOf(ZBUS_BREAK_OUTPUT,);
            expect(error?.code,).toBe('E0433',);
            expect(error?.message,).toBe('failed to resolve: could not find `blocking` in `zbus`',);
            expect(error?.excerpt,).toContain('src/session.rs:12:11',);
            expect(error?.excerpt,).not.toContain('could not compile',);
          },
        },),
        it({
          name: 'finds a TypeScript error code and a Rust error without one, and strips colors and the task prefix',
          fn: async () => {
            expect(firstErrorOf('src/x.ts(3,5): error TS2307: Cannot find module \'foo\'.',)?.code,).toBe('TS2307',);
            expect(firstErrorOf('error: failed to run custom build command for `x`',)?.code,).toBe('',);
            expect(firstErrorOf('\u001B[31merror[E0425]\u001B[0m: cannot find value `y`',)?.code,).toBe('E0425',);
            expect(firstErrorOf('[//package/a/b:lint:clippy] error[E0425]: cannot find value `y`',)?.message,).toBe(
              'cannot find value `y`',
            );
          },
        },),
        it({
          name: 'answers nothing for output without an error',
          fn: async () => {
            expect(firstErrorOf('all good\nwarning: something',),).toBe(undefined,);
          },
        },),
      ],
    },),
    describe({
      name: causeOf.name,
      children: [
        it({
          name: 'is the error code when there is one, and a digit-blind digest of the message when there is not',
          fn: async () => {
            expect(causeOf({ code: 'E0433', message: 'a at 12', excerpt: '', },),).toBe('E0433',);
            const one = causeOf({ code: '', message: 'failed to run build script at line 12', excerpt: '', },);
            const two = causeOf({ code: '', message: 'failed to run build script at line 98', excerpt: '', },);
            const other = causeOf({ code: '', message: 'linker not found', excerpt: '', },);
            expect(one,).toBe(two,);
            expect(one,).not.toBe(other,);
          },
        },),
      ],
    },),
    describe({
      name: compareChecks.name,
      children: [
        it({
          name: 'names a check newly red only when it passed before and fails after',
          fn: async () => {
            const check = checksFromTaskNames({ taskNames: ['//package/a/b:lint:clippy',], },)[0]!;
            /**
             Builds a result with a status.
             */
            const result = (status: CheckResult['status'],): CheckResult => ({ check, status, seconds: 1, firstError: undefined, });
            const verdicts = {
              newlyRed: compareChecks({ before: [result('passed',),], after: [result('failed',),], },)[0]?.verdict,
              alreadyRed: compareChecks({ before: [result('failed',),], after: [result('failed',),], },)[0]?.verdict,
              recovered: compareChecks({ before: [result('failed',),], after: [result('passed',),], },)[0]?.verdict,
              stillGreen: compareChecks({ before: [result('passed',),], after: [result('passed',),], },)[0]?.verdict,
            };
            expect(verdicts,).toEqual({
              newlyRed: 'newly-red',
              alreadyRed: 'already-red',
              recovered: 'recovered',
              stillGreen: 'still-green',
            },);
          },
        },),
        it({
          name: 'cannot tell when either side timed out, never ran or is missing, and says why',
          fn: async () => {
            const check = checksFromTaskNames({ taskNames: ['//package/a/b:lint:clippy',], },)[0]!;
            const result = (status: CheckResult['status'],): CheckResult => ({ check, status, seconds: 1, firstError: undefined, });
            for (
              const [before, after,] of [
                ['timed-out', 'failed',],
                ['passed', 'timed-out',],
                ['not-run', 'failed',],
                ['passed', 'not-run',],
              ] as const
            ) {
              const [comparison,] = compareChecks({ before: [result(before,),], after: [result(after,),], },);
              expect(comparison?.verdict,).toBe('unknown',);
              expect(comparison?.reason.length,).toBeGreaterThan(0,);
            }
            expect(compareChecks({ before: [], after: [result('failed',),], },)[0]?.verdict,).toBe('unknown',);
          },
        },),
      ],
    },),
    describe({
      name: 'lock moves',
      children: [
        it({
          name: `${cargoLockMoves.name} lists moved, added and removed crates and leaves unchanged ones out`,
          fn: async () => {
            const before = `${cargoLock('5.19.0',)}\n[[package]]\nname = "gone"\nversion = "0.1.0"\n`;
            const after = `${cargoLock('6.2.1',)}\n[[package]]\nname = "fresh"\nversion = "2.0.0"\n`;
            expect(cargoLockMoves({ before, after, },),).toEqual([
              { name: 'fresh', from: '', to: '2.0.0', },
              { name: 'gone', from: '0.1.0', to: '', },
              { name: 'zbus', from: '5.19.0', to: '6.2.1', },
            ],);
          },
        },),
        it({
          name: `${cargoLockMoves.name} reports only the version of a crate locked twice that moved`,
          fn: async () => {
            const lock = (second: string,) =>
              `[[package]]\nname = "syn"\nversion = "1.0.1"\n\n[[package]]\nname = "syn"\nversion = "${second}"\n`;
            expect(cargoLockMoves({ before: lock('2.0.1',), after: lock('2.0.2',), },),).toEqual([
              { name: 'syn', from: '2.0.1', to: '2.0.2', },
            ],);
          },
        },),
        it({
          name: `${pnpmImporterMoves.name} reads only the importer asked for, not one whose path starts the same`,
          fn: async () => {
            const before = pnpmLock({ version: '5.9.0', otherVersion: '5.9.0', },);
            const after = pnpmLock({ version: '6.0.1', otherVersion: '7.0.0', },);
            expect(pnpmImporterMoves({ before, after, importer: 'package/a/b', },),).toEqual([
              { name: 'typescript', from: '5.9.0', to: '6.0.1', },
            ],);
            expect(pnpmImporterMoves({ before, after, importer: 'package/a/bc', },),).toEqual([
              { name: 'typescript', from: '5.9.0', to: '7.0.0', },
            ],);
            expect(pnpmImporterMoves({ before, after, importer: 'package/none', },),).toEqual([],);
          },
        },),
        it({
          name: `${toolchainLockMoves.name} lists the tools whose locked version moved`,
          fn: async () => {
            const before = toolchainLock({ rust: 'nightly-2026-11-01', zig: '0.15.2', },);
            const after = toolchainLock({ rust: 'nightly-2026-11-07', zig: '0.15.2', },);
            expect(toolchainLockMoves({ before, after, },),).toEqual([
              { name: 'rust', from: 'nightly-2026-11-01', to: 'nightly-2026-11-07', },
            ],);
            expect(toolchainLockMoves({ before: '', after, },),).toEqual([],);
            expect(toolchainLockMoves({ before: 'not json', after, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: renderIssue.name,
      children: [
        it({
          name: 'says the update commit, what moved with the named crate first, the first error and how to reproduce it',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a', 'e72ed2045b',],
              baselineCommit: 'c0ffee0001',
              moves: [
                {
                  source: 'package/a/b/Cargo.lock',
                  moves: [
                    { name: 'serde', from: '1.0.1', to: '1.0.2', },
                    { name: 'zbus', from: '5.19.0', to: '6.2.1', },
                  ],
                },
                { source: 'toolchain.lock.json', moves: [{ name: 'rust', from: 'nightly-2026-11-01', to: 'nightly-2026-11-07', },], },
              ],
            },);
            expect(issue.title,).toContain('package/a/b',);
            expect(issue.title,).toContain('lint:clippy',);
            expect(issue.title,).toContain('E0433',);
            expect(issue.labels,).toEqual(['needs-triage',],);
            expect(issue.body,).toContain('d61dc1934a',);
            expect(issue.body,).toContain('e72ed2045b',);
            expect(issue.body,).toContain('c0ffee0001',);
            expect(issue.body,).toContain('`zbus` 5.19.0 to 6.2.1',);
            expect(issue.body,).toContain('`rust` nightly-2026-11-01 to nightly-2026-11-07',);
            expect(issue.body.indexOf('`zbus`',),).toBeLessThan(issue.body.indexOf('`serde`',),);
            expect(issue.body,).toContain('error[E0433]',);
            expect(issue.body,).toContain('src/session.rs:12:11',);
            expect(issue.body,).toContain('mise run //package/a/b:lint:clippy',);
            expect(issue.body.trimEnd().endsWith(issue.marker,),).toBe(true,);
            expect(issue.comment,).toContain('d61dc1934a',);
            expect(issue.comment,).toContain('E0433',);
          },
        },),
        it({
          name: 'caps the moves it lists and says how many it left out',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const many = Array.from({ length: MAXIMUM_MOVE_LINES + 7, }, (_, index,) => ({
              name: `crate${String(index,).padStart(3, '0',)}`,
              from: '1.0.0',
              to: '1.0.1',
            }),);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [{ source: 'package/a/b/Cargo.lock', moves: many, },],
            },);
            expect(issue.body.match(/^- `crate\d+`/gm,)?.length,).toBe(MAXIMUM_MOVE_LINES,);
            expect(issue.body,).toContain('and 7 more',);
          },
        },),
        it({
          name: 'says plainly when no lock moved for the package, and when no update commit exists',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: [],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            expect(issue.body,).toContain('no lock of this package moved',);
            expect(issue.body,).toContain('no commit',);
          },
        },),
        it({
          name: 'builds a marker that one package never shares with another whose path starts the same',
          fn: async () => {
            const one = issueMarker({ package: 'package/a/b', task: 'lint:clippy', cause: 'E0433', },);
            const other = issueMarker({ package: 'package/a/bc', task: 'lint:clippy', cause: 'E0433', },);
            expect(one.includes(other,),).toBe(false,);
            expect(other.includes(one,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: publishIssues.name,
      children: [
        it({
          name: 'creates an issue with the triage label and sends the body on standard input, never on the command line',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const calls: { readonly args: readonly string[]; readonly stdin: string; }[] = [];
            const outcomes = await publishIssues({
              gh: (request,) => {
                calls.push(request,);
                return Promise.resolve(answerLikeAnEmptyTracker(request,),);
              },
              issues: [issue,],
            },);
            expect(outcomes[0]?.action,).toBe('created',);
            expect(outcomes[0]?.issueNumber,).toBe(501,);
            expect(calls.map((call,) => call.args.slice(0, 2,).join(' ',),),).toEqual(['issue list', 'issue create',],);
            const create = calls[1]!;
            expect(create.args,).toContain('--label',);
            expect(create.args,).toContain('needs-triage',);
            expect(create.args,).toContain('--title',);
            expect(create.args.join(' ',),).not.toContain(issue.body,);
            expect(create.stdin,).toBe(issue.body,);
          },
        },),
        it({
          name: 'comments on an open issue for the same package and cause instead of opening another',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const calls: { readonly args: readonly string[]; readonly stdin: string; }[] = [];
            const outcomes = await publishIssues({
              gh: (request,) => {
                calls.push(request,);
                return Promise.resolve(
                  request.args[1] === 'list'
                    ? {
                      exitCode: 0,
                      stdout: JSON.stringify([{ number: 42, body: `an earlier report\n${issue.marker}`, },],),
                      stderr: '',
                    }
                    : answerLikeAnEmptyTracker(request,),
                );
              },
              issues: [issue,],
            },);
            expect(outcomes[0]?.action,).toBe('commented',);
            expect(outcomes[0]?.issueNumber,).toBe(42,);
            expect(calls.some((call,) => call.args[1] === 'create',),).toBe(false,);
            const comment = calls.find((call,) => call.args[1] === 'comment',)!;
            expect(comment.args[2],).toBe('42',);
            expect(comment.stdin,).toBe(issue.comment,);
          },
        },),
        it({
          name: 'does not mistake an open issue of a package whose path starts the same for the same package',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const neighbor = issueMarker({ package: 'package/a/bc', task: 'lint:clippy', cause: 'E0433', },);
            const outcomes = await publishIssues({
              gh: (request,) =>
                Promise.resolve(
                  request.args[1] === 'list'
                    ? { exitCode: 0, stdout: JSON.stringify([{ number: 7, body: neighbor, },],), stderr: '', }
                    : answerLikeAnEmptyTracker(request,),
                ),
              issues: [issue,],
            },);
            expect(outcomes[0]?.action,).toBe('created',);
          },
        },),
        it({
          name: 'turns a second finding with the same marker in one run into a comment on the issue the first created',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const outcomes = await publishIssues({
              gh: (request,) => Promise.resolve(answerLikeAnEmptyTracker(request,),),
              issues: [issue, issue,],
            },);
            expect(outcomes.map((outcome,) => [outcome.action, outcome.issueNumber,],),).toEqual([
              ['created', 501,],
              ['commented', 501,],
            ],);
          },
        },),
        it({
          name: 'reports every issue as failed with the reason when listing fails, and when gh cannot even start',
          fn: async () => {
            const failed = redResult('package/a/b',);
            const issue = renderIssue({
              check: failed.check,
              firstError: failed.firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const offline = await publishIssues({
              gh: () => Promise.resolve({ exitCode: 1, stdout: '', stderr: 'error connecting to api.github.com', },),
              issues: [issue,],
            },);
            expect(offline[0]?.action,).toBe('failed',);
            expect(offline[0]?.reason,).toContain('error connecting to api.github.com',);
            const missing = await publishIssues({
              gh: () => Promise.reject(new Error('spawn gh ENOENT',),),
              issues: [issue,],
            },);
            expect(missing[0]?.action,).toBe('failed',);
            expect(missing[0]?.reason,).toContain('spawn gh ENOENT',);
          },
        },),
        it({
          name: 'keeps going after one issue fails to be created and reports that one as failed',
          fn: async () => {
            const first = renderIssue({
              check: redResult('package/a/b',).check,
              firstError: redResult('package/a/b',).firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            const second = renderIssue({
              check: redResult('package/c/d',).check,
              firstError: redResult('package/c/d',).firstError!,
              updateCommits: ['d61dc1934a',],
              baselineCommit: 'c0ffee0001',
              moves: [],
            },);
            let creates = 0;
            const outcomes = await publishIssues({
              gh: (request,) => {
                if (request.args[1] === 'create') {
                  creates += 1;
                  if (creates === 1)
                    return Promise.resolve({ exitCode: 1, stdout: '', stderr: 'HTTP 502', },);
                }
                return Promise.resolve(answerLikeAnEmptyTracker(request,),);
              },
              issues: [first, second,],
            },);
            expect(outcomes.map((outcome,) => outcome.action,),).toEqual(['failed', 'created',],);
          },
        },),
        it({
          name: 'asks gh for nothing when there is nothing to publish',
          fn: async () => {
            let called = false;
            const outcomes = await publishIssues({
              gh: () => {
                called = true;
                return Promise.resolve({ exitCode: 0, stdout: '', stderr: '', },);
              },
              issues: [],
            },);
            expect(outcomes,).toEqual([],);
            expect(called,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: runChecks.name,
      children: [
        it({
          name: 'never runs more checks side by side than the bound of each kind allows',
          fn: async () => {
            const names = [
              ...Array.from({ length: RUST_CHECK_PARALLEL + 3, }, (_, index,) => `//package/r/crate${index}:lint:clippy`,),
              ...Array.from({ length: TYPESCRIPT_CHECK_PARALLEL + 3, }, (_, index,) => `//package/t/pkg${index}:lint:types`,),
            ];
            const { checkEffects, world, } = fixture({
              tracked: {},
              checks: Object.fromEntries(names.map((name,) => [name, { before: PASSES, after: PASSES, },],),),
            },);
            const checks = checksFromTaskNames({ taskNames: names, },);
            const results = await runChecks({ effects: checkEffects, checks, deadlineAt: CHECK_PASS_DEADLINE_MS, log: () => undefined, },);
            expect(results.length,).toBe(checks.length,);
            expect(results.every((result,) => result.status === 'passed',),).toBe(true,);
            expect(world.maximumInFlight,).toBeLessThanOrEqual(RUST_CHECK_PARALLEL + TYPESCRIPT_CHECK_PARALLEL,);
            expect(world.maximumInFlight,).toBeGreaterThan(1,);
          },
        },),
        it({
          name: 'marks a check that ran past its bound as timed out and one that starts after the deadline as not run',
          fn: async () => {
            const names = Array.from({ length: TYPESCRIPT_CHECK_PARALLEL + 2, }, (_, index,) => `//package/t/p${index}:lint:types`,);
            const { checkEffects, } = fixture({
              tracked: {},
              checks: Object.fromEntries(names.map((name, index,) => [
                name,
                {
                  before: index === 0
                    ? { exitCode: 1, timedOut: true, seconds: 5, }
                    : index === 1
                    ? { exitCode: 1, seconds: 5, output: 'error TS1005: x', }
                    : { exitCode: 0, seconds: 5, },
                  after: PASSES,
                },
              ],),),
            },);
            const results = await runChecks({
              effects: checkEffects,
              checks: checksFromTaskNames({ taskNames: names, },),
              deadlineAt: 1,
              log: () => undefined,
            },);
            expect(results.slice(0, 2,).map((result,) => result.status,),).toEqual(['timed-out', 'failed',],);
            expect(results.slice(-2,).map((result,) => result.status,),).toEqual(['not-run', 'not-run',],);
            expect(results[1]?.firstError?.code,).toBe('TS1005',);
          },
        },),
      ],
    },),
    describe({
      name: runUpdateWithChecks.name,
      children: [
        it({
          name: 'opens one issue for the package the update broke and none for the one that was already red',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: {
                'toolchain.lock.json': toolchainLock({ rust: 'nightly-2026-11-01', zig: '0.15.2', },),
                'package/a/b/Cargo.lock': cargoLock('5.19.0',),
                'package/c/d/Cargo.lock': cargoLock('5.19.0',),
              },
              checks: {
                '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 101, output: ZBUS_BREAK_OUTPUT, }, },
                '//package/c/d:lint:clippy': {
                  before: { exitCode: 101, output: 'error[E0277]: old trouble', },
                  after: { exitCode: 101, output: 'error[E0277]: old trouble', },
                },
                '//package/e/f:lint:types': { before: PASSES, after: PASSES, },
              },
              edits: {
                'mise run toolchain:update': {
                  'toolchain.lock.json': toolchainLock({ rust: 'nightly-2026-11-07', zig: '0.15.2', },),
                },
                'mise exec -- cargo-shadow update': {
                  'package/a/b/Cargo.lock': cargoLock('6.2.1',),
                  'package/c/d/Cargo.lock': cargoLock('6.2.1',),
                },
              },
            },);
            const result = await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            const creates = world.ghCalls.filter((call,) => call.args[1] === 'create',);
            expect(creates.length,).toBe(1,);
            expect(creates[0]?.stdin,).toContain('package/a/b',);
            expect(creates[0]?.stdin,).not.toContain('package/c/d',);
            expect(creates[0]?.stdin,).toContain('`zbus` 5.19.0 to 6.2.1',);
            expect(creates[0]?.stdin,).toContain('`rust` nightly-2026-11-01 to nightly-2026-11-07',);
            expect(creates[0]?.stdin,).toContain(world.commits[0]!.id,);
            expect(creates[0]?.stdin,).toContain('E0433',);
            expect(result.exitCode,).toBe(0,);
            expect(result.checks.newlyRed.map((entry,) => entry.check.package,),).toEqual(['package/a/b',],);
            expect(result.checks.alreadyRed.map((entry,) => entry.check.package,),).toEqual(['package/c/d',],);
          },
        },),
        it({
          name: 'checks every package before the update and again after it commits, and keeps the commits whatever the checks say',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: {
                '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 101, output: ZBUS_BREAK_OUTPUT, }, },
                '//package/e/f:lint:types': { before: PASSES, after: PASSES, },
              },
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
            },);
            await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(world.checksRun.map((run,) => `${run.pass}:${run.key}`,).toSorted(),).toEqual([
              'after://package/a/b:lint:clippy',
              'after://package/e/f:lint:types',
              'before://package/a/b:lint:clippy',
              'before://package/e/f:lint:types',
            ],);
            expect(world.commits.length,).toBe(1,);
            expect(world.files.get('/repo/toolchain.lock.json',),).toBe('after',);
            const firstUpdateCommand = world.ran[0];
            expect(firstUpdateCommand,).toBe('mise run toolchain:update',);
          },
        },),
        it({
          name: 'runs the baseline before the first update command and the second pass only after the commit',
          fn: async () => {
            const order: string[] = [];
            const { effects, checkEffects, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: PASSES, }, },
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
            },);
            const run = effects.runCommand;
            const commit = effects.commit;
            const check = checkEffects.runCheck;
            await runUpdateWithChecks({
              effects: {
                ...effects,
                runCommand: (command,) => {
                  order.push(`update ${command.label}`,);
                  return run(command,);
                },
                commit: (request,) => {
                  order.push('commit',);
                  return commit(request,);
                },
              },
              checkEffects: {
                ...checkEffects,
                runCheck: (item,) => {
                  order.push(`check ${item.package}`,);
                  return check(item,);
                },
              },
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(order[0],).toBe('check package/a/b',);
            expect(order.lastIndexOf('check package/a/b',),).toBeGreaterThan(order.indexOf('commit',),);
            expect(order.indexOf('update mise run toolchain:update',),).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'checks nothing a second time and asks gh for nothing when the update moved nothing',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'same', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 101, output: ZBUS_BREAK_OUTPUT, }, }, },
            },);
            await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(world.checksRun.map((run,) => run.pass,),).toEqual(['before',],);
            expect(world.ghCalls,).toEqual([],);
          },
        },),
        it({
          name: 'comments on the open issue when the same package breaks for the same cause again',
          fn: async () => {
            const earlier = renderIssue({
              check: redResult('package/a/b',).check,
              firstError: redResult('package/a/b',).firstError!,
              updateCommits: ['0000000000',],
              baselineCommit: '1111111111',
              moves: [],
            },);
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 101, output: ZBUS_BREAK_OUTPUT, }, }, },
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
              gh: (request,) =>
                request.args[1] === 'list'
                  ? { exitCode: 0, stdout: JSON.stringify([{ number: 9, body: earlier.marker, },],), stderr: '', }
                  : answerLikeAnEmptyTracker(request,),
            },);
            await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(world.ghCalls.some((call,) => call.args[1] === 'create',),).toBe(false,);
            expect(world.ghCalls.find((call,) => call.args[1] === 'comment',)?.args[2],).toBe('9',);
          },
        },),
        it({
          name: 'never opens an issue for a check that timed out or never ran, and says so in the notification',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 1, timedOut: true, }, }, },
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
            },);
            const result = await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(world.ghCalls,).toEqual([],);
            expect(result.checks.unknown.length,).toBe(1,);
            expect(world.notifications.map((note,) => note.body,).join('\n',),).toContain('package/a/b',);
          },
        },),
        it({
          name: 'logs a gh failure, keeps the issue for the next run and still exits as the update did',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: { exitCode: 101, output: ZBUS_BREAK_OUTPUT, }, }, },
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
              gh: () => ({ exitCode: 1, stdout: '', stderr: 'error connecting to api.github.com', }),
            },);
            const result = await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(result.exitCode,).toBe(0,);
            expect(world.pending.length,).toBe(1,);
            expect(world.notifications.map((note,) => note.body,).join('\n',),).toContain('error connecting to api.github.com',);
          },
        },),
        it({
          name: 'retries a pending issue at the next run, even when that run moves nothing, and forgets it once created',
          fn: async () => {
            const pendingIssue = renderIssue({
              check: redResult('package/a/b',).check,
              firstError: redResult('package/a/b',).firstError!,
              updateCommits: ['0000000000',],
              baselineCommit: '1111111111',
              moves: [],
            },);
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'same', },
              checks: { '//package/a/b:lint:clippy': { before: PASSES, after: PASSES, }, },
              pending: [pendingIssue,],
            },);
            await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(world.ghCalls.filter((call,) => call.args[1] === 'create',).length,).toBe(1,);
            expect(world.pending,).toEqual([],);
          },
        },),
        it({
          name: 'finishes with the update result when listing the checks throws, and tells the owner',
          fn: async () => {
            const { effects, checkEffects, world, } = fixture({
              tracked: { 'toolchain.lock.json': 'before', },
              checks: {},
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'after', }, },
              listingFails: true,
            },);
            const result = await runUpdateWithChecks({
              effects,
              checkEffects,
              groups: dailyUpdateGroups({ globalLockPath: FIXTURE_GLOBAL_LOCK, },),
            },);
            expect(result.exitCode,).toBe(0,);
            expect(world.commits.length,).toBe(1,);
            expect(world.notifications.map((note,) => note.body,).join('\n',),).toContain('mise tasks ls failed',);
          },
        },),
      ],
    },),
  ],
},);
