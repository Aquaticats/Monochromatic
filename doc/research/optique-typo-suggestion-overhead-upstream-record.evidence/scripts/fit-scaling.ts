// Fits the scaling curves and judges the predictions. Host-side script.
//
// For each (sweep, arm): a log-log slope per round (so the spread across rounds
// gives the slope's uncertainty) and a log-log slope on each level's minimum.
// For the options sweep: linear and quadratic fits on the per-round medians.
//
// Run with: node scripts/fit-scaling.ts

import { readFileSync, writeFileSync, } from 'node:fs';

import { SWEEPS, } from './scaling-grid.ts';

//region Types

/**
 * One timed record from the scaling runner.
 */
type ScalingRecord = {
  readonly kind: 'run' | 'count';
  readonly arm: string;
  readonly sweep: string;
  readonly level: number;
  readonly round: number;
  readonly ms?: readonly number[];
  readonly probes?: number;
  readonly ok: boolean;
  readonly tokens: number;
};

/**
 * Fit result for one curve.
 */
type CurveFit = {
  readonly slopePerRound: readonly number[];
  readonly slopeMean: number;
  readonly slopeStandardDeviation: number;
  readonly slopeMin: number;
  readonly slopeMax: number;
  readonly slopeOnLevelMinimum: number;
};

//endregion Types

//region Statistics

/**
 * Interpolated median.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Median value.
 */
function median(values: readonly number[],): number {
  /** Sorted copy. */
  const sorted = [...values].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Middle index. */
  const middle = Math.floor(sorted.length / 2,);

  return sorted.length % 2 === 0
    ? ((sorted[middle - 1] ?? Number.NaN) + (sorted[middle] ?? Number.NaN)) / 2
    : (sorted[middle] ?? Number.NaN);
}

/**
 * Arithmetic mean.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Mean value.
 */
function mean(values: readonly number[],): number {
  return values.reduce(function sum(total, value,) {
    return total + value;
  }, 0,) / values.length;
}

/**
 * Sample standard deviation.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Standard deviation, or 0 for fewer than two values.
 */
function standardDeviation(values: readonly number[],): number {
  if (values.length < 2) {
    return 0;
  }

  /** Mean of the values. */
  const average = mean(values,);

  return Math.sqrt(
    values.reduce(function sum(total, value,) {
      return total + ((value - average) ** 2);
    }, 0,) / (values.length - 1),
  );
}

/**
 * Least-squares slope of y on x.
 *
 * @param xs - Independent values.
 *
 * @param ys - Dependent values.
 *
 * @returns Slope and intercept.
 */
function linearFit(xs: readonly number[], ys: readonly number[],): { slope: number; intercept: number; } {
  /** Mean of xs. */
  const xMean = mean(xs,);
  /** Mean of ys. */
  const yMean = mean(ys,);
  /** Sum of squared deviations of x. */
  let denominator = 0;
  /** Sum of cross deviations. */
  let numerator = 0;

  for (const [index, x,] of xs.entries()) {
    /** Deviation of x from its mean. */
    const dx = x - xMean;
    /** Deviation of y from its mean. */
    const dy = (ys[index] ?? Number.NaN) - yMean;

    denominator += dx * dx;
    numerator += dx * dy;
  }

  /** Fitted slope. */
  const slope = numerator / denominator;

  return {
    slope,
    intercept: yMean - (slope * xMean),
  };
}

/**
 * Coefficient of determination of a fit.
 *
 * @param ys - Observed values.
 *
 * @param predict - Prediction for each observed value.
 *
 * @returns R squared.
 */
function rSquared(ys: readonly number[], predict: (index: number,) => number,): number {
  /** Mean of observations. */
  const yMean = mean(ys,);
  /** Total sum of squares. */
  const total = ys.reduce(function sum(accumulated, y,) {
    return accumulated + ((y - yMean) ** 2);
  }, 0,);
  /** Residual sum of squares. */
  const residual = ys.reduce(function sum(accumulated, y, index,) {
    return accumulated + ((y - predict(index,)) ** 2);
  }, 0,);

  return 1 - (residual / total);
}

/**
 * Solves a small linear system with Gaussian elimination.
 *
 * @param matrix - Square coefficient matrix.
 *
 * @param rhs - Right-hand side.
 *
 * @returns Solution vector.
 */
