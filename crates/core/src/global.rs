use std::cell::RefCell;

thread_local! {
    /// Stack of the trace_ids for the currently-executing invocations, innermost last.
    /// Pushed on entry to a func/action and popped on exit, so a diagnostic built
    /// anywhere inside sees the trace_id of the invocation it belongs to.
    static TRACE_IDS: RefCell<Vec<ulid::Ulid>> = const { RefCell::new(Vec::new()) };
}

/// The trace_id of the currently-executing invocation, or a fresh `Ulid` if called
/// outside any invocation (so a stray diagnostic still gets a unique id).
pub fn current_trace_id() -> ulid::Ulid {
    match TRACE_IDS.with(|ids| ids.borrow().last().copied()) {
        Some(id) => id,
        None => ulid::Ulid::new(),
    }
}

/// Push `id` as the current trace_id until the returned guard is dropped.
pub fn enter_trace(id: ulid::Ulid) -> TraceGuard {
    TRACE_IDS.with(|ids| ids.borrow_mut().push(id));
    TraceGuard
}

/// Pops the trace_id pushed by [`enter_trace`] on drop, restoring the previous one.
/// Restores even on early return or unwind.
pub struct TraceGuard;

impl Drop for TraceGuard {
    fn drop(&mut self) {
        TRACE_IDS.with(|ids| {
            ids.borrow_mut().pop();
        });
    }
}
