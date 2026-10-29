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
