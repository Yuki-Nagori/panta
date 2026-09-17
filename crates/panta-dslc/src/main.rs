use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use panta_dsl_core::{Kind, emit_ts, format_source, parse};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("panta-dslc: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "check" | "validate" => {
            let input = required_path(&arguments, 1)?;
            let source = fs::read_to_string(input)
                .map_err(|error| format!("read {}: {error}", input.display()))?;
            parse(&source)
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        "emit-ts" => {
            let input = required_path(&arguments, 1)?;
            let output = required_path(&arguments, 2)?;
            let source = fs::read_to_string(input)
                .map_err(|error| format!("read {}: {error}", input.display()))?;
            let document = parse(&source).map_err(|error| error.to_string())?;
            if document.kind != Kind::Language {
                return Err("emit-ts requires kind: language".to_owned());
            }
            let locale = arguments
                .get(3)
                .map(String::as_str)
                .or(document.language.as_deref())
                .unwrap_or("en");
            let ts = emit_ts(&document, locale).map_err(|error| error.to_string())?;
            write_atomically(output, ts.as_bytes())
        }
        "format" => {
            let check = arguments.get(1).is_some_and(|value| value == "--check");
            let input_index = usize::from(check) + 1;
            let input = required_path(&arguments, input_index)?;
            let source = fs::read_to_string(input)
                .map_err(|error| format!("read {}: {error}", input.display()))?;
            let formatted = format_source(&source).map_err(|error| error.to_string())?;
            if check {
                if formatted == source {
                    Ok(())
                } else {
                    Err(format!("{} requires formatting", input.display()))
                }
            } else if formatted == source {
                Ok(())
            } else {
                write_atomically(input, formatted.as_bytes())
            }
        }
        _ => Err(usage()),
    }
}

fn required_path(arguments: &[String], index: usize) -> Result<&Path, String> {
    arguments.get(index).map(Path::new).ok_or_else(usage)
}

fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("out")
    ));
    let mut file = fs::File::create(&temporary)
        .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    file.write_all(contents)
        .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path).map_err(|error| format!("replace {}: {error}", path.display()))
}

fn usage() -> String {
    "usage: panta-dslc check <input.pa> | emit-ts <input.pa> <output.ts> [locale] | format [--check] <input.pa>".to_owned()
}
