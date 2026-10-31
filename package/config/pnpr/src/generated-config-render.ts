import {
  PNPR_CONFIG_HEADER_LINE,
  PNPR_EXCLUDED_PACKAGES,
  PNPR_GITHUB_IDENTITY,
  PNPR_GITHUB_WORKFLOW_REF,
  PNPR_GITHUB_WORKLOAD_SUBJECT,
  PNPR_HOSTED_REGISTRY,
  PNPR_PACKAGE_SCOPE,
  PNPR_PUBLIC_ORIGIN,
  PNPR_PUBLISH_USERNAME,
} from './generated-config-settings.ts';

//region Rendering the generated pnpr server config

/**
 Renders the pnpr server config text for a selection of published packages.

 @param publishedNames - Sorted names that publish to pnpr and populate the OIDC trust list.

 @param excludedPackages - Reviewed exclusions keyed by package name, rendered as header comments;
 defaults to the checked-in list.

 @returns Complete `config.yaml` text.

 @throws Error when an exclusion reason spans several lines, which would break out of its comment.

 @example
 ```ts
 renderPnprConfig({ publishedNames: ['@monochromatic-dev/or-throw-core'] });
 ```
 */
export function renderPnprConfig(
  {
    publishedNames,
    excludedPackages = PNPR_EXCLUDED_PACKAGES,
  }: {
    readonly publishedNames: readonly string[];
    readonly excludedPackages?: Readonly<Record<string, string>>;
  },
): string {
  /**
   YAML comment lines listing reviewed exclusions with their reasons.
   */
  const exclusionComments = Object.entries(excludedPackages,)
    .map(function toComment([
      name,
      reason,
    ],) {
      if (reason.includes('\n',) || reason.includes('\r',))
        throw new Error(`pnpr config: exclusion reason for ${name} must be one line`,);
      return `#   ${name}: ${reason}`;
    },);
  return `${PNPR_CONFIG_HEADER_LINE}
# Decision: doc/decision/private-npm-registry.md
# Published package count: ${publishedNames.length}
# Reviewed exclusions:${exclusionComments.length === 0 ? ' none' : `\n${exclusionComments.join('\n',)}`}

storage: /pnpr/storage
secret: \${PNPR_SECRET}

auth:
  htpasswd:
    file: /pnpr/storage/htpasswd
    max_users: -1
  oidc:
    - name: github
      issuer: https://token.actions.githubusercontent.com
      audience: ${PNPR_PUBLIC_ORIGIN}
      workloads:
        - identity:
            subject: '${PNPR_GITHUB_WORKLOAD_SUBJECT}'
            username: ${PNPR_PUBLISH_USERNAME}
            claims:
              repository_id: '${PNPR_GITHUB_IDENTITY.repositoryId}'
              repository_owner_id: '${PNPR_GITHUB_IDENTITY.repositoryOwnerId}'
              workflow_ref: '${PNPR_GITHUB_WORKFLOW_REF}'
          registry: ${PNPR_HOSTED_REGISTRY}
          packages:
${
    publishedNames.map(function toYamlItem(name,) {
      return `            - '${name}'`;
    },)
      .join('\n',)
  }

ui:
  enabled: false

registries:
  ${PNPR_HOSTED_REGISTRY}:
    type: hosted
    access: $all
    packages:
      '${PNPR_PACKAGE_SCOPE}*':
        access: $all
        publish: ${PNPR_PUBLISH_USERNAME}
        unpublish: ${PNPR_PUBLISH_USERNAME}

defaultRegistry: ${PNPR_HOSTED_REGISTRY}

log:
  type: stdout
  format: json
  level: info
`;
}

//endregion Rendering the generated pnpr server config
