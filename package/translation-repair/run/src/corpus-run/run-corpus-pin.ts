import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  CORPUS_COMMIT_SHA,
  type CorpusPin,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CorpusPinSetting,
  readCorpusPinSetting,
} from './corpus-pin-override.ts';

//region Run corpus pin
// SPLIT OUT OF `run-config.ts` to keep that file under the line budget; the pin is its own concern.

/**
 Built-in corpus read location: the user's local clone at the benchmark
 commit. Content is read at runtime and never committed here (the clone is
 UNLICENSED).
 */
const RUN_CORPUS_PIN_FALLBACK: CorpusPin = {
  cloneDir: join(
    homedir(),
    'one-among-us',
    'data',
  ),
  commitSha: CORPUS_COMMIT_SHA,
};

/**
 Reads the run's corpus pin: the built-in clone and commit unless the
 environment overrides either half (`TRANSLATION_REPAIR_CORPUS_CLONE_DIR`,
 `TRANSLATION_REPAIR_CORPUS_COMMIT`), which fixture runs against an unmerged
 corpus pull request need, beside where each half came from, for the launch
 line `corpusPinOverrideNote` makes of it.

 CALLED BY AN ENTRY INSIDE ITS REFUSAL BOUNDARY, and handed down as a value.
 Until 2026-10-10 this was a module constant, resolved while this file
 loaded, so an unreadable override exited every command that loads this
 file at 1 under a dump of bundled source, before `reportingRefusals` ran,
 whether or not the command reads the corpus (ledger B302, B381).

 @param env - environment of the process the entry runs in

 @returns The pin beside a per-half source

 @throws StatedRefusalError when an override is written and unreadable

 @example
 ```ts
 const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
 ```
 */
export function readRunCorpusPinSetting(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): CorpusPinSetting {
  return readCorpusPinSetting({
    env,
    fallback: RUN_CORPUS_PIN_FALLBACK,
  },);
}

//endregion Run corpus pin
