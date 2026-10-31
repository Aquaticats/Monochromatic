/**
 Tool-result content helpers for Pi Search Fetch.
 
 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  DEFAULT_MAX_LINES,
  formatSize,
  truncateHead,
  type TruncationOptions,
  type TruncationResult,
} from '@monochromatic-dev/agent-harness-shared-truncate/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { withFileMutationQueue, } from './file-mutation-queue.ts';
import type {
  ProviderFallback,
  SearchFetchProvider,
} from './search-fetch-client.ts';
import { modelTextForLinkupResponse, } from './tool-output-format.ts';

import { scratchParent, } from './scratch-parent.ts';
/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

//region Constants

/**
 Prefix used for temp directories that hold full response output.
 */
const TEMP_RESPONSE_DIR_PREFIX = 'pi-search-fetch-response-';

/**
 Number of random bytes added to temp directory prefix.
 */
const TEMP_RANDOM_BYTES = 4;

/**
 Bytes in one kibibyte, matching Pi's byte-limit size formatting.
 */
const BYTES_PER_KIBIBYTE = 1_024;

/**
 Linkup response kibibytes visible to the model before temp-file fallback.
 */
const LINKUP_VISIBLE_JSON_MAX_KIBIBYTES = 100;

/**
 Linkup response bytes visible to the model before temp-file fallback.
 */
const LINKUP_VISIBLE_JSON_MAX_BYTES: number = LINKUP_VISIBLE_JSON_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;

//endregion Constants

//region Types

/**
 Tool result shape host adapters receive, with details kept out of model-visible text.
 */
type SearchFetchToolResult = {
  /**
   Model-visible content items in result order.
   */
  content: TextContentItem[];
  /**
   Structured details hosts may render separately.
   */
  details: LinkupToolDetails;
};

/**
 Text content item returned by a Pi tool.
 */
type TextContentItem = {
  /**
   Content item type.
   */
  readonly type: 'text';
  /**
   Text visible to the model.
   */
  readonly text: string;
};

/**
 Details stored by Linkup tool results.
 */
type LinkupToolDetails = {
  /**
   Model-visible Linkup-shaped response.
   */
  readonly linkupResponse: unknown;
  /**
   Untouched upstream provider response.
   */
  readonly rawLinkupResponse: unknown;
  /**
   Provider that produced response.
   */
  readonly provider?: SearchFetchProvider;
  /**
   Fallback steps taken before the responding provider answered.
   */
  readonly fallbackChain?: readonly ProviderFallback[];
  /**
   Ignored compatibility keys, when supplied.
   */
  readonly ignoredKeys?: readonly string[];
  /**
   Blocked search result URLs removed locally, when any were removed.
   */
  readonly removedBlockedUrls?: readonly string[];
  /**
   Full response temp-file path, kept under this field for compatibility.
   */
  readonly fullJsonPath?: string;
  /**
   Truncation metadata, when visible response text was truncated.
   */
  readonly truncation?: TruncationResult;
};

/**
 Options for creating response content items.
 */
type JsonContentOptions = {
  /**
   Linkup response value to render.
   */
  readonly value: unknown;
  /**
   Whether accepted search result envelopes should render as JSONL.
   */
  readonly renderResultsArrayAsJsonl?: boolean;
  /**
   Optional truncation limits, primarily for tests.
   */
  readonly truncationOptions?: Readonly<TruncationOptions>;
};

/**
 Response content item plus temp-file metadata.
 */
type JsonContentResult = {
  /**
   Text content item visible to the model.
   */
  readonly content: TextContentItem;
  /**
   Full response temp-file path when truncation occurred.
   */
  readonly fullJsonPath?: string;
  /**
   Truncation metadata when truncation occurred.
   */
  readonly truncation?: TruncationResult;
};

/**
 Options for warning content text.
 */
