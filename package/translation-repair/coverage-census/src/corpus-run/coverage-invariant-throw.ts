import { atomAt, } from './coverage-stretch-atoms.ts';

//region Coverage invariant throw
// Ledger T8: whether one cold stretch is nothing but invariant throws, read
// from the stretch's own text in the coverage bundle. A broken invariant
// throws here (`throw new Error('unreachable: ...',)`, or a class named for
// it, such as `DeliveryInvariantError`), and by definition no honest input
// reaches such a throw, so each one no test drives reads as code no test ran
// for ever. Counted among the cold stretches, it is a standing reason to
// delete the guard (ledger M112 recorded exactly that as its prevention), so
// the census counts such a stretch apart.
//
// The reading is strict, because the bucket must never hide cold code. A
// stretch qualifies only where every statement in it is a throw of
// `new Error(...)` whose first argument is one string or template literal
// beginning `unreachable:`, or of `new` a class whose name ends in
// `InvariantError`. Around those statements it may hold only what V8 folds
// into a stretch without a statement of its own: braces, a semicolon, `else`,
// and the `if (...)` head guarding a throw. Anything else, a call beside the
// throw, a `catch` head, a message joined with `+`, keeps the whole stretch
// cold.
//
// The class is read off its name, so no list of invariant classes is kept
// here to go stale: `RenderingAuditInvariantError` and
// `DeliveryInvariantError` qualify today, and a class added under the same
// naming qualifies at once. A class that reports a broken invariant under
// another name stays cold until it is renamed, which errs toward cold.
//
// It reads the bundle's text, not the source lines. The offsets are what V8
// counted, so the text between them is exactly the code no test ran. A
// stretch's source lines run from the lowest line any of its characters maps
// to to the highest, and blank space before a statement maps to the statement
// before it: the stretch that is one throw in `src/artifact-read.ts` reads
// lines 138 to 150 there, which open with the closing lines of a return that
// ran. Read by line, a strict reading would refuse that throw, and a lenient
// one would have to accept fragments of statements. What the bundle's text
// can get wrong is that it is generated code: the bundler writes a guard's
// braces away, folds a message joined from literals into one literal, and
// keeps TSDoc comments, so this reads JavaScript as the unminified coverage
// build writes it and skips comments, which hold no statement.
//
// It is one linear pass over the tokens `coverage-stretch-atoms.ts` reads,
// no parser and no pattern. Where that reading cannot be certain, a slash
// that opens no comment, or a string, template, comment or bracket that never
// closes, the stretch stays cold.

/**
 What a broken invariant's message begins with in this package.
 */
const INVARIANT_MESSAGE_START = 'unreachable:';

/**
 What the name of a class that only ever reports a broken invariant ends
 with.
 */
const INVARIANT_CLASS_END = 'InvariantError';

/**
 Marks a stretch may hold beside its throws: braces and the semicolon, which
 carry no code.
 */
const STRUCTURE_MARKS: ReadonlySet<string> = new Set([
  '{',
  '}',
  ';',
],);

/**
 Marks that may follow an invariant message: the comma before the next
 argument, or the bracket closing the arguments. Anything else continues the
 first argument, which is then more than one literal.
 */
const ARGUMENT_ENDS: ReadonlySet<string> = new Set([
  ',',
  ')',
],);

/**
 What the census reads of one cold stretch's text.

 @example
 ```ts
 const reading: StretchReading = { kind: 'invariant throws', thrown: ['Error',], };
 ```
 */
export type StretchReading = { readonly kind: 'cold'; } | {
  readonly kind: 'invariant throws';

  /**
   Class each throw of the stretch builds, in the order the stretch holds
   them: `Error` for a message beginning `unreachable:`, or the class named
   for a broken invariant.
   */
  readonly thrown: readonly string[];
};

/**
 One thing a stretch holds at its own level: structure carrying no code, an
 `if` head, an invariant throw, the end of the text, or anything else.
 */
type StretchItem = {
  readonly kind: 'guard' | 'structure';

  /**
   One past its last character.
   */
  readonly end: number;
} | {
  readonly kind: 'invariant throw';

  /**
   Class it builds.
   */
  readonly thrown: string;

  /**
   One past the bracket closing its arguments.
   */
  readonly end: number;
} | { readonly kind: 'end'; } | { readonly kind: 'other'; };

/**
 The reading of a stretch that is cold code.
 */
const COLD: StretchReading = { kind: 'cold', };

/**
 The reading of an item that is neither structure nor an invariant throw.
 */
const OTHER: StretchItem = { kind: 'other', };

/**
 Whether the arguments of `new Error(...)` open with an invariant message:
 one string or template literal beginning `unreachable:`, with nothing
 joined to it. A message joined with `+` is refused, since one pass cannot
 tell what the join binds to; the coverage build writes a message joined
 from literals as one literal.

 @param text - stretch text

 @param from - offset one past the bracket opening the arguments

 @returns Whether the first argument is such a literal alone

 @example
 ```ts
 opensWithInvariantMessage({ text: '("unreachable: no lap", { cause })', from: 1, },); // true
 ```
 */
function opensWithInvariantMessage(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): boolean {
  /**
   The first token of the arguments.
   */
  const message = atomAt({
    text,
    from,
  },);
  if ((message.kind !== 'text') || (!text.startsWith(
    INVARIANT_MESSAGE_START,
    message.start + 1,
  )))
    return false;
  /**
   The token after the literal.
   */
  const next = atomAt({
    text,
    from: message.end,
  },);
  return (next.kind === 'mark') && ARGUMENT_ENDS.has(text.charAt(next.start,),);
}

