import { unified, } from 'unified';

/**
 The message the engine gives a stack exhaustion.
 */
export const ENGINE_EXHAUSTION_MESSAGE = 'Maximum call stack size exceeded';

/**
 The part of a processor the grammars call, replaceable for a test.
 */
type ParserHolder = {
  parse: (file: unknown,) => unknown;
};

/**
 Which parses a replacement takes over.
 */
type ParseSelection = {
  /**
   How many parses go to the real parser first, none by default.
   */
  readonly afterParses?: number;

  /**
   Whether a parse of this input is taken over, every one by default.
   */
  readonly appliesTo?: (input: unknown,) => boolean;
};

/**
 Whether a value holds a `parse` method.

 @param value - the value under test

 @returns Whether `parse` is a function on it

 @example
 ```ts
 const holds = isParserHolder({ parse: parseIt, },); // true
 ```
 */
function isParserHolder(value: unknown,): value is ParserHolder {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('parse' in value)
    && ((typeof value.parse) === 'function');
}

/**
 The prototype every processor the grammars build shares, whose `parse` both grammars call.
 */
const PROCESSOR_PROTOTYPE: ParserHolder = (function prototypeOfProcessors(): ParserHolder {
  /**
   The prototype of a processor.
   */
  const prototype: unknown = Object.getPrototypeOf(unified(),);
  if (!isParserHolder(prototype,))
    throw new Error('unreachable: a processor shares a prototype with a parse method',);
  return prototype;
})();

/**
 A stack exhaustion as the engine raises it.

 @returns A fresh `RangeError` with the engine's message

 @example
 ```ts
 const error = engineExhaustion(); // RangeError: Maximum call stack size exceeded
 ```
 */
export function engineExhaustion(): RangeError {
  return new RangeError(ENGINE_EXHAUSTION_MESSAGE,);
}

/**
 A failure of the parser whose own refusal cannot be built: reading its message raises the engine's stack exhaustion, as
 building the refusal does with the stack nearly spent, so the exhaustion leaves the grammar's function raw.

 @returns A `RangeError` whose message reads as a stack exhaustion

 @example
 ```ts
 const error = parserFailureWithoutRoomForRefusal();
 ```
 */
function parserFailureWithoutRoomForRefusal(): RangeError {
  /**
   The failure, whose message is made to raise.
   */
  const failure = new RangeError('the parser stopped',);
  Object.defineProperty(
    failure,
    'message',
    {
      get(): never {
        throw engineExhaustion();
      },
    },
  );
  return failure;
}

/**
 A parse that fails with no room to build the refusal.

 @throws The parser's failure, whose message cannot be read without a stack exhaustion

 @example
 ```ts
 raisesFromParse();
 ```
 */
function raisesFromParse(): never {
  throw parserFailureWithoutRoomForRefusal();
}

/**
 Names of the grammar's own functions. Each one reads the tree it has just built, to count what the parser dropped and
 to note where a setext heading's own lines start, before it returns, and those reads are part of the parse: an
 exhaustion they raise is the parse's, which the boundaries read as the refusal.
 */
const GRAMMAR_FUNCTION_NAMES: readonly string[] = [
  'parseMdxBody',
  'parseMarkdownBody',
];

/**
 Whether the read being made is one the grammar's own function makes of the tree it has just built.

 @returns Whether a frame of the call stack is a grammar function from the list of names

 @example
 ```ts
 const own = readsFromTheGrammar(); // false in a test body, true inside parseMdxBody
 ```
 */
function readsFromTheGrammar(): boolean {
  /**
   How many frames a stack keeps, restored once the stack is read.
   */
  const keptFrames = Error.stackTraceLimit;
  Error.stackTraceLimit = Number.POSITIVE_INFINITY;
  /**
   The stack of the read.
   */
  const stack = new Error('stack of the read',).stack ?? '';
  Error.stackTraceLimit = keptFrames;
  return GRAMMAR_FUNCTION_NAMES.some(function isOnStack(name,): boolean {
    // The bundler may suffix a name it had to make unique, as `parseMdxBody$1`.
    return stack.includes(`at ${name} `,) || stack.includes(`at ${name}$`,);
  },);
}

/**
 A tree that raises a stack exhaustion whenever the code that walks a parse after the parse call returned reads it, and
 answers the grammar's own reads from the tree the parser built, so the exhaustion comes from the boundary's code and not
 from the grammar's.

 @param built - the tree the real parser built

 @returns A stand-in for a parsed root

 @example
 ```ts
 const root = treeThatExhaustsTheStack({ built: realRoot, },);
 ```
 */
function treeThatExhaustsTheStack({ built, }: { readonly built: object; },): object {
  return new Proxy(
    built,
    {
      get(
        target,
        key,
      ): unknown {
        if (readsFromTheGrammar())
          return Reflect.get(
            target,
            key,
          );
        throw engineExhaustion();
      },
      has(
        target,
        key,
      ): boolean {
        if (readsFromTheGrammar())
          return Reflect.has(
            target,
            key,
          );
        throw engineExhaustion();
      },
      ownKeys(target,): (string | symbol)[] {
        if (readsFromTheGrammar())
          return Reflect.ownKeys(target,);
        throw engineExhaustion();
      },
      getOwnPropertyDescriptor(
        target,
        key,
      ) {
        if (readsFromTheGrammar())
          return Reflect.getOwnPropertyDescriptor(
            target,
            key,
          );
        throw engineExhaustion();
      },
    },
  );
}

/**
 A parse that returns the tree that raises at the first read the boundary makes.

 @param built - runs the real parser and gives the tree it built

 @returns The tree

 @example
 ```ts
 const root = returnsExhaustingTree({ built: function build() { return realRoot; }, },);
 ```
 */
