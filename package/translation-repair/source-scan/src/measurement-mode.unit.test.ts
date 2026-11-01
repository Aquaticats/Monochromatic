/**
 Guards the mode every launch names for the re-ask of a reasoning runaway (ledger B1141): a probe, a bench, a calibration
 or a census scores the model it asks, so its runaway is re-asked of that model itself (`measured-model`), and the pass
 asks another model (`other-model`). A rival's answer recorded as the measured model's would be scored as its own, which
 is how the judge fidelity probe, the editor and producer calibrations, the translate probe and the audit sensitivity
 probe came to measure models they did not seat.

 WHAT THE SCAN READS, in package source (not tests and fixtures): every launch reads its settings with
 `readStageSettings({ env, reask, })`, which has no default, and the scan reads the value each call names. The packages
 of the measurements name `measured-model` and every other package `other-model`. Every production call of the gather
 and the windowed rounds passes the launch's mode on, as `stageSettings.reask`, and never a literal, so a stage added
 later cannot ask the run's mode of a measurement.

 OUT OF THE SCAN'S REACH: a settings object built as a literal (the type requires `reask` there), and a stage that
 receives the mode under another name.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  packageDirectoryOf,
  readPackageSource,
} from './family-source.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';
import {
  rolesNamedAtCalls,
  sourceNamesFunction,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';
import {
  keysNamedInLiteralsOfType,
  keysNamedInLiteralsUnder,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-literal.test-fixture.ts';

//region Measurement mode scan

/**
 Packages whose launches are measurements.
 */
const MEASURING_PACKAGES: ReadonlySet<string> = new Set([
  'bench',
  'coverage-census',
  'probe',
],);

/**
 Functions whose every production call passes the launch's mode on.
 */
const MODE_CARRIERS: readonly string[] = [
  'gatherStageVoices',
  'repairInvalidCandidates',
  'runWindowedRounds',
];

/**
 Functions of the picture readers whose every production call names the mode: the launch's, passed on, or the one mode
 its own package names (the pass reads pictures in `other-model`, a measurement of a reader in `measured-model`).
 */
const READER_MODE_CARRIERS: readonly string[] = [
  'readDocumentPictures',
  'readImagePair',
  'readPastRefusal',
];

/**
 Spellings of the launch's mode a call may pass on: the settings it holds, or a parameter of that name it was handed.
 */
const PASSED_ON: ReadonlySet<string> = new Set([
  'stageSettings.reask',
  'reask',
],);

