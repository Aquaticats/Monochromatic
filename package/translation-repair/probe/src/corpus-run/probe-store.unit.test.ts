/**
 Tests for keeping what a quota-spending probe measured.

 The defect this module exists to close is not a crash: it is a measurement
 that was bought, printed, and then existed nowhere. So the cases that matter
 most are the ones about NOT LOSING a run, and the sharpest of them is the
 second run of the same probe, which a fixed filename would quietly destroy.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  persistProbeRun,
  type ProbeRun,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Complete run record every case starts from.

 SPREAD AND OVERRIDDEN rather than built by a helper taking optional fields:
 a partial-shaped parameter would reopen exactly the holes this codebase
 closes, and a case that varies one field reads perfectly well as this record
 with that field replaced.
 */
const BASE_RUN: ProbeRun = {
  startedAt: '2026-08-17T09:00:00.000Z',
  runnerClosure: {
    kind: 'read',
    entry: 'tabby-probe.mjs',
    chunks: [],
  },
  finishedAt: '2026-08-17T09:02:00.000Z',
  pipelineDigest: 'sha256-tree-v1:cafef00d',
  roster: ['hf:cat/Tabby-1', 'hf:cat/Calico-2',],
  subject: { fixtures: ['sunbeam-nap',], },
  rows: [{ verdict: 'no-defect-found', },],
};

/**
 Reads a persisted run back as a later reader holding only the file would.

 The cast is the claim under test rather than a shortcut: this module wrote
 the bytes, and whether they still parse as the record it was handed is the
 whole round trip.

 @param path - file a persist call reported writing

 @returns Run as parsed from disk

 @example
 ```ts
 const read = await readRun({ path: at, },);
 ```
 */
async function readRun({ path, }: { readonly path: string; },): Promise<ProbeRun> {
  return JSON.parse(await readFile(
    path,
    'utf8',
  ),) as ProbeRun;
}

/**
 Lists what one probe's directory holds.

 @param runsDir - throwaway runs directory

 @param probeName - subdirectory a persist call wrote into

 @returns Filenames present

 @example
 ```ts
 const kept = await keptFiles({ runsDir, probeName: 'coverage-probe', },);
 ```
 */
async function keptFiles(
  {
    runsDir,
    probeName,
  }: {
    readonly runsDir: string;
    readonly probeName: string;
  },
): Promise<readonly string[]> {
  return await readdir(join(
    runsDir,
    probeName,
  ),);
}

await describe({
  name: persistProbeRun.name,
  children: [
    it({
      name: 'writes the run under a directory named for the probe, and returns '
        + 'where it went, so a caller can say where the answers are instead of '
        + 'leaving a reader to search a runs directory for them',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        const at = await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: BASE_RUN,
        },);

        /**
         Directory the run should have landed in.
         */
        const expected = join(
          runsDir,
          'coverage-probe',
        );
        expect(at.startsWith(expected,),).toBe(true,);
        expect(at.endsWith('.json',),).toBe(true,);

        /**
         What that directory holds afterwards.
         */
        const kept = await keptFiles({
          runsDir,
          probeName: 'coverage-probe',
        },);
        expect(kept.length,).toBe(1,);
      },
    },),

    it({
      name: 'ACCEPTS a runs directory that does not carry the probe directory '
        + 'yet, since the first run of any probe meets exactly that and a store '
        + 'that refused it would lose the very run it was built to keep',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        await persistProbeRun({
          runsDir,
          probeName: 'audit-sensitivity',
          run: BASE_RUN,
        },);

        /**
         Top level of the runs directory, which had nothing in it.
         */
        const top = await readdir(runsDir,);
        expect(top,).toContain('audit-sensitivity',);
      },
    },),

    it({
      name: 'KEEPS BOTH RUNS when one probe is run twice, which is the loss this '
        + 'module exists to stop: these probes are rerun against the same '
        + 'subject on purpose to see whether a verdict is stable, so a fixed '
        + 'filename would destroy the run each rerun was bought to be compared '
        + 'against',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        const first = await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: {
            ...BASE_RUN,
            startedAt: '2026-08-17T09:00:00.000Z',
            rows: [{ verdict: 'absent', },],
          },
        },);
        const second = await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: {
            ...BASE_RUN,
            startedAt: '2026-08-17T11:30:00.000Z',
            rows: [{ verdict: 'carried', },],
          },
        },);
        expect(first,).not
          .toBe(second,);

        /**
         Both files, neither having overwritten the other.
         */
        const kept = await keptFiles({
          runsDir,
          probeName: 'coverage-probe',
        },);
        expect(kept.length,).toBe(2,);

        // Each still says what it said. A surviving file holding the other
        // run's rows would pass a count and fail a reader.
        const firstRead = await readRun({ path: first, },);
        const secondRead = await readRun({ path: second, },);
        expect(JSON.stringify(firstRead.rows,),).toContain('absent',);
        expect(JSON.stringify(secondRead.rows,),).toContain('carried',);
      },
    },),

    it({
      name: 'SEPARATES two builds that started within the same second, because '
        + 'the question these probes are rerun to answer is often whether a '
        + 'change moved a verdict, and collapsing the before and after into one '
        + 'file answers it by deleting half the evidence',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: {
            ...BASE_RUN,
            pipelineDigest: 'sha256-tree-v1:e327ca8b',
          },
        },);
        await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: {
            ...BASE_RUN,
            pipelineDigest: 'sha256-tree-v1:d8507690',
          },
        },);

        /**
         Both builds' files, at one instant.
         */
        const kept = await keptFiles({
          runsDir,
          probeName: 'coverage-probe',
        },);
        expect(kept.length,).toBe(2,);
      },
    },),

    it({
      name: 'CARRIES THE IDENTITY back out of the file, so a run found on disk '
        + 'months later answers for itself rather than needing the transcript '
        + 'this module exists to stop depending on',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        const at = await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: BASE_RUN,
        },);

        /**
         Run as a later reader holding only this file would find it.
         */
        const read = await readRun({ path: at, },);
        expect(read.pipelineDigest,).toBe('sha256-tree-v1:cafef00d',);
        expect(read.roster,).toContain('hf:cat/Tabby-1',);
        expect(read.startedAt,).toBe('2026-08-17T09:00:00.000Z',);
        expect(read.finishedAt,).toBe('2026-08-17T09:02:00.000Z',);
        expect(JSON.stringify(read.subject,),).toContain('sunbeam-nap',);
      },
    },),

    it({
      name: 'names the file with no colon in it, since an instant rendered '
        + 'straight into a name is painful to quote, copy and complete on every '
        + 'shell a reader might reach for it from',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'probe-store-', },);
        const runsDir = scratch.path;
        const at = await persistProbeRun({
          runsDir,
          probeName: 'coverage-probe',
          run: BASE_RUN,
        },);

        /**
         Filename alone, which is what a reader types.
         */
        const name = at.split('/',)
          .at(-1,) ?? at;
        expect(name.includes(':',),).toBe(false,);
        // Still sortable and still readable back to the instant it names.
        expect(name,).toContain('2026-08-17T09-00-00.000Z',);
      },
    },),
  ],
},);
