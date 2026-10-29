import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { PassOutsideReads, } from './pass-outside-reads.ts';
import type { PassVisualEvidenceReader, } from './pass-visual-evidence.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Pass preparation request
// What one pass entry's preparation takes, named apart from `pass-prepare.ts`,
// whose procedure fills that module's code-line budget; `preparePassEntry`
// documents each field where it reads it.

/**
 Everything one pass entry's preparation reads.

 @example
 ```ts
 const request: PassPrepareRequest = { client, entryId, entryCacheDir, pipelineDigest, modelIds, sourceText, targetText, signal, exchangeTimeoutMs, l, outsideReads, stageSettings, };
 ```
 */
export type PassPrepareRequest = {
  readonly client: ModelClient;
  readonly entryId: string;
  readonly entryCacheDir: string;
  readonly pipelineDigest: PipelineDigest;
  readonly modelIds: readonly RosterModelId[];
  readonly sourceText: string;
  readonly targetText: string;
  readonly signal: AbortSignal;
  readonly exchangeTimeoutMs: number;
  readonly l: Logger;
  readonly readPictures?: PassVisualEvidenceReader;
  readonly beforeItem?: () => Promise<BenchSeating>;
  readonly outsideReads: PassOutsideReads;
  readonly fanOut?: FanOutMode;
  readonly stageSettings: StageSettings;
};

//endregion Pass preparation request