function returnsExhaustingTree({ built, }: { readonly built: () => unknown; },): object {
  /**
   The tree the real parser built.
   */
  const tree = built();
  if (((typeof tree) !== 'object') || (tree === null))
    throw new Error('unreachable: the parser returns a tree, which is an object',);
  return treeThatExhaustsTheStack({ built: tree, },);
}

/**
 Replaces the parser for the parses a selection names, until the returned value is disposed of.

 @param replacement - what a taken-over parse does, given a way to run the real parser

 @param selection - which parses are taken over

 @returns A disposable that puts the parser back

 @example
 ```ts
 using replaced = parserReplaced({ replacement: raisesFromParse, selection: {}, },);
 ```
 */
function parserReplaced(
  {
    replacement,
    selection,
  }: {
    readonly replacement: (arguments_: { readonly built: () => unknown; },) => unknown;
    readonly selection: ParseSelection;
  },
): Disposable {
  /**
   The parser as it was.
   */
  const original = PROCESSOR_PROTOTYPE.parse;
  /**
   How many parses the call has made so far, mutated only by the replacement.
   */
  const made = { count: 0, };
  PROCESSOR_PROTOTYPE.parse = function replacedParse(
    this: unknown,
    file: unknown,
  ): unknown {
    made.count += 1;
    if ((made.count <= (selection.afterParses ?? 0)) || (!(selection.appliesTo?.(file,) ?? true)))
      return original.call(
        this,
        file,
      );
    return replacement({
      built: original.bind(
        this,
        file,
      ),
    },);
  };
  return {
    [Symbol.dispose]: function putParserBack(): void {
      PROCESSOR_PROTOTYPE.parse = original;
    },
  };
}

/**
 Runs a call while the parse it makes lets the engine's stack exhaustion out of the grammar's function raw, the case a stack
 too short to build the refusal produces. The call is synchronous and runs to its end before any other test step, so
 nothing else sees the replacement.

 @param run - the call under test

 @param selection - which parses raise, every one by default

 @returns What the call returns

 @example
 ```ts
 const refused = withExhaustionFromParse({ run: function read(): boolean { return isLineStructured({ text: 'a', },); }, },);
 ```
 */
export function withExhaustionFromParse<Result>(
  {
    run,
    selection = {},
  }: {
    readonly run: () => Result;
    readonly selection?: ParseSelection;
  },
): Result {
  /**
   The replacement, in force until this function returns.
   */
  using _replaced = parserReplaced({
    replacement: raisesFromParse,
    selection,
  },);
  return run();
}

/**
 Runs a call while the parse it makes returns a tree that raises the engine's stack exhaustion at the first read the code
 under test makes, so the exhaustion comes from the code that reads the parse and not from the parse. The reads the
 grammar's own function makes of the tree it has just built (`parseMdxBody` and `parseMarkdownBody` count the dropped
 characters and note the setext heading floors) are answered from the real tree, since they are part of the parse. The
 call is synchronous and runs to its end before any other test step.

 @param run - the call under test

 @param selection - which parses return the tree, every one by default

 @returns What the call returns

 @example
 ```ts
 const walked = withExhaustionFromWalk({ run: function read(): number { return blockBreaksIn({ text: 'a\n\nb', },); }, },);
 ```
 */
export function withExhaustionFromWalk<Result>(
  {
    run,
    selection = {},
  }: {
    readonly run: () => Result;
    readonly selection?: ParseSelection;
  },
): Result {
  /**
   The replacement, in force until this function returns.
   */
  using _replaced = parserReplaced({
    replacement: returnsExhaustingTree,
    selection,
  },);
  return run();
}

/**
 A method of a built-in object, as a test replaces it.
 */
type Method = (
  this: unknown,
  first: unknown,
  second: unknown,
) => unknown;

/**
 Whether a value is a function.

 @param value - the value under test

 @returns Whether it can be called

 @example
 ```ts
 const callable = isMethod(Array.prototype.join,); // true
 ```
 */
function isMethod(value: unknown,): value is Method {
  return (typeof value) === 'function';
}

/**
 Runs a call while one method of a built-in object raises the engine's stack exhaustion for the calls a test names, so code
 that runs after the parse and reads no tree can be made to raise. The call is synchronous and the method is put back
 when it ends.

 @param owner - the object holding the method, such as `Array.prototype`

 @param method - the method's name

 @param applies - whether a call of the method, given its receiver and its first argument, raises

 @param run - the call under test

 @returns What the call returns

 @example
 ```ts
 const value = withExhaustionFromMethod({ owner: Array.prototype, method: 'join', applies: isTheCall, run: read, },);
 ```
 */
export function withExhaustionFromMethod<Result>(
  {
    owner,
    method,
    applies,
    run,
  }: {
    readonly owner: Record<string, unknown>;
    readonly method: string;
    readonly applies: (
      receiver: unknown,
      first: unknown,
    ) => boolean;
    readonly run: () => Result;
  },
): Result {
  /**
   The method as it was.
   */
  const original = owner[method];
  if (!isMethod(original,))
    throw new Error(`unreachable: ${method} is a method of the owner`,);
  owner[method] = function replacedMethod(
    this: unknown,
    first: unknown,
    second: unknown,
  ): unknown {
    if (applies(
      this,
      first,
    ))
      throw engineExhaustion();
    return original.call(
      this,
      first,
      second,
    );
  };
  /**
   The method put back when this function returns.
   */
  using _putBack = {
    [Symbol.dispose]: function putMethodBack(): void {
      owner[method] = original;
    },
  };
  return run();
}
