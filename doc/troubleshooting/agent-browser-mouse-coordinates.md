# agent-browser 0.36.0: fractional mouse coordinates report missing arguments

## Symptom

The native-pointer shop verifier supplied both coordinates:

```text
agent-browser --session promises-shop-sending-113745 --allow-file-access --json mouse move 360.5546875 288
```

Process `proc_3a83` received:

```json
{"error":"Missing arguments for: mouse move\nUsage: agent-browser mouse move <x> <y>","success":false,"type":"missing_arguments"}
```

The arguments were present.
The first coordinate was fractional.
This is distinct from the partially visible element-center incident in
`agent-browser-partial-viewport-click.md`.

## Root cause

The installed CLI reports version `0.36.0`.
The inspected release source is commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`,
in the read-only clone `~/temp/agent/agent-browser-quality-explorer-2026-09-06`.

`cli/src/commands.rs:2974` defines `parse_mouse`.
After retrieving both arguments,
lines 2987 to 2998 parse each as a signed integer
and classify conversion failure as missing arguments:

```rust
// cli/src/commands.rs:2987
let x = x_str
    .parse::<i32>()
    .map_err(|_| ParseError::MissingArguments {
        context: "mouse move".to_string(),
        usage: "mouse move <x> <y>",
    })?;
```

The `y` conversion repeats that pattern at line 2993.
Line 2999 constructs the `mousemove` command only after both conversions succeed.
The observed failure is at this CLI argument boundary,
not evidence that the shop failed to handle a click.

## Verification

The runnable fixture is `~/temp/agent/promises-revision/verify-shop-sending.mjs`:

```bash
# From the private Promise-revision authoring directory.
mise run test:shop-sending
```

Failing catalog:

- `proc_3a83` passed `360.5546875 288` and received `missing_arguments`.
- Keeping both positional arguments does not repair an invalid integer representation.

Working catalog:

- `proc_1b08` rounded the translated centers,
  hit-tested the resulting integer points,
  and completed native mouse move/down/up input in the opaque iframe.
- The iframe recorded both Send events with `event.isTrusted === true`.
  Both questions entered the transcript before either reply.

## Verified workaround

Round at the mouse-command boundary,
then hit-test the point that will actually be sent:

```javascript
// Consumer-side pointer helper; no installed CLI changes.
const x = Math.round(frameLeft + frameBorder + childCenterX);
const y = Math.round(frameTop + frameBorder + childCenterY);
```

The fixture verifies that the translated point hits its owned iframe before dispatching input.
It also checks the resulting message state and trusted Send events.
Rounding changes the requested point;
do not assume an integer point remains inside a narrow or clipped target.
The inspected shop controls provide an interior click target.

## What does not work

Passing `getBoundingClientRect()` coordinates unchanged failed at numeric parsing.
Treating the error as proof of absent positional arguments would diagnose the wrong input property.
No retry against the same fractional arguments was needed.

## Upstream filing decision

Nothing is filed or drafted from this consumer-input correction.

1.  Upstream fault:
    integer parsing is explicit;
    the harness supplied a fractional coordinate.
    The diagnostic wording obscures that distinction,
    but no upstream behavior change is proposed.
2.  Upstream fixability:
    no upstream fix is needed for the verified integer-coordinate caller.
3.  Supported use:
    the parser accepts integer `mouse move` coordinates;
    fractional use was not established.
4.  Contribution policy:
    not assessed for a contribution because none is proposed.
5.  Maintainer intent:
    not assessed;
    no claim about willingness to change numeric parsing or wording.
6.  Upstream prototype:
    inapplicable to the consumer-side coordinate conversion tested here.

No third-party source was modified,
and no external issue,
comment,
or pull request was posted.
No tracker or exemption search was needed for a filing because no filing is proposed.
