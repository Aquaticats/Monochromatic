# TSDoc 0.16.0 reports `@mutates` as undefined until consumers register a custom block tag

## Symptom

Parsing a declaration containing the project tag:

```typescript
/**
 * Changes caller-owned state.
 *
 * @mutates value - Increments caller-owned count.
 */
```

with an unconfigured `@microsoft/tsdoc` 0.16.0 parser reports:

```text
(4,4): The TSDoc tag "@mutates" is not defined in this configuration
```

Defining the tag without marking it supported can instead report:

```text
(4,4): The TSDoc tag "@mutates" is not supported by this tool
```

The repository's in-house Oxlint TSDoc scanner does not use `@microsoft/tsdoc` at runtime.
This behavior matters for external documentation consumers that parse emitted declarations containing `@mutates`.

## Root cause

Source was inspected at `microsoft/tsdoc` commit `b6cf4d23d01bc4536985007a36eea9ba63972e01`,
the commit checked out by tag `@microsoft/tsdoc_v0.16.0`.

`tsdoc/src/parser/NodeParser.ts:303` to `tsdoc/src/parser/NodeParser.ts:313` looks up every parsed block tag in
consumer configuration before routing its content:

```typescript
const tagDefinition: TSDocTagDefinition | undefined = configuration.tryGetTagDefinitionWithUpperCase(
  docBlockTag.tagNameWithUpperCase
);
this._validateTagDefinition(
  tagDefinition,
  docBlockTag.tagName,
  /* expectingInlineTag */ false,
  docBlockTag.getTokenSequence(),
  docBlockTag
);
```

`tsdoc/src/parser/NodeParser.ts:258` to `tsdoc/src/parser/NodeParser.ts:270` emits the undefined-tag diagnostic when no
definition exists and `ignoreUndefinedTags` is false:

```typescript
} else {
  // The tag is not defined
  if (!this._parserContext.configuration.validation.ignoreUndefinedTags) {
    this._parserContext.log.addMessageForTokenSequence(
      TSDocMessageId.UndefinedTag,
      `The TSDoc tag "${tagName}" is not defined in this configuration`,
      tokenSequenceForErrorContext,
      nodeForErrorContext
    );
  }
}
```

`tsdoc/src/configuration/TSDocConfiguration.ts:115` to
`tsdoc/src/configuration/TSDocConfiguration.ts:138` provides the supported registration boundary:

```typescript
public addTagDefinition(tagDefinition: TSDocTagDefinition): void {
  const existingDefinition: TSDocTagDefinition | undefined = this._tagDefinitionsByName.get(
    tagDefinition.tagNameWithUpperCase
  );

  // ...duplicate-definition checks...

  this._tagDefinitions.push(tagDefinition);
  this._tagDefinitionsByName.set(tagDefinition.tagNameWithUpperCase, tagDefinition);
}
```

A custom `BlockTag` takes the generic-block branch at `tsdoc/src/parser/NodeParser.ts:317` to
`tsdoc/src/parser/NodeParser.ts:348`.
TSDoc stores block content but does not interpret the project's `parameterName - description` grammar.
The external consumer must perform that interpretation.

`tsdoc/src/configuration/TSDocTagDefinition.ts:72` to
`tsdoc/src/configuration/TSDocTagDefinition.ts:85` stores `allowMultiple`:

```typescript
/**
 * If true, then this TSDoc tag may appear multiple times in a doc comment.
 * By default, a tag may only appear once.
 */
public readonly allowMultiple: boolean;

public constructor(parameters: ITSDocTagDefinitionParameters) {
  // ...other fields...
  this.allowMultiple = !!parameters.allowMultiple;
}
```

A full source search found no read of `allowMultiple` outside tag definitions in 0.16.0.
The probe consequently accepted repeated blocks even when that field was false.
Set `allowMultiple: true` to publish intended cardinality to consumers,
but do not treat the field as parser-side duplicate validation.

## Verification

Verified package:

- npm package `@microsoft/tsdoc` 0.16.0;
- npm shasum `2249090633e04063176863a050c8f0808d2b6d2b`;
- npm integrity prefix `sha512-xgAyonlVVS+q7`;
- source tag commit `b6cf4d23d01bc4536985007a36eea9ba63972e01`.

The disposable harness extracted the published tarball and ran:

