/**
 Tests for the two checks a versioned artifact reader needs.

 WHAT THESE PIN is the TOLERANCE BOUNDARY, which is the part of a schema
 reader that is easy to get backwards: the fields version 2 owns refuse
 anything they do not name, and the two fields it deliberately leaves open
 accept what a later pipeline writes there. A reader strict everywhere refuses
 valid artifacts as the pipeline grows; a reader tolerant everywhere accepts
 files nobody wrote.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  requireArtifactJsonRecord,
  requireArtifactJsonValue,
  requireExactKeys,
  requireKeyOf,
  requireOneOf,
  requireOpenRecord,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: requireExactKeys.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'REFUSES a key the version does not name, and says which keys it does: an unknown key is either '
              + 'a field from a generation this reader cannot read or a typo that dropped one it needs, and '
              + 'both produce a confident answer about a file nobody wrote',
          fn: async () => {
            /**
             What unknownKey raised, read for its class as well as its wording.
             */
            const refusalOfUnknownKeyInLane = caught(function unknownKeyInLane() {
              requireExactKeys({
                record: {
                  result: {},
                  delivery: [],
                  whiskers: 3,
                },
                allowed: [
                  'result',
                  'delivery',
                ],
                path: 'lanes.repair',
              },);
            },);

            expect(refusalOfUnknownKeyInLane,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownKeyInLane as Error).message,).toContain('lanes.repair.whiskers',);
            /**
             What unknownKey raised, read for its class as well as its wording.
             */
            const refusalOfUnknownKeyBeyondResult = caught(function unknownKeyBeyondResult() {
              requireExactKeys({
                record: { whiskers: 3, },
                allowed: [
                  'result',
                  'delivery',
                ],
                path: 'lanes.repair',
              },);
            },);

            expect(refusalOfUnknownKeyBeyondResult,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownKeyBeyondResult as Error).message,).toContain('no key here beyond result, delivery',);
          },
        },),
        it({
          name:
            'ACCEPTS a record naming FEWER keys than the version describes, since a field that is missing is '
              + 'reported by whatever reads it, at a path naming the field rather than the whole record',
          fn: async () => {
            requireExactKeys({
              record: { result: {}, },
              allowed: [
                'result',
                'delivery',
              ],
              path: 'lanes.repair',
            },);
            requireExactKeys({
              record: {},
              allowed: ['result',],
              path: 'lanes.repair',
            },);
          },
        },),
      ],
    },),

    describe({
      name: requireArtifactJsonValue.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'REFUSES `null` AT EVERY DEPTH, naming the path that reached it, because null is absence spelled '
              + 'as a value and the writer of this field leaves an unset key out instead',
          fn: async () => {
            /**
             What nullAtTop raised, read for its class as well as its wording.
             */
            const refusalOfNullAtTop = caught(function nullAtTop() {
              requireArtifactJsonValue({
                value: null,
                path: 'callConfig',
              },);
            },);

            expect(refusalOfNullAtTop,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNullAtTop as Error).message,).toContain('callConfig',);
            /**
             What nullInObject raised, read for its class as well as its wording.
             */
            const refusalOfNullInObject = caught(function nullInObject() {
              requireArtifactJsonValue({
                value: { roster: { critic: null, }, },
                path: 'callConfig',
              },);
            },);

            expect(refusalOfNullInObject,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNullInObject as Error).message,).toContain('callConfig.roster.critic',);
            /**
             What nullInArray raised, read for its class as well as its wording.
             */
            const refusalOfNullInArray = caught(function nullInArray() {
              requireArtifactJsonValue({
                value: { roster: [
                  'Tabby',
                  null,
                ], },
                path: 'callConfig',
              },);
            },);

            expect(refusalOfNullInArray,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNullInArray as Error).message,).toContain('callConfig.roster[1]',);
          },
        },),
        it({
          name:
            'ACCEPTS the shapes JSON carries and returns them, so a caller reads configuration rather than '
              + 'only learning it was well formed',
          fn: async () => {
            /**
             One nested configuration with every accepted shape in it.
             */
            const value = requireArtifactJsonValue({
              value: {
                retries: 2,
                quiet: false,
                roster: [
                  'Tabby',
                  'Calico',
                ],
                budgets: { slice: 4_000, },
                label: 'nap',
              },
              path: 'callConfig',
            },);
            expect(value,).toEqual({
              retries: 2,
              quiet: false,
              roster: [
                'Tabby',
                'Calico',
              ],
              budgets: { slice: 4_000, },
              label: 'nap',
            },);
          },
        },),
        it({
          name:
            'REFUSES a value JSON cannot carry at all, which reaches this guard only when it is called on a '
              + 'live object rather than on parsed text, and is worth refusing there rather than serializing',
          fn: async () => {
            /**
             What undefinedHeld raised, read for its class as well as its wording.
             */
            const refusalOfUndefinedHeld = caught(function undefinedHeld() {
              requireArtifactJsonValue({
                value: { retries: undefined, },
                path: 'callConfig',
              },);
            },);

            expect(refusalOfUndefinedHeld,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUndefinedHeld as Error).message,).toContain('callConfig.retries',);
            /**
             What functionHeld raised, read for its class as well as its wording.
             */
            const refusalOfFunctionHeld = caught(function functionHeld() {
              requireArtifactJsonValue({
                value: { retries: function two(): number {
                  return 2;
                }, },
                path: 'callConfig',
              },);
            },);

            expect(refusalOfFunctionHeld,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfFunctionHeld as Error).message,).toContain('callConfig.retries',);
          },
        },),
      ],
    },),

    describe({
      name: requireOneOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'RETURNS the member it found rather than the value it was handed, which is what lets a caller '
              + 'read a narrowed word without asserting a type over a string that merely compared equal',
          fn: async () => {
            expect(requireOneOf({
              value: 'decided',
              allowed: [
                'decided',
                'unfilled',
              ],
              path: 'outcome.kind',
            },),).toBe('decided',);
          },
        },),
        it({
          name:
            'REFUSES a word outside the list and a value that is not a word at all, listing what this '
              + 'version knows, since a vocabulary a reader cannot name is a vocabulary it cannot report on',
          fn: async () => {
            /**
             What unknownWord raised, read for its class as well as its wording.
             */
            const refusalOfUnknownWord = caught(function unknownWord() {
              requireOneOf({
                value: 'napped',
                allowed: [
                  'decided',
                  'unfilled',
                ],
                path: 'outcome.kind',
              },);
            },);

            expect(refusalOfUnknownWord,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownWord as Error).message,).toContain('decided, unfilled',);
            /**
             What notAWord raised, read for its class as well as its wording.
             */
            const refusalOfNotAWord = caught(function notAWord() {
              requireOneOf({
                value: 2,
                allowed: ['decided',],
                path: 'outcome.kind',
              },);
            },);

            expect(refusalOfNotAWord,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNotAWord as Error).message,).toContain('outcome.kind',);
          },
        },),
      ],
    },),

    describe({
      name: requireKeyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'RETURNS the key it found, so a caller reads a word narrowed by the record that holds the '
              + 'vocabulary, which the compiler keeps complete over the type the record is keyed by',
          fn: async () => {
            expect(requireKeyOf({
              value: 'unfilled',
              record: {
                decided: true,
                unfilled: true,
              },
              path: 'outcome.kind',
            },),).toBe('unfilled',);
          },
        },),
        it({
          name:
            'REFUSES a word outside the record, a name every object inherits, and a value that is not a '
              + 'word at all, naming the keys in the order the record spells them',
          fn: async () => {
            /**
             The vocabulary the refusals are read against.
             */
            const record = {
              decided: true,
              unfilled: true,
            } as const;

            /**
             What unknownWord raised, read for its class as well as its wording.
             */
            const refusalOfUnknownWord = caught(function unknownWord() {
              requireKeyOf({
                value: 'napped',
                record,
                path: 'outcome.kind',
              },);
            },);

            /**
             What inheritedName raised, read for its class as well as its wording.
             */
            const refusalOfInheritedName = caught(function inheritedName() {
              requireKeyOf({
                value: 'constructor',
                record,
                path: 'outcome.kind',
              },);
            },);

            /**
             What notAWord raised, read for its class as well as its wording.
             */
            const refusalOfNotAWord = caught(function notAWord() {
              requireKeyOf({
                value: 2,
                record,
                path: 'outcome.kind',
              },);
            },);

            expect(refusalOfUnknownWord,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusalOfUnknownWord,),).toBe(
              'ArtifactParseError: artifact parse failed at outcome.kind: expected one of decided, unfilled.',
            );
            expect(refusalOfInheritedName,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusalOfInheritedName,),).toBe(
              'ArtifactParseError: artifact parse failed at outcome.kind: expected one of decided, unfilled.',
            );
            expect(refusalOfNotAWord,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusalOfNotAWord,),).toBe(
              'ArtifactParseError: artifact parse failed at outcome.kind: expected a string.',
            );
          },
        },),
      ],
    },),

    describe({
      name: requireArtifactJsonRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ACCEPTS a record whose every value survives the value guard, and REFUSES one holding a `null` '
              + 'anywhere beneath it, so a caller reading a whole configuration gets the same answer per field '
              + 'as one reading fields singly',
          fn: async () => {
            expect(requireArtifactJsonRecord({
              value: {
                retries: 2,
                budgets: { slice: 4_000, },
              },
              path: 'callConfig',
            },),).toEqual({
              retries: 2,
              budgets: { slice: 4_000, },
            },);
            /**
             What nullBeneath raised, read for its class as well as its wording.
             */
            const refusalOfNullBeneath = caught(function nullBeneath() {
              requireArtifactJsonRecord({
                value: { budgets: { slice: null, }, },
                path: 'callConfig',
              },);
            },);

            expect(refusalOfNullBeneath,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNullBeneath as Error).message,).toContain('callConfig.budgets.slice',);
          },
        },),
        it({
          name:
            'REFUSES a value that is not a record, at the path naming it, rather than reading an array or a '
              + 'bare word as a configuration with every field missing',
          fn: async () => {
            /**
             What arrayGiven raised, read for its class as well as its wording.
             */
            const refusalOfArrayGiven = caught(function arrayGiven() {
              requireArtifactJsonRecord({
                value: [{ retries: 2, },],
                path: 'callConfig',
              },);
            },);

            expect(refusalOfArrayGiven,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfArrayGiven as Error).message,).toContain('callConfig',);
          },
        },),
      ],
    },),

    describe({
      name: requireOpenRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ACCEPTS a nested `null` where the other guard refuses one, which is the whole difference: this '
              + 'reads a lane result typed by the live pipeline, so refusing what a later pipeline may write '
              + 'there would refuse artifacts version 2 legitimately covers',
          fn: async () => {
            /**
             A raw result carrying a field version 2 never described, holding a
             null a later pipeline is free to write.
             */
            const raw = requireOpenRecord({
              value: {
                status: 'unchanged',
                confidence: null,
                slices: [{ ballot: null, },],
              },
              path: 'lanes.repair.result',
            },);
            expect(raw.confidence,).toBe(null,);
          },
        },),
        it({
          name:
            'REFUSES an array, since every call site here reads a JSON OBJECT and an array read as one comes '
              + 'back with every named field undefined, which surfaces later as a missing field rather than '
              + 'here as the shape error it is',
          fn: async () => {
            /**
             What arrayGiven raised, read for its class as well as its wording.
             */
            const refusalOfArrayGiven = caught(function arrayGiven() {
              requireOpenRecord({
                value: [],
                path: 'lanes.repair.result',
              },);
            },);

            expect(refusalOfArrayGiven,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfArrayGiven as Error).message,).toContain('lanes.repair.result',);
          },
        },),
      ],
    },),
  ],
},);
