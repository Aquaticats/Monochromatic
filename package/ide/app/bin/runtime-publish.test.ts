// Checks of the runtime publishing helpers that the mise task `runtime` uses.
// What: staging names are unique, a write lands atomically or not at all, an unchanged file is left alone, a failed
// write leaves no staging file, and concurrent processes publishing the same tree all succeed and agree. The lock
// admits one holder at a time and is released when its holder is killed.
// Why: `build` and `test` started together both ran `runtime`; they shared the staging name `<grammar>.so.new`, one
// renamed it away, and the other failed with ENOENT. The concurrent test below fails when the staging name is fixed.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import {
  publishBytes,
  publishFile,
  publishTree,
  removeStaging,
  stagingPathFor,
  stagingSuffix,
  withExclusiveLock,
} from './runtime-publish.ts';

const modulePath = fileURLToPath(new URL('./runtime-publish.ts', import.meta.url));

const scratch = (): string => mkdtempSync(join(tmpdir(), 'runtime-publish-'));

const namesBelow = (root: string): string[] => readdirSync(root, { recursive: true }).map(String).sort();

// Runs a TypeScript program in a child node process and resolves with its exit status and output.
const runChild = async ({ program, args }: { program: string; args: string[] }) => {
  const child = spawn(
    process.execPath,
    ['--input-type=module-typescript', '--eval', program, '--', ...args],
    { stdio: ['ignore', 'pipe', 'pipe'] },
  );
  const collect = async (stream: AsyncIterable<unknown>): Promise<string> => {
    let text = '';
    for await (const chunk of stream) {
      text += String(chunk);
    }
    return text;
  };
  const [stdout, stderr, [status]] = await Promise.all([collect(child.stdout), collect(child.stderr), once(child, 'close')]);
  return { status: status as number | null, output: stdout + stderr };
};

test('staging paths of one output differ between calls and match the sweep pattern', () => {
  const first = stagingPathFor({ output: '/x/awk.so' });
  const second = stagingPathFor({ output: '/x/awk.so' });
  assert.notEqual(first, second);
  assert.match(first, stagingSuffix);
  assert.ok(first.startsWith('/x/awk.so.' + process.pid + '.'));
});

test('publishBytes writes a new file, replaces a different one, and leaves no staging file', () => {
  const root = scratch();
  const output = join(root, 'manifest.json');
  publishBytes({ output, bytes: Buffer.from('one') });
  assert.equal(readFileSync(output, 'utf8'), 'one');
  publishBytes({ output, bytes: Buffer.from('two') });
  assert.equal(readFileSync(output, 'utf8'), 'two');
  assert.deepEqual(namesBelow(root), ['manifest.json']);
  rmSync(root, { recursive: true });
});

test('publishFile leaves a file with equal bytes untouched and replaces one with other bytes', () => {
  const root = scratch();
  const source = join(root, 'source.so');
  const output = join(root, 'out.so');
  writeFileSync(source, 'grammar');
  publishFile({ source, output });
  const first = statSync(output);
  publishFile({ source, output });
  assert.equal(statSync(output).ino, first.ino, 'an equal file is not replaced');
  assert.equal(statSync(output).mtimeMs, first.mtimeMs);
  writeFileSync(source, 'grammar changed');
  publishFile({ source, output });
  assert.equal(readFileSync(output, 'utf8'), 'grammar changed');
  assert.deepEqual(namesBelow(root), ['out.so', 'source.so']);
  rmSync(root, { recursive: true });
});

test('publishFile keeps the permission bits of its source', () => {
  const root = scratch();
  const source = join(root, 'tool');
  writeFileSync(source, 'x', { mode: 0o755 });
  publishFile({ source, output: join(root, 'copy') });
  assert.equal(statSync(join(root, 'copy')).mode & 0o777, statSync(source).mode & 0o777);
  rmSync(root, { recursive: true });
});

test('a failed publish removes its staging file and keeps the error', () => {
  const root = scratch();
  const source = join(root, 'source');
  writeFileSync(source, 'data');
  // A non-empty directory at the output path makes the final rename fail after the staging file exists.
  const blocked = join(root, 'blocked');
  mkdirSync(join(blocked, 'inside'), { recursive: true });
  assert.throws(() => publishFile({ source, output: blocked }));
  assert.throws(() => publishBytes({ output: blocked, bytes: Buffer.from('data') }));
  assert.deepEqual(namesBelow(root), ['blocked', 'blocked/inside', 'source']);
  rmSync(root, { recursive: true });
});

test('publishTree copies nested directories and single files, and merges into an existing tree', () => {
  const root = scratch();
  const source = join(root, 'source');
  mkdirSync(join(source, 'a', 'b'), { recursive: true });
  writeFileSync(join(source, 'top.scm'), 'top');
  writeFileSync(join(source, 'a', 'b', 'deep.scm'), 'deep');
  const destination = join(root, 'destination');
  mkdirSync(destination);
  writeFileSync(join(destination, 'kept.scm'), 'kept');
  publishTree({ source, destination });
  assert.deepEqual(namesBelow(destination), ['a', 'a/b', 'a/b/deep.scm', 'kept.scm', 'top.scm']);
  publishTree({ source: join(source, 'top.scm'), destination: join(root, 'single.scm') });
  assert.equal(readFileSync(join(root, 'single.scm'), 'utf8'), 'top');
  rmSync(root, { recursive: true });
});

