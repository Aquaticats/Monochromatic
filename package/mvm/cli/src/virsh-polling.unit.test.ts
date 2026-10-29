import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  delimiter,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  waitForGuestAgent,
  waitForShutdown,
} from '../dist/final/node/index.mjs';

type FakeVirshMode =
  | 'guest-ready-second'
    | 'guest-timeout'
    | 'shutdown-second';

type FakeVirshFixture = AsyncDisposable & {
  readonly counterPath: string;
};

const FAKE_VIRSH_SOURCE = `#!/usr/bin/env node
const { readFileSync, writeFileSync } = require('node:fs');
const counterPath = process.env.MVM_FAKE_VIRSH_COUNTER;
const mode = process.env.MVM_FAKE_VIRSH_MODE;
if ((counterPath === undefined) || (mode === undefined)) {
  throw new Error('fake virsh environment is incomplete');
}
let count = 0;
try {
  count = Number.parseInt(readFileSync(counterPath, 'utf8'), 10);
}
catch (error) {
  if ((error === null) || (typeof error !== 'object') || (error.code !== 'ENOENT')) {
    throw error;
  }
}
count += 1;
writeFileSync(counterPath, String(count));
if (mode === 'guest-ready-second') {
  if (count === 1) {
    process.stderr.write('guest agent unavailable\\n');
    process.exitCode = 1;
  }
  else {
    process.stdout.write('{}\\n');
  }
}
else if (mode === 'guest-timeout') {
  process.stderr.write('guest agent unavailable\\n');
  process.exitCode = 1;
}
else if (mode === 'shutdown-second') {
  process.stdout.write(count === 1 ? 'running\\n' : 'shut off\\n');
}
else {
  throw new Error('unsupported fake virsh mode');
}
`;

async function installFakeVirsh(mode: FakeVirshMode,): Promise<FakeVirshFixture> {
  const directory = await mkdtemp(join(
    tmpdir(),
    'mvm-virsh-polling-',
  ),);
  const virshPath = join(
    directory,
    'virsh',
  );
  const counterPath = join(
    directory,
    'counter.txt',
  );
  await writeFile(
    virshPath,
    FAKE_VIRSH_SOURCE,
    { mode: 0o700, },
  );

  const priorPath = process.env.PATH;
  const priorCounter = process.env.MVM_FAKE_VIRSH_COUNTER;
  const priorMode = process.env.MVM_FAKE_VIRSH_MODE;
  process.env.PATH = [
    directory,
    ...(priorPath === undefined ? [] : [priorPath,]),
  ].join(delimiter,);
  process.env.MVM_FAKE_VIRSH_COUNTER = counterPath;
  process.env.MVM_FAKE_VIRSH_MODE = mode;

  return {
    counterPath,
    async [Symbol.asyncDispose]() {
      if (priorPath === undefined)
        Reflect.deleteProperty(process.env, 'PATH',);
      else
        process.env.PATH = priorPath;
      if (priorCounter === undefined)
        Reflect.deleteProperty(process.env, 'MVM_FAKE_VIRSH_COUNTER',);
      else
        process.env.MVM_FAKE_VIRSH_COUNTER = priorCounter;
      if (priorMode === undefined)
        Reflect.deleteProperty(process.env, 'MVM_FAKE_VIRSH_MODE',);
      else
        process.env.MVM_FAKE_VIRSH_MODE = priorMode;
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

async function readCallCount(fixture: FakeVirshFixture,): Promise<number> {
  const rawCount = await readFile(
    fixture.counterPath,
    'utf8',
  );
  return Math.trunc(Number(rawCount,),);
}

await describe({
  name: 'virsh polling',
  concurrency: 1,
  children: [
    it({
      name: 'retries guest agent readiness after a failed first attempt',
      fn: async () => {
        await using fixture = await installFakeVirsh('guest-ready-second',);
        await waitForGuestAgent({ name: 'test', },);
        expect(await readCallCount(fixture,),).toBe(2,);
      },
    },),
    it({
      name: 'times out guest agent readiness after the first failed attempt',
      fn: async () => {
        await using fixture = await installFakeVirsh('guest-timeout',);
        await expect(waitForGuestAgent({
          name: 'test',
          timeoutMs: 0,
        },),).rejects.toThrow('did not respond within 0s',);
        expect(await readCallCount(fixture,),).toBe(1,);
      },
    },),
    it({
      name: 'polls shutdown state immediately and stops at shut off',
      fn: async () => {
        await using fixture = await installFakeVirsh('shutdown-second',);
        await waitForShutdown({ name: 'test', },);
        expect(await readCallCount(fixture,),).toBe(2,);
      },
    },),
  ],
},);
