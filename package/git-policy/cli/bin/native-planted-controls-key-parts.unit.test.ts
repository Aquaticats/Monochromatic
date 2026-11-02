import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { campaignKeyParts } from './native-planted-controls-key-parts.ts';
import { controlsKeyOf } from './native-planted-controls-record.ts';
import { plantedControls } from './native-planted-controls.ts';

/**
 Key parts of a campaign at the default limits.

 @param overrides - parts of the call that replace the defaults

 @returns the key parts

 @example
 ```ts
 partsWith({ harness: 'harness-b' });
 ```
 */
function partsWith(overrides: {
  readonly harness?: string;
  readonly limits?: { readonly memory: string; readonly pidsLimit: number };
}): readonly string[] {
  return campaignKeyParts({
    controls: controlsKeyOf(plantedControls),
    harness: 'harness-a',
    limits: {
      memory: '2g',
      pidsLimit: 512,
    },
    ...overrides,
  });
}

await describe({
  name: campaignKeyParts.name,
  children: [
    it({
      name: 'holds the controls digest, the runner identity and the normalized limits, and changes with each',
      fn: async () => {
        const base = partsWith({});
        expect(base[0]).toBe(controlsKeyOf(plantedControls).sha256);
        expect(base[1]).toBe('harness-a');
        expect(partsWith({ harness: 'harness-b' })).not.toEqual(base);
        expect(partsWith({ limits: { memory: '4g', pidsLimit: 512 } })).not.toEqual(base);
        expect(partsWith({ limits: { memory: '2g', pidsLimit: 1_024 } })).not.toEqual(base);
        expect(partsWith({ limits: { memory: '2g', pidsLimit: 512 } })).toEqual(base);
      },
    }),
    it({
      name: 'treats the same memory in another unit as one condition',
      fn: async () => {
        expect(partsWith({ limits: { memory: '2048m', pidsLimit: 512 } })).toEqual(partsWith({}));
        expect(partsWith({ limits: { memory: '2097152k', pidsLimit: 512 } })).toEqual(partsWith({}));
      },
    }),
  ],
});
