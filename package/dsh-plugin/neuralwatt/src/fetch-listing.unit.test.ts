import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fetchNeuralWattListing,
  NeuralWattListingError,
  NEURALWATT_LISTING_URL,
} from '../dist/final/node/index.mjs';

await describe({
  name: fetchNeuralWattListing.name,
  children: [
    //region Success

    it({
      name: 'returns the listing entries the endpoint sent, dropping non-record members',
      fn: async () => {
        const listing = await fetchNeuralWattListing({
          fetcher: async ({ url, }) => {
            expect(url,).toBe(NEURALWATT_LISTING_URL,);
            return {
              ok: true,
              status: 200,
              json: async () => ({ data: [{ id: 'a', }, 5, null, 'x',], }),
            };
          },
        },);
        expect(listing,).toHaveLength(1,);
        expect(listing[0]?.id,).toBe('a',);
      },
    },),

    it({
      name: 'reads the listing endpoint through the default fetcher',
      fn: async ctx => {
        const fake = ctx.sinon.stub(globalThis, 'fetch',).callsFake(
          async () => new Response('{"data": []}', { status: 200, },),
        );
        const listing = await fetchNeuralWattListing({ timeoutMs: 2_000, },);
        expect(fake,).toHaveBeenCalledTimes(1,);
        expect(fake.firstCall?.args[0],).toBe(NEURALWATT_LISTING_URL,);
        expect(fake.firstCall?.args[1]?.signal,).toBeInstanceOf(AbortSignal,);
        expect(listing,).toEqual([],);
      },
    },),

    //endregion Success

    //region Failures

    it({
      name: 'throws a listing error naming the HTTP status when the endpoint answers non-OK',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchNeuralWattListing({
            fetcher: async () => ({
              ok: false,
              status: 503,
              json: async () => ({}),
            }),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).message,)
          .toBe(`NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed: HTTP 503`,);
      },
    },),

    it({
      name: 'throws a listing error when the body is not a record',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchNeuralWattListing({
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => 5,
            }),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).message,)
          .toBe(`NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed: response carries no model list`,);
      },
    },),

    it({
      name: 'throws a listing error when the body carries no model list',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchNeuralWattListing({
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => ({ data: 'not a list', }),
            }),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).message,)
          .toBe(`NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed: response carries no model list`,);
      },
    },),

    it({
      name: 'wraps a transport rejection with its cause',
      fn: async () => {
        const transportFailure = new Error('connection refused',);
        let caught: unknown;
        try {
          await fetchNeuralWattListing({
            fetcher: async () => {
              throw transportFailure;
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).cause,).toBe(transportFailure,);
        expect((caught as Error).message,)
          .toBe(`NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed at the transport step`,);
      },
    },),

    it({
      name: 'wraps a body parse rejection with its cause',
      fn: async () => {
        const parseFailure = new Error('invalid json',);
        let caught: unknown;
        try {
          await fetchNeuralWattListing({
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => {
                throw parseFailure;
              },
            }),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).cause,).toBe(parseFailure,);
      },
    },),

    it({
      name: 'aborts and wraps the fetch when the bound expires',
      fn: async () => {
        let caught: unknown;
        let aborted = false;
        try {
          await fetchNeuralWattListing({
            timeoutMs: 5,
            fetcher: ({ signal, }) => new Promise((_resolve, reject) => {
              signal.addEventListener('abort', () => {
                aborted = true;
                reject(new Error('aborted by timeout',),);
              },);
            }),
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(aborted,).toBe(true,);
        expect(caught,).toBeInstanceOf(NeuralWattListingError,);
        expect((caught as Error).message,)
          .toBe(`NeuralWatt listing GET ${NEURALWATT_LISTING_URL} failed at the transport step`,);
      },
    },),

    //endregion Failures
  ],
},);
