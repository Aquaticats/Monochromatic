/**
 Query-parameter codec tests for {@link extractAttrType} and {@link stripAttrQuery}.
 
 The plugin tags resolved IDs with `?__importattr=<type>` and reads the tag back in
 both resolveId and load, so the codec must survive the query shapes a real build
 produces.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ATTR_QUERY_KEY,
  extractAttrType,
  NO_QUERY_ATTR,
  stripAttrQuery,
} from '../dist/final/node/index.mjs';

/** Module ID the transform tags, used as the positive control for both helpers. */
const TAGGED_ID = `./schema.sql?${ATTR_QUERY_KEY}=text`;

await describe({
  name: '',
  children: [
    describe({
      name: extractAttrType.name,
      children: [
        it({
          name: 'reads the type out of a tagged ID',
          fn: async () => {
            expect(extractAttrType(TAGGED_ID,),).toBe('text',);
          },
        },),
        it({
          name: 'reads the type when another query parameter follows it',
          fn: async () => {
            /**
             Tagged ID carrying a second parameter after the attribute marker.
             */
            const id = `${TAGGED_ID}&raw=1`;
            expect(extractAttrType(id,),).toBe('text',);
          },
        },),
        it({
          name: 'reads the type when another query parameter comes first',
          fn: async () => {
            /**
             Tagged ID whose attribute follows an unrelated parameter, which carries no marker.
             */
            const id = `./schema.sql?raw=1&${ATTR_QUERY_KEY}=text`;
            expect(extractAttrType(id,),).toBe(NO_QUERY_ATTR,);
          },
        },),
        it({
          name: 'reads an empty type value as an empty string rather than as no marker',
          fn: async () => {
            /**
             Tagged ID whose type value is missing; the marker is present, so the type is the empty string.
             */
            const id = `./schema.sql?${ATTR_QUERY_KEY}=`;
            expect(extractAttrType(id,),).toBe('',);
          },
        },),
        it({
          name: 'reports no marker for an ID the transform never tagged',
          fn: async () => {
            expect(extractAttrType('./schema.sql',),).toBe(NO_QUERY_ATTR,);
          },
        },),
        it({
          name: 'reports no marker for an unknown attribute type spelled by hand',
          fn: async () => {
            /**
             Hand-written marker naming a type no handler registers; the codec still reads it,
             because registration is the load hook's check.
             */
            const id = `./schema.sql?${ATTR_QUERY_KEY}=unheard`;
            expect(extractAttrType(id,),).toBe('unheard',);
          },
        },),
      ],
    },),
    describe({
      name: stripAttrQuery.name,
      children: [
        it({
          name: 'removes the attribute query from a tagged ID',
          fn: async () => {
            expect(stripAttrQuery(TAGGED_ID,),).toBe('./schema.sql',);
          },
        },),
        it({
          name: 'leaves an untagged ID alone',
          fn: async () => {
            expect(stripAttrQuery('./schema.sql',),).toBe('./schema.sql',);
          },
        },),
        it({
          name: 'leaves an ID whose unrelated query carries no marker alone',
          fn: async () => {
            /**
             ID whose only query parameter is not the attribute marker.
             */
            const id = './schema.sql?raw=1';
            expect(stripAttrQuery(id,),).toBe(id,);
          },
        },),
        it({
          name: 'removes a trailing unrelated parameter along with the marker',
          fn: async () => {
            /**
             Tagged ID whose second parameter is unrelated; the path before the marker is what remains.
             */
            const id = `${TAGGED_ID}&raw=1`;
            expect(stripAttrQuery(id,),).toBe('./schema.sql',);
          },
        },),
      ],
    },),
  ],
},);
