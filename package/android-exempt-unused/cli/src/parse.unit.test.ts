/**
 Tests for the pure adb-output parsers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { AdbCommandError, } from './errors.ts';
import {
  isValidPackageName,
  parseDevices,
  parseExemptedQuery,
  parsePackageList,
} from './parse.ts';

await describe({
  name: 'parse',
  children: [
    describe({
      name: isValidPackageName.name,
      children: [
        it({
          name: 'accepts a dotted application id',
          fn: async () => {
            expect(isValidPackageName({ name: 'com.example.app', },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects empty, spaced, and shell-metachar names',
          fn: async () => {
            expect(isValidPackageName({ name: '', },),).toBe(false,);
            expect(isValidPackageName({ name: 'com example', },),).toBe(false,);
            expect(isValidPackageName({ name: 'com;rm -rf', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: parsePackageList.name,
      children: [
        it({
          name: 'strips the package: prefix',
          fn: async () => {
            expect(parsePackageList({ stdout: 'package:com.a\npackage:com.b\n', },),).toEqual(['com.a', 'com.b',],);
          },
        },),
        it({
          name: 'drops blank lines, CR, and non-package lines',
          fn: async () => {
            expect(
              parsePackageList({ stdout: 'warning: noise\npackage:com.a\r\n\npackage:com.b\r\n', },),
            ).toEqual(['com.a', 'com.b',],);
          },
        },),
        it({
          name: 'throws on an unparseable package name',
          fn: async () => {
            expect(function parseInvalid() {
              parsePackageList({ stdout: 'package:com a\n', },);
            },)
              .toThrow(AdbCommandError,);
          },
        },),
      ],
    },),
    describe({
      name: parseDevices.name,
      children: [
        it({
          name: 'parses serial and state, skipping the header',
          fn: async () => {
            expect(parseDevices({ stdout: 'List of devices attached\nABC123\tdevice\n', },),).toEqual([
              { serial: 'ABC123', state: 'device', },
            ],);
          },
        },),
        it({
          name: 'keeps every state and skips daemon chatter',
          fn: async () => {
            const stdout = '* daemon not running; starting now\n'
              + 'List of devices attached\n'
              + 'ABC\tdevice\n'
              + 'XYZ\tunauthorized\n'
              + 'emulator-5554\toffline\n';
            expect(parseDevices({ stdout, },),).toEqual([
              { serial: 'ABC', state: 'device', },
              { serial: 'XYZ', state: 'unauthorized', },
              { serial: 'emulator-5554', state: 'offline', },
            ],);
          },
        },),
        it({
          name: 'tolerates space separation',
          fn: async () => {
            expect(parseDevices({ stdout: 'ABC   device\n', },),).toEqual([{ serial: 'ABC', state: 'device', },],);
          },
        },),
      ],
    },),
    describe({
      name: parseExemptedQuery.name,
      children: [
        it({
          name: 'keeps bare valid package names',
          fn: async () => {
            expect(parseExemptedQuery({ stdout: 'com.a\ncom.b\n', },),).toEqual(['com.a', 'com.b',],);
          },
        },),
        it({
          name: 'ignores headers and chatter',
          fn: async () => {
            expect(
              parseExemptedQuery({ stdout: 'Uid 10123:\ncom.a\nNo operations.\n', },),
            ).toEqual(['com.a',],);
          },
        },),
        it({
          name: 'returns empty for empty output',
          fn: async () => {
            expect(parseExemptedQuery({ stdout: '', },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
