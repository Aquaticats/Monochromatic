/**
 The channel to Pi against a stand-in `pi`: answers, silence, odd output, and every stage of ending. Each test uses its own
 disposable folder.

 @module
 */

import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { kill } from 'node:process';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type ConnectionToPi,
  type PiRecord,
  startConnectionToPi,
} from '../dist/final/node/index.mjs';

const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/** Milliseconds Pi gets at each stage of ending; the stand-in needs no more than its own start-up. */
const GRACE_MILLISECONDS = 1_500;
/** Milliseconds to wait for an answer that will never come. */
const SHORT_ANSWER_MILLISECONDS = 50;

type Heard = {
  readonly events: PiRecord[];
  readonly unreadable: string[];
  readonly problems: string[];
};

type Opened = {
  readonly connection: ConnectionToPi;
  readonly heard: Heard;
  readonly log: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function openConnection(
  { scenario, withoutPi = false, }: { readonly scenario: string; readonly withoutPi?: boolean; },
): Promise<Opened> {
  const folder = await mkdtemp(join(scratchRoot(), 'pi-connection-',),);
  const log = join(folder, 'commands.log',);
  if (!withoutPi) {
    await writeFile(join(folder, 'pi',), `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,);
    await chmod(join(folder, 'pi',), 0o700,);
  }
  const heard: Heard = { events: [], unreadable: [], problems: [], };
  const connection = startConnectionToPi({
    piArguments: [],
    workingFolder: folder,
    environment: {
      PATH: withoutPi ? dirname(process.execPath,) : [folder, dirname(process.execPath,), '/usr/bin',].join(delimiter,),
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
    },
    standardErrorFile: join(folder, 'stderr.log',),
    graceMilliseconds: GRACE_MILLISECONDS,
    listeners: {
      onEvent: (record,) => {
        heard.events.push(record,);
      },
      onUnreadableLine: (line,) => {
        heard.unreadable.push(line,);
      },
      onExit: () => {
        heard.problems.push('exit noted',);
      },
      onProblem: (problem,) => {
        heard.problems.push(problem,);
      },
    },
  },);
  return {
    connection,
    heard,
    log,
    async [Symbol.asyncDispose]() {
      await connection.shutDown();
      await rm(folder, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'the connection to Pi',
  children: [
    it({
      name: 'matches an answer to its command and ends Pi by closing its input',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'normal', },);
        const answer = await opened.connection.send({ command: { type: 'get_state', }, },);
        expect(answer.success,).toBe(true,);
        await opened.connection.shutDown();
        expect(opened.connection.hasExited(),).toBe(true,);
        expect(opened.connection.exitDetails().code,).toBe(0,);
      },
    },),
    it({
      name: 'a silent Pi gives a failed answer within the bound',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'silent', },);
        const answer = await opened.connection.send({ command: { type: 'get_state', }, answerMilliseconds: SHORT_ANSWER_MILLISECONDS, },);
        expect(answer.success,).toBe(false,);
        expect(String(answer.error,),).toContain('no answer',);
      },
    },),
    it({
      name: 'a command waiting when Pi exits gets a failed answer, and later commands fail at once',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'exits-silently', },);
        const answer = await opened.connection.send({ command: { type: 'get_state', }, },);
        expect(answer.error,).toBe('pi exited',);
        expect(opened.connection.exitDetails().code,).toBe(3,);
        const later = await opened.connection.send({ command: { type: 'get_state', }, },);
        expect(later.error,).toBe('pi exited',);
        opened.connection.write({ type: 'get_state', },);
      },
    },),
    it({
      name: 'a Pi that stays after its input closes is ended with SIGTERM',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'ignores-input-close', },);
        await opened.connection.send({ command: { type: 'get_state', }, },);
        await opened.connection.shutDown();
        expect(opened.connection.exitDetails().signal,).toBe('SIGTERM',);
      },
    },),
    it({
      name: 'a Pi that ignores SIGTERM is ended with SIGKILL',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'ignores-sigterm', },);
        await opened.connection.send({ command: { type: 'get_state', }, },);
        await opened.connection.shutDown();
        expect(opened.connection.exitDetails().signal,).toBe('SIGKILL',);
      },
    },),
    it({
      name: 'a tool still holding the output open does not keep the driver waiting',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'keeps-output-open', },);
        await opened.connection.send({ command: { type: 'get_state', }, },);
        const toolLine = (await readFile(opened.log, 'utf8',)).split('\n',).find((line,) => line.startsWith('tool ',));
        await opened.connection.shutDown();
        kill(
          Number(String(toolLine,).slice('tool '.length,),),
        );
        expect(opened.connection.hasExited(),).toBe(true,);
        expect(opened.connection.exitDetails().code,).toBe(0,);
      },
    },),
    it({
      name: 'skips a blank line, reports a line that is not a record, and treats a response without a text id as an event',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'odd-lines', },);
        const answer = await opened.connection.send({ command: { type: 'get_state', }, },);
        expect(answer.success,).toBe(true,);
        expect(opened.heard.unreadable,).toEqual(['[1]',],);
        expect(opened.heard.events,).toEqual([{ type: 'response', id: 7, },],);
      },
    },),
    it({
      name: 'a Pi that cannot be started is reported, and a command sent to it fails',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'normal', withoutPi: true, },);
        const answer = await opened.connection.send({ command: { type: 'get_state', }, },);
        expect(answer.error,).toBe('pi exited',);
        expect(String(opened.connection.exitDetails().processError,),).toContain('ENOENT',);
      },
    },),
  ],
},);
