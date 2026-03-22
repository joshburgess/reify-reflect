//! Demonstrates serializing a cyclic-capable graph structure to JSON
//! and reconstructing it.
//!
//! Run with: `cargo run -p reify-graph --features serde --example serialize_graph`

#[cfg(feature = "serde")]
fn main() {
    use reify_graph::{reflect_graph, reify_graph};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct Node {
        label: String,
        #[serde(skip)]
        children: Vec<Rc<RefCell<Node>>>,
    }

    // Build a DAG with sharing:
    //   root -> [a, b]
    //   a -> [shared]
    //   b -> [shared]
    let shared = Rc::new(RefCell::new(Node {
        label: "shared".into(),
        children: vec![],
    }));
    let a = Rc::new(RefCell::new(Node {
        label: "a".into(),
        children: vec![shared.clone()],
    }));
    let b = Rc::new(RefCell::new(Node {
        label: "b".into(),
        children: vec![shared.clone()],
    }));
    let root = Rc::new(RefCell::new(Node {
        label: "root".into(),
        children: vec![a, b],
    }));

    // Reify
    let graph = reify_graph(root, |n| n.children.clone());
    println!(
        "Reified graph: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // Serialize
    let json = serde_json::to_string_pretty(&graph).unwrap();
    println!("\nJSON:\n{json}");

    // Deserialize and reconstruct
    let deserialized: reify_graph::ReifiedGraph<Node> = serde_json::from_str(&json).unwrap();
    let rebuilt = reflect_graph(deserialized, |n, kids| n.children = kids);

    println!("\nReconstructed root: {:?}", rebuilt.borrow().label);
    for child in &rebuilt.borrow().children {
        println!("  child: {:?}", child.borrow().label);
        for grandchild in &child.borrow().children {
            println!("    grandchild: {:?}", grandchild.borrow().label);
        }
    }
}

#[cfg(not(feature = "serde"))]
fn main() {
    eprintln!("This example requires the `serde` feature: cargo run -p reify-graph --features serde --example serialize_graph");
}
