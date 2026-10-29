import type { JsonValue, } from '../pipeline/json.ts';

//region Shapes: declare a data-driven LSP4IJ server settings policy

/**
 LSP4IJ persistent-state file names holding global and user-defined settings.
 */
export type Lsp4ijOptionsFiles = {
  readonly languageSettings: string;
  readonly userDefined: string;
};

/**
 Predicate data identifying the base user-defined server to derive from.
 */
export type Lsp4ijBaseServerMatch = {
  readonly commandLineIncludes?: string;
  readonly excludeServerIds?: readonly string[];
  readonly serverNameEquals?: string;
  readonly templateId?: string;
};

/**
 Flat-JSON patch applied to a server's embedded configuration content.
 */
export type Lsp4ijConfigPatch = {
  readonly arrayUnion?: Readonly<Record<string, readonly string[]>>;
  readonly set?: Readonly<Record<string, JsonValue>>;
};

/**
 Additional scoped server derived from the base server for specific files.
 */
export type Lsp4ijScopedServer = {
  readonly config?: Lsp4ijConfigPatch;
  readonly configOmitKeys?: readonly string[];
  readonly copyOptions: readonly string[];
  readonly fileNames: readonly string[];
  readonly id: string;
  readonly languageId: string;
  readonly name: string;
};

/**
 Declarative policy for syncing one LSP4IJ language server and its scoped variants.
 */
export type Lsp4ijServerSettings = {
  readonly baseConfig?: Lsp4ijConfigPatch;
  readonly baseServerMatch: Lsp4ijBaseServerMatch;
  readonly optionsFiles?: Lsp4ijOptionsFiles;
  readonly productPrefixes: readonly string[];
  readonly schemaDefaults?: Readonly<Record<string, JsonValue>>;
  readonly scopedServers: readonly Lsp4ijScopedServer[];
};

//endregion Shapes
