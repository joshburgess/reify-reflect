//! Demonstrates tracing an async workflow, extracting the step graph,
//! and rendering it as DOT.
//!
//! Run with: `cargo run -p async-reify --example trace_workflow`

use async_reify::{labeled_await, reify_execution, to_dot, PollEvent};
use std::sync::{Arc, Mutex};

async fn simulate_work() {
    tokio::task::yield_now().await;
}

#[tokio::main]
async fn main() {
    let log = Arc::new(Mutex::new(Vec::<PollEvent>::new()));

    // Step 1: fetch
    labeled_await!(simulate_work(), log).await;

    // Step 2: transform
    labeled_await!(simulate_work(), log).await;

    // Step 3: store
    labeled_await!(async { 42 }, log).await;

    let events = Arc::try_unwrap(log)
        .expect("single owner")
        .into_inner()
        .expect("not poisoned");

    println!("Collected {} poll events", events.len());

    let graph = reify_execution(events);
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

    // Serde round-trip (if feature enabled)
    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string_pretty(&graph).unwrap();
        println!("\nJSON:\n{json}");
    }
}
