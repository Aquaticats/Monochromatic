#!/usr/bin/env node
/**
 Test double for the `btrfs` program, for the unit tests of the move tools.

 A subvolume is a folder. A source subvolume carries a `.stub-subvolume` file
 whose text is its UUID; every other subvolume this double makes carries a
 `.meta` file of `key=value` lines. Only the subcommands the move tools call
 exist; anything else exits 2.

 Behavior knobs, by environment variable (each names the folder it applies to):
 `FAIL_SEND` makes the send of that snapshot fail, `SLOW_SEND` makes it sleep five seconds, `FAIL_RO` makes the received
 copy of it writable, and `STUBS` lists nested folders that a writable snapshot gets, the way a received parent holds
 empty stubs of its nested subvolumes. `BTRFS_DOUBLE_LOG` names a file that gets one JSON line of argv per call.

 @module
 */

import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import {
  basename,
  join,
} from 'node:path';
import { exit, } from 'node:process';

/**
 Milliseconds the slow send sleeps, long enough for a one-second deadline to end it.
 */
const SLOW_SEND_MILLISECONDS = 5000;

/**
 Name of the marker file of a source subvolume.
 */
const SOURCE_MARKER = '.stub-subvolume';

/**
 Name of the metadata file of every other subvolume.
 */
const META_FILE = '.meta';

type Meta = Record<string, string>;

function metaPath(folder: string,): string {
  return join(folder, META_FILE,);
}

function readMeta(folder: string,): Meta {
  if (existsSync(metaPath(folder,),)) {
    return Object.fromEntries(
      readFileSync(metaPath(folder,), 'utf8',).split('\n',).filter((line,) => line !== '').map((line,) =>
        line.split('=',)
      ),
    ) as Meta;
  }
  const marker = join(folder, SOURCE_MARKER,);
  if (!existsSync(marker,)) {
    throw new Error(`not a subvolume: ${folder}`,);
  }
  return { uuid: readFileSync(marker, 'utf8',).trim(), parent: '-', received: '-', ro: 'false', };
}

function writeMeta(folder: string, meta: Meta,): void {
  writeFileSync(
    metaPath(folder,),
    `${Object.entries(meta,).map(([key, value,],) => `${key}=${value}`).join('\n',)}\n`,
  );
}

function sleepMilliseconds(milliseconds: number,): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, milliseconds,);
}

const arguments_ = process.argv.slice(2,);
if (process.env['BTRFS_DOUBLE_LOG'] !== undefined) {
  appendFileSync(process.env['BTRFS_DOUBLE_LOG'], `${JSON.stringify(arguments_,)}\n`,);
}
const [first, second,] = arguments_;

if (first === 'subvolume' && second === 'snapshot') {
  const readOnly = arguments_[2] === '-r';
  const [source, destination,] = readOnly ? [arguments_[3], arguments_[4],] : [arguments_[2], arguments_[3],];
  if (source === undefined || destination === undefined || existsSync(destination,)) {
    console.error(`stub: cannot snapshot ${String(source,)} to ${String(destination,)}`,);
    exit(1,);
  }
  mkdirSync(destination,);
  writeMeta(destination, {
    uuid: `u-${basename(destination,)}`,
    parent: readMeta(source,).uuid,
    received: '-',
    ro: String(readOnly,),
  },);
  if (!readOnly && process.env['STUBS'] !== undefined) {
    for (const stub of process.env['STUBS'].split(',',)) {
      mkdirSync(join(destination, stub,), { recursive: true, },);
    }
  }
  exit(0,);
}
if (first === 'subvolume' && second === 'show') {
  const meta = readMeta(arguments_[2] ?? '',);
  console.log(`UUID:\t\t\t${meta['uuid']}\nParent UUID:\t\t${meta['parent']}\nReceived UUID:\t\t${meta['received']}`,);
  exit(0,);
}
if (first === 'subvolume' && second === 'create') {
  const destination = arguments_[2] ?? '';
  mkdirSync(destination,);
  writeFileSync(join(destination, SOURCE_MARKER,), `u-created-${basename(destination,)}\n`,);
  exit(0,);
}
if (first === 'property' && second === 'get') {
  console.log(`ro=${readMeta(arguments_[3] ?? '',)['ro']}`,);
  exit(0,);
}
if (first === 'property' && second === 'set') {
  const folder = arguments_[3] ?? '';
  writeMeta(folder, { ...readMeta(folder,), ro: arguments_[5] ?? 'false', },);
  exit(0,);
}
if (first === 'send') {
  const snapshot = arguments_.at(-1,) ?? '';
  const name = basename(snapshot,);
  if (process.env['SLOW_SEND'] === name) {
    sleepMilliseconds(SLOW_SEND_MILLISECONDS,);
  }
  if (process.env['FAIL_SEND'] === name) {
    console.error('stub: send failed',);
    exit(1,);
  }
  const options = arguments_.slice(1, -1,);
  const parent = options[options.indexOf('-p',) + 1];
  const clones = options.flatMap((option, index,) => (option === '-c' ? [basename(options[index + 1] ?? '',),] : []));
  process.stdout.write(
    `${
      JSON.stringify({
        uuid: readMeta(snapshot,)['uuid'],
        name,
        parent: options.includes('-p',) && parent !== undefined ? basename(parent,) : '-',
        clones: clones.join(',',),
      },)
    }\n`,
  );
  exit(0,);
}
if (first === 'receive') {
  const input = readFileSync(0, 'utf8',).trim();
  const destinationFolder = arguments_[1] ?? '';
  if (input === '') {
    mkdirSync(join(destinationFolder, 'PARTIAL',),);
    writeMeta(join(destinationFolder, 'PARTIAL',), { uuid: 'r-PARTIAL', parent: '-', received: '-', ro: 'true', },);
    exit(0,);
  }
  const stream = JSON.parse(input,) as { uuid: string; name: string; parent: string; clones: string; };
  const destination = join(destinationFolder, stream.name,);
  mkdirSync(destination,);
  writeMeta(destination, {
    uuid: `r-${stream.name}`,
    parent: '-',
    received: stream.uuid,
    ro: process.env['FAIL_RO'] === stream.name ? 'false' : 'true',
    sentParent: stream.parent,
    sentClones: stream.clones,
  },);
  exit(0,);
}
console.error(`stub: unsupported ${arguments_.join(' ',)}`,);
exit(2,);
