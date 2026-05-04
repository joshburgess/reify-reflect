#![deny(unsafe_code)]

//! Convert `Rc<RefCell<T>>` and `Arc<Mutex<T>>` pointer graphs into a flat,
//! inspectable, serializable form, and back.
//!
//! Pointer-shaped data (trees with shared subtrees, DAGs, cyclic graphs)
//! is awkward to serialize, log, or transform: walking it naively either
//! duplicates shared nodes or loops forever on cycles. This crate gives
//! you a small, non-invasive API to:
//!
//! - **Reify**: turn a pointer graph rooted at an `Rc<RefCell<T>>` (or
//!   `Arc<Mutex<T>>`) into a flat [`ReifiedGraph`] of nodes and edges,
//!   keyed by pointer identity, detecting cycles and preserving sharing.
//! - **Reflect**: rebuild the original pointer graph from a [`ReifiedGraph`],
//!   restoring the same sharing topology (one allocation per node id, even
//!   when many edges point to it).
//!
//! With the default `serde` feature enabled, [`ReifiedGraph`] is
//! `Serialize` + `Deserialize`, which gives you JSON / postcard / etc.
//! serialization of arbitrary cyclic data essentially for free.
//!
//! # When to use it
//!
//! - You want to dump or log a graph for debugging (an AST with shared
//!   sub-expressions, a circuit netlist, a scene graph).
//! - You want to send pointer-shaped state across a process boundary.
//! - You want to apply a structural transform (renumbering, GC, deduping)
//!   that's awkward on the live `Rc` graph but easy on a flat node+edge
//!   form.
//!
//! # Design choices
//!
//! - Node identity comes from [`Rc::as_ptr`] (or `Arc::as_ptr`) cast to
//!   `usize`. No traits or wrappers required on your `T`.
//! - Children are extracted by a closure you supply, so the library never
//!   guesses at the structure of your type.
//! - Node data is cloned into the [`ReifiedGraph`] (use `Arc<Inner>` inside
//!   `T` if cloning is expensive).
//! - Reconstruction in [`reflect_graph`] preserves sharing exactly: each
//!   [`NodeId`] becomes one [`Rc`] and is re-pointed everywhere it was
//!   referenced.
//!
//! See the [`serialize_graph` example][example] for an end-to-end
//! `serde_json` round trip.
//!
//! [example]: https://github.com/joshburgess/reify-reflect/blob/main/reify-graph/examples/serialize_graph.rs
//!
//! # Examples
//!
//! Round-trip a small `Rc<RefCell<_>>` tree through the flat form:
//!
//! ```
//! use reify_graph::{reify_graph, reflect_graph};
//! use std::cell::RefCell;
//! use std::rc::Rc;
//!
//! #[derive(Clone, Debug)]
//! struct Node {
//!     value: i32,
//!     children: Vec<Rc<RefCell<Node>>>,
//! }
//!
//! let leaf = Rc::new(RefCell::new(Node { value: 1, children: vec![] }));
//! let root = Rc::new(RefCell::new(Node {
//!     value: 0,
//!     children: vec![leaf.clone()],
//! }));
//!
//! // Flatten: 2 nodes, 1 edge
//! let graph = reify_graph(root.clone(), |n| n.children.clone());
//! assert_eq!(graph.nodes.len(), 2);
//! assert_eq!(graph.edges.len(), 1);
//!
//! // Rebuild, restoring the original sharing
//! let reconstructed = reflect_graph(graph, |n, kids| n.children = kids);
//! assert_eq!(reconstructed.borrow().value, 0);
//! assert_eq!(reconstructed.borrow().children[0].borrow().value, 1);
//! ```
//!
//! For `Arc<Mutex<T>>` graphs (the same pattern, thread-safe), see the
//! [`arc`] module's `reify_graph_arc` and `reflect_graph_arc`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod arc;

/// A stable node identifier derived from pointer identity.
///
/// # Examples
///
/// ```
/// use reify_graph::NodeId;
///
/// let id = NodeId(42);
/// assert_eq!(id.0, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(pub usize);

/// An adjacency-list representation of a pointer graph.
///
/// Contains all nodes with their data, directed edges between nodes,
/// and the identifier of the root node.
///
/// # Examples
///
/// ```
/// use reify_graph::{ReifiedGraph, NodeId};
///
/// let graph = ReifiedGraph {
///     nodes: vec![(NodeId(0), "root"), (NodeId(1), "child")],
///     edges: vec![(NodeId(0), NodeId(1))],
///     root: NodeId(0),
/// };
/// assert_eq!(graph.nodes.len(), 2);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReifiedGraph<T> {
    /// The nodes in the graph, each paired with its stable identifier.
    pub nodes: Vec<(NodeId, T)>,
    /// Directed edges from parent to child.
    pub edges: Vec<(NodeId, NodeId)>,
    /// The root node's identifier.
    pub root: NodeId,
}

