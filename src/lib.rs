use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};
use std::error::Error;

type NodeId = u64;

/// A directed graph supporting topological sort operations
pub struct TopoGraph {
    /// adjacency list: node -> set of outgoing neighbors
    edges: HashMap<NodeId, HashSet<NodeId>>,
    /// reverse edges for incoming tracking
    reverse: HashMap<NodeId, HashSet<NodeId>>,
    /// all nodes ever seen
    nodes: HashSet<NodeId>,
    /// priorities for nodes (higher = scheduled first when multiple valid)
    priorities: HashMap<NodeId, i64>,
    /// cached in-degrees for incremental updates
    in_degree: HashMap<NodeId, usize>,
}

#[derive(Debug, Clone)]
pub struct CycleError(pub Vec<NodeId>);

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle detected: {:?}", self.0)
    }
}
impl Error for CycleError {}

#[derive(Debug, Clone, PartialEq)]
pub enum SortResult {
    Sorted(Vec<NodeId>),
    Cycle(Vec<NodeId>),
}

impl TopoGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            reverse: HashMap::new(),
            nodes: HashSet::new(),
            priorities: HashMap::new(),
            in_degree: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: NodeId) {
        self.nodes.insert(node);
        self.in_degree.entry(node).or_insert(0);
    }

    pub fn set_priority(&mut self, node: NodeId, priority: i64) {
        self.priorities.insert(node, priority);
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.add_node(from);
        self.add_node(to);
        if self.edges.entry(from).or_default().insert(to) {
            *self.in_degree.entry(to).or_insert(0) += 1;
            self.reverse.entry(to).or_default().insert(from);
        }
    }

    /// Incremental: add an edge and update an existing sort if possible
    pub fn add_edge_incremental(&mut self, from: NodeId, to: NodeId, existing_sort: &mut Vec<NodeId>) -> SortResult {
        // Check if 'from' comes before 'to' in existing sort
        let pos_from = existing_sort.iter().position(|&n| n == from);
        let pos_to = existing_sort.iter().position(|&n| n == to);
        
        if let (Some(pf), Some(pt)) = (pos_from, pos_to) {
            if pf < pt {
                // Already in correct order, just add the edge
                self.add_edge(from, to);
                return SortResult::Sorted(existing_sort.clone());
            }
        }
        
        // Need to re-sort, but check for cycle first
        self.add_edge(from, to);
        match self.kahn_sort() {
            SortResult::Sorted(order) => {
                *existing_sort = order.clone();
                SortResult::Sorted(order)
            }
            SortResult::Cycle(cycle) => SortResult::Cycle(cycle),
        }
    }

    /// Kahn's algorithm (BFS-based topological sort)
    pub fn kahn_sort(&self) -> SortResult {
        let mut in_deg: HashMap<NodeId, usize> = self.nodes.iter()
            .map(|&n| (n, *self.in_degree.get(&n).unwrap_or(&0)))
            .collect();
        
        let mut queue: VecDeque<NodeId> = in_deg.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| n)
            .collect();
        queue.make_contiguous().sort();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            result.push(node);
            if let Some(neighbors) = self.edges.get(&node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_deg.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        
        if result.len() == self.nodes.len() {
            SortResult::Sorted(result)
        } else {
            // Find cycle
            let remaining: Vec<NodeId> = in_deg.iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(&n, _)| n)
                .collect();
            SortResult::Cycle(remaining)
        }
    }

    /// DFS-based topological sort
    pub fn dfs_sort(&self) -> SortResult {
        let mut visited = HashSet::new();
        let mut on_stack = HashSet::new();
        let mut result = Vec::new();
        let mut cycle: Vec<NodeId> = Vec::new();
        
        fn dfs(
            node: NodeId,
            graph: &TopoGraph,
            visited: &mut HashSet<NodeId>,
            on_stack: &mut HashSet<NodeId>,
            result: &mut Vec<NodeId>,
            cycle: &mut Vec<NodeId>,
        ) -> bool {
            if on_stack.contains(&node) {
                cycle.push(node);
                return false;
            }
            if visited.contains(&node) {
                return true;
            }
            visited.insert(node);
            on_stack.insert(node);
            
            if let Some(neighbors) = graph.edges.get(&node) {
                for &neighbor in neighbors {
                    if !dfs(neighbor, graph, visited, on_stack, result, cycle) {
                        if !cycle.is_empty() && cycle[0] == node {
                            cycle.push(node);
                            return false;
                        }
                        if !cycle.is_empty() {
                            cycle.push(node);
                            return false;
                        }
                        return false;
                    }
                }
            }
            
            on_stack.remove(&node);
            result.push(node);
            true
        }
        
        let mut nodes: Vec<NodeId> = self.nodes.iter().copied().collect();
        nodes.sort();
        
        for node in nodes {
            if !visited.contains(&node) {
                if !dfs(node, self, &mut visited, &mut on_stack, &mut result, &mut cycle) {
                    return SortResult::Cycle(cycle.into_iter().rev().collect());
                }
            }
        }
        
        result.reverse();
        SortResult::Sorted(result)
    }

    /// Priority-aware topological sort using a max-heap
    pub fn priority_sort(&self) -> SortResult {
        let mut in_deg: HashMap<NodeId, usize> = self.nodes.iter()
            .map(|&n| (n, *self.in_degree.get(&n).unwrap_or(&0)))
            .collect();
        
        // Max-heap by priority (negative for min-heap behavior)
        let mut heap: BinaryHeap<(i64, NodeId)> = in_deg.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| (*self.priorities.get(&n).unwrap_or(&0), n))
            .collect();
        
        let mut result = Vec::new();
        
        while let Some((_, node)) = heap.pop() {
            result.push(node);
            if let Some(neighbors) = self.edges.get(&node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_deg.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            let prio = *self.priorities.get(&neighbor).unwrap_or(&0);
                            heap.push((prio, neighbor));
                        }
                    }
                }
            }
        }
        
        if result.len() == self.nodes.len() {
            SortResult::Sorted(result)
        } else {
            SortResult::Cycle(result) // incomplete = has cycle
        }
    }

    /// Parallel topological sort - identify levels, sort within each level using rayon
    pub fn parallel_sort(&self) -> SortResult {
        let mut in_deg: HashMap<NodeId, usize> = self.nodes.iter()
            .map(|&n| (n, *self.in_degree.get(&n).unwrap_or(&0)))
            .collect();
        
        let mut result = Vec::new();
        let mut current_level: Vec<NodeId> = in_deg.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| n)
            .collect();
        
        while !current_level.is_empty() {
            // Sort current level in parallel (trivial parallelism for sorting)
            current_level.sort();
            
            // Process all nodes at this level in parallel to collect next level
            let next_level_sets: Vec<Vec<NodeId>> = current_level.par_iter()
                .map(|&node| {
                    let mut next = Vec::new();
                    if let Some(neighbors) = self.edges.get(&node) {
                        for &neighbor in neighbors {
                            next.push(neighbor);
                        }
                    }
                    next
                })
                .collect();
            
            result.extend_from_slice(&current_level);
            
            let mut next_level: BTreeSet<NodeId> = BTreeSet::new();
            for nodes in &next_level_sets {
                for &neighbor in nodes {
                    if let Some(deg) = in_deg.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_level.insert(neighbor);
                        }
                    }
                }
            }
            
            current_level = next_level.into_iter().collect();
        }
        
        if result.len() == self.nodes.len() {
            SortResult::Sorted(result)
        } else {
            SortResult::Cycle(result)
        }
    }

    /// Detect if the graph contains a cycle
    pub fn has_cycle(&self) -> Option<Vec<NodeId>> {
        match self.kahn_sort() {
            SortResult::Sorted(_) => None,
            SortResult::Cycle(c) => Some(c),
        }
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|s| s.len()).sum()
    }
}