type WarningContentOptions = {
  /**
   Tool name reporting ignored keys.
   */
  readonly toolName: string;
  /**
   Ignored compatibility keys.
   */
  readonly ignoredKeys: readonly string[];
  /**
   Fixed behavior explanation.
   */
  readonly fixedBehavior: string;
};

/**
 Options for full Linkup tool output.
 */
type LinkupToolOutputOptions = {
  /**
   Tool name reporting warnings.
   */
  readonly toolName: string;
  /**
   Model-visible response object.
   */
  readonly linkupResponse: unknown;
  /**
   Untouched upstream provider response object.
   */
  readonly rawLinkupResponse: unknown;
  /**
   Ignored compatibility keys.
   */
  readonly ignoredKeys: readonly string[];
  /**
   Provider that produced response.
   */
  readonly provider?: SearchFetchProvider;
  /**
   Fallback steps taken before the responding provider answered.
   */
  readonly fallbackChain?: readonly ProviderFallback[];
  /**
   Fixed behavior explanation for ignored-key warning.
   */
  readonly fixedBehavior: string;
  /**
   Whether accepted search result envelopes should render as JSONL.
   */
  readonly renderResultsArrayAsJsonl?: boolean;
  /**
   Blocked search result URLs removed locally.
   */
  readonly removedBlockedUrls?: readonly string[];
};

//endregion Types

/**
 Module logger.
 */
const l = tagged({
  tag: 'tool-output',
  l: linkupLogger,
},);

//region Public API

/**
 Create complete Linkup tool output with optional ignored-key warning.
 
 @param options - response, raw response, and warning metadata
 
 @returns Pi tool result with response content and structured details
 
 @mutates options - `JSON.stringify` may invoke hooks on response value stored in options.
 
 @example
 ```ts
 await createLinkupToolOutput({
   toolName: 'linkup_web_search',
   linkupResponse: { results: [] },
   rawLinkupResponse: { results: [] },
   ignoredKeys: [],
   fixedBehavior: 'This extension always uses fixed search behavior.',
 });
 ```
 */
async function createLinkupToolOutput(
  options: LinkupToolOutputOptions,
): Promise<SearchFetchToolResult> {
  /**
   Model-visible content item and truncation metadata.
   */
  const modelContent = await createJsonContent({
    value: options.linkupResponse,
    renderResultsArrayAsJsonl: options.renderResultsArrayAsJsonl === true,
  },);
  /**
   Warning content item, when compatibility keys were ignored.
   */
  const warningContent = options.ignoredKeys
    .length
    === 0
    ? undefined
    : createWarningContent({
      toolName: options.toolName,
      ignoredKeys: options.ignoredKeys,
      fixedBehavior: options.fixedBehavior,
    },);
  /**
   Model-visible content items in final result order.
   */
  const content = warningContent === undefined
    ? [modelContent.content,]
    : [
      warningContent,
      modelContent.content,
    ];

  return {
    content,
    details: {
      linkupResponse: options.linkupResponse,
      rawLinkupResponse: options.rawLinkupResponse,
      ...(options.provider === undefined ? {} : { provider: options.provider, }),
      ...(options.fallbackChain === undefined ? {} : { fallbackChain: options.fallbackChain, }),
      ...(options.ignoredKeys
        .length
        === 0 ? {} : { ignoredKeys: options.ignoredKeys, }),
      ...((options.removedBlockedUrls === undefined) || (options.removedBlockedUrls
        .length
        === 0)
        ? {}
        : { removedBlockedUrls: options.removedBlockedUrls, }),
      ...(modelContent.fullJsonPath === undefined ? {} : { fullJsonPath: modelContent.fullJsonPath, }),
      ...(modelContent.truncation === undefined ? {} : { truncation: modelContent.truncation, }),
    },
  };
}

/**
 Create a model-visible text content item, truncating and writing full response text to temp when needed.
 
 @param options - value and optional truncation limits
 
 @returns content item and truncation metadata
 
 @mutates options - `JSON.stringify` may invoke hooks on response value stored in options.
 
 @example
 ```ts
 await createJsonContent({ value: { ok: true } });
 ```
 */
