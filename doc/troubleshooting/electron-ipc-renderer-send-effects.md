# Electron 43.1.0 `ipcRenderer.send` serialization needs an explicit readonly-effect boundary

## Symptom

The project-owned readonly rule reports this error when a preload bridge forwards renderer state through
`ipcRenderer.send`:

```text
The function input named "state" is used by these calls: ipcRenderer.send.
This rule cannot inspect enough of those calls to know what they might change.
```

The affected surface is
`package/file-manager/variant-electron/src/preload.ts`.
The state is a plain immutable DTO,
but Electron's declaration does not describe serialization,
caller-code invocation,
or retention.

## Root cause

The installed `electron@43.1.0/electron.d.ts:9255` declaration exposes only a callable type:

```ts
send(channel: string, ...args: any[]): void;
```

Electron tag `v43.1.0`,
commit `b5c102b3f0f7e9c5be064ba337a3547a83cf7d09`,
implements that declaration in `lib/renderer/api/ipc-renderer.ts:8-11` by collecting every supplied argument and passing
the array to the native binding:

```ts
class IpcRenderer extends EventEmitter implements Electron.IpcRenderer {
  send(channel: string, ...args: any[]) {
    return ipc.send(internal, channel, args);
  }
```

`shell/renderer/api/electron_api_ipc_renderer.cc:71-84` synchronously serializes that argument array before sending the
message:

```cpp
blink::CloneableMessage message;
if (!electron::SerializeV8Value(isolate, arguments, &message)) {
  return;
}
electron_ipc_remote_->Message(internal, channel, std::move(message));
```

`shell/common/v8_util.cc:46-67` calls V8's value serializer,
releases its bytes into an owned message buffer,
and stores that buffer in the `CloneableMessage`:

```cpp
if (!serializer_.WriteValue(isolate_->GetCurrentContext(), value)
         .To(&wrote_value)) {
  isolate_->ThrowException(v8::Exception::Error(
      gin::StringToV8(isolate_, "An object could not be cloned.")));
  return false;
}
// ...
const auto [data_bytes, data_len] = serializer_.Release();
// ...
out->owned_encoded_message = std::move(data_);
out->encoded_message = out->owned_encoded_message;
```

Electron therefore copies serializable values before transport instead of retaining the original JavaScript object.
The serializer is still an external native capability.
Its TypeScript declaration cannot prove whether reading a caller-owned object invokes accessors,
proxy traps,
or other user code.
The semantic rule must preserve that uncertainty rather than treating a `void` return as evidence of observation only.

Electron's `doc/api/ipc-renderer.md:118-136` independently documents that arguments use the Structured Clone Algorithm,
prototype chains are omitted,
and unsupported values throw.

## Verification

Verified identities:

- installed npm package `electron@43.1.0`;
- upstream tag `v43.1.0` at commit `b5c102b3f0f7e9c5be064ba337a3547a83cf7d09`;
- installed declaration `electron.d.ts`;
- JavaScript wrapper,
  native renderer binding,
  V8 serializer wrapper,
  and API documentation from that tag.

A comparable current-V8 Structured Clone probe demonstrates why getter effects cannot be discarded:

```sh
node --input-type=module -e "let getterCalls=0;let value='before';const input={get value(){getterCalls++;return value}};const clone=structuredClone(input);value='after';console.log(JSON.stringify({getterCalls,cloneValue:clone.value,currentValue:value}))"
```

Verified output:

```json
{"getterCalls":1,"cloneValue":"before","currentValue":"after"}
```

This probe supports the conservative caller-code boundary.
The Electron source trace,
not the Node probe,
proves that Electron owns copied serialized bytes before IPC transport.

### Working catalog

- strings,
  numbers,
  booleans,
  arrays,
  and plain state objects are documented Structured Clone inputs;
- the native binding serializes arguments before calling the IPC remote;
- changing a caller object after serialization cannot change the already owned encoded message.

### Failing or uncertain catalog

