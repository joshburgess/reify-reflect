use criterion::{black_box, criterion_group, criterion_main, Criterion};

use reflect_nat::{HCons, HNil, S, Z};
use reify_graph::{reify_graph, ReifiedGraph};
use reify_reflect_core::Reflect;

use std::cell::RefCell;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Benchmark 1: Reflect::reflect() on a 5-level nested HList
// ---------------------------------------------------------------------------

// Build a 5-level nested HList type:
// HCons<S<S<Z>>, HCons<S<Z>, HCons<Z, HCons<S<S<S<Z>>>, HCons<S<S<S<S<Z>>>>, HNil>>>>>
type Level5 = HCons<S<S<Z>>, HCons<S<Z>, HCons<Z, HCons<S<S<S<Z>>>, HCons<S<S<S<S<Z>>>>, HNil>>>>>;

fn bench_hlist_reflect(c: &mut Criterion) {
    c.bench_function("hlist_5_level_reflect", |b| {
        b.iter(|| {
            let val = Level5::reflect();
            black_box(val);
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark 2: reify_graph on a graph of 1,000 nodes
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Node {
    #[allow(dead_code)]
    value: usize,
    children: Vec<Rc<RefCell<Node>>>,
}

fn build_chain(n: usize) -> Rc<RefCell<Node>> {
    let nodes: Vec<Rc<RefCell<Node>>> = (0..n)
        .map(|i| {
            Rc::new(RefCell::new(Node {
                value: i,
                children: vec![],
            }))
        })
        .collect();

    // Wire up: each node points to the next, plus some cross-links for DAG structure
    for i in 0..n - 1 {
        let next = nodes[i + 1].clone();
        nodes[i].borrow_mut().children.push(next);
        // Add a cross-link every 10 nodes for DAG richness
        if i + 10 < n {
            let cross = nodes[i + 10].clone();
            nodes[i].borrow_mut().children.push(cross);
        }
    }

    nodes[0].clone()
}

fn bench_reify_graph_1000(c: &mut Criterion) {
    let root = build_chain(1000);

    c.bench_function("reify_graph_1000_nodes", |b| {
        b.iter(|| {
            let graph: ReifiedGraph<Node> = reify_graph(root.clone(), |n| n.children.clone());
            black_box(graph);
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark 3: with_ord! sort vs plain closure sort on 10,000 elements
// ---------------------------------------------------------------------------

fn bench_sort_comparison(c: &mut Criterion) {
    let data: Vec<i32> = (0..10_000).rev().collect();

    c.bench_function("sort_with_ord_macro_10k", |b| {
        b.iter(|| {
            let ctx = context_trait::OrdContext {
                compare: |a: &i32, b: &i32| a.cmp(b),
            };
            let mut wrapped: Vec<context_trait::WithContext<i32, context_trait::OrdContext<i32>>> =
                data.iter()
                    .map(|&v| context_trait::WithContext { inner: v, ctx })
                    .collect();
            wrapped.sort();
            black_box(wrapped);
        });
    });

    c.bench_function("sort_plain_closure_10k", |b| {
        b.iter(|| {
            let mut v = data.clone();
            #[allow(clippy::unnecessary_sort_by)]
            v.sort_by(|a, b| a.cmp(b));
            black_box(v);
        });
    });
}

criterion_group!(
    benches,
    bench_hlist_reflect,
    bench_reify_graph_1000,
    bench_sort_comparison,
);
criterion_main!(benches);
