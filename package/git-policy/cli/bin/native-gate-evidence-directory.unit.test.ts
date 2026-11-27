import { mkdir, mkdtempDisposable, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NO_CONTAINER_ID,
  NO_EVIDENCE_DIRECTORY,
  appendedLogText,
  evidenceDirectoryOf,
  readContainerId,
} from './native-gate-evidence-directory.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 An ID in the shape podman writes to a container's ID file.
 */
const containerId = '3f2a9c1e'.repeat(8,);

/**
 The evidence line a gate prints, for a run whose evidence directory is the one given.

 @param evidence - the run's evidence directory

 @returns the line, with the newline the gate ends it with
 */
function evidenceLine(evidence: string,): string {
  return `Native wrapper verification evidence: ${evidence}\n`;
}

await describe({
  name: '',
  children: [
    describe({
      name: evidenceDirectoryOf.name,
      children: [
        it({
          name: 'reads the evidence directory the gate printed, which every container\'s ID file is written to',
          fn: async () => {
            expect(evidenceDirectoryOf({
              logText: [
                'The image build is bounded at 8g, its own measured bound.',
                'Native wrapper verification evidence: /w/target/verification/native-HZPg8O',
                'gpc-gate-native-HZPg8O-suite runs as a container of this run',
              ].join('\n',),
            }),).toBe('/w/target/verification/native-HZPg8O',);
          },
        }),
        it({
          name: 'answers nothing while a log holds no such line yet, which is a gate still building its image',
          fn: async () => {
            expect(evidenceDirectoryOf({ logText: 'The image build is bounded at 8g.', },),).toBe(NO_EVIDENCE_DIRECTORY,);
          },
        }),
      ],
    }),
    describe({
      name: appendedLogText.name,
      children: [
        it({
          name: 'answers only what a log gained since the last read, and where the next read starts',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-evidence-log-',
            ),);
            /**
             The log a gate is writing.
             */
            const logPath = join(
              tree.path,
              'gate.log',
            );
            await writeFile(
              logPath,
              'the image build starts\n',
            );
            /**
             The first read, which takes the log as it stands.
             */
            const first = await appendedLogText({
              logPath,
              fromByte: 0,
            },);
            expect(first.text,).toBe('the image build starts\n',);
            expect(first.fromByte,).toBeGreaterThan(0,);
            await writeFile(
              logPath,
              `the image build starts\n${evidenceLine('/w/target/verification/native-HZPg8O',)}`,
            );
            /**
             The second read, which takes only what the write added.
             */
            const second = await appendedLogText({
              logPath,
              fromByte: first.fromByte,
            },);
            expect(second.text,).toBe(evidenceLine('/w/target/verification/native-HZPg8O',),);
          },
          timeout: 20_000,
        }),
        it({
          name: 'answers an empty read for a log that is not there yet, rather than failing the run it follows',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-evidence-no-log-',
            ),);
            expect(await appendedLogText({
              logPath: join(
                tree.path,
                'not-written.log',
              ),
              fromByte: 0,
            },),).toEqual({
              text: '',
              fromByte: 0,
            },);
          },
          timeout: 20_000,
        }),
      ],
    }),
    describe({
      name: readContainerId.name,
      children: [
        it({
          name: 'reads the ID a container\'s client wrote to its own evidence file',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-evidence-id-',
            ),);
            /**
             The run's evidence directory.
             */
            const evidence = join(
              tree.path,
              'native-HZPg8O',
            );
            await mkdir(
              evidence,
              { recursive: true, },
            );
            await writeFile(
              join(evidence, 'suite.cid',),
              `${containerId}\n`,
            );
            expect(await readContainerId({
              evidence,
              role: 'suite',
            },),).toBe(containerId,);
          },
          timeout: 20_000,
        }),
        it({
          name: 'answers that no ID was written when a container has not started, and when no evidence directory is named',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-evidence-no-id-',
            ),);
            expect(await readContainerId({
              evidence: tree.path,
              role: 'clippy',
            },),).toBe(NO_CONTAINER_ID,);
            expect(await readContainerId({
              evidence: NO_EVIDENCE_DIRECTORY,
              role: 'suite',
            },),).toBe(NO_CONTAINER_ID,);
          },
          timeout: 20_000,
        }),
      ],
    }),
  ],
});