function solve(matrix: readonly (readonly number[])[], rhs: readonly number[],): readonly number[] {
  /** Working copy of the matrix. */
  const a = matrix.map(function copyRow(row,) {
    return [...row,];
  },);
  /** Working copy of the right-hand side. */
  const b = [...rhs,];

  for (let column = 0; column < a.length; column += 1) {
    /** Row with the largest pivot in this column. */
    let pivotRow = column;

    for (let row = column + 1; row < a.length; row += 1) {
      if (Math.abs(a[row]?.[column] ?? 0,) > Math.abs(a[pivotRow]?.[column] ?? 0,)) {
        pivotRow = row;
      }
    }

    /** Temporary row swap target. */
    const swap = a[column];

    a[column] = a[pivotRow] as number[];
    a[pivotRow] = swap as number[];
    /** Temporary right-hand-side swap. */
    const swapB = b[column];

    b[column] = b[pivotRow] as number;
    b[pivotRow] = swapB as number;

    /** Pivot value. */
    const pivot = a[column]?.[column] ?? 1;

    for (let row = column + 1; row < a.length; row += 1) {
      /** Factor eliminating this row's column entry. */
      const factor = (a[row]?.[column] ?? 0) / pivot;

      for (let inner = column; inner < a.length; inner += 1) {
        a[row]![inner] = (a[row]?.[inner] ?? 0) - (factor * (a[column]?.[inner] ?? 0));
      }
      b[row] = (b[row] ?? 0) - (factor * (b[column] ?? 0));
    }
  }

  /** Solution vector, filled from the bottom up. */
  const solution = new Array<number>(a.length,).fill(0,);

  for (let row = a.length - 1; row >= 0; row -= 1) {
    /** Accumulated known terms. */
    let accumulated = b[row] ?? 0;

    for (let column = row + 1; column < a.length; column += 1) {
      accumulated -= (a[row]?.[column] ?? 0) * (solution[column] ?? 0);
    }
    solution[row] = accumulated / (a[row]?.[row] ?? 1);
  }

  return solution;
}

//endregion Statistics

//region Entry point

/**
 * Builds every fit, judges the predictions, and writes the report.
 */
