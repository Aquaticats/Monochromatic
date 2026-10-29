import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { CloneSizeError, } from './errors.ts';

/**
 Streams `git rev-list ... | git pack-objects --stdout` and counts the exact
 bytes of the resulting single pack, without decoding the binary stream or
 buffering it in memory.
 
 `node:child_process` is used directly (not nano-spawn) because the byte count
 must come from raw `Buffer` chunk lengths on a real OS pipe; nano-spawn
 decodes stdout to a string (lossy for binary) and cannot attach an in-memory
 Writable as stdio (no backing file descriptor).
 
 @param cwd - repository directory passed to git via `-C`
 
 @param revListArgs - argument vector for the producing `git rev-list`
 
 @param packArgs - argument vector for the consuming `git pack-objects`
 
 @returns exact byte length of the produced pack
 
 @throws {@link CloneSizeError} when either git process exits non-zero
 
 @example
 ```ts
 const bytes = await measurePackBytes({
   cwd: '/repo',
   revListArgs: ['rev-list', '--objects', '--branches', '--tags'],
   packArgs: ['pack-objects', '--stdout', '--delta-base-offset'],
 });
 ```
 */
export async function measurePackBytes(
  {
    cwd,
    revListArgs,
    packArgs,
  }: {
    readonly cwd: string;
    readonly revListArgs: readonly string[];
    readonly packArgs: readonly string[];
  },
): Promise<number> {
  /**
   Tagged logger naming the pack measurement.
   */
  const rl = tagged({
    tag: measurePackBytes.name,
    l: logger,
  },);

  /**
   Producer: emits the object id list on stdout, no stdin, stderr discarded.
   */
  const producer = spawnChild(
    'git',
    [
      '-C',
      cwd,
      ...revListArgs,
    ],
    { stdio: [
      'ignore',
      'pipe',
      'ignore',
    ], },
  );
  /**
   Consumer: reads object ids on stdin, writes the pack to stdout.
   */
  const consumer = spawnChild(
    'git',
    [
      '-C',
      cwd,
      ...packArgs,
    ],
    { stdio: [
      'pipe',
      'pipe',
      'ignore',
    ], },
  );

  /**
   Producer stdout stream feeding the consumer.
   */
  const producerOut = nonNullishOrThrow(producer.stdout,);
  /**
   Consumer stdin stream receiving the object id list.
   */
  const consumerIn = nonNullishOrThrow(consumer.stdin,);
  /**
   Consumer stdout stream carrying the pack bytes.
   */
  const consumerOut = nonNullishOrThrow(consumer.stdout,);

  producerOut.on(
    'error',
    function onProducerStreamError(): void {
      rl.debug('producer stdout closed early (consumer likely exited)',);
    },
  );
  consumerIn.on(
    'error',
    function onConsumerStdinError(): void {
      rl.debug('consumer stdin closed early',);
    },
  );
  producerOut.pipe(consumerIn,);

  /**
   Byte accumulator; const binding with a mutated field for the lint.
   */
  const acc = { bytes: 0, };
  consumerOut.on(
    'data',
    function countChunk(chunk: Buffer,): void {
      acc.bytes += chunk.length;
    },
  );

  await Promise.all([
    once(
      producer,
      'close',
    ),
    once(
      consumer,
      'close',
    ),
  ],);

  if ((producer.exitCode !== 0) || (consumer.exitCode !== 0))
    throw new CloneSizeError({
      message: `pack measurement failed (rev-list=${String(producer.exitCode,)}, pack-objects=${
        String(consumer.exitCode,)
      })`,
    },);

  rl.debug(`measured pack ${String(acc.bytes,)} bytes`,);
  return acc.bytes;
}
