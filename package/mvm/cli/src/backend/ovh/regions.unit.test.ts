/**
 Unit tests for region selection: the country comes from the API's own
 `countryCode` field and never from a region's name, an unusable region is left
 out, a region that offers no instance service is left out, and an explicit
 `--location` or `MVM_OVH_REGIONS` replaces the default list in that order.

 Region bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.Region`), never a recording, so nothing in this file claims what
 OVHcloud really answers for a project.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  listDefaultRegions,
  resolveRegions,
  selectDefaultRegions,
  type OvhRegion,
} from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  installEnv,
  installStandIn,
  jsonAnswer,
  regionAnswer,
  withEnv,
} from './fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 Regions whose names would mislead a reader and whose country codes are the only
 thing that decides them: an American region named for a Canadian city, two
 Canadian regions, a Canadian region that is down, and a region with no country.
 Two more are Canadian and up and still cannot take a machine: `bhs` offers
 storage only, the shape a live project showed on 2026-11-28 beside `BHS5`, and
 `cac1` lists an instance service that is down.
 */
const REGIONS = [
  regionAnswer({
    countryCode: 'us',
    name: 'ottawa-ca',
  },),
  regionAnswer({ name: 'bhs5', },),
  regionAnswer({ name: 'gra9', },),
  regionAnswer({
    name: 'toronto-ca',
    status: 'DOWN',
  },),
  regionAnswer({
    name: 'bhs',
    services: [
      {
        endpoint: 'https://storage.stand-in.invalid/',
        name: 'storage',
        status: 'UP',
      },
    ],
  },),
  regionAnswer({
    name: 'cac1',
    services: [
      {
        endpoint: 'https://compute.stand-in.invalid/',
        name: 'instance',
        status: 'DOWN',
      },
    ],
  },),
] as const satisfies readonly OvhRegion[];

//endregion Fixtures

await describe({
  name: 'ovh regions',
  concurrency: 1,
  children: [
    //region Selection

    it({
      name: 'selectDefaultRegions keeps the Canadian regions that are up, in name order, and no others',
      fn: async () => {
        expect(selectDefaultRegions({ details: REGIONS, },),).toEqual(['bhs5', 'gra9',],);
      },
    },),
    it({
      name: 'selectDefaultRegions reads the country field, not the region name',
      fn: async () => {
        expect(
          selectDefaultRegions({ details: REGIONS, },),
        ).not.toContain('ottawa-ca',);
        expect(
          selectDefaultRegions({ details: REGIONS, },),
        ).not.toContain('toronto-ca',);
      },
    },),
    it({
      name: 'selectDefaultRegions leaves out a region that offers no instance service that is up',
      fn: async () => {
        /**
         Names selected from regions that include a storage-only one and one
         whose instance service is down.
         */
        const selected = selectDefaultRegions({ details: REGIONS, },);
        expect(selected,).not.toContain('bhs',);
        expect(selected,).not.toContain('cac1',);
      },
    },),
    it({
      name: 'selectDefaultRegions returns nothing when no region qualifies',
      fn: async () => {
        expect(
          selectDefaultRegions({ details: REGIONS.slice(0, 1,), },),
        ).toEqual([],);
      },
    },),

    //endregion Selection

    //region Resolution

    it({
      name: 'resolveRegions lets an explicit location series win',
      fn: async () => {
        await using _env = await installEnv();
        using _set = withEnv({
          name: 'MVM_OVH_REGIONS',
          value: 'from-env',
        },);
        expect(await resolveRegions('first, second',),).toEqual(['first', 'second',],);
      },
    },),
    it({
      name: 'resolveRegions takes MVM_OVH_REGIONS when no location is given',
      fn: async () => {
        await using _env = await installEnv();
        using _set = withEnv({
          name: 'MVM_OVH_REGIONS',
          value: 'from-env, other-env',
        },);
        expect(await resolveRegions(),).toEqual(['from-env', 'other-env',],);
      },
    },),
    it({
      name: 'resolveRegions reads the project own regions when nothing overrides them',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(function answer(call,) {
          if (call.path === cloudPath('/region',)) {
            return jsonAnswer(['bhs5', 'ottawa-ca', 'toronto-ca',],);
          }
          return jsonAnswer(REGIONS.find(function named(region,) {
            return call.path === cloudPath(`/region/${region.name}`,);
          },) ?? {},);
        },);
        expect(await resolveRegions(),).toEqual(['bhs5',],);
        // One clock read, one region list, and one detail read per region name.
        expect(standIn.calls.length,).toBe(5,);
      },
    },),
    //endregion Resolution

    //region Empty project

    it({
      name: 'listDefaultRegions throws naming the country code and the overrides when none qualifies',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(function answer(call,) {
          if (call.path === cloudPath('/region',)) {
            return jsonAnswer(['ottawa-ca',],);
          }
          return jsonAnswer(regionAnswer({
            countryCode: 'us',
            name: 'ottawa-ca',
          },),);
        },);
        const error = await errorFrom({ run: () => listDefaultRegions() },);
        expect(Error.isError(error,),).toBe(true,);
        const text = caughtValueText(error,);
        expect(text,).toContain('countryCode',);
        expect(text,).toContain('MVM_OVH_REGIONS',);
      },
    },),

    //endregion Empty project
  ],
},);
