# @monochromatic-dev/observable-core

Observable value containers with method-based get and set and change notification.

Two variants share one shape:
 a container holding a single value,
 read with `getValue()`
and written with `setValue()`.
 Writing stores the value and notifies a change handler.
The synchronous variant (`createObservable`) calls the handler synchronously;
the asynchronous variant (`createObservableAsync`) awaits it.

This is a method-based API.
 There is no `value` property;
 reads go through `getValue()`
and writes through `setValue()`.

## Method-based API

Both variants expose the same two methods:

- `getValue()` returns the current value synchronously.
- `setValue(newValue)` stores `newValue`,
   then invokes the change handler with the new
  value followed by the previous value.

## State updates before the handler runs

`setValue` writes the new value into the container before it calls `onChange`.
A handler that calls `getValue()` therefore observes the new value,
 not the previous one.
The previous value is still available as the handler's second argument.

```ts
import { createObservable, } from '@monochromatic-dev/observable-core';

const count = createObservable({
  initialValue: 0,
  onChange: function onChange(next, prev) {
    // count.getValue() === next here, because state already moved.
    console.log(`${prev} -> ${next}, current ${count.getValue()}`,);
  },
});

count.setValue(1,); // logs "0 -> 1, current 1"
```

## Sync versus async

The two variants differ only in how `setValue` treats the handler.

Synchronous,
 `createObservable`:

- `setValue` returns `void`.
- `onChange` is `(newValue, oldValue) => void`.
- A thrown error from `onChange` propagates out of `setValue` synchronously,
   so wrap the call
  in a `try` or assert on the throwing call directly.

Asynchronous,
 `createObservableAsync`:

- Construction returns a promise;
   `await` it.
   Construction itself is synchronous,
   but the
  promise lets callers `await` the container in top-level module code.
- `setValue` is `async` and returns `Promise<void>`;
   it awaits `onChange` before resolving.
- `onChange` is `(newValue, oldValue) => void | Promise<void>`.
- Because `setValue` awaits the handler,
   a rejected handler rejects the `setValue` promise.
  This differs from a fire-and-forget design:
   callers can `await setValue(...)` and catch the
  rejection.

## Usage

Synchronous container:

```ts
import { createObservable, } from '@monochromatic-dev/observable-core';

const theme = createObservable({
  initialValue: 'light',
  onChange: function onChange(next, prev) {
    document.documentElement.dataset['theme'] = next;
    console.log(`theme: ${prev} -> ${next}`,);
  },
});

theme.getValue(); // 'light'
theme.setValue('dark',); // handler runs synchronously
theme.getValue(); // 'dark'
```

Asynchronous container with an awaited handler:

```ts
import { createObservableAsync, } from '@monochromatic-dev/observable-core';

const feeds = await createObservableAsync({
  initialValue: [] as readonly string[],
  onChange: async function onChange(next, prev) {
    await persist(next,);
  },
});

await feeds.setValue(['a', 'b',],); // resolves only after persist completes
feeds.getValue(); // ['a', 'b']
```

Propagating a synchronous handler error:

```ts
import { createObservable, } from '@monochromatic-dev/observable-core';

const limited = createObservable({
  initialValue: 0,
  onChange: function onChange(next) {
    if (next < 0) {
      throw new Error('value must not be negative',);
    }
  },
});

try {
  limited.setValue(-1,); // throws synchronously
}
catch (error) {
  console.error(error,);
}
```

Propagating an async handler rejection:

```ts
import { createObservableAsync, } from '@monochromatic-dev/observable-core';

const saved = await createObservableAsync({
  initialValue: 0,
  onChange: async function onChange(next) {
    await saveOrThrow(next,);
  },
});

try {
  await saved.setValue(42,); // rejects if saveOrThrow rejects
}
catch (error) {
  console.error(error,);
}
```

## Types

```ts
import type {
  Observable,
  ObservableAsync,
} from '@monochromatic-dev/observable-core';
```

- `Observable<T>`:
   `{ getValue: () => T; setValue: (newValue: T) => void }`.
- `ObservableAsync<T>`:
   `{ getValue: () => T; setValue: (newValue: T) => Promise<void> }`.
