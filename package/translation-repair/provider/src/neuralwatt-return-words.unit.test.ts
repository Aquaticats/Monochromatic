/**
 Tests for the words the refusal that ends the run gives for NeuralWatt: what the run knows of its day, why it reads
 dry, and when its energy comes back, only the true cause of each.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  keyUnblockTextOf,
  namedDrynessOf,
  openingClauseOf,
  returnClauseOf,
  returnTimeOf,
} from '../dist/final/node/index.mjs';

/**
 The day the run decided: it starts at 19:15:08 UTC.
 */
const DECIDED_DAY = { kind: 'at', msOfDay: 69_308_000, } as const;

await describe({
  name: 'NeuralWatt return words',
  concurrency: 1,
  children: [
    describe({
      name: keyUnblockTextOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'SPELLS THE INSTANT IN UTC and states none the account gave no instant for, so the reply\'s own text '
            + 'never reaches the refusal',
          fn: async () => {
            expect([
              keyUnblockTextOf({ until: '2026-10-23T05:00:00Z', },),
              keyUnblockTextOf({ until: 'when the moon is full', },),
            ],).toEqual(['2026-10-23T05:00:00.000Z', 'a time the account did not state',],);
          },
        },),
      ],
    },),

    describe({
      name: returnTimeOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'TAKES THE TIME OF DAY FROM THE READING\'S OWN DAY START and from the decided day otherwise, and names '
            + 'none where neither gives one',
          fn: async () => {
            expect([
              returnTimeOf({
                day: { kind: 'unanchored', },
                dryness: { kind: 'spent-day', dayStart: '2026-10-22T07:30:00.000Z', },
              },),
              returnTimeOf({ day: DECIDED_DAY, },),
              returnTimeOf({ day: { kind: 'unanchored', }, },),
            ],).toEqual([{ text: '07:30:00 UTC', }, { text: '19:15:08 UTC', }, {},],);
          },
        },),
      ],
    },),

    describe({
      name: namedDrynessOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'PREFERS THE CAUSE THE CALLER NAMED, reads a bare distance as a spent day, and names none otherwise',
          fn: async () => {
            expect([
              namedDrynessOf({ neuralwattDryness: { kind: 'reserved', }, neuralwattReturnsInMs: 5, },),
              namedDrynessOf({ neuralwattReturnsInMs: 301_000, },),
              namedDrynessOf({},),
            ],).toEqual([
              { dryness: { kind: 'reserved', }, },
              { dryness: { kind: 'spent-day', dayStart: '', returnsInMs: 301_000, }, },
              {},
            ],);
          },
        },),
      ],
    },),

    describe({
      name: openingClauseOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'OPENS WITH THE TRUE CAUSE: a blocked key, energy held by calls in flight, an undecided day start, a '
            + 'spent day, and nothing for an off provider',
          fn: async () => {
            expect([
              openingClauseOf({ day: DECIDED_DAY, dryness: { kind: 'key-blocked', until: 'x', }, },),
              openingClauseOf({ day: DECIDED_DAY, dryness: { kind: 'reserved', }, },),
              openingClauseOf({ day: DECIDED_DAY, dryness: { kind: 'no-day-start', }, },),
              openingClauseOf({ day: { kind: 'unanchored', }, },),
              openingClauseOf({ day: DECIDED_DAY, },),
              openingClauseOf({ day: { kind: 'off', }, },),
            ],).toEqual([
              'NeuralWatt\'s key allowance is blocked, ',
              'NeuralWatt has no energy to spend while calls in flight hold what is left, ',
              'NeuralWatt is refused until its day start is learned, ',
              'NeuralWatt is refused until its day start is learned, ',
              'NeuralWatt has no free daily energy left, ',
              '',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: returnClauseOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'STATES THE DAY START AND THE DISTANCE for a spent day, that the account still shows the day spent '
            + 'when it names no distance, and says "after waiting" only when the call waited',
          fn: async () => {
            const spent = { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', } as const;
            expect([
              returnClauseOf({ day: DECIDED_DAY, dryness: { ...spent, returnsInMs: 301_000, }, afterWaiting: false, },),
              returnClauseOf({ day: DECIDED_DAY, dryness: spent, afterWaiting: true, },),
              returnClauseOf({ day: DECIDED_DAY, dryness: spent, afterWaiting: false, },),
            ],).toEqual([
              'NeuralWatt\'s free energy returns when its day starts again, at 19:15:08 UTC (in 5 min 1 s), the day '
                + 'start this run decided or learned; ',
              'NeuralWatt\'s day started again at 19:15:08 UTC, but its account still reads the day spent after '
                + 'waiting for it; ',
              'NeuralWatt\'s day started again at 19:15:08 UTC, but its account still reads the day spent; ',
            ],);
          },
        },),

        it({
          name: 'STATES THE KEY\'S OWN TIME, THE CALLS IN FLIGHT, AND AN UNDECIDED DAY START for the other causes, and '
            + 'that NeuralWatt is off for an off provider, none of them tied to the day start',
          fn: async () => {
            expect([
              returnClauseOf({
                day: DECIDED_DAY,
                dryness: { kind: 'key-blocked', until: '2026-10-23T05:00:00Z', },
                afterWaiting: false,
              },),
              returnClauseOf({ day: DECIDED_DAY, dryness: { kind: 'reserved', }, afterWaiting: false, },),
              returnClauseOf({ day: { kind: 'unanchored', }, afterWaiting: false, },),
              returnClauseOf({ day: { kind: 'off', }, afterWaiting: false, },),
            ],).toEqual([
              'NeuralWatt\'s key allowance stays blocked until 2026-10-23T05:00:00.000Z and is not tied to the day '
                + 'start; ',
              'NeuralWatt\'s remaining energy is held by calls in flight and comes back as they settle, not at the day '
                + 'start; ',
              'NeuralWatt\'s day start was not decided for this run, so no time of day can be stated for its return; ',
              'NeuralWatt is off for this run, so it returns no energy; ',
            ],);
          },
        },),
      ],
    },),
  ],
},);
