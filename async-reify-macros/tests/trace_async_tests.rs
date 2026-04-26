use async_reify::{PollResult, Trace};
use async_reify_macros::trace_async;
use std::sync::{Arc, Mutex};

async fn fetch() -> i32 {
    42
}

async fn slow() -> i32 {
    tokio::task::yield_now().await;
    7
}

#[trace_async(trace = trace)]
async fn workflow_simple(trace: Arc<Mutex<Trace>>) -> i32 {
    let a = fetch().await;
    let b = fetch().await;
    a + b
}

#[tokio::test]
async fn rewrites_simple_awaits() {
    let trace = Trace::shared();
    let value = workflow_simple(trace.clone()).await;
    assert_eq!(value, 84);

    let snapshot = Arc::try_unwrap(trace).unwrap().into_inner().unwrap();
    // Each `fetch().await` should record at least one Ready event.
    let ready_count = snapshot
        .events
        .iter()
        .filter(|e| matches!(e.result, PollResult::Ready))
        .count();
    assert!(
        ready_count >= 2,
        "expected >=2 Ready events, got {ready_count}: {:?}",
        snapshot.events
    );

    // Labels should include source location info.
    assert!(snapshot.events.iter().all(|e| {
        e.label
            .as_ref()
            .map(|l| l.contains('@') && l.contains(".rs:"))
            .unwrap_or(false)
    }));
}

#[trace_async(trace = trace)]
async fn workflow_pending(trace: Arc<Mutex<Trace>>) -> i32 {
    slow().await + slow().await
}

#[tokio::test]
async fn rewrites_pending_awaits() {
    let trace = Trace::shared();
    let value = workflow_pending(trace.clone()).await;
    assert_eq!(value, 14);

    let snapshot = Arc::try_unwrap(trace).unwrap().into_inner().unwrap();
    let pending_count = snapshot
        .events
        .iter()
        .filter(|e| matches!(e.result, PollResult::Pending))
        .count();
    let ready_count = snapshot
        .events
        .iter()
        .filter(|e| matches!(e.result, PollResult::Ready))
        .count();
    // yield_now produces at least one Pending then Ready per call.
    assert!(pending_count >= 2, "expected pending events");
    assert!(ready_count >= 2, "expected ready events");
}

#[trace_async(trace = trace)]
async fn workflow_chained(trace: Arc<Mutex<Trace>>) -> i32 {
    fetch().await + fetch().await
}

#[tokio::test]
async fn rewrites_chained_method_awaits() {
    let trace = Trace::shared();
    let value = workflow_chained(trace.clone()).await;
    assert_eq!(value, 84);

    let snapshot = Arc::try_unwrap(trace).unwrap().into_inner().unwrap();
    assert!(snapshot.events.len() >= 2);
}

// A nested async block inside the function. Awaits inside a nested closure
// or item should be left alone, but awaits in nested async blocks within
// the same scope are still rewritten (they share the outer trace).
#[trace_async(trace = trace)]
async fn workflow_nested_block(trace: Arc<Mutex<Trace>>) -> i32 {
    let inner = async { fetch().await }.await;
    inner
}

#[tokio::test]
async fn rewrites_awaits_in_nested_async_block() {
    let trace = Trace::shared();
    let value = workflow_nested_block(trace.clone()).await;
    assert_eq!(value, 42);

    let snapshot = Arc::try_unwrap(trace).unwrap().into_inner().unwrap();
    assert!(!snapshot.events.is_empty());
}
