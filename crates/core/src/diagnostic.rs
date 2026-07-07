#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub id: ulid::Ulid,
    pub trace_id: ulid::Ulid,
    pub severity: Option<Severity>,
    pub message: Option<String>,
    pub children: Vec<Self>,
}

impl Diagnostic {
    pub fn new(trace_id: ulid::Ulid) -> Self {
        Self {
            id: ulid::Ulid::new(),
            trace_id,
            severity: None,
            message: None,
            children: vec![],
        }
    }

    pub fn sev(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn info(self, message: impl Into<String>) -> Self {
        self.emit(Severity::Info, message)
    }

    pub fn warn(self, message: impl Into<String>) -> Self {
        self.emit(Severity::Warn, message)
    }

    pub fn error(self, message: impl Into<String>) -> Self {
        self.emit(Severity::Error, message)
    }

    pub fn child(mut self, value: impl Into<Self>) -> Self {
        self.children.push(value.into());
        self
    }

    pub fn emit(mut self, severity: Severity, message: impl Into<String>) -> Self {
        let child = Self::new(self.trace_id).sev(severity).message(message);
        self.children.push(child);
        self
    }

    pub fn severity(&self) -> Severity {
        let own = self.severity.unwrap_or(Severity::Info);
        self.children
            .iter()
            .map(|child| child.severity())
            .chain([own])
            .max()
            .unwrap_or(own)
    }
}

/// Anything that carries a `trace_id`, so the `info!`/`warn!`/`error!` macros can
/// thread it into the diagnostics they build. Implemented for `Context` and `Scope`.
pub trait Traced {
    fn trace_id(&self) -> ulid::Ulid;
}

impl Traced for ulid::Ulid {
    fn trace_id(&self) -> ulid::Ulid {
        *self
    }
}

/// Build a `Diagnostic` of the given severity. The `trace_id` is read implicitly from
/// the currently-executing invocation (see [`current_trace_id`]), so no context or id
/// is passed at the call site:
///
/// ```ignore
/// warn!("rate {} exceeded", limit);       // format-string message
/// info!("outer" ; [                       // nested children via `; [ ]`
///     info!("detail one"),
///     error!("detail {}", 2),
/// ]);
/// ```
#[macro_export]
macro_rules! diagnostic {
    ($sev:expr, $($fmt:tt)*) => {
        $crate::__diagnostic_impl!($sev, $($fmt)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __diagnostic_impl {
    ($sev:expr, $fmt:literal $(, $arg:expr)* $(,)? ; [ $($child:expr),* $(,)? ]) => {{
        let mut __d = $crate::Diagnostic::new($crate::current_trace_id())
            .sev($sev)
            .message(::std::format!($fmt $(, $arg)*));
        $( __d = __d.child($child); )*
        __d
    }};
    ($sev:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::Diagnostic::new($crate::current_trace_id())
            .sev($sev)
            .message(::std::format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($rest:tt)*) => { $crate::__diagnostic_impl!($crate::Severity::Info, $($rest)*) };
}

#[macro_export]
macro_rules! warn {
    ($($rest:tt)*) => { $crate::__diagnostic_impl!($crate::Severity::Warn, $($rest)*) };
}

#[macro_export]
macro_rules! error {
    ($($rest:tt)*) => { $crate::__diagnostic_impl!($crate::Severity::Error, $($rest)*) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_builds_message_with_format_args() {
        let trace_id = ulid::Ulid::new();
        let _guard = crate::enter_trace(trace_id);
        let d = warn!("rate {} of {}", 9, 10);

        assert_eq!(d.trace_id, trace_id);
        assert_eq!(d.severity, Some(Severity::Warn));
        assert_eq!(d.message.as_deref(), Some("rate 9 of 10"));
        assert!(d.children.is_empty());
    }

    #[test]
    fn macro_nests_children_and_rolls_up_severity() {
        let trace_id = ulid::Ulid::new();
        let _guard = crate::enter_trace(trace_id);
        let d = info!("outer" ; [
            info!("child one"),
            error!("child {}", 2),
        ]);

        assert_eq!(d.trace_id, trace_id);
        assert_eq!(d.severity, Some(Severity::Info));
        assert_eq!(d.children.len(), 2);
        assert_eq!(d.children[1].message.as_deref(), Some("child 2"));
        assert_eq!(d.severity(), Severity::Error);
    }

    #[test]
    fn macro_outside_invocation_gets_fresh_id() {
        let d = warn!("orphan");
        assert_ne!(d.trace_id, ulid::Ulid::nil());
    }

    #[test]
    fn child_builders_append_and_inherit_trace_id() {
        let trace_id = ulid::Ulid::new();
        let d = Diagnostic::new(trace_id).warn("a").error("b");

        assert_eq!(d.children.len(), 2);
        assert_eq!(d.children[0].severity, Some(Severity::Warn));
        assert_eq!(d.children[0].message.as_deref(), Some("a"));
        assert_eq!(d.children[1].trace_id, trace_id);
        assert_eq!(d.severity(), Severity::Error);
    }
}