```bash
mise --cd /var/home/user/temp/agent/tsdoc-mutates-probe-2026-07-13 run probe
```

Its relevant registration was:

```javascript
const mutates = new TSDocTagDefinition({
  tagName: '@mutates',
  syntaxKind: TSDocTagSyntaxKind.BlockTag,
  allowMultiple: true,
});
registered.addTagDefinition(mutates);
registered.setSupportForTag(mutates, true);
```

### Passing catalog

- one registered and supported `@mutates` block:
  zero messages;
- two registered and supported blocks:
  zero messages;
- two registered blocks with `allowMultiple` omitted:
  zero messages.

### Failing catalog

- two unregistered blocks:
  two `The TSDoc tag "@mutates" is not defined in this configuration` messages;
- one defined but explicitly unsupported block:
  one `The TSDoc tag "@mutates" is not supported by this tool` message.

Repository fixture tests separately verified the in-house scanner against named targets,
destructured targets,
missing targets,
missing descriptions,
unknown targets,
duplicates,
and fenced examples.

## Verified workarounds

### Register and support a repeatable custom block

External consumers should construct the definition,
add it to the parser configuration,
and mark it supported before parsing:

```typescript
import {
  TSDocConfiguration,
  TSDocTagDefinition,
  TSDocTagSyntaxKind,
} from '@microsoft/tsdoc';

const configuration = new TSDocConfiguration();
const mutatesTag = new TSDocTagDefinition({
  tagName: '@mutates',
  syntaxKind: TSDocTagSyntaxKind.BlockTag,
  allowMultiple: true,
});
configuration.addTagDefinition(mutatesTag);
configuration.setSupportForTag(mutatesTag, true);
```

Tradeoff:
TSDoc accepts and preserves generic block content but does not validate target names,
descriptions,
or duplicate targets.
Consumers must apply the project grammar themselves or trust the repository's emitted declarations.

### Use the repository scanner for lint diagnostics

The Oxlint plugin recognizes `@mutates` directly and validates its project grammar without adding
`@microsoft/tsdoc` as a runtime dependency.

Tradeoff:
the scanner intentionally implements the repository's narrow TSDoc surface rather than exporting TSDoc's full AST.
External documentation tools still need their own registration.

## What does not work

- Parsing without a custom definition produces one undefined-tag message per block.
- Calling only `addTagDefinition()` while unsupported-tag reporting is enabled produces an unsupported-tag message.
- Setting `allowMultiple: true` does not validate the contents of repeated blocks.
- Omitting `allowMultiple` does not make the 0.16.0 parser reject repeated generic blocks.
- Treating `@mutates` as a modifier tag loses the required target and rationale content because modifiers are presence
  markers rather than content sections.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** No.
    Undefined custom tags require registration by design.
    The `allowMultiple` field is not parser-side cardinality enforcement,
    but this project does not require such enforcement from TSDoc.
2.  **Can upstream fix it?
    ** A future parser could validate custom content or cardinality,
    but no upstream change is required for this integration.
3.  **Are they supporting this use case?
    ** The source test
    `tsdoc/src/__tests__/ParsingBasics.test.ts:27` to `tsdoc/src/__tests__/ParsingBasics.test.ts:59` explicitly registers
    custom block and modifier tags.
    Custom content grammars are not represented by the current tag definition API.
4.  **Would the repository welcome this contribution?
    ** The inspected tag has a code of conduct but no
    `CONTRIBUTING.md`,
    issue template,
    pull-request template,
    or AI-assistance policy.
    This does not override the failed first constraint.
5.  **Will they likely fix it?
    ** [Issue 366][issue-366] already requests custom tag syntax and has no maintainer response.
    The present integration does not add a missing reproduction or workaround to that request.
6.  **Have we prototyped a minimal compatible fix?
    ** No upstream fix is warranted because registration works.
    The verified consumer-side registration and repository scanner are the complete integration.
    The automatic upstream prototype gate does not apply because the first constraint fails.

`.out-of-scope/` contains no TSDoc-specific exemption.
Open and closed issues and pull requests were searched for `custom tag configuration` and `allowMultiple`.
Issue 366 is the only directly relevant custom-grammar request.
There is nothing additive to post,
so no issue or comment draft is retained.

[issue-366]: https://github.com/microsoft/tsdoc/issues/366
