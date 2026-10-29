import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { findTypeAnnotations, } from './type-annotations.ts';

/** Count of repeated annotations used to exercise the scan on long input. */
const LONG_RUN = 1_000;

await describe({
  name: findTypeAnnotations.name,
  children: [
    it({
      name: 'returns empty array for empty string',
      fn: async () => {
        expect(findTypeAnnotations('',),).toEqual([],);
      },
    },),
    it({
      name: 'returns empty array when no tag is present',
      fn: async () => {
        expect(findTypeAnnotations('no tags here',),).toEqual([],);
      },
    },),
    it({
      name: 'captures a type body following a tag and whitespace',
      fn: async () => {
        expect(findTypeAnnotations('@param {string} x',),).toEqual(['string',],);
      },
    },),
    it({
      name: 'captures a type body at end of line',
      fn: async () => {
        expect(findTypeAnnotations('@returns {number}',),).toEqual(['number',],);
      },
    },),
    it({
      name: 'ignores an inline tag at the start of block content',
      fn: async () => {
        expect(
          findTypeAnnotations(
            '@throws {@link JsoncParseError} on malformed input',
          ),
        ).toEqual([],);
      },
    },),
    it({
      name: 'keeps scanning after an inline tag block opener',
      fn: async () => {
        expect(
          findTypeAnnotations('@throws {@link JsoncParseError} @returns {number}',),
        ).toEqual(['number',],);
      },
    },),
    it({
      name: 'skips an empty type body and keeps scanning',
      fn: async () => {
        expect(findTypeAnnotations('@param {} x',),).toEqual([],);
      },
    },),
    it({
      name: 'stops scanning at an unterminated brace',
      fn: async () => {
        expect(findTypeAnnotations('@param {string',),).toEqual([],);
      },
    },),
    it({
      name: 'requires whitespace between the tag and the brace',
      fn: async () => {
        expect(findTypeAnnotations('@param{string}',),).toEqual([],);
      },
    },),
    it({
      name: 'ignores a brace that does not follow a tag',
      fn: async () => {
        expect(findTypeAnnotations('@param string',),).toEqual([],);
      },
    },),
    it({
      name: 'returns empty array for a bare at-sign with no tag name',
      fn: async () => {
        expect(findTypeAnnotations('@ {x}',),).toEqual([],);
      },
    },),
    it({
      name: 'captures multiple type bodies in order',
      fn: async () => {
        expect(findTypeAnnotations('@param {a} @returns {b}',),).toEqual([
          'a',
          'b',
        ],);
      },
    },),
    it({
      name: 'keeps bodies collected before an unterminated brace halts the scan',
      fn: async () => {
        expect(findTypeAnnotations('@p {a} @q {b',),).toEqual(['a',],);
      },
    },),
    it({
      name: 'captures a long run of annotations without overflow',
      fn: async () => {
        const result = findTypeAnnotations('@p {a} '.repeat(LONG_RUN,),);
        expect(result.length,).toBe(LONG_RUN,);
        expect(result.every(function isA(body,): boolean {
          return body === 'a';
        },),).toBe(true,);
      },
    },),
  ],
},);
