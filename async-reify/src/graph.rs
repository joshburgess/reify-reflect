//! Step graph extraction and DOT rendering.

use crate::traced::{PollEvent, PollResult};

/// The outcome of an async step.
///
/// # Examples
///
/// ```
/// use async_reify::StepOutcome;
///
/// let completed = StepOutcome::Completed;
/// let pending = StepOutcome::Pending;
/// assert_ne!(completed, pending);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StepOutcome {
    /// The step completed successfully.
    Completed,
    /// The step is still pending.
    Pending,
    /// The step was cancelled.
    Cancelled,
}

/// A node in the async step graph.
///
/// # Examples
///
/// ```
/// use async_reify::{StepNode, StepOutcome};
///
/// let node = StepNode {
///     id: 0,
///     label: "fetch_data".to_string(),
///     duration_us: 150,
///     outcome: StepOutcome::Completed,
/// };
/// assert_eq!(node.id, 0);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StepNode {
    /// Unique step identifier.
    pub id: usize,
    /// Human-readable label for this step.
    pub label: String,
    /// Duration of this step in microseconds.
    pub duration_us: u64,
    /// How this step concluded.
    pub outcome: StepOutcome,
}

/// An extracted async step graph.
///
/// Steps are connected by sequential edges representing execution order.
///
/// # Examples
///
/// ```
/// use async_reify::{AsyncStepGraph, StepNode, StepOutcome};
///
/// let graph = AsyncStepGraph {
///     steps: vec![
///         StepNode { id: 0, label: "start".into(), duration_us: 100, outcome: StepOutcome::Completed },
///         StepNode { id: 1, label: "end".into(), duration_us: 50, outcome: StepOutcome::Completed },
///     ],
///     edges: vec![(0, 1)],
/// };
/// assert_eq!(graph.steps.len(), 2);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AsyncStepGraph {
    /// The steps (nodes) in the graph.
    pub steps: Vec<StepNode>,
    /// Directed edges between steps (sequential and branch).
    pub edges: Vec<(usize, usize)>,
}

/// Extract an [`AsyncStepGraph`] from a sequence of [`PollEvent`]s.
///
/// Groups consecutive events by label to form steps, and connects
/// them with sequential edges.
///
/// # Examples
///
/// ```
/// use async_reify::{PollEvent, PollResult, StepOutcome};
/// use async_reify::reify_execution;
/// use std::time::Instant;
///
/// let now = Instant::now();
/// let events = vec![
///     PollEvent { step: 0, timestamp: now, result: PollResult::Pending, label: Some("a".into()) },
///     PollEvent { step: 1, timestamp: now, result: PollResult::Ready, label: Some("a".into()) },
///     PollEvent { step: 2, timestamp: now, result: PollResult::Ready, label: Some("b".into()) },
/// ];
/// let graph = reify_execution(events);
/// assert_eq!(graph.steps.len(), 2); // "a" and "b"
/// assert_eq!(graph.edges.len(), 1); // a -> b
/// ```
pub fn reify_execution(events: Vec<PollEvent>) -> AsyncStepGraph {
    if events.is_empty() {
        return AsyncStepGraph {
            steps: vec![],
            edges: vec![],
        };
    }

    let mut steps = Vec::new();
    let mut edges = Vec::new();

    // Group consecutive events by label
    let mut current_label = events[0].label.clone();
    let mut group_start = &events[0];
    let mut group_last_result = &events[0].result;

    for (i, event) in events.iter().enumerate().skip(1) {
        if event.label != current_label {
            // Finish previous group
            let duration = event
                .timestamp
                .duration_since(group_start.timestamp)
                .as_micros() as u64;
            let outcome = match group_last_result {
                PollResult::Ready => StepOutcome::Completed,
                PollResult::Pending => StepOutcome::Pending,
            };
            let step_id = steps.len();
            steps.push(StepNode {
                id: step_id,
                label: current_label
                    .clone()
                    .unwrap_or_else(|| format!("step_{step_id}")),
                duration_us: duration,
                outcome,
            });

            // Edge from previous step to this one
            if step_id > 0 {
                edges.push((step_id - 1, step_id));
            }

            current_label = event.label.clone();
            group_start = event;
        }
        group_last_result = &event.result;

        // Last event — close final group
        if i == events.len() - 1 {
            let duration = event
                .timestamp
                .duration_since(group_start.timestamp)
                .as_micros() as u64;
            let outcome = match &event.result {
                PollResult::Ready => StepOutcome::Completed,
                PollResult::Pending => StepOutcome::Pending,
            };
            let step_id = steps.len();
            steps.push(StepNode {
                id: step_id,
                label: current_label
                    .clone()
                    .unwrap_or_else(|| format!("step_{step_id}")),
                duration_us: duration,
                outcome,
            });

            if step_id > 0 {
                edges.push((step_id - 1, step_id));
            }
        }
    }

    // Handle single-event trace
    if events.len() == 1 {
        let outcome = match &events[0].result {
            PollResult::Ready => StepOutcome::Completed,
            PollResult::Pending => StepOutcome::Pending,
        };
        steps.push(StepNode {
            id: 0,
            label: current_label.unwrap_or_else(|| "step_0".to_string()),
            duration_us: 0,
            outcome,
        });
    }

    AsyncStepGraph { steps, edges }
}

