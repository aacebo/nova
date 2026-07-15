#[cfg(feature = "ai")]
#[doc(inline)]
pub use nova_ai as ai;
#[cfg(feature = "codec")]
#[doc(inline)]
pub use nova_codec as codec;
pub use nova_core::*;
#[cfg(feature = "fs")]
#[doc(inline)]
pub use nova_fs as fs;
#[cfg(feature = "http")]
#[doc(inline)]
pub use nova_http as http;
#[cfg(feature = "macros")]
#[doc(inline)]
pub use nova_macros::*;
#[doc(inline)]
pub use nova_reflect as reflect;
#[doc(inline)]
pub use nova_runtime::*;
#[cfg(feature = "schema")]
#[doc(inline)]
pub use nova_schema as schema;

#[cfg(feature = "ai")]
pub trait AI {
    fn ai(self) -> Self;
}

#[cfg(feature = "ai")]
impl AI for Builder {
    fn ai(self) -> Self {
        self.var("ai", Binding::namespace(nova_ai::Ai))
    }
}

#[cfg(feature = "fs")]
pub trait FileSystem {
    fn fs(self) -> Self;
}

#[cfg(feature = "fs")]
impl FileSystem for Builder {
    fn fs(self) -> Self {
        self.var("fs", Binding::namespace(nova_fs::Fs))
    }
}

#[cfg(feature = "http")]
pub trait Http {
    fn http(self) -> Self;
}

#[cfg(feature = "http")]
impl Http for Builder {
    fn http(self) -> Self {
        self.var("http", Binding::namespace(nova_http::Client))
    }
}

#[cfg(feature = "codec")]
pub trait Codec {
    fn json(self) -> Self;
    fn yaml(self) -> Self;
}

#[cfg(feature = "codec")]
impl Codec for Builder {
    fn json(self) -> Self {
        self.var("json", Binding::namespace(nova_codec::Json))
    }

    fn yaml(self) -> Self {
        self.var("yaml", Binding::namespace(nova_codec::Yaml))
    }
}
