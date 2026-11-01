# An extensionless Node forwarder inherits the repository module type

## Symptom

A research executable used a Node shebang and CommonJS `require()` calls,
but had no filename extension.
Under the frozen repository's `package.json` with `"type": "module"`,
Node `26.12.0` rejected the actual compositor version probe:

```text
ReferenceError: require is not defined in ES module scope
```

Depot CI run `vg66k9ck9k`,
job `t3zwj3whn8`,
attempt `gzxpn8416k`,
at research harness `9d39515de6cdf01261746637626ab263c1d6a287`
preserves the failure.
The producer archive and both executable hashes had already been verified,
but no desktop GUI body ran in that consumer.

## Cause and controls

Node's [package-format documentation][node-packages] defines the nearest `package.json` module scope
and the explicit `.cjs` CommonJS extension.
The initial local fixture omitted the repository's `package.json` scope,
so its successful extensionless execution did not represent the cloud invocation.

The corrected local fixture includes `"type": "module"`.
Its extensionless negative control reproduces the actual `require` error.
Its explicit `.cjs` positive control forwards the real version output unchanged
and adds `--headless` for an ordinary compositor launch.
The filename is the format boundary;
the shebang alone did not select CommonJS in the observed package scope.

Harness `8070a3c83ec41a63ae8614ffe93dfcdbcd3ae435` uses `compositor-headless.cjs`.
Its real cloud version control passed,
and the unchanged desktop E2E consumer reached 69 successful UI checks
before a separate source wait failure.
See [the Linux prerequisite and consumer evidence](music-player-linux-ci-prerequisites.md).

[node-packages]: https://nodejs.org/docs/v26.12.0/api/packages.html#packagejson-and-file-extensions
