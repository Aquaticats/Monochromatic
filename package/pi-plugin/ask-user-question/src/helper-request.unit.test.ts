import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readHelperRequest, } from '../dist/final/node/index.mjs';

//region Fixture

/**
 Disposable helper request file fixture.
 */
type RequestFixture = AsyncDisposable & {
  readonly path: string;
};

/**
 Writes one private request payload in disposable temp directory.
 
 @param text - request file text
 
 @returns disposable fixture path
 */
async function requestFixture(text: string,): Promise<RequestFixture> {
  /**
   Unique fixture directory.
   */
  const directory = await mkdtemp(join(
    scratchRoot(),
    'ask-user-request-test-',
  ),);
  /**
   Request path under fixture directory.
   */
  const path = join(
    directory,
    'request.json',
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Fixture

await describe({
  name: readHelperRequest.name,
  children: [
    it({
      name: 'accepts complete request',
      fn: async () => {
        await using fixture = await requestFixture('{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"/tmp/ANSWER.md","editorCommand":["nano","--nowrap"]}',);
        expect(await readHelperRequest({ requestPath: fixture.path, }),)
          .toEqual({
            host: '127.0.0.1',
            port: 1_234,
            token: 'token',
            answerPath: '/tmp/ANSWER.md',
            editorCommand: [
              'nano',
              '--nowrap',
            ],
          },);
      },
    },),
    ...[
      'null',
      '{}',
      '{"host":"","port":1234,"token":"token","answerPath":"/tmp/a","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":0,"token":"token","answerPath":"/tmp/a","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":1.5,"token":"token","answerPath":"/tmp/a","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":"1234","token":"token","answerPath":"/tmp/a","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":1234,"token":"","answerPath":"/tmp/a","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"","editorCommand":["nano"]}',
      '{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"/tmp/a"}',
      '{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"/tmp/a","editorCommand":[]}',
      '{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"/tmp/a","editorCommand":[""]}',
      '{"host":"127.0.0.1","port":1234,"token":"token","answerPath":"/tmp/a","editorCommand":[42]}',
    ].map(function toInvalidRequestTest(text,) {
      return it({
        name: `rejects ${text}`,
        fn: async () => {
          await using fixture = await requestFixture(text,);
          /**
           Captured validation failure.
           */
          const caught: { value?: unknown; } = {};
          try {
            await readHelperRequest({ requestPath: fixture.path, },);
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
