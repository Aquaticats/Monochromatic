import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Mixin reference errors

/**
 Error thrown when an `\@apply` references a mixin absent from the registry.
 
 @example
 ```ts
 try {
   expandApplyRules({ root, mixins: new Map(), },);
 } catch (error) {
   if (error instanceof UnknownCssMixinError) console.error(error.mixinName);
 }
 ```
 */
export class UnknownCssMixinError extends Error {
  /**
   Mixin name the failing reference asked for.
   */
  readonly mixinName: string;

  /**
   @param message - Human-readable failure description.
   
   @param mixinName - Unresolved mixin reference.
   */
  constructor({
    message,
    mixinName,
  }: {
    readonly message: string;
    readonly mixinName: string;
  },) {
    super(message,);
    this.name = 'UnknownCssMixinError';
    this.mixinName = mixinName;
  }
}

/**
 Error thrown when mixin definitions reference each other in a cycle.
 
 Carries the exact reference trail, so the failing chain is visible instead
 of a pass-count heuristic.
 
 @example
 ```ts
 try {
   expandMixinRegistry({ mixins, },);
 } catch (error) {
   if (error instanceof CircularCssMixinError) console.error(error.trail);
 }
 ```
 */
export class CircularCssMixinError extends Error {
  /**
   Reference chain ending at the repeated mixin.
   */
  readonly trail: readonly string[];

  /**
   @param trail - Reference chain ending at the repeated mixin.
   */
  constructor({
    trail,
  }: {
    readonly trail: readonly string[];
  },) {
    super(`Circular @apply reference between mixins: ${trail.join(' -> ',)}`,);
    this.name = 'CircularCssMixinError';
    this.trail = trail;
  }
}

//endregion Mixin reference errors

//region Package manifest errors

/**
 Error thrown when a `package.json` manifest exists but cannot be read or
 parsed. Carries the manifest path and the underlying failure, so a
 resolution failure names the corrupt file instead of claiming the package
 exports nothing.
 
 @example
 ```ts
 try {
   readPackageJson('/project/node_modules/\@scope/broken');
 } catch (error) {
   if (error instanceof CssPackageManifestError) console.error(error.manifestPath);
 }
 ```
 */
export class CssPackageManifestError extends Error {
  /**
   Absolute path of the unreadable manifest.
   */
  readonly manifestPath: string;

  /**
   @param manifestPath - Absolute path of the unreadable manifest.
   
   @param cause - Read or parse failure the manifest threw.
   */
  constructor({
    manifestPath,
    cause,
  }: {
    readonly manifestPath: string;
    readonly cause: unknown;
  },) {
    super(
      `Cannot read package manifest '${manifestPath}': ${caughtValueText(cause,)}`,
      { cause, },
    );
    this.name = 'CssPackageManifestError';
    this.manifestPath = manifestPath;
  }
}

//endregion Package manifest errors