test('publishTree keeps a link a link and replaces a link that already exists', () => {
  const root = scratch();
  writeFileSync(join(root, 'real.txt'), 'real');
  symlinkSync(join(root, 'real.txt'), join(root, 'LICENSE'));
  publishTree({ source: join(root, 'LICENSE'), destination: join(root, 'copy') });
  publishTree({ source: join(root, 'LICENSE'), destination: join(root, 'copy') });
  assert.ok(lstatSync(join(root, 'copy')).isSymbolicLink());
  assert.equal(readFileSync(join(root, 'copy'), 'utf8'), 'real');
  rmSync(root, { recursive: true });
});

test('removeStaging deletes only names the staging scheme produces', () => {
  const root = scratch();
  mkdirSync(join(root, 'queries'));
  writeFileSync(join(root, 'queries', 'highlights.scm'), 'keep');
  writeFileSync(join(root, 'queries', 'notes.new'), 'keep: not a staging name');
  writeFileSync(join(root, 'queries', 'highlights.scm.4242.0123abcd.new'), 'leftover');
  writeFileSync(join(root, 'manifest.json.7.deadbeef.new'), 'leftover');
  removeStaging({ root });
  assert.deepEqual(namesBelow(root), ['queries','queries/highlights.scm', 'queries/notes.new']);
  removeStaging({ root: join(root, 'missing') });
  rmSync(root, { recursive: true });
});

test('several processes publishing the same files at once all succeed and leave one complete tree', async () => {
  const root = scratch();
  const source = join(root, 'source');
  mkdirSync(source);
  const names = Array.from({ length: 12 }, (_, index) => `grammar-${index}.so`);
  for (const name of names) {
    writeFileSync(join(source, name), `${name}:`.repeat(20000));
  }
  const destination = join(root, 'destination');
  mkdirSync(destination);
  // Every process publishes every file ten times; the destination already holds the first writer's copy, so later
  // rounds exercise the skip, and the first round of all four processes races on the staging names.
  const program = `
    const { publishFile, publishBytes } = await import(${JSON.stringify(modulePath)})
    const [source, destination, ...names] = process.argv.slice(1)
    for (let round = 0; round < 10; round++) {
      for (const name of names) {
        publishFile({ source: source + '/' + name, output: destination + '/' + name })
      }
    }
    publishBytes({ output: destination + '/manifest.json', bytes: Buffer.from(JSON.stringify(names)) })
  `;
  const results = await Promise.all(
    Array.from({ length: 4 }, () => runChild({ program, args: [source, destination, ...names] })),
  );
  for (const result of results) {
    assert.equal(result.status, 0, result.output);
  }
  assert.deepEqual(namesBelow(destination), [...names, 'manifest.json'].sort());
  for (const name of names) {
    assert.equal(readFileSync(join(destination, name), 'utf8'), readFileSync(join(source, name), 'utf8'));
  }
  rmSync(root, { recursive: true });
});

test('the lock admits one holder at a time and returns the result of its work', async () => {
  const root = scratch();
  const lockFile = join(root, 'runtime.lock');
  const log = join(root, 'log.txt');
  const program = `
    const { withExclusiveLock } = await import(${JSON.stringify(modulePath)})
    const { appendFileSync } = await import('node:fs')
    const [lockFile, log, label] = process.argv.slice(1)
    await withExclusiveLock({ lockFile, work: async () => {
      appendFileSync(log, 'start ' + label + '\\n')
      await (await import('node:timers/promises')).setTimeout(300)
      appendFileSync(log, 'end ' + label + '\\n')
    } })
  `;
  const results = await Promise.all(
    ['a', 'b', 'c'].map((label) => runChild({ program, args: [lockFile, log, label] })),
  );
  for (const result of results) {
    assert.equal(result.status, 0, result.output);
  }
  const lines = readFileSync(log, 'utf8').trim().split('\n');
  assert.equal(lines.length, 6);
  for (let index = 0; index < lines.length; index += 2) {
    const label = lines[index]!.slice('start '.length);
    assert.equal(lines[index], 'start ' + label);
    assert.equal(lines[index + 1], 'end ' + label, 'a holder ends before the next one starts');
  }
  assert.equal(await withExclusiveLock({ lockFile, work: () => 42 }), 42);
  await assert.rejects(withExclusiveLock({ lockFile, work: () => Promise.reject(new Error('work failed')) }), /work failed/);
  assert.equal(await withExclusiveLock({ lockFile, work: () => 'released after a failure' }), 'released after a failure');
  rmSync(root, { recursive: true });
});

test('a lock whose holder is killed is released', async () => {
  const root = scratch();
  const lockFile = join(root, 'runtime.lock');
  const program = `
    const { withExclusiveLock } = await import(${JSON.stringify(modulePath)})
    const [lockFile] = process.argv.slice(1)
    await withExclusiveLock({ lockFile, work: async () => {
      console.log('holding')
      await (await import('node:timers/promises')).setTimeout(600000)
    } })
  `;
  const holder = spawn(
    process.execPath,
    ['--input-type=module-typescript', '--eval', program, '--', lockFile],
    { stdio: ['ignore', 'pipe', 'inherit'] },
  );
  await once(holder.stdout, 'data');
  holder.kill('SIGKILL');
  assert.equal(await withExclusiveLock({ lockFile, work: () => 'taken over' }), 'taken over');
  rmSync(root, { recursive: true });
});
