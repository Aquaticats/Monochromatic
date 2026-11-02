import {
  mkdir,
  mkdtempDisposable,
  readFile,
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
  attemptDirectories,
  campaignKey,
  campaignKeyLength,
  createAttemptDirectory,
  readScoredList,
  readScoredNames,
  recordCampaignIdentity,
  scoredCount,
  seedScoredNames,
} from './campaign-ledger.ts';
import {
  composeReport,
  cumulativeStatus,
} from './campaign-report.ts';
import {
  missedExitStatus,
  timeoutExitStatus,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Image ID the tests use: 64 hexadecimal digits.
 */
const imageId = 'a'.repeat(64);

/**
 Scored names with every outcome present.
 */
const mixed = {
  caught: ['c1'],
  missed: ['m1'],
  timeout: ['t1'],
  unviable: ['u1'],
};

/**
 Make a disposable directory under the agent scratch area, never the system temporary directory.

 @returns the directory, removed when disposed

 @example
 ```ts
 await using directory = await scratch();
 ```
 */
async function scratch(): Promise<AsyncDisposable & { readonly path: string }> {
  /**
   Parent of every scratch directory.
   */
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  return mkdtempDisposable(join(
    parent,
    'cargo-mutants-ledger-',
  ));
}

/**
 Write the list files of one attempt as cargo-mutants leaves them.

 @param attempt - attempt directory

 @param files - file name to text

 @example
 ```ts
 await writeAttempt({ attempt, files: { 'caught.txt': 'a\n' } });
 ```
 */
async function writeAttempt({
  attempt,
  files,
}: {
  readonly attempt: string;
  readonly files: Readonly<Record<string, string>>;
}): Promise<void> {
  await mkdir(
    join(
      attempt,
      'mutants.out',
    ),
    { recursive: true },
  );
  await Promise.all(Object.entries(files)
    .map(function writeOne([name, text]) {
      return writeFile(
        join(
          attempt,
          'mutants.out',
          name,
        ),
        text,
      );
    }));
}

await describe({
  name: '',
  children: [
    describe({
      name: campaignKey.name,
      children: [
        it({
          name: 'is the same for the same image and command, and differs when either differs',
          fn: async () => {
            /**
             Key of the base case.
             */
            const base = campaignKey({
              baseImage: imageId,
              command: [
                'cargo',
                'mutants'
              ],
            });
            expect(base).toHaveLength(campaignKeyLength);
            expect(campaignKey({
              baseImage: imageId,
              command: [
                'cargo',
                'mutants'
              ],
            })).toBe(base);
            expect(campaignKey({
              baseImage: 'b'.repeat(imageId.length),
              command: [
                'cargo',
                'mutants'
              ],
            })).not.toBe(base);
            expect(campaignKey({
              baseImage: imageId,
              command: [
                'cargo',
                'mutants',
                '--file',
                'src/a.rs'
              ],
            })).not.toBe(base);
          },
        }),
        it({
          name: 'keeps the key of a product without further conditions, and differs when a condition is added, changed or reordered',
          fn: async () => {
            const command = ['cargo', 'mutants'];
            const base = campaignKey({ baseImage: imageId, command });
            // No condition and an empty list are the same campaign as before, so existing evidence is still found.
            expect(campaignKey({ baseImage: imageId, command, keyParts: [] })).toBe(base);
            const withControls = campaignKey({ baseImage: imageId, command, keyParts: ['controls-a', 'harness-a'] });
            expect(withControls).not.toBe(base);
            expect(campaignKey({ baseImage: imageId, command, keyParts: ['controls-a', 'harness-a'] })).toBe(withControls);
            expect(campaignKey({ baseImage: imageId, command, keyParts: ['controls-b', 'harness-a'] })).not.toBe(withControls);
            expect(campaignKey({ baseImage: imageId, command, keyParts: ['controls-a', 'harness-b'] })).not.toBe(withControls);
            expect(campaignKey({ baseImage: imageId, command, keyParts: ['harness-a', 'controls-a'] })).not.toBe(withControls);
            expect(campaignKey({ baseImage: imageId, command, keyParts: ['controls-aharness-a'] })).not.toBe(withControls);
          },
        }),
        it({
          name: 'does not let arguments that concatenate alike share a key',
          fn: async () => {
            expect(campaignKey({
              baseImage: imageId,
              command: [
                'a',
                'bc'
              ],
            })).not.toBe(campaignKey({
              baseImage: imageId,
              command: [
                'ab',
                'c'
              ],
            }));
          },
        }),
      ],
    }),
    describe({
      name: readScoredList.name,
      children: [
        it({
          name: 'drops a final line that has no newline, because a kill tore it, and reads a missing file as empty',
          fn: async () => {
            await using directory = await scratch();
            /**
             List file with a torn last line.
             */
            const file = join(
              directory.path,
              'caught.txt',
            );
            await writeFile(
              file,
              'one\ntwo\nthr',
            );
            expect(await readScoredList(file)).toEqual([
              'one',
              'two'
            ]);
            expect(
              await readScoredList(join(
                directory.path,
                'absent.txt',
              )),
            ).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: createAttemptDirectory.name,
      children: [
        it({
          name: 'numbers attempts from one, in order, and lists them in that order',
          fn: async () => {
            await using directory = await scratch();
            /**
             First and second attempts.
             */
            const first = await createAttemptDirectory(directory.path);
            const second = await createAttemptDirectory(directory.path);
            expect([
              first.number,
              second.number
            ]).toEqual([
              1,
              2
            ]);
            expect(await attemptDirectories(directory.path)).toEqual([
              first.path,
              second.path
            ]);
          },
        }),
      ],
    }),
    describe({
      name: seedScoredNames.name,
      children: [
        it({
          name: 'seeds caught, missed, timed-out and unviable names alike, so that none is scored again',
          fn: async () => {
            await using directory = await scratch();
            await seedScoredNames({
              attemptDirectory: directory.path,
              names: mixed,
            });
            expect((await readFile(
              join(
                directory.path,
                'mutants.out',
                'previously_caught.txt',
              ),
              'utf8',
            )).split('\n')
              .filter(function isName(line) {
                return line !== '';
              })
              .toSorted()).toEqual(Object.values(mixed)
                .flat()
                .toSorted());
          },
        }),
      ],
    }),
    describe({
      name: recordCampaignIdentity.name,
      children: [
        it({
          name: 'accepts the same image and command again and rejects a directory that belongs to another',
          fn: async () => {
            await using directory = await scratch();
            /**
             Identity of the first campaign.
             */
            const identity = {
              evidence: directory.path,
              baseImage: imageId,
              command: ['cargo'],
            };
            await recordCampaignIdentity(identity);
            await recordCampaignIdentity(identity);
            await expect(recordCampaignIdentity({
              ...identity,
              command: ['cargo', '--other'],
            })).rejects.toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: composeReport.name,
      children: [
        it({
          name: 'unions the lists of every attempt without repeats, counts them in outcomes.json and survives a torn outcomes file',
          fn: async () => {
            await using directory = await scratch();
            await writeAttempt({
              attempt: join(
                directory.path,
                'attempt-001',
              ),
              files: {
                'caught.txt': 'a\nb\n',
                'missed.txt': 'm\n',
                'outcomes.json': '{"outcomes": [',
              },
            });
            await writeAttempt({
              attempt: join(
                directory.path,
                'attempt-002',
              ),
              files: {
                'caught.txt': 'c\n',
                'timeout.txt': 't\n',
                'outcomes.json': JSON.stringify({
                  outcomes: [{ summary: 'caught' }],
                  success: 0,
                }),
              },
            });
            /**
             Names the composition reports.
             */
            const names = await composeReport({ evidence: directory.path });
            expect(names).toEqual({
              caught: [
                'a',
                'b',
                'c'
              ],
              missed: ['m'],
              timeout: ['t'],
              unviable: [],
            });
            expect(await readScoredNames(directory.path)).toEqual(names);
            expect(
              JSON.parse(await readFile(
                join(
                  directory.path,
                  'mutants.out',
                  'outcomes.json',
                ),
                'utf8',
              )),
            ).toMatchObject({
              total_mutants: scoredCount(names),
              caught: names.caught.length,
              missed: names.missed.length,
              timeout: names.timeout.length,
              unviable: 0,
            });
          },
        }),
      ],
    }),
    describe({
      name: cumulativeStatus.name,
      children: [
        it({
          name: 'follows the whole report for a finished attempt: timeouts first, then missed mutants, else success',
          fn: async () => {
            expect(cumulativeStatus({
              attemptStatus: 0,
              names: mixed,
            })).toBe(timeoutExitStatus);
            expect(cumulativeStatus({
              attemptStatus: 0,
              names: {
                ...mixed,
                timeout: [],
              },
            })).toBe(missedExitStatus);
            expect(cumulativeStatus({
              attemptStatus: missedExitStatus,
              names: {
                caught: ['c'],
                missed: [],
                timeout: [],
                unviable: [],
              },
            })).toBe(0);
          },
        }),
        it({
          name: 'keeps the attempt status when the attempt did not finish, such as a failed baseline or a kill',
          fn: async () => {
            expect(cumulativeStatus({
              attemptStatus: 4,
              names: mixed,
            })).toBe(4);
            expect(cumulativeStatus({
              attemptStatus: 137,
              names: mixed,
            })).toBe(137);
          },
        }),
      ],
    }),
  ],
});
