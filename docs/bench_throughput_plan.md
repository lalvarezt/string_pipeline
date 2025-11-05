# 📊 Bench Throughput Analysis Enhancement Plan

**Project**: string_pipeline
**Use Case**: Performance analysis for television TUI file browser
**Last Updated**: 2025-11-05

## 🎯 Problem Statement

The television project receives large lists of file paths that need formatting via templates. We need:
1. **Scaling analysis** - How performance changes with input size (100 → 100K paths)
2. **Operation-level profiling** - Which specific operations are bottlenecks
3. **Cache effectiveness** - Understanding the impact of split/regex caching
4. **Real-world templates** - Focused on file path use cases
5. **Actionable insights** - Data to drive optimization decisions

## 🔍 Current State Analysis

### ✅ What's Working Well
- Parse-once, format-many pattern (optimal for library usage)
- Realistic path generation with varying depths
- Scaling factor analysis
- Multiple input sizes (100 → 100K)
- Warmup iterations for stable measurements

### ❌ What's Missing

**1. Granular Breakdown**
- Only measures total format time, not individual operations
- No visibility into which operations dominate (split vs join vs regex)
- Can't identify optimization opportunities

**2. Limited Template Coverage**
- Only 7 templates tested
- Missing: `strip_ansi`, `regex_extract`, `pad`, `surround`, `unique`, `sort`
- Missing combinations: `{split:/:..|map:{upper}|join:/}`

**3. Cache Analytics**
- Split cache exists but no hit/miss tracking
- Regex cache exists but no effectiveness metrics
- Can't quantify caching benefit

**4. No Per-Operation Metrics**
- Need: time per split, time per join, time per regex
- Need: memory allocation patterns
- Need: operation call counts

**5. Output Limitations**
- Only human-readable console output
- Can't track performance over time (no JSON output)
- No comparison between git commits

## 📋 Implementation Phases

### Phase 1: Instrumentation Infrastructure ⚙️

Add internal timing hooks to measure individual operations:

```rust
// New struct to track operation-level metrics
struct OperationMetrics {
    operation_name: String,
    total_time: Duration,
    call_count: usize,
    avg_time_per_call: Duration,
}

// New struct to track cache metrics
struct CacheMetrics {
    split_cache_hits: usize,
    split_cache_misses: usize,
    regex_cache_hits: usize,
    regex_cache_misses: usize,
}
```

**Implementation approach:**
- Add optional instrumentation flag to `apply_ops_internal`
- Collect timing for each operation type
- Track cache access patterns
- Minimal overhead when disabled

### Phase 2: Comprehensive Template Suite 📝

Expand to **25+ templates** covering all operations:

**Core Operations (Individual):**
1. `{split:/:..}` - Split only
2. `{split:/:-1}` - Split with index
3. `{join:/}` - Join only
4. `{upper}` - Case conversion
5. `{lower}` - Case conversion
6. `{trim}` - Whitespace removal
7. `{replace:s/\\.txt$/.md/}` - Simple regex
8. `{replace:s/\\/\\/+/\\//g}` - Complex regex with global flag
9. `{substring:0..10}` - Substring extraction
10. `{reverse}` - String reversal
11. `{strip_ansi}` - ANSI stripping
12. `{filter:\\.txt$}` - Filtering
13. `{sort}` - Sorting
14. `{unique}` - Deduplication
15. `{pad:50: :right}` - Padding

**Real-World Path Templates (Television Use Cases):**
16. `{split:/:-1}` - Extract filename
17. `{split:/:0..-1|join:/}` - Extract directory
18. `{split:/:-1|split:.:0}` - Basename without extension
19. `{split:/:-1|split:.:-1}` - File extension
20. `{replace:s/^.*\\/([^/]+)$/$1/}` - Regex-based filename extraction
21. `{split:/:..|map:{upper}|join:/}` - Uppercase all components (expensive!)
22. `{split:/:..|filter_not:^\\.|join:/}` - Remove hidden dirs
23. `{split:/:-1|trim|lower}` - Normalize filename
24. `{replace:s/ /_/g|lower}` - Slug generation
25. `{split:/:..|slice:-3..|join: > }` - Show last 3 dirs with breadcrumb

