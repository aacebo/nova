use std::cell::RefCell;

use crate::Scope;

thread_local! {
    static SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
}

pub fn scope() -> Scope {
    SCOPE
        .with(|scope| scope.borrow().clone())
        .expect("scope macro used outside an invocation")
}

pub fn current_trace_id() -> ulid::Ulid {
    match SCOPE.with(|scope| scope.borrow().as_ref().map(|s| *s.trace_id())) {
        Some(id) => id,
        None => ulid::Ulid::new(),
    }
}

pub fn enter(scope: &Scope) -> Guard {
    let previous = SCOPE.with(|current| current.replace(Some(scope.clone())));
    Guard { previous }
}

pub struct Guard {
    previous: Option<Scope>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        SCOPE.with(|current| *current.borrow_mut() = self.previous.take());
    }
}

#[macro_export]
macro_rules! call {
    ($name:expr $(,)? => $ty:ty) => {
        $crate::__call_coerce!($crate::scope().call($name, $crate::Args::new())?, $ty)
    };
    ($name:expr, $args:expr $(,)? => $ty:ty) => {
        $crate::__call_coerce!($crate::scope().call($name, $args)?, $ty)
    };
    ($name:expr $(,)?) => {
        $crate::scope().call($name, $crate::Args::new())?
    };
    ($name:expr, $args:expr $(,)?) => {
        $crate::scope().call($name, $args)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __call_coerce {
    ($value:expr, $ty:ty) => {
        match $value {
            ::std::option::Option::Some(v) => {
                ::std::option::Option::Some(<$ty as ::std::convert::TryFrom<$crate::Value>>::try_from(v)?)
            }
            ::std::option::Option::None => ::std::option::Option::None,
        }
    };
}

#[macro_export]
macro_rules! get {
    ($key:expr $(,)?) => {
        $crate::scope().get($key)
    };
}

#[macro_export]
macro_rules! get_mut {
    ($key:expr $(,)?) => {
        $crate::scope().get_mut($key)
    };
}

#[macro_export]
macro_rules! set {
    ($key:expr, $obj:expr $(,)?) => {
        $crate::scope().set($key, $obj)
    };
}

#[macro_export]
macro_rules! var {
    ($name:expr, $value:expr $(,)?) => {{
        let __name: ::std::string::String = ::std::convert::Into::into($name);
        $crate::scope().set(__name.clone(), $crate::Var::new(__name, $value))
    }};
}

#[macro_export]
macro_rules! func {
    ($name:expr, $func:expr $(,)?) => {{
        let __name: ::std::string::String = ::std::convert::Into::into($name);
        $crate::scope().set(__name.clone(), $crate::Object::func(__name, $func))
    }};
}

#[macro_export]
macro_rules! has {
    ($key:expr $(,)?) => {
        $crate::scope().has($key)
    };
}

#[macro_export]
macro_rules! del {
    ($key:expr $(,)?) => {
        $crate::scope().del($key)
    };
}
