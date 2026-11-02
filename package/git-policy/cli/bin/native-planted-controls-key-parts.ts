/**
 What a native campaign's kept outcomes depend on, for the key of its evidence directory.

 The outcomes were scored under one set of planted controls, one runner and judge, and one memory and process limit, and
 an obligation to prove the controls compares those limits. A resume under any of them changed is therefore another
 directory that keeps nothing, so no retained result can be paid by a proof made for something else.
 */
import { memoryBytes } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';

import type { ExecutionLimits } from './native-gate-proof.ts';
import type { ControlsKey } from './native-planted-controls-record.ts';

/**
 The conditions as key parts.

 @param controls - digest and names of the planted controls

 @param harness - identity of the code that runs and judges them

 @param limits - the memory and process limit of the campaign's containers; the memory is normalized to bytes, so `2g` and
 `2048m` are one condition

 @returns the parts, in a fixed order

 @example
 ```ts
 campaignKeyParts({ controls, harness, limits: { memory: '2g', pidsLimit: 512 } });
 ```
 */
export function campaignKeyParts({
  controls,
  harness,
  limits,
}: {
  readonly controls: ControlsKey;
  readonly harness: string;
  readonly limits: ExecutionLimits;
}): readonly string[] {
  return [
    controls.sha256,
    harness,
    String(memoryBytes(limits.memory)),
    String(limits.pidsLimit),
  ];
}
