# Topological Sort Agent

An **advanced topological sorting** library for Rust supporting Kahn's algorithm (BFS), priority-aware sorting, parallel level scheduling via Rayon, and incremental edge insertion without full re-sort. Detects cycles and reports them as ordered node sequences.

## Why It Matters

Topological sorting is one of the most fundamental graph algorithms — it produces a linear ordering of nodes where every edge (u, v) places u before v. This is essential for: build systems (make, cargo), task schedulers, course prerequisite planning, dependency resolution, and causal ordering in distributed systems. While basic topological sort is well-known, production systems need more: priority awareness (schedule critical-path nodes first), parallelism (nodes at the same topological level can run concurrently), and incremental updates (adding one edge shouldn't require a full re-sort). This library provides all four — making it suitable for real-time task graphs where the DAG evolves dynamically.

## How It Works

### Kahn's Algorithm (BFS)

The classic approach:

```
1. Compute in-degree for each node
2. Initialize queue with all zero-in-degree nodes
3. While queue non-empty:
   a. Dequeue node u, append to sorted output
   b. For each neighbor v of u: decrement in-degree(v)
   c. If in-degree(v) == 0: enqueue v
4. If output size < node count: cycle exists
```

Complexity: O(V + E) time, O(V) space.

### Priority-Aware Sort

When multiple nodes have in-degree 0, standard Kahn's picks arbitrarily. This library uses a **priority queue** (max-heap) instead:

```
queue = max-heap by priority
```

Nodes with higher priority (e.g., on the critical path) are scheduled first. This doesn't change the O(V + E) complexity but produces schedules that minimize total latency.

### Parallel Level Scheduling

Nodes at the same topological level (same BFS depth) have no dependencies between them and can execute in parallel. Using Rayon:

```rust
// Level 0: all zero-in-degree nodes
// Level 1: nodes whose in-edges all come from Level 0
// ...
for level in levels {
    level.par_iter().for_each(|node| process(node));
}
```

This enables maximum parallelism — each level is a wavefront.

### Incremental Edge Insertion

Adding a single edge (u, v) doesn't require re-sorting the entire DAG:

```
If u appears before v in the existing sort:
    → Already consistent, just add the edge
Else:
    → Need to re-sort the affected subset (u..v segment)
    → Worst case: full re-sort
    → Average case: O(k) where k = affected nodes
```

The `add_edge_incremental` method checks whether the existing sort is still valid and only re-sorts the necessary subset.

### Cycle Detection

When a cycle is found, the library returns the cycle path:

```rust
pub enum SortResult {
    Sorted(Vec<NodeId>),
    Cycle(Vec<NodeId>),  // the nodes forming the cycle
}
```

This is invaluable for debugging dependency graphs — you immediately see *which* nodes create the circular dependency.

## Quick Start

```rust
use topological_sort_agent::{TopoGraph, SortResult};

fn main() {
    let mut graph = TopoGraph::new();

    // Build DAG: 1→2, 1→3, 2→4, 3→4
    graph.add_edge(1, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 4);
    graph.add_edge(3, 4);

    // Set priorities (higher = first when available)
    graph.set_priority(3, 10);
    graph.set_priority(2, 5);

    match graph.kahn_sort() {
        SortResult::Sorted(order) => {
            println!("Topological order: {:?}", order);
            // [1, 3, 2, 4] — 3 before 2 due to higher priority
        }
        SortResult::Cycle(cycle) => {
            println!("Cycle: {:?}", cycle);
        }
    }

    // Incremental edge addition
    let mut order = vec![1, 3, 2, 4];
    match graph.add_edge_incremental(4, 5, &mut order) {
        SortResult::Sorted(o) => println!("Updated: {:?}", o),
        SortResult::Cycle(c) => println!("Cycle: {:?}", c),
    }
}
```

```bash
cargo build
cargo test
cargo bench  # criterion benchmarks
```

## API

| Type/Method | Description |
|-------------|-------------|
| `TopoGraph::new()` | Create empty DAG |
| `add_node(id)` | Add isolated node |
| `add_edge(from, to)` | Add directed edge |
| `add_edge_incremental(from, to, &mut sort)` | Insert edge, update sort incrementally |
| `set_priority(node, prio)` | Set scheduling priority |
| `kahn_sort()` | BFS-based sort with priority |
| `parallel_levels()` | Group nodes by topological level |
| `SortResult::Sorted(Vec) \| Cycle(Vec)` | Result enum |

## Architecture Notes

Topological Sort Agent provides the **ordering substrate** for γ + η = C. In the fleet, tasks have dependencies — agent A must compute before agent B can consume. Topological sort produces the valid execution order (γ — the constructive sequence). Priority-aware sorting optimizes for critical-path completion (maximizing C per unit time). Incremental updates handle the η side: when a task is *removed* (η elimination), the sort adapts without full recomputation. The cycle detection prevents deadlocks where γ and η create circular dependencies. See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Kahn, A. B. (1962). "Topological Sorting of Large Networks." *Communications of the ACM*, 5(11), 558–562.
2. Cormen, T. H., et al. (2009). *Introduction to Algorithms*, 3rd ed. MIT Press. Section 22.4.
3. Leiserson, C. E., & Mirman, P. (2010). "How to Write a Parallel Program." *MIT 6.172*. — On level scheduling for parallel DAG execution.

## License

MIT
