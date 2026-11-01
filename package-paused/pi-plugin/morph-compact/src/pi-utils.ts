/**
 Local re-implementations of pi-coding-agent helpers used by morph-compact.
 
 The upstream `convertToLlm` and `serializeConversation` are pure
 transformations with no external runtime deps. Vendoring them here lets the
 extension treat `@earendil-works/pi-coding-agent` (and its dependency
 subtree) as a type-only package, so pnpm doesn't have to install
 `@aws-sdk/client-bedrock-runtime`, `chalk`, `marked`, `cli-highlight`, and
 the rest of the upstream's runtime closure.
 
 The LLM wire-format types live in {@link ./pi-message-types.ts}.
 
 @module
 */

import type {
  AgentMessage,
  BashExecutionAgentMessage,
  ImageContent,
  Message,
  TextContent,
} from './pi-message-types.ts';

//region Summary text wrappers

/**
 Prefix prepended to compaction summaries when they reappear as a `user`
 message in LLM context. Mirrors pi-coding-agent's `COMPACTION_SUMMARY_PREFIX`
 verbatim so summaries produced here remain interchangeable with summaries
 produced by pi's default compaction path.
 */
const COMPACTION_SUMMARY_PREFIX =
  'The conversation history before this point was compacted into the following summary:\n\n<summary>\n';

/**
 Suffix paired with {@link COMPACTION_SUMMARY_PREFIX}. Matches upstream.
 */
const COMPACTION_SUMMARY_SUFFIX = '\n</summary>';

/**
 Prefix for a branch-summary user message. Matches upstream.
 */
const BRANCH_SUMMARY_PREFIX =
  'The following is a summary of a branch that this conversation came back from:\n\n<summary>\n';

/**
 Suffix paired with {@link BRANCH_SUMMARY_PREFIX}. Matches upstream.
 */
const BRANCH_SUMMARY_SUFFIX = '</summary>';

/**
 Maximum characters retained when serializing a `toolResult` content into
 summary text. Excess characters are truncated with a marker. Matches
 upstream's `TOOL_RESULT_MAX_CHARS` (2000) so summaries are byte-identical to
 what pi's default compaction would produce.
 */
const TOOL_RESULT_MAX_CHARS = 2_000;

//endregion

//region BashExecution and tool-result helpers

/**
 Convert a bash-execution message into the user-visible text shown to the
 LLM in summarized form.
 
 @param msg - bash execution message
 
 @returns formatted multi-line string with command, output, and any
   exit-code or truncation annotations
 
 @example
 ```typescript
 bashExecutionToText({
   role: 'bashExecution',
   command: 'ls',
   output: 'a\\nb',
   exitCode: 0,
   cancelled: false,
   truncated: false,
   timestamp: 0,
 });
 // 'Ran `ls`\\n```\\na\\nb\\n```'
 ```
 */
function bashExecutionToText(
  msg: BashExecutionAgentMessage,
): string {
  /**
   Per-section pieces joined with newlines to produce the final summary text.
   */
  const sections: string[] = [
    `Ran \`${msg.command}\``,
    (msg.output
      !== '')
      ? `\`\`\`\n${msg.output}\n\`\`\``
      : '(no output)',
  ];
  if (msg.cancelled)
    sections.push('\n(command cancelled)',);
  else if (
    (msg.exitCode
      !== undefined)
      && (msg.exitCode
        !== 0)
  ) {
    sections.push(`\nCommand exited with code ${msg.exitCode}`,);
  }
  if (
    msg.truncated
      && (msg.fullOutputPath
        !== undefined)
      && (msg.fullOutputPath
        !== '')
  ) {
    sections.push(`\n[Output truncated. Full output: ${msg.fullOutputPath}]`,);
  }
  return sections.join('\n',);
}

/**
 Truncate text to the given character budget, appending a notice describing
 how many characters were dropped.
 
 @param text - input text
 
 @param maxChars - inclusive maximum length to keep verbatim
 
 @returns text unchanged when within budget, otherwise the first `maxChars`
   characters followed by `[... N more characters truncated]`
 
 @example
 ```typescript
 truncateForSummary({ text: 'hello world', maxChars: 5 });
 // 'hello\\n\\n[... 6 more characters truncated]'
 ```
 */
