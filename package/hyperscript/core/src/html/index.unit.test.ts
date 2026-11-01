import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { $, } from './index.ts';

await describe({
  name: 'camelToKebab via $ style serialization',
  children: [
    it({
      name: 'lowercases and dashes a single interior capital (flexDirection)',
      fn: async () => {
        expect($({ tag: 'div', style: { flexDirection: 'row', }, },),).toBe(
          '<div style="flex-direction:row"></div>',
        );
      },
    },),
    it({
      name: 'dashes backgroundColor',
      fn: async () => {
        expect($({ tag: 'div', style: { backgroundColor: 'red', }, },),).toBe(
          '<div style="background-color:red"></div>',
        );
      },
    },),
    it({
      name: 'leaves an all-lowercase property unchanged (display)',
      fn: async () => {
        expect($({ tag: 'div', style: { display: 'flex', }, },),).toBe(
          '<div style="display:flex"></div>',
        );
      },
    },),
    it({
      name: 'emits a leading dash for a leading capital',
      fn: async () => {
        expect($({ tag: 'div', style: { WebkitTransform: 'none', }, },),).toBe(
          '<div style="-webkit-transform:none"></div>',
        );
      },
    },),
    it({
      name: 'dashes a trailing capital',
      fn: async () => {
        expect($({ tag: 'div', style: { fooB: 'x', }, },),).toBe(
          '<div style="foo-b:x"></div>',
        );
      },
    },),
    it({
      name: 'dashes each of consecutive capitals',
      fn: async () => {
        expect($({ tag: 'div', style: { aBC: 'x', }, },),).toBe(
          '<div style="a-b-c:x"></div>',
        );
      },
    },),
    it({
      name: 'maps an empty property name to an empty kebab string',
      fn: async () => {
        expect($({ tag: 'div', style: { '': 'red', }, },),).toBe(
          '<div style=":red"></div>',
        );
      },
    },),
    it({
      name: 'leaves already-kebab input unchanged',
      fn: async () => {
        expect($({ tag: 'div', style: { 'flex-direction': 'row', }, },),).toBe(
          '<div style="flex-direction:row"></div>',
        );
      },
    },),
    it({
      name: 'converts a long all-uppercase run in a single linear pass',
      fn: async () => {
        const runLength = 50_000;
        const property = 'A'.repeat(runLength,);
        const expectedDecl = '-a'.repeat(runLength,);
        expect($({ tag: 'div', style: { [property]: 'x', }, },),).toBe(
          `<div style="${expectedDecl}:x"></div>`,
        );
      },
    },),
  ],
},);
