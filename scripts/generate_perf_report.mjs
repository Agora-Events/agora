#!/usr/bin/env node
// ==============================================================================
// Agora Performance Regression Report Generator (Issue #1178)
// ==============================================================================
// Reads criterion's per-benchmark `estimates.json` files and a k6
// `--summary-export` JSON file, and produces a single Markdown regression
// report with p50/p95/p99 latency and throughput (RPS) for CI runs — the
// "structured performance regression report" the issue asks for.
//
// Zero npm dependencies (fs/path/process only) so it never needs an install
// step in CI — it just needs a `node` binary, already required by the rest
// of this monorepo's frontend tooling.
//
// Usage:
//   node scripts/generate_perf_report.mjs \
//     --k6-summary=scripts/.stress-results/summary.json \
//     --criterion-dir=server/target/criterion \
//     --baseline=scripts/perf_baseline.json \
//     [--fail-on-regression] [--update-baseline] [--threshold=20]
//
// Any input that's missing is skipped gracefully — the report still renders
// with whatever data is available (e.g. a run with only criterion results
// and no k6 summary still produces a useful report).
// ==============================================================================

import { readFileSync, writeFileSync, existsSync, appendFileSync, readdirSync } from 'node:fs';
import { resolve, relative, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..');

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

function parseArgs(argv) {
  const args = {
    k6Summary: 'scripts/.stress-results/summary.json',
    criterionDir: 'server/target/criterion',
    baseline: 'scripts/perf_baseline.json',
    failOnRegression: false,
    updateBaseline: false,
    thresholdPercent: 20,
  };
  for (const raw of argv) {
    const [flag, value] = raw.split('=');
    switch (flag) {
      case '--k6-summary':
        args.k6Summary = value;
        break;
      case '--criterion-dir':
        args.criterionDir = value;
        break;
      case '--baseline':
        args.baseline = value;
        break;
      case '--fail-on-regression':
        args.failOnRegression = true;
        break;
      case '--update-baseline':
        args.updateBaseline = true;
        break;
      case '--threshold':
        args.thresholdPercent = Number(value);
        break;
      default:
        break;
    }
  }
  return args;
}

// ---------------------------------------------------------------------------
// criterion: walk target/criterion/**/new/estimates.json
// ---------------------------------------------------------------------------

function findEstimatesFiles(dir) {
  const results = [];
  if (!existsSync(dir)) return results;

  const walk = (current) => {
    let entries;
    try {
      entries = readdirSync(current, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const full = join(current, entry.name);
      if (entry.isDirectory()) {
        walk(full);
      } else if (entry.isFile() && entry.name === 'estimates.json' && current.endsWith('new')) {
        results.push(full);
      }
    }
  };
  walk(dir);
  return results;
}

function nsToMs(ns) {
  return ns / 1_000_000;
}

function loadCriterionResults(criterionDir) {
  const absDir = resolve(REPO_ROOT, criterionDir);
  const files = findEstimatesFiles(absDir);
  const results = [];

  for (const file of files) {
    // .../target/criterion/<benchmark path...>/new/estimates.json
    const rel = relative(absDir, file);
    const parts = rel.split(/[\\/]/).filter((p) => p !== 'new' && p !== 'estimates.json');
    const name = parts.join(' / ') || rel;

    try {
      const data = JSON.parse(readFileSync(file, 'utf8'));
      results.push({
        name,
        meanMs: nsToMs(data.mean.point_estimate),
        stdDevMs: nsToMs(data.std_dev.point_estimate),
        medianMs: nsToMs(data.median?.point_estimate ?? data.mean.point_estimate),
      });
    } catch (e) {
      console.error(`warning: failed to parse ${file}: ${e.message}`);
    }
  }

  results.sort((a, b) => a.name.localeCompare(b.name));
  return results;
}

// ---------------------------------------------------------------------------
// k6: --summary-export JSON
// ---------------------------------------------------------------------------

function loadK6Summary(k6SummaryPath) {
  const absPath = resolve(REPO_ROOT, k6SummaryPath);
  if (!existsSync(absPath)) return null;

  let raw;
  try {
    raw = JSON.parse(readFileSync(absPath, 'utf8'));
  } catch (e) {
    console.error(`warning: failed to parse k6 summary at ${absPath}: ${e.message}`);
    return null;
  }

  const metrics = raw.metrics || {};
  const pick = (metricName, field, fallback = null) => {
    const m = metrics[metricName];
    if (!m) return fallback;
    // k6 nests percentile/rate values under `values` in newer summary
    // formats and directly on the metric object in older ones.
    const values = m.values || m;
    return values[field] ?? fallback;
  };

  const httpReqs = metrics.http_reqs;
  const testDurationSeconds = raw.state?.testRunDurationMs
    ? raw.state.testRunDurationMs / 1000
    : null;
  const totalRequests = httpReqs ? (httpReqs.values || httpReqs).count : null;
  const rps =
    totalRequests && testDurationSeconds ? totalRequests / testDurationSeconds : pick('http_reqs', 'rate');

  return {
    httpReqDuration: {
      p50: pick('http_req_duration', 'med'),
      p95: pick('http_req_duration', 'p(95)'),
      p99: pick('http_req_duration', 'p(99)'),
      avg: pick('http_req_duration', 'avg'),
    },
    httpReqFailedRate: pick('http_req_failed', 'rate'),
    rps,
    totalRequests,
    perPhase: {
      browse: extractTrend(metrics, 'agora_browse_duration'),
      queue: extractTrend(metrics, 'agora_queue_duration'),
      checkout: extractTrend(metrics, 'agora_checkout_duration'),
      scan: extractTrend(metrics, 'agora_scan_duration'),
      powSolve: extractTrend(metrics, 'agora_pow_solve_duration'),
    },
    funnelErrorRate: pick('agora_funnel_error_rate', 'rate'),
  };
}

function extractTrend(metrics, name) {
  const m = metrics[name];
  if (!m) return null;
  const values = m.values || m;
  return { p50: values.med, p95: values['p(95)'], p99: values['p(99)'], avg: values.avg };
}

// ---------------------------------------------------------------------------
// Baseline comparison
// ---------------------------------------------------------------------------

function loadBaseline(baselinePath) {
  const absPath = resolve(REPO_ROOT, baselinePath);
  if (!existsSync(absPath)) return null;
  try {
    return JSON.parse(readFileSync(absPath, 'utf8'));
  } catch (e) {
    console.error(`warning: failed to parse baseline at ${absPath}: ${e.message}`);
    return null;
  }
}

function pctChange(current, baseline) {
  if (baseline === null || baseline === undefined || baseline === 0) return null;
  return ((current - baseline) / baseline) * 100;
}

function compareCriterion(current, baseline, thresholdPercent) {
  const baselineByName = new Map((baseline?.criterion || []).map((b) => [b.name, b]));
  const regressions = [];
  const rows = current.map((bench) => {
    const base = baselineByName.get(bench.name);
    const change = base ? pctChange(bench.meanMs, base.meanMs) : null;
    const regressed = change !== null && change > thresholdPercent;
    if (regressed) regressions.push({ name: bench.name, change, kind: 'criterion' });
    return { ...bench, change, regressed };
  });
  return { rows, regressions };
}

function compareK6(current, baseline, thresholdPercent) {
  if (!current || !baseline?.k6) return { regressions: [] };
  const regressions = [];
  for (const key of ['p50', 'p95', 'p99']) {
    const cur = current.httpReqDuration[key];
    const base = baseline.k6.httpReqDuration?.[key];
    const change = pctChange(cur, base);
    if (change !== null && change > thresholdPercent) {
      regressions.push({ name: `k6 http_req_duration ${key}`, change, kind: 'k6' });
    }
  }
  return { regressions };
}

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

function fmtMs(v) {
  if (v === null || v === undefined || Number.isNaN(v)) return '—';
  return `${v.toFixed(2)}ms`;
}

function fmtPct(v) {
  if (v === null || v === undefined) return '—';
  return `${(v * 100).toFixed(2)}%`;
}

function fmtChange(v) {
  if (v === null || v === undefined) return '—';
  const sign = v > 0 ? '+' : '';
  const flag = v > 0 ? ' ⚠️' : '';
  return `${sign}${v.toFixed(1)}%${flag}`;
}

function renderReport({ criterionComparison, k6Summary, k6Comparison, generatedAt }) {
  const lines = [];
  lines.push('## Performance Regression Report');
  lines.push('');
  lines.push(`_Generated ${generatedAt}_`);
  lines.push('');

  lines.push('### k6 Load Test — browse → queue → checkout → scan');
  lines.push('');
  if (k6Summary) {
    lines.push('| Metric | p50 | p95 | p99 |');
    lines.push('|---|---|---|---|');
    lines.push(
      `| Overall \`http_req_duration\` | ${fmtMs(k6Summary.httpReqDuration.p50)} | ${fmtMs(
        k6Summary.httpReqDuration.p95,
      )} | ${fmtMs(k6Summary.httpReqDuration.p99)} |`,
    );
    for (const [phase, trend] of Object.entries(k6Summary.perPhase)) {
      if (!trend) continue;
      lines.push(`| ${phase} | ${fmtMs(trend.p50)} | ${fmtMs(trend.p95)} | ${fmtMs(trend.p99)} |`);
    }
    lines.push('');
    lines.push(`**Throughput:** ${k6Summary.rps ? k6Summary.rps.toFixed(1) : '—'} req/s`);
    lines.push(`**Total requests:** ${k6Summary.totalRequests ?? '—'}`);
    lines.push(`**HTTP failure rate:** ${fmtPct(k6Summary.httpReqFailedRate)}`);
    lines.push(`**Funnel error rate:** ${fmtPct(k6Summary.funnelErrorRate)}`);

    if (k6Comparison.regressions.length > 0) {
      lines.push('');
      lines.push('**⚠️ Regressions vs. baseline:**');
      for (const r of k6Comparison.regressions) {
        lines.push(`- ${r.name}: ${fmtChange(r.change)}`);
      }
    }
  } else {
    lines.push('_No k6 summary found — run `scripts/run_stress_test.sh` first._');
  }
  lines.push('');

  lines.push('### Criterion Benchmarks');
  lines.push('');
  if (criterionComparison.rows.length > 0) {
    lines.push('| Benchmark | Mean | Std Dev | vs. Baseline |');
    lines.push('|---|---|---|---|');
    for (const row of criterionComparison.rows) {
      const flag = row.regressed ? ' ⚠️' : '';
      lines.push(
        `| ${row.name} | ${fmtMs(row.meanMs)} | ${fmtMs(row.stdDevMs)} | ${fmtChange(row.change)}${flag} |`,
      );
    }
  } else {
    lines.push(
      '_No criterion results found — run `cargo bench` in `server/` first (some benches ' +
        'require `DATABASE_URL` to register more than zero benchmarks; see `server/benches/`)._',
    );
  }

  if (criterionComparison.regressions.length > 0) {
    lines.push('');
    lines.push(
      `**⚠️ ${criterionComparison.regressions.length} benchmark(s) regressed beyond threshold.**`,
    );
  }

  return lines.join('\n') + '\n';
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

function main() {
  const args = parseArgs(process.argv.slice(2));

  const criterionResults = loadCriterionResults(args.criterionDir);
  const k6Summary = loadK6Summary(args.k6Summary);
  const baseline = loadBaseline(args.baseline);

  const criterionComparison = compareCriterion(criterionResults, baseline, args.thresholdPercent);
  const k6Comparison = compareK6(k6Summary, baseline, args.thresholdPercent);

  const report = renderReport({
    criterionComparison,
    k6Summary,
    k6Comparison,
    generatedAt: new Date().toISOString(),
  });

  console.log(report);

  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, report);
  }

  if (args.updateBaseline) {
    const newBaseline = {
      generatedAt: new Date().toISOString(),
      criterion: criterionResults.map(({ name, meanMs, stdDevMs }) => ({ name, meanMs, stdDevMs })),
      k6: k6Summary ? { httpReqDuration: k6Summary.httpReqDuration, rps: k6Summary.rps } : undefined,
    };
    writeFileSync(resolve(REPO_ROOT, args.baseline), JSON.stringify(newBaseline, null, 2) + '\n');
    console.log(`Baseline updated at ${args.baseline}`);
  }

  const totalRegressions = criterionComparison.regressions.length + k6Comparison.regressions.length;
  if (args.failOnRegression && totalRegressions > 0) {
    console.error(`\n${totalRegressions} performance regression(s) exceeded the ${args.thresholdPercent}% threshold.`);
    process.exit(1);
  }
}

main();
