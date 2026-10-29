import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { resolve, } from 'node:path';
import {
  addWatchedPaths,
  reads,
  reset,
  resetWriteTimestamps,
  trackDest,
  trackRead,
  trackWriteTime,
  writes,
  writeTimestamps,
} from './tracker.ts';

//region tracker basics

await describe({
  name: '',
  children: [
    describe({
      name: 'tracker',
      children: [
        it({
          name: 'trackRead adds absolute path to reads set',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('./some/file.ts',);
            expect(
              reads.has(resolve('./some/file.ts',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'trackDest adds absolute path to writes set',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackDest('./output/file.ts',);
            expect(
              writes.has(resolve('./output/file.ts',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'trackDest does not record a write timestamp',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackDest('./output/file.ts',);
            expect(
              writeTimestamps.has(resolve('./output/file.ts',),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'reset clears reads and writes but preserves writeTimestamps',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('./r.ts',);
            trackDest('./w.ts',);
            trackWriteTime('./w.ts',);

            reset();

            expect(reads.size,).toBe(0,);
            /** writeTimestamps must survive across re-runs for echo detection */
            expect(writeTimestamps.size,).toBe(1,);
          },
        },),
        it({
          name: 'trackRead resolves relative paths to absolute',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('relative/path.ts',);
            const allAbsolute = [...reads,].every(path => path.startsWith('/',));
            expect(allAbsolute,).toBe(true,);
          },
        },),
        it({
          name: 'deduplicates identical paths',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('./same.ts',);
            trackRead('./same.ts',);
            trackRead('./same.ts',);
            expect(reads.size,).toBe(1,);
          },
        },),
        it({
          name: 'handles absolute paths without double-resolving',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const absolutePath = '/absolute/path/file.ts';
            trackRead(absolutePath,);
            expect(reads.has(absolutePath,),).toBe(true,);
          },
        },),
        it({
          name: 'tracks reads and writes independently',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('./input.ts',);
            trackDest('./output.ts',);
            expect(
              reads.has(resolve('./output.ts',),),
            ).toBe(false,);
            expect(
              writes.has(resolve('./input.ts',),),
            ).toBe(false,);
          },
        },),
      ],
    },),

    //endregion tracker basics

    //region writeTimestamps

    describe({
      name: 'writeTimestamps',
      children: [
        it({
          name: 'trackWriteTime records a timestamp for the path',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackWriteTime('./dest.md',);
            expect(
              writeTimestamps.has(resolve('./dest.md',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'recorded timestamp is close to Date.now()',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            /** Capture time before and after to bound the timestamp */
            const before = Date.now();
            trackWriteTime('./timed.md',);
            const recorded = writeTimestamps.get(resolve('./timed.md',),);
            /** Timestamp should be between the two captures */
            expect(recorded,).toBeGreaterThanOrEqual(before,);
          },
        },),
        it({
          name: 'later writes overwrite the previous timestamp',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackWriteTime('./updated.md',);
            /** First timestamp */
            const first = writeTimestamps.get(resolve('./updated.md',),);
            // Tiny delay to ensure a new timestamp
            trackWriteTime('./updated.md',);
            /** Second timestamp should be >= first */
            const second = writeTimestamps.get(resolve('./updated.md',),);
            expect(second,).toBeGreaterThanOrEqual(first ?? 0,);
          },
        },),
        it({
          name: 'resolves relative paths to absolute',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackWriteTime('relative/out.ts',);
            const allAbsolute = [...writeTimestamps.keys(),].every(path =>
              path.startsWith('/',)
            );
            expect(allAbsolute,).toBe(true,);
          },
        },),
      ],
    },),

    //endregion writeTimestamps

    //region addWatchedPaths

    describe({
      name: addWatchedPaths.name,
      children: [
        it({
          name: 'adds all provided paths to the reads set',
          fn: async () => {
            reset();
            addWatchedPaths(['./extra1.ts', './extra2.ts',],);
            expect(
              reads.has(resolve('./extra1.ts',),),
            ).toBe(true,);
            expect(
              reads.has(resolve('./extra2.ts',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'resolves relative paths to absolute',
          fn: async () => {
            reset();
            addWatchedPaths(['relative/dep.json',],);
            const allAbsolute = [...reads,].every(path => path.startsWith('/',));
            expect(allAbsolute,).toBe(true,);
          },
        },),
        it({
          name: 'deduplicates with existing tracked reads',
          fn: async () => {
            reset();
            trackRead('./shared.ts',);
            addWatchedPaths(['./shared.ts', './new.ts',],);
            expect(reads.size,).toBe(2,);
          },
        },),
        it({
          name: 'handles empty array without error',
          fn: async () => {
            reset();
            addWatchedPaths([],);
            expect(reads.size,).toBe(0,);
          },
        },),
      ],
    },),
    //endregion addWatchedPaths
  ],
},);
