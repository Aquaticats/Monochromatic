/**
 Unit tests for resolving the monorepo's pnpm on the host.
 
 Every fixture installs with the pnpm this repository runs, so the matrix asks
 the host's `pnpm` for its version and for its store. Under mise that `pnpm` is
 this repository's `pnpm-shadow`, which prints its own notice after a
 top-level `--version`, so the version is read from the version line and not
 from the whole output.
 
 @module
 */

import {
  stat,
} from 'node:fs/promises';
import {
  isAbsolute,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pnpmSpecFromVersionOutput,
  resolvePnpmSpec,
  resolveStoreDir,
} from './host-pnpm.ts';

//region Fixtures

/**
 Version line pnpm itself prints, measured on 2026-11-10.
 */
const PNPM_VERSION = '12.18.0';

/**
 Notice this repository's `pnpm-shadow` prints after pnpm's own version,
 quoted from the wrapper's `version_line` in `package/pnpm-shadow/cli/src/shadow_notice.rs`.
 */
const SHADOW_NOTICE = 'pnpm-shadow 0.1.3-alpha.1, shadowing pnpm at /opt/pnpm';

/**
 Spec corepack and `packageManager` accept for {@link PNPM_VERSION}.
 */
const EXPECTED_SPEC = `pnpm@${PNPM_VERSION}`;

//endregion Fixtures

await describe({
  name: 'host pnpm',
  children: [
    //region Version line

    describe({
      name: pnpmSpecFromVersionOutput.name,
      children: [
        it({
          name: 'takes the version line and drops the shadow notice after it',
          fn: async () => {
            /**
             `pnpm --version` output as the wrapper on PATH prints it.
             */
            const output = `${PNPM_VERSION}\n${SHADOW_NOTICE}\n`;
            expect(pnpmSpecFromVersionOutput(output,),).toBe(EXPECTED_SPEC,);
          },
        },),

        it({
          name: 'reads a single-line version output',
          fn: async () => {
            expect(pnpmSpecFromVersionOutput(`${PNPM_VERSION}\n`,),).toBe(EXPECTED_SPEC,);
          },
        },),

        it({
          name: 'refuses a first line that is not a single token',
          fn: async () => {
            /**
             Rejection from output whose first line is the wrapper's notice,
             which names the notice rather than a version.
             */
            let caught: unknown;
            try {
              pnpmSpecFromVersionOutput(`${SHADOW_NOTICE}\n${PNPM_VERSION}\n`,);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain(
              'could not resolve the monorepo pnpm version',
            );
          },
        },),

        it({
          name: 'rejects output that carries no version line',
          fn: async () => {
            /**
             Rejection from a version output with the notice only.
             */
            let caught: unknown;
            try {
              pnpmSpecFromVersionOutput(`${SHADOW_NOTICE}\n`,);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain(
              'could not resolve the monorepo pnpm version',
            );
          },
        },),

        it({
          name: 'rejects empty output',
          fn: async () => {
            /**
             Rejection from an empty version output.
             */
            let caught: unknown;
            try {
              pnpmSpecFromVersionOutput('',);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),

    //endregion Version line

    //region Against the pnpm on PATH

    it({
      name: 'resolvePnpmSpec reads the pnpm the host actually runs',
      fn: async () => {
        /**
         Spec resolved from the pnpm on PATH, the wrapper included.
         */
        const spec = await resolvePnpmSpec();
        expect(spec.startsWith('pnpm@',),).toBe(true,);
        expect(
          pnpmSpecFromVersionOutput(`${spec.slice('pnpm@'.length,)}\n`,),
        ).toBe(spec,);
      },
    },),

    it({
      name: 'resolveStoreDir reads an absolute directory the host can mount',
      fn: async () => {
        /**
         Store directory reported by the pnpm on PATH.
         */
        const storeDir = await resolveStoreDir();
        expect(isAbsolute(storeDir,),).toBe(true,);
        /**
         What the host has at the reported store path.
         */
        const storeStats = await stat(storeDir,);
        expect(storeStats.isDirectory(),).toBe(true,);
      },
    },),

    //endregion Against the pnpm on PATH
  ],
},);
