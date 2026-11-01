import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { resolve, } from 'node:path';

import {
  declaresBuildTask,
  isBuildTaskName,
  taskNameOfHeader,
} from '../dist/final/node/index.mjs';

/** Fixture package holding one nested pseudo-package per manifest shape. */
const FIXTURE_ROOT = resolve(
  import.meta.dirname,
  '../../../test-fixture/oxlint-test-import/case',
);

await describe({
  name: 'build task detection',
  children: [
    describe({
      name: taskNameOfHeader.name,
      children: [
        it({
          name: 'reads an unquoted task name',
          fn: async () => {
            expect(taskNameOfHeader({ line: '[tasks.build]', },),).toBe('build',);
          },
        },),
        it({
          name: 'reads a quoted task name, which names containing colons require',
          fn: async () => {
            expect(taskNameOfHeader({ line: '[tasks."build:js:node"]', },),).toBe('build:js:node',);
          },
        },),
        it({
          name: 'reads a non-build task name',
          fn: async () => {
            expect(taskNameOfHeader({ line: '[tasks.lint]', },),).toBe('lint',);
          },
        },),
        it({
          name: 'returns nothing for a non-task table header',
          fn: async () => {
            expect(taskNameOfHeader({ line: '[env]', },),).toBe('',);
          },
        },),
        it({
          name: 'returns nothing for a key-value line',
          fn: async () => {
            expect(taskNameOfHeader({ line: 'extends = "build"', },),).toBe('',);
          },
        },),
        it({
          name: 'returns nothing for an empty line',
          fn: async () => {
            expect(taskNameOfHeader({ line: '', },),).toBe('',);
          },
        },),
        it({
          name: 'returns nothing for an unterminated header',
          fn: async () => {
            expect(taskNameOfHeader({ line: '[tasks.build', },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: isBuildTaskName.name,
      children: [
        it({
          name: 'accepts the plain build task',
          fn: async () => {
            expect(isBuildTaskName({ name: 'build', },),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a scoped build task',
          fn: async () => {
            expect(isBuildTaskName({ name: 'build:js:node', },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects a task merely starting with the same letters',
          fn: async () => {
            expect(isBuildTaskName({ name: 'buildless', },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects an unrelated task',
          fn: async () => {
            expect(isBuildTaskName({ name: 'lint', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: declaresBuildTask.name,
      children: [
        it({
          name: 'accepts a package declaring a plain build task',
          fn: async () => {
            expect(declaresBuildTask({
              packageRoot: resolve(
                FIXTURE_ROOT,
                'standard',
              ),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a package declaring only a scoped build task',
          fn: async () => {
            expect(declaresBuildTask({
              packageRoot: resolve(
                FIXTURE_ROOT,
                'asset-dist',
              ),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects a package with no task file at all',
          fn: async () => {
            expect(declaresBuildTask({
              packageRoot: resolve(
                FIXTURE_ROOT,
                'buildless',
              ),
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
