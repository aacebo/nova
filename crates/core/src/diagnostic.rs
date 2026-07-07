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
        self.child_of(Severity::Info, message)
    }

    pub fn warn(self, message: impl Into<String>) -> Self {
        self.child_of(Severity::Warn, message)
    }

    pub fn error(self, message: impl Into<String>) -> Self {
        self.child_of(Severity::Error, message)
    }

    pub fn child(mut self, value: impl Into<Self>) -> Self {
        self.children.push(value.into());
        self
    }

    pub fn child_of(mut self, severity: Severity, message: impl Into<String>) -> Self {
        let child = Self::new(self.trace_id).sev(severity).message(message);
        self.children.push(child);
        self
    }

    pub fn emit(self) {
        crate::scope().emit(self);
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
/// thread it into the diagnostics they build. Implemented for `Scope` and `Ulid`.
pub trait Traced {
    fn trace_id(&self) -> ulid::Ulid;
}

impl Traced for ulid::Ulid {
    fn trace_id(&self) -> ulid::Ulid {
        *self
    }
}

/// Build a `Diagnostic` of the given severity, threading the ambient invocation's
/// `trace_id` implicitly. Children nest via `; [ ... ]`. The result is a `Diagnostic`
/// value — call [`Diagnostic::emit`] to push it onto the current scope:
///
/// ```ignore
/// warn!("rate {} exceeded", limit).emit();
/// info!("outer" ; [
///     info!("detail one"),
///     error!("detail {}", 2),
/// ]).emit();
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
