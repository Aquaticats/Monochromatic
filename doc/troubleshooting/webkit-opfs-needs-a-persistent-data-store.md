# Playwright 1.65.0's ephemeral WebKit 27.2 context refuses `navigator.storage.getDirectory()`, so OPFS tests fail or skip on webkit only

## Symptom

In the `monochromatic-playwright` image (Playwright 1.65.0 with WebKit 27.2,
Chromium 158.0.8127.18 and Firefox 159.0),
inside the container that `mise run test:browser` starts,
the repository's browser suite fails on the webkit project
alone:

```text
  1) [webkit] › package/logger/core/src/sink/opfs.browser.test.ts:36:3 › OPFS sink › verify detects OPFS availability

    Error: expect(received).toBe(expected) // Object.is equality

    Expected: true
    Received: false
```

Two further OPFS tests report themselves skipped on webkit rather than failing,
each quoting the browser's own words:

```text
  skipped OPFS sink accepts every record with no breadcrumb: origin-private file system refused: The operation failed for
  an unknown transient reason (e.g. out of memory).
```

The page-level refusal behind all three is one rejected promise.
The same probe page,
in the same image and run,
gives:

```json
{
  "webkit": {
    "secureContext": true,
    "hasGetDirectory": "function",
    "getDirectory": { "outcome": "threw", "name": "UnknownError",
                      "message": "The operation failed for an unknown transient reason (e.g. out of memory)." },
    "estimate": { "quota": 1048576000, "usage": 0 }
  },
  "chromium": { "getDirectory": { "outcome": "ok" }, "fileWrite": { "outcome": "ok" }, "estimate": { "quota": 10737418409, "usage": 169 } },
  "firefox":  { "getDirectory": { "outcome": "ok" }, "fileWrite": { "outcome": "ok" }, "estimate": { "quota": 10737418240, "usage": 491521 } }
}
```

`navigator.storage` exists,
`getDirectory` is a function,
and `estimate()` answers with a quota,
so the Storage API is
present and the origin is secure.
Only the file system half refuses,
and it refuses on `getDirectory`,
on
`getFileHandle` and on `createWritable` alike.

## Root cause

WebKit implements the origin private file system on a directory under the session's website data store,
and an
ephemeral (non-persistent) website data store has no directory at all.
Playwright Test creates every test context with
`browser.newContext()`,
which is ephemeral,
so webkit gets no directory,
and `getDirectory()` is where the absence
surfaces.

The chain,
each step a source citation:

1.  Playwright Test's `context` fixture calls `browser.newContext()`.
    On webkit that is
    [`WKBrowser.doCreateNewContext`](https://github.com/microsoft/playwright/blob/a1f35e1955976e84411f9c10358228a80b763ebe/packages/playwright-core/src/server/webkit/wkBrowser.ts#L82-L92),
    which sends `Playwright.createContext` with no data store:
    `const { browserContextId } = await this._browserSession.send('Playwright.createContext', createOptions);`.
    Only `launchPersistentContext` reaches the branch above it that keeps a context on disk
    (`if (options.persistent) { browser._defaultContext = new WKBrowserContext(browser, undefined, options.persistent); }`,
    same file,
    lines 52 to 55).
2.  A non-persistent data store is created without any directory.
    The configuration constructor runs `initializePaths()`
    only for a persistent store:
    [`WebsiteDataStoreConfiguration.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/UIProcess/WebsiteData/WebsiteDataStoreConfiguration.cpp#L53-L58)

    ```cpp
    if (isPersistent == IsPersistent::Yes && shouldInitializePaths == ShouldInitializePaths::Yes) {
    ```

    so `generalStorageDirectory` keeps its empty default.
    The same file sets it only in the persistent path
    (`setGeneralStorageDirectory(WebsiteDataStore::defaultGeneralStorageDirectory(m_baseDataDirectory));`),
    and
    [`WebsiteDataStore.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/UIProcess/WebsiteData/WebsiteDataStore.cpp#L189-L190)
    guards its use on persistence:
    `if (auto directory = m_configuration->generalStorageDirectory(); isPersistent() && !directory.isEmpty())`.
    WebKitGTK's public documentation states the same contract for its ephemeral manager:
    [`WebKitWebsiteDataManager.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/UIProcess/API/glib/WebKitWebsiteDataManager.cpp#L60-L62),
    "A WebKitWebsiteDataManager can be ephemeral,
    in which case all the directory configuration is not needed because
    website data will never persist".
3.  The network process keeps that empty string as its storage root and derives every origin directory from it,
    returning
    an empty path for an empty root:
    [`NetworkStorageManager.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/NetworkProcess/storage/NetworkStorageManager.cpp#L124-L127)

    ```cpp
    static String originDirectoryPath(const String& rootPath, const WebCore::ClientOrigin& origin, FileSystem::Salt salt)
    {
        if (rootPath.isEmpty())
            return emptyString();
    ```

4.  The per-origin storage bucket therefore has an empty root,
    and the file system path built from it is empty too:
    [`OriginStorageManager.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/NetworkProcess/storage/OriginStorageManager.cpp#L208-L222)

    ```cpp
    String OriginStorageManager::StorageBucket::typeStoragePath(StorageType type) const
    {
        auto storageIdentifier = toStorageIdentifier(type);
        if (m_rootPath.isEmpty() || storageIdentifier.isEmpty())
            return emptyString();
        return FileSystem::pathByAppendingComponent(m_rootPath, storageIdentifier);
    }
    ...
        m_fileSystemStorageManager = FileSystemStorageManager::create(typeStoragePath(StorageType::FileSystem), registry, origin, WTF::move(quotaCheckFunction));
    ```

5.  `getDirectory` hands that empty path to `createHandle`,
    which rejects an empty path outright:
    [`FileSystemStorageManager.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/NetworkProcess/storage/FileSystemStorageManager.cpp#L169-L173)

    ```cpp
    std::expected<...> FileSystemStorageManager::getDirectory(IPC::Connection::UniqueID connection)
    {
        ASSERT(!RunLoop::isMain());
        return createHandle(connection, FileSystemStorageHandle::Type::Directory, String { m_path }, { }, true);
    }
    ```

    and,
    in the same file,
    [`FileSystemStorageManager.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/NetworkProcess/storage/FileSystemStorageManager.cpp#L82-L83)

    ```cpp
        if (path.isEmpty())
            return makeUnexpected(FileSystemStorageError::Unknown);
    ```

6.  `FileSystemStorageError::Unknown` becomes `ExceptionCode::UnknownError`,
    whose message is the one the page sees:
    [`FileSystemStorageError.h`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebKit/NetworkProcess/storage/FileSystemStorageError.h#L69-L77)

    ```cpp
        case FileSystemStorageError::Unknown:
            break;
        }
        return WebCore::Exception { WebCore::ExceptionCode::UnknownError };
    ```

    and
    [`DOMException.cpp`](https://github.com/WebKit/WebKit/blob/3a2cd4ea9ef8700615250a918d28e5166a707876/Source/WebCore/dom/DOMException.cpp#L63)

    ```cpp
    DOMException::Description { "UnknownError"_s, "The operation failed for an unknown transient reason (e.g. out of memory)."_s, 0 },
    ```

The words "unknown transient reason (e.g. out of memory)" are a generic `UnknownError` message,
not a report about this
machine:
a second `getDirectory()` in the same page throws the same error,
at 2 GiB and at 8 GiB of container memory,
and chromium and firefox pass the same pages in the same container.

Upstream Playwright documents the same limitation in the `storageState({ opfs: true })` note,
"OPFS is currently not
supported in ephemeral WebKit contexts"
([`class-browsercontext.md`](https://github.com/microsoft/playwright/blob/a1f35e1955976e84411f9c10358228a80b763ebe/docs/src/api/class-browsercontext.md#L1574-L1577)).

## Verification

Under test:
the `monochromatic-playwright` image built from `playwright.Dockerfile` on top of
`mcr.microsoft.com/playwright:v1.65.0-noble`,
whose `@playwright/test` is 1.65.0 and whose
`node_modules/@playwright/test` matches the store copy of that version.

Harness:
one node script in the container that serves a blank page on `127.0.0.1:4321` and asks each browser for the
same page calls.
Every step prints before it starts and is raced against a 15 s deadline,
so a call that never settles
names itself instead of holding the container open:

```bash
podman run --rm --security-opt label=disable \
  -v "$PWD:/work" -v "$HOME/.local/share/pnpm/store:$HOME/.local/share/pnpm/store:ro" \
  -v "$HOME/temp/agent/browser-fit:/probe:ro" -w /work \
  monochromatic-playwright node /probe/opfs-probe.ts
```

```ts
// The page calls of the probe, one evaluate per API so a caller that never settles names itself.
observations.getDirectory = await tab.evaluate(async () => {
  try {
    const root = await navigator.storage.getDirectory();
    return { outcome: 'ok', rootKind: Object.prototype.toString.call(root,) };
  }
  catch (error) {
    return { outcome: 'threw', name: (error as Error).name, message: (error as Error).message };
  }
});
```

What works,
measured 2026-11-08T22:5x-05:00 inside the image:

- `webkit.launchPersistentContext(<directory>)`:
  `getDirectory` returns `[object FileSystemDirectoryHandle]`,
  `createWritable` is a function,
  a written byte reads back as `x`,
  and `estimate()` reports `usage: 1`.
- `chromium.launch()` with `newContext()`:
  directory handle,
  write round-trip,
  `estimate().usageDetails.fileSystem = 169`.
- `firefox.launch()` with `newContext()`:
  directory handle,
  write round-trip,
  `estimate().usage = 491521`.

What fails,
measured the same way:

- `webkit.launch()` with `newContext()`:
  `getDirectory` throws `UnknownError` with the message above,
  and so do
  `getFileHandle` and `createWritable` once a directory handle comes from anywhere else.
- `firefox.launch()` with `newContext()`:
  `navigator.storage.persist()` never settled.
  The step printed
  `TIMEOUT firefox/ephemeral persist at 15000 ms` and the following steps ran,
  so the promise neither resolved nor
  rejected within 15 s.
  That is a property of the probe's `persist()` call,
  not of OPFS:
  `getDirectory`,
  the write
  round-trip and `estimate()` all completed in the same firefox page first.

The three repository tests that touch OPFS reproduce the same split:
at `mise run test:browser -- --project=webkit`
before the fix,
`opfs.browser.test.ts` failed one of its five tests,
the fs-path root discovery test skipped,
and the
fuzz OPFS property skipped;
chromium and firefox passed all of them.

## Verified workarounds

### Launch the test's context as a persistent context

What this repository now does,
through `package/logger/core/src/sink/opfs-browser-fixture.ts`,
which overrides Playwright
Test's `context` fixture with `launchPersistentContext` in a fresh temporary directory per test.
The logger sink's OPFS test imports it;
the fs-path root discovery test and the fuzz OPFS property keep their own skip,
because a shared fixture would need a private harness package both of them already depend on.
Measured at
`mise run test:browser` on 2026-11-08T23:40:35-05:00,
host load average 55 on 8 CPUs:
83 passed and 1 skipped in 48.1 s,
the skip being the fuzz OPFS property,
against 82 passed,
1 skipped and 1 failed in 21.1 s before it.
With the same fixture on all three OPFS files the run was 84 passed in 34.6 s,
but that variant could not be type-checked from the other two packages (see What does not work).

Tradeoffs:

- Each test pays one browser launch,
  about 2 s per test on webkit against 0.3 s for an ephemeral context,
  because a
  persistent context cannot be shared across tests without sharing its storage.
- Storage written by one test survives only until that test's context closes,
  because the directory is fresh per test
  and removed with it.
  A test that needs to observe its own residue across two contexts cannot use it.
- The context is no longer the runner's `browser.newContext()`,
  so the runner's artifact hooks see a context it did
  not create;
  screenshots and traces of a failure therefore arrive when the context closes.

### Keep the ephemeral default and declare OPFS unavailable to the sink

The sink's `verify()` already returns `false` on any refusal,
and the fuzz property already turns the refusal into a
skip reason.
That is a workaround for the build,
not for the test:
it asserts that webkit cannot do what a real Safari
session does,
and it is the skip this repository removed rather than kept.
Tradeoff:
coverage of the OPFS sink drops to
two of the three browsers,
and nothing notices a regression in the sink's webkit path.

## What does not work

- **Retrying.**
  The message says "unknown transient reason",
  but the second `getDirectory()` in the same page throws the
  same `UnknownError`,
  and a fresh page in a fresh ephemeral context throws it too.
  Nothing is transient.
- **A larger memory bound.**
  The refusal is identical with the container at 8 GiB of memory and 8 CPUs and with it at
  2 GiB and 2 CPUs.
  The error's "(e.g. out of memory)" wording is a fixed `UnknownError` message.
- **`navigator.storage.persist()` first.**
  It answers `false` in the ephemeral webkit context and changes nothing about
  the following `getDirectory()`.
  Persistence of the origin's data is not what OPFS needs;
  a directory on disk is,
  and
  only the data store decides that.
- **A different origin.**
  The refusal is not about `http://localhost` being insecure:
  the page reports
  `isSecureContext: true`,
  `estimate()` answers with a 1 GB quota,
  and the same origin works in the same container once
  the context is persistent.
- **Passing WebKit settings through Playwright.**
  `launchPersistentContext` is the only lever Playwright exposes over
  the data store;
  the browser type's launch arguments do not carry one.

## Upstream filing decision

Nothing to file,
and no draft is kept as fileable.
The duplicate search found the behavior already documented and
already reported:

- Playwright's own docs carry the note quoted under Root cause (v1.65.0,
  and it is still on `main`).
- `microsoft/playwright` #42579,
  "[Bug]:
  storageState with opfs:
  [] cannot be restored in WebKit,
  and capturing OPFS
  there fails outright",
  closed,
  quotes the same `Unable to restore OPFS: The operation failed for an unknown transient
  reason (e.g. out of memory).` and links the docs note.
- `microsoft/playwright` #29901,
  "[Bug]:
  Origin Private File System is not isolated between tests",
  closed.
- `microsoft/playwright` pull request #42570,
  "test:
  fix referer check on msedge,
  drop persistent OPFS test",
  merged,
  removed a test named "should round-trip OPFS in a persistent WebKit context" and gives the reason as a failure on
  WebKit Windows.
  That removed test is upstream's evidence,
  independent of this machine,
  that a persistent webkit
  context does serve OPFS on the other platforms.

The six-constraint check:

1.  **Upstream's fault?**
    It is WebKit's architectural restriction for ephemeral data stores,
    surfaced through
    Playwright's ephemeral-by-default test contexts.
    Neither is a wording problem,
    and both are documented.
2.  **Can upstream fix it?**
    A fix would mean an in-memory OPFS backend inside WebKit,
    which is a design change to
    WebKit's storage layer,
    not a bug fix.
    Playwright cannot fix it from `launchPersistentContext` alone because its
    test runner has no persistent context fixture.
3.  **Are they supporting the use case?**
    Playwright runs a persistent-context OPFS test upstream,
    and documents the
    limitation;
    the combination we need,
    "assert OPFS through the test runner",
    is a supported library call.
4.  **Would the repo welcome a contribution?**
    Not reached:
    there is nothing additive to report.
    Its `CONTRIBUTING.md`
    and issue templates were not audited for this decision because no filing is proposed.
5.  **Will they likely fix it?**
    The docs note is the opposite signal,
    a documented limitation.
6.  **Prototyped minimal fix?**
    Not applicable upstream.
    The consumer-side fix (a persistent context in our fixture) is
    prototyped,
    measured and committed in this repository.

`microsoft/playwright` runs a trust system on issue authors;
that risk is recorded here for completeness only,
since
nothing is filed and the decision to file would be the poster's,
not this document's.