**Combination Chains (Multi-Operation):**
- Test operation composition overhead
- Measure map operation performance impact

### Phase 3: Per-Operation Profiling 🔬

Add detailed breakdown output:

```
==================================================
Operation Performance Breakdown (100K paths)
==================================================
Operation          Calls    Total Time    Avg/Call    % of Total
-----------------------------------------------------------------
split:/:..        100,000     45.2ms      452ns        35.2%
map:{...}         100,000     52.8ms      528ns        41.1%
  ↳ trim          100,000      8.2ms       82ns        15.5% (of map)
  ↳ upper         100,000     18.6ms      186ns        35.2% (of map)
join:/            100,000     15.3ms      153ns        11.9%
-----------------------------------------------------------------
Total Format                 128.5ms
Cache Hit Rate (split):     98.2% (98,200 hits, 1,800 misses)
Cache Hit Rate (regex):     100% (50,000 hits, 0 misses)
Memory Allocations:         3.2M (32 bytes/path avg)
```

### Phase 4: Cache Effectiveness Analysis 💾

Instrument cache access patterns:

```rust
struct CacheAnalysis {
    // Per-template cache behavior
    split_cache_effectiveness: f64,  // 0.0 to 1.0
    regex_cache_effectiveness: f64,

    // Cache pressure metrics
    cache_size_bytes: usize,
    eviction_count: usize,

    // Benefit quantification
    time_saved_by_caching: Duration,
}
```

**Key insights to extract:**
- Which templates benefit most from caching
- Optimal cache size for real-world usage
- When to clear caches

### Phase 5: Statistical Analysis 📈

Beyond averages, add:
- **Percentiles**: p50, p95, p99 latency
- **Standard deviation**: Measure consistency
- **Outlier detection**: Identify anomalies
- **Warmup analysis**: Cold vs hot performance

```
Statistical Analysis (100K paths):
  Min:      1.15ms
  p50:      1.28ms
  p95:      1.45ms
  p99:      1.82ms
  Max:      3.21ms
  Stddev:   0.15ms
  Outliers: 127 (0.127%)
```

### Phase 6: Output Formats 📄

Add machine-readable JSON output:

```json
{
  "benchmark_id": "extract_filename",
  "template": "{split:/:-1}",
  "timestamp": "2025-11-05T10:30:00Z",
  "git_commit": "df93f9b",
  "input_sizes": [100, 500, 1000, ...],
  "results": [{
    "input_size": 100000,
    "parse_time_ns": 12450,
    "total_format_time_ns": 128500000,
    "throughput_per_sec": 778210.5,
    "operations": [
      {"name": "split", "time_ns": 45200000, "calls": 100000},
      ...
    ],
    "cache": {
      "split_hit_rate": 0.982,
      "regex_hit_rate": 1.0
    },
    "statistics": {
      "min_ns": 1150,
      "p50_ns": 1280,
      "p95_ns": 1450,
      "p99_ns": 1820,
      "max_ns": 3210,
      "stddev_ns": 150
    }
  }]
}
```

**Benefits:**
- Track performance over time
- Compare before/after optimizations
- Generate visualizations (gnuplot, matplotlib)
- Future CI/CD integration

### Phase 7: Comparative Analysis 🔄

Add regression detection:

```rust
// Compare two benchmark runs
struct BenchmarkComparison {
    baseline: BenchmarkResult,
    current: BenchmarkResult,

    regression_detected: bool,
    improvement_percent: f64,

    operation_deltas: Vec<OperationDelta>,
}
```

**Use cases:**
- Detect performance regressions in CI
- Quantify optimization improvements
- A/B test different implementations

### Phase 8: Memory Profiling 🧠

Add memory tracking:

```rust
struct MemoryMetrics {
    peak_memory_bytes: usize,
    total_allocations: usize,
    bytes_per_path: f64,

    // Per-operation memory
    split_allocations: usize,
    join_allocations: usize,
    regex_allocations: usize,
}
```

**Key questions to answer:**
- Memory usage growth with input size
- Which operations allocate most
- Opportunities for pooling/reuse

