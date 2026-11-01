import {
  appendFile,
  mkdir,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

//region Ledger line append
// ONE WAY TO ADD A LINE TO A SPEND LEDGER, shared by the ledgers that keep a
// JSON-lines file (`bedrock-ledger.ts`, `neuralwatt-ledger.ts`): the line goes
// on the end of the file and the directory is made on the way, so the first
// call of a fresh machine writes where a later one appends. APPEND-ONLY, so
// two processes writing at once cannot lose each other's lines.

/**
 Appends one entry to a ledger file as a JSON line, creating the directory on
 the way.

 @param path - ledger file

 @param entry - record to write, which must serialize as JSON

 @example
 ```ts
 await appendLedgerLine({ path: '/state/translation-repair/spend.jsonl', entry: { at: '2026-10-20T12:00:00.000Z', }, },);
 ```
 */
export async function appendLedgerLine(
  {
    path,
    entry,
  }: {
    readonly path: string;
    readonly entry: unknown;
  },
): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await appendFile(
    path,
    `${JSON.stringify(entry,)}\n`,
    'utf8',
  );
}

//endregion Ledger line append
