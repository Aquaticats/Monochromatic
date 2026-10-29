import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  buildRunReport,
  chunk,
  composeReshard,
  composeShards,
  effectiveTimeoutMs,
  formatTerminalSummary,
  parseCliOptions,
  sanitizeShardTag,
  stemsRelated,
  type Mutant,
  type RunOutcome,
} from '../../dist/final/node/index.mjs';

/**
 Minimal mutant fixture factory for shard and report tests.
 
 @param options - Distinguishing id and file.
 
 @returns Mutant with fixed span coordinates.
 
 @example
 ```ts
 fixtureMutant({ id: 'm1', file: 'src/a.ts' });
 ```
 */
function fixtureMutant(options: {
  readonly id: string;
  readonly file: string;
},): Mutant {
  return {
    id: options.id,
    file: options.file,
    start: 0,
    end: 1,
    line: 1,
    column: 0,
    operator: 'arithmetic',
    original: '+',
    replacement: '-',
    description: 'swapped + with -',
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: parseCliOptions.name,
      children: [
        it({
          name: 'parses package, toggles, resources, and positional sources',
          fn: async () => {
            /**
             Parsed options for a fully-specified invocation.
             */
            const options = parseCliOptions([
              '--package',
              'package/module/fs-path',
              '--full-suite',
              '--dry-run',
              '--shard-size',
              '8',
              '--containers',
              '3',
              '--memory',
              '3g',
              '--timeout-factor',
              '5',
              'src/trim.ts',
            ],);
            expect(options.packagePath,).toBe('package/module/fs-path',);
            expect(options.fullSuite,).toBe(true,);
            expect(options.dryRun,).toBe(true,);
            expect(options.shardSize,).toBe(8,);
            expect(options.containers,).toBe(3,);
            expect(options.resources.memory,).toBe('3g',);
            expect(options.timeoutFactor,).toBe(5,);
            expect(options.sourceFiles,).toEqual(['src/trim.ts',],);
          },
        },),
        it({
          name: 'rejects missing package and unknown flags',
          fails: true,
          fn: async () => {
            parseCliOptions(['--bogus',],);
          },
        },),
      ],
    },),
    describe({
      name: chunk.name,
      children: [
        it({
          name: 'chunks preserving order and rejects zero size',
          fn: async () => {
            expect(chunk({
              items: [
                1,
                2,
                3,
              ],
              size: 2,
            },),).toEqual([
              [
                1,
                2,
              ],
              [3,],
            ],);

            let caught: unknown;
            try {
              chunk({
                items: [],
                size: 0,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
    describe({
      name: composeShards.name,
      children: [
        it({
          name: 'never mixes files in one shard and chunks big files',
          fn: async () => {
            /**
             Manifests for two files, one exceeding the shard size.
             */
            const manifests = composeShards({
              groups: [
                {
                  file: 'src/a.ts',
                  mutants: [
                    fixtureMutant({
                      id: 'a1',
                      file: 'src/a.ts',
                    },),
                    fixtureMutant({
                      id: 'a2',
                      file: 'src/a.ts',
                    },),
                    fixtureMutant({
                      id: 'a3',
                      file: 'src/a.ts',
                    },),
                  ],
                  tests: ['src/a.unit.test.ts',],
                },
                {
                  file: 'src/b.ts',
                  mutants: [fixtureMutant({
                    id: 'b1',
                    file: 'src/b.ts',
                  },),],
                  tests: ['src/b.unit.test.ts',],
                },
              ],
              shardSize: 2,
              timeoutFloorMs: 5_000,
              timeoutFactor: 3,
              packagePath: 'package/module/fs-path',
            },);
            expect(manifests,).toHaveLength(3,);
            expect(manifests.map(function ids(manifest,): number {
              return manifest.mutants
                .length;
            },),).toEqual([
              2,
              1,
              1,
            ],);
            expect(new Set(manifests.map(function tags(manifest,): string {
              return manifest.shardId;
            },),).size,).toBe(3,);
          },
        },),
      ],
    },),
    describe({
      name: composeReshard.name,
      children: [
        it({
          name: 'regroups ids by file through the lookup',
          fn: async () => {
            /**
             Reshard manifests for two ids of one file at size 1.
             */
            const manifests = composeReshard({
              ids: [
                'a1',
                'a2',
              ],
              size: 1,
              entryOf: function lookup(id,) {
                return {
                  mutant: fixtureMutant({
                    id,
                    file: 'src/a.ts',
                  },),
                  tests: ['src/a.unit.test.ts',],
                };
              },
              timeoutFloorMs: 5_000,
              timeoutFactor: 3,
              packagePath: 'package/module/fs-path',
            },);
            expect(manifests,).toHaveLength(2,);
            expect(manifests.every(function singleton(manifest,): boolean {
              return manifest.mutants
                .length
                === 1;
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: sanitizeShardTag.name,
      children: [
        it({
          name: 'replaces slashes and disallowed characters',
          fn: async () => {
            expect(sanitizeShardTag('src/io/glob.ts',),).toBe('src__io__glob.ts',);
            expect(sanitizeShardTag('a b/c',),).toBe('a_b__c',);
          },
        },),
      ],
    },),
    describe({
      name: effectiveTimeoutMs.name,
      children: [
        it({
          name: 'floors small baselines and scales large ones',
          fn: async () => {
            expect(effectiveTimeoutMs({
              floorMs: 5_000,
              factor: 3,
              baselineMs: 900,
            },),).toBe(5_000,);
            expect(effectiveTimeoutMs({
              floorMs: 5_000,
              factor: 3,
              baselineMs: 4_000,
            },),).toBe(12_000,);
          },
        },),
      ],
    },),
    describe({
      name: stemsRelated.name,
      children: [
        it({
          name: 'matches exact and dot-sidecar stems, not hyphen siblings',
          fn: async () => {
            expect(stemsRelated({
              sourceStem: 'foo',
              testName: 'foo.unit.test.ts',
            },),).toBe(true,);
            expect(stemsRelated({
              sourceStem: 'foo',
              testName: 'foo.regression.unit.test.ts',
            },),).toBe(true,);
            expect(stemsRelated({
              sourceStem: 'foo',
              testName: 'foo-bar.unit.test.ts',
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: buildRunReport.name,
      children: [
        it({
          name: 'totals statuses, counts suppressions, and formats findings',
          fn: async () => {
            /**
             Outcome with one of each interesting status.
             */
            const outcome: RunOutcome = {
              results: [
                {
                  mutant: fixtureMutant({
                    id: 'k1',
                    file: 'src/a.ts',
                  },),
                  status: 'killed',
                  position: 2,
                  rerunCount: 0,
                  confirmed: false,
                  detail: '',
                },
                {
                  mutant: fixtureMutant({
                    id: 's1',
                    file: 'src/a.ts',
                  },),
                  status: 'survived',
                  position: 1,
                  rerunCount: 1,
                  confirmed: true,
                  detail: '',
                },
              ],
              ignored: [{
                ...fixtureMutant({
                  id: 'i1',
                  file: 'src/a.ts',
                },),
                reason: 'known noise',
              },],
              infraErrors: [],
              shardCount: 4,
            };
            /**
             Report generated from the fixture outcome.
             */
            const report = buildRunReport({
              outcome,
              packagePath: 'package/module/fs-path',
            },);
            expect(report.totals,).toEqual({
              killed: 1,
              survived: 1,
              timeout: 0,
              compileError: 0,
              runtimeError: 0,
              ignored: 1,
            },);

            /**
             Terminal rendering of the fixture report.
             */
            const summary = formatTerminalSummary(report,);
            expect(summary,).toContain('Survived: src/a.ts:1:0 arithmetic',);
            expect(summary,).toContain('Killed: 1',);
            expect(summary,).toContain('Shards: 4',);
            expect(summary,).not.toContain('unconfirmed',);
          },
        },),
      ],
    },),
  ],
},);