### Phase 9: Real-World Scenarios 🌍

Add television-specific benchmarks:

```rust
enum ScenarioType {
    // Television channel types
    FileBrowser,      // Large directory listings
    GitFiles,         // Repository file lists
    ProcessList,      // System processes
    SearchResults,    // ripgrep output
}
```

**Example: FileBrowser scenario**
- 50,000 real paths from typical projects
- Templates: filename extraction, syntax highlighting prep
- Measure: time to format entire TUI buffer
- Goal: < 16ms for 60 FPS rendering

### Phase 10: Optimization Guidance 🎓

Generate actionable recommendations:

```
🎯 Optimization Recommendations:

1. [HIGH IMPACT] Split operation takes 35% of time
   → Consider pre-splitting common separators
   → Increase split cache size from 10K to 50K chars

2. [MEDIUM IMPACT] Map operation has 15% overhead
   → For simple operations, consider flattening
   → Profile allocation patterns in map closure

3. [LOW IMPACT] Cache hit rate is 98.2%
   ✓ Current caching strategy is effective
   → No action needed
```

## 🚀 Implementation Priority

**High Priority (Do First):**
1. ✅ Phase 2: Complete template coverage (comprehensive test suite)
2. ✅ Phase 3: Per-operation timing breakdown (core instrumentation)
3. ✅ Phase 6: JSON output (tracking over time)

**Medium Priority:**
4. Phase 4: Cache analysis (understand optimization opportunities)
5. Phase 5: Statistical analysis (reliability metrics)
6. Phase 9: Real-world scenarios (television-specific)

**Lower Priority (Nice to Have):**
7. Phase 7: Comparative analysis (regression detection)
8. Phase 8: Memory profiling (deep optimization)
9. Phase 10: Auto-recommendations (advanced analysis)

## 🎨 Proposed CLI Interface

```bash
# Basic usage (existing)
bench_throughput --sizes 1000,10000,100000 --iterations 50

# New: Detailed breakdown
bench_throughput --detailed --operation-timing

# New: JSON output
bench_throughput --format json --output results.json

# New: Compare runs
bench_throughput --compare baseline.json

# New: Television scenario
bench_throughput --scenario file-browser --real-paths ~/projects

# New: Cache analysis
bench_throughput --analyze-cache

# New: Memory profiling
bench_throughput --profile-memory
```

## 📊 Success Metrics

After implementation, you'll be able to answer:

✅ **"Which operation should I optimize first?"**
   → Per-operation timing breakdown shows bottlenecks

✅ **"Is my optimization working?"**
   → JSON output enables before/after comparison

✅ **"How does it scale to television's use case?"**
   → Real-world scenario benchmarks with 50K paths

✅ **"Are the caches effective?"**
   → Cache hit rate and time-saved metrics

✅ **"What's the memory footprint?"**
   → Memory profiling per operation

✅ **"Can we handle 100K paths in < 100ms?"**
   → Throughput metrics at scale

## 🔧 Technical Approach

**Minimal Library Changes:**
- Add optional instrumentation via feature flag or conditional compilation
- Use `thread_local!` for per-thread metrics
- Zero overhead when disabled
- Backward compatible

**Benchmark Architecture:**
```
bench_throughput.rs
├── BenchmarkRunner (orchestration)
├── MetricsCollector (instrumentation)
├── ResultsAnalyzer (statistics)
├── OutputFormatter (JSON/console)
└── TemplateRegistry (comprehensive suite)
```

## ❓ Open Questions & Design Decisions

1. **Instrumentation overhead**: Target < 5% overhead acceptable
2. **Cache instrumentation**: Wrapper around dashmap for tracking
3. **Memory profiling**: Custom tracking for precision
4. **Real paths**: Generate synthetic paths (varied depths, realistic names)
5. **CI integration**: Defer specifics for later
6. **CSV output**: Not needed - JSON is sufficient

## 📝 Notes

- CSV output is not needed (JSON covers machine-readable needs)
- CI/CD integration specifics deferred to later
- Focus on immediate value: operation profiling and comprehensive templates
- Keep backward compatibility - existing bench tools should continue working