function main(): void {
  /** Every scaling record. */
  const records = readFileSync(
    'results-scaling/scaling.jsonl',
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as ScalingRecord;
    },);
  /** Timed records only. */
  const timed = records.filter(function isRun(record,) {
    return record.kind === 'run';
  },);
  /** Counting records only. */
  const counts = records.filter(function isCount(record,) {
    return record.kind === 'count';
  },);

  /** Per-round medians per (sweep, level, arm). */
  const roundMedians = new Map<string, Map<number, number>>();
  /** Every sample per (sweep, level, arm). */
  const samples = new Map<string, number[]>();
  /** Arms seen for each sweep. */
  const armsBySweep = new Map<string, Set<string>>();

  for (const record of timed) {
    /** Key without the round. */
    const key = `${record.sweep}|${record.level}|${record.arm}`;
    /** Round medians for this key. */
    const perRound = roundMedians.get(key,) ?? new Map<number, number>();

    perRound.set(
      record.round,
      median(record.ms ?? [],),
    );
    roundMedians.set(
      key,
      perRound,
    );
    samples.set(
      key,
      [
        ...(samples.get(key,) ?? []),
        ...(record.ms ?? []),
      ],
    );
    /** Arms recorded for this sweep. */
    const arms = armsBySweep.get(record.sweep,) ?? new Set<string>();

    arms.add(record.arm,);
    armsBySweep.set(
      record.sweep,
      arms,
    );
  }

  /** Fit per sweep and arm. */
  const fits: Record<string, Record<string, CurveFit>> = {};

  for (const sweep of SWEEPS) {
    /** Fits for this sweep. */
    const perArm: Record<string, CurveFit> = {};

    for (const arm of armsBySweep.get(sweep.name,) ?? []) {
      /** Rounds present. */
      const rounds = [...new Set(
        timed
          .filter(function matches(record,) {
            return record.sweep === sweep.name && record.arm === arm;
          },)
          .map(function roundOf(record,) {
            return record.round;
          },),
      ),];
      /** Log-log slope for each round. */
      const slopes: number[] = [];

      for (const round of rounds) {
        /** x values: log of the level. */
        const xs: number[] = [];
        /** y values: log of that round's median. */
        const ys: number[] = [];

        for (const level of sweep.levels) {
          /** This round's median at this level. */
          const value = roundMedians
            .get(`${sweep.name}|${level}|${arm}`,)
            ?.get(round,);

          if (value === undefined || value <= 0) {
            continue;
          }
          xs.push(Math.log(level,),);
          ys.push(Math.log(value,),);
        }

        if (xs.length >= 2) {
          slopes.push(linearFit(
            xs,
            ys,
          ).slope,);
        }
      }

      /** x values from the levels. */
      const xs = sweep.levels.map(function logOf(level,) {
        return Math.log(level,);
      },);
      /** y values from each level's smallest sample. */
      const ys = sweep.levels.map(function minOf(level,) {
        /** Samples at this level. */
        const levelSamples = samples.get(`${sweep.name}|${level}|${arm}`,) ?? [];

        return Math.log(
          levelSamples.length === 0 ? Number.NaN : Math.min(...levelSamples,),
        );
      },);

      perArm[arm] = {
        slopePerRound: slopes.map(function rounded(value,) {
          return Number(value.toFixed(4),);
        },),
        slopeMean: Number(mean(slopes,).toFixed(4),),
        slopeStandardDeviation: Number(standardDeviation(slopes,).toFixed(4),),
        slopeMin: Number(Math.min(...slopes,).toFixed(4),),
        slopeMax: Number(Math.max(...slopes,).toFixed(4),),
        slopeOnLevelMinimum: Number(linearFit(
          xs,
          ys,
        ).slope.toFixed(4),),
      };
    }

    fits[sweep.name] = perArm;
  }

  /** Options-sweep shape checks: linear and quadratic fits. */
  const optionsShape: Record<string, {
    linearR2PerRound: readonly number[];
    quadraticR2PerRound: readonly number[];
    quadraticCoefficientPerRound: readonly number[];
    squaredOnlyR2PerRound: readonly number[];
    squaredOnlyCoefficientPerRound: readonly number[];
    squaredOnlyInterceptPerRound: readonly number[];
  }> = {};
  /**
   * Predicted-shape fits for the token and length sweeps: a straight line in the
   * varied factor, so the intercept separates fixed per-parse overhead from the
   * per-unit cost the source predicts.
   */
  const predictedShapes: Record<string, Record<string, {
    slopePerRound: readonly number[];
    interceptPerRound: readonly number[];
    r2PerRound: readonly number[];
  }>> = {};
  /** Fitted shape per sweep and arm for the token and length sweeps. */
  for (const sweep of SWEEPS) {
    if (sweep.name === 'options') {
      continue;
    }

    /** Fits for this sweep. */
    const perArm: Record<string, {
      slopePerRound: readonly number[];
      interceptPerRound: readonly number[];
      r2PerRound: readonly number[];
    }> = {};

    for (const arm of armsBySweep.get(sweep.name,) ?? []) {
      /** Slope per round. */
      const slopes: number[] = [];
      /** Intercept per round. */
      const intercepts: number[] = [];
      /** R squared per round. */
      const r2: number[] = [];
      /** Rounds present. */
      const rounds = [
        ...new Set(
          timed
            .filter(function matches(record,) {
              return record.sweep === sweep.name && record.arm === arm;
            },)
            .map(function roundOf(record,) {
              return record.round;
            },),
        ),
      ];

      for (const round of rounds) {
        /** Level values. */
        const xs: number[] = [];
        /** Median at this round. */
        const ys: number[] = [];

        for (const level of sweep.levels) {
          /** This round's median at this level. */
          const value = roundMedians
            .get(`${sweep.name}|${level}|${arm}`,)
            ?.get(round,);

          if (value === undefined) {
            continue;
          }
          xs.push(level,);
          ys.push(value,);
        }

        /** Straight-line fit in the varied factor. */
        const fit = linearFit(
          xs,
          ys,
        );

        slopes.push(fit.slope,);
        intercepts.push(fit.intercept,);
        r2.push(rSquared(
          ys,
          function predicted(index,) {
            return fit.intercept + (fit.slope * (xs[index] ?? Number.NaN));
          },
        ),);
      }

      perArm[arm] = {
        slopePerRound: slopes.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        interceptPerRound: intercepts.map(function rounded(value,) {
          return Number(value.toFixed(3),);
        },),
        r2PerRound: r2.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
      };
    }

    predictedShapes[sweep.name] = perArm;
  }

  /** Options-sweep definition. */
  const optionsSweep = SWEEPS.find(function isOptions(sweep,) {
    return sweep.name === 'options';
  },);

  if (optionsSweep !== undefined) {
    for (const arm of armsBySweep.get('options',) ?? []) {
      /** Linear R squared per round. */
      const linearR2: number[] = [];
      /** Quadratic R squared per round. */
      const quadraticR2: number[] = [];
      /** Quadratic coefficient per round, per option squared. */
      const quadraticCoefficient: number[] = [];
      /** R squared of the two-parameter model t = a + b * P squared. */
      const squaredOnlyR2: number[] = [];
      /** Fitted b of that model. */
      const squaredOnlyCoefficient: number[] = [];
      /** Fitted a of that model. */
      const squaredOnlyIntercept: number[] = [];
      /** Rounds present. */
      const rounds = [
        ...new Set(
          timed
            .filter(function matches(record,) {
              return record.sweep === 'options' && record.arm === arm;
            },)
            .map(function roundOf(record,) {
              return record.round;
            },),
        ),
      ];

      for (const round of rounds) {
        /** Level values. */
        const xs = optionsSweep.levels;
        /** Median at this round. */
        const ys = xs.map(function valueAt(level,) {
          return roundMedians
            .get(`options|${level}|${arm}`,)
            ?.get(round,) ?? Number.NaN;
        });
        /** Linear fit on the raw values. */
        const linear = linearFit(
          xs,
          ys,
        );

        linearR2.push(rSquared(
          ys,
          function predicted(index,) {
            return linear.intercept + (linear.slope * (xs[index] ?? Number.NaN));
          },
        ),);

        /** Normal equations for the quadratic fit. */
        const moments = [0, 0, 0, 0, 0];
        /** Right-hand side of the normal equations. */
        const rhs = [0, 0, 0];

        for (const [index, x,] of xs.entries()) {
          /** Observed value. */
          const y = ys[index] ?? Number.NaN;

          for (let power = 0; power <= 4; power += 1) {
            moments[power] = (moments[power] ?? 0) + (x ** power);
          }
          for (let power = 0; power <= 2; power += 1) {
            rhs[power] = (rhs[power] ?? 0) + ((x ** power) * y);
          }
        }

        /** Fitted coefficients, constant first. */
        const coefficients = solve(
          [
            [
              moments[0] ?? 0,
              moments[1] ?? 0,
              moments[2] ?? 0,
            ],
            [
              moments[1] ?? 0,
              moments[2] ?? 0,
              moments[3] ?? 0,
            ],
            [
              moments[2] ?? 0,
              moments[3] ?? 0,
              moments[4] ?? 0,
            ],
          ],
          rhs,
        );

        quadraticR2.push(rSquared(
          ys,
          function predicted(index,) {
            /** Level. */
            const x = xs[index] ?? Number.NaN;

            return (coefficients[0] ?? 0)
              + ((coefficients[1] ?? 0) * x)
              + ((coefficients[2] ?? 0) * x * x);
          },
        ),);
        quadraticCoefficient.push((coefficients[2] ?? 0) / 1000,);

        /** Two-parameter fit in P squared. */
        const squaredOnly = linearFit(
          xs.map(function squared(level,) {
            return level * level;
          },),
          ys,
        );

        squaredOnlyR2.push(rSquared(
          ys,
          function predicted(index,) {
            /** Level. */
            const level = xs[index] ?? Number.NaN;

            return squaredOnly.intercept + (squaredOnly.slope * level * level);
          },
        ),);
        squaredOnlyCoefficient.push(squaredOnly.slope,);
        squaredOnlyIntercept.push(squaredOnly.intercept,);
      }

      optionsShape[arm] = {
        linearR2PerRound: linearR2.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        quadraticR2PerRound: quadraticR2.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        quadraticCoefficientPerRound: quadraticCoefficient.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        squaredOnlyR2PerRound: squaredOnlyR2.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        squaredOnlyCoefficientPerRound: squaredOnlyCoefficient.map(function rounded(value,) {
          return Number(value.toFixed(6),);
        },),
        squaredOnlyInterceptPerRound: squaredOnlyIntercept.map(function rounded(value,) {
          return Number(value.toFixed(3),);
        },),
      };
    }
  }

  /** Per-level medians, for the report table. */
  const levels = SWEEPS.map(function asSweep(sweep,) {
    return {
      sweep: sweep.name,
      fixedOptionCount: sweep.fixedOptionCount,
      fixedTokenLength: sweep.fixedTokenLength,
      fixedTokenCount: sweep.fixedTokenCount,
      points: sweep.levels.map(function asLevel(level,) {
        return {
          level,
          arms: Object.fromEntries(
            [...(armsBySweep.get(sweep.name,) ?? []),].map(function asArm(arm,) {
              /** Key for this point. */
              const key = `${sweep.name}|${level}|${arm}`;
              /** Every sample at this point. */
              const pointSamples = samples.get(key,) ?? [];

              return [
                arm,
                {
                  medianMs: Number(median(pointSamples,).toFixed(4),),
                  minMs: Number(Math.min(...pointSamples,).toFixed(4),),
                  roundMediansMs: [...(roundMedians.get(key,)?.values() ?? [])].map(function rounded(value,) {
                    return Number(value.toFixed(4),);
                  },),
                },
              ];
            },),
          ),
        };
      },),
    };
  },);

  /** Probe counts per (sweep, level, arm). */
  const probeTable = counts.map(function asRow(record,) {
    /** Point definition. */
    const shape = {
      optionCount: 0,
      tokenLength: 0,
      tokenCount: 0,
      ...(record as unknown as { shape?: Record<string, number>; }).shape,
    };

    return {
      sweep: record.sweep,
      level: record.level,
      arm: record.arm,
      probes: record.probes ?? 0,
      tokens: record.tokens,
      optionCount: shape.optionCount,
      tokenCount: shape.tokenCount,
      expectedTokensTimesOptions: shape.optionCount * shape.tokenCount,
      ok: record.ok,
    };
  },);

  /** Per-token cost on 1.1.1, used to cross-check the per-cell cost of Part 2. */
  const tokensKey = 'tokens|1000|1.1.1';
  /** Median at 1000 tokens, 16 options, 12-character tokens. */
  const tokensMedian = median(samples.get(tokensKey,) ?? [],);
  /** Edit-distance cells one such parse performs. */
  const percentCells = 16 * 16 * 12 * 12 * 1000;
  /** Implied nanoseconds per edit-distance cell. */
  const nanosecondsPerCell = (tokensMedian * 1e6) / percentCells;

  /** The report. */
  const report = {
    generatedAt: new Date().toISOString(),
    fits,
    optionsShape,
    predictedShapes,
    levels,
    probes: probeTable,
    crossCheck: {
      arm: '1.1.1',
      tokensMedianMs: Number(tokensMedian.toFixed(4),),
      editDistanceCellsPerParse: percentCells,
      nanosecondsPerCell: Number(nanosecondsPerCell.toFixed(3),),
    },
  };

  writeFileSync(
    'results-scaling/fit.json',
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
  );

  for (const [sweep, perArm,] of Object.entries(fits,)) {
    for (const [arm, fit,] of Object.entries(perArm,)) {
      console.log(
        `slope ${sweep.padEnd(7,)} ${arm.padEnd(18,)} mean=${
          fit.slopeMean
        } sd=${fit.slopeStandardDeviation} [${fit.slopeMin}, ${fit.slopeMax}] min-fit=${
          fit.slopeOnLevelMinimum
        }`,
      );
    }
  }

  for (const [sweep, perArm,] of Object.entries(predictedShapes,)) {
    for (const [arm, shape,] of Object.entries(perArm,)) {
      console.log(
        `straight ${sweep.padEnd(7,)} ${arm.padEnd(18,)} slope=${JSON.stringify(shape.slopePerRound)} intercept=${
          JSON.stringify(shape.interceptPerRound)
        } R2=${JSON.stringify(shape.r2PerRound)}`,
      );
    }
  }

  for (const [arm, shape,] of Object.entries(optionsShape,)) {
    console.log(
      `options shape ${arm.padEnd(18,)} R2 linear=${JSON.stringify(shape.linearR2PerRound)} R2 a+bP2=${
        JSON.stringify(shape.squaredOnlyR2PerRound)
      } b=${JSON.stringify(shape.squaredOnlyCoefficientPerRound)} a=${
        JSON.stringify(shape.squaredOnlyInterceptPerRound)
      }`,
    );
  }

  for (const row of probeTable) {
    console.log(
      `probes ${row.sweep.padEnd(7,)} level=${String(row.level,).padStart(4,)} ${
        row.arm.padEnd(18,)
      } probes=${row.probes} expected=${row.expectedTokensTimesOptions}`,
    );
  }

  console.log(
    `cross-check 1.1.1: ${tokensMedian.toFixed(4)} ms per parse, ${percentCells} cells => ${
      nanosecondsPerCell.toFixed(3)
    } ns per cell`,
  );
}

main();

//endregion Entry point
