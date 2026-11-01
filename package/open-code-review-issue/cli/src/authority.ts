/**
 Non-interactive apply authority enforcement.
 
 @module
 */

import type { InputPosition, } from './model.ts';
import type {
  ApplyAuthority,
  ApplySelection,
  PublicationPlan,
} from './plan-model.ts';

/**
 Reports missing explicit authority without repeating security content.
 
 @example
 ```ts
 throw new SecurityAuthorityError('security authority is required');
 ```
 */
export class SecurityAuthorityError extends Error {
  /**
   Creates security authority failure.
   
   @param message - Safe diagnostic containing counts and positions only.
   
   @example
   ```ts
   const error = new SecurityAuthorityError('security authority is required');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'SecurityAuthorityError';
  }
}

/**
 Collects security positions without retaining content projection.
 
 @param plan - Complete internal publication plan.
 
 @returns Security finding positions in input order.
 
 @example
 ```ts
 securityPositions(plan);
 ```
 */
function securityPositions(plan: PublicationPlan,): readonly InputPosition[] {
  return plan.issues
    .filter(function isSecurity(issue,): boolean {
      return issue.security;
    },)
    .map(function toPosition(issue,): InputPosition {
      return issue.position;
    },);
}

/**
 Formats safe position list for authority diagnostics.
 
 @param positions - Security input positions.
 
 @returns Comma-separated position labels.
 
 @example
 ```ts
 formatPositions([{ kind: 'record', value: 2 }]); // 'record 2'
 ```
 */
function formatPositions(positions: readonly InputPosition[],): string {
  return positions.map(function formatPosition(position,): string {
    return `${position.kind} ${String(position.value,)}`;
  },)
    .join(', ',);
}

/**
 Selects exactly authorized issues before any creation starts.
 
 @param plan - Complete internal publication plan.
 
 @param authority - Explicit non-interactive authority level.
 
 @returns Authorized issues and withheld security positions.
 
 @throws {@link SecurityAuthorityError} when bare apply includes security.
 
 @example
 ```ts
 selectApplyPlan({ plan, authority: 'non-security-only' });
 ```
 */
export function selectApplyPlan({
  plan,
  authority,
}: {
  readonly plan: PublicationPlan;
  readonly authority: ApplyAuthority;
},): ApplySelection {
  /**
   Security positions used by both withholding and bare-apply rejection.
   */
  const positions = securityPositions(plan,);
  if (authority === 'all') {
    return {
      issues: plan.issues,
      withheldPositions: [],
    };
  }
  /**
   Ordinary issues selected by default and non-security-only authority.
   */
  const ordinaryIssues = plan.issues
    .filter(function isOrdinary(issue,): boolean {
      return !issue.security;
    },);
  if (authority === 'non-security-only') {
    return {
      issues: ordinaryIssues,
      withheldPositions: positions,
    };
  }
  if (positions.length > 0) {
    throw new SecurityAuthorityError(
      `bare --apply found ${String(positions.length,)} security finding(s) at ${formatPositions(positions,)}; use --non-security-only or --all`,
    );
  }
  return {
    issues: ordinaryIssues,
    withheldPositions: [],
  };
}
