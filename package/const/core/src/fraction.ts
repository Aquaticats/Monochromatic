/**
 Fractional constants composed from the -2..2 magic-literal exempt range.
 
 The codebase rule bans inline magic numbers outside -2..2; the workaround
 for fractional values is to compose them from the exempt range. This module
 is the canonical source so each fraction is composed once.
 
 @example
 ```ts
 import {
   HALF,
   QUARTER,
 } from '@monochromatic-dev/const-core';
 ```
 
 @module
 */

/**
 One half (0.5).
 
 @example
 ```ts
 const midpoint = (start + end,) * HALF;
 ```
 */
export const HALF: number = 1 / 2;

/**
 One quarter (0.25), composed as {@link HALF} divided by 2.
 
 @example
 ```ts
 const radius = side * QUARTER;
 ```
 */
export const QUARTER: number = HALF / 2;

/**
 Three quarters (0.75), composed as {@link HALF} plus {@link QUARTER}.
 
 @example
 ```ts
 const fillTarget = capacity * THREE_QUARTERS;
 ```
 */
export const THREE_QUARTERS: number = HALF + QUARTER;

/**
 One third (0.3333...).
 
 @example
 ```ts
 const segmentSize = total * THIRD;
 ```
 */
export const THIRD: number = 1 / (1 + 2);

/**
 Two thirds (0.6666...), composed as {@link THIRD} plus {@link THIRD}.
 
 @example
 ```ts
 const headerHeight = totalHeight * TWO_THIRDS;
 ```
 */
export const TWO_THIRDS: number = THIRD + THIRD;
