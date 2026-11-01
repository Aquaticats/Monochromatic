import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  copyFile,
  rm,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  createRequestRegistry,
  requestExternalAnswer,
} from '../dist/final/node/index.mjs';

//region Isolated running consumer

/**
 Filesystem permission bits checked independently of file type.
 */
const PERMISSION_MASK = 0o777;
/**
 Copied helper must be readable only by its owner.
 */
const PRIVATE_FILE_MODE = 0o600;
/**
 Bootstrap helper argument precedes the request option and its value.
 */
const HELPER_ARGUMENT_FROM_END = -3;

/**
 Relocates both launch inputs to paths that must never become JavaScript syntax.

 @param command - production argv whose final inputs belong to a disposable workspace

 @returns equivalent argv containing quotes, separators, URL escapes, and a newline
 */
async function quoteLaunchPaths(command: readonly string[],): Promise<readonly string[]> {
  /**
   Request-owned files created by the actual requester.
   */
  const helperPath = command.at(HELPER_ARGUMENT_FROM_END,);
  /**
   Coordination path remains the final argument.
   */
  const requestPath = command.at(-1,);
  if ((helperPath === undefined) || (requestPath === undefined))
    throw new Error('Missing fixture launch inputs.',);
  /**
   Characters significant to shell, JavaScript, and URL grammars.
   */
  const quotedHelper = join(
    dirname(helperPath,),
    `helper ' " # % ; $ \\n\n.mjs`,
  );
  /**
   Independently quoted request path exercises helper option parsing.
   */
  const quotedRequest = join(
    dirname(requestPath,),
    `request ' " # % ; $ \\n\n.json`,
  );
  await Promise.all([
    copyFile(
      helperPath,
      quotedHelper,
    ),
    copyFile(
      requestPath,
      quotedRequest,
    ),
  ],);
  return [
    ...command.slice(
      0,
      HELPER_ARGUMENT_FROM_END,
    ),
    quotedHelper,
    '--request',
    quotedRequest,
  ];
}
/**
 Driver runs only after being copied beside disposable built artifacts.
 */
const scenario = process.argv
  .at(-1,);
/**
 Runtime path captured before optional removal of its disposable copy.
 */
const originalRuntime = process.execPath;
/**
 Scripted editor submits multiline text through real helper protocol.
 */
const editorCommand = [
  originalRuntime,
  '--input-type=module',
  '--eval',
  String.raw`import {writeFile} from "node:fs/promises"; await writeFile(process.argv.at(-1), "first\nsecond\n");`,
];
if (scenario === 'runtime-removed') {
  if (dirname(originalRuntime,) !== process.cwd())
    throw new Error('Refusing to remove runtime outside disposable fixture.',);
  await rm(originalRuntime,);
  // Editor also needs the same still-running executable after its path is removed.
  editorCommand[0] = `/proc/${String(process.pid,)}/exe`;
}
if (scenario === 'helper-missing-before-launch')
  await rm(new URL(
    '../dist/final/node/answer-helper.mjs',
    import.meta.url,
  ),);
try {
  /**
   Result traverses extension, helper process, attached editor, and authenticated socket.
   */
  const outcome = await requestExternalAnswer({
    cwd: process.cwd(),
    registry: createRequestRegistry(),
    editorCommand,
    launch: async function launchFixture({ command, },) {
      if (scenario === 'helper-missing-before-launch')
        throw new Error('Opened terminal despite missing helper bundle.',);
      /**
       Snapshot and request must share request lifetime, not installation lifetime.
       */
      const helperPath = command.at(HELPER_ARGUMENT_FROM_END,);
      /**
       Request remains final argument so no token is exposed in process arguments.
       */
      const requestPath = command.at(-1,);
      if ((helperPath === undefined) || (requestPath === undefined))
        throw new Error('Missing private helper or request path.',);
      if (dirname(helperPath,) !== dirname(requestPath,))
        throw new Error('Helper launch still depends on installed bundle path.',);
      if ((process.platform !== 'win32') && (((await stat(helperPath,)).mode & PERMISSION_MASK) !== PRIVATE_FILE_MODE))
        throw new Error('Private helper is not restricted to its owner.',);
      if (scenario === 'helper-removed-after-launch')
        await rm(new URL(
          '../dist/final/node/answer-helper.mjs',
          import.meta.url,
        ),);
      /**
       Exact command assembled by production requester.
       */
      const [executable, ...args] = scenario === 'quoted-launch-paths'
        ? await quoteLaunchPaths(command,)
        : command;
      if (executable === undefined)
        throw new Error('Fixture received empty helper command.',);
      if (scenario === 'detached-start') {
        /**
         Real terminal launcher returns on spawn and does not retain child handle.
         */
        const detached = spawn(
          executable,
          args,
          { stdio: [
            'ignore',
            'ignore',
            'inherit',
          ], },
        );
        await once(
          detached,
          'spawn',
        );
        detached.unref();
        return;
      }
      /**
       No shell interpolation and no dependency path supplied to relocated helper.
       */
      const child = spawn(
        executable,
        args,
        { stdio: [
          'ignore',
          'pipe',
          'pipe',
        ], },
      );
      /**
       Failure text must never be hidden by successful requester assertions.
       */
      const output = { stderr: '', };
      child.stdout
        .resume();
      child.stderr
        .setEncoding('utf8',);
      child.stderr
        .on(
        'data',
        function captureError(chunk: string,): void {
          output.stderr += chunk;
        },
        );
      /**
       Close occurs after captured stderr has drained.
       */
      const exit: readonly unknown[] = await once(
        child,
        'close',
      );
      /**
       Exit code narrowed independently from untyped event tuple.
       */
      const [code,] = exit;
      if (code !== 0)
        throw new Error(`Detached helper exited ${String(code,)}:\n${output.stderr}`,);
      if (output.stderr !== '')
        throw new Error(`Detached helper wrote unexpected stderr:\n${output.stderr}`,);
    },
  },);
  if ((outcome.status !== 'answered') || (outcome.answer !== 'first\nsecond'))
    throw new Error(`Unexpected answer: ${JSON.stringify(outcome,)}`,);
  console.log(`verified ${String(scenario,)}`,);
}
catch (error: unknown) {
  if ((scenario !== 'helper-missing-before-launch') || (!Error.isError(error,))
    || (error.name !== 'AnswerLaunchError')
    || (!error.message
      .includes('answer-helper.mjs')))
    throw error;
  console.log(`verified ${scenario}`,);
}

//endregion Isolated running consumer
