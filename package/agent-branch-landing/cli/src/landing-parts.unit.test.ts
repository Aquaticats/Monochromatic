/**
 The parts a landing is made of, without a repository: the paths a branch changed, the tasks one package declares, the words one
 command runs in, what a task printed, a file's conflict regions, a list file's entries, and a command line's grammar.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  blockedFilesIn,
  changesTheLockfile,
  cheapestCheckTaskAmong,
  cgroupWordsOf,
  entriesIn,
  errorLinesOf,
  isReleasePath,
  keepOneSideOfEachRegion,
  PACKAGE_TASK_LIMITS,
  packagesAmong,
  parseCommandLine,
  releasePathsAmong,
  tailOf,
  tallyLineOf,
  tallyOf,
  tasksDeclaredIn,
  USAGE,
  WRAPPER_COMPILE_LIMITS,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: blockedFilesIn.name,
      children: [
        it({
          name: 'names the paths Git lists after its sentence about overwriting, and nothing else',
          fn: async () => {
            expect(blockedFilesIn('error: Your local changes would be overwritten by merge:\n\tdoc/a.md\n\tdoc/b.md\nPlease commit your changes\n',),).toEqual([
              'doc/a.md',
              'doc/b.md',
            ],);
            expect(blockedFilesIn('CONFLICT (content): merge conflict in doc/a.md',),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: packagesAmong.name,
      children: [
        it({
          name: 'names each changed package once and leaves everything else out',
          fn: async () => {
            expect(packagesAmong([
              'package/a/b/src/x.ts',
              'package/a/b/README.md',
              'doc/a.md',
              'Cargo.toml',
            ],),).toEqual(['package/a/b',],);
          },
        },),
      ],
    },),
    describe({
      name: isReleasePath.name,
      children: [
        it({
          name: 'counts anything below the wrapper and any manifest or lockfile at any depth',
          fn: async () => {
            expect(isReleasePath('package/git-policy/cli/src/lib.rs',),).toBe(true,);
            expect(isReleasePath('package/git-policy',),).toBe(true,);
            expect(isReleasePath('package/a/b/Cargo.toml',),).toBe(true,);
            expect(isReleasePath('Cargo.lock',),).toBe(true,);
            expect(isReleasePath('package/git-policy-clone/src/lib.rs',),).toBe(false,);
            expect(isReleasePath('doc/a.md',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: releasePathsAmong.name,
      children: [
        it({
          name: 'keeps the release paths in the order given, each once',
          fn: async () => {
            expect(releasePathsAmong(['Cargo.toml', 'doc/a.md', 'Cargo.toml',],),).toEqual(['Cargo.toml',],);
          },
        },),
      ],
    },),
    describe({
      name: changesTheLockfile.name,
      children: [
        it({
          name: 'answers for the lockfile alone',
          fn: async () => {
            expect(changesTheLockfile(['pnpm-lock.yaml',],),).toBe(true,);
            expect(changesTheLockfile(['doc/a.md',],),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: tasksDeclaredIn.name,
      children: [
        it({
          name: 'reads quoted, bare and coloned headers, and refuses a table below a task',
          fn: async () => {
            /**
             What the tasks of one `mise.toml` are.
             */
            const declared = tasksDeclaredIn([
              '[tasks.lint]',
              'extends = "lint"',
              '',
              '[tasks."lint:types"]',
              'extends = "lint:types"',
              '',
              '[tasks.test:unit]',
              'extends = "test:unit"',
              '',
              '[tasks.build.env]',
              'EXAMPLE = "1"',
            ].join('\n',),);
            expect([...declared,],).toEqual([
              'lint',
              'lint:types',
              'test:unit',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: cheapestCheckTaskAmong.name,
      children: [
        it({
          name: 'takes the first of the cheapest tasks the package declares',
          fn: async () => {
            expect(
              cheapestCheckTaskAmong(new Set([
                'test',
                'build',
              ],),),
            ).toEqual({
              found: true,
              value: 'build',
            },);
            expect(cheapestCheckTaskAmong(new Set(['lint:oxlint',],),).found,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: cgroupWordsOf.name,
      children: [
        it({
          name: 'gives each kind of work its own limits, and withholds swap only where one is named',
          fn: async () => {
            expect(cgroupWordsOf({
              limits: WRAPPER_COMPILE_LIMITS,
              collect: false,
            },),).toEqual([
              '--user',
              '--scope',
              '--quiet',
              '--property=CPUQuota=800%',
              '--property=MemoryMax=12G',
              '--property=MemorySwapMax=0',
              '--property=TasksMax=3000',
              '--property=IOWeight=10',
              '--',
            ],);
            expect(cgroupWordsOf({
              limits: PACKAGE_TASK_LIMITS,
              collect: true,
            },),).toContain('--collect',);
          },
        },),
      ],
    },),
    describe({
      name: tallyOf.name,
      children: [
        it({
          name: 'counts the harness cases and reads both node counts',
          fn: async () => {
            expect(tallyOf(' [PASS] a\n [FAIL] b\n pass 12\n fail 3\n',),).toEqual({
              passed: 1,
              failed: 1,
              nodePassed: '12',
              nodeFailed: '3',
            },);
            expect(tallyOf('nothing ran\n',),).toEqual({
              passed: 0,
              failed: 0,
              nodePassed: '-',
              nodeFailed: '-',
            },);
          },
        },),
      ],
    },),
    describe({
      name: tallyLineOf.name,
      children: [
        it({
          name: 'names the task, its status and every count on one line',
          fn: async () => {
            expect(tallyLineOf({
              task: '//package/a/b:test:unit',
              status: 0,
              output: ' pass 4\n',
            },),).toBe('//package/a/b:test:unit: status 0, PASS lines 0, FAIL lines 0, node pass 4, node fail -',);
          },
        },),
      ],
    },),
    describe({
      name: errorLinesOf.name,
      children: [
        it({
          name: 'keeps the error lines and the lines naming where they are, each once',
          fn: async () => {
            expect(errorLinesOf({
              output: 'error[E0308]: mismatched types\n --> src/lib.rs:1:1\nnote: elsewhere\nerror[E0308]: mismatched types\n',
              limit: 30,
            },),).toEqual([
              'error[E0308]: mismatched types',
              ' --> src/lib.rs:1:1',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: tailOf.name,
      children: [
        it({
          name: 'keeps the end of what a task printed',
          fn: async () => {
            expect(tailOf({
              output: 'a long failure',
              characters: 7,
            },),).toBe('failure',);
          },
        },),
      ],
    },),
    describe({
      name: keepOneSideOfEachRegion.name,
      children: [
        it({
          name: 'keeps the checkout side, the incoming side, or both in either order',
          fn: async () => {
            /**
             A file holding one conflict region.
             */
            const text = 'before\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> main\nafter\n';
            expect(keepOneSideOfEachRegion({
              text,
              side: 'current',
            },),).toEqual({
              text: 'before\nours\nafter\n',
              regions: 1,
            },);
            expect(keepOneSideOfEachRegion({
              text,
              side: 'incoming',
            },).text,).toBe('before\ntheirs\nafter\n',);
            expect(keepOneSideOfEachRegion({
              text,
              side: 'current-then-incoming',
            },).text,).toBe('before\nours\ntheirs\nafter\n',);
            expect(keepOneSideOfEachRegion({
              text,
              side: 'incoming-then-current',
            },).text,).toBe('before\ntheirs\nours\nafter\n',);
          },
        },),
        it({
          name: 'drops the common ancestor a diff3 conflict shows',
          fn: async () => {
            expect(keepOneSideOfEachRegion({
              text: '<<<<<<< HEAD\nours\n||||||| base\nancestor\n=======\ntheirs\n>>>>>>> main\n',
              side: 'incoming',
            },).text,).toBe('theirs\n',);
          },
        },),
        it({
          name: 'refuses a region that does not close',
          fn: async () => {
            /**
             What was thrown.
             */
            let thrown: unknown;
            try {
              keepOneSideOfEachRegion({
                text: '<<<<<<< HEAD\nours\n',
                side: 'current',
              },);
            }
            catch (caught) {
              thrown = caught;
            }
            expect((thrown as Error).message,).toContain('does not close',);
          },
        },),
      ],
    },),
    describe({
      name: entriesIn.name,
      children: [
        it({
          name: 'reads an entry that names its branch, its worktree, its check, its tasks and its note',
          fn: async () => {
            expect(entriesIn({
              where: '/list.json',
              text: JSON.stringify([{
                branch: 'fix/a',
                worktree: 'pi-fix-a',
                check: 'tasks',
                tasks: ['//package/a/b:lint',],
                message: 'merge(a): b',
                note: 'its lint is the same on main',
                key: 'a',
              },],),
            },),).toEqual([{
              branch: 'fix/a',
              worktree: 'pi-fix-a',
              check: 'tasks',
              tasks: ['//package/a/b:lint',],
              message: 'merge(a): b',
              note: 'its lint is the same on main',
              key: 'a',
            },],);
          },
        },),
        it({
          name: 'derives the check a file written before this command existed meant',
          fn: async () => {
            expect(entriesIn({
              where: '/list.json',
              text: JSON.stringify([
                { agent: 'pi-a', message: 'merge(a): b', },
                { name: 'pi-b', message: 'merge(b): c', },
                { name: 'pi-c', message: 'merge(c): d', consumerTasks: ['//package/c/d:test:unit',], },
                { branch: 'fix/docs', message: 'docs(*): e', },
              ],),
            },),).toEqual([
              {
                branch: '',
                worktree: 'pi-a',
                check: 'wrapper',
                tasks: [],
                message: 'merge(a): b',
                note: '',
                key: '',
              },
              {
                branch: '',
                worktree: 'pi-b',
                check: 'package',
                tasks: [],
                message: 'merge(b): c',
                note: '',
                key: '',
              },
              {
                branch: '',
                worktree: 'pi-c',
                check: 'package',
                tasks: ['//package/c/d:test:unit',],
                message: 'merge(c): d',
                note: '',
                key: '',
              },
              {
                branch: 'fix/docs',
                worktree: '',
                check: 'none',
                tasks: [],
                message: 'docs(*): e',
                note: '',
                key: '',
              },
            ],);
          },
        },),
        it({
          name: 'refuses text that is not JSON, an entry with no message, and a check nobody knows',
          fn: async () => {
            /**
             What each refusal said.
             */
            const refusals: string[] = [];
            for (const text of [
              'not json',
              '{}',
              '[{"branch":"fix/a"}]',
              '[{"message":"m","check":"everything"}]',
            ]) {
              try {
                entriesIn({ where: '/list.json', text, },);
              }
              catch (caught) {
                refusals.push((caught as Error).message,);
              }
            }
            expect(refusals.length,).toBe(4,);
            expect(refusals.join('\n',),).toContain('this is not JSON',);
            expect(refusals.join('\n',),).toContain('a list file holds an array of entries',);
            expect(refusals.join('\n',),).toContain('names no message',);
            expect(refusals.join('\n',),).toContain('check must be one of',);
          },
        },),
      ],
    },),
    describe({
      name: parseCommandLine.name,
      children: [
        it({
          name: 'reads each subcommand and the values it needs',
          fn: async () => {
            expect(parseCommandLine({
              args: [
                'land',
                '--branch=fix/a',
                '--message=m',
              ],
            },),).toEqual({
              command: 'land',
              branch: 'fix/a',
              message: 'm',
              bounds: {
                tries: 15,
                pauseMilliseconds: 20_000,
                commitTimeoutMilliseconds: 120_000,
                gitTimeoutMilliseconds: 900_000,
              },
            },);
            expect(parseCommandLine({
              args: [
                'check',
                '--worktree=pi-a',
                '--check=wrapper',
                '--task=//package/a/b:lint:types',
                '--main-checkout=/main',
              ],
            },).command,).toBe('check',);
            expect(parseCommandLine({ args: ['worktrees',], },).command,).toBe('worktrees',);
            expect(parseCommandLine({ args: ['remove-worktrees', '--apply',], },).command,).toBe('remove-worktrees',);
          },
        },),
        it({
          name: 'answers with its usage for --help and for no word at all',
          fn: async () => {
            expect(parseCommandLine({ args: [], },),).toEqual({
              command: 'help',
              text: USAGE,
            },);
            expect(parseCommandLine({ args: ['land', '--help',], },).command,).toBe('help',);
          },
        },),
        it({
          name: 'refuses a subcommand nobody knows, a value a subcommand needs, and an option it does not take',
          fn: async () => {
            /**
             What each refusal said.
             */
            const refusals: string[] = [];
            for (const args of [
              ['landed',],
              ['land', '--message=m',],
              ['land', '--branch=a', '--message=m', '--apply',],
              ['land', '--branch=a', '--message=m', '--commit-tries=0',],
            ]) {
              try {
                parseCommandLine({ args, },);
              }
              catch (caught) {
                refusals.push((caught as Error).message,);
              }
            }
            expect(refusals.length,).toBe(4,);
            expect(refusals.join('\n',),).toContain('there is no such subcommand',);
            expect(refusals.join('\n',),).toContain('needs --branch',);
            expect(refusals.join('\n',),).toContain('a whole number of at least 1',);
          },
        },),
      ],
    },),
  ],
},);