/**
 Reads what follows the word `throw` as an invariant throw: `new`, a bare
 class name and its arguments, the class `Error` with an invariant message
 or one whose name ends in `InvariantError`.

 @param text - stretch text

 @param from - offset one past the word `throw`

 @returns The class thrown and where its arguments close, or that what is
 thrown is anything else

 @example
 ```ts
 invariantThrowAt({ text: 'throw new NapInvariantError({ lap })', from: 5, },); // { kind: 'invariant throw', thrown: 'NapInvariantError', end: 36, }
 ```
 */
function invariantThrowAt(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): StretchItem {
  /**
   The word `new`, when the thrown value is built there.
   */
  const made = atomAt({
    text,
    from,
  },);
  if ((made.kind !== 'word') || (text.slice(
    made.start,
    made.end,
  ) !== 'new'))
    return OTHER;
  /**
   The class built, a bare name.
   */
  const built = atomAt({
    text,
    from: made.end,
  },);
  if (built.kind !== 'word')
    return OTHER;
  /**
   The arguments it is handed, bracketed.
   */
  const handed = atomAt({
    text,
    from: built.end,
  },);
  if (handed.kind !== 'group')
    return OTHER;
  /**
   The class's name.
   */
  const thrown = text.slice(
    built.start,
    built.end,
  );
  /**
   Whether the throw reports a broken invariant.
   */
  const invariant = (thrown === 'Error')
    ? opensWithInvariantMessage({
      text,
      from: handed.start + 1,
    },)
    : thrown.endsWith(INVARIANT_CLASS_END,);
  return invariant
    ? {
      kind: 'invariant throw',
      thrown,
      end: handed.end,
    }
    : OTHER;
}

/**
 Reads the next thing a stretch holds at its own level.

 @param text - stretch text

 @param from - offset to read from

 @returns Structure carrying no code, an `if` head, an invariant throw, the
 end of the text, or anything else

 @example
 ```ts
 itemAt({ text: 'if (lap === void 0) throw', from: 0, },); // { kind: 'guard', end: 19, }
 ```
 */
function itemAt(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): StretchItem {
  /**
   The token there.
   */
  const atom = atomAt({
    text,
    from,
  },);
  if (atom.kind === 'end')
    return atom;
  if ((atom.kind !== 'mark') && (atom.kind !== 'word'))
    return OTHER;
  /**
   The token as written.
   */
  const written = text.slice(
    atom.start,
    atom.end,
  );
  if (atom.kind === 'mark')
    return STRUCTURE_MARKS.has(written,)
      ? {
        kind: 'structure',
        end: atom.end,
      }
      : OTHER;
  if (written === 'else')
    return {
      kind: 'structure',
      end: atom.end,
    };
  if (written === 'throw')
    return invariantThrowAt({
      text,
      from: atom.end,
    },);
  if (written !== 'if')
    return OTHER;
  /**
   The condition after `if`, bracketed.
   */
  const head = atomAt({
    text,
    from: atom.end,
  },);
  return (head.kind === 'group')
    ? {
      kind: 'guard',
      end: head.end,
    }
    : OTHER;
}

/**
 Everything a stretch holds at its own level, in order, up to its end or to
 the first thing that is neither structure nor an invariant throw, which is
 yielded last.

 @param text - stretch text

 @returns Each item, none of them the end

 @example
 ```ts
 const items = [...itemsOf({ text: 'throw new Error("unreachable: no lap");', },),];
 ```
 */
function* itemsOf({ text, }: { readonly text: string; },): Generator<StretchItem> {
  /**
   Offset read so far.
   */
  const cursor = { at: 0, };
  while (cursor.at < text.length) {
    /**
     The next thing the stretch holds.
     */
    const item = itemAt({
      text,
      from: cursor.at,
    },);
    if (item.kind === 'end')
      return;
    yield item;
    if (item.kind === 'other')
      return;
    cursor.at = item.end;
  }
}

/**
 Reads one cold stretch's text as invariant throws or as cold code.

 @param text - bundle text between the stretch's offsets, as the unminified
 coverage build wrote it

 @returns The classes thrown where every statement of the stretch is an
 invariant throw, each `if` head in it standing before one; cold for
 anything else, a stretch holding no throw among them

 @example
 ```ts
 stretchReadingOf({ text: 'throw new Error("unreachable: the cat always lands");', },); // { kind: 'invariant throws', thrown: ['Error',], }
 ```
 */
export function stretchReadingOf({ text, }: { readonly text: string; },): StretchReading {
  /**
   Everything the stretch holds at its own level.
   */
  const items = [...itemsOf({ text, },),];
  /**
   Class each invariant throw of the stretch builds.
   */
  const thrown = items.flatMap(function classOf(item,): readonly string[] {
    return (item.kind === 'invariant throw') ? [item.thrown,] : [];
  },);
  /**
   Whether the stretch holds anything but structure, `if` heads and
   invariant throws.
   */
  const holdsOther = items.some(function isOther(item,): boolean {
    return item.kind === 'other';
  },);
  /**
   Whether an `if` head stands after the last throw, guarding code outside
   the stretch. The search answers -1 where the stretch holds none.
   */
  const guardsOutside = items.findLastIndex(function isGuard(item,): boolean {
    return item.kind === 'guard';
  },) > items.findLastIndex(function isThrow(item,): boolean {
    return item.kind === 'invariant throw';
  },);
  if (holdsOther || guardsOutside)
    return COLD;
  // A stretch of structure alone, a brace or an `else`, throws nothing.
  return (thrown.length === 0)
    ? COLD
    : {
      kind: 'invariant throws',
      thrown,
    };
}

//endregion Coverage invariant throw
