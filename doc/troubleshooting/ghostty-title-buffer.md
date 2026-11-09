# Ghostty 63e75e86 ignores OSC title payloads at 256 UTF-8 bytes

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

Ghostty can leave a stale terminal title visible when an agent integration sends an OSC title payload
whose UTF-8 byte length is 256 bytes or longer.

The visible effect is not a malformed title.
Ghostty ignores the title change entirely,
so the previous title remains in the terminal tab or window.

In this repository the affected surfaces are terminal-title integrations:

- `package/pi-plugin/terminal-title/src/index.ts`,
  which sends title text through `ctx.ui.setTitle()`.
- `package/claude-code-plugin/source/src/handlers/terminal-title/index.ts`,
  which writes OSC 0 title sequences to `/dev/tty`.

`package/claude-code-plugin/source/src/handlers/terminal-title/index.ts` is not in the working tree
and no commit in this repository carries it.

## Root cause

Ghostty's stream handler receives OSC title payload text as `[]const u8`,
so the limit is byte-counted,
not JavaScript character-counted.

Verified source clone:

```sh
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
gh repo clone ghostty-org/ghostty /tmp/agent/ghostty-title-buffer-269 -- --depth 1
git -C /tmp/agent/ghostty-title-buffer-269 rev-parse HEAD
# 63e75e86c282ca1d07de9588f0c2cfc268b2621b
```

`src/termio/stream_handler.zig:1012-1016` allocates a fixed 256-byte buffer
and returns without setting the title when the payload length reaches that buffer size:

```zig
fn windowTitle(self: *StreamHandler, title: []const u8) !void {
    var buf: [256]u8 = undefined;
    if (title.len >= buf.len) {
        log.warn("change title requested larger than our buffer size, ignoring", .{});
        return;
    }
```

The branch means:

- `title.len == 255` continues to `self.terminal.setTitle(title)`.
- `title.len == 256` logs the warning and returns.
- `title.len > 256` logs the warning and returns.

## Verification

The relevant source still contains the fixed buffer and reject branch in
`ghostty-org/ghostty@63e75e86c282ca1d07de9588f0c2cfc268b2621b`:

```sh
cd /tmp/agent/ghostty-title-buffer-269
rg --line-number "change title requested|var buf: \\[256\\]u8|title\\.len >= buf\\.len" src/termio/stream_handler.zig
# 1013:        var buf: [256]u8 = undefined;
# 1014:        if (title.len >= buf.len) {
# 1015:            log.warn("change title requested larger than our buffer size, ignoring", .{});
```

A minimal Zig harness reproduces the branch predicate from Ghostty's source:

```zig
// /tmp/agent/ghostty-title-buffer-check.zig
const std = @import("std");

pub fn main() void {
    const cases = [_]usize{ 255, 256, 257 };
    for (cases) |title_len| {
        const ignored = title_len >= 256;
        std.debug.print("title_len={d} ignored={any}\n", .{ title_len, ignored });
    }
}
```

```sh
cd /tmp/agent
zig run ghostty-title-buffer-check.zig
# title_len=255 ignored=false
# title_len=256 ignored=true
# title_len=257 ignored=true
```

Working catalog:

- 255 UTF-8 bytes:
  accepted by the branch predicate and eligible for `self.terminal.setTitle(title)`.

Failing catalog:

- 256 UTF-8 bytes:
  rejected by `title.len >= buf.len`.
- Any larger payload:
  rejected by the same branch.

Repository verification for the workaround:

```sh
cd /var/home/user/Monochromatic
mise run //package/agent-harness-shared/terminal-title:buildAndTest
mise run //package/pi-plugin/terminal-title:test:unit
mise run //package/claude-code-plugin/source:test:unit
```

These tests cover ASCII over the limit,
exactly safe byte length,
unsafe byte length,
multi-byte BMP text,
emoji surrogate pairs,
and existing short-title behavior.

## Verified workarounds

### Cap final title payload text to 255 UTF-8 bytes

Use `safeTerminalTitlePayload()` from `@monochromatic-dev/agent-harness-shared-terminal-title/ts`
at the final terminal-title output boundary:

```ts
import { safeTerminalTitlePayload } from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

const payload = safeTerminalTitlePayload({ value: title });
```

The helper replaces OSC-breaking controls with visible control pictures,
uses `TextEncoder` for UTF-8 byte measurement,
iterates with `for...of` so surrogate pairs are not split,
and appends `…` only when the ellipsis fits inside the byte budget.

Tradeoff:
long titles are shortened before reaching the terminal.
This is intentional because a shortened title is fresher and more useful than Ghostty preserving a stale title.

### Keep display-length caps as a separate compactness policy

The existing 60-character display cap can remain.
It keeps common titles compact,
but it is not the Ghostty compatibility boundary because JavaScript string length is not UTF-8 byte length.

Tradeoff:
there are now two caps with different meanings:
display compactness and terminal byte safety.
The code comments and tests must keep that distinction visible.

## What does not work

### Sending exactly 256 bytes

A 256-byte payload still satisfies Ghostty's reject condition:

```zig
const ignored = title_len >= 256;
```

The repository cap therefore uses 255 bytes,
not 256 bytes.

### Relying only on JavaScript string length

A JavaScript character or code-unit cap does not encode Ghostty's boundary.
Multi-byte BMP characters and emoji can use several UTF-8 bytes per displayed character.

### Truncating the complete OSC escape sequence

The byte cap must apply to title payload text before the OSC sequence is constructed.
Truncating `ESC ] 0 ; ... BEL` as one byte string risks emitting a partial escape sequence.
The Claude Code handler caps the payload text first,
then wraps it with complete OSC delimiters.

## Upstream filing decision

Duplicate search:

```sh
gh issue list --repo ghostty-org/ghostty --state all --search "title buffer 256" --json number,title,state,url --limit 20
gh issue list --repo ghostty-org/ghostty --state all --search "\"change title requested larger than our buffer size\"" --json number,title,state,url --limit 20
```

The exact warning search returned no issues.
The broad search returned unrelated issues,
including `ghostty-org/ghostty#256`,
which is about PC-style function-key escape sequences rather than title payload length.

- Is it really upstream's fault?
  Partly.
  Ghostty owns the fixed buffer and ignore branch,
  but applications can avoid the branch by respecting the byte limit.
- Can upstream fix it?
  Yes.
  Ghostty could allocate dynamically or accept a larger title buffer.
- Are they supporting this use case?
  OSC title support is implemented by `windowTitle`,
  but over-255-byte title payloads are explicitly rejected in the current implementation.
- Would the repo welcome our contribution?
  Not checked beyond issue search because this repository has a complete consumer-side workaround
  and does not need an upstream change to resolve its stale-title risk.
- Will they likely fix it?
  Unknown.
  No matching upstream issue or maintainer signal was found in the duplicate search.
- Have we prototyped a minimal fix compatible with their architecture?
  No upstream patch was prototyped.
  The verified fix for this repository is the consumer-side 255-byte payload cap.

Decision:
do not file upstream now.
The local workaround is small,
verified,
and avoids creating an upstream report for behavior this repository can handle at its output boundary.
