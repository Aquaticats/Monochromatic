/**
 Tests for the files a failing check keeps beside its failure file: the names they are kept under and what is written
 when a file is not there.
 
 @module
 */

import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readEvidenceFile,
  UNIT_ACCOUNTING_FILES,
  UNIT_LOG_NAME,
} from './session-evidence.ts';

await describe({
  name: readEvidenceFile.name,
  concurrency: 1,
  children: [
    it({
      name: 'reads a file under the name the session keeps it by and remembers where it came from',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-session-evidence-',),);
        const workDir = scratchDirectory.path;
        const logPath = join(
          workDir,
          'wide.log',
        );
        await writeFile(
          logPath,
          'ui_state seq=7 reason=key\n',
        );
        const file = await readEvidenceFile({
          name: UNIT_LOG_NAME,
          source: logPath,
        },);
        expect(file,).toEqual({
          kind: 'kept',
          name: 'unit.log',
          source: logPath,
          text: 'ui_state seq=7 reason=key\n',
        },);
      },
    },),
    it({
      name: 'says why a file is missing instead of failing the check that needs it',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-session-evidence-',),);
        const workDir = scratchDirectory.path;
        await mkdir(join(workDir, 'gone.service',),);
        const source = join(
          workDir,
          'gone.service',
          'io.pressure',
        );
        const file = await readEvidenceFile({
          name: 'io.pressure',
          source,
        },);
        expect(file.kind,).toBe('missing',);
        expect(file.name,).toBe('io.pressure',);
        expect(file.source,).toBe(source,);
        expect(('reason' in file) && file.reason.includes('ENOENT',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps the unit log under its own name and the accounting under the names the kernel gives them',
      fn: async ({ expect, }) => {
        expect(UNIT_LOG_NAME,).toBe('unit.log',);
        expect(UNIT_ACCOUNTING_FILES,).toEqual([
          'cpu.stat',
          'io.pressure',
        ],);
      },
    },),
  ],
},);
