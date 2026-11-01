import { DRAWN_EDITOR_SPECS_1, } from './editor-fidelity-pool-drawn-1.ts';
import { DRAWN_EDITOR_SPECS_2, } from './editor-fidelity-pool-drawn-2.ts';
import { DRAWN_EDITOR_SPECS_3, } from './editor-fidelity-pool-drawn-3.ts';
import { DRAWN_EDITOR_SPECS_4, } from './editor-fidelity-pool-drawn-4.ts';
import { DRAWN_EDITOR_SPECS_5, } from './editor-fidelity-pool-drawn-5.ts';
import { DRAWN_EDITOR_SPECS_6, } from './editor-fidelity-pool-drawn-6.ts';
import { DRAWN_EDITOR_SPECS_7, } from './editor-fidelity-pool-drawn-7.ts';
import { DRAWN_EDITOR_SPECS_8, } from './editor-fidelity-pool-drawn-8.ts';
import { DRAWN_EDITOR_SPECS_9, } from './editor-fidelity-pool-drawn-9.ts';
import { DRAWN_EDITOR_SPECS_10, } from './editor-fidelity-pool-drawn-10.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications
// Every specification the draw of seed `round-4` took, the parts in order. Written by `editor-fidelity-pool-draw`; do not
// edit by hand.

/**
 The drawn specifications of the editor fidelity pool.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS;
 ```
 */
export const DRAWN_EDITOR_SPECS: readonly FidelityReferenceSpec[] = [
  ...DRAWN_EDITOR_SPECS_1,
  ...DRAWN_EDITOR_SPECS_2,
  ...DRAWN_EDITOR_SPECS_3,
  ...DRAWN_EDITOR_SPECS_4,
  ...DRAWN_EDITOR_SPECS_5,
  ...DRAWN_EDITOR_SPECS_6,
  ...DRAWN_EDITOR_SPECS_7,
  ...DRAWN_EDITOR_SPECS_8,
  ...DRAWN_EDITOR_SPECS_9,
  ...DRAWN_EDITOR_SPECS_10,
];

//endregion Editor fidelity pool, drawn specifications
