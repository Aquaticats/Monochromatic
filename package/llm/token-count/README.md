# @monochromatic-dev/llm-token-count

Token counting for Claude models using the Anthropic API.

Wraps the Anthropic `messages.countTokens` endpoint into a minimal library
with file-reading conveniences and an optique-based CLI.

**No inference costs.
**
This uses the dedicated token counting endpoint (`POST /v1/messages/count_tokens`),
not the messages endpoint.
Content is never sent to a model for generation:
the API only runs the tokenizer and returns a count.
The `model` parameter is required by the API to select the correct tokenizer,
but all current Claude models share the same tokenizer so the choice has no effect on the result.

## Setup

Set one of the following env vars (checked in priority order),
or pass `apiKey` in config:

1. `TOKEN_COUNT_CLAUDE_API_KEY`
2. `CLAUDE_API_KEY`
3. `ANTHROPIC_API_KEY`

Mise auto-injects values from `.env.local`.

## Library usage

### Count tokens in text

```ts
import { countTokens, } from '@monochromatic-dev/llm-token-count';

const result = await countTokens({ content: 'Hello, world!', },);
console.log(result.inputTokens,);
```

### Count tokens in a file

```ts
import { countFileTokens, } from '@monochromatic-dev/llm-token-count';

const result = await countFileTokens({ filePath: './CLAUDE.md', },);
console.log(`${result.filePath}: ${result.inputTokens} tokens`,);
```

### Custom model

```ts
const result = await countTokens({
  content: 'Hello',
  config: { model: 'claude-haiku-4-5', },
},);
```

## CLI

```bash
token-count CLAUDE.md
token-count --model claude-haiku-4-5 file1.md file2.md
```

Output follows `wc`-style formatting (right-aligned count,
 path):

```text
4700 CLAUDE.md
```

Multiple files include a total:

```text
4700 CLAUDE.md
1200 README.md
5900 total
```

## API

### `countTokens({ content, config? })`

Count input tokens for a text string.

- **content**:
   text to tokenize
- **config.
  model**:
   Claude model (default:
   `claude-sonnet-4-6`)
- **config.
  apiKey**:
   Anthropic API key (default:
   env var fallback chain above)

Returns `{ inputTokens, model }`.

### `countFileTokens({ filePath, config? })`

Read a file and count its tokens.

Same config as `countTokens`.
Returns `{ inputTokens, model, filePath }`.

### `DEFAULT_MODEL`

The default model used for tokenization (`claude-sonnet-4-6`).
