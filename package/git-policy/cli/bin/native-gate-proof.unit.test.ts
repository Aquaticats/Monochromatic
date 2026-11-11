import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  diskWeightArguments,
  lowestDiskWeight,
  memoryBytes,
  runLimitArguments,
} from './container-limits.ts';
import { gateContainerLimits, } from './native-gate-container-limits.ts';
import {
  gatePassedRecord,
  passingGateEvidence,
} from './native-gate-proof.ts';

/**
 Image ID of the gate image a campaign is asked about.
 */
const image = 'a'.repeat(64);

/**
 Image ID of some other gate image.
 */
const otherImage = 'b'.repeat(64);

/**
 Container arguments of a gate run, as the gate's manifest records them.
 */
const gateLimits = [
  '--rm',
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: gateContainerLimits,
    withPidsLimit: true,
  }),
];

/**
 Limits of a campaign that is as large as the gate run, derived from the gate's own so a bound the gate gains changes
 this fixture with it.
 */
const campaignLimits = {
  memory: gateContainerLimits.memory,
  pidsLimit: gateContainerLimits.pidsLimit,
};

/**
 Scratch parent of the test directories; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Write one gate evidence directory the way the gate leaves it.

 @param root - evidence root

 @param name - directory name

 @param files - file name to file text; an absent entry leaves that file out

 @example
 ```ts
 await writeGateDirectory({ root, name: 'native-abc123', files: { 'manifest.json': '{}' } });
 ```
 */
async function writeGateDirectory({
  root,
  name,
  files,
}: {
  readonly root: string;
  readonly name: string;
  readonly files: Readonly<Record<string, string>>;
}): Promise<void> {
  await mkdir(join(
    root,
    name,
  ));
  await Promise.all(Object.entries(files)
    .map(function writeEntry([file, text]) {
      return writeFile(
        join(
          root,
          name,
          file,
        ),
        text,
      );
    }));
}

/**
 Text of a gate manifest that names an image.

 @param manifestImage - image ID the manifest records

 @param limits - the container arguments the manifest records, as the gate writes them

 @returns the file text

 @example
 ```ts
 manifestOf(image);
 ```
 */
function manifestOf(manifestImage: string, limits: readonly string[] = gateLimits): string {
  return JSON.stringify({
    base: 'c'.repeat(64),
    image: manifestImage,
    limits,
  });
}

/**
 Create the evidence root a test fills.

 @returns the disposable root; declare it with `await using`

 @example
 ```ts
 await using root = await evidenceRoot();
 ```
 */
async function evidenceRoot(): Promise<AsyncDisposable & { readonly path: string }> {
  await mkdir(
    scratchParent,
    { recursive: true },
  );
  return await mkdtempDisposable(join(
    scratchParent,
    'gate-proof-',
  ));
}

