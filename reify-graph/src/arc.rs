//! Arc-pointer graph reification, parallel to the [`Rc<RefCell<T>>`](std::rc::Rc) API.
//!
//! Identical semantics to the top-level [`crate::reify_graph`] /
//! [`crate::reflect_graph`] pair, but for [`Arc<Mutex<T>>`](std::sync::Arc)
//! graphs. [`NodeId`] is shared between the two APIs.

use crate::NodeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Extract a [`NodeId`] from an `Arc<Mutex<T>>` using pointer identity.
///
/// # Examples
///
/// ```
/// use reify_graph::arc::node_id_of_arc;
/// use std::sync::{Arc, Mutex};
///
/// let a = Arc::new(Mutex::new(42));
/// let b = a.clone();
/// assert_eq!(node_id_of_arc(&a), node_id_of_arc(&b));
/// ```
pub fn node_id_of_arc<T>(arc: &Arc<Mutex<T>>) -> NodeId {
    NodeId(Arc::as_ptr(arc) as *const () as usize)
}

/// Collect all reachable nodes from `root`, detecting shared pointers.
///
/// # Examples
///
/// ```
/// use reify_graph::arc::collect_nodes_arc;
/// use std::sync::{Arc, Mutex};
///
/// #[derive(Clone)]
/// struct Node { children: Vec<Arc<Mutex<Node>>> }
///
/// let leaf = Arc::new(Mutex::new(Node { children: vec![] }));
/// let root = Arc::new(Mutex::new(Node { children: vec![leaf.clone()] }));
///
/// let nodes = collect_nodes_arc(&root, &|n: &Node| n.children.clone());
/// assert_eq!(nodes.len(), 2);
/// ```
pub fn collect_nodes_arc<T, F>(root: &Arc<Mutex<T>>, children: &F) -> HashMap<NodeId, Arc<Mutex<T>>>
where
    F: Fn(&T) -> Vec<Arc<Mutex<T>>>,
{
    let mut visited = HashMap::new();
    collect_inner(root, children, &mut visited);
    visited
}

fn collect_inner<T, F>(
    node: &Arc<Mutex<T>>,
    children: &F,
    visited: &mut HashMap<NodeId, Arc<Mutex<T>>>,
) where
    F: Fn(&T) -> Vec<Arc<Mutex<T>>>,
{
    let id = node_id_of_arc(node);
    if visited.contains_key(&id) {
        return;
    }
    visited.insert(id, node.clone());

    let kids = children(&node.lock().expect("mutex should not be poisoned"));
    for kid in &kids {
        collect_inner(kid, children, visited);
    }
}

/// Reify an `Arc<Mutex<T>>` graph into a [`crate::ReifiedGraph`].
///
/// # Examples
///
/// ```
/// use reify_graph::arc::reify_graph_arc;
/// use std::sync::{Arc, Mutex};
///
/// #[derive(Clone, Debug)]
/// struct Node {
///     value: i32,
///     children: Vec<Arc<Mutex<Node>>>,
/// }
///
/// let leaf = Arc::new(Mutex::new(Node { value: 1, children: vec![] }));
/// let root = Arc::new(Mutex::new(Node {
///     value: 0,
///     children: vec![leaf.clone()],
/// }));
///
/// let graph = reify_graph_arc(root, |n| n.children.clone());
/// assert_eq!(graph.nodes.len(), 2);
/// assert_eq!(graph.edges.len(), 1);
/// ```
pub fn reify_graph_arc<T, F>(root: Arc<Mutex<T>>, children: F) -> crate::ReifiedGraph<T>
where
    F: Fn(&T) -> Vec<Arc<Mutex<T>>>,
    T: Clone,
{
    let all_nodes = collect_nodes_arc(&root, &children);
    let root_id = node_id_of_arc(&root);

    let mut nodes = Vec::with_capacity(all_nodes.len());
    let mut edges = Vec::new();

    for (id, arc) in &all_nodes {
        let guard = arc.lock().expect("mutex should not be poisoned");
        nodes.push((*id, guard.clone()));

        for kid in children(&guard) {
            let kid_id = node_id_of_arc(&kid);
            edges.push((*id, kid_id));
        }
    }

    crate::ReifiedGraph {
        nodes,
        edges,
        root: root_id,
    }
}

