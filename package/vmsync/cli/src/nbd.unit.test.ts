import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ddTransferArgs, } from '../dist/final/node/testing.mjs';

/**
 Bytes per qcow2 cluster, the alignment `qemu-img map` reports regions at.
 */
const QCOW2_CLUSTER_SIZE = 65_536;

/**
 1 MiB, the block size the transfer still hands dd for I/O granularity.
 */
const MIB = 1_048_576;

/**
 Bytes per sector, the only alignment a region transfer accepts.
 */
const SECTOR_SIZE = 512;

/**
 Builds a `qemu-img map` region descriptor for a transfer test, with every
 block-map field set to the value a changed data region carries; only `start`
 and `length` reach the dd argument list.
 
 @param start - byte offset of the region in the image
 
 @param length - length of the region in bytes
 
 @returns region descriptor shaped like a block-map entry
 */
function regionAt(
  {
    start,
    length,
  }: {
    readonly start: number;
    readonly length: number;
  },
) {
  return {
    start,
    length,
    depth: 0,
    present: true,
    zero: false,
    data: true,
    offset: 0,
    filename: 'overlay.qcow2',
  };
}

/**
 Reads the value of one `key=value` operand out of a dd argument list.
 
 @param args - dd arguments produced by the transfer builder
 
 @param key - operand name to look up
 
 @returns the operand value
 
 @throws Error when the operand is absent
 
 @example
 ```ts
 operandValue(['skip=512',], 'skip'); // "512"
 ```
 */
function operandValue(
  args: readonly string[],
  key: string,
): string {
  /**
   Full operand prefix, so `skip` does not match `iflag=skip_bytes`.
   */
  const prefix = `${key}=`;
  /**
   Matching operand, absent when the builder omits it.
   */
  const found = args
    .find(function hasKey(arg,) {
      return arg.startsWith(prefix,);
    },);
  if (found === undefined) {
    throw new Error(
      `dd arguments carry no ${key} operand: ${args.join(' ',)}`,
    );
  }
  return found.slice(prefix.length,);
}

await describe({
  name: ddTransferArgs.name,
  children: [
    //region Exact operands for a cluster-aligned region

    it({
      name: 'copies a 64 KiB-aligned region with byte-offset operands',
      fn: async () => {
        const START = QCOW2_CLUSTER_SIZE;
        const LENGTH = QCOW2_CLUSTER_SIZE * 8;
        const args = ddTransferArgs({
          sourceDevice: '/dev/nbd0',
          targetDevice: '/dev/nbd1',
          region: regionAt({
            start: START,
            length: LENGTH,
          },),
        },);

        expect(args,).toEqual([
          'if=/dev/nbd0',
          'of=/dev/nbd1',
          `bs=${String(MIB,)}`,
          'iflag=skip_bytes,count_bytes',
          'oflag=seek_bytes',
          `skip=${String(START,)}`,
          `seek=${String(START,)}`,
          `count=${String(LENGTH,)}`,
          'conv=notrunc',
          'status=none',
        ],);
      },
    },),

    //endregion Exact operands for a cluster-aligned region

    //region No fractional or over-reading operands

    it({
      name: 'keeps every numeric operand an integer below one dd block',
      fn: async () => {
        const START = QCOW2_CLUSTER_SIZE;
        const LENGTH = SECTOR_SIZE * 3;
        const args = ddTransferArgs({
          sourceDevice: '/dev/nbd0',
          targetDevice: '/dev/nbd1',
          region: regionAt({
            start: START,
            length: LENGTH,
          },),
        },);

        for (const key of ['skip', 'seek', 'count',]) {
          const value = operandValue(
            args,
            key,
          );
          /**
           Operand read back as a JS number, for the integrality check.
           */
          const numeric = Number(value,);
          expect(value.includes('.',),).toBe(false,);
          expect(Number.isInteger(numeric,),).toBe(true,);
        }
      },
    },),

    it({
      name: 'never counts past the region end when the length misses a block boundary',
      fn: async () => {
        const LENGTH = SECTOR_SIZE * 3;
        const args = ddTransferArgs({
          sourceDevice: '/dev/nbd0',
          targetDevice: '/dev/nbd1',
          region: regionAt({
            start: MIB,
            length: LENGTH,
          },),
        },);

        expect(operandValue(
          args,
          'count',
        ),)
          .toBe(String(LENGTH,),);
      },
    },),

    //endregion No fractional or over-reading operands

    //region Rejected regions

    it({
      name: 'rejects a start that is not sector-aligned',
      fn: async () => {
        expect(() => {
          ddTransferArgs({
            sourceDevice: '/dev/nbd0',
            targetDevice: '/dev/nbd1',
            region: regionAt({
              start: SECTOR_SIZE + 1,
              length: SECTOR_SIZE,
            },),
          },);
        },)
          .toThrow('sector-aligned',);
      },
    },),

    it({
      name: 'rejects a length that is not sector-aligned',
      fn: async () => {
        expect(() => {
          ddTransferArgs({
            sourceDevice: '/dev/nbd0',
            targetDevice: '/dev/nbd1',
            region: regionAt({
              start: 0,
              length: SECTOR_SIZE + 1,
            },),
          },);
        },)
          .toThrow('sector-aligned',);
      },
    },),

    //endregion Rejected regions
  ],
},);
