import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  SLICE_COST_EXITS,
  SLICE_COST_LANES,
  SLICE_COST_MARKER,
  type SliceCostExit,
  type SliceCostLane,
} from './slice-cost-log.ts';

//region Slice cost read
// Reads back what `armSliceCost` wrote, so a pass's log answers whether a
// slice's cost scales with its size.
//
// REFUSES RATHER THAN GUESSES, and says why. A log is written while a pass runs,
// so its last line can be half-written, and a reader that silently skipped
// malformed lines would report a smaller corpus without saying so. Every line
// carrying the marker is either a row or a named refusal.
//
// NO REGEX: the grammar is `key=value` separated by single spaces, which an
// index scan and two splits express directly, in one linear pass over each line.

/**
 One slice's measured cost.

 @example
 ```ts
 const row: SliceCostRow = { lane: 'repair', sliceIndex: 3, sourceChars: 812, elapsedMs: 45210, };
 ```
 */
export type SliceCostRow = {
  /**
   Lane that paid.
   */
  readonly lane: SliceCostLane;

  /**
   Slice this measures.
   */
  readonly sliceIndex: number;

  /**
   Size of what was translated.
   */
  readonly sourceChars: number;

  /**
   Real time the slice took, read on the writer's monotonic clock.
   */
  readonly elapsedMs: number;

  /**
   How the lane left this slice, which says whether its time prices work.
   */
  readonly exit: SliceCostExit;
};

/**
 Everything one log said about slice cost, refusals kept beside rows.

 @example
 ```ts
 const reading: SliceCostReading = readSliceCosts({ log, },);
 ```
 */
export type SliceCostReading = {
  /**
   Lines that parsed, in the order the log carried them.
   */
  readonly rows: readonly SliceCostRow[];

  /**
   Why a line carrying the marker produced no row.
   */
  readonly dropped: readonly string[];
};

/**
 What one marker-bearing line carries after the marker.

 @example
 ```ts
 const { fields, repeated, }: LineFields = fieldsOf({ line, },);
 ```
 */
type LineFields = {
  /**
   Pairs read off the line, each name with the first value written under it.
   */
  readonly fields: ReadonlyMap<string, string>;

  /**
   Names the line writes more than once, each listed once, in the order the
   repeats stand on the line.
   */
  readonly repeated: ReadonlySet<string>;
};

/**
 Splits one marker-bearing line into its `key=value` pairs.

 A NAME WRITTEN TWICE IS REPORTED, NOT SETTLED BY ORDER. `armSliceCost` writes
 each field once, so a line naming one twice is two lines run together or a
 line cut and resumed. Setting the name again kept the later value, which
 built a row out of two slices' fields and reported a cost no slice had.

 @param line - whole log line, including whatever the logger prefixed

 @returns Pairs found after the marker, beside every name written more than once

 @example
 ```ts
 const { fields, repeated, } = fieldsOf({ line, },);
 ```
 */
function fieldsOf({ line, }: { readonly line: string; },): LineFields {
  /**
   Where the cost report starts, past anything the logger put in front.
   */
  const start = line.indexOf(SLICE_COST_MARKER,);

  /**
   Report body, with the marker itself dropped.
   */
  const body = line.slice(start + SLICE_COST_MARKER.length,);

  /**
   Pairs read so far.
   */
  const fields = new Map<string, string>();

  /**
   Names met a second time so far.
   */
  const repeated = new Set<string>();
  for (const token of body.split(' ',)) {
    /**
     Where this token separates its name from its value.
     */
    const split = token.indexOf('=',);
    if (split <= 0)
      continue;

    /**
     Name this token is written under.
     */
    const name = token.slice(
      0,
      split,
    );
    if (fields.has(name,)) {
      repeated.add(name,);
      continue;
    }

    fields.set(
      name,
      token.slice(split + 1,),
    );
  }

  return {
    fields,
    repeated,
  };
}

/**
 What one field of a cost line reads as: the value it carries, or the words
 naming why it carries none.

 DISCRIMINATED rather than a nullish union, because zero is an ordinary
 answer here: a slice can genuinely cost 0 ms, and a sentinel would make that
 indistinguishable from a field the log never carried.

 @example
 ```ts
 const reading: FieldReading<number> = { kind: 'read', value: 45_210, };
 ```
 */
