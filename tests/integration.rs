//! Cross-crate integration tests for reify-reflect.
#![allow(dead_code)]

use reify_reflect::core::{Reflect, RuntimeValue};
use reify_reflect::nat::{False, HCons, HNil, True, N3, N5, S, Z};

// --- Phase 1: Derive + type-level naturals ---

#[derive(reflect_derive::Reflect)]
struct Point {
    x: S<S<Z>>,
    y: N3,
}

#[derive(reflect_derive::Reflect)]
struct Wrapper {
    point: Point,
    flag: Z,
}

#[test]
fn derive_reflect_with_type_level_naturals() {
    let reflected = Point::reflect();
    match &reflected {
        RuntimeValue::List(fields) => {
            assert_eq!(fields.len(), 2);
            // x = 2
            if let RuntimeValue::List(pair) = &fields[0] {
                assert_eq!(pair[1], RuntimeValue::Nat(2));
            } else {
                panic!("expected list");
            }
            // y = 3
            if let RuntimeValue::List(pair) = &fields[1] {
                assert_eq!(pair[1], RuntimeValue::Nat(3));
            } else {
                panic!("expected list");
            }
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn derive_reflect_nested() {
    let reflected = Wrapper::reflect();
    match &reflected {
        RuntimeValue::List(fields) => {
            assert_eq!(fields.len(), 2);
            // point is itself a List of fields
            if let RuntimeValue::List(pair) = &fields[0] {
                assert!(matches!(pair[1], RuntimeValue::List(_)));
            }
            // flag = 0
            if let RuntimeValue::List(pair) = &fields[1] {
                assert_eq!(pair[1], RuntimeValue::Nat(0));
            }
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn type_level_hlist_reflect() {
    type MyList = HCons<N5, HCons<N3, HCons<Z, HNil>>>;
    let reflected = MyList::reflect();
    assert_eq!(
        reflected,
        vec![
            RuntimeValue::Nat(5),
            RuntimeValue::Nat(3),
            RuntimeValue::Nat(0),
        ]
    );
}

#[test]
fn type_level_booleans_reflect() {
    assert!(True::reflect());
    assert!(!False::reflect());
}

// --- Phase 2: Graph reification via JSON round-trip ---

#[cfg(feature = "serde")]
mod graph_serde {
    use reify_reflect::graph::{reflect_graph, reify_graph, ReifiedGraph};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct Node {
        value: i32,
        #[serde(skip)]
        children: Vec<Rc<RefCell<Node>>>,
    }

    #[test]
    fn graph_json_round_trip() {
        let shared = Rc::new(RefCell::new(Node {
            value: 99,
            children: vec![],
        }));
        let a = Rc::new(RefCell::new(Node {
            value: 1,
            children: vec![shared.clone()],
        }));
        let root = Rc::new(RefCell::new(Node {
            value: 0,
            children: vec![a, shared],
        }));

        let graph = reify_graph(root, |n| n.children.clone());
        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: ReifiedGraph<Node> = serde_json::from_str(&json).unwrap();
        let rebuilt = reflect_graph(deserialized, |n, kids| n.children = kids);

        assert_eq!(rebuilt.borrow().value, 0);
        assert_eq!(rebuilt.borrow().children.len(), 2);

        // shared node is the same Rc
        let c0_child = rebuilt.borrow().children[0].borrow().children[0].clone();
        let c1 = rebuilt.borrow().children[1].clone();
        assert_eq!(Rc::as_ptr(&c0_child) as usize, Rc::as_ptr(&c1) as usize);
    }
}

// --- Phase 3: BTreeSet with custom Ord via with_ord! ---

mod context_ord {
    use reify_reflect::context::{with_ord, OrdContext, WithContext};
    use std::collections::BTreeSet;

    #[test]
    fn btreeset_with_custom_comparator() {
        #[derive(Clone, Debug)]
        struct Item {
            score: i32,
            name: String,
        }

        let items = [
            Item {
                score: 3,
                name: "c".into(),
            },
            Item {
                score: 1,
                name: "a".into(),
            },
            Item {
                score: 2,
                name: "b".into(),
            },
            Item {
                score: 1,
                name: "d".into(),
            }, // duplicate score
        ];

        with_ord!(
            items,
            |a: &Item, b: &Item| a.score.cmp(&b.score),
            |wrapped: &[WithContext<Item, OrdContext<Item>>]| {
                let set: BTreeSet<_> = wrapped.iter().cloned().collect();
                // Deduplicates by score, so 3 unique scores
                assert_eq!(set.len(), 3);

                let scores: Vec<i32> = set.into_iter().map(|w| w.inner.score).collect();
                assert_eq!(scores, vec![1, 2, 3]);
            }
        );
    }

    #[test]
    fn sort_with_custom_comparator() {
        let items = [5i32, 2, 8, 1, 9];
        with_ord!(
            items,
            |a: &i32, b: &i32| b.cmp(a), // reverse
            |wrapped: &[WithContext<i32, OrdContext<i32>>]| {
                let mut sorted = wrapped.to_vec();
                sorted.sort();
                let values: Vec<i32> = sorted.into_iter().map(|w| w.inner).collect();
                assert_eq!(values, vec![9, 8, 5, 2, 1]);
            }
        );
    }
}

// --- Phase 4: Async tracing ---

mod async_trace {
    use reify_reflect::async_trace::{labeled_await, reify_execution, to_dot, Trace};
    use std::sync::Arc;

    #[tokio::test]
    async fn traced_workflow_step_graph() {
        let trace = Trace::shared();

        labeled_await!(async { 1 }, trace).await;
        labeled_await!(async { 2 }, trace).await;
        labeled_await!(async { 3 }, trace).await;

        let trace = Arc::try_unwrap(trace).ok().unwrap().into_inner().unwrap();

        let graph = reify_execution(trace.events);
        assert_eq!(graph.steps.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        let dot = to_dot(&graph);
        assert!(dot.contains("digraph"));
        assert!(dot.contains("n0 -> n1"));
        assert!(dot.contains("n1 -> n2"));
    }

    #[cfg(feature = "serde")]
    #[tokio::test]
    async fn step_graph_serialization() {
        let trace = Trace::shared();

        labeled_await!(async { "a" }, trace).await;
        labeled_await!(async { "b" }, trace).await;

        let trace = Arc::try_unwrap(trace).ok().unwrap().into_inner().unwrap();

        let graph = reify_execution(trace.events);
        let json = serde_json::to_string_pretty(&graph).unwrap();
        assert!(json.contains("steps"));
        assert!(json.contains("edges"));

        let parsed: async_reify::AsyncStepGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.steps.len(), graph.steps.len());
    }

    #[cfg(feature = "serde")]
    #[tokio::test]
    async fn raw_trace_serialization() {
        let trace = Trace::shared();
        labeled_await!(async { 1 }, trace).await;
        labeled_await!(async { 2 }, trace).await;
        let trace = Arc::try_unwrap(trace).ok().unwrap().into_inner().unwrap();

        let json = serde_json::to_string(&trace).unwrap();
        let restored: Trace = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.events.len(), trace.events.len());
        for (a, b) in trace.events.iter().zip(restored.events.iter()) {
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.result, b.result);
            assert_eq!(a.label, b.label);
        }
    }
}

// --- Phase 5: Const-reify ---

#[cfg(feature = "const-reify")]
mod const_bridge {
    use reify_reflect::const_bridge::reify_const;

    #[test]
    fn reify_and_use_const() {
        for v in [0u64, 1, 42, 100, 255] {
            let result = reify_const(v, |m| m.modulus());
            assert_eq!(result, v);
        }
    }

    #[test]
    fn reify_computation() {
        let result = reify_const(10, |m| {
            let n = m.modulus();
            n * n + 1
        });
        assert_eq!(result, 101);
    }
}
