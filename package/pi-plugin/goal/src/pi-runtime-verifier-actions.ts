/**
 Disposable action adapters for real Pi goal loader verification.
 
 @module
 */

import type { ExtensionRuntime, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Runtime message contracts

/**
 Custom message and delivery metadata observed through real loader runtime.
 */
type CapturedRuntimeMessage = {
  readonly customType: string;
  readonly content: unknown;
  readonly triggerTurn: boolean;
};

/**
 Runtime custom message accepted by Pi send action.
 */
type RuntimeMessage = Parameters<ExtensionRuntime['sendMessage']>[0];

/**
 Runtime custom-message options accepted by Pi send action.
 */
type RuntimeMessageOptions = NonNullable<Parameters<ExtensionRuntime['sendMessage']>[1]>;

/**
 Minimal persisted session mutation capability used by adapters.
 */
type ExtensionRuntimeSessionManager = {
  readonly appendCustomEntry: (
    customType: string,
    data?: unknown,
  ) => string;
  readonly appendCustomMessageEntry: (
    customType: string,
    content: RuntimeMessage['content'],
    display: boolean,
    details?: unknown,
  ) => string;
};

//endregion Runtime message contracts

//region Message capture

/**
 Capture and persist one visible custom message.
 
 @param sessionManager - runtime session mutation capability
 
 @param messages - verifier capture array
 
 @param message - loaded-extension custom message
 
 @param options - Pi delivery metadata
 
 @mutates sessionManager - sessionManager.appendCustomMessageEntry appends visible transcript state
 
 @mutates messages - messages.push records custom delivery
 
 @mutates message - sessionManager.appendCustomMessageEntry may invoke serialization hooks reachable from message
 
 @example
 ```ts
 captureDisposableMessage({ sessionManager, messages, message, options: { triggerTurn: true } });
 ```
 */
function captureDisposableMessage(
  {
    sessionManager,
    messages,
    message,
    options,
  }: {
    readonly sessionManager: ExtensionRuntimeSessionManager;
    readonly messages: CapturedRuntimeMessage[];
    readonly message: ForeignBorrowed<RuntimeMessage>;
    readonly options?: ForeignBorrowed<RuntimeMessageOptions>;
  },
): void {
  messages.push({
    customType: message.customType,
    content: message.content,
    triggerTurn: options?.triggerTurn === true,
  },);
  sessionManager.appendCustomMessageEntry(
    message.customType,
    message.content,
    message.display,
    message.details,
  );
}

//endregion Message capture

//region Action binding

/**
 Bind stateful actions used by built goal extension after package discovery.
 
 @param runtime - real Pi extension runtime returned by package loader
 
 @param sessionManager - disposable persisted session owner
 
 @param messages - custom-message capture
 
 @mutates runtime - replaces uninitialized action stubs with disposable fixture adapters
 
 @mutates sessionManager - bound actions append custom state and visible messages
 
 @mutates messages - bound send action records delivery metadata
 
 @example
 ```ts
 bindRuntimeActions({ runtime, sessionManager, messages: [] });
 ```
 */
function bindRuntimeActions(
  {
    runtime,
    sessionManager,
    messages,
  }: {
    readonly runtime: ExtensionRuntime;
    readonly sessionManager: ExtensionRuntimeSessionManager;
    readonly messages: CapturedRuntimeMessage[];
  },
): void {
  runtime.appendEntry = function appendDisposableEntry(
    customType: string,
    data,
  ): void {
    sessionManager.appendCustomEntry(
      customType,
      data,
    );
  };
  runtime.sendMessage = function sendDisposableMessage(
    message: ForeignBorrowed<RuntimeMessage>,
    options?: ForeignBorrowed<RuntimeMessageOptions>,
  ): void {
    captureDisposableMessage({
      sessionManager,
      messages,
      message,
      ...(options === undefined ? {} : { options, }),
    },);
  };
}

//endregion Action binding

export { bindRuntimeActions, };
export type { CapturedRuntimeMessage, };
