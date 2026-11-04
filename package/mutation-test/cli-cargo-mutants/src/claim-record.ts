/**
 The text of a claim file and the names of a claim series, as `package/git-policy/cli/src/native/lock/single_owner_claim.rs`
 spells them: a claim file is `<prefix>.<generation>`, and says either that its owner gave the item back or which process
 owns it (id, birth, PID namespace).
 */
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';

/**
 The first line of every claim file (`single_owner_claim.rs:36`).
 */
const claimHeader = 'cargo-mutants owner-claim v1';

/**
 The line of a released claim (`single_owner_claim.rs:39`).
 */
const releasedLine = 'released';

/**
 The text that stands for no namespace in a claim file.
 */
const noNamespaceText = '-';

/**
 Stands for text that is not a claim of this command.
 */
export const notAClaim: unique symbol = Symbol('claim-record.invalid');

/**
 A claim file of an owner that gave the item back.
 */
export type ReleasedRecord = {
  /**
   Marks the owner as having given the item back.
   */
  readonly kind: 'released';
};

/**
 A claim file of a process that owns the item.
 */
export type OwnedRecord = {
  /**
   Marks a process as owning the item.
   */
  readonly kind: 'owned';
  /**
   The owner's process id.
   */
  readonly pid: number;
  /**
   The owner's birth text.
   */
  readonly birth: string;
  /**
   The owner's PID namespace, absent when it was not known.
   */
  readonly namespace?: string;
};

/**
 What a claim file says.
 */
export type ClaimRecord = ReleasedRecord | OwnedRecord;

/**
 The text of a claim file (`single_owner_claim.rs:112`).

 @param record - what the file says

 @returns the text

 @example
 ```ts
 encodeClaimRecord({ kind: 'released' });
 ```
 */
export function encodeClaimRecord(record: ClaimRecord): string {
  if (record.kind === 'released')
    return `${claimHeader}\n${releasedLine}\n`;
  return `${claimHeader}\npid=${String(record.pid)}\nbirth=${record.birth}\nnamespace=${record.namespace ?? noNamespaceText}\n`;
}

/**
 The number a text of digits stands for, when it is the spelling of a number the claim files use.

 @param digits - the part of a file name after the prefix and the dot

 @returns the number, or {@link notAClaim} for empty text, text that holds anything but digits, a spelling with a leading
 zero, or a number too large to hold exactly

 @example
 ```ts
 numberFromDigits('12');
 ```
 */
function numberFromDigits(digits: string): number | typeof notAClaim {
  if (digits === '')
    return notAClaim;
  for (const character of digits)
    if ((character < '0') || (character > '9'))
      return notAClaim;
  /**
   The number.
   */
  const parsed = Number(digits);
  return (Number.isSafeInteger(parsed) && (String(parsed) === digits)) ? parsed : notAClaim;
}

/**
 Read the text after a label on a claim file's line.

 @param line - the line, empty when the file ends before it

 @param label - what the line starts with

 @returns the text after the label, or {@link notAClaim} when the line is absent or starts with something else

 @example
 ```ts
 valueAfter({ line: 'pid=12', label: 'pid=' });
 ```
 */
function valueAfter({
  line,
  label,
}: {
  readonly line: string;
  readonly label: string;
}): string | typeof notAClaim {
  if (!line.startsWith(label))
    return notAClaim;
  return line.slice(label.length);
}

/**
 The record a claim file's text holds (`single_owner_claim.rs:120`).

 @param text - the text of the file

 @returns the record, or {@link notAClaim} when the text is not a claim of this command

 @example
 ```ts
 decodeClaimRecord('cargo-mutants owner-claim v1\nreleased\n');
 ```
 */
export function decodeClaimRecord(text: string): ClaimRecord | typeof notAClaim {
  /**
   The lines of the text: the header, then either the released line or the owner's id, birth and namespace.
   */
  const [
    header,
    second,
    birthLine,
    namespaceLine
  ] = text.split('\n');
  if (header !== claimHeader)
    return notAClaim;
  if (second === releasedLine)
    return { kind: 'released' };
  /**
   The owner's process id text.
   */
  const pidText = valueAfter({
    line: second ?? '',
    label: 'pid=',
  });
  /**
   The owner's process id.
   */
  const pid = pidText === notAClaim ? notAClaim : numberFromDigits(pidText);
  /**
   The birth text.
   */
  const birth = valueAfter({
    line: birthLine ?? '',
    label: 'birth=',
  });
  /**
   The namespace text.
   */
  const namespace = valueAfter({
    line: namespaceLine ?? '',
    label: 'namespace=',
  });
  if ((pid === notAClaim) || (pid === 0))
    return notAClaim;
  if ((birth === notAClaim) || (birth === '')
    || (namespace === notAClaim))
    return notAClaim;
  return {
    kind: 'owned',
    pid,
    birth,
    ...(namespace === noNamespaceText ? {} : { namespace }),
  };
}

/**
 The path of one generation (`single_owner_claim.rs:156`).

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @param generation - the number

 @returns the path

 @example
 ```ts
 claimPath({ directory: '/d', prefix: 'item.owner', generation: 2 });
 ```
 */
export function claimPath({
  directory,
  prefix,
  generation,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly generation: number;
}): string {
  return join(
    directory,
    `${prefix}.${String(generation)}`,
  );
}

/**
 Every generation of a series in a directory, ascending (`single_owner_claim.rs:148`).

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @returns the numbers of the files named `<prefix>.<digits>`; any other name is not a generation

 @example
 ```ts
 await listGenerations({ directory: '/d', prefix: 'item.owner' });
 ```
 */
export async function listGenerations({
  directory,
  prefix,
}: {
  readonly directory: string;
  readonly prefix: string;
}): Promise<readonly number[]> {
  /**
   Start of every file name of the series.
   */
  const start = `${prefix}.`;
  return (await readdir(directory))
    .filter(function ofSeries(name) {
      return name.startsWith(start);
    })
    .map(function numberOf(name) {
      return numberFromDigits(name.slice(start.length));
    })
    .filter(function isNumber(generation) {
      return generation !== notAClaim;
    })
    .toSorted(function ascending(
      left,
      right,
    ) {
      return left - right;
    });
}
