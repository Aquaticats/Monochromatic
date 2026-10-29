/**
 In-container historical-recall run for result-type bugs
 (`doc/audit/deepmerge-ts-recall-2026-09-24.md`), in `node:<major>-slim` with
 this repo read-only and `dist/recall/types` writable at their host paths,
 and extracted npm releases read-only at `RECALL_NPM_DIR/<version>/package`.

 `corpus <seed> <size>` draws a fresh declared-type corpus (the declared-type
 campaign's generator, run against the installed v8.0.2 runtime).
 `check <version>...` type-checks the whole sidecar plus that corpus with
 `deepmerge-ts` mapped to each release's declarations, and
 `control <id>` type-checks one ledger row's control against its buggy and
 fixed releases. Each job writes its diagnostics to `results/<job>.json`.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { emitModule, } from './declared-type-check.ts';
import { drawCases, } from './declared-type-generate.ts';
import { TYPE_BUGS, } from './recall-ledger-type.ts';
import {
  checkJob,
  OUT_DIR,
  PACKAGE_ROOT,
  RecallTypesError,
  typesEntry,
} from './recall-tsc.ts';

if (import.meta.main) {
  /**
   Mode and its arguments.
   */
  const [mode, ...args] = process.argv
    .slice(2,);
  /**
   Fresh corpus file.
   */
  const corpusFile = join(
    OUT_DIR,
    'fresh',
    'cases.ts',
  );
  if (mode === 'corpus') {
    await mkdir(
      join(
        OUT_DIR,
        'fresh',
      ),
      { recursive: true, },
    );
    await writeFile(
      corpusFile,
      emitModule({
        cases: drawCases({
          seed: Number(args[0],),
          size: Number(args[1],),
          unions: true,
        },),
        exportName: 'RECALL_CASES',
      },)
        .source,
    );
  } else if (mode === 'check') {
    for (const version of args) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one compiler at a time inside the 2 GiB cap.
      await checkJob({
        // oxlint-disable-next-line eslint/no-await-in-loop -- same.
        entry: await typesEntry(version,),
        include: [
          join(
            PACKAGE_ROOT,
            'src',
            '**',
            '*.ts',
          ),
          corpusFile,
        ],
        job: `sidecar-${version}`,
      },);
    }
  } else if (mode === 'control') {
    /**
     Requested ids that name no row with a control, which would otherwise run nothing silently.
     */
    const unmatched = args.filter(function noControl(id,) {
      return !TYPE_BUGS.some(function hasControl(row,) {
        return (row.id === id) && (row.control !== '');
      },);
    },);
    if (unmatched.length > 0)
      throw new RecallTypesError(`no type ledger row with a control for: ${unmatched.join(', ',)}`,);
    for (const bug of TYPE_BUGS.filter(function wanted(row,) {
      return (row.control !== '') && ((args.length === 0) || args.includes(row.id,));
    },)) {
      /**
       Control module path.
       */
      const file = join(
        OUT_DIR,
        'controls',
        `${bug.id}.ts`,
      );
      // oxlint-disable-next-line eslint/no-await-in-loop -- one compiler at a time inside the 2 GiB cap.
      await mkdir(
        join(
          OUT_DIR,
          'controls',
        ),
        { recursive: true, },
      );
      // oxlint-disable-next-line eslint/no-await-in-loop -- same.
      await writeFile(
        file,
        bug.control,
      );
      for (const side of [
        'buggy',
        'fixed',
      ] as const) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- same.
        await checkJob({
          // oxlint-disable-next-line eslint/no-await-in-loop -- same.
          entry: await typesEntry(bug[side],),
          include: [file,],
          job: `control-${bug.id}-${side}`,
        },);
      }
    }
  } else {
    throw new RecallTypesError(`unknown mode ${String(mode,)}; expected corpus, check, or control`,);
  }
}
