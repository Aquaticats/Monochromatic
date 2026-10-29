## module-async-time

Ready to publish.

Async timing primitives.

### Helpers

<table>
<thead>
<tr>
<th>Function</th>
<th>Behavior</th>
</tr>
</thead>
<tbody>
<tr>
<td>`wait`</td>
<td>Resolves after a delay in milliseconds; thin `setTimeout` wrapper</td>
</tr>
<tr>
<td>`withTimeout`</td>
<td>Races a promise against a deadline; rejects with a labeled `Error` if the deadline wins</td>
</tr>
</tbody>
</table>

### Usage

```ts
import {
  wait,
  withTimeout,
} from '@monochromatic-dev/async-time-core';

await wait(500,);

const data = await withTimeout({
  promise: fetch('/api/data',),
  ms: 5000,
  label: 'fetch user data',
},);
```

### Design decisions

- **Source-only.
  ** No build step;
   consumers import directly from `src/index.ts`.
- **Direct named exports.
  ** No `$` aliasing.
- **One file per helper.
  ** `wait.ts`,
   `with-timeout.ts`;
   `index.ts` re-exports.
- **Family scope.
  ** This package owns deadline- and delay-based async primitives.
  Future additions (`debounce`,
   `throttle`,
   `retry-with-backoff`,
   `cancelable-after`)
  belong here as long as their core abstraction is "operate against a duration.
  "
- **Extracted from `module-es`.
  ** Previously lived at
  `t object/t promise/f/{t number/wait, t object/withTimeout}/r a/p p/index.ts`
  and was exposed via the `./wait` and `./with-timeout` subpath exports.
  Promoting to its own package makes the dep edge honest at the `package.json`
  level:
   consumers that only need an async timer no longer advertise a
  `module-es` dependency.
