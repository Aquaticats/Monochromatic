/**
 Reading a create's hourly prices once, and saying what happened when they
 cannot be had.

 The project decided on 2026-11-28 that a create takes the cheapest size that
 fits, and that where the price read cannot be had it "falls back to smallest
 numbers and says so"
 (`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`, "The size
 taken among those that fit"). This module is that decision's other half: the
 read, and the record of which order the create will use.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  readPriceIndex,
  subsidiaryOf,
} from './catalog-prices.ts';
import { requireCredentials, } from './config.ts';
import type { PriceChoice, } from './flavors.ts';

//region Reading


/**
 Reads the public catalogue's hourly prices once for a create, falling back to
 the numbers alone and saying so when the read cannot be had.
 
 @param rl - logger for the read's outcome
 
 @returns the prices, or the record that they could not be read
 
 @example
 ```ts
 const priceChoice = await readPriceChoice({ rl });
 ```
 */
export async function readPriceChoice({ rl, }: { readonly rl: Logger; },): Promise<PriceChoice> {
  try {
    /**
     Prices the public catalogue publishes for this endpoint's subsidiary.
     */
    const index = await readPriceIndex({
      subsidiary: subsidiaryOf({ endpointRoot: requireCredentials()
        .endpointRoot, },),
    },);
    /**
     How many sizes the answer prices in full.
     */
    const pricedSizes = index.prices
      .size;
    rl.info(
      `read the whole hourly price of ${String(pricedSizes,)} ${(pricedSizes === 1) ? 'size' : 'sizes'} in ${index.currency} from OVHcloud's public catalogue`,
    );
    return {
      kind: 'priced',
      currency: index.currency,
      prices: index.prices,
    };
  }
  catch (error: unknown) {
    rl.info(
      `hourly prices could not be read (${caughtValueText(error,)}), so the smallest numbers win`,
    );
    return { kind: 'unpriced', };
  }
}


//endregion Reading