type FieldReading<ValueT,> = {
  /**
   Field carries a value this reader accepts.
   */
  readonly kind: 'read';

  /**
   Value it carries.
   */
  readonly value: ValueT;
} | {
  /**
   Field is absent, or carries something this reader does not accept.
   */
  readonly kind: 'refused';

  /**
   Field name and what it carried, as a dropped line's reason lists it.
   */
  readonly reason: string;
};

/**
 Words a field whose text this reader does not accept.

 @param name - field refused

 @param raw - text the line carried for it

 @returns Field name and its text, with an empty text said as `empty` so the
 reason does not end in a bare space

 @example
 ```ts
 spelledAs({ name: 'ms', raw: '45e', },); // 'ms 45e'
 ```
 */
function spelledAs(
  {
    name,
    raw,
  }: {
    readonly name: string;
    readonly raw: string;
  },
): string {
  return (raw === '') ? `${name} empty` : `${name} ${raw}`;
}

/**
 Reads a field that carries one of a fixed set of values.

 @param fields - pairs read off one line

 @param name - field to read

 @param allowed - values it may carry, read from the writer's own list so a
 lane or exit added there is accepted here without a second edit

 @returns Member it carries, or the refusal naming what it carried instead

 @example
 ```ts
 const lane = memberField({ fields, name: 'lane', allowed: SLICE_COST_LANES, },);
 ```
 */
function memberField<const MemberT extends string,>(
  {
    fields,
    name,
    allowed,
  }: {
    readonly fields: ReadonlyMap<string, string>;
    readonly name: string;
    readonly allowed: readonly MemberT[];
  },
): FieldReading<MemberT> {
  /**
   Text the line carried for this field, absent when it named none.
   */
  const raw = fields.get(name,);
  if (raw === undefined) {
    return {
      kind: 'refused',
      reason: `${name} missing`,
    };
  }

  /**
   Member that text spells, absent when it spells none.
   */
  const found = allowed.find(function matches(member,): boolean {
    return member === raw;
  },);
  if (found === undefined) {
    return {
      kind: 'refused',
      reason: spelledAs({
        name,
        raw,
      },),
    };
  }
  return {
    kind: 'read',
    value: found,
  };
}

/**
 Reads a field that carries a count: a slice index, a character count or a
 duration.

 PLAIN DECIMAL DIGITS ONLY, which is all `armSliceCost` writes. This read
 `Number(raw)` and asked whether that was an integer, and `Number` also reads
 an empty text as 0, `0x1F` as 31, `1e3` as 1000 and a leading sign, so a
 field nobody wrote as a count became one (ledger B71). A digit run past the
 largest integer a double holds exactly is refused too, since it would read as
 a neighbouring number. The rule is the package's one count rule
 (`whole-number-text.ts`, ledger B73).

 @param fields - pairs read off one line

 @param name - field to read

 @returns Count it carries, or the refusal naming what it carried instead

 @example
 ```ts
 const ms = countField({ fields, name: 'ms', },);
 ```
 */
function countField(
  {
    fields,
    name,
  }: {
    readonly fields: ReadonlyMap<string, string>;
    readonly name: string;
  },
): FieldReading<number> {
  /**
   Text the line carried for this field, absent when it named none.
   */
  const raw = fields.get(name,);
  if (raw === undefined) {
    return {
      kind: 'refused',
      reason: `${name} missing`,
    };
  }

  if (!isWholeNumberText({ text: raw, },)) {
    return {
      kind: 'refused',
      reason: spelledAs({
        name,
        raw,
      },),
    };
  }
  return {
    kind: 'read',
    value: Number(raw,),
  };
}

/**
 What one marker-bearing line yielded: a row, or the reason it yielded none.

 @example
 ```ts
 const read: LineReading = readLine({ fields, },);
 ```
 */