async function createJsonContent(options: JsonContentOptions,): Promise<JsonContentResult> {
  /**
   Model-visible response text and format metadata.
   */
  const modelText = modelTextForLinkupResponse({
    value: options.value,
    renderResultsArrayAsJsonl: options.renderResultsArrayAsJsonl === true,
  },);
  /**
   Truncation result using Linkup byte cap and Pi line cap unless tests override limits.
   */
  const truncation = truncateHead({
    content: modelText.text,
    options: {
      maxLines: options.truncationOptions
        ?.maxLines
        ?? DEFAULT_MAX_LINES,
      maxBytes: options.truncationOptions
        ?.maxBytes
        ?? LINKUP_VISIBLE_JSON_MAX_BYTES,
    },
  },);
  if (!truncation.truncated)
    return {
      content: {
        type: 'text',
        text: truncation.content,
      },
    };

  /**
   Temp file path containing the full response text.
   */
  const fullJsonPath = await writeFullResponseToTemp({
    responseText: modelText.text,
    filename: modelText.tempFilename,
  },);
  /**
   Visible text with truncation notice appended.
   */
  const visibleText = [
    truncation.content,
    `[${modelText.truncationLabel} truncated: showing ${String(truncation.outputLines,)} of ${String(truncation.totalLines,)} lines (${formatSize(truncation.outputBytes,)} of ${formatSize(truncation.totalBytes,)}). Full ${modelText.truncationLabel} saved to: ${fullJsonPath}]`,
  ].join('\n\n',);

  l.warn(`truncated Linkup response; full response at ${fullJsonPath}`,);
  return {
    content: {
      type: 'text',
      text: visibleText,
    },
    fullJsonPath,
    truncation,
  };
}

/**
 Create model-visible warning for ignored compatibility keys.
 
 @param options - tool name, ignored keys, and fixed behavior text
 
 @returns warning text content item
 
 @example
 ```ts
 createWarningContent({ toolName: 'linkup_web_search', ignoredKeys: ['depth'], fixedBehavior: 'Fixed.' });
 ```
 */
function createWarningContent(options: WarningContentOptions,): TextContentItem {
  return {
    type: 'text',
    text: [
      `Warning: ignored extension-unsupported ${options.toolName} parameters: ${options.ignoredKeys
        .join(', ',)}.`,
      options.fixedBehavior,
    ].join('\n',),
  };
}

//endregion Public API

//region Helpers

/**
 Write full response text to a temp file.
 
 @param responseText - response text to write
 
 @param filename - temp file basename
 
 @returns temp file path
 */
async function writeFullResponseToTemp(
  {
    responseText,
    filename,
  }: {
    readonly responseText: string;
    readonly filename: string;
  },
): Promise<string> {
  /**
   Random suffix to avoid temp directory collisions.
   */
  const randomSuffix = randomBytes(TEMP_RANDOM_BYTES,)
    .toString('hex',);
  /**
   Temp directory dedicated to this response.
   */
  const tempDir = await mkdtemp(join(
    await scratchParent(),
    `${TEMP_RESPONSE_DIR_PREFIX}${randomSuffix}-`,
  ),);
  /**
   Temp file path storing full response text.
   */
  const tempFile = join(
    tempDir,
    filename,
  );
  await withFileMutationQueue({
    filePath: tempFile,
    mutate: async function writeQueuedResponse() {
      await writeFile(
        tempFile,
        responseText,
        'utf8',
      );
    },
  },);
  return tempFile;
}

//endregion Helpers

export {
  createJsonContent,
  createLinkupToolOutput,
  createWarningContent,
  LINKUP_VISIBLE_JSON_MAX_BYTES,
};
export type {
  JsonContentResult,
  LinkupToolDetails,
  LinkupToolOutputOptions,
  SearchFetchToolResult,
  TextContentItem,
  WarningContentOptions,
};
