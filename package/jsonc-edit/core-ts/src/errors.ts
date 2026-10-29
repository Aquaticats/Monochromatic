/**
 Error thrown when a JSONC source string cannot be parsed.
 
 Carries the byte offset where parsing failed so callers can point at the
 problem. Thrown rather than returned, per the workspace error policy.
 
 @example
 ```ts
 try {
   parseJsonc({ source: '{' as StringJsonc });
 } catch (error) {
   if (error instanceof JsoncParseError) console.error(error.offset);
 }
 ```
 */
export class JsoncParseError extends Error {
  /**
   Zero-based byte offset into the source where parsing failed.
   */
  readonly offset: number;

  /**
   @param message - Human-readable failure description.
   
   @param offset - Byte offset into source where parsing failed.
   */
  constructor({
    message,
    offset,
  }: {
    readonly message: string;
    readonly offset: number;
  },) {
    super(`${message} (at offset ${String(offset,)})`,);
    this.name = 'JsoncParseError';
    this.offset = offset;
  }
}

/**
 Error thrown when a path passed to an edit or read function does not resolve
 to a node in the document.
 
 @example
 ```ts
 jsoncGet({ state, path: ['missing'] }); // throws JsoncPathNotFoundError
 ```
 */
export class JsoncPathNotFoundError extends Error {
  /**
   Path segments that failed to resolve.
   */
  readonly path: readonly (string | number)[];

  /**
   @param path - Path segments that failed to resolve.
   
   @mutates path - `JSON.stringify` may invoke array accessors or proxy traps.
   */
  constructor({
    path,
  }: {
    path: readonly (string | number)[];
  },) {
    super(`no JSONC node at path ${JSON.stringify(path,)}`,);
    this.name = 'JsoncPathNotFoundError';
    this.path = path;
  }
}

/**
 Error thrown when an edit targets a node whose kind cannot hold the requested
 operation, such as indexing into a string or keying into an array.
 
 @example
 ```ts
 jsoncSet({ state, path: ['a', 0], value: 1 }); // throws if 'a' is a string
 ```
 */
export class JsoncTypeError extends Error {
  /**
   @param message - Description of the type mismatch.
   */
  constructor({
    message,
  }: {
    readonly message: string;
  },) {
    super(message,);
    this.name = 'JsoncTypeError';
  }
}