use rayon::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = TopoGraph::new();
        assert_eq!(g.kahn_sort(), SortResult::Sorted(vec![]));
    }

    #[test]
    fn test_single_node() {
        let mut g = TopoGraph::new();
        g.add_node(1);
        assert_eq!(g.kahn_sort(), SortResult::Sorted(vec![1]));
    }

    #[test]
    fn test_linear_chain() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        g.add_edge(3, 4);
        assert_eq!(g.kahn_sort(), SortResult::Sorted(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_diamond() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 4);
        g.add_edge(3, 4);
        match g.kahn_sort() {
            SortResult::Sorted(order) => {
                assert_eq!(order[0], 1);
                assert_eq!(order[3], 4);
                assert_eq!(order.len(), 4);
            }
            _ => panic!("Expected sorted"),
        }
    }

    #[test]
    fn test_detect_cycle() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        g.add_edge(3, 1);
        assert!(matches!(g.kahn_sort(), SortResult::Cycle(_)));
        assert!(g.has_cycle().is_some());
    }

    #[test]
    fn test_dfs_matches_kahn() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 4);
        g.add_edge(3, 4);
        let kahn = g.kahn_sort();
        let dfs = g.dfs_sort();
        assert!(matches!(&kahn, SortResult::Sorted(_)));
        assert!(matches!(&dfs, SortResult::Sorted(_)));
        if let (SortResult::Sorted(k), SortResult::Sorted(d)) = (&kahn, &dfs) {
            assert_eq!(k.len(), d.len());
        }
    }

    #[test]
    fn test_priority_sort() {
        let mut g = TopoGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_node(3);
        g.add_node(4);
        g.set_priority(1, 10);
        g.set_priority(2, 5);
        g.set_priority(3, 20);
        g.set_priority(4, 1);
        g.add_edge(1, 4);
        g.add_edge(2, 4);
        g.add_edge(3, 4);
        
        match g.priority_sort() {
            SortResult::Sorted(order) => {
                assert_eq!(order[3], 4); // 4 must be last
                // Among 1,2,3 they should be ordered by priority: 3(20), 1(10), 2(5)
                assert_eq!(order[0], 3);
                assert_eq!(order[1], 1);
                assert_eq!(order[2], 2);
            }
            _ => panic!("Expected sorted"),
        }
    }

    #[test]
    fn test_parallel_sort() {
        let mut g = TopoGraph::new();
        for i in 0..100 {
            g.add_node(i);
            if i > 0 { g.add_edge(i - 1, i); }
        }
        match g.parallel_sort() {
            SortResult::Sorted(order) => {
                assert_eq!(order.len(), 100);
                for i in 0..100 {
                    assert_eq!(order[i], i as u64);
                }
            }
            _ => panic!("Expected sorted"),
        }
    }

    #[test]
    fn test_incremental_add_edge_already_sorted() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        let mut sort = match g.kahn_sort() {
            SortResult::Sorted(s) => s,
            _ => panic!("expected sorted"),
        };
        // Add edge that doesn't violate order
        let result = g.add_edge_incremental(1, 3, &mut sort);
        assert!(matches!(result, SortResult::Sorted(_)));
    }

    #[test]
    fn test_incremental_add_edge_creates_cycle() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        let mut sort = match g.kahn_sort() {
            SortResult::Sorted(s) => s,
            _ => panic!("expected sorted"),
        };
        let result = g.add_edge_incremental(3, 1, &mut sort);
        assert!(matches!(result, SortResult::Cycle(_)));
    }

    #[test]
    fn test_node_and_edge_count() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        g.add_edge(1, 3);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn test_disconnected_components() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(3, 4);
        match g.kahn_sort() {
            SortResult::Sorted(order) => {
                assert_eq!(order.len(), 4);
                // 1 before 2, 3 before 4
                assert!(order.iter().position(|&x| x == 1) < order.iter().position(|&x| x == 2));
                assert!(order.iter().position(|&x| x == 3) < order.iter().position(|&x| x == 4));
            }
            _ => panic!("Expected sorted"),
        }
    }

    #[test]
    fn test_large_graph() {
        let mut g = TopoGraph::new();
        for i in 1u64..1000 {
            g.add_edge(i, i + 1);
        }
        match g.kahn_sort() {
            SortResult::Sorted(order) => {
                assert_eq!(order.len(), 1000);
            }
            _ => panic!("Expected sorted"),
        }
    }

    #[test]
    fn test_self_loop_cycle() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 1);
        assert!(g.has_cycle().is_some());
    }

    #[test]
    fn test_priority_with_cycle() {
        let mut g = TopoGraph::new();
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        g.add_edge(3, 1);
        g.set_priority(1, 100);
        assert!(matches!(g.priority_sort(), SortResult::Cycle(_)));
    }
}
