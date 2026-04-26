//! Demonstrates tracing an async workflow, extracting the step graph,
//! and rendering it as DOT.
//!
//! Run with: `cargo run -p async-reify --example trace_workflow`

use async_reify::{labeled_await, reify_execution, to_dot, Trace};

async fn simulate_work() {
    tokio::task::yield_now().await;
}

#[tokio::main]
async fn main() {
    let trace = Trace::shared();

    // Step 1: fetch
    labeled_await!(simulate_work(), trace).await;

    // Step 2: transform
    labeled_await!(simulate_work(), trace).await;

    // Step 3: store
    labeled_await!(async { 42 }, trace).await;

    let trace = Arc::try_unwrap(trace)
        .expect("single owner")
        .into_inner()
        .expect("not poisoned");

    println!("Collected {} poll events", trace.events.len());

    let graph = reify_execution(trace.events);
    println!(
        "Step graph: {} steps, {} edges",
        graph.steps.len(),
        graph.edges.len()
    );

    for step in &graph.steps {
        println!(
            "  Step {}: {} ({:?}, {}us)",
            step.id, step.label, step.outcome, step.duration_us
        );
    }

    let dot = to_dot(&graph);
    println!("\nDOT output:\n{dot}");

    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string_pretty(&graph).unwrap();
        println!("\nJSON:\n{json}");
    }
}

use std::sync::Arc;
