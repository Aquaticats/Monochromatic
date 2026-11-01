/**
 Domain types and adapter seams for filesystem identity resolution.
 
 @module
 */

/**
 Colon-free filesystem identity suitable for trust-key grammars.
 
 @example
 ```ts
 const id = 'fs-uuid_1234' as FsId;
 ```
 */
export type FsId = string & { readonly __brand: 'FsId'; };

/**
 Mechanism that supplied an identity payload.
 
 @example
 ```ts
 const source: FsIdSource = 'volume-serial';
 ```
 */
export type FsIdSource =
  | 'fs-uuid'
    | 'volume-uuid'
    | 'volume-serial'
    | 'f-fsid'
    | 'device-number';

/**
 Successful filesystem identity resolution with stability metadata.
 
 @example
 ```ts
 const result: FsIdResolution = {
   value: 'fs-uuid_1234' as FsId,
   stable: true,
   source: 'fs-uuid',
 };
 ```
 */
export type FsIdResolution = {
  readonly value: FsId;
  readonly stable: boolean;
  readonly source: FsIdSource;
  readonly reason?: string;
};

/**
 Supported host platforms.
 
 @example
 ```ts
 const platform: SupportedFsIdPlatform = 'linux';
 ```
 */
export type SupportedFsIdPlatform = 'linux' | 'darwin' | 'win32';

/**
 One subprocess request made by a platform resolver.
 
 @example
 ```ts
 const request: FsIdCommand = {
   command: 'findmnt',
   args: ['--target', '/repo'],
 };
 ```
 */
export type FsIdCommand = {
  readonly command: string;
  readonly args: readonly string[];
};

/**
 Effect adapters used by one resolver instance.
 
 @example
 ```ts
 const adapters: FsIdResolverAdapters = {
   platform: () => 'linux',
   canonicalizePath: async ({ path }) => path,
   run: async () => 'uuid',
   deviceNumber: async () => '1',
   warn: () => undefined,
 };
 ```
 */
export type FsIdResolverAdapters = {
  readonly platform: () => NodeJS.Platform;
  readonly canonicalizePath: (input: { readonly path: string; }) => Promise<string>;
  readonly run: (input: FsIdCommand,) => Promise<string>;
  readonly deviceNumber: (input: { readonly path: string; }) => Promise<string>;
  readonly warn: (input: {
    readonly path: string;
    readonly reason: string;
  }) => void;
  /**
   Reports diagnostic branch detail when caller enables logging.
   */
  readonly debug?: (input: {
    /**
     Function-boundary logger tag.
     */
    readonly tag: string;
    /**
     Complete diagnostic message.
     */
    readonly message: string;
  }) => void;
  /**
   Reports resolution failure before it is rethrown.
   */
  readonly reportError?: (input: {
    /**
     Function-boundary logger tag.
     */
    readonly tag: string;
    /**
     Complete failure context.
     */
    readonly message: string;
    /**
     Underlying resolution failure.
     */
    readonly error: unknown;
  }) => void;
};

/**
 Asynchronous fresh-observation resolver function.
 
 @example
 ```ts
 const result = await resolver({ path: '/repo' });
 ```
 */
export type FsIdResolver = (input: {
  readonly path: string;
}) => Promise<FsIdResolution>;
