/**
 What a built extension bundle asks for at run time: the specifiers it cannot answer from itself, which is what stopped every Pi
 started in this session on 2026-11-11.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { lodgedImportsIn, } from '../dist/final/node/index.mjs';

await describe({
  name: lodgedImportsIn.name,
  children: [
    it({
      name: 'answers nothing for a bundle that asks only for node own modules and files beside it',
      fn: async () => {
        expect(lodgedImportsIn({
          bundle: 'import{a}from"node:fs";import{b}from"./chunk-A1.js";import"/abs/x.js";import("node:path");export{c}from"../y.js";',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'names a workspace package the bundle left for the machine to resolve, which is the failure Pi reports as a load failure',
      fn: async () => {
        expect(lodgedImportsIn({
          bundle: 'import{wait}from"@monochromatic-dev/async-time-core/ts";import{spawn}from"node:child_process";',
        },),).toEqual(['@monochromatic-dev/async-time-core/ts',],);
      },
    },),
    it({
      name: 'reads every shape a module asks for another in, and names a specifier once however often it appears',
      fn: async () => {
        expect(lodgedImportsIn({
          bundle: [
            'import{a}from"pkg-one";',
            'export{b}from"pkg-two";',
            'import"pkg-three";',
            'const lazy=()=>import("pkg-four");',
            'const cjs=require("pkg-five");',
            'import{d}from"pkg-one";',
          ].join('',),
        },),).toEqual(['pkg-one', 'pkg-two', 'pkg-three', 'pkg-four', 'pkg-five',],);
      },
    },),
    it({
      name: 'answers nothing for an empty bundle',
      fn: async () => {
        expect(lodgedImportsIn({ bundle: '', },),).toEqual([],);
      },
    },),
  ],
},);
