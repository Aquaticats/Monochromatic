/**
 Tests for global pi settings restoration.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { restoreGlobalDefaultThinkingLevel, } from './global-settings.ts';

//region Test helpers

/** In-memory settings file harness. */
type SettingsFileHarness = {
  /** Reads stored settings JSON. */
  readSettingsFile: (path: string,) => Promise<string>;
  /** Writes stored settings JSON. */
  writeSettingsFile: (
    options: { readonly path: string; readonly content: string; },
  ) => Promise<void>;
  /** Returns last stored settings JSON. */
  getContent: () => string;
  /** Returns write count. */
  getWriteCount: () => number;
};

/**
 Creates in-memory settings file accessors.
 
 @param initialContent - starting settings JSON text
 
 @returns fake settings reader and writer
 
 @example
 ```typescript
 const harness = createSettingsFileHarness({ initialContent: '{"defaultThinkingLevel":"xhigh"}' });
 ```
 */
function createSettingsFileHarness(
  {
    initialContent,
  }: {
    initialContent: string;
  },
): SettingsFileHarness {
  /** Mutable settings text. */
  const contentSlot = new Map<'value', string>([
    ['value', initialContent,],
  ],);
  /** Number of writes performed. */
  const writeCountSlot = new Map<'value', number>([
    ['value', 0,],
  ],);

  return {
    readSettingsFile: async function readSettingsFile(path: string,): Promise<string> {
      void path;
      return contentSlot.get('value',) ?? '';
    },
    writeSettingsFile: async function writeSettingsFile(
      {
        path,
        content,
      }: { readonly path: string; readonly content: string; },
    ): Promise<void> {
      void path;
      contentSlot.set('value', content,);
      writeCountSlot.set(
        'value',
        (writeCountSlot.get('value',) ?? 0) + 1,
      );
    },
    getContent: function getContent(): string {
      return contentSlot.get('value',) ?? '';
    },
    getWriteCount: function getWriteCount(): number {
      return writeCountSlot.get('value',) ?? 0;
    },
  };
}

//endregion Test helpers

await describe({
  name: restoreGlobalDefaultThinkingLevel.name,
  children: [
    it({
      name: 'rewrites xhigh scalar default back to high',
      fn: async function testRestoreFromXhigh() {
        const harness = createSettingsFileHarness({
          initialContent: JSON.stringify({ defaultThinkingLevel: 'xhigh', },),
        },);

        const changed = await restoreGlobalDefaultThinkingLevel({
          settingsPath: '/settings.json',
          readSettingsFile: harness.readSettingsFile,
          writeSettingsFile: harness.writeSettingsFile,
        },);

        expect(changed,).toBe(true,);
        expect(
          JSON.parse(harness.getContent(),),
        ).toEqual({
          defaultThinkingLevel: 'high',
        },);
        expect(harness.getWriteCount(),).toBe(1,);
      },
    },),
    it({
      name: 'does not rewrite settings already at high',
      fn: async function testAlreadyHigh() {
        const initialContent = JSON.stringify({ defaultThinkingLevel: 'high', },);
        const harness = createSettingsFileHarness({ initialContent, },);

        const changed = await restoreGlobalDefaultThinkingLevel({
          settingsPath: '/settings.json',
          readSettingsFile: harness.readSettingsFile,
          writeSettingsFile: harness.writeSettingsFile,
        },);

        expect(changed,).toBe(false,);
        expect(harness.getContent(),).toBe(initialContent,);
        expect(harness.getWriteCount(),).toBe(0,);
      },
    },),
    it({
      name: 'throws for non-object settings JSON',
      fn: async function testNonObjectSettings() {
        const harness = createSettingsFileHarness({ initialContent: '[]', },);
        /** Error caught from invalid settings restoration. */
        const caught = await (async function catchRestoreError(): Promise<unknown> {
          try {
            await restoreGlobalDefaultThinkingLevel({
              settingsPath: '/settings.json',
              readSettingsFile: harness.readSettingsFile,
              writeSettingsFile: harness.writeSettingsFile,
            },);
            return undefined;
          }
          catch (error) {
            return error;
          }
        })();

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toBe(
          'Global pi settings JSON must be an object.',
        );
      },
    },),
  ],
},);
