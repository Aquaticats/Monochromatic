/**
 Tests for applying a tool call's text edits.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  textAfterEdits,
  textAfterOneEdit,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

await describe({
  name: '',
  children: [
    describe({
      name: textAfterOneEdit.name,
      children: [
        it({
          name: 'replaces the first occurrence only',
          fn: async function testFirstOccurrence() {
            expect(
              textAfterOneEdit({
                text: 'a a a',
                edit: {
                  oldText: 'a',
                  newText: 'b',
                },
              },),
            ).toBe('b a a',);
          },
        },),
        it({
          name: 'replaces every occurrence when the call asks for it',
          fn: async function testEveryOccurrence() {
            expect(
              textAfterOneEdit({
                text: 'a a a',
                edit: {
                  oldText: 'a',
                  newText: 'b',
                  replaceAll: true,
                },
              },),
            ).toBe('b b b',);
          },
        },),
        it({
          name: 'leaves the text alone when the replaced text is absent',
          fn: async function testAbsentText() {
            expect(
              textAfterOneEdit({
                text: 'unchanged',
                edit: {
                  oldText: 'missing',
                  newText: 'x',
                },
              },),
            ).toBe('unchanged',);
          },
        },),
        it({
          name: 'puts a created file\'s content in front of empty text',
          fn: async function testEmptyReplacement() {
            expect(
              textAfterOneEdit({
                text: '',
                edit: {
                  oldText: '',
                  newText: '#!/bin/sh\n',
                },
              },),
            ).toBe('#!/bin/sh\n',);
          },
        },),
      ],
    },),
    describe({
      name: textAfterEdits.name,
      children: [
        it({
          name: 'applies the edits in the order the call reports them',
          fn: async function testEditOrder() {
            expect(
              textAfterEdits({
                text: 'one',
                edits: [
                  {
                    oldText: 'one',
                    newText: 'two',
                  },
                  {
                    oldText: 'two',
                    newText: 'three',
                  },
                ],
              },),
            ).toBe('three',);
          },
        },),
        it({
          name: 'returns the text unchanged when the call carries no edit',
          fn: async function testNoEdits() {
            expect(
              textAfterEdits({
                text: 'one',
                edits: [],
              },),
            ).toBe('one',);
          },
        },),
        it({
          name: 'builds a file from empty text',
          fn: async function testCreatedFile() {
            expect(
              textAfterEdits({
                text: '',
                edits: [{
                  oldText: '',
                  newText: '#!/bin/sh\n',
                },],
              },),
            ).toBe('#!/bin/sh\n',);
          },
        },),
      ],
    },),
  ],
},);