/// Render an [`AsyncStepGraph`] as a Graphviz DOT string.
///
/// # Examples
///
/// ```
/// use async_reify::{AsyncStepGraph, StepNode, StepOutcome, to_dot};
///
/// let graph = AsyncStepGraph {
///     steps: vec![
///         StepNode { id: 0, label: "start".into(), duration_us: 100, outcome: StepOutcome::Completed },
///         StepNode { id: 1, label: "end".into(), duration_us: 50, outcome: StepOutcome::Completed },
///     ],
///     edges: vec![(0, 1)],
/// };
/// let dot = to_dot(&graph);
/// assert!(dot.contains("digraph"));
/// assert!(dot.contains("start"));
/// assert!(dot.contains("end"));
/// ```
pub fn to_dot(graph: &AsyncStepGraph) -> String {
    let mut out = String::from("digraph async_trace {\n    rankdir=TB;\n    node [shape=box];\n\n");

    for step in &graph.steps {
        let color = match step.outcome {
            StepOutcome::Completed => "green",
            StepOutcome::Pending => "yellow",
            StepOutcome::Cancelled => "red",
        };
        out.push_str(&format!(
            "    n{} [label=\"{}\\n({}us)\" style=filled fillcolor={}];\n",
            step.id, step.label, step.duration_us, color
        ));
    }

    out.push('\n');

    for (from, to) in &graph.edges {
        out.push_str(&format!("    n{from} -> n{to};\n"));
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_event(step: usize, result: PollResult, label: Option<&str>) -> PollEvent {
        PollEvent {
            step,
            timestamp: Instant::now(),
            result,
            label: label.map(String::from),
        }
    }

    #[test]
    fn empty_trace() {
        let graph = reify_execution(vec![]);
        assert!(graph.steps.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn single_event() {
        let graph = reify_execution(vec![make_event(0, PollResult::Ready, Some("only"))]);
        assert_eq!(graph.steps.len(), 1);
        assert_eq!(graph.steps[0].label, "only");
        assert_eq!(graph.steps[0].outcome, StepOutcome::Completed);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn two_steps() {
        let graph = reify_execution(vec![
            make_event(0, PollResult::Pending, Some("a")),
            make_event(1, PollResult::Ready, Some("a")),
            make_event(2, PollResult::Ready, Some("b")),
        ]);
        assert_eq!(graph.steps.len(), 2);
        assert_eq!(graph.steps[0].label, "a");
        assert_eq!(graph.steps[0].outcome, StepOutcome::Completed);
        assert_eq!(graph.steps[1].label, "b");
        assert_eq!(graph.edges, vec![(0, 1)]);
    }

    #[test]
    fn three_steps_chain() {
        let graph = reify_execution(vec![
            make_event(0, PollResult::Ready, Some("x")),
            make_event(1, PollResult::Pending, Some("y")),
            make_event(2, PollResult::Ready, Some("y")),
            make_event(3, PollResult::Ready, Some("z")),
        ]);
        assert_eq!(graph.steps.len(), 3);
        assert_eq!(graph.edges, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn unlabeled_steps() {
        let graph = reify_execution(vec![
            make_event(0, PollResult::Ready, None),
            make_event(1, PollResult::Ready, Some("b")),
        ]);
        assert_eq!(graph.steps.len(), 2);
        assert_eq!(graph.steps[0].label, "step_0");
        assert_eq!(graph.steps[1].label, "b");
    }

    #[test]
    fn dot_output() {
        let graph = AsyncStepGraph {
            steps: vec![
                StepNode {
                    id: 0,
                    label: "start".into(),
                    duration_us: 100,
                    outcome: StepOutcome::Completed,
                },
                StepNode {
                    id: 1,
                    label: "end".into(),
                    duration_us: 50,
                    outcome: StepOutcome::Pending,
                },
            ],
            edges: vec![(0, 1)],
        };
        let dot = to_dot(&graph);
        assert!(dot.contains("digraph async_trace"));
        assert!(dot.contains("start"));
        assert!(dot.contains("end"));
        assert!(dot.contains("n0 -> n1"));
        assert!(dot.contains("green")); // completed
        assert!(dot.contains("yellow")); // pending
    }
}
