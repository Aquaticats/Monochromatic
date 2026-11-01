import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  askUserQuestionConfigPath,
  loadAskUserQuestionConfig,
} from '../dist/final/node/index.mjs';

//region Fixture

/**
 Disposable home for user-level config tests.
 */
type ConfigHome = AsyncDisposable & {
  readonly path: string;
};

/**
 Creates isolated home directory.
 
 @returns disposable home fixture
 */
async function configHome(): Promise<ConfigHome> {
  /**
   Unique home path.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'ask-user-config-test-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Writes user-level extension configuration.
 
 @param home - isolated home path
 
 @param text - config JSON source
 */
async function writeConfig({
  home,
  text,
}: {
  readonly home: string;
  readonly text: string;
}): Promise<void> {
  /**
   Config path under isolated home.
   */
  const path = askUserQuestionConfigPath({ home, },);
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
}

//endregion Fixture

await describe({
  name: loadAskUserQuestionConfig.name,
  children: [
    it({
      name: 'uses environment editor when user config is absent',
      fn: async () => {
        await using home = await configHome();
        expect(await loadAskUserQuestionConfig({
          home: home.path,
          env: { EDITOR: 'hx', },
          platform: 'linux',
        },),)
          .toEqual({
            editorCommand: ['hx',],
            source: {
              path: askUserQuestionConfigPath({ home: home.path, },),
              loaded: false,
            },
          },);
      },
    },),
    it({
      name: 'user config editor overrides VISUAL and EDITOR',
      fn: async () => {
        await using home = await configHome();
        await writeConfig({
          home: home.path,
          text: '{"editor":"nano --nowrap"}',
        },);
        expect(await loadAskUserQuestionConfig({
          home: home.path,
          env: {
            VISUAL: 'code --wait',
            EDITOR: 'hx',
          },
          platform: 'linux',
        },),)
          .toEqual({
            editorCommand: [
              'nano',
              '--nowrap',
            ],
            source: {
              path: askUserQuestionConfigPath({ home: home.path, },),
              loaded: true,
            },
          },);
      },
    },),
    ...[
      '{',
      'null',
      '{"editor":""}',
      '{"editor":42}',
      '{"editor":"nano","extra":true}',
    ].map(function toInvalidConfigTest(text,) {
      return it({
        name: `rejects invalid config ${text}`,
        fn: async () => {
          await using home = await configHome();
          await writeConfig({
            home: home.path,
            text,
          },);
          /**
           Captured config failure.
           */
          const caught: { value?: unknown; } = {};
          try {
            await loadAskUserQuestionConfig({
              home: home.path,
              env: {},
              platform: 'linux',
            },);
          }
          catch (error: unknown) {
            caught.value = error;
          }
          expect(caught.value,)
            .toBeInstanceOf(Error,);
        },
      },);
    },),
  ],
},);
