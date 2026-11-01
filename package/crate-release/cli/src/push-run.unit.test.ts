import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  commitType,
  compareFileLimit,
  comparisonStatuses,
  defaultPushAttempts,
  failureExitCode,
  lockFileName,
  manifestFileName,
  staleHeadMessage,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { treeIdsRequestFileName } from './push-decision.ts';
import type { PushOptions } from './push-options.ts';
import {
  headParentsArguments,
  headPathsArguments,
  headSubjectArguments,
  pushRelease,
  requestFileName,
} from './push-run.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';

/**
 Options of the fixture push: a crate with one lock directory.
 */
const options: PushOptions = {
  root: '/work',
  repo: 'Aquaticats/Monochromatic',
  branch: 'main',
  scratchDirectory: '/runner/temp',
  crateName: 'crate-a',
  crateDirectory: 'package/a',
  lockDirectories: ['package/a.fuzz'],
  maxAttempts: defaultPushAttempts,
};

/**
 Make a distinct full object name.

 @param index - which name
 @returns forty lower-case hexadecimal digits
 */
function objectName(index: number): string {
  return index.toString(16)
    .padStart(
    40,
    '0'
    );
}

/**
 The commit the checkout started from, and the release commit on top of it.
 */
const checkout = objectName(1);
const releaseCommit = objectName(2);

/**
 The first of the object names the branch moves through.
 */
const movedFrom = 3;

/**
 The subject the release commit carries, built the way a release builds it.
 */
const headline = `${commitType}(a): release 0.1.1: n`;

/**
 The files the release commit changes: the crate's own, a pin file in another package and a lock directory's lockfile.
 */
const commitPaths = [
  'package/a/Cargo.toml',
  'package/a/Cargo.lock',
  'package/pin/Cargo.toml',
  'package/a.fuzz/Cargo.lock',
];

/**
 Build a finished call that succeeded.

 @param stdout - what the program printed
 @returns the result
 */
function succeeded(stdout: string): ProgramResult {
  return {
    status: 0,
    stdout,
    stderr: ''
  };
}

/**
 The answer of the commit mutation when the commit was created.
 */
const created = succeeded(JSON.stringify({
  data: {
    createCommitOnBranch: {
      commit: {
        oid: 'cccc',
        url: 'https://github.com/o/r/commit/cccc'
      }
    }
  }
}));

/**
 The answer of the commit mutation when the branch is no longer at the expected head.
 */
const stale: ProgramResult = {
  status: 1,
  stdout: JSON.stringify({ errors: [{ message: `${staleHeadMessage} "${checkout}" but it did not.` }] }),
  stderr: 'gh: stale',
};

/**
 Build the answer of the reference endpoint.

 @param head - the commit the branch points to
 @returns the result
 */
function refAnswer(head: string): ProgramResult {
  return succeeded(JSON.stringify({
    object: {
      type: 'commit',
      sha: head
    }
  }));
}

/**
 Build the answer of the compare endpoint.

 @param status - how the head compares with the base
 @param files - the paths the comparison lists as changed
 @returns the result
 */
function compareAnswer({
  status,
  files,
}: {
  readonly status: string;
  readonly files: readonly string[];
}): ProgramResult {
  return succeeded(JSON.stringify({
    status,
    files: files.map(function entryOf(filename) {
      return { filename };
    })
  }));
}

/**
 Build the answer of `cargo metadata` for path packages under the fixture root.

 @param directories - repository-relative directories of the path packages
 @param workspace - repository-relative workspace root
 @returns the result
 */
function metadataAnswer({
  directories,
  workspace,
}: {
  readonly directories: readonly string[];
  readonly workspace: string;
}): ProgramResult {
  return succeeded(JSON.stringify({
    packages: [
      ...directories.map(function packageOf(directory) {
        return {
          name: directory,
          source: null,
          manifest_path: join(
            options.root,
            directory,
            manifestFileName
          )
        };
      }),
      {
        name: 'registry-crate',
        source: 'registry+https://github.com/rust-lang/crates.io-index',
        manifest_path: join(
          '/cargo/registry',
          manifestFileName
        )
      },
    ],
    workspace_root: join(
      options.root,
      workspace
    ),
  }));
}

