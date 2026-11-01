# module-caught-value

Ready to publish.

Formats values caught from `catch` blocks without discarding diagnostic information.

`caughtValueText` returns `Error.message` for genuine Error values.
`caughtValueStack` prefers `Error.stack` and falls back to that message.
For every other value,
both helpers follow JavaScript string conversion so objects can provide diagnostic text through `Symbol.toPrimitive`,
`toString`,
or `valueOf`.
Those hooks can execute caller-defined behavior,
so both functions expose that uncertainty through their `@mutates value` contracts.

```ts
import { caughtValueText, } from '@monochromatic-dev/caught-value-core';

caughtValueText(new Error('offline')); // 'offline'
caughtValueText({ toString: () => 'provider details' }); // 'provider details'
```