function truncateForSummary({
  text,
  maxChars,
}: {
  readonly text: string;
  readonly maxChars: number;
},): string {
  if (text.length
    <= maxChars)
    return text;
  /**
   Dropped-character count surfaced in the truncation marker.
   */
  const truncatedChars = text.length
    - maxChars;
  return `${
    text.slice(
      0,
      maxChars,
    )
  }\n\n[... ${truncatedChars} more characters truncated]`;
}

//endregion

//region convertToLlm

/**
 Sentinel returned by {@link toLlmMessage} when a message is excluded from
 LLM context. A unique symbol rather than `undefined` so the no-nullish-union
 rule is satisfied while {@link convertToLlm} can filter it out.
 */
const OMIT = Symbol('morph compact message omitted from context',);

/**
 Map an extended `AgentMessage` (possibly carrying pi-coding-agent's custom
 roles) into a base LLM-compatible {@link Message}, or {@link OMIT} if the
 message should be omitted from LLM context.
 
 @param m - agent message to convert
 
 @returns LLM-compatible message, or {@link OMIT} when the message is excluded
   from context (e.g. `bashExecution` with `excludeFromContext`)
 
 @mutates m - `JSON.stringify` may invoke hooks when an unsupported message is reported.
 */
function toLlmMessage(
  m: AgentMessage,
): Message | typeof OMIT {
  if (m.role
    === 'bashExecution') {
    if (m.excludeFromContext
      === true)
      return OMIT;
    return {
      role: 'user',
      content: [{
        type: 'text',
        text: bashExecutionToText(m,),
      },],
      timestamp: m.timestamp,
    };
  }
  if (m.role
    === 'custom') {
    /**
     Normalized content array; raw strings are wrapped before forwarding.
     */
    const content = ((typeof m.content) === 'string')
      ? [{
        type: 'text' as const,
        text: m.content,
      },]
      : m.content;
    return {
      role: 'user',
      content,
      timestamp: m.timestamp,
    };
  }
  if (m.role
    === 'branchSummary') {
    return {
      role: 'user',
      content: [{
        type: 'text',
        text: BRANCH_SUMMARY_PREFIX + m
          .summary
          + BRANCH_SUMMARY_SUFFIX,
      },],
      timestamp: m.timestamp,
    };
  }
  if (m.role
    === 'compactionSummary') {
    return {
      role: 'user',
      content: [{
        type: 'text',
        text: COMPACTION_SUMMARY_PREFIX + m
          .summary
          + COMPACTION_SUMMARY_SUFFIX,
      },],
      timestamp: m.timestamp,
    };
  }
  if ((m.role
    === 'user') || (m.role
      === 'assistant')
    || (m.role
      === 'toolResult'))
    return m;
  throw new Error(`convertToLlm: unhandled message role: ${JSON.stringify(m,)}`,);
}

/**
 Transform extended `AgentMessage[]` (which may include pi-coding-agent's
 custom roles like `bashExecution`, `custom`, `branchSummary`,
 `compactionSummary`) into base LLM-compatible {@link Message | Messages}.
 
 Bit-for-bit compatible with pi-coding-agent's `convertToLlm` so summaries
 fed to Morph Compact match what pi's default compaction would feed to its
 summarization model.
 
 @param messages - extended agent messages
 
 @returns filtered list of LLM-compatible messages
 
 @mutates messages - `JSON.stringify` may invoke hooks when unsupported messages are reported.
 
 @example
 ```typescript
 convertToLlm(branchEntries.map(e => e.message));
 // [{ role: 'user', content: [...], timestamp: ... }, ...]
 ```
 */
export function convertToLlm(messages: AgentMessage[],): Message[] {
  /**
   Converted messages retained after omission filtering.
   */
  const converted: Message[] = [];
  for (const message of messages) {
    /**
     Converted message or omission sentinel for current input.
     */
    const candidate = toLlmMessage(message,);
    if (candidate !== OMIT)
      converted.push(candidate,);
  }
  return converted;
}

//endregion

//region serializeConversation

