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

 @returns the file text

 @example
 ```ts
 manifestOf(image);
 ```
 */
function manifestOf(manifestImage: string): string {
  return JSON.stringify({
    base: 'c'.repeat(64),
    image: manifestImage,
    limits: [],
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
            })).toBe(join(
              root.path,
              'native-passed',
            ));
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
            })).toBe(join(
              root.path,
              'native-older',
            ));
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
            })).toBe(join(
              root.path,
              'native-c-passed',
            ));
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
            })).toBeUndefined();
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
            })).toBeUndefined();
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
            })).toBeUndefined();
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
            })).toBeUndefined();
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
            })).toBeUndefined();
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
            })).toBeUndefined();
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
            })).toBeUndefined();
          },
        }),
      ],
    }),
  ],
});
