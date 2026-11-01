## module-llm-type

Ready to publish.

Shared OpenAI-compatible LLM client types.
The canonical home for the chat-completion shapes that recur across every package
wrapping an OpenAI-compatible endpoint (openai,
 openrouter,
 anthropic-compatible,
 local llama-server),
so the `role` / `content` envelope and the response shape are defined once
and composed per consumer.

### Exports

- `CHAT_ROLES`:
   runtime tuple `['system', 'user', 'assistant']`;
   the source of truth `ChatRole` derives from.
- `ChatRole`:
   union of valid author roles,
   derived from `CHAT_ROLES`.
- `ChatMessage`:
   plain-text message envelope (`role` + string `content`).
- `ContentPart`:
   multimodal content part (text run or image URL) for vision requests.
- `ChatCompletionChoice`,
   `ChatCompletionResponse`:
   non-streaming response shapes.
- `CompletionUsage`:
   prompt and completion token counts.

### Usage

```ts
// text chat
import type {
  ChatCompletionResponse,
  ChatMessage,
} from '@monochromatic-dev/llm-llm-type';

const messages: readonly ChatMessage[] = [
  { role: 'system', content: 'You are concise.', },
  { role: 'user', content: 'Summarise this.', },
];

const data = (await response.json()) as ChatCompletionResponse;
const text = data.choices[0]?.message.content ?? '';
```

```ts
// vision chat: compose over ContentPart instead of redeclaring the envelope
import type {
  ChatRole,
  ContentPart,
} from '@monochromatic-dev/llm-llm-type';

type VisionMessage = {
  readonly role: Extract<ChatRole, 'user'>;
  readonly content: readonly ContentPart[];
};
```

```ts
// usage-aware response: intersect rather than redeclare
import type {
  ChatCompletionResponse,
  CompletionUsage,
} from '@monochromatic-dev/llm-llm-type';

type CompletionResponse = ChatCompletionResponse & { readonly usage: CompletionUsage; };
```

### Design decisions

- Text-first envelope.
  `ChatMessage.content` is a `string`.
  Multimodal callers compose a message over `ContentPart` arrays so the common text path never branches on content type.
- Response superset.
  Every field past `choices[].message.content` is optional,
   so a consumer's unchecked `as ChatCompletionResponse` cast keeps compiling and content reads stay non-null `string`.
- Snake_case usage fields.
  `CompletionUsage` mirrors the wire format (`prompt_tokens`,
   `completion_tokens`) so a parsed body satisfies the type without remapping.
- Roles derive from a runtime tuple.
  `ChatRole = typeof CHAT_ROLES[number]` keeps the union and the validation array in one place,
   and gives the build a non-empty bundle.

### Out of scope

- Streaming chunk types.
  No consumer streams through a shared type today;
   `inference-canary` streams via the OpenAI SDK's own types.
  Add `ChatCompletionChunk` when a second streaming consumer appears.
- A single client implementation.
  Each consumer keeps its own openai / anthropic / fetch client.
- `paper2vn`.
  It lives under `package-paused/` (not a workspace member),
   so it cannot resolve a `workspace:*` dependency.
  When un-paused it should adopt these types,
   composing a nullable variant (`content: string | null`) for its response.
