import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { runCoverageStage, } from '@monochromatic-dev/translation-repair-archive/ts';
import {
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
  RUN_PER_CALL_TIMEOUT_MS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runCoverageProbe, } from './coverage-probe-run.ts';
import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { readRunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage probe
// Asks the coverage roster about the passages the aligners refuse to pair, and
// keeps what came back under the runs directory's `coverage-probe/`. SPENDS
// QUOTA. Wiring only: the walk is `coverage-probe-run.ts`, the rows
// `coverage-probe-row.ts`, the entry reads `coverage-probe-pair.ts`.

/**
 Digest over the built output this entry sits in, read at the start of a run.

 @returns The digest, which moves whenever anything that ran changed

 @example
 ```ts
 const digest = await readDigest();
 ```
 */
async function readDigest(): Promise<string> {
  return (await digestPipeline({ dir: import.meta.dirname, },)).digest;
}

/**
 The present instant.

 @returns ISO 8601 instant

 @example
 ```ts
 const startedAt = nowIso();
 ```
 */
function nowIso(): string {
  return new Date().toISOString();
}

// Guarded so this runs only when INVOKED, never as an import side effect: for a
// probe that spends quota, loading the library would otherwise buy model calls.
if (import.meta.main)
  await reportingRefusals({
    what: 'coverage-probe',
    argv: process.argv,
    env: process.env,
    run: function runCoverageProbeEntry({ line, },) {
      /**
       Corpus clone and commit this run reads, after any environment
       override, read inside the refusal boundary (ledger B302).
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return runCoverageProbe({
        line,
        pin,
        newClient: createProbeRunClient,
        stage: runCoverageStage,
        roster: RUN_ROSTER,
        exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
        readDigest,
        readClosure: function readClosure() {
          return readRunnerClosure({ entryPath: line.script, },);
        },
        resolveRunsDir,
        now: nowIso,
        log: tagged({ tag: 'coverage-probe', },),
        stageSettings: readStageSettings({ env: process.env, },),
      },);
    },
  },);

//endregion Coverage probe
