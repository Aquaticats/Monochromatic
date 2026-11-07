/**
 Runs a shell command for a stage, under `timeout`, with its output teed to a file and the screen.

 @module
 */

import { spawn, } from 'node:child_process';
import { createWriteStream, } from 'node:fs';
import { once, } from 'node:events';
import { EXIT_STAGE_FAILED, } from './guided-stop.ts';

/**
 What a shell run needs.
 */
export type ShellRequest = {
  readonly command: string;
  readonly outputFile: string;

  /**
   Deadline in seconds, or zero for none.
   */
  readonly seconds: number;
  readonly killGrace: number;
  readonly outputFileEnv: string;
  readonly out: (text: string,) => void;
};

/**
 Builds the program and arguments of a shell run: `bash -e -o pipefail -c`, under `timeout` when a deadline is given.

 @param command - the shell text
 
 @param seconds - the deadline, or zero for none
 
 @param killGrace - seconds between terminate and kill
 
 @returns the program and its arguments
 */
function shellInvocation(
  {
    command,
    seconds,
    killGrace,
  }: {
    readonly command: string;
    readonly seconds: number;
    readonly killGrace: number;
  },
): readonly [
  string,
  readonly string[]
] {
  /**
   The bash of this step.
   */
  const bash = [
    'bash',
    '-e',
    '-o',
    'pipefail',
    '-c',
    command,
  ];
  return seconds > 0
    ? [
      'timeout',
      [
        `--kill-after=${String(killGrace,)}`,
        String(seconds,),
        ...bash,
      ],
    ]
    : [
      'bash',
      bash.slice(1,),
    ];
}

/**
 Runs a shell command with `bash -e -o pipefail -c`, under `timeout`, teeing its output to a file and the screen.

 @param request - the command, output file, deadline and sinks
 
 @returns the exit status

 @example
 ```ts
 import { runShell, } from './guided-shell.ts';
 
 const result = await runShell(request,);
 ```
 */
export async function runShell(request: ShellRequest,): Promise<number> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const [program, args,] = shellInvocation(request,);
  /**
   The sink of this step.
   */
  const sink = createWriteStream(request.outputFile,);
  /**
   The child of this step.
   */
  const child = spawn(
    program,
    args,
    {
      env: {
        ...process.env,
        OUTPUT_FILE: request.outputFileEnv,
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  for (const stream of [
    child.stdout,
    child.stderr,
  ]) {
    stream.on(
      'data',
      function tee(chunk: Buffer,) {
        sink.write(chunk,);
        request.out(chunk.toString('utf8',),);
      },
    );
  }
  await once(
    child,
    'close',
  );
  sink.end();
  await once(
    sink,
    'close',
  );
  return child.exitCode ?? EXIT_STAGE_FAILED;
}
