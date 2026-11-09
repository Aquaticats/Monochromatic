/**
 Tests for the render-node choice of a headless run: reading what `--list-gpus` prints, preferring the device the
 compositor itself would use, refusing silently falling back to the software renderer, and the display a run gets.

 @module
 */

import {
  chmod,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  chooseHeadlessGpu,
  parseGpuListing,
  probeHeadlessGpu,
  resolveDisplay,
} from './gpu-choice.ts';

/**
 What `monochromatic-nested-wayland-session --list-gpus` 0.6.1 printed on the development host on 2026-11-07, with the
 device lines unchanged: two render nodes, the software renderer, and the device `auto` uses.
 */
const REAL_LISTING = [
  '/dev/dri/renderD128 pci=1002:7480 driver=amdgpu renderer=AMD Radeon RX 7600 (radeonsi, navi33, ACO, DRM 3.64, 7.2.15-ogc1.1.fc45.x86_64)',
  '/dev/dri/renderD129 pci=1002:164e driver=amdgpu renderer=AMD Ryzen 9 7950X 16-Core Processor (radeonsi, raphael_mendocino, ACO, DRM 3.64, 7.2.15-ogc1.1.fc45.x86_64)',
  'software renderer=llvmpipe (LLVM 23.1.2, 256 bits)',
  'auto uses: /dev/dri/renderD129 (AMD Ryzen 9 7950X 16-Core Processor (radeonsi, raphael_mendocino, ACO, DRM 3.64, 7.2.15-ogc1.1.fc45.x86_64))',
].join('\n',);

/**
 A listing in which the first render node cannot be opened, and `auto` found nothing.
 */
const FIRST_NODE_UNUSABLE = [
  '/dev/dri/renderD128 pci=1002:7480 driver=amdgpu unusable: eglInitialize failed',
  '/dev/dri/renderD129 pci=1002:164e driver=amdgpu renderer=AMD Ryzen 9 7950X',
  'software renderer=llvmpipe (LLVM 23.1.2, 256 bits)',
  'auto uses: nothing, see the lines above',
].join('\n',);

/**
 A listing with no render node at all: only the software renderer, which `auto` would use.
 */
const SOFTWARE_ONLY = [
  'software renderer=llvmpipe (LLVM 23.1.2, 256 bits)',
  'auto uses: software (llvmpipe (LLVM 23.1.2, 256 bits))',
].join('\n',);

/**
 Writes a compositor stand-in that prints a listing for `--list-gpus` and a version for anything else.
 
 @param directory - scratch directory
 
 @param listing - what the stand-in prints for `--list-gpus`
 
 @param exitCode - status of the stand-in for `--list-gpus`
 
 @returns the stand-in's path
 */
async function fakeCompositor(
  { directory, listing, exitCode, }: { readonly directory: string; readonly exitCode: number; readonly listing: string; },
): Promise<string> {
  const path = join(directory, 'fake-compositor',);
  await writeFile(
    path,
    '#!/usr/bin/env node\n'
      + `if (process.argv.includes('--list-gpus')) { console.log(${JSON.stringify(listing,)}); process.exit(${exitCode}); }\n`
      + 'console.log("nested 0.6.1");\n',
  );
  await chmod(path, 0o700,);
  return path;
}

