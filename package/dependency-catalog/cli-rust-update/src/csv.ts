/**
 Streaming CSV row parser for the crates.io database dump.

 The dump's CSV files follow RFC 4180: fields are separated by commas, rows
 by `\n`, and a field that contains a comma, quote, or newline is wrapped in
 double quotes with inner quotes doubled. Descriptions and readmes span many
 lines, so rows are found by a byte state machine in one linear pass, never by
 splitting lines. The parser takes chunks as they arrive from the gunzip
 stream and calls back once per complete row.

 @module
 */

//region Bytes

/**
 Byte value of `"`.
 */
const QUOTE = 0x22;

/**
 Byte value of `,`.
 */
const COMMA = 0x2C;

/**
 Byte value of `\n`.
 */
const LINE_FEED = 0x0A;

/**
 Byte value of `\r`, dropped outside quotes so CRLF rows parse like LF rows.
 */
const CARRIAGE_RETURN = 0x0D;

/**
 Initial capacity of the field buffer; it doubles when a field outgrows it.
 */
const INITIAL_FIELD_CAPACITY = 256;

//endregion Bytes

//region Parser

/**
 Push side of the parser.

 @example
 ```ts
 const parser: CsvParser = createCsvParser({ onRow: (row) => console.log(row), });
 ```
 */
export type CsvParser = {
  /**
   Feeds the next chunk of the file.
   */
  readonly push: (chunk: Uint8Array,) => void;
  /**
   Flushes a final row that has no trailing newline.
   */
  readonly end: () => void;
};

/**
 Mutable parse state, kept in one object so the parser needs no function-level `let`.
 */
type CsvState = {
  field: Uint8Array;
  fieldLength: number;
  row: string[];
  inQuotes: boolean;
  quoteSeen: boolean;
  rowStarted: boolean;
};

/**
 Creates a parser that calls `onRow` with each complete row's fields.

 @param onRow - receives the fields of one row, decoded as UTF-8

 @returns push side of the parser

 @example
 ```ts
 const parser = createCsvParser({ onRow: function show(row) { console.log(row); }, });
 parser.push(new TextEncoder().encode('a,"b\nc"\n'));
 // logs ['a', 'b\nc']
 ```
 */
export function createCsvParser({ onRow, }: { readonly onRow: (row: readonly string[],) => void; },): CsvParser {
  /**
   Decoder for completed fields.
   */
  const decoder = new TextDecoder();
  /**
   Parse state.
   */
  const state: CsvState = {
    field: new Uint8Array(INITIAL_FIELD_CAPACITY,),
    fieldLength: 0,
    row: [],
    inQuotes: false,
    quoteSeen: false,
    rowStarted: false,
  };

  /**
   Appends one byte to the current field, growing the buffer when full.

   @param byte - byte to append

   @example
   ```ts
   appendByte(0x61);
   ```
   */
  function appendByte(byte: number,): void {
    if (state.fieldLength
      === state.field
        .length) {
      /**
       Larger buffer holding the bytes so far.
       */
      const grown = new Uint8Array(state.field
        .length
        * 2,);
      grown.set(state.field,);
      state.field = grown;
    }
    state.field[state.fieldLength] = byte;
    state.fieldLength += 1;
    state.rowStarted = true;
  }

  /**
   Ends the current field.

   @example
   ```ts
   endField();
   ```
   */
  function endField(): void {
    state.row
      .push(decoder.decode(state.field
        .subarray(
        0,
        state.fieldLength,
        ),),);
    state.fieldLength = 0;
  }

  /**
   Ends the current row and reports it.

   @example
   ```ts
   endRow();
   ```
   */
  function endRow(): void {
    endField();
    onRow(state.row,);
    state.row = [];
    state.rowStarted = false;
  }

  /**
   Handles one byte inside a quoted field.

   @param byte - next byte

   @returns whether the byte was consumed; false means it closes the quote and must be re-read unquoted

   @example
   ```ts
   quotedByte(0x22);
   ```
   */
  function quotedByte(byte: number,): boolean {
    if (state.quoteSeen) {
      state.quoteSeen = false;
      if (byte === QUOTE) {
        appendByte(QUOTE,);
        return true;
      }
      state.inQuotes = false;
      return false;
    }
    if (byte === QUOTE) {
      state.quoteSeen = true;
      return true;
    }
    appendByte(byte,);
    return true;
  }

  /**
   Handles one byte outside quotes.

   @param byte - next byte

   @example
   ```ts
   unquotedByte(0x2C);
   ```
   */
  function unquotedByte(byte: number,): void {
    if ((byte === QUOTE) && (state.fieldLength === 0)) {
      state.inQuotes = true;
      state.rowStarted = true;
      return;
    }
    if (byte === COMMA) {
      state.rowStarted = true;
      endField();
      return;
    }
    if (byte === LINE_FEED) {
      endRow();
      return;
    }
    if (byte !== CARRIAGE_RETURN)
      appendByte(byte,);
  }

  return {
    push: function push(chunk: Uint8Array,): void {
      for (const byte of chunk) {
        if (state.inQuotes && quotedByte(byte,))
          continue;
        unquotedByte(byte,);
      }
    },
    end: function end(): void {
      if (state.inQuotes && state.quoteSeen)
        state.inQuotes = false;
      if (state.rowStarted || (state.fieldLength > 0)
        || (state.row
          .length
          > 0))
        endRow();
    },
  };
}

/**
 Column positions of a CSV header, looked up by name because the dump's column order can change.

 @param header - first row of the file

 @param columns - names the caller reads

 @returns column name to position

 @throws DumpFormatError when a column is missing

 @example
 ```ts
 columnIndexes({ header: ['id', 'name',], columns: ['name',], }).get('name'); // 1
 ```
 */
export function columnIndexes(
  {
    header,
    columns,
  }: {
    readonly header: readonly string[];
    readonly columns: readonly string[]
  },
): ReadonlyMap<string, number> {
  /**
   Positions found.
   */
  const indexes = new Map<string, number>();
  for (const column of columns) {
    /**
     Position of this column.
     */
    const index = header.indexOf(column,);
    if (index === (-1))
      throw new DumpFormatError(`the crates.io database dump has no "${column}" column (header: ${header.join(', ',)}); its schema changed, so update package/dependency-catalog/cli-rust-update/src/dump.ts`,);
    indexes.set(
      column,
      index,
    );
  }
  return indexes;
}

//endregion Parser

//region Errors

/**
 Thrown when the crates.io database dump does not have the layout the reader expects.

 @example
 ```ts
 throw new DumpFormatError('no "num" column');
 ```
 */
export class DumpFormatError extends Error {
  /**
   Builds the error.

   @param message - what differs from the expected layout
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'DumpFormatError';
  }
}

//endregion Errors
