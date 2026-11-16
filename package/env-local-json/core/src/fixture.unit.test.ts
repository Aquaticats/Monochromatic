/**
 The fixture's own command runner returns the whole standard output of the
 tool it ran.

 The regression this guards: `runCommand` joined its captured chunks at the
 tool's `exit` event, which fires while the tool's standard output pipe can
 still hold unread data. A tool whose output reaches this process after that
 event leaves the join short, and `encryptJson` then writes the cut text as a
 store that every reader refuses as "not the JSON a sops store is".

 The stand-in tool below is a real process, not a synthetic emitter: it prints
 the head of a store, leaves a background subshell holding its standard output
 open, and exits. The tail therefore reaches this process strictly after the
 direct child's `exit` event, whichever way the scheduler runs the two.

 @module
 */

import {
  chmod,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempRoot,
  runCommand,
} from './fixture.ts';

//region Constants

/**
 Head of the document the stand-in tool prints, in the shape of a store.
 */
const STORE_HEAD = '{"values":';

/**
 Tail of that document, which the stand-in prints after its direct child has
 ended.
 */
const STORE_TAIL = '{"STAND_IN_NAME":"stand-in-value"}}';

/**
 Whole document the stand-in tool prints, and the only text a reader can parse.
 */
const WHOLE_STORE = `${STORE_HEAD}${STORE_TAIL}`;

/**
 Values {@link WHOLE_STORE} declares, which is what a cut store cannot parse to.
 */
const STAND_IN_VALUES: Readonly<Record<string, string>> = { STAND_IN_NAME: 'stand-in-value', };

/**
 Seconds the stand-in's background subshell waits before printing the tail:
 long enough that the direct child has ended in every observed run, so the
 tail cannot arrive before `exit`.
 */
const TAIL_DELAY_SECONDS = 0.3;

/**
 File name of the stand-in executable inside its disposable root.
 */
const STAND_IN_TOOL_NAME = 'stand-in-tool';

//endregion Constants

//region Helpers

/**
 Writes an executable that prints a store's head, holds its standard output
 open from a background subshell, and exits before that subshell prints the
 tail.

 @param root - disposable directory that holds the executable

 @returns path of the executable

 @example
 ```ts
 const toolPath = await writeStandInTool({ root: temp.root });
 ```
 */
async function writeStandInTool({
  root,
}: {
  readonly root: string;
}): Promise<string> {
  /**
   Path of the stand-in executable.
   */
  const path = join(
    root,
    STAND_IN_TOOL_NAME,
  );
  /**
   Script whose background subshell inherits the standard output pipe, so the
   pipe stays open after the direct child ends.
   */
  const script = [
    '#!/bin/sh',
    `printf '%s' '${STORE_HEAD}'`,
    `( sleep ${String(TAIL_DELAY_SECONDS,)}; printf '%s' '${STORE_TAIL}' ) &`,
    'exit 0',
    '',
  ]
    .join('\n',);
  await writeFile(
    path,
    script,
    { mode: 0o700, },
  );
  await chmod(
    path,
    0o700,
  );
  return path;
}

//endregion Helpers

await describe({
  children: [
    it({
      fn: async () => {
        await using temp = await createTempRoot({ prefix: 'env-local-json-run-command-', },);
        /**
         Path of the tool whose tail arrives after its exit event.
         */
        const toolPath = await writeStandInTool({ root: temp.root, },);
        /**
         What the fixture's runner captured from that tool.
         */
        const result = await runCommand({
          args: [],
          command: toolPath,
        },);
        expect(result.stdout,)
          .toBe(WHOLE_STORE,);
        // The consequence, not just the text: a cut capture is a store no reader can parse, which is how the failure of
        // `encryptJson`'s store reached the driver's suite as a SyntaxError.
        expect(JSON.parse(result.stdout,),)
          .toEqual(STAND_IN_VALUES,);
      },
      name: 'returns the standard output a tool writes after its exit event, once its child closes',
    },),
  ],
  name: 'the fixture command runner',
},);
