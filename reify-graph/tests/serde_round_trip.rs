#![cfg(feature = "serde")]

use reify_graph::{reflect_graph, reify_graph, ReifiedGraph};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Node {
    value: i32,
    #[serde(skip)]
    children: Vec<Rc<RefCell<Node>>>,
}

#[test]
fn serde_json_round_trip() {
    let leaf = Rc::new(RefCell::new(Node {
        value: 10,
        children: vec![],
    }));
    let root = Rc::new(RefCell::new(Node {
        value: 0,
        children: vec![leaf.clone(), leaf],
    }));

    // Reify
    let graph: ReifiedGraph<Node> = reify_graph(root, |n| n.children.clone());
    assert_eq!(graph.nodes.len(), 2);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&graph).unwrap();
    assert!(!json.is_empty());

    // Deserialize
    let deserialized: ReifiedGraph<Node> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.nodes.len(), 2);

    // Reconstruct
    let rebuilt = reflect_graph(deserialized, |n, kids| n.children = kids);
    assert_eq!(rebuilt.borrow().value, 0);
    assert_eq!(rebuilt.borrow().children.len(), 2);

    // Both children point to the same reconstructed node
    assert_eq!(
        Rc::as_ptr(&rebuilt.borrow().children[0]) as usize,
        Rc::as_ptr(&rebuilt.borrow().children[1]) as usize
    );
    assert_eq!(rebuilt.borrow().children[0].borrow().value, 10);
}
