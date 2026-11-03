import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import { evidenceJson } from './campaign-files.ts';
import {
  attemptProtocolFile,
  startedMarkerProtocolVersion,
} from './constants.ts';

/**
 Record, in an attempt directory this runner has just made, that the started marker is written before every container
 start, so that a later reader may take the marker's absence beside this attempt as proof that no container started.

 The record belongs to the attempt, not to the campaign: a directory that a marker-aware runner began can be resumed by an
 older runner, whose attempts the record must not vouch for.

 @param attemptDirectory - the new attempt's directory

 @example
 ```ts
 await recordStartedMarkerProtocol('/e/native-mutation-0123456789ab/attempt-001');
 ```
 */
export async function recordStartedMarkerProtocol(attemptDirectory: string): Promise<void> {
  await writeFile(
    join(
      attemptDirectory,
      attemptProtocolFile,
    ),
    evidenceJson({ version: startedMarkerProtocolVersion }),
  );
}