- functions,
  promises,
  symbols,
  weak maps,
  and weak sets throw during Electron IPC serialization;
- DOM and unsupported native-backed objects throw;
- Electron's declaration carries no effect metadata;
- native V8 property observation remains an external caller-code boundary to the TypeScript rule.

## Verified workaround

The preload bridge now accepts the renderer-originated value as
`ForeignBorrowed<ObservedStripState>` and isolates the native call in a named `reportState` function.
Its contract says:

```ts
@mutates state - `ipcRenderer.send` serializes state and may invoke caller-owned property accessors while copying it.
```

This preserves the original IPC payload and allocation behavior while documenting the complete unresolved effect.
The ownership marker is appropriate at this context-bridge ingress;
descendants do not repeat it.

Tradeoff:
the contract remains conservative even though this application's current DTO has primitive readonly fields.
That conservatism protects future fields and reflects the native serializer boundary.

## What does not work

- **Treating `send(...): void` as observational:
  ** return type says nothing about serialization or caller-code execution.
- **Adding an Electron package-catalog effect:
  ** the npm package ships a launcher and declarations,
  while authoritative behavior lives in the versioned native Electron binary and upstream C++ source.
  Package implementation inference cannot prove that mapping from shipped JavaScript alone.
- **Copying the DTO before calling Electron:
  ** another shallow copy can itself invoke accessors,
  adds an allocation,
  and does not remove the external serialization boundary.
- **Claiming direct mutation:
  ** the audited implementation serializes into owned bytes;
  uncertainty concerns observation and caller-code hooks,
  not evidence that Electron writes into the original DTO.
- **Launching the disposable Electron renderer probe on this host:
  ** both `--headless` and hidden Wayland launches remained blocked before `app.whenReady()` and emitted no diagnostic.
  The processes were terminated and no runtime result was used as evidence.

## Upstream filing decision

No `.out-of-scope/` entry covers Electron IPC.
Searches of open and closed Electron issues and pull requests for
`ipcRenderer send structured clone getter serialization`,
`structured clone ipcRenderer`,
and `An object could not be cloned` found no matching report.
No report should be opened:

1.  **Is it really upstream's fault?
    ** No.
    Electron documents and implements Structured Clone serialization.
    Missing effect semantics are a limitation of TypeScript declarations and this project's analyzer boundary.
2.  **Can upstream fix it?
    ** Electron could add prose,
    but its existing prose already describes serialization and unsupported values.
    TypeScript has no standard declaration syntax for caller-code effects.
3.  **Are they supporting this use case?
    ** Yes.
    Renderer-to-main IPC with serializable arguments is documented.
4.  **Would the repository welcome our contribution?
    ** Electron's `CONTRIBUTING.md` accepts reviewed contributions and requires human review under its linked AI Tool
    Policy.
    That policy forbids autonomous posting and requires contributors to understand and test submitted work.
5.  **Will they likely fix it?
    ** There is no defect or missing Electron behavior to fix.
6.  **Was a minimal upstream fix prototyped?
    ** No.
    Constraints 1 and 5 fail,
    and the correct fix is the verified consumer-side ownership and effect contract.

The upstream filing artifact is therefore intentionally empty.
Posting an issue would incorrectly ask Electron to encode a project-specific semantic effect model in a TypeScript
surface that cannot express it.

## Sources

- [Electron `ipcRenderer` API][ipc-renderer]
- [Electron IPC tutorial][ipc-tutorial]
- [Electron AI Tool Policy][electron-ai-policy]

[ipc-renderer]: https://github.com/electron/electron/blob/b5c102b3f0f7e9c5be064ba337a3547a83cf7d09/docs/api/ipc-renderer.md
[ipc-tutorial]: https://github.com/electron/electron/blob/b5c102b3f0f7e9c5be064ba337a3547a83cf7d09/docs/tutorial/ipc.md
[electron-ai-policy]: https://github.com/electron/governance/blob/main/policy/ai.md
