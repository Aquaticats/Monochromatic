import {
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { evidenceJson } from './campaign-files.ts';
import {
  campaignProtocolFile,
  startedMarkerProtocolVersion,
} from './constants.ts';

/**
 Record, in a campaign directory this runner creates, that the started marker is written before every container start, so
 that a later reader may take the marker's absence beside an opened attempt as proof that no container started.

 A directory that already holds anything was begun by an earlier runner (or by an earlier invocation of this one before it
 recorded this), whose runs may have started a container without leaving the marker, so it is never given the record: the
 record is a statement about the whole history of the directory, not about the runner that happens to resume it.

 @param evidence - the campaign's evidence directory, whose right to run is already held

 @example
 ```ts
 await recordStartedMarkerProtocol('/e/native-mutation-0123456789ab');
 ```
 */
export async function recordStartedMarkerProtocol(evidence: string): Promise<void> {
  if ((await readdir(evidence)).length > 0)
    return;
  await writeFile(
    join(
      evidence,
      campaignProtocolFile,
    ),
    evidenceJson({ version: startedMarkerProtocolVersion }),
  );
}
