/**
 Tests for shell-script file recognition.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fileExtensionOf,
  isShellScriptFile,
  NO_SHEBANG_PROGRAM,
  shellShebangProgramName,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 First lines whose program a shell must be, with the name read from each.
 */
const SHELL_SHEBANG_LINES: readonly { readonly line: string; readonly program: string; }[] = [
  { line: '#!/bin/bash', program: 'bash', },
  { line: '#!/bin/sh -e', program: 'sh', },
  { line: '#!/usr/bin/env bash', program: 'bash', },
  { line: '#!/usr/bin/env -S bash -e', program: 'bash', },
  { line: '#!/usr/bin/env FOO=1 dash', program: 'dash', },
  { line: '#!/bin/ksh', program: 'ksh', },
  { line: '#!/usr/local/bin/zsh', program: 'zsh', },
  { line: '#!/bin/mksh', program: 'mksh', },
];

/**
 First lines whose program a shell must not be, with the name read from each.
 */
const OTHER_SHEBANG_LINES: readonly { readonly line: string; readonly program: string; }[] = [
  { line: '#!/usr/bin/python3', program: 'python3', },
  { line: '#!/usr/bin/env node', program: 'node', },
  { line: '#!/usr/bin/perl -w', program: 'perl', },
];

/**
 First lines that name no program at all.
 */
const LINE_WITHOUT_PROGRAM: readonly string[] = [
  '# not a shebang',
  '',
  '#!/usr/bin/env',
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: shellShebangProgramName.name,
      children: [
        ...SHELL_SHEBANG_LINES.map(function shellShebangLine(fixture,) {
          return it({
            name: `reads \`${fixture.line}\` as ${fixture.program}`,
            fn: async function testShellShebangLine() {
              expect(shellShebangProgramName(`${fixture.line}\necho hi\n`,),).toBe(fixture.program,);
            },
          },);
        },),
        ...OTHER_SHEBANG_LINES.map(function otherShebangLine(fixture,) {
          return it({
            name: `reads \`${fixture.line}\` as ${fixture.program}, which is no shell name`,
            fn: async function testOtherShebangLine() {
              expect(shellShebangProgramName(`${fixture.line}\necho hi\n`,),).toBe(fixture.program,);
            },
          },);
        },),
        ...LINE_WITHOUT_PROGRAM.map(function lineWithoutProgram(line,) {
          return it({
            name: `reads no program in \`${line}\``,
            fn: async function testLineWithoutProgram() {
              expect(shellShebangProgramName(`${line}\necho hi\n`,),).toBe(NO_SHEBANG_PROGRAM,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: fileExtensionOf.name,
      children: [
        ...['script.sh', 'a.b.bash'].map(function dottedName(name,) {
          /**
           Extension the fixture name ends with.
           */
          const expectedExtension = name.slice(name.lastIndexOf('.',),);
          return it({
            name: `reads the final extension of \`${name}\``,
            fn: async function testDottedName() {
              expect(fileExtensionOf(`/repo/${name}`,),).toBe(expectedExtension,);
            },
          },);
        },),
        it({
          name: 'lower-cases the extension',
          fn: async function testUpperCaseExtension() {
            expect(fileExtensionOf('/repo/script.SH',),).toBe('.sh',);
          },
        },),
        it({
          name: 'reads no extension for a dotfile or a bare name',
          fn: async function testNoExtension() {
            expect(fileExtensionOf('/repo/cleanup',),).toBe('',);
            expect(fileExtensionOf('/repo/.hidden',),).toBe('',);
          },
        },),
        it({
          name: 'reads no extension from a folder name carrying a dot',
          fn: async function testDotInFolder() {
            expect(fileExtensionOf('/repo/dir.with.dots/cleanup',),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: isShellScriptFile.name,
      children: [
        it({
          name: 'recognizes every shell-script extension',
          fn: async function testExtensions() {
            for (const extension of ['.sh', '.bash', '.zsh', '.ksh', '.dash', '.bats',]) {
              expect(
                isShellScriptFile({ path: `/repo/script${extension}`, text: 'echo hi\n', },),
              ).toBe(true,);
            }
          },
        },),
        it({
          name: 'recognizes a file whose first line asks for a shell',
          fn: async function testShebangFile() {
            expect(
              isShellScriptFile({ path: '/repo/cleanup', text: '#!/bin/bash\nrm -rf /tmp/x\n', },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'refuses a name and first line that name no shell',
          fn: async function testNotAScript() {
            expect(
              isShellScriptFile({ path: '/repo/notes.txt', text: 'plain text\n', },),
            ).toBe(false,);
            expect(
              isShellScriptFile({ path: '/repo/tool.py', text: '#!/usr/bin/python3\n', },),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