/**
 Extract concatenated text from a `user`-message content field, which may be
 a raw string or a structured array of {@link TextContent} or
 {@link ImageContent} blocks. Image blocks are dropped because Morph Compact
 operates on text only.
 
 @param content - user message content (raw or structured)
 
 @returns concatenated text or empty string when no text was present
 */
function userTextFromContent(
  content: string | readonly (TextContent | ImageContent)[],
): string {
  if ((typeof content) === 'string')
    return content;
  return content
    .filter(function isText(c,): c is TextContent {
      return c.type
        === 'text';
    },)
    .map(function readText(c,) {
      return c.text;
    },)
    .join('',);
}

/**
 Serialize an LLM-compatible conversation into the bracketed
 `[Role]: text` text format that pi's compaction summarizer expects. Tool
 results are truncated to {@link TOOL_RESULT_MAX_CHARS} per call to keep the
 summarization request within reasonable token budgets.
 
 Bit-for-bit compatible with pi-coding-agent's `serializeConversation`.
 
 @param messages - LLM-compatible messages, typically the output of
   {@link convertToLlm}
 
 @returns multi-paragraph string suitable as Morph Compact input
 
 @mutates messages - `JSON.stringify` may invoke hooks on tool-call argument values.
 
 @example
 ```typescript
 serializeConversation(convertToLlm(allMessages));
 // '[User]: hello\\n\\n[Assistant]: hi'
 ```
 */
export function serializeConversation(messages: Message[],): string {
  /**
   Top-level accumulator joined into the final serialized transcript.
   */
  const parts: string[] = [];
  for (const msg of messages) {
    if (msg.role
      === 'user') {
      /**
       User-role text harvested from raw string or structured content.
       */
      const content = userTextFromContent(msg.content,);
      if (content)
        parts.push(`[User]: ${content}`,);
      continue;
    }
    if (msg.role
      === 'assistant') {
      /**
       Per-message accumulator for visible assistant text blocks.
       */
      const textParts: string[] = [];
      /**
       Per-message accumulator for hidden reasoning blocks.
       */
      const thinkingParts: string[] = [];
      /**
       Per-message accumulator for formatted tool-call signatures.
       */
      const toolCalls: string[] = [];
      for (const block of msg.content) {
        if (block.type
          === 'text') {
          textParts.push(block.text,);
          continue;
        }
        if (block.type
          === 'thinking') {
          thinkingParts.push(block.thinking,);
          continue;
        }
        if (block.type
          === 'toolCall') {
          /**
           Human-readable argument string injected into the tool-call signature.
           */
          const argsStr = Object
            .entries(block.arguments,)
            .map(
              /**
               Formats one tool argument for transcript output.
               
               @param entry - Tool argument key and potentially effectful value.
               
               @returns key and serialized value text.
               
               @mutates entry - `JSON.stringify` may invoke hooks on argument value.
               */
            function fmtArg(entry,) {
                /**
                 Tool argument key and value.
                 */
              const [key, value,] = entry;
              return `${key}=${JSON.stringify(value,)}`;
            },
            )
            .join(', ',);
          toolCalls.push(`${block.name}(${argsStr})`,);
        }
      }
      if (thinkingParts.length
        > 0)
        parts.push(`[Assistant thinking]: ${thinkingParts.join('\n',)}`,);
      if (textParts.length
        > 0)
        parts.push(`[Assistant]: ${textParts.join('\n',)}`,);
      if (toolCalls.length
        > 0)
        parts.push(`[Assistant tool calls]: ${toolCalls.join('; ',)}`,);
      continue;
    }
    if (msg.role
      === 'toolResult') {
      /**
       Concatenated text payload after filtering out image blocks.
       */
      const content = msg
        .content
        .filter(function isText(c,): c is TextContent {
          return c.type
            === 'text';
        },)
        .map(function readText(c,) {
          return c.text;
        },)
        .join('',);
      if (content) {
        parts.push(
          `[Tool result]: ${
            truncateForSummary({
              text: content,
              maxChars: TOOL_RESULT_MAX_CHARS,
            },)
          }`,
        );
      }
    }
  }
  return parts.join('\n\n',);
}

//endregion