await describe({
  name: 'the proof that a gate passed on an image',
  children: [
    describe({
      name: gatePassedRecord.name,
      children: [
        it({
          name: 'records the image beside the passed tests and Clippy, so the record stands alone',
          fn: async () => {
            expect(gatePassedRecord(image)).toEqual({
              tests: true,
              clippy: true,
              image,
            });
          },
        }),
      ],
    }),
    describe({
      name: passingGateEvidence.name,
      children: [
        it({
          name: 'names the gate directory whose manifest image is the asked image and whose run passed',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-passed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([join(
              root.path,
              'native-passed',
            )]);
          },
        }),
        it({
          name: 'accepts a passed record written before the image was added to it, by the manifest of the same directory',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-older',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify({
                  tests: true,
                  clippy: true,
                }),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([join(
              root.path,
              'native-older',
            )]);
          },
        }),
        it({
          name: 'finds the passing directory among failed runs and runs of other images',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-a-failed',
              files: { 'manifest.json': manifestOf(image) },
            });
            await writeGateDirectory({
              root: root.path,
              name: 'native-b-other',
              files: {
                'manifest.json': manifestOf(otherImage),
                'passed.json': JSON.stringify(gatePassedRecord(otherImage)),
              },
            });
            await writeGateDirectory({
              root: root.path,
              name: 'native-c-passed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([join(
              root.path,
              'native-c-passed',
            )]);
          },
        }),
        it({
          name: 'finds nothing when the gate stopped before it passed (no passed record)',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-failed',
              files: { 'manifest.json': manifestOf(image) },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing when only another image passed',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-other',
              files: {
                'manifest.json': manifestOf(otherImage),
                'passed.json': JSON.stringify(gatePassedRecord(otherImage)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing when the passed record names another image than the manifest does',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-mixed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(otherImage)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing when the record does not say that the tests and Clippy passed',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-tests-false',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify({
                  tests: false,
                  clippy: true,
                }),
              },
            });
            await writeGateDirectory({
              root: root.path,
              name: 'native-clippy-missing',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify({ tests: true }),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing, without failing, when a record is torn or is not an object',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-torn',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': '{"tests": tr',
              },
            });
            await writeGateDirectory({
              root: root.path,
              name: 'native-list',
              files: {
                'manifest.json': '[]',
                'passed.json': '[]',
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing under an evidence root that does not exist',
          fn: async () => {
            await using root = await evidenceRoot();
            expect(await passingGateEvidence({
              evidenceRoot: join(
                root.path,
                'absent',
              ),
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'accepts a campaign whose memory and process limit are at least the gate\'s, in any unit',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-passed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            /**
             Campaigns with the gate's memory in another unit, and with more of both.
             */
            const campaigns = [
              {
                memory: '2048m',
                pidsLimit: gateContainerLimits.pidsLimit,
              },
              {
                memory: '4g',
                pidsLimit: gateContainerLimits.pidsLimit
                  * 2,
              },
            ];
            // The first campaign's memory is the gate's in another unit, and that is the whole point of the case.
            expect(memoryBytes('2048m')).toBe(memoryBytes(gateContainerLimits.memory));
            /**
             What each campaign finds.
             */
            const found = await Promise.all(campaigns.map(function find(limits) {
              return passingGateEvidence({
                evidenceRoot: root.path,
                image,
                limits,
              });
            }));
            expect(found).toEqual(campaigns.map(function expected() {
              return [join(
                root.path,
                'native-passed',
              )];
            }));
          },
        }),
        it({
          name: 'gives no proof for argument lists whose effective limits cannot be established, whatever their form',
          fn: async () => {
            await using root = await evidenceRoot();
            // Each list is something a runner never writes: a repeated bound, a split or attached short option, a
            // cluster with a value, an option whose value looks like a bound, a trailing bare override, an extra option.
            const unreadable = [
              ['--rm', '--memory=1g', '--memory', '4g', '--pids-limit=512'],
              ['--rm', '--memory=1g', '-m4g', '--pids-limit=512'],
              ['--rm', '--memory=1g', '-im4g', '--pids-limit=512'],
              ['--rm', '--memory=4g', '--label', '-m1g', '--pids-limit=512'],
              ['--rm', '--memory=4g', '--pids-limit=512', '--memory'],
              ['--rm', '--memory=2g', '--pids-limit=128', '--pids-limit=512'],
              ['--rm', '--memory=2g', '--pids-limit=512', '--memory-swap=9g'],
            ];
            await Promise.all(unreadable.map(function writeRun(
              limits,
              index
            ) {
              return writeGateDirectory({
                root: root.path,
                name: `native-unreadable-${String(index)}`,
                files: {
                  'manifest.json': manifestOf(image, limits),
                  'passed.json': JSON.stringify(gatePassedRecord(image)),
                },
              });
            }));
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: { memory: '64g', pidsLimit: 100_000 },
            })).toEqual([]);
          },
        }),
        it({
          name: 'reads the disk-weight arguments the runners write, and records made before them, but not another weight',
          fn: async () => {
            /**
             Arguments of a gate that ran before the runners wrote a disk weight.
             */
            const beforeWeight = gateLimits.filter(function isNotWeight(argument) {
              return !diskWeightArguments().includes(argument);
            });
            expect(beforeWeight.length).toBeLessThan(gateLimits.length);
            /**
             Arguments with a disk weight above the lowest, which the runners never write.
             */
            const otherWeight = [
              ...beforeWeight,
              `--blkio-weight=${String(lowestDiskWeight + 1)}`,
            ];
            /**
             Gate arguments, and whether the gate that recorded them proves the image.
             */
            const cases = [
              { limits: gateLimits, proves: true },
              { limits: beforeWeight, proves: true },
              { limits: otherWeight, proves: false },
            ];
            await Promise.all(cases.map(async function checkCase({ limits, proves }) {
              await using root = await evidenceRoot();
              await writeGateDirectory({
                root: root.path,
                name: 'native-run',
                files: {
                  'manifest.json': manifestOf(image, limits),
                  'passed.json': JSON.stringify(gatePassedRecord(image)),
                },
              });
              expect(await passingGateEvidence({
                evidenceRoot: root.path,
                image,
                limits: campaignLimits,
              })).toEqual(proves ? [join(root.path, 'native-run')] : []);
            }));
          },
        }),
        it({
          name: 'gives no proof when the gate ran with unlimited or zero processes or memory',
          fn: async () => {
            await using root = await evidenceRoot();
            const runs = [
              ['--memory=2g', '--pids-limit=-1'],
              ['--memory=2g', '--pids-limit=0'],
              ['--memory=0', '--pids-limit=512'],
            ];
            await Promise.all(runs.map(function writeRun(
              limits,
              index
            ) {
              return writeGateDirectory({
                root: root.path,
                name: `native-run-${String(index)}`,
                files: {
                  'manifest.json': manifestOf(image, limits),
                  'passed.json': JSON.stringify(gatePassedRecord(image)),
                },
              });
            }));
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: {
                memory: '1g',
                pidsLimit: 64,
              },
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing for a campaign with a lower process limit than the gate, which is how the fork failures came back',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-passed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: {
                memory: '2g',
                pidsLimit: 128,
              },
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing for a campaign with less memory than the gate',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-passed',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: {
                memory: '1500m',
                pidsLimit: 512,
              },
            })).toEqual([]);
          },
        }),
        it({
          name: 'finds nothing when the manifest does not record a memory and a process limit it can compare',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-no-limits',
              files: {
                'manifest.json': manifestOf(image, []),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            await writeGateDirectory({
              root: root.path,
              name: 'native-odd-limits',
              files: {
                'manifest.json': manifestOf(image, ['--memory=lots', '--pids-limit=many']),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
        it({
          name: 'ignores mutation campaign directories, which share the gate prefix but are not gate runs',
          fn: async () => {
            await using root = await evidenceRoot();
            await writeGateDirectory({
              root: root.path,
              name: 'native-mutation-0123456789ab',
              files: {
                'manifest.json': manifestOf(image),
                'passed.json': JSON.stringify(gatePassedRecord(image)),
              },
            });
            expect(await passingGateEvidence({
              evidenceRoot: root.path,
              image,
              limits: campaignLimits,
            })).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
