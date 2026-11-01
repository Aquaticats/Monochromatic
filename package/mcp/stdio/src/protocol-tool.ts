// Tool definition, content, and handler types for MCP spec revision 2026-07-28.
// Revision 2026-07-28 made `description` optional and added `title`, `outputSchema`,
// `annotations`, and structured tool output.

//region Tool schemas: how a tool declares its arguments and its structured output

/**
 JSON Schema describing tool input parameters.
 The root must be an object because tool arguments are always a JSON object; beyond that
 any JSON Schema 2020-12 keyword may appear, so composition keywords such as `oneOf` and
 `not` are admitted by the index signature rather than enumerated here.
 
 @example
 ```ts
 const schema: ToolInputSchema = {
   type: 'object',
   properties: { query: { type: 'string' } },
   required: ['query'],
 };
 ```
 */
export type ToolInputSchema = {
  readonly type: 'object';
  readonly properties?: Readonly<Record<string, unknown>>;
  readonly required?: readonly string[];
  readonly $schema?: string;
  readonly [keyword: string]: unknown;
};

/**
 JSON Schema describing what a tool places in `structuredContent`.
 Unconstrained at the root because structured output may be any JSON value.
 
 @example
 ```ts
 const schema: ToolOutputSchema = {
   type: 'object',
   properties: { exitCode: { type: 'number' } },
 };
 ```
 */
export type ToolOutputSchema = Readonly<Record<string, unknown>>;

//endregion

//region Tool definition: what `tools/list` exposes to clients

/**
 Behavioral hints a client may surface or use to decide how freely to call a tool.
 Hints are advisory: a client must not treat them as guarantees from an untrusted server.
 
 @example
 ```ts
 const annotations: ToolAnnotations = { title: 'List VMs', readOnlyHint: true };
 ```
 */
export type ToolAnnotations = {
  readonly title?: string;
  readonly readOnlyHint?: boolean;
  readonly destructiveHint?: boolean;
  readonly idempotentHint?: boolean;
  readonly openWorldHint?: boolean;
};

/**
 Tool definition exposed to clients via `tools/list`.
 Display precedence for a name runs `title`, then `annotations.title`, then `name`.
 
 @example
 ```ts
 const tool: ToolDefinition = {
   name: 'get_weather',
   description: 'Fetches current weather for a location.',
   inputSchema: { type: 'object' },
 };
 ```
 */
export type ToolDefinition = {
  readonly name: string;
  readonly title?: string;
  readonly description?: string;
  readonly inputSchema: ToolInputSchema;
  readonly outputSchema?: ToolOutputSchema;
  readonly annotations?: ToolAnnotations;
};

//endregion

//region Tool results: what a handler returns and what the server sends back

/**
 Hints telling a client who a content item is for and how much it matters.
 Advisory only: they come from the server, so a client weighs them rather than trusting them.
 
 @example
 ```ts
 const annotations: ContentAnnotations = { audience: ['user'], priority: 0.5 };
 ```
 */
export type ContentAnnotations = {
  readonly audience?: readonly ('user' | 'assistant')[];
  readonly priority?: number;
  readonly lastModified?: string;
};

/**
 Text content item, the form every tool in this repo returns.
 
 @example
 ```ts
 const content: TextContent = { type: 'text', text: 'Hello, world!' };
 ```
 */
export type TextContent = {
  readonly type: 'text';
  readonly text: string;
  readonly annotations?: ContentAnnotations;
};

/**
 Image content item carrying base64 image bytes.
 `data` is base64 rather than raw bytes because content items cross the wire as JSON.
 
 @example
 ```ts
 const content: ImageContent = { type: 'image', data: 'iVBORw0KGgo=', mimeType: 'image/png' };
 ```
 */
export type ImageContent = {
  readonly type: 'image';
  readonly data: string;
  readonly mimeType: string;
  readonly annotations?: ContentAnnotations;
};

/**
 Audio content item carrying base64 audio bytes.
 
 @example
 ```ts
 const content: AudioContent = { type: 'audio', data: 'SUQzBA==', mimeType: 'audio/mpeg' };
 ```
 */
export type AudioContent = {
  readonly type: 'audio';
  readonly data: string;
  readonly mimeType: string;
  readonly annotations?: ContentAnnotations;
};

/**
 Pointer to a resource the client may fetch, rather than its bytes.
 Preferred over embedding when the payload is large or the client may not need it.
 
 @example
 ```ts
 const content: ResourceLink = { type: 'resource_link', uri: 'file:///var/log/build.log' };
 ```
 */
export type ResourceLink = {
  readonly type: 'resource_link';
  readonly uri: string;
  readonly name?: string;
  readonly title?: string;
  readonly description?: string;
  readonly mimeType?: string;
  readonly size?: number;
  readonly annotations?: ContentAnnotations;
};

/**
 Contents of a resource embedded directly in a result.
 Carries exactly one of `text` or base64 `blob`, matching the two resource-contents shapes.
 
 @example
 ```ts
 const contents: ResourceContents = { uri: 'file:///etc/hosts', text: '127.0.0.1 localhost' };
 ```
 */
export type ResourceContents = {
  readonly uri: string;
  readonly mimeType?: string;
  readonly text?: string;
  readonly blob?: string;
};

/**
 Resource embedded directly in a result, bytes and all.
 
 @example
 ```ts
 const content: EmbeddedResource = {
   type: 'resource',
   resource: { uri: 'file:///etc/hosts', text: '127.0.0.1 localhost' },
 };
 ```
 */
export type EmbeddedResource = {
  readonly type: 'resource';
  readonly resource: ResourceContents;
  readonly annotations?: ContentAnnotations;
};

/**
 Single content item in a tool call result.
 Every tool in this repo returns {@link TextContent}; the rest of the union exists so a
 handler returning an image, a sound, or a resource is expressible without widening the
 protocol types at that point.
 
 @example
 ```ts
 const content: ToolContent = { type: 'text', text: 'Hello, world!' };
 ```
 */
export type ToolContent =
  | TextContent
    | ImageContent
    | AudioContent
    | ResourceLink
    | EmbeddedResource;

/**
 Result returned from a tool call handler.
 The server stamps protocol envelope fields onto it, so a handler supplies only its payload.
 
 Failures inside a tool belong here with `isError` set, not in a JSON-RPC error response:
 a protocol error hides the failure from the model, which then cannot correct itself.
 
 @example
 ```ts
 const result: ToolCallResult = {
   content: [{ type: 'text', text: 'Done.' }],
 };
 ```
 */
export type ToolCallResult = {
  readonly content: readonly ToolContent[];
  readonly structuredContent?: unknown;
  readonly isError?: boolean;
};

/**
 Async handler function invoked when a tool is called.
 
 @param args - Parsed arguments from client's `tools/call` request.
 
 @returns Tool execution result containing content items.
 
 @example
 ```ts
 const handler: ToolHandler = async (args) => ({
   content: [{ type: 'text', text: `Got: ${JSON.stringify(args)}` }],
 });
 ```
 */
export type ToolHandler = (
  args: Readonly<Record<string, unknown>>,
) => ToolCallResult | Promise<ToolCallResult>;

//endregion
