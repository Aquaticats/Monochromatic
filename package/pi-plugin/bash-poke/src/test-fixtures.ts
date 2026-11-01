/**
 Test-only Pi fakes for bash-poke suites.

 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  MessageRenderer,
  Theme,
} from '@earendil-works/pi-coding-agent';

//region Types

/**
 Handler shape captured from the fake Pi API.
 */
type EventHandler = (
  event: unknown,
  context: unknown
) => unknown;

/**
 One custom message handed to the fake `sendMessage`.
 */
type SentMessage = {
  /**
   Message payload, including the custom type and renderer details.
   */
  readonly message: {
    readonly customType: string;
    readonly content: unknown;
    readonly display: boolean;
    readonly details: unknown;
  };

  /**
   Delivery options, which decide whether a turn is triggered.
   */
  readonly options: unknown;
};

/**
 Fake Pi API plus everything it recorded.
 */
type FakePi = {
  /**
   API handed to the extension under test.
   */
  readonly api: ExtensionAPI;

  /**
   Handlers grouped by event name.
   */
  readonly handlers: ReadonlyMap<string, readonly EventHandler[]>;

  /**
   Renderers grouped by custom message type.
   */
  readonly renderers: ReadonlyMap<string, MessageRenderer>;

  /**
   Custom messages sent, in order.
   */
  readonly sentMessages: readonly SentMessage[];
};

/**
 One widget call, distinguished by kind so absence needs no nullish member.
 */
type WidgetCall =
  | {
    readonly kind: 'draw';
    readonly lines: readonly string[];
  }
    | {
      readonly kind: 'clear';
    };

/**
 Fake interactive context plus everything it recorded.
 */
type FakeContext = {
  /**
   Context handed to handlers under test.
   */
  readonly ctx: ExtensionContext;

  /**
   Widget draws and clears, in call order.
   */
  readonly widgets: readonly WidgetCall[];

  /**
   Terminal input listeners registered by the extension.
   */
  readonly terminalListeners: readonly ((data: string) => unknown)[];

  /**
   Notifications shown to the user, in order.
   */
  readonly notifications: readonly string[];

  /**
   Editor text the context reports, mutable per case.
   */
  readonly editor: { text: string; };

  /**
   Idle flag the context reports, mutable per case.
   */
  readonly idle: { value: boolean; };
};

//endregion Types

//region Fakes

/**
 Creates a fake Pi API that records registrations and sent messages.
 
 @param sendMessage - replacement delivery boundary, to simulate a stale runtime
 
 @returns fake API and its recordings
 
 @example
 ```ts
 const fake = createFakePi();
 ```
 */
function createFakePi(
  {
    sendMessage,
  }: {
    readonly sendMessage?: (message: SentMessage) => void;
  } = {},
): FakePi {
  /**
   Handlers recorded per event name.
   */
  const handlers = new Map<string, EventHandler[]>();

  /**
   Renderers recorded per custom message type.
   */
  const renderers = new Map<string, MessageRenderer>();

  /**
   Custom messages recorded in send order.
   */
  const sentMessages: SentMessage[] = [];

  /**
   Fake API exposing only the surface this package touches.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the fake implements the three members this package calls; the full ExtensionAPI surface is Pi-owned and irrelevant to these cases.
  const api = {
    on(
      event: string,
      handler: EventHandler,
    ): void {
      handlers.set(
        event,
        [
          ...(handlers.get(event) ?? []),
          handler,
        ],
      );
    },
    registerMessageRenderer(
      customType: string,
      renderer: MessageRenderer,
    ): void {
      renderers.set(
        customType,
        renderer,
      );
    },
    sendMessage(
      message: SentMessage['message'],
      options: unknown,
    ): void {
      /**
       One recorded delivery.
       */
      const sent: SentMessage = {
        message,
        options,
      };
      sentMessages.push(sent, );
      if (sendMessage !== undefined)
        sendMessage(sent, );
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    handlers,
    renderers,
    sentMessages,
  };
}

/**
 Creates a fake interactive context that records widget and notification calls.
 
 @param mode - run mode the context reports
 
 @param hasUI - whether the context claims a UI surface
 
 @returns fake context and its recordings
 
 @example
 ```ts
 const fake = createFakeContext({ mode: 'tui', hasUI: true, },);
 ```
 */
function createFakeContext(
  {
    mode = 'tui',
    hasUI = true,
  }: {
    readonly mode?: string;
    readonly hasUI?: boolean;
  } = {},
): FakeContext {
  /**
   Widget draws and clears in call order.
   */
  const widgets: WidgetCall[] = [];

  /**
   Terminal input listeners in registration order.
   */
  const terminalListeners: ((data: string) => unknown)[] = [];

  /**
   Notification messages in call order.
   */
  const notifications: string[] = [];

  /**
   Editor text the context reports, mutable so a case can enter bash mode.
   */
  const editor = { text: '', };

  /**
   Idle flag the context reports, mutable so a case can simulate streaming.
   */
  const idle = { value: true, };

  /**
   Fake context exposing only the surface this package touches.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the fake implements the mode, idle, and UI members this package reads; the rest of Pi's context is irrelevant to these cases.
  const ctx = {
    mode,
    hasUI,
    cwd: '/work',
    isIdle(): boolean {
      return idle.value;
    },
    ui: {
      // An omitted content argument is how Pi clears a widget, so the parameter
      // is optional rather than a nullish union.
      setWidget(
        _key: string,
        content?: readonly string[],
      ): void {
        widgets.push(
          content === undefined
            ? { kind: 'clear', }
            : {
              kind: 'draw',
              lines: content,
            },
        );
      },
      onTerminalInput(handler: (data: string) => unknown, ): () => void {
        terminalListeners.push(handler, );
        return function unsubscribe(): void {
          /**
           Position of this listener, negative once it is already gone.
           */
          const index = terminalListeners.indexOf(handler, );
          if (index !== (-1))
            terminalListeners.splice(
              index,
              1,
            );
        };
      },
      getEditorText(): string {
        return editor.text;
      },
      notify(message: string, ): void {
        notifications.push(message, );
      },
    },
  } as unknown as ExtensionContext;

  return {
    ctx,
    widgets,
    terminalListeners,
    notifications,
    editor,
    idle,
  };
}

/**
 Creates a theme fixture that returns text uncolored.
 
 @returns theme preserving plain text
 
 @example
 ```ts
 plainTheme().fg('accent', 'text');
 ```
 */
function plainTheme(): Theme {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the fixture supplies the two colorizers the renderer calls; the rest of Pi's theme is irrelevant here.
  return {
    fg(
      _color: string,
      text: string,
    ) {
      return text;
    },
    bold(text: string, ) {
      return text;
    },
  } as unknown as Theme;
}

//endregion Fakes

export {
  createFakeContext,
  createFakePi,
  plainTheme,
};

export type {
  EventHandler,
  FakeContext,
  FakePi,
  SentMessage,
  WidgetCall,
};
