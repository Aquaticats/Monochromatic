import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  CorpusPin,
  listCorpusPeople,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type CorpusPageReader,
  drawEntry,
} from './window-trial-probe-draw.ts';
import {
  checkWindowReached,
  WINDOW_UNCHECKED,
} from './window-trial-probe-check.ts';
import {
  boughtLine,
  ledgerReadLine,
  openingLine,
  walkEndLine,
} from './window-trial-probe-lines.ts';
import {
  NOTHING_BOUGHT,
  tallyRefusal,
  tallyRows,
} from './window-trial-probe-tally.ts';
import {
  accountTrialLedger,
  completedArms,
  readTrialLedger,
} from './window-trial-ledger.ts';
import type { runPick, } from './window-trial-pick.ts';
import { protocolDigest, } from './window-trial-protocol.ts';
import {
  reportWindowTrial,
  windowTrialReportLine,
} from './window-trial-report.ts';
import { witnessSheets, } from './window-trial-witness.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Window trial probe run
// The window trial, run end to end: does showing the judges the neighbouring original
// change how often the archive's English is replaced? Moved out of
// `window-trial-probe.ts`, which keeps the wiring.
//
// SPENDS QUOTA when the client it is given reaches a provider, roughly three
// judgings plus one slate per drawn slice. Everything the process owns arrives
// as an argument: the runs directory, the head the protocol digest is of, the
// client (built only where the walk begins, so a refusal to build one comes
// after the ledger read as it always did), the corpus and the buyer of one slice.
//
// EVERY PART OF THE READING IS TESTED WITHOUT QUOTA and lives elsewhere: the
// ledger, the draw, the per-slice arms and the report each have their own file
// and their own tests. What is here is composition and the corpus walk.

/**
 Lists the people the pinned corpus holds, as `listCorpusPeople` does.
 */
type PeopleLister = typeof listCorpusPeople;

/**
 Runs the trial over the pinned corpus.

 @param runsDir - run directory whose `window-trial` folder holds the ledger

 @param headSha - head the protocol digest is of

 @param makeClient - builds the client every call goes through, called once
 after the ledger is read

 @param pin - corpus checkout and commit to read at

 @param listPeople - lister of the corpus's people

 @param readPage - reader of one corpus page

 @param pickSlice - buyer of one drawn slice's arms

 @param roster - models that translate and judge every slice

 @param perCallTimeoutMs - deadline per model exchange

 @param l - logger the run writes under

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when refusals in a row reach the stop

 @throws {@link WindowEvidenceError} when the first wide arm did not show the
 window to every judge

 @example
 ```ts
 await runWindowTrial({
   runsDir,
   headSha,
   makeClient: createProbeRunClient,
   pin,
   listPeople: listCorpusPeople,
   readPage: readCorpusFile,
   pickSlice: runPick,
   roster,
   perCallTimeoutMs,
   l,
   stageSettings,
 },);
 ```
 */
export async function runWindowTrial(
  {
    runsDir,
    headSha,
    makeClient,
    pin,
    listPeople,
    readPage,
    pickSlice,
    roster,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: {
    readonly runsDir: string;
    readonly headSha: string;
    readonly makeClient: () => ModelClient;
    readonly pin: CorpusPin;
    readonly listPeople: PeopleLister;
    readonly readPage: CorpusPageReader;
    readonly pickSlice: typeof runPick;
    readonly roster: readonly RosterModelId[];
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Where this run's ledger lives.
   */
  const ledgerPath = join(
    runsDir,
    'window-trial',
    'arms.jsonl',
  );

  /**
   Digest this run buys under.
   */
  const protocol = protocolDigest({
    headSha,
    pin,
  },);

  /**
   Arms already bought under it.
   */
  const done = completedArms({
    rows: await readTrialLedger({ path: ledgerPath, },),
    protocol,
  },);
  l.info(openingLine({
    protocol,
    armsBought: done.size,
  },),);

  /**
   Client every call goes through.
   */
  const client = makeClient();

  /**
   Nothing here aborts the run, so a kill is what stops it.
   */
  const { signal, } = new AbortController();

  /**
   Wrapper the run buys under until the window is seen on the wire.
   */
  const witness = witnessSheets({ client, },);

  /**
   What the walk has bought, refused and checked so far, each part replaced
   by the step that changes it.
   */
  const walked = {
    tally: NOTHING_BOUGHT,
    check: WINDOW_UNCHECKED,
  };

  for (const entryId of await listPeople({ pin, },)) {
    /* oxlint-disable no-await-in-loop -- entries are walked in order so a kill leaves a prefix */
    /**
     Slices this entry contributes, with the preparation they index into and
     the page facts of that preparation.
     */
    const drawn = await drawEntry({
      entryId,
      pin,
      readPage,
      l,
    },);
    /* oxlint-enable no-await-in-loop */
    for (const pick of drawn.picks) {
      /* oxlint-disable no-await-in-loop -- arms are bought one slice at a time and appended as they complete */
      /**
       Whether the window check has passed, so the recording client is dropped.
       */
      const { passed, } = walked.check;
      /**
       What this slice yielded: arms it bought, empty when the ledger already
       held them all, or a refusal the walk steps over.
       */
      const outcome = await pickSlice({
        client: passed ? client : witness.client,
        slices: drawn.slices,
        pick,
        entryId,
        protocol,
        ledgerPath,
        done,
        models: {
          translatorModelIds: roster,
          judgeModelIds: roster,
        },
        signal,
        perCallTimeoutMs,
        l,
        sheetFacts: drawn.sheetFacts,
        stageSettings,
      },);
      /* oxlint-enable no-await-in-loop */
      if (outcome.kind === 'refused') {
        walked.tally = tallyRefusal({ tally: walked.tally, },);
        continue;
      }
      /**
       Arms this call bought.
       */
      const { rows, } = outcome;
      walked.tally = tallyRows({
        tally: walked.tally,
        rows,
      },);
      if (rows.length === 0)
        continue;
      walked.check = checkWindowReached({
        check: walked.check,
        rows,
        sheets: witness.sheets,
        judges: roster.length,
        l,
      },);
      l.info(boughtLine({
        entryId,
        pick,
        rows,
      },),);
    }
  }

  /**
   What the walk bought and refused.
   */
  const { tally, } = walked;
  l.info(walkEndLine({
    count: tally.count,
    refused: tally.refused,
  },),);

  /**
   What the ledger holds now the walk is over, with what the read left out of
   the rows every line of this report is counted from.
   */
  const {
    rows: ledgerRows,
    leftOut,
    tornTail,
  } = await accountTrialLedger({ path: ledgerPath, },);
  l.info(ledgerReadLine({
    rows: ledgerRows.length,
    leftOut,
    tornTail,
  },),);
  for (const report of reportWindowTrial({
    rows: ledgerRows,
    protocol,
  },)) {
    l.info(windowTrialReportLine({ report, },),);
  }
}

//endregion Window trial probe run
