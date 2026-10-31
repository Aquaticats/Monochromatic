//! What:
//! Whether the process runs in a continuous-integration environment,
//! by the same rules as the `ci-info` package.
//! Why:
//! pnpm decides its default for `--frozen-lockfile` this way
//! ("pnpm detects a CI environment with the same rules as the ci-info package.
//! Set `CI=false` to turn detection off", <https://pnpm.io/cli/install>),
//! and `mpm-shadow install` follows pnpm:
//! frozen in CI,
//! resolving new plan entries elsewhere.
//! The rules were read from `index.js` and `vendors.json` of <https://github.com/watson/ci-info> on 2026-10-31.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ci-detect.ts
//! export function isCi(env: Record<string, string | undefined>): boolean
//! ```

/// Imports `EnvMap` from `super::scope`.
use super::scope::EnvMap;

/// The variables whose presence (non-empty) alone says CI,
/// from the `isCI` expression of `ci-info`.
const GENERIC_VARIABLES: [&str; 9] = [
    "BUILD_ID",
    "BUILD_NUMBER",
    "CI",
    "CI_APP_ID",
    "CI_BUILD_ID",
    "CI_BUILD_NUMBER",
    "CI_NAME",
    "CONTINUOUS_INTEGRATION",
    "RUN_ID",
];

/// The vendors that are set by one variable being present (non-empty),
/// from `vendors.json`.
const VENDOR_VARIABLES: [&str; 48] = [
    "AGOLA_GIT_REF",
    "ALPIC_HOST",
    "AC_APPCIRCLE",
    "APPVEYOR",
    "CODEBUILD_BUILD_ARN",
    "TF_BUILD",
    "bamboo_planKey",
    "BITBUCKET_COMMIT",
    "BITRISE_IO",
    "BUDDY_WORKSPACE_ID",
    "BUILDKITE",
    "CIRCLECI",
    "CIRRUS_CI",
    "CF_PAGES",
    "WORKERS_CI",
    "CF_BUILD_ID",
    "CM_BUILD_ID",
    "DRONE",
    "DSARI",
    "EARTHLY_CI",
    "EAS_BUILD",
    "GERRIT_PROJECT",
    "GITEA_ACTIONS",
    "GITHUB_ACTIONS",
    "GITLAB_CI",
    "GO_PIPELINE_LABEL",
    "BUILDER_OUTPUT",
    "HARNESS_BUILD_ID",
    "HUDSON_URL",
    "LAYERCI",
    "MAGNUM",
    "NETLIFY",
    "NEVERCODE",
    "PROW_JOB_ID",
    "RELEASE_BUILD_ID",
    "RENDER",
    "RWX",
    "SAILCI",
    "SCREWDRIVER",
    "SEMAPHORE",
    "STRIDER",
    "TEAMCITY_VERSION",
    "TRAVIS",
    "VELA",
    "APPCENTER_BUILD_ID",
    "CI_XCODE_PROJECT",
    "XCS",
    "VERCEL",
];

/// Whether the variable is present and not empty (JavaScript truthiness of a string).
fn present(env: &EnvMap, name: &str) -> bool {
    return env.get(name).is_some();
}

/// Whether the variable is set to exactly this text.
fn equals(env: &EnvMap, name: &str, expected: &str) -> bool {
    return env.get(name) == Some(expected);
}

/// What:
/// Whether `env` says the process runs in CI:
/// `CI=false` says it does not,
/// whatever else is set;
/// otherwise a generic CI variable,
/// a vendor's marker,
/// or one of the vendors with a combined rule decides.
/// Why:
/// The same answer `ci-info`'s `isCI` gives,
/// so a machine where pnpm freezes its lockfile is one where `mpm-shadow install` freezes the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isCi(env: Record<string, string | undefined>): boolean
/// ```
pub fn is_ci(env: &EnvMap) -> bool {
    if equals(env, "CI", "false") {
        return false;
    }
    for name in GENERIC_VARIABLES {
        if present(env, name) {
            return true;
        }
    }
    for name in VENDOR_VARIABLES {
        if present(env, name) {
            return true;
        }
    }
    // Vendors whose rule is a variable with a given value, or two variables together.
    if equals(env, "CI_NAME", "codeship") || equals(env, "CI_NAME", "sourcehut") || equals(env, "CI", "woodpecker") {
        return true;
    }
    if present(env, "JENKINS_URL") && present(env, "BUILD_ID") {
        return true;
    }
    if present(env, "TASK_ID") && present(env, "RUN_ID") {
        return true;
    }
    if present(env, "NOW_BUILDER") {
        return true;
    }
    // Heroku: the `NODE` variable holds the path of Heroku's node binary.
    if let Some(node) = env.get("NODE")
        && node.contains("/app/.heroku/node/bin/node")
    {
        return true;
    }
    return false;
}