/// Extract a [`NodeId`] from an `Rc<RefCell<T>>` using pointer identity.
///
/// # Examples
///
/// ```
/// use reify_graph::{node_id_of, NodeId};
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// let a = Rc::new(RefCell::new(42));
/// let b = a.clone();
/// assert_eq!(node_id_of(&a), node_id_of(&b));
/// ```
pub fn node_id_of<T>(rc: &Rc<RefCell<T>>) -> NodeId {
    NodeId(Rc::as_ptr(rc) as *const () as usize)
}

/// Collect all reachable nodes from `root`, detecting shared pointers.
///
/// Returns a map from [`NodeId`] to the corresponding `Rc<RefCell<T>>`.
///
/// # Examples
///
/// ```
/// use reify_graph::collect_nodes;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// #[derive(Clone)]
/// struct Node { children: Vec<Rc<RefCell<Node>>> }
///
/// let leaf = Rc::new(RefCell::new(Node { children: vec![] }));
/// let root = Rc::new(RefCell::new(Node { children: vec![leaf.clone()] }));
///
/// let nodes = collect_nodes(&root, &|n: &Node| n.children.clone());
/// assert_eq!(nodes.len(), 2);
/// ```
pub fn collect_nodes<T, F>(root: &Rc<RefCell<T>>, children: &F) -> HashMap<NodeId, Rc<RefCell<T>>>
where
    F: Fn(&T) -> Vec<Rc<RefCell<T>>>,
{
    let mut visited = HashMap::new();
    collect_nodes_inner(root, children, &mut visited);
    visited
}

fn collect_nodes_inner<T, F>(
    node: &Rc<RefCell<T>>,
    children: &F,
    visited: &mut HashMap<NodeId, Rc<RefCell<T>>>,
) where
    F: Fn(&T) -> Vec<Rc<RefCell<T>>>,
{
    let id = node_id_of(node);
    if visited.contains_key(&id) {
        return;
    }
    visited.insert(id, node.clone());

    let kids = children(&node.borrow());
    for kid in &kids {
        collect_nodes_inner(kid, children, visited);
    }
}

/// Reify an `Rc<RefCell<T>>` graph into a [`ReifiedGraph`].
///
/// The `children` closure extracts child pointers from a node's data.
/// The resulting graph contains cloned node data and edges derived
/// from pointer identity.
///
/// # Examples
///
/// ```
/// use reify_graph::reify_graph;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// #[derive(Clone, Debug)]
/// struct Node {
///     value: i32,
///     children: Vec<Rc<RefCell<Node>>>,
/// }
///
/// let shared = Rc::new(RefCell::new(Node { value: 2, children: vec![] }));
/// let root = Rc::new(RefCell::new(Node {
///     value: 0,
///     children: vec![
///         Rc::new(RefCell::new(Node { value: 1, children: vec![shared.clone()] })),
///         shared.clone(),
///     ],
/// }));
///
/// let graph = reify_graph(root, |n| n.children.clone());
/// // 3 unique nodes: root(0), child(1), shared(2)
/// assert_eq!(graph.nodes.len(), 3);
/// ```
pub fn reify_graph<T, F>(root: Rc<RefCell<T>>, children: F) -> ReifiedGraph<T>
where
    F: Fn(&T) -> Vec<Rc<RefCell<T>>>,
    T: Clone,
{
    let all_nodes = collect_nodes(&root, &children);
    let root_id = node_id_of(&root);

    let mut nodes = Vec::with_capacity(all_nodes.len());
    let mut edges = Vec::new();

    for (id, rc) in &all_nodes {
        let borrowed = rc.borrow();
        nodes.push((*id, borrowed.clone()));

        for kid in children(&borrowed) {
            let kid_id = node_id_of(&kid);
            edges.push((*id, kid_id));
        }
    }

    ReifiedGraph {
        nodes,
        edges,
        root: root_id,
    }
}

