/** Exercise the shared `test:unit` template against disposable packages. @module */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rootConfig, } from './build-and-test-fixture-config.ts';
import { runTestUnitFixture, } from './test-unit-fixture.ts';

/**
 A package path the root configuration does not list as exempt.
 */
const UNLISTED_PACKAGE = 'package/fixture/demo';

/**
 The package path the root configuration lists in `unit_test_exempt_packages`.

 The exemption cases below run against this path rather than a fixture-only entry, so the repository's own list is the
 data under test: removing the entry makes them fail instead of leaving a stale exemption silently unexercised.
 */
const LISTED_PACKAGE = 'package/pi-plugin/auto-mode';

/**
 One unit test file proves a passing run executed something, and nothing else is needed to distinguish the two states.
 */
const ONE_TEST_FILE = 'sample.unit.test.ts';

await describe({
  name: 'shared test:unit template',
  concurrency: 1,
  timeout: 60_000,
  children: [
    it({
      name: 'root configuration lists the exempt fixture package',
      fn: async (): Promise<void> => {
        /** The exemption cases read this list, so its entry for the fixture path is asserted directly. */
        expect(rootConfig,).toContain(`'${LISTED_PACKAGE}'`,);
      },
    },),
    it({
      name: 'fails a package whose tree holds no unit test file',
      fn: async (): Promise<void> => {
        /** A green run over zero files is the defect the template must refuse. */
        const result = await runTestUnitFixture({ packagePath: UNLISTED_PACKAGE, unitTestFiles: [], },);
        expect(result.exitCode,).toBe(1,);
        expect(result.output,).toContain('no *.unit.test.ts file under',);
        expect(result.output,).toContain(UNLISTED_PACKAGE,);
        expect(result.output,).toContain('unit_test_exempt_packages',);
        expect(result.events,).toEqual([],);
      },
    },),
    it({
      name: 'fails the root aggregate when the whole repository holds no unit test file',
      fn: async (): Promise<void> => {
        /** Same discovery path, and its diagnostic names the monorepo root rather than a package. */
        const result = await runTestUnitFixture({ packagePath: '', unitTestFiles: [], },);
        expect(result.exitCode,).toBe(1,);
        expect(result.output,).toContain('no *.unit.test.ts file under the monorepo root',);
        expect(result.events,).toEqual([],);
      },
    },),
    it({
      name: 'passes a package whose unit test file runs',
      fn: async (): Promise<void> => {
        /** Positive control: the same fixture reports success and the file's own trace when a file exists. */
        const result = await runTestUnitFixture({
          packagePath: UNLISTED_PACKAGE,
          unitTestFiles: [ONE_TEST_FILE,],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.events,).toEqual([ONE_TEST_FILE,],);
      },
    },),
    it({
      name: 'passes a listed package with no unit test file and names its reason',
      fn: async (): Promise<void> => {
        /** An exemption is a loud skip, never a silent one. */
        const result = await runTestUnitFixture({ packagePath: LISTED_PACKAGE, unitTestFiles: [], },);
        expect(result.exitCode,).toBe(0,);
        expect(result.output,).toContain('is exempt in',);
        expect(result.output,).toContain(LISTED_PACKAGE,);
        expect(result.events,).toEqual([],);
      },
    },),
    it({
      name: 'fails a listed package that has gained a unit test file',
      fn: async (): Promise<void> => {
        /** An exemption cannot outlive the gap it excused. */
        const result = await runTestUnitFixture({
          packagePath: LISTED_PACKAGE,
          unitTestFiles: [ONE_TEST_FILE,],
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.output,).toContain('unit_test_exempt_packages',);
        expect(result.output,).toContain('remove the entry',);
        expect(result.events,).toEqual([],);
      },
    },),
  ],
},);
