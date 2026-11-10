/**
 * Types and callbacks for an LSP client handle.
 *
 * Split out of `lsp-client.ts` so the lifecycle module stays inside the
 * repository's max-lines budget; the lifecycle module re-exports the public
 * ones, so consumers keep importing from `./lsp-client.ts`.
 */

import type { Logger, } from '../log.ts';

import type { LspServerCapabilities, } from './types.ts';

/**
 * Server-initiated notification callback payload.
 */
export type LspClientNotification = {
  /**
   * LSP notification method name.
   */
  readonly method: string;
  /**
   * LSP notification params.
   */
  readonly params: unknown;
};

/**
 * Child-process exit callback payload.
 */
export type LspClientExit = {
  /**
   * Whether the process exited without an explicit shutdown request.
   */
  readonly unexpected: boolean;
  /**
   * Exit code reported by Node, or null when signal-only.
   */
  readonly code: number | null;
  /**
   * Rolling stderr tail captured before exit.
   */
  readonly recentStderr: string;
};

/**
 * Options for {@link createLspClient}.
 */
export type LspClientOptions = {
  /**
   * Executable path or name.
   */
  readonly command: string;
  /**
   * Command-line arguments.
   */
  readonly args: readonly string[];
  /**
   * Display name for logging.
   */
  readonly name: string;
  /**
   * Working directory for the child process.
   */
  readonly cwd: string;
  /**
   * Environment variables for the child process.
   */
  readonly env: Readonly<Record<string, string | undefined>>;
  /**
   * Parent logger to compose tags from.
   */
  readonly l: Logger;
  /**
   * Callback for server-initiated notifications.
   */
  readonly onNotification: (event: LspClientNotification,) => void;
  /**
   * Callback invoked when the process exits.
   */
  readonly onExit: (event: LspClientExit,) => void;
};

/**
 * Initialize request payload for {@link LspClient.initialize}.
 */
export type LspInitializeOptions = {
  /**
   * Workspace root URI such as `file:///home/user/project`.
   */
  readonly rootUri: string;
  /**
   * Server-specific initialization options.
   */
  readonly initializationOptions?: Readonly<Record<string, unknown>>;
};

/**
 * JSON-RPC request payload for {@link LspClient.request}.
 */
export type LspRequestOptions = {
  /**
   * LSP method name such as `textDocument/hover`.
   */
  readonly method: string;
  /**
   * Method parameters.
   */
  readonly params: unknown;
  /**
   * Optional per-request timeout in milliseconds.
   */
  readonly timeoutMs?: number;
};

/**
 * JSON-RPC notification payload for {@link LspClient.notify}.
 */
export type LspNotifyOptions = {
  /**
   * LSP method name such as `textDocument/didOpen`.
   */
  readonly method: string;
  /**
   * Notification parameters.
   */
  readonly params: unknown;
};

/**
 * Mutable LSP client state captured by the factory closure.
 */
export type LspClientState = {
  /**
   * Counter for generating unique JSON-RPC request IDs.
   */
  nextId: number;
  /**
   * Rolling buffer of recent stderr output.
   */
  stderrBuffer: string;
  /**
   * Whether the LSP initialize handshake has completed.
   */
  initialized: boolean;
  /**
   * Whether a graceful shutdown has been initiated.
   */
  shuttingDown: boolean;
  /**
   * Whether the child process has exited.
   */
  dead: boolean;
  /**
   * Server capabilities reported during initialization.
   */
  capabilities: LspServerCapabilities;
};

/**
 * Client for a single LSP server process.
 */
export type LspClient = Readonly<{
  /**
   * Server capabilities reported during initialization.
   */
  readonly capabilities: LspServerCapabilities;
  /**
   * Whether the LSP initialize handshake has completed.
   */
  readonly initialized: boolean;
  /**
   * Whether the child process has exited.
   */
  readonly dead: boolean;
  /**
   * Performs the LSP initialize handshake.
   */
  readonly initialize: (opts: LspInitializeOptions,) => Promise<LspServerCapabilities>;
  /**
   * Sends a JSON-RPC request and waits for the correlated response.
   */
  readonly request: (opts: LspRequestOptions,) => Promise<unknown>;
  /**
   * Sends a JSON-RPC notification.
   */
  readonly notify: (opts: LspNotifyOptions,) => void;
  /**
   * Gracefully shuts down the LSP server.
   */
  readonly shutdown: () => Promise<void>;
}>;
