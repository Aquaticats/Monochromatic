/**
 Splits Pi's output into lines. A chunk can end inside a record or inside a character, so the pieces of an unfinished line are
 joined once, when it ends.

 @module
 */

/**
 The byte that ends a record. Records are split on it alone, because Unicode line separators are valid inside JSON strings.
 */
const LINE_FEED = 0x0A;

/**
 What a splitter hands each line to.

 @param line - one whole line, without its line feed
 */
export type SplitPiOutputRequest = {
  readonly onLine: (line: string,) => void;
};

/**
 Finds where each line in a chunk ends.

 @param chunk - bytes as they arrived

 @returns the offset of each line feed, one at a time

 @example
 ```ts
 const ends = [...lineEndsIn(Buffer.from('a\nb\n',),),];
 ```
 */
function* lineEndsIn(chunk: Buffer,): Generator<number> {
  for (let lineEnd = chunk.indexOf(LINE_FEED,); lineEnd !== (-1); lineEnd = chunk.indexOf(
    LINE_FEED,
    lineEnd + 1,
  )) {
    yield lineEnd;
  }
}

/**
 Starts a splitter for Pi's output.

 @param onLine - where each whole line goes

 @returns what to hand every chunk as it arrives

 @example
 ```ts
 const split = startLineSplitter({ onLine: readLine, },);
 child.stdout.on('data', split,);
 ```
 */
export function startLineSplitter({ onLine, }: SplitPiOutputRequest,): (chunk: Buffer,) => void {
  /**
   What the splitter carries from one chunk to the next: the bytes of an unfinished line, which end a later chunk.
   */
  const carried: { partialLine: Buffer[] } = { partialLine: [], };
  return function splitChunk(chunk: Buffer,): void {
    /**
     Where the part after the last whole line starts.
     */
    const tailStart = [...lineEndsIn(chunk,),].reduce(
      function readOneLine(
        lineStart,
        lineEnd,
      ) {
        onLine(Buffer.concat([
          ...carried.partialLine,
          chunk.subarray(
            lineStart,
            lineEnd,
          ),
        ],)
          .toString('utf8',),);
        carried.partialLine = [];
        return lineEnd + 1;
      },
      0,
    );
    if (tailStart < chunk.length) {
      carried.partialLine
        .push(chunk.subarray(tailStart,),);
    }
  };
}