/// Reconstruct an `Rc<RefCell<T>>` graph from a [`ReifiedGraph`].
///
/// The `set_children` closure wires up child pointers on a node given
/// its reconstructed children.
///
/// # Examples
///
/// ```
/// use reify_graph::{reify_graph, reflect_graph};
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// #[derive(Clone, Debug)]
/// struct Node {
///     value: i32,
///     children: Vec<Rc<RefCell<Node>>>,
/// }
///
/// let leaf = Rc::new(RefCell::new(Node { value: 1, children: vec![] }));
/// let root = Rc::new(RefCell::new(Node { value: 0, children: vec![leaf] }));
///
/// let graph = reify_graph(root, |n| n.children.clone());
/// let rebuilt = reflect_graph(graph, |n, kids| n.children = kids);
///
/// assert_eq!(rebuilt.borrow().value, 0);
/// assert_eq!(rebuilt.borrow().children[0].borrow().value, 1);
/// ```
pub fn reflect_graph<T, F>(graph: ReifiedGraph<T>, set_children: F) -> Rc<RefCell<T>>
where
    F: Fn(&mut T, Vec<Rc<RefCell<T>>>),
    T: Clone,
{
    // Build Rc<RefCell<T>> for each node (without children wired up yet)
    let mut rc_map: HashMap<NodeId, Rc<RefCell<T>>> = HashMap::new();
    for (id, data) in &graph.nodes {
        rc_map.insert(*id, Rc::new(RefCell::new(data.clone())));
    }

    // Build adjacency: parent -> [child_ids] preserving edge order
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for (from, to) in &graph.edges {
        adj.entry(*from).or_default().push(*to);
    }

    // Wire up children
    for (id, rc) in &rc_map {
        if let Some(child_ids) = adj.get(id) {
            let children: Vec<Rc<RefCell<T>>> =
                child_ids.iter().map(|cid| rc_map[cid].clone()).collect();
            set_children(&mut rc.borrow_mut(), children);
        }
    }

    rc_map[&graph.root].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Node {
        value: i32,
        children: Vec<Rc<RefCell<Node>>>,
    }

    fn children_of(n: &Node) -> Vec<Rc<RefCell<Node>>> {
        n.children.clone()
    }

    fn set_children_of(n: &mut Node, kids: Vec<Rc<RefCell<Node>>>) {
        n.children = kids;
    }

    #[test]
    fn node_id_identity() {
        let a = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![],
        }));
        let b = a.clone();
        assert_eq!(node_id_of(&a), node_id_of(&b));

        let c = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![],
        }));
        assert_ne!(node_id_of(&a), node_id_of(&c));
    }

    #[test]
    fn collect_nodes_simple_linked_list() {
        let c = Rc::new(RefCell::new(Node {
            value: 2,
            children: vec![],
        }));
        let b = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![c.clone()],
        }));
        let a = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![b.clone()],
        }));

        let nodes = collect_nodes(&a, &children_of);
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn collect_nodes_with_cycle() {
        let a = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![],
        }));
        let b = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![a.clone()],
        }));
        // Create cycle: a -> b -> a
        a.borrow_mut().children.push(b.clone());

        let nodes = collect_nodes(&a, &children_of);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn reify_dag_with_shared_children() {
        // DAG: root -> {a, b, shared}, a -> {shared}, b -> {shared}
        let shared = Rc::new(RefCell::new(Node {
            value: 99,
            children: vec![],
        }));
        let a = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![shared.clone()],
        }));
        let b = Rc::new(RefCell::new(Node {
            value: 2,
            children: vec![shared.clone()],
        }));
        let root = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![a.clone(), b.clone(), shared.clone()],
        }));

        let graph = reify_graph(root, children_of);

        // 4 unique nodes
        assert_eq!(graph.nodes.len(), 4);
        // root->a, root->b, root->shared, a->shared, b->shared = 5 edges
        assert_eq!(graph.edges.len(), 5);
    }

    #[test]
    fn reify_five_node_dag() {
        // n0 -> {n1, n2}, n1 -> {n3, n4}, n2 -> {n3}, n3 -> {n4}
        let n4 = Rc::new(RefCell::new(Node {
            value: 4,
            children: vec![],
        }));
        let n3 = Rc::new(RefCell::new(Node {
            value: 3,
            children: vec![n4.clone()],
        }));
        let n2 = Rc::new(RefCell::new(Node {
            value: 2,
            children: vec![n3.clone()],
        }));
        let n1 = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![n3.clone(), n4.clone()],
        }));
        let n0 = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![n1.clone(), n2.clone()],
        }));

        let graph = reify_graph(n0, children_of);
        assert_eq!(graph.nodes.len(), 5);
        // n0->n1, n0->n2, n1->n3, n1->n4, n2->n3, n3->n4 = 6 edges
        assert_eq!(graph.edges.len(), 6);
    }

    #[test]
    fn round_trip_reify_reflect() {
        let leaf1 = Rc::new(RefCell::new(Node {
            value: 10,
            children: vec![],
        }));
        let leaf2 = Rc::new(RefCell::new(Node {
            value: 20,
            children: vec![],
        }));
        let root = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![leaf1, leaf2],
        }));

        let graph = reify_graph(root.clone(), children_of);
        let rebuilt = reflect_graph(graph, set_children_of);

        assert_eq!(rebuilt.borrow().value, 0);
        assert_eq!(rebuilt.borrow().children.len(), 2);

        let mut child_values: Vec<i32> = rebuilt
            .borrow()
            .children
            .iter()
            .map(|c| c.borrow().value)
            .collect();
        child_values.sort();
        assert_eq!(child_values, vec![10, 20]);
    }

    #[test]
    fn round_trip_preserves_sharing() {
        // shared node should be the same Rc in the reconstruction
        let shared = Rc::new(RefCell::new(Node {
            value: 42,
            children: vec![],
        }));
        let root = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![shared.clone(), shared.clone()],
        }));

        let graph = reify_graph(root, children_of);
        let rebuilt = reflect_graph(graph, set_children_of);

        let children = &rebuilt.borrow().children;
        assert_eq!(children.len(), 2);
        // Both children should point to the same Rc
        assert_eq!(
            Rc::as_ptr(&children[0]) as usize,
            Rc::as_ptr(&children[1]) as usize
        );
    }
}
