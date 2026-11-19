/**
 The child that reads the handover a driver wrote into the descriptor it was started with, and says only what the values are
 named, so a test of the drain reads no value.

 @module
 */

import { stdout, } from 'node:process';
import {
  readTheSecretValuesHandedOver,
  SECRET_VALUES_FD,
} from '../dist/final/node/index.mjs';

/**
 What the driver handed over, by the names of the values and never by their values.
 */
const values = readTheSecretValuesHandedOver({ fd: SECRET_VALUES_FD, },);

stdout.write(
  `${values.map(function nameOf(value,) {
    return value.name;
  },)
    .join('\n',)}\n`,
);
