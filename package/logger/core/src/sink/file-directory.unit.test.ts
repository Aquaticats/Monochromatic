/** Verify configured destinations through the actual sink and default logger. @module */
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { existsSync } from 'node:fs';
import { chmod, mkdir, mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { describe, it } from '@monochromatic-dev/test-harness-core/ts';
import { createFileSink } from '../../dist/final/node/node.mjs';

//region Actual filesystem and child-process boundaries retain independent destinations.
/**
 Node owns the child lifecycle; captured stderr must remain empty through cleanup.

 @param cwd - disposable workspace exercising ancestor discovery
 @param home - disposable home excluding shared logger state
 @param directory - explicit destination or empty ancestor-discovery control
 @param marker - expected persisted record identifying the contrast
 @param source - consumer import and actual logger calls
 @param environment - extra child variables layered over the logger destination
 */
async function run({ cwd, home, directory, marker, source, environment = {} }: {
  readonly cwd: string;
  readonly home: string;
  readonly directory: string;
  readonly marker: string;
  readonly source: string;
  readonly environment?: Readonly<Record<string, string>>;
}): Promise<void> {
  const child = spawn(process.execPath, ['--max-old-space-size=128', '--input-type=module', '--eval', source], {
    cwd,
    env: { ...process.env, HOME: home, MONOCHROMATIC_LOG_DIRECTORY: directory, LOGGER_DIRECTORY_MARKER: marker, ...environment },
    stdio: ['ignore', 'ignore', 'pipe'],
  });
  const errors: Buffer[] = [];
  child.stderr.on('data', function capture(chunk: Buffer): void {
    errors.push(chunk);
  });
  const status: unknown[] = await once(child, 'close');
  const [code, signal] = status;
  const stderr = Buffer.concat(errors).toString();
  assert.equal(code, 0, stderr);
  assert.equal(signal, null);
  assert.equal(stderr, '');
}
/** Create disposable workspaces without changing shared process environment or cwd. */
async function workspace() {
  const scratch = join(homedir(), 'temp', 'agent');
  await mkdir(scratch, { recursive: true, mode: 0o700 });
  await chmod(scratch, 0o700);
  const path = await mkdtemp(join(scratch, 'logger-directory-'));
  return { path, async [Symbol.asyncDispose]() {
    await rm(path, { recursive: true });
  } };
}
/** List sink log files one level below the destination, skipping stamps and probe residue. */
async function logNames(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const names = await Promise.all(entries.map(async function collect(entry) {
    if (!entry.isDirectory()) return entry.name.endsWith('.log.jsonl') ? [entry.name] : [];
    const inner = await readdir(join(directory, entry.name));
    return inner.filter(name => name.endsWith('.log.jsonl'))
      .map(name => join(entry.name, name));
  }));
  return names.flat().toSorted();
}
/** Read the real JSONL records rather than an intercepted write callback. */
async function records(directory: string): Promise<unknown[]> {
  const result: unknown[] = [];
  const contents = await Promise.all((await logNames(directory)).map(function readLog(name) {
    return readFile(join(directory, name), 'utf8');
  }));
  for (const content of contents) {
    for (const line of content.trim().split('\n')) result.push(JSON.parse(line));
  }
  return result;
}
/** Match ordinary serialized data without asserting any backend state. */
function hasMessage({ values, message }: Readonly<{ values: readonly unknown[]; message: string }>): boolean {
  return values.some(value => ((typeof value) === 'object') && (value !== null) && ('message' in value) && (value.message === message));
}

await describe({ name: createFileSink.name, children: [
  it({ name: 'writes and verifies an explicit destination with real ordered records', fn: async function explicitDirectory() {
    await using fixture = await workspace();
    const directory = join(fixture.path, 'explicit');
    const sink = createFileSink({ directory });
    assert.equal(await sink.verify(), true);
    await sink.write({ level: 'info', timestamp: 0, message: 'first directory record' });
    await sink.write({ level: 'info', timestamp: 1, message: 'second directory record' });
    const values = await records(directory);
    assert.equal(hasMessage({ values, message: 'first directory record' }), true);
    assert.equal(hasMessage({ values, message: 'second directory record' }), true);
    // One process keeps exactly one log file under its day directory.
    assert.equal((await logNames(directory)).length, 1);
  } }),
  it({ name: 'default logger honors the environment destination and preserves unset ancestor discovery', fn: async function defaultDirectory() {
    await using fixture = await workspace();
    const cwd = join(fixture.path, 'project');
    await mkdir(join(cwd, 'node_modules'), { recursive: true });
    const ordinary = join(cwd, 'node_modules', '.monochromatic');
    const configured = join(fixture.path, 'configured');
    const source = `import {logger} from ${JSON.stringify(new URL('../../dist/final/node/index.mjs', import.meta.url).href)}; logger.info(process.env.LOGGER_DIRECTORY_MARKER); await logger.flush();`;
    await Promise.all([{ directory: '', marker: 'ancestor control' }, { directory: configured, marker: 'configured control' }]
      .map(function check(contrast) {
        return run({ cwd, home: fixture.path, source, ...contrast });
      }));
    assert.equal(hasMessage({ values: await records(ordinary), message: 'ancestor control' }), true);
    assert.equal(hasMessage({ values: await records(ordinary), message: 'configured control' }), false);
    assert.equal(hasMessage({ values: await records(configured), message: 'configured control' }), true);
  } }),
  it({ name: 'a process that writes no records leaves no log files', fn: async function lazyCreation() {
    await using fixture = await workspace();
    const directory = join(fixture.path, 'quiet');
    const source = `import {logger} from ${JSON.stringify(new URL('../../dist/final/node/index.mjs', import.meta.url).href)}; await logger.flush();`;
    await run({ cwd: fixture.path, home: fixture.path, directory, marker: 'unused', source });
    // Lazy file creation plus the always-removed probe mean a zero-record
    // process persists zero log files and zero probe residue.
    const names = existsSync(directory) ? await readdir(directory) : [];
    assert.equal(names.filter(name => name.endsWith('.log.jsonl')).length, 0);
    assert.equal(names.filter(name => name.endsWith('.probe.tmp')).length, 0);
  } }),
  it({ name: 'MONOCHROMATIC_LOG_FILE=off leaves no files even with an explicit destination', fn: async function killSwitch() {
    await using fixture = await workspace();
    const directory = join(fixture.path, 'off');
    const source = `import {logger} from ${JSON.stringify(new URL('../../dist/final/node/index.mjs', import.meta.url).href)}; logger.info(process.env.LOGGER_DIRECTORY_MARKER); await logger.flush();`;
    await run({ cwd: fixture.path, home: fixture.path, directory, marker: 'off control', source, environment: { MONOCHROMATIC_LOG_FILE: 'off' } });
    // The disabled file sink never probes or creates its destination.
    assert.equal(existsSync(directory), false);
  } }),
] });
//endregion
