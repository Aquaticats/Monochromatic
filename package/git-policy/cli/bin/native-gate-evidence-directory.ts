/**
 The evidence directory of one gate run, which names its containers, and what a log has gained since it was last read.

 `bin/test-native-container.ts` prints the directory it keeps a run's evidence in, and writes each container's ID into a
 file there while that container runs. A reader that watches a gate it did not start finds that directory in the gate's
 log; a reader that starts one finds it in the line the gate printed.

 The log is read in the parts that are new, because a gate's log reaches tens of megabytes and a poller that read all of
 it every interval would spend its time on text it has already seen.
 */
import {
  open,
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import type { GateContainerRole, } from './native-gate-container-identity.ts';

/**
 The line `bin/test-native-container.ts` prints to name the evidence directory of its run.
 */
export const gateEvidenceLinePrefix = 'Native wrapper verification evidence: ';

/**
 The containers every gate run starts, in the order its evidence lists them.
 */
export const gatePeakRoles: readonly GateContainerRole[] = [
  'suite',
  'clippy',
];

/**
 The statement that a log names no evidence directory yet, which a sentinel carries because this repository models
 absence without nullish unions.
 */
export const NO_EVIDENCE_DIRECTORY: unique symbol = Symbol('gate log that names no evidence directory yet',);

/**
 The statement that a role's container has written no ID yet, which a container that has not started leaves behind.
 */
export const NO_CONTAINER_ID: unique symbol = Symbol('container that has written no ID to its evidence file yet',);

/**
 The evidence directory a gate log names.

 @param logText - the text the gate printed so far

 @returns the absolute path of the run's evidence directory, or {@link NO_EVIDENCE_DIRECTORY} while the log holds no
   such line yet

 @example
 ```ts
 evidenceDirectoryOf({ logText: 'Native wrapper verification evidence: /w/target/verification/native-HZPg8O' });
 ```
 */
export function evidenceDirectoryOf({ logText }: { readonly logText: string; },): string | typeof NO_EVIDENCE_DIRECTORY {
  for (const line of logText.split('\n',))
    if (line.startsWith(gateEvidenceLinePrefix,))
      return line.slice(gateEvidenceLinePrefix.length,)
        .trim();
  return NO_EVIDENCE_DIRECTORY;
}

/**
 What a log has gained since it was last read.
 */
export type AppendedLogText = {
  /**
   The bytes appended since the last read, as text.
   */
  readonly text: string;
  /**
   The offset this read ended at, which the next read starts from.
   */
  readonly fromByte: number;
};

/**
 Read the text a log gained since it was last read.

 @param logPath - the log

 @param fromByte - the offset the last read ended at

 @returns the appended text and the new offset, both unchanged when the log is not there or holds nothing new

 @example
 ```ts
 const appended = await appendedLogText({ logPath, fromByte: 0 });
 ```
 */
export async function appendedLogText({
  logPath,
  fromByte,
}: {
  readonly logPath: string;
  readonly fromByte: number;
},): Promise<AppendedLogText> {
  try {
    /**
     How large the log is now, which is where a read of it ends.
     */
    const {size} = (await stat(logPath,));
    if (size <= fromByte)
      return {
        text: '',
        fromByte: Math.min(
          fromByte,
          size,
        ),
      };
    /**
     The log, held open only for this read.
     */
    await using log = await open(
      logPath,
      'r',
    );
    /**
     The bytes appended since the last read.
     */
    const buffer = Buffer.alloc(size - fromByte,);
    await log.read(
      buffer,
      0,
      buffer.length,
      fromByte,
    );
    return {
      text: buffer.toString('utf8',),
      fromByte: size,
    };
  }
  catch (error) {
    // A log not written yet is no reading at all; a log this account may not read is a run this reader cannot follow.
    if (isMissing(error,))
      return {
        text: '',
        fromByte,
      };
    throw error;
  }
}

/**
 The ID one role's container wrote to its own evidence file.

 @param evidence - the run's evidence directory, or {@link NO_EVIDENCE_DIRECTORY} while the log names none

 @param role - the container whose ID file to read

 @returns the ID without the newline podman writes, or {@link NO_CONTAINER_ID} when there is no file to read yet

 @example
 ```ts
 const id = await readContainerId({ evidence, role: 'suite' });
 ```
 */
export async function readContainerId({
  evidence,
  role,
}: {
  readonly evidence: string | typeof NO_EVIDENCE_DIRECTORY;
  readonly role: GateContainerRole;
},): Promise<string | typeof NO_CONTAINER_ID> {
  if ((typeof evidence) === 'symbol')
    return NO_CONTAINER_ID;
  try {
    /**
     The ID as the file holds it.
     */
    const id = (await readFile(
      join(
        evidence,
        `${role}.cid`,
      ),
      'utf8',
    )).trim();
    return id === '' ? NO_CONTAINER_ID : id;
  }
  catch (error) {
    // A client that has not created its container yet wrote no file, which is not a failure of the run.
    if (isMissing(error,))
      return NO_CONTAINER_ID;
    throw error;
  }
}