await describe({
  name: parseGpuListing.name,
  children: [
    it({
      name: 'reads render nodes, the software line and the device auto uses from the real listing',
      fn: async ({ expect, }) => {
        const listing = parseGpuListing(REAL_LISTING,);
        expect(listing.usableNodes,).toEqual(['/dev/dri/renderD128', '/dev/dri/renderD129',],);
        expect(listing.autoUses,).toBe('/dev/dri/renderD129',);
        expect(listing.hasSoftware,).toBe(true,);
      },
    },),
    it({
      name: 'leaves out a node marked unusable and reads auto uses nothing as no device',
      fn: async ({ expect, }) => {
        const listing = parseGpuListing(FIRST_NODE_UNUSABLE,);
        expect(listing.usableNodes,).toEqual(['/dev/dri/renderD129',],);
        expect(listing.autoUses,).toBe('',);
      },
    },),
    it({
      name: 'reads an empty or unrelated text as no device',
      fn: async ({ expect, }) => {
        expect(parseGpuListing('',).usableNodes,).toEqual([],);
        expect(parseGpuListing('devices: no EGL devices',).usableNodes,).toEqual([],);
        expect(parseGpuListing('devices: no EGL devices',).hasSoftware,).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: chooseHeadlessGpu.name,
  children: [
    it({
      name: 'chooses the device the compositor itself would use, not a fixed node',
      fn: async ({ expect, }) => {
        expect(chooseHeadlessGpu({ output: REAL_LISTING, compositorPath: '/c', },),).toBe('/dev/dri/renderD129',);
        expect(
          chooseHeadlessGpu({
            output: REAL_LISTING.replace('auto uses: /dev/dri/renderD129', 'auto uses: /dev/dri/renderD128',),
            compositorPath: '/c',
          },),
        ).toBe('/dev/dri/renderD128',);
      },
    },),
    it({
      name: 'falls back to the first usable render node when auto names none',
      fn: async ({ expect, }) => {
        expect(chooseHeadlessGpu({ output: FIRST_NODE_UNUSABLE, compositorPath: '/c', },),).toBe('/dev/dri/renderD129',);
      },
    },),
    it({
      name: 'refuses a host with only the software renderer, naming the compositor, the listing and the ways out',
      fn: async ({ expect, }) => {
        let message = '';
        try {
          chooseHeadlessGpu({ output: SOFTWARE_ONLY, compositorPath: '/the/compositor', },);
        }
        catch (error: unknown) {
          message = caughtValueText(error,);
        }
        expect(message,).toContain('/the/compositor --list-gpus',);
        expect(message,).toContain('software renderer=llvmpipe',);
        expect(message,).toContain('E2E_GPU',);
        expect(message,).toContain('--visible',);
      },
    },),
    it({
      name: 'refuses a listing with no device, and says what it read',
      fn: async ({ expect, }) => {
        let message = '';
        try {
          chooseHeadlessGpu({ output: '', compositorPath: '/c', },);
        }
        catch (error: unknown) {
          message = caughtValueText(error,);
        }
        expect(message,).toContain('printed nothing',);
      },
    },),
  ],
},);

await describe({
  name: probeHeadlessGpu.name,
  children: [
    it({
      name: 'asks the compositor for its listing and returns the chosen node',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-gpu-',),);
        const directory = scratchDirectory.path;
        const compositorPath = await fakeCompositor({ directory, listing: REAL_LISTING, exitCode: 0, },);
        const gpu = await probeHeadlessGpu({ compositorPath, },);
        expect(gpu,).toBe('/dev/dri/renderD129',);
      },
    },),
    it({
      name: 'fails with the status and the text when the listing command fails',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-gpu-',),);
        const directory = scratchDirectory.path;
        const compositorPath = await fakeCompositor({ directory, listing: 'boom text', exitCode: 3, },);
        let message = '';
        try {
          await probeHeadlessGpu({ compositorPath, },);
        }
        catch (error: unknown) {
          message = caughtValueText(error,);
        }
        expect(message,).toContain('status 3',);
        expect(message,).toContain(`${compositorPath} --list-gpus`,);
      },
    },),
  ],
},);

await describe({
  name: resolveDisplay.name,
  children: [
    it({
      name: 'gives a headless display on the probed node when no visible window was asked for',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-gpu-',),);
        const directory = scratchDirectory.path;
        const compositorPath = await fakeCompositor({ directory, listing: REAL_LISTING, exitCode: 0, },);
        const display = await resolveDisplay({ compositorPath, visible: false, gpu: '', },);
        expect(display,).toEqual({ kind: 'headless', gpu: '/dev/dri/renderD129', },);
      },
    },),
    it({
      name: 'never turns a failed probe into a visible window',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-gpu-',),);
        const directory = scratchDirectory.path;
        const compositorPath = await fakeCompositor({ directory, listing: SOFTWARE_ONLY, exitCode: 0, },);
        let outcome: unknown = 'not reached';
        try {
          outcome = await resolveDisplay({ compositorPath, visible: false, gpu: '', },);
        }
        catch (error: unknown) {
          outcome = error;
        }
        expect(outcome,).toBeInstanceOf(Error,);
      },
    },),
    it({
      name: 'uses the node a person named without probing, still headless',
      fn: async ({ expect, }) => {
        const display = await resolveDisplay({ compositorPath: '/no/such/compositor', visible: false, gpu: 'software', },);
        expect(display,).toEqual({ kind: 'headless', gpu: 'software', },);
      },
    },),
    it({
      name: 'gives a visible window only when one was asked for, and probes nothing then',
      fn: async ({ expect, }) => {
        const display = await resolveDisplay({ compositorPath: '/no/such/compositor', visible: true, gpu: '', },);
        expect(display,).toEqual({ kind: 'visible', },);
      },
    },),
  ],
},);