/**
 A workspace root that is no package directory and holds a manifest and a lockfile the lock directory inherits.
 */
const workspaceRoot = 'package/wsroot';

/**
 The directories of the fixture's dependency closure, by the manifest `cargo metadata` reads.
 */
const closures: Readonly<Record<string, ProgramResult>> = {
  [`package/a/${manifestFileName}`]: metadataAnswer({
    directories: ['package/a', 'package/dep'],
    workspace: 'package/a'
  }),
  [`package/a.fuzz/${manifestFileName}`]: metadataAnswer({
    directories: ['package/a.fuzz', 'package/a', 'package/dep2'],
    workspace: workspaceRoot,
  }),
};

/**
 A file of the crate directory and a file of a path dependency of the crate, which the tests name in a refusal.
 */
const crateFileChange = 'package/a/src/lib.rs';
const dependencyFileChange = 'package/dep/src/lib.rs';

/**
 Paths that lie in each part of what the release depends on, so a change to any one of them must stop the push.
 */
const watchedChanges: readonly (readonly [string, string])[] = [
  ['a file the release commit changes', 'package/pin/Cargo.toml'],
  ['a file of the crate directory', crateFileChange],
  ['a file of a lock directory', 'package/a.fuzz/fuzz_targets/target.rs'],
  ['a file of a path dependency of the crate', dependencyFileChange],
  ['a file of a path dependency of the lock directory', 'package/dep2/build.rs'],
  ['the lockfile of a workspace root', `${workspaceRoot}/${lockFileName}`],
];

/**
 Paths that change on a busy branch and that the release does not depend on, including names that only start like a
 watched directory.
 */
const unrelatedChanges: readonly string[] = [
  'doc/handover/notes.md',
  'package/other/src/lib.rs',
  'package/ab/src/lib.rs',
  'package/a.fuzz2/src/lib.rs',
  'package/dep-extra/src/lib.rs',
  `${workspaceRoot}/README.md`,
];

/**
 Canned answers of the fake world around the push.
 */
type World = {
  /**
   Successive answers of the commit mutation; the last one repeats.
   */
  readonly commitAnswers?: readonly ProgramResult[];
  /**
   Successive answers of the reference endpoint; the last one repeats.
   */
  readonly heads?: readonly ProgramResult[];
  /**
   Answer of the compare endpoint, by the head it is asked about.
   */
  readonly compare?: (head: string) => ProgramResult;
  /**
   Paths whose object names differ between the base and any head.
   */
  readonly changedTreePaths?: readonly string[];
  /**
   Answer of every `git rev-list` call.
   */
  readonly parents?: ProgramResult;
  /**
   Answer of every `git diff-tree` call.
   */
  readonly changedFiles?: ProgramResult;
  /**
   Answer of every `git log` call.
   */
  readonly subject?: ProgramResult;
  /**
   Answer of every `cargo metadata` call, in place of the fixture closures.
   */
  readonly cargo?: ProgramResult;
  /**
   Options of the push.
   */
  readonly pushOptions?: PushOptions;
};

/**
 What a run left behind, for assertions.
 */
type Recording = {
  readonly status: number;
  readonly calls: readonly ProgramCall[];
  readonly written: readonly { readonly path: string; readonly text: string }[];
  readonly printed: readonly string[];
  readonly reads: readonly string[];
  /**
   The input of every commit mutation that was sent, in order.
   */
  readonly sends: readonly {
    readonly expectedHeadOid: string;
    readonly message: { readonly headline: string };
    readonly fileChanges: { readonly additions: readonly { readonly path: string; readonly contents: string }[] };
  }[];
};

/**
 Take the answer at an index of a list whose last answer repeats.

 @param answers - the answers
 @param index - which send or read this is, from zero
 @returns the answer
 */
function answerAt({
  answers,
  index,
}: {
  readonly answers: readonly ProgramResult[];
  readonly index: number;
}): ProgramResult {
  return answers[Math.min(
    index,
    answers.length - 1
  )] ?? succeeded('');
}