/// Reconstruct an `Arc<Mutex<T>>` graph from a [`crate::ReifiedGraph`].
///
/// # Examples
///
/// ```
/// use reify_graph::arc::{reify_graph_arc, reflect_graph_arc};
/// use std::sync::{Arc, Mutex};
///
/// #[derive(Clone, Debug)]
/// struct Node {
///     value: i32,
///     children: Vec<Arc<Mutex<Node>>>,
/// }
///
/// let leaf = Arc::new(Mutex::new(Node { value: 1, children: vec![] }));
/// let root = Arc::new(Mutex::new(Node { value: 0, children: vec![leaf] }));
///
/// let graph = reify_graph_arc(root, |n| n.children.clone());
/// let rebuilt = reflect_graph_arc(graph, |n, kids| n.children = kids);
/// assert_eq!(rebuilt.lock().unwrap().value, 0);
/// ```
pub fn reflect_graph_arc<T, F>(graph: crate::ReifiedGraph<T>, set_children: F) -> Arc<Mutex<T>>
where
    F: Fn(&mut T, Vec<Arc<Mutex<T>>>),
    T: Clone,
{
    let mut arc_map: HashMap<NodeId, Arc<Mutex<T>>> = HashMap::new();
    for (id, data) in &graph.nodes {
        arc_map.insert(*id, Arc::new(Mutex::new(data.clone())));
    }

    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for (from, to) in &graph.edges {
        adj.entry(*from).or_default().push(*to);
    }

    for (id, arc) in &arc_map {
        if let Some(child_ids) = adj.get(id) {
            let kids: Vec<Arc<Mutex<T>>> =
                child_ids.iter().map(|cid| arc_map[cid].clone()).collect();
            set_children(&mut arc.lock().expect("mutex should not be poisoned"), kids);
        }
    }

    arc_map[&graph.root].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct Node {
        #[allow(dead_code)]
        value: i32,
        children: Vec<Arc<Mutex<Node>>>,
    }

    fn children_of(n: &Node) -> Vec<Arc<Mutex<Node>>> {
        n.children.clone()
    }

    fn set_children_of(n: &mut Node, kids: Vec<Arc<Mutex<Node>>>) {
        n.children = kids;
    }

    #[test]
    fn arc_node_id_identity() {
        let a = Arc::new(Mutex::new(Node {
            value: 1,
            children: vec![],
        }));
        let b = a.clone();
        assert_eq!(node_id_of_arc(&a), node_id_of_arc(&b));

        let c = Arc::new(Mutex::new(Node {
            value: 1,
            children: vec![],
        }));
        assert_ne!(node_id_of_arc(&a), node_id_of_arc(&c));
    }

    #[test]
    fn arc_round_trip_preserves_sharing() {
        let shared = Arc::new(Mutex::new(Node {
            value: 42,
            children: vec![],
        }));
        let root = Arc::new(Mutex::new(Node {
            value: 0,
            children: vec![shared.clone(), shared.clone()],
        }));

        let graph = reify_graph_arc(root, children_of);
        let rebuilt = reflect_graph_arc(graph, set_children_of);

        let children = rebuilt.lock().unwrap().children.clone();
        assert_eq!(children.len(), 2);
        assert_eq!(
            Arc::as_ptr(&children[0]) as usize,
            Arc::as_ptr(&children[1]) as usize
        );
    }

    #[test]
    fn arc_cycle_detection() {
        let a = Arc::new(Mutex::new(Node {
            value: 0,
            children: vec![],
        }));
        let b = Arc::new(Mutex::new(Node {
            value: 1,
            children: vec![a.clone()],
        }));
        a.lock().unwrap().children.push(b.clone());

        let nodes = collect_nodes_arc(&a, &children_of);
        assert_eq!(nodes.len(), 2);
    }
}
