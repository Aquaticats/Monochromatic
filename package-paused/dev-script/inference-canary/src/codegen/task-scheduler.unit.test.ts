import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { extractAtDigits, } from './task-scheduler.ts';

await describe({
  name: extractAtDigits.name,
  children: [
    it({
      name: 'extracts the digit run after the at-sign',
      fn: async () => {
        expect(extractAtDigits('DONE A @100',),).toBe('100',);
      },
    },),
    it({
      name: 'returns empty when there is no at-sign',
      fn: async () => {
        expect(extractAtDigits('TOTAL 150',),).toBe('',);
      },
    },),
    it({
      name: 'returns empty when the at-sign is not followed by a digit',
      fn: async () => {
        expect(extractAtDigits('@abc',),).toBe('',);
      },
    },),
    it({
      name: 'returns empty when the at-sign is the last character',
      fn: async () => {
        expect(extractAtDigits('elapsed@',),).toBe('',);
      },
    },),
    it({
      name: 'stops the run at the first non-digit',
      fn: async () => {
        expect(extractAtDigits('@12.5',),).toBe('12',);
      },
    },),
    it({
      name: 'uses only the first at-sign and stops at the next',
      fn: async () => {
        expect(extractAtDigits('@1@2',),).toBe('1',);
      },
    },),
    it({
      name: 'stops the run at trailing text',
      fn: async () => {
        expect(extractAtDigits('DONE A @100 extra',),).toBe('100',);
      },
    },),
    it({
      name: 'collects a long digit run in a single linear pass',
      fn: async () => {
        const runLength = 100_000;
        const digits = '9'.repeat(runLength,);
        expect(extractAtDigits(`@${digits}x`,),).toBe(digits,);
      },
    },),
  ],
},);
