# Windows code signing since 2023-06-01: OV/EV keys are hardware-only, and the cert subject is public

## Symptom

Two surprises hit anyone trying to Authenticode-sign a Windows binary,
 especially from Linux or CI:

- You buy a standard organization (OV) code-signing certificate expecting a `.pfx` file to feed to `signtool` or
  `osslsigncode`,
   but the CA instead ships a physical USB token or a cloud-signing account,
   and the private key is
  non-exportable.
   The naive cross-platform command no longer works:
  ```bash
  # Impossible for a real OV cert issued after 2023-06-01: no exportable .pfx.
  osslsigncode sign -pkcs12 cert.pfx -pass "$PFX_PASS" -in app.exe -out app-signed.exe
  ```
- Whatever you use,
   the signature's subject is shown to end users (the Windows "publisher" prompt and the file's
  **Digital Signatures** tab).
   An individual certificate therefore publishes your legal name.

## Root cause

The CA/Browser Forum's Baseline Requirements for the Issuance and Management of Code Signing Certificates,
 via
Ballot CSC-17,
 made hardware key protection mandatory effective 2023-06-01.
 Subscriber private keys for both OV
(standard) and EV code-signing certificates must be generated and stored in a crypto module meeting FIPS 140-2
Level 2 / Common Criteria EAL 4+ (or equivalent),
 and the keys must be non-exportable.
 Source:
 CA/Browser Forum
code-signing requirements,
 <https://cabforum.org/working-groups/code-signing/requirements/> (the versioned PDF,
 for
example v3.
x,
 states the key-protection requirement and its effective date).
 Reputable CAs document the same change,
for example GlobalSign's advisory,
<https://support.globalsign.com/code-signing/advisory/new-requirements-related-private-key-protection-codesigning-certificates>,
and SSL.
com note that since 2023-06-01 OV/IV certs are issued only on FIPS USB tokens or through a cloud signing
service.

Consequence for cross-platform signing:
 a physical USB token is awkward to use from a headless Linux host (it needs
vendor PKCS#11 middleware and a connected reader),
 so a cloud HSM exposed over PKCS#11 is the practical
Linux-drivable path.

The name-exposure point is not a 2023 change but is easy to forget:
 Authenticode records the certificate subject,
and Windows surfaces it.
 An organization (OV) certificate's subject is the organization's legal name,
 which is why
this project signs under a registered organization rather than an individual (see
[../decision/desktop-app-code-signing.md](../decision/desktop-app-code-signing.md)).

## Verification

Tool under test:
 `osslsigncode` 2.14 (the latest upstream release on 2026-10-29,
 compiled from source by the build stage of this repo's
`package/music-player/desktop-app/Containerfile.sign` on CachyOS),
 built against OpenSSL 3.6.5.
 The first verification, on 2026-10-14, used osslsigncode 2.9 from the Fedora 41 package with OpenSSL 3.2;
 neither the Arch nor the CachyOS repositories package the tool.
 Its `sign` usage confirms both
the legacy `.pfx` path and the PKCS#11 path (`-pkcs11module` and `-pkcs11cert` still exist;
 the 2.9 option `-pkcs11engine` is now `-engine`, and `-provider` was added):

```text
[ sign ] -pkcs12 <file> | ( [ -certs <file|URI> | -spc <file> ]
            ( -key <file|URI> [ -pkcs11module <module> ] [ -pkcs11cert <pkcs11 cert URI> ] )
            [ -provider <provider> | ( -engine <engine> [ -login ] [ -engineCtrl <command[:parameter]> ] ) ] )
          ...
          [ -ts <timestampurl> [ -ts ... ] [ -p <proxy> ] [ -noverifypeer ] ]
          [ -in ] <file> [-out ] <file>
```

The `sign:windows` task was run on 2026-10-29 with the throwaway self-signed `.pfx` path
 on a 19-section PE executable built with the Windows GNU cross compiler of this repository;
 it ended with `Signature verification: ok`, `Number of verified signatures: 1` and `Succeeded`.
 The PKCS#11 path was not exercised, because it needs a real certificate and the CA's module.

What still works:

- A self-signed `.pfx` for local testing (generated with `openssl req`/`openssl pkcs12`),
   signed via `-pkcs12`.
  Proves the signing mechanics;
   not publicly trusted.
- A real cloud key over PKCS#11,
   signed via `-pkcs11module`/`-pkcs11cert`/`-key` with an RFC3161 timestamp via
  `-ts`.

What fails:

- Obtaining a plain exportable `.pfx` for a newly issued OV/EV certificate.
   The CA will not issue one;
   the key is
  generated in hardware or the cloud HSM.

## Verified workarounds

- Cloud-key signing over PKCS#11 from Linux.
   Use a CA whose product is a cloud signing service exposing PKCS#11 (for
  example Certum SimplySign,
   or SSL.
  com eSigner),
   and drive it with `osslsigncode -pkcs11module ...`.
   Tradeoff:
   you
  depend on the CA's cloud availability and middleware,
   and the exact module path and object IDs are CA-specific
  (the Windows credential runbook records how to discover them).
   This is the only genuinely cross-platform,
  Linux-drivable path for a real certificate.
- Organization-validated certificate for name privacy.
   Tradeoff:
   organization OV costs more than an individual
  certificate and requires business-registration validation,
   but the published subject is the org's name rather than
  a person's.
- Self-signed `.pfx` for the test path only.
   Tradeoff:
   SmartScreen does not trust it,
   so it is useful only to
  exercise the pipeline,
   never for distribution.

## What does not work

- A plain exportable `.pfx` from a real OV/EV certificate:
   not issued since 2023-06-01.
- Azure Trusted Signing from Linux:
   its signing client is Windows-only (a `signtool` dlib),
   so it cannot be driven
  from a plain Linux host,
   and onboarding is restricted (organizations need a multi-year verifiable history;
  individuals are limited to the US and Canada).
   Verified against Microsoft's Trusted Signing pricing and FAQ pages.
- A physical USB token on headless CI:
   technically possible via PKCS#11 and vendor middleware,
   but it needs the
  token physically present,
   so it is impractical for an automated Linux signing host.

## Upstream filing decision

Nothing to file.
 This walks the 6-constraint check only to record why:

1. Is it upstream's fault?
    No. The hardware-key mandate is a CA/Browser Forum industry policy,
    not a defect in any
   tool we use (`osslsigncode` already supports the PKCS#11 path the policy requires,
    as shown above).
    The
   name-exposure point is intrinsic to how Authenticode works,
    also not a tool defect.

Because constraint 1 fails,
 the remaining constraints do not apply and there is no tool tracker to file against.
`osslsigncode` behaves correctly;
 the constraint is the policy itself,
 which is not something a GitHub issue can
change.
 No `.out-of-scope/` exemption is needed because there is no upstream filing to make.
