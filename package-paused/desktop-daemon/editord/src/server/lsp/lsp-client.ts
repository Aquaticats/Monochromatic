/**
 * Manages a single LSP server child process.
 *
 * Spawns the server, handles the JSON-RPC initialize handshake,
 * routes incoming responses and notifications, and provides
 * typed `request` and `notify` methods for outgoing messages.
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';

import {
  type Logger,
  tagged,
} from '../log.ts';
import {
  createLspParser,
  encodeLspMessage,
  type JsonRpcMessage,
} from './json-rpc.ts';
import { buildInitializeParams, } from './lsp-client-init.ts';
import {
  type PendingLspRequest,
  routeJsonRpcMessage,
} from './lsp-client-routing.ts';
import type { LspServerCapabilities, } from './types.ts';

/**
 * Timeout for LSP feature requests such as hover, completion,
 * inlay hints, and navigation (milliseconds).
 * Shorter than the client-side WebSocket timeout (30 s) so
 * the server can reply with an empty result before the client
 * gives up. Long enough for legitimate slow responses on large files.
 */
export const LSP_FEATURE_TIMEOUT_MS = 10_000;

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
type LspInitializeOptions = {
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
type LspRequestOptions = {
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
type LspNotifyOptions = {
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
type LspClientState = {
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

/**
 * Maximum bytes kept in the rolling stderr buffer.
 */
const STDERR_BUFFER_LIMIT = 4_096;

/**
 * Creates a client for a single LSP server process.
 *
 * Spawns the process, manages the JSON-RPC framing over stdio,
 * and provides request/notification methods. Incoming server
 * notifications (e.g. `textDocument/publishDiagnostics`) are
 * forwarded to the `onNotification` callback.
 *
 * @param command - executable path or name
 *
 * @param args - command-line arguments
 *
 * @param name - display name for logging
 *
 * @param cwd - working directory for the child process
 *
 * @param env - environment variables for the child process
 *
 * @param l - parent logger to compose tags from
 *
 * @param onNotification - callback for server-initiated notifications
 *
 * @param onExit - callback when the child process exits
 *
 * @returns frozen LSP client handle
 *
 * @example
 * ```ts
 * const client = createLspClient({
 *   command: 'tsc',
 *   args: ['--lsp', '--stdio'],
 *   name: 'tsc',
 *   cwd: '/home/user/project',
 *   env: process.env,
 *   l: logger,
 *   onNotification: function handleNotification(event) { console.info(event.method); },
 *   onExit: function handleExit(event) { console.info(event.code); },
 * });
 * ```
 */
export function createLspClient({
  command,
  args,
  name,
  cwd,
  env,
  l,
  onNotification,
  onExit,
}: LspClientOptions,): LspClient {
  /**
   * Tagged logger for this LSP client.
   */
  const clientLog = tagged({
    tag: name,
    l,
  },);
  /**
   * Map of pending requests keyed by JSON-RPC ID.
   */
  const pending = new Map<number, PendingLspRequest>();
  /**
   * Mutable lifecycle state kept private to this client.
   */
  const state: LspClientState = {
    nextId: 0,
    stderrBuffer: '',
    initialized: false,
    shuttingDown: false,
    dead: false,
    capabilities: {},
  };
  /**
   * Child process handle.
   */
  const proc: ChildProcess = spawn(
    command,
    [...args,],
    {
      cwd,
      env: { ...env, },
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
    },
  );

  proc.stdin
    ?.on(
    'error',
    function handleStdinError(error,) {
      /**
       * A broken stdin pipe means the server can receive no further messages.
       * The child may still be on its way out, so mark this client dead here:
       * without a listener Node raises this EPIPE as an unhandled `error` event
       * and takes the whole daemon down.
       */
      state.dead = true;
      clientLog.error(`stdin write failed: ${String(error,)}`,);
    },
    );

  /**
   * Writes a JSON-RPC message to the child process stdin.
   *
   * @param message - message object to encode and send
   */
  function send(message: unknown,): void {
    if (state.dead) {
      /**
       * Nothing can answer any more, so the write is dropped instead of failing
       * again against a closed pipe.
       */
      clientLog.warn('dropped message: LSP server is no longer running',);
      return;
    }
    /**
     * Length-prefixed JSON-RPC frame ready for stdin; produced by the framing encoder.
     */
    const encoded = encodeLspMessage({ message, },);
    proc.stdin
      ?.write(encoded,);
  }

  /**
   * Sends a JSON-RPC request and waits for the correlated response.
   *
   * @param method - LSP method name (e.g. "textDocument/hover")
   *
   * @param params - method parameters
   *
   * @param timeoutMs - optional per-request timeout; when set, the pending
   *   entry is cleaned up and the promise rejected if no response arrives
   *   within this duration. Omit for unbounded waits (e.g. `initialize`).
   *
   * @returns resolved response result, or rejects on error / timeout
   *
   * @throws when the server responds with a JSON-RPC error or the request times out
   */
  function request({
    method,
    params,
    timeoutMs,
  }: LspRequestOptions,): Promise<unknown> {
    /**
     * Monotonic JSON-RPC request id; incremented after capture so each request gets a fresh value.
     */
    const id = state.nextId;
    state.nextId += 1;
    /**
     * Outgoing JSON-RPC request envelope; serialized below and sent over stdin.
     */
    const message = {
      jsonrpc: '2.0' as const,
      id,
      method,
      params,
    };

    /* oxlint-disable eslint-plugin-promise/avoid-new -- request correlation requires storing resolve/reject in a map */
    /**
     * Promise resolved when the matching response arrives, or rejected on timeout/error.
     */
    const responsePromise = new Promise<unknown>(
      function awaitLspResponse(
        resolve,
        reject,
      ) {
        /**
         * Pending-request record stored under `id`; the response handler resolves it on arrival.
         */
        const entry: PendingLspRequest = {
          resolve,
          reject,
          timeoutId: null,
        };
        if (timeoutMs !== undefined) {
          entry.timeoutId = setTimeout(
            function rejectOnTimeout() {
              if (pending.delete(id,)) {
                clientLog.error(
                  `${method} (id ${id}) timed out after ${timeoutMs}ms`,
                );
                reject(
                  new Error(`${method} (id ${id}) timed out after ${timeoutMs}ms`,),
                );
              }
            },
            timeoutMs,
          );
        }
        pending.set(
          id,
          entry,
        );
      },
    );
    /* oxlint-enable eslint-plugin-promise/avoid-new */

    send(message,);
    return responsePromise;
  }

  /**
   * Sends a JSON-RPC notification (fire-and-forget, no response expected).
   *
   * @param method - LSP method name (e.g. "textDocument/didOpen")
   *
   * @param params - notification parameters
   */
  function notify({
    method,
    params,
  }: LspNotifyOptions,): void {
    send({
      jsonrpc: '2.0',
      method,
      params,
    },);
  }

  /**
   * Performs the LSP initialize handshake.
   * Sends `initialize` request, stores capabilities, then sends `initialized` notification.
   *
   * @param rootUri - workspace root URI (e.g. `file:///home/user/project`)
   *
   * @returns server capabilities
   */
  async function initialize({
    rootUri,
    initializationOptions,
  }: LspInitializeOptions,): Promise<LspServerCapabilities> {
    /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- LSP initialize always returns { capabilities } */
    /**
     * Raw initialize response, narrowed to the capabilities shape required by the spec.
     */
    const result = await request({
      method: 'initialize',
      params: buildInitializeParams({
        rootUri,
        initializationOptions,
      },),
    },) as { readonly capabilities: LspServerCapabilities; };
    /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */

    state.capabilities = result.capabilities;
    notify({
      method: 'initialized',
      params: {},
    },);
    state.initialized = true;
    clientLog.info('initialized',);
    return state.capabilities;
  }

  /**
   * Gracefully shuts down the LSP server.
   * Sends `shutdown` request followed by `exit` notification.
   * Falls back to killing the process if shutdown fails.
   */
  async function shutdown(): Promise<void> {
    state.shuttingDown = true;
    try {
      await request({
        method: 'shutdown',
        params: null,
      },);
      notify({
        method: 'exit',
        params: null,
      },);
    }
    catch (error) {
      clientLog.error(`shutdown failed, killing process: ${String(error,)}`,);
      proc.kill();
    }
  }

  /**
   * Routes an incoming JSON-RPC message to the appropriate handler.
   * Delegates to {@link routeJsonRpcMessage} for response correlation,
   * notification forwarding, and server-initiated request acknowledgment.
   *
   * @param message - parsed JSON-RPC message
   */
  function handleMessage(message: JsonRpcMessage,): void {
    routeJsonRpcMessage({
      message,
      pending,
      name,
      send,
      onNotification,
    },);
  }

  /**
   * JSON-RPC framing parser fed from stdout; emits parsed messages or parse errors.
   */
  const parser = createLspParser({
    onMessage: handleMessage,
    onError: function handleParseError(error,) {
      clientLog.error(`malformed JSON-RPC message: ${String(error,)}`,);
    },
  },);
  proc.stdout
    ?.on(
    'data',
    function handleStdout(chunk: Buffer,) {
      parser.feed(chunk,);
    },
  );
  proc.stderr
    ?.on(
    'data',
    function handleStderr(chunk: Buffer,) {
      /**
       * Decoded stderr chunk with trailing newline stripped; logged and appended to the rolling buffer.
       */
      const text = chunk.toString('utf8',)
        .trimEnd();
      clientLog.error(`stderr: ${text}`,);
      state.stderrBuffer += `${text}\n`;
      if (state.stderrBuffer
        .length
        > STDERR_BUFFER_LIMIT)
        state.stderrBuffer = state.stderrBuffer
          .slice(-STDERR_BUFFER_LIMIT,);
    },
  );

  proc.on(
    'exit',
    function handleExit(code,) {
      state.dead = true;
      /**
       * True when the exit was not preceded by an explicit `shutdown()`; signals a crash to `onExit`.
       */
      const unexpected = !state.shuttingDown;
      if (unexpected) {
        clientLog.error(`crashed with code ${String(code,)}`,);
        /**
         * Reject all pending requests so callers don't hang forever.
         */
        pending.forEach(function rejectPending(entry,): void {
          if (entry.timeoutId
            !== null)
            clearTimeout(entry.timeoutId,);
          entry.reject(new Error(`LSP server crashed (exit code ${String(code,)})`,),);
        },);
        pending.clear();
      }
      else {
        clientLog.info(`exited with code ${String(code,)}`,);
      }
      onExit({
        unexpected,
        code,
        recentStderr: state.stderrBuffer,
      },);
    },
  );

  return Object.freeze({
    /**
     * Server capabilities reported during initialization.
     *
     * @returns current server capabilities
     */
    get capabilities(): LspServerCapabilities {
      return state.capabilities;
    },
    /**
     * Whether the LSP initialize handshake has completed.
     *
     * @returns true when initialized
     */
    get initialized(): boolean {
      return state.initialized;
    },
    /**
     * Whether the child process has exited.
     *
     * @returns true when the process is no longer running
     */
    get dead(): boolean {
      return state.dead;
    },
    initialize,
    request,
    notify,
    shutdown,
  },);
}
