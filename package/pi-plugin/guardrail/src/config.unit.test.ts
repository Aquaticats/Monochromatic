/**
 Tests for pi guardrail config loading and normalization.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { FILE_NOT_FOUND_CODE, } from './constants.ts';
import { loadGuardrailConfig, } from './config.ts';
import { configPathForHome, } from './config-paths.ts';
import { normalizeConfigFile, } from './config-normalize.ts';
import { parseConfigJson, } from './config-file.ts';

/**
 Config path used by normalization tests.
 */
const CONFIG_PATH = '/home/user/.pi/agent/extensions/pi-guardrail.json';

/**
 Builds a config reader that returns fixed content.
 
 @param content - JSON text to return
 
 @returns config reader function
 
 @example
 ```typescript
 const reader = fixedReader('{"a":"b"}');
 ```
 */
function fixedReader(content: string,): (path: string) => Promise<string> {
  return async function readConfig(_path: string,): Promise<string> {
    return content;
  };
}

/**
 Config reader that behaves like a missing file.
 
 @param path - ignored requested path
 
 @throws missing-file error
 */
async function missingReader(path: string,): Promise<string> {
  void path;
  /**
   Missing-file error carrying ENOENT code.
   */
  const error = new Error('missing',) as Error & { code: string; };
  error.code = FILE_NOT_FOUND_CODE;
  throw error;
}

await describe({
  name: 'guardrail config',
  children: [
    it({
      name: 'resolves global config path under pi extension config dir',
      fn: async function testConfigPath() {
        expect(configPathForHome({ home: '/home/user', },),)
          .toBe('/home/user/.pi/agent/extensions/pi-guardrail.json',);
      },
    },),
    it({
      name: 'uses defaults when config file is absent',
      fn: async function testAbsentConfigDefaults() {
        const config = await loadGuardrailConfig({
          home: '/home/user',
          readConfigFile: missingReader,
        },);
        expect(config.source.loaded,).toBe(false,);
        expect(config.blockBunTest,).toBe(true,);
        expect(config.pathRules.map(function pattern(rule,) {
          return rule.pattern;
        },),)
          .toEqual(['pnpm-lock.yaml',],);
      },
    },),
    it({
      name: 'loads direct map config and appends it after defaults',
      fn: async function testDirectMapConfig() {
        const config = await loadGuardrailConfig({
          home: '/home/user',
          readConfigFile: fixedReader(JSON.stringify({
            'package-lock.json': 'use npm install',
          },),),
        },);
        expect(config.source.loaded,).toBe(true,);
        expect(config.pathRules.map(function pattern(rule,) {
          return rule.pattern;
        },),)
          .toEqual([
            'pnpm-lock.yaml',
            'package-lock.json',
          ],);
      },
    },),
    it({
      name: 'loads advanced config with pathRules and blockBunTest override',
      fn: async function testAdvancedConfig() {
        const config = await loadGuardrailConfig({
          home: '/home/user',
          readConfigFile: fixedReader(JSON.stringify({
            blockBunTest: false,
            pathRules: {
              'Cargo.lock': 'run cargo update intentionally',
            },
          },),),
        },);
        expect(config.blockBunTest,).toBe(false,);
        expect(config.pathRules.at(-1,),).toEqual({
          pattern: 'Cargo.lock',
          message: 'run cargo update intentionally',
        },);
      },
    },),
    it({
      name: 'rejects invalid JSON and invalid config shape',
      fn: async function testInvalidConfig() {
        expect(function parseBadJson() {
          parseConfigJson({ content: '{', configPath: CONFIG_PATH, },);
        },).toThrow('parsing failed',);
        expect(function normalizeArray() {
          normalizeConfigFile({ value: [], configPath: CONFIG_PATH, },);
        },).toThrow('must contain a JSON object',);
        expect(function normalizeBadValue() {
          normalizeConfigFile({ value: { 'pnpm-lock.yaml': 1, }, configPath: CONFIG_PATH, },);
        },).toThrow('must be a string message',);
      },
    },),
    it({
      name: 'rejects unknown advanced keys and invalid advanced values',
      fn: async function testInvalidAdvancedConfig() {
        expect(function normalizeUnknownAdvancedKey() {
          normalizeConfigFile({
            value: {
              blockBunTest: true,
              other: true,
            },
            configPath: CONFIG_PATH,
          },);
        },).toThrow('unknown advanced config keys',);
        expect(function normalizeBadBlockBunTest() {
          normalizeConfigFile({ value: { blockBunTest: 'yes', }, configPath: CONFIG_PATH, },);
        },).toThrow('blockBunTest must be boolean',);
        expect(function normalizeBadPathRules() {
          normalizeConfigFile({ value: { pathRules: [], }, configPath: CONFIG_PATH, },);
        },).toThrow('pathRules must be an object',);
      },
    },),
  ],
},);
