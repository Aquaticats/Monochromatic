# Stable syntax modules over transient domain modules

## Status

Accepted,
2026-06-14.

## Context

Architecture review of `package/hyperscript/core` considered deeper shared Modules above
`hHtml`,
`hDom`,
`hCss`,
and `hXml`,
including page shell and libvirt XML Modules.
Those suggestions mixed long-lived syntax grammar with domain or tool policy that may disappear
while the underlying syntax remains.
For example,
XML is durable enough to justify `hXml`,
but libvirt may not matter to this repository in 10 years.

## Decision

Shared deep Modules should encode long-lived syntax and grammar policy,
not transient domain or tool policy.
`module-hyperscript` stays focused on generic syntax Adapters and may deepen around stable syntax concerns:
escaping,
raw-slot safety,
DOM namespace handling,
XML well-formedness,
and CSS value/type policy.

Domain-specific or tool-specific knowledge stays in the caller area that owns it.
A libvirt XML Module,
app HTML shell Module,
or app icon-set Module can exist locally when it gives Locality to that caller,
but it should not be promoted into `module-hyperscript` or a shared package unless the concept has become a
durable repository-wide concept independent of the current tool.

## Consequences

Future architecture reviews should not re-suggest shared libvirt,
page-shell,
or app-icon Modules only because repeated code exists above `hXml`,
`hHtml`,
or `hDom`.
The deletion test must ask whether deleting the shared Module would spread stable syntax complexity across callers,
not just whether it would deduplicate current domain markup.

Deepening remains appropriate inside generic syntax Modules.
Examples:
distinguish escaped text from rendered HTML at the HTML Seam,
make SVG namespace handling part of a DOM Adapter,
test CSS policy at the `hCss` Interface,
and keep XML escaping and self-closing behaviour local to `hXml`.