/**
 Run a push against a fake world.

 @param world - canned answers
 @returns the recorded calls, writes, reads, sends and printed lines
 */
async function record(world: World = {}): Promise<Recording> {
  const counters = {
    commits: 0,
    refs: 0
  };
  const calls: ProgramCall[] = [];
  const written: { path: string; text: string }[] = [];
  const printed: string[] = [];
  const reads: string[] = [];
  const status = await pushRelease({
    options: world.pushOptions ?? options,
    run: async function run(call) {
      calls.push(call);
      const [first = '', second = ''] = call.args;
      if (call.program === 'git') {
        if (first === headParentsArguments[0])
          return world.parents ?? succeeded(`${releaseCommit} ${checkout}\n`);
        if (first === headSubjectArguments[0])
          return world.subject ?? succeeded(`${headline}\n`);
        return world.changedFiles ?? succeeded(`${commitPaths.join('\0')}\0`);
      }
      if (call.program === 'cargo') {
        const manifest = (call.args.find(function isManifest(argument) {
          return argument.startsWith('--manifest-path=');
        }) ?? '').slice('--manifest-path='.length);
        return world.cargo ?? closures[manifest] ?? succeeded('{}');
      }
      if (second === 'graphql') {
        const requestFile = first === 'api' ? (call.args[2] ?? '').slice('--input='.length) : '';
        const request = written.findLast(function isFile(file) {
          return file.path === requestFile;
        })?.text ?? '';
        if (request.includes('createCommitOnBranch')) {
          const answer = answerAt({
            answers: world.commitAnswers ?? [created],
            index: counters.commits
          });
          counters.commits += 1;
          return answer;
        }
        const {variables} = (JSON.parse(request) as { variables: Record<string, string> });
        const changed = world.changedTreePaths ?? [];
        const objects = Object.fromEntries(Object.entries(variables)
          .filter(function isRevision([name]) {
            return name.startsWith('b') || name.startsWith('h');
          })
          .map(function objectOf([name, expression]) {
            const path = expression.slice(expression.indexOf(':') + 1);
            const differs = name.startsWith('h') && changed.includes(path);
            return [name, { oid: `${path}${differs ? '-changed' : ''}` }];
          }));
        return succeeded(JSON.stringify({ data: { repository: objects } }));
      }
      if (second.includes('/git/ref/heads/')) {
        const answer = answerAt({
          answers: world.heads ?? [refAnswer(objectName(3))],
          index: counters.refs
        });
        counters.refs += 1;
        return answer;
      }
      const head = second.slice(second.indexOf('...') + '...'.length);
      return (world.compare ?? function unchanged() {
        return compareAnswer({
          status: comparisonStatuses.ahead,
          files: unrelatedChanges
        });
      })(head);
    },
    readBytes: async function readBytes(path) {
      reads.push(path);
      return new TextEncoder().encode(`contents of ${path}`);
    },
    writeText: async function writeText(file) {
      written.push(file);
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  const sends = written.filter(function isCommit(file) {
    return file.text.includes('createCommitOnBranch');
  })
    .map(function inputOf(file) {
      return (JSON.parse(file.text) as { variables: { input: Recording['sends'][number] } }).variables.input;
    });
  return {
    status,
    calls,
    written,
    printed,
    reads,
    sends
  };
}

/**
 Run a push that must throw and return what it threw.

 @param world - canned answers
 @returns the thrown value
 */
async function thrownBy(world: World): Promise<unknown> {
  try {
    await record(world);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Pick the calls of one program.

 @param recording - what the run left behind
 @param program - program name
 @returns the calls, in order
 */
function callsOf({
  recording,
  program,
}: {
  readonly recording: Recording;
  readonly program: string;
}): readonly ProgramCall[] {
  return recording.calls.filter(function isProgram(call) {
    return call.program === program;
  });
}

/**
 Pick the calls that read the head of the branch.

 @param recording - what the run left behind
 @returns the calls, in order
 */
function refReads(recording: Recording): readonly ProgramCall[] {
  return recording.calls.filter(function isRefRead(call) {
    return (call.args[1] ?? '').includes('/git/ref/heads/');
  });
}

/**
 Pick the calls that compare the branch with the checkout.

 @param recording - what the run left behind
 @returns the calls, in order
 */
function compareReads(recording: Recording): readonly ProgramCall[] {
  return recording.calls.filter(function isCompare(call) {
    return (call.args[1] ?? '').includes('/compare/');
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: pushRelease.name,
      children: [
        it({
          name: 'sends the commit once on the checkout head when the branch did not move, and reads nothing about the branch',
          fn: async () => {
            const result = await record();
            expect(result.status).toBe(0);
            expect(result.sends).toHaveLength(1);
            expect(result.sends[0]?.expectedHeadOid).toBe(checkout);
            expect(result.sends[0]?.message).toEqual({ headline });
            expect(refReads(result)).toEqual([]);
            expect(compareReads(result)).toEqual([]);
            expect(result.printed.at(-1)).toBe('PUSHED cccc https://github.com/o/r/commit/cccc');
          },
        }),
        it({
          name: 'sends the whole contents of the files the release commit changes, read once',
          fn: async () => {
            const result = await record();
            const additions = result.sends[0]?.fileChanges.additions ?? [];
            expect(additions.map(function pathOf(addition) {
              return addition.path;
            })).toEqual(commitPaths);
            for (const addition of additions)
              expect(Buffer.from(addition.contents, 'base64').toString('utf8')).toBe(
                `contents of ${join(options.root, addition.path)}`,
              );
            expect(result.reads).toHaveLength(commitPaths.length);
            expect(result.written[0]?.path).toBe(join(options.scratchDirectory, requestFileName));
          },
        }),
        it({
          name: 'sends the request with gh in the checkout, from the request file in the scratch directory',
          fn: async () => {
            const result = await record();
            expect(callsOf({ recording: result, program: 'gh' })).toEqual([
              {
                program: 'gh',
                args: ['api', 'graphql', `--input=${join(options.scratchDirectory, requestFileName)}`],
                cwd: options.root,
              },
            ]);
          },
        }),
        it({
          name: 'runs git in the checkout and asks for the changed paths NUL-separated, so no path is split or quoted',
          fn: async () => {
            const result = await record();
            const git = callsOf({ recording: result, program: 'git' });
            expect(git.map(function argsOf(call) {
              return call.args.join(' ');
            }).toSorted()).toEqual([
              headPathsArguments.join(' '),
              headSubjectArguments.join(' '),
              headParentsArguments.join(' '),
            ].toSorted());
            expect(git.every(function inRoot(call) {
              return call.cwd === options.root;
            })).toBe(true);
          },
        }),
        it({
          name: 'records the dependency closure of the crate and of each lock directory with cargo metadata before it sends anything',
          fn: async () => {
            const result = await record();
            const cargo = callsOf({ recording: result, program: 'cargo' });
            expect(cargo.map(function manifestOf(call) {
              return call.args.find(function isManifest(argument) {
                return argument.startsWith('--manifest-path=');
              }) ?? '';
            }).toSorted()).toEqual([options.crateDirectory, ...options.lockDirectories].map(function pathOf(directory) {
              return `--manifest-path=${directory}/${manifestFileName}`;
            }).toSorted());
            expect(cargo.every(function locked(call) {
              return (call.cwd === options.root) && call.args.includes('--locked') && (call.args[0] === 'metadata');
            })).toBe(true);
            const firstGh = result.calls.findIndex(function isGh(call) {
              return call.program === 'gh';
            });
            const lastCargo = result.calls.findLastIndex(function isCargo(call) {
              return call.program === 'cargo';
            });
            expect(lastCargo).toBeLessThan(firstGh);
          },
        }),
        it({
          name: 'prints the base, the files and everything it watches before the first send',
          fn: async () => {
            const result = await record();
            const joined = result.printed.join('\n');
            expect(joined).toContain(`base ${checkout}`);
            for (const watched of ['package/dep', 'package/dep2', options.crateDirectory, ...options.lockDirectories])
              expect(joined).toContain(watched);
          },
        }),
        it({
          name: 'sends the same commit again on the new head when the branch moved and nothing the release depends on changed',
          fn: async () => {
            const moved = objectName(3);
            const result = await record({
              commitAnswers: [stale, created],
              heads: [refAnswer(moved)],
            });
            expect(result.status).toBe(0);
            expect(result.sends).toHaveLength(2);
            expect(result.sends.map(function headOf(send) {
              return send.expectedHeadOid;
            })).toEqual([checkout, moved]);
            expect(result.sends[1]?.message).toEqual(result.sends[0]?.message);
            expect(result.sends[1]?.fileChanges).toEqual(result.sends[0]?.fileChanges);
            expect(result.reads).toHaveLength(commitPaths.length);
            expect(result.printed.at(-1)).toBe('PUSHED cccc https://github.com/o/r/commit/cccc');
          },
        }),
        it({
          name: 'prints the base, the new head, the comparison and the decision when it sends again',
          fn: async () => {
            const moved = objectName(3);
            const result = await record({ commitAnswers: [stale, created], heads: [refAnswer(moved)] });
            const joined = result.printed.join('\n');
            expect(joined).toContain(`main now points to ${moved}; the checkout started from ${checkout}`);
            expect(joined).toContain(`compare ${checkout}...${moved}: ${comparisonStatuses.ahead}, ${String(unrelatedChanges.length)} changed files`);
            expect(joined).toContain(`decision: none of the watched paths changed; send the commit again on ${moved}`);
          },
        }),
        it({
          name: 'compares from the checkout base every time, with the head it just read',
          fn: async () => {
            const heads = [objectName(3), objectName(4)];
            const result = await record({
              commitAnswers: [stale, stale, created],
              heads: heads.map(refAnswer),
            });
            expect(result.status).toBe(0);
            expect(compareReads(result).map(function endpointOf(call) {
              return call.args[1];
            })).toEqual(heads.map(function endpointFor(head) {
              return `repos/${options.repo}/compare/${checkout}...${head}`;
            }));
            expect(result.sends.map(function headOf(send) {
              return send.expectedHeadOid;
            })).toEqual([checkout, ...heads]);
          },
        }),
        it({
          name: 'sends again on the same head when the branch is back at the checkout',
          fn: async () => {
            const result = await record({ commitAnswers: [stale, created], heads: [refAnswer(checkout)] });
            expect(result.status).toBe(0);
            expect(compareReads(result)).toEqual([]);
            expect(result.sends.map(function headOf(send) {
              return send.expectedHeadOid;
            })).toEqual([checkout, checkout]);
          },
        }),
        ...watchedChanges.map(function watchedCase([label, path]) {
          return it({
            name: `pushes nothing and names the path when ${label} changed on the branch`,
            fn: async () => {
              const result = await record({
                commitAnswers: [stale, created],
                compare: () => compareAnswer({ status: comparisonStatuses.ahead, files: [...unrelatedChanges, path] }),
              });
              expect(result.status).toBe(failureExitCode);
              expect(result.sends).toHaveLength(1);
              const refusal = result.printed.find(function isRefusal(line) {
                return line.startsWith('PUSH REFUSED');
              }) ?? '';
              expect(refusal).toContain(path);
              expect(refusal).toContain('dispatch the release again');
              expect(refusal).not.toContain(unrelatedChanges[0] ?? '');
            },
          });
        }),
        it({
          name: 'refuses a file renamed out of a watched directory, which the new path alone would hide',
          fn: async () => {
            const oldPath = 'package/dep/src/old.rs';
            const result = await record({
              commitAnswers: [stale, created],
              compare: () => succeeded(JSON.stringify({
                status: comparisonStatuses.ahead,
                files: [{ filename: 'doc/moved.rs', previous_filename: oldPath }],
              })),
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.join('\n')).toContain(oldPath);
          },
        }),
        it({
          name: 'names every watched path that changed, once',
          fn: async () => {
            const first = crateFileChange;
            const second = dependencyFileChange;
            const result = await record({
              commitAnswers: [stale, created],
              compare: () => compareAnswer({ status: comparisonStatuses.ahead, files: [first, second, first] }),
            });
            const refusal = result.printed.find(function isRefusal(line) {
              return line.startsWith('PUSH REFUSED');
            }) ?? '';
            expect(refusal).toContain(`${first}, ${second}`);
            expect(refusal.split(first)).toHaveLength(2);
          },
        }),
        it({
          name: 'refuses when the branch moves before every allowed send, after one fresh comparison per refusal',
          fn: async () => {
            const result = await record({
              commitAnswers: [stale],
              heads: Array.from({ length: options.maxAttempts }, function headAt(_value, index) {
                return refAnswer(objectName(movedFrom + index));
              }),
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(options.maxAttempts);
            expect(compareReads(result)).toHaveLength(options.maxAttempts - 1);
            expect(result.printed.at(-1)).toContain(`moved before each of the ${String(options.maxAttempts)} allowed sends`);
            expect(result.printed.at(-1)).toContain('dispatch the release again');
          },
        }),
        it({
          name: 'sends exactly once and never reads the branch when one send is all that is allowed',
          fn: async () => {
            const result = await record({ commitAnswers: [stale], pushOptions: { ...options, maxAttempts: 1 } });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(1);
            expect(refReads(result)).toEqual([]);
          },
        }),
        it({
          name: 'decides by the object names of the watched paths when the file list reached its limit, and sends again when equal',
          fn: async () => {
            const moved = objectName(3);
            const result = await record({
              commitAnswers: [stale, created],
              heads: [refAnswer(moved)],
              compare: () => compareAnswer({
                status: comparisonStatuses.ahead,
                files: Array.from({ length: compareFileLimit }, function fileAt(_value, index) {
                  return `doc/file-${String(index)}.md`;
                }),
              }),
            });
            expect(result.status).toBe(0);
            expect(result.sends.map(function headOf(send) {
              return send.expectedHeadOid;
            })).toEqual([checkout, moved]);
            const treeRequest = result.written.find(function isTreeRequest(file) {
              return file.path === join(options.scratchDirectory, treeIdsRequestFileName);
            });
            const {variables} = (JSON.parse(treeRequest?.text ?? '{}') as { variables: Record<string, string> });
            const compared = Object.entries(variables)
              .filter(function isBase([name]) {
                return name.startsWith('b');
              })
              .map(function pathOf([, expression]) {
                return expression.slice(expression.indexOf(':') + 1);
              });
            for (const watched of [...commitPaths, options.crateDirectory, ...options.lockDirectories, 'package/dep', 'package/dep2'])
              expect(compared).toContain(watched);
            expect(result.printed.join('\n')).toContain('may be cut');
          },
        }),
        it({
          name: 'pushes nothing and names the path when the object names show a change under a watched directory',
          fn: async () => {
            const changedDirectory = 'package/dep';
            const result = await record({
              commitAnswers: [stale, created],
              changedTreePaths: [changedDirectory],
              compare: () => compareAnswer({
                status: comparisonStatuses.ahead,
                files: Array.from({ length: compareFileLimit }, function fileAt(_value, index) {
                  return `doc/file-${String(index)}.md`;
                }),
              }),
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(1);
            expect(result.printed.find(function isRefusal(line) {
              return line.startsWith('PUSH REFUSED');
            })).toContain(changedDirectory);
          },
        }),
        it({
          name: 'pushes nothing when the comparison cannot be read, naming why',
          fn: async () => {
            const result = await record({
              commitAnswers: [stale, created],
              compare: () => ({ status: 1, stdout: '', stderr: 'HTTP 404: Not Found' }),
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(1);
            const refusal = result.printed.find(function isRefusal(line) {
              return line.startsWith('PUSH REFUSED');
            }) ?? '';
            expect(refusal).toContain('cannot be compared');
            expect(refusal).toContain('HTTP 404: Not Found');
          },
        }),
        ...['behind', 'diverged'].map(function notAheadCase(status) {
          return it({
            name: `pushes nothing when the branch is ${status} relative to the checkout`,
            fn: async () => {
              const result = await record({
                commitAnswers: [stale, created],
                compare: () => compareAnswer({ status, files: unrelatedChanges }),
              });
              expect(result.status).toBe(failureExitCode);
              expect(result.sends).toHaveLength(1);
              expect(result.printed.join('\n')).toContain(`main is ${status} relative to the checkout`);
            },
          });
        }),
        it({
          name: 'pushes nothing when the new head cannot be read',
          fn: async () => {
            const result = await record({
              commitAnswers: [stale, created],
              heads: [{ status: 1, stdout: '', stderr: 'HTTP 502' }],
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(1);
            expect(compareReads(result)).toEqual([]);
            expect(result.printed.join('\n')).toContain('The new head of main cannot be read: HTTP 502');
          },
        }),
        it({
          name: 'ends at once, without reading the branch, on a refusal that is not a moved branch',
          fn: async () => {
            const result = await record({
              commitAnswers: [{
                status: 1,
                stdout: JSON.stringify({ errors: [{ message: 'Resource not accessible by integration' }] }),
                stderr: 'gh: Resource not accessible',
              }],
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.sends).toHaveLength(1);
            expect(refReads(result)).toEqual([]);
            expect(result.printed).toContain('PUSH REFUSED: Resource not accessible by integration');
            expect(result.printed.join('\n')).toContain('Nothing was pushed. gh exited 1: gh: Resource not accessible');
          },
        }),
        it({
          name: 'reports an answer that is not JSON as a refusal and still names the gh status',
          fn: async () => {
            const result = await record({ commitAnswers: [{ status: 4, stdout: '', stderr: 'gh: authentication required' }] });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.join('\n')).toContain('PUSH REFUSED: The text is not JSON');
            expect(result.printed.join('\n')).toContain('gh exited 4: gh: authentication required');
          },
        }),
        it({
          name: 'sends nothing when cargo metadata fails, naming the call and its error text',
          fn: async () => {
            const caught = await thrownBy({ cargo: { status: 101, stdout: '', stderr: 'error: the lock file needs to be updated' } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('cargo metadata');
            expect((caught as Error).message).toContain('status 101');
            expect((caught as Error).message).toContain('the lock file needs to be updated');
          },
        }),
        it({
          name: 'sends nothing when cargo metadata prints something that cannot be used',
          fn: async () => {
            const caught = await thrownBy({ cargo: succeeded('not json') });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('cannot be used');
          },
        }),
        it({
          name: 'refuses a head that is a merge, because it has no single commit that was checked out',
          fn: async () => {
            const caught = await thrownBy({ parents: succeeded(`${releaseCommit} ${checkout} ${objectName(9)}\n`) });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('exactly one parent');
          },
        }),
        it({
          name: 'refuses a head with no parent',
          fn: async () => {
            const caught = await thrownBy({ parents: succeeded(`${releaseCommit}\n`) });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('has 0');
          },
        }),
        it({
          name: 'refuses a head that changes no file',
          fn: async () => {
            const caught = await thrownBy({ changedFiles: succeeded('') });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('nothing to push');
          },
        }),
        it({
          name: 'names the git call, its status and its error text when git fails',
          fn: async () => {
            const caught = await thrownBy({ subject: { status: 128, stdout: '', stderr: 'fatal: bad object HEAD\n' } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain(`git ${headSubjectArguments.join(' ')} failed with status 128`);
            expect((caught as Error).message).toContain('fatal: bad object HEAD');
          },
        }),
        it({
          name: 'sends nothing to GitHub when the head cannot be read',
          fn: async () => {
            const calls: ProgramCall[] = [];
            let caught: unknown;
            try {
              await pushRelease({
                options,
                run: async function run(call) {
                  calls.push(call);
                  return { status: 128, stdout: '', stderr: 'fatal' };
                },
                readBytes: async function readBytes() {
                  return new Uint8Array();
                },
                writeText: async function writeText() {},
                print: function print() {},
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect(calls.some(function isGh(call) {
              return call.program === 'gh';
            })).toBe(false);
          },
        }),
      ],
    }),
  ],
});
