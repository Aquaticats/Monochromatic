import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EditorCommandError,
  resolveEditorCommand,
} from '../dist/final/node/index.mjs';

await describe({
  name: resolveEditorCommand.name,
  children: [
    it({
      name: 'prefers user config over VISUAL and EDITOR',
      fn: async () => {
        expect(resolveEditorCommand({
          configuredEditor: 'nano --nowrap',
          env: {
            VISUAL: 'code --wait',
            EDITOR: 'hx',
          },
          platform: 'linux',
        },),)
          .toEqual([
            'nano',
            '--nowrap',
          ],);
      },
    },),
    it({
      name: 'prefers VISUAL with configured arguments',
      fn: async () => {
        expect(resolveEditorCommand({
          env: {
            VISUAL: 'code --wait',
            EDITOR: 'hx',
          },
          platform: 'linux',
        },),)
          .toEqual([
            'code',
            '--wait',
          ],);
      },
    },),
    it({
      name: 'uses EDITOR when VISUAL is blank',
      fn: async () => {
        expect(resolveEditorCommand({
          env: {
            VISUAL: '  ',
            EDITOR: 'hx "--working-dir=/tmp/a b"',
          },
          platform: 'linux',
        },),)
          .toEqual([
            'hx',
            '--working-dir=/tmp/a b',
          ],);
      },
    },),
    it({
      name: 'uses vi fallback outside Windows',
      fn: async () => {
        expect(resolveEditorCommand({
          env: {},
          platform: 'linux',
        },),)
          .toEqual(['vi',],);
      },
    },),
    it({
      name: 'uses notepad fallback on Windows',
      fn: async () => {
        expect(resolveEditorCommand({
          env: {},
          platform: 'win32',
        },),)
          .toEqual(['notepad.exe',],);
      },
    },),
    it({
      name: 'rejects unsupported shell syntax',
      fn: async () => {
        /**
         Captured editor configuration failure.
         */
        const caught: { value?: unknown; } = {};
        try {
          resolveEditorCommand({
            env: { EDITOR: 'hx; other-command', },
            platform: 'linux',
          },);
        }
        catch (error: unknown) {
          caught.value = error;
        }
        expect(caught.value,)
          .toBeInstanceOf(EditorCommandError,);
      },
    },),
  ],
},);
