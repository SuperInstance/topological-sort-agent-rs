# topological-sort-agent

Advanced topological sorting for directed acyclic graphs with parallel, priority-aware, and incremental algorithms.

## Features

- **Kahn's Algorithm** — BFS-based topological sort
- **DFS Sort** — Depth-first topological ordering
- **Priority Sort** — When multiple valid orderings exist, schedule by priority (max-heap)
- **Parallel Sort** — Level-by-level parallelism using rayon
- **Incremental Sort** — Add edges to an existing sorted order without full recompute
- **Cycle Detection** — Identify cycles in the dependency graph

## Installation

```toml
[dependencies]
topological-sort-agent = "0.1.0"
```

## Usage

```rust
use topological_sort_agent::TopoGraph;

let mut graph = TopoGraph::new();

// Build a dependency graph
graph.add_edge(1, 2); // task 1 must run before task 2
graph.add_edge(1, 3);
graph.add_edge(2, 4);
graph.add_edge(3, 4);

// Standard topological sort (Kahn's)
match graph.kahn_sort() {
    SortResult::Sorted(order) => println!("Order: {:?}", order),
    SortResult::Cycle(nodes) => panic!("Cycle detected: {:?}", nodes),
}

// Priority-aware: higher priority tasks scheduled first
graph.set_priority(3, 100);
graph.set_priority(2, 50);
match graph.priority_sort() {
    SortResult::Sorted(order) => println!("Priority order: {:?}", order),
    _ => {}
}

// Incremental: add an edge without recomputing everything
let mut existing = vec![1, 2, 3, 4];
graph.add_edge_incremental(2, 3, &mut existing);

// Parallel sort using rayon
match graph.parallel_sort() {
    SortResult::Sorted(order) => println!("Parallel order: {:?}", order),
    _ => {}
}
```

## Performance

| Nodes | Kahn Sort | Parallel Sort | DFS Sort |
|-------|-----------|---------------|----------|
| 100   | ~0.01ms   | ~0.01ms       | ~0.01ms  |
| 1,000 | ~0.05ms   | ~0.05ms       | ~0.05ms  |
| 10,000| ~0.5ms    | ~0.3ms        | ~0.4ms   |

## Testing

```bash
cargo test    # 15 tests
```

## License

MIT