await describe({
  name: 'the mode a launch names for the re-ask of a reasoning runaway',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE MEASURED-MODEL MODE AT EVERY LAUNCH OF A PROBE, A BENCH OR A CENSUS and the other-model mode '
        + 'everywhere else',
      fn: async () => {
        const launches = (await readPackageSource())
          .filter(function readsSettings(file,): boolean {
            // BY TOKENS, as `rolesNamedAtCalls` reads: a comment that names the function is no launch (ledger B1163).
            return (!file.isTest) && sourceNamesFunction({ source: file.text, name: 'readStageSettings', },);
          },);
        expect(launches.length > 0,).toBe(true,);
        expectNoFindings({
          findings: launches.flatMap(function misnamed(file,): readonly string[] {
            /**
             Mode this file's package must name.
             */
            const expected = MEASURING_PACKAGES.has(packageDirectoryOf({ path: file.path, },),)
              ? 'measured-model'
              : 'other-model';
            return rolesNamedAtCalls({ source: file.text, callee: 'readStageSettings', key: 'reask', },)
              .filter(function wrong(named,): boolean {
                return named !== expected;
              },)
              .map(function toFinding(named,): string {
                return `${file.path}: readStageSettings names ${named}, and this package must name ${expected}`;
              },);
          },),
        },);
      },
    },),

    it({
      name: 'NAMES THE OTHER-MODEL MODE NOWHERE IN A PROBE, A BENCH OR A CENSUS, so a settings object built as a literal '
        + 'cannot put a measurement on the mode of the pass',
      fn: async () => {
        expectNoFindings({
          findings: (await readPackageSource())
            .filter(function isMeasuring(file,): boolean {
              return (!file.isTest) && MEASURING_PACKAGES.has(packageDirectoryOf({ path: file.path, },),);
            },)
            .filter(function namesThePass(file,): boolean {
              return file.text.includes('\'other-model\'',);
            },)
            .map(function toFinding(file,): string {
              return `${file.path}: names other-model in a package of the measurements`;
            },),
        },);
      },
    },),

    it({
      name: 'PASSES THE LAUNCH\'S MODE ON AT EVERY PRODUCTION CALL OF THE GATHER AND THE WINDOWED ROUNDS, never a literal',
      fn: async () => {
        const files = (await readPackageSource())
          .filter(function isProduction(file,): boolean {
            return !file.isTest;
          },);
        expectNoFindings({
          findings: MODE_CARRIERS.flatMap(function carrierFindings(callee,): readonly string[] {
            return files
              .filter(function names(file,): boolean {
                return sourceNamesFunction({ source: file.text, name: callee, },);
              },)
              .flatMap(function misses(file,): readonly string[] {
                return rolesNamedAtCalls({ source: file.text, callee, key: 'reask', },)
                  .filter(function notPassedOn(named,): boolean {
                    return !PASSED_ON.has(named,);
                  },)
                  .map(function toFinding(named,): string {
                    return `${file.path}: a call of ${callee} names reask as ${named}`;
                  },);
              },);
          },),
        },);
      },
    },),

    it({
      name: 'NAMES THE MODE AT EVERY PRODUCTION CALL OF THE PICTURE READERS (ledger B1163): passed on from the launch, or the '
        + 'one mode the package of the call names, so a measurement of a reader cannot take the mode of the pass',
      fn: async () => {
        const files = (await readPackageSource())
          .filter(function isProduction(file,): boolean {
            return !file.isTest;
          },);
        expectNoFindings({
          findings: READER_MODE_CARRIERS.flatMap(function carrierFindings(callee,): readonly string[] {
            return files
              .filter(function names(file,): boolean {
                return sourceNamesFunction({ source: file.text, name: callee, },);
              },)
              .flatMap(function misses(file,): readonly string[] {
                /**
                 Mode this file's package names when it names one itself.
                 */
                const own = MEASURING_PACKAGES.has(packageDirectoryOf({ path: file.path, },),)
                  ? 'measured-model'
                  : 'other-model';
                return rolesNamedAtCalls({ source: file.text, callee, key: 'reask', },)
                  .filter(function notNamed(named,): boolean {
                    return (!PASSED_ON.has(named,)) && (named !== own);
                  },)
                  .map(function toFinding(named,): string {
                    return `${file.path}: a call of ${callee} names reask as ${named}`;
                  },);
              },);
          },),
        },);
      },
    },),

    it({
      name: 'NAMES THE MODE IN EVERY SEAT LITERAL OF A RE-ASK (ledger B1163): a `reaskSeat: { ... }` written in production '
        + 'code has a `reask` key, or spreads a seat that has one, so a seat built for a new caller cannot leave the mode out',
      fn: async () => {
        expectNoFindings({
          findings: (await readPackageSource())
            .filter(function isProduction(file,): boolean {
              return !file.isTest;
            },)
            .flatMap(function seatFindings(file,): readonly string[] {
              return [
                ...keysNamedInLiteralsUnder({ source: file.text, holder: 'reaskSeat', key: 'reask', },),
                ...keysNamedInLiteralsOfType({ source: file.text, type: 'ReaskSeat', key: 'reask', },),
              ]
                .filter(function lacksMode(named,): boolean {
                  return (named === 'none') || named.startsWith('duplicate',);
                },)
                .map(function toFinding(named,): string {
                  return `${file.path}: a reaskSeat literal names reask as ${named}`;
                },);
            },),
        },);
      },
    },),
  ],
},);

//endregion Measurement mode scan
