/**
 Tests for the Charm Hyper balance reader.

 THE FINITE CHECK IS THE ONE THAT MATTERS. Every other case here refuses a
 body that is plainly wrong, but a non-finite balance would compare against
 every threshold as though the budget were unlimited, which is the single
 outcome this reader exists to prevent: a run that keeps buying because it
 believes it can.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CreditsShapeError,
  parseHyperCredits,
} from '../dist/final/node/index.mjs';

await describe({
  name: parseHyperCredits.name,
  children: [
    it({
      name: 'READS the balance out of the body this provider was measured to send',
      fn: async () => {
        expect(parseHyperCredits({ bodyText: '{"balance": 249}', },),).toEqual({ balance: 249, },);
      },
    },),

    it({
      name: 'READS a spent-out balance as zero rather than as an absence, since zero is the whole '
        + 'signal a router needs',
      fn: async () => {
        expect(parseHyperCredits({ bodyText: '{"balance": 0}', },),).toEqual({ balance: 0, },);
      },
    },),

    it({
      name: 'READS a fractional balance, which per-million-token pricing produces',
      fn: async () => {
        expect(parseHyperCredits({ bodyText: '{"balance": 12.5}', },),).toEqual({ balance: 12.5, },);
      },
    },),

    it({
      name: 'IGNORES fields beside the balance, so a provider adding one does not break the read',
      fn: async () => {
        expect(parseHyperCredits({ bodyText: '{"balance": 7, "plan": "cat"}', },),)
          .toEqual({ balance: 7, },);
      },
    },),

    it({
      name: 'REFUSES a body that is not JSON, which is what a gateway error page is',
      fn: async () => {
        expect(() => {
          parseHyperCredits({ bodyText: '<html>the bookshop is closed</html>', },);
        },).toThrow(CreditsShapeError,);
      },
    },),

    it({
      name: 'REFUSES valid JSON that is not an object',
      fn: async () => {
        expect(() => {
          parseHyperCredits({ bodyText: '249', },);
        },).toThrow(CreditsShapeError,);
      },
    },),

    it({
      name: 'REFUSES a missing balance rather than reading it as nothing left, which would fail a '
        + 'run over a field the provider renamed',
      fn: async () => {
        expect(() => {
          parseHyperCredits({ bodyText: '{"credits": 249}', },);
        },).toThrow(CreditsShapeError,);
      },
    },),

    it({
      name: 'REFUSES a balance sent as a string, which a rewriting gateway can produce and which '
        + 'would compare wrongly against every threshold',
      fn: async () => {
        expect(() => {
          parseHyperCredits({ bodyText: '{"balance": "249"}', },);
        },).toThrow(CreditsShapeError,);
      },
    },),

    it({
      name: 'REFUSES a non-finite balance, which would read as an unlimited budget and let a run '
        + 'keep buying past the point it can pay',
      fn: async () => {
        expect(() => {
          parseHyperCredits({ bodyText: '{"balance": 1e999}', },);
        },).toThrow(CreditsShapeError,);
      },
    },),
  ],
},);
