mod common;

use common::Recorder;
use nova::reflect::Value;
use nova::{Binding, Scope, args};

fn recorder_runtime(recorder: &Recorder) -> nova::Builder {
    nova::new().observe(recorder.clone())
}

#[test]
fn scope_args_resolve_in_a_template() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .action("show", |scope: &dyn nova::Context| {
            let out = scope.render("{{ args[0] }}-{{ args[1] }}-{{ args | length }}")?;
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
    use nova::Codec;

    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .json()
        .action("go", |scope: &dyn nova::Context| {
            for src in ["[1,2,3]", "{\"a\":{\"b\":[1,2,3]}}", "[[1,2],[3]]", "[{\"x\":1},{\"x\":2}]"] {
                let out = scope.render(&format!("{{{{ json.encode(json.decode('{src}')) }}}}"))?;
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
    use nova::Codec;

    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .json()
        .action("go", |scope: &dyn nova::Context| {
            let out = scope.render(
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
        .action("go", |scope: &dyn nova::Context| {
            let out = scope.render("{{ xs | length }}:{{ xs[2] }}")?;
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
    use nova::FileSystem;

    let dir = std::env::temp_dir().join(format!("nova-bin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("blob.bin");

    let raw: Vec<u8> = vec![0x00, 0xff, 0xfe, 0x80, 0x41];
    let bytes: Vec<Binding> = raw.iter().map(|b| Binding::from(*b)).collect();

    let runtime = nova::new()
        .fs()
        .var("path", path.to_string_lossy().to_string())
        .var("blob", bytes)
        .action("go", |scope: &dyn nova::Context| {
            scope.render("{{ fs.write(path, blob) }}")?;
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
        .func("f", |scope: &dyn nova::Context| {
            let args = scope.args();
            let n: u64 = args.kargs().get_required("n")?;
            let missing: u64 = args.kargs().get_or_default("nope");
            Ok(Binding::from(Value::from(n + missing)))
        })
        .build()
        .unwrap();

    let out = runtime.func("f", args!(n = 7u64)).unwrap();
    assert_eq!(out, Value::from(7u64));
}

#[derive(Debug, Default)]
struct UpperEngine;

impl nova::TemplateEngine for UpperEngine {
    type Context = Scope;

    fn render(&self, src: &str, ctx: &Scope) -> Result<String, nova::Error> {
        let mut out = src.to_string();

        for (key, value) in ctx.iter() {
            if let Some(key) = key.as_str() {
                out = out.replace(&format!("{{{key}}}"), &value.to_string());
            }
        }

        Ok(out.to_uppercase())
    }

    fn eval(&self, src: &str, ctx: &Scope) -> Result<Value, nova::Error> {
        Ok(nova::Context::get(ctx, src).map(|v| v.into_value()).unwrap_or(Value::Null))
    }
}

#[test]
fn a_custom_engine_can_replace_minijinja() {
    let recorder = Recorder::new();
    let runtime = recorder_runtime(&recorder)
        .engine(UpperEngine)
        .var("who", "world")
        .action("go", |scope: &dyn nova::Context| {
            let out = scope.render("hello {who}")?;
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
