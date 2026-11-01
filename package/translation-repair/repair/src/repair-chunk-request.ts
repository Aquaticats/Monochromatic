import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { AttestedDetail, } from '@monochromatic-dev/translation-repair-reference/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Chunk repair request
// What one chunk pair's repair takes, named apart from `repair-chunk.ts`,
// whose procedure fills that module's code-line budget; `repairChunk`
// documents each field where it reads it.

/**
 Everything one chunk pair's repair reads.

 @example
 ```ts
 const request: RepairChunkRequest = { client, sliceIndex: 0, sourceText, targetText, lineStructured: false, models, declaredNames: [], signal, perCallTimeoutMs, l, stageSettings, };
 ```
 */
export type RepairChunkRequest = {
  readonly client: ModelClient;
  readonly sliceIndex: number;
  readonly sourceText: string;
  readonly targetText: string;
  readonly lineStructured: boolean;
  readonly models: RepairModels;
  readonly reseat?: () => Promise<RepairSliceSeating>;
  readonly identityContext?: string;
  readonly referenceContext?: string;
  readonly attestedDetails?: readonly AttestedDetail[];
  readonly declaredNames: readonly string[];
  readonly declaredSubjectForms?: readonly string[];
  readonly neighbouringIncumbentText?: string;
  readonly neighbouringSourceText?: string;
  readonly documentSourceText?: string;
  readonly signal: AbortSignal;
  readonly perCallTimeoutMs: number;
  readonly l: Logger;
  readonly stageSettings: StageSettings;
};

//endregion Chunk repair request
