# Node 26.12.0 VM evaluations redeclare bindings when the context object is reused

## Symptom

Node `26.12.0` emitted this diagnostic during a local CI source-control preflight:

```text
SyntaxError: Identifier 'changedSource' has already been declared
```

The first `runInNewContext` call succeeded.
The second evaluated the same `const` declaration with the same context object.
The failure preceded admission-file creation and cloud dispatch.
It is separate from duplicate imports assembled into one Mise task body,
documented in [the inline-import incident](mise-inline-task-import-collision.md).

## Root cause

In Node tag `v26.12.0`,
`lib/vm.js:305` passes the supplied object through `createContext`.
At `lib/vm.js:226`,
`createContext` returns an already contextified object unchanged.
The supplied object can therefore retain the earlier evaluation's lexical bindings.
The function name does not guarantee a fresh context when its object argument is reused.

```js
// nodejs/node lib/vm.js:226, v26.12.0
function createContext(contextObject = { __proto__: ObjectPrototype }, options = kEmptyObject) {
  if (contextObject !== vm_context_no_contextify && isContext(contextObject)) {
    return contextObject;
  }
```

```js
// nodejs/node lib/vm.js:305, v26.12.0
function runInNewContext(code, contextObject, options) {
  if (typeof options === 'string') {
    options = { filename: options };
  }
  contextObject = createContext(contextObject, getContextOptions(options));
  options = { ...options, [kParsingContext]: contextObject };
  return createScript(code, options).runInNewContext(contextObject, options);
}
```

The inspected Git blob is `2c7446af607637bc002d20099a929cc737c7c1be`;
the full source SHA256 is `774a31123f8ca83e7bc1d3966d1e738d9034ec2b3f8e98791fa5d40d058a9cdf`.
The [versioned implementation](https://github.com/nodejs/node/blob/v26.12.0/lib/vm.js)
is the primary source for this boundary.
The shallow source clone resolved that tag to commit `26e72d85a611e222e3f06025de8a3c496923bacd`
and independently returned the same `lib/vm.js` Git object identity.

## Verified workaround

Pass a fresh context object for each evaluation that should have independent lexical bindings.
The affected preflight now uses a fresh shallow copy of its explicit scope for each call.
That copy separates the VM context;
it does not deep-clone values inside the scope or establish a security boundary.

The consumer-side change is:

```diff
// research controller, independent lexical evaluation boundary
-runInNewContext(source, scope)
+runInNewContext(source, { ...scope })
```

## Verification

An actual Node `26.12.0` control evaluated `const selected = 1` in a shared object,
then rejected `const selected = 2` with the same duplicate-binding diagnostic.
`isContext` confirmed that shared object was contextified.
The same declarations both passed when supplied separate fresh objects.
The restored CI preflight then completed unchanged-source,
changed-object,
removed-guard and restored-source measurements before dispatch.

The private controller retained the fetched implementation,
source hash,
exact diagnostic and control outcomes in `node-vm-context-source.json` on 2026-10-18.

This minimal task reproduces the failing pattern and checks the independent-context workaround:

```toml
# disposable scratch mise.toml
[tasks."probe:vm-context"]
shell = "node --input-type=module-typescript -e"
run = '''
const vm = await import('node:vm');
const shared = {};
vm.runInNewContext('const selected = 1', shared);
let diagnostic;
try {
  vm.runInNewContext('const selected = 2', shared);
} catch (error) {
  diagnostic = String(error);
}
if (diagnostic !== "SyntaxError: Identifier 'selected' has already been declared") {
  throw new Error('Expected shared-context lexical collision was not observed');
}
vm.runInNewContext('const selected = 1', {});
vm.runInNewContext('const selected = 2', {});
console.log(JSON.stringify({ diagnostic, sharedIsContext: vm.isContext(shared), freshContextsPassed: true }));
'''
```

The shared-object case fails with the recorded syntax diagnostic.
Both fresh-object cases pass,
and the successful full source-control preflight is a separate real consumer.

## What does not work

Reusing the same scope object while calling `runInNewContext` again does not reset its lexical bindings.
The first preflight failed before cloud admission because it relied on that reading.
Repeated execution is a consumer error here;
the observation does not identify a Node regression.

## Upstream filing decision

The `.out-of-scope/` inventory contains no Node VM exemption.
No upstream change or filing is proposed:

- The upstream-fault gate has no established defect;
  the deciding source intentionally reuses an already contextified object.
- The consumer has a verified independent-context correction;
  no upstream repair is needed for this incident.
- The inspected API explicitly accepts and reuses context objects.
- Contribution policy was not assessed because no upstream defect or contribution is proposed.
- No maintainer refusal or willingness is inferred;
  the consumer correction resolves the observed failure.
- The actual consumer correction and failing/passing controls are recorded;
  no upstream prototype is justified after the fault gate fails.

The bounded tracker queries on 2026-10-18 were
`gh search issues 'runInNewContext already declared' --repo nodejs/node`
and `gh search prs 'runInNewContext context' --repo nodejs/node`.
The first returned no matches;
the second returned performance PR `47739`.
These search results do not establish an upstream lexical-binding defect or a comprehensive absence of prior reports.
There is nothing to file as an upstream defect from this incident.
