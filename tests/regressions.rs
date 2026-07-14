mod common;

use common::Recorder;
use nova::reflect::Value;
use nova::template::{Args, Pointer};
use nova::{Scope, args};

fn recorder_runtime(recorder: &Recorder) -> nova::Builder {
    nova::new().observe(recorder.clone())
}

#[test]
fn scope_args_resolve_in_a_template() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .action("show", |_args: &Args, scope: &Scope| {
            let out = scope.render_str("{{ args[0] }}-{{ args[1] }}-{{ args | length }}")?;
            nova::info!("{}", out).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("show", args!("a", "b")).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "a-b-2"), "{:?}", recorder.messages());
}

#[cfg(feature = "codec")]
#[test]
fn json_arrays_round_trip_as_arrays() {
    use nova::codec::Codec;

    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .json()
        .action("go", |_args: &Args, scope: &Scope| {
            for src in ["[1,2,3]", "{\"a\":{\"b\":[1,2,3]}}", "[[1,2],[3]]", "[{\"x\":1},{\"x\":2}]"] {
                let out = scope.render_str(&format!("{{{{ json.encode(json.decode('{src}')) }}}}"))?;
                nova::info!("{}", out).emit(scope);
            }

            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("go", args!()).unwrap();
    drop(runtime);

    let seen = recorder.messages();
    assert!(seen.iter().any(|m| m == "[1,2,3]"), "{seen:?}");
    assert!(seen.iter().any(|m| m == "{\"a\":{\"b\":[1,2,3]}}"), "{seen:?}");
    assert!(seen.iter().any(|m| m == "[[1,2],[3]]"), "{seen:?}");
    assert!(seen.iter().any(|m| m == "[{\"x\":1},{\"x\":2}]"), "{seen:?}");
}

#[cfg(feature = "codec")]
#[test]
fn decoded_arrays_are_sequences_not_index_keyed_maps() {
    use nova::codec::Codec;

    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .json()
        .action("go", |_args: &Args, scope: &Scope| {
            let out = scope.render_str(
                "{% set xs = json.decode('[10,20,30]') %}{{ xs | length }}:{{ xs[1] }}:{% for x in xs %}{{ x }},{% endfor %}",
            )?;
            nova::info!("{}", out).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("go", args!()).unwrap();
    drop(runtime);

    assert!(
        recorder.messages().iter().any(|m| m == "3:20:10,20,30,"),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn a_list_var_iterates_and_indexes() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .var("xs", vec!["a", "b", "c"])
        .action("go", |_args: &Args, scope: &Scope| {
            let out = scope.render_str("{{ xs | length }}:{{ xs[2] }}")?;
            nova::info!("{}", out).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("go", args!()).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "3:c"), "{:?}", recorder.messages());
}

#[cfg(feature = "fs")]
#[test]
fn fs_write_round_trips_binary_bytes() {
    use nova::fs::FileSystem;

    let dir = std::env::temp_dir().join(format!("nova-bin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("blob.bin");

    let raw: Vec<u8> = vec![0x00, 0xff, 0xfe, 0x80, 0x41];
    let bytes: Vec<Pointer> = raw.iter().map(|b| Pointer::from(*b)).collect();

    let runtime = nova::new()
        .fs()
        .var("path", path.to_string_lossy().to_string())
        .var("blob", bytes)
        .action("go", |_args: &Args, scope: &Scope| {
            scope.render_str("{{ fs.write(path, blob) }}")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("go", args!()).unwrap();
    drop(runtime);

    let written = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(written, raw, "binary bytes must survive fs.write unmangled");
}

#[test]
fn karg_helpers_are_available() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .func("f", |args: &Args, _scope: &Scope| {
            let n: u64 = args.kargs().get_required("n")?;
            let missing: u64 = args.kargs().get_or_default("nope");
            Ok(Pointer::from(Value::from(n + missing)))
        })
        .build()
        .unwrap();

    let out = runtime.func("f", args!(n = 7u64)).unwrap();
    assert_eq!(out, Value::from(7u64));
}

#[derive(Debug, Default)]
struct UpperEngine {
    templates: std::collections::BTreeMap<String, String>,
}

impl nova::template::Engine for UpperEngine {
    fn add_template(&mut self, name: &str, source: &str) -> Result<(), nova::template::Error> {
        self.templates.insert(name.to_string(), source.to_string());
        Ok(())
    }

    fn render(&self, name: &str, ctx: &std::sync::Arc<dyn nova::template::Context>) -> Result<String, nova::template::Error> {
        let src = self
            .templates
            .get(name)
            .ok_or_else(|| nova::template::Error::message(format!("no template '{name}'")))?
            .clone();

        self.render_str(&src, ctx)
    }

    fn render_str(
        &self,
        source: &str,
        ctx: &std::sync::Arc<dyn nova::template::Context>,
    ) -> Result<String, nova::template::Error> {
        let mut out = source.to_string();

        for key in ctx.names() {
            if let Some(v) = ctx.resolve(&key) {
                out = out.replace(&format!("{{{key}}}"), &v.value().to_string());
            }
        }

        Ok(out.to_uppercase())
    }

    fn eval(&self, expr: &str, ctx: &std::sync::Arc<dyn nova::template::Context>) -> Result<Pointer, nova::template::Error> {
        Ok(ctx.resolve(expr).unwrap_or_else(|| Pointer::new(Value::Null)))
    }
}

#[test]
fn a_custom_engine_can_replace_minijinja() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .engine(UpperEngine::default)
        .var("who", "world")
        .action("go", |_args: &Args, scope: &Scope| {
            let out = scope.render_str("hello {who}")?;
            nova::info!("{}", out).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("go", args!()).unwrap();
    drop(runtime);

    assert!(
        recorder.messages().iter().any(|m| m == "HELLO WORLD"),
        "custom engine must be used instead of minijinja: {:?}",
        recorder.messages()
    );
}