type LineReading = {
  /**
   Line parsed.
   */
  readonly kind: 'row';

  /**
   What it said.
   */
  readonly row: SliceCostRow;
} | {
  /**
   Line carried the marker and no usable measurement.
   */
  readonly kind: 'dropped';

  /**
   Which fields failed, and how.
   */
  readonly reason: string;
};

/**
 Turns one line's fields into a row, or says why they are not one.

 VALIDATES AND BUILDS TOGETHER: each field is read once, into its value or its
 refusal, and the row is built from those readings. This used to check every
 field in one loop and read each again through a helper that threw when the
 loop had not checked it, a throw no line could reach.

 @param fields - pairs read off one line

 @returns Row, or the named refusal

 @example
 ```ts
 const read = readLine({ fields, },);
 ```
 */
function readLine(
  { fields, }: { readonly fields: ReadonlyMap<string, string>; },
): LineReading {
  // IN THE ORDER THE WRITER WRITES THEM, so a dropped line names its fields in
  // the order they appear on it.
  /**
   Lane that paid.
   */
  const lane = memberField({
    fields,
    name: 'lane',
    allowed: SLICE_COST_LANES,
  },);

  /**
   Slice measured.
   */
  const sliceIndex = countField({
    fields,
    name: 'chunk',
  },);

  /**
   Size of what was translated.
   */
  const sourceChars = countField({
    fields,
    name: 'sourceChars',
  },);

  /**
   Real time the slice took, read on the writer's monotonic clock.
   */
  const elapsedMs = countField({
    fields,
    name: 'ms',
  },);

  // `exit` IS REQUIRED rather than optional, though it was added after the
  // rest. No production log carries the older shape: the telemetry landed after
  // the only pass that had run, so there are no legacy lines to stay compatible
  // with, and an optional field would mean inventing an exit for a line that
  // named none.
  /**
   How the lane left the slice.
   */
  const exit = memberField({
    fields,
    name: 'exit',
    allowed: SLICE_COST_EXITS,
  },);
  if ((lane.kind === 'read')
    && (sliceIndex.kind === 'read')
    && (sourceChars.kind === 'read')
    && (elapsedMs.kind === 'read')
    && (exit.kind === 'read')) {
    return {
      kind: 'row',
      row: {
        lane: lane.value,
        exit: exit.value,
        sliceIndex: sliceIndex.value,
        sourceChars: sourceChars.value,
        elapsedMs: elapsedMs.value,
      },
    };
  }
  return {
    kind: 'dropped',
    reason: [
      lane,
      sliceIndex,
      sourceChars,
      elapsedMs,
      exit,
    ]
      .flatMap(function refusal(reading: FieldReading<unknown>,): readonly string[] {
        return (reading.kind === 'refused') ? [reading.reason,] : [];
      },)
      .join(', ',),
  };
}

/**
 Reads every slice cost a log reported.

 @param log - whole log text, of any length, including lines about other things

 @returns Rows in log order, beside a named refusal for every marker-bearing
 line that produced none

 @example
 ```ts
 const { rows, dropped, } = readSliceCosts({ log: await readFile(path, 'utf8',), },);
 ```
 */
export function readSliceCosts({ log, }: { readonly log: string; },): SliceCostReading {
  /**
   Rows built so far.
   */
  const rows: SliceCostRow[] = [];

  /**
   Refusals collected so far.
   */
  const dropped: string[] = [];
  for (const line of log.split('\n',)) {
    if (!line.includes(SLICE_COST_MARKER,))
      continue;

    /**
     Pairs the line carries, beside every name it writes more than once.
     */
    const {
      fields,
      repeated,
    } = fieldsOf({ line, },);
    if (repeated.size > 0) {
      /**
       Each repeated name, said as a dropped line's reason lists a field.
       */
      const reasons = [...repeated,].map(function writtenTwice(name,): string {
        return `${name} written more than once`;
      },);
      dropped.push(reasons.join(', ',),);
      continue;
    }

    /**
     What this line yielded.
     */
    const read = readLine({ fields, },);
    if (read.kind === 'dropped') {
      dropped.push(read.reason,);
      continue;
    }

    rows.push(read.row,);
  }

  return {
    rows,
    dropped,
  };
}

//endregion Slice cost read
