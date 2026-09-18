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
            let source = read_source(input)?;
            parse(&source)
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        "emit-ts" => {
            let input = required_path(&arguments, 1)?;
            let output = required_path(&arguments, 2)?;
            let source = read_source(input)?;
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
            let source = read_source(input)?;
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

fn read_source(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    String::from_utf8(bytes).map_err(|error| {
        format!(
            "pa.invalid_utf8 at byte {} in {}",
            error.utf8_error().valid_up_to(),
            path.display()
        )
    })
}

fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("out")
    ));
    // 创建/写入/落盘三步共用同一错误面（同目标同操作语义），任一失败都
    // 以临时文件路径报告；临时文件残留由调用方重试覆盖。
    let write = fs::File::create(&temporary)
        .and_then(|mut file| file.write_all(contents).and_then(|_| file.sync_all()));
    write.map_err(|error| format!("write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path).map_err(|error| format!("replace {}: {error}", path.display()))
}

fn usage() -> String {
    "usage: panta-dslc check <input.pa> | emit-ts <input.pa> <output.ts> [locale] | format [--check] <input.pa>".to_owned()
}

#[cfg(test)]
mod tests {
    // 测试统一以 ? 传播错误：避免永不执行的错误闭包（会被计为未覆盖行），
    // 同时不使用 unwrap/expect（工作区 lint 与 011 决策）。
    use super::{required_path, run, usage, write_atomically};
    use std::fs;
    use std::path::PathBuf;

    // canonical 形态经 formatter 实测（cn 别名归一为 zh-CN、补唯一末尾换行）；
    // dirty 夹具利用解析器容忍的差异（cn 别名、缺末尾换行），4 空格缩进会
    // 被解析器直接拒绝，不能作为"待格式化"样本。
    const LANGUAGE_CANONICAL: &str = "version: 1\nkind: language\nlanguage: zh-CN\nsourcelanguage: en\n\n[App]\ntitle:\n  src: Hi\n  tr: Hi\n";
    const LANGUAGE_NEEDS_FORMAT: &str = "version: 1\nkind: language\nlanguage: cn\nsourcelanguage: en\n\n[App]\ntitle:\n  src: Hi\n  tr: Hi";
    const VARIABLES: &str = "version: 1\nkind: variables\n\nvalues:\n  spacing-small: real = 8\n";

    /// 隔离临时目录：测试结束整体清理，避免触碰仓库工作区。
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> std::io::Result<Self> {
            let path =
                std::env::temp_dir().join(format!("panta-dslc-{}-{label}", std::process::id()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn write(&self, name: &str, contents: &str) -> std::io::Result<PathBuf> {
            let file = self.path.join(name);
            fs::write(&file, contents)?;
            Ok(file)
        }

        fn read(&self, name: &str) -> std::io::Result<String> {
            fs::read_to_string(self.path.join(name))
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn missing_or_unknown_command_reports_usage() {
        for arguments in [vec![], vec!["nonsense".to_owned()]] {
            assert!(
                matches!(run(arguments), Err(ref error) if *error == usage()),
                "无效应命令必须报 usage"
            );
        }
    }

    #[test]
    fn check_and_validate_accept_canonical_dictionary() -> Result<(), String> {
        let dir = TempDir::new("check").map_err(|e| e.to_string())?;
        let input = dir
            .write("ok.pa", LANGUAGE_CANONICAL)
            .map_err(|e| e.to_string())?;
        for command in ["check", "validate"] {
            run(vec![command.to_owned(), input.display().to_string()])?;
        }
        Ok(())
    }

    #[test]
    fn check_reports_parse_diagnostics() -> Result<(), String> {
        let dir = TempDir::new("check-bad").map_err(|e| e.to_string())?;
        let input = dir
            .write("bad.pa", "version: 1\nkind: language\nlanguage: en\n")
            .map_err(|e| e.to_string())?;
        assert!(
            matches!(
                run(vec!["check".to_owned(), input.display().to_string()]),
                Err(ref error) if error.contains("pa.")
            ),
            "坏字典必须带诊断失败"
        );
        Ok(())
    }

    #[test]
    fn emit_ts_writes_requested_locale_and_cleans_temporary() -> Result<(), String> {
        let dir = TempDir::new("emit").map_err(|e| e.to_string())?;
        let input = dir
            .write("cn.pa", LANGUAGE_CANONICAL)
            .map_err(|e| e.to_string())?;
        let output = dir.path.join("out.ts");
        run(vec![
            "emit-ts".to_owned(),
            input.display().to_string(),
            output.display().to_string(),
            "zh_CN".to_owned(),
        ])?;
        let ts = dir.read("out.ts").map_err(|e| e.to_string())?;
        assert!(ts.contains("zh_CN"), "locale 应写入 TS");
        assert!(
            !dir.path.join("out.ts.tmp").exists(),
            "临时文件应被 rename 消费"
        );
        Ok(())
    }

    #[test]
    fn emit_ts_falls_back_to_document_locale() -> Result<(), String> {
        let dir = TempDir::new("emit-doc-locale").map_err(|e| e.to_string())?;
        let input = dir
            .write("en.pa", LANGUAGE_CANONICAL)
            .map_err(|e| e.to_string())?;
        let output = dir.path.join("out.ts");
        run(vec![
            "emit-ts".to_owned(),
            input.display().to_string(),
            output.display().to_string(),
        ])?;
        assert!(output.exists());
        Ok(())
    }

    #[test]
    fn emit_ts_rejects_non_language_kind_and_missing_output() -> Result<(), String> {
        let dir = TempDir::new("emit-bad").map_err(|e| e.to_string())?;
        let input = dir.write("vars.pa", VARIABLES).map_err(|e| e.to_string())?;
        assert!(
            matches!(
                run(vec![
                    "emit-ts".to_owned(),
                    input.display().to_string(),
                    dir.path.join("out.ts").display().to_string(),
                ]),
                Err(ref error) if error == "emit-ts requires kind: language"
            ),
            "variables 字典不得生成 TS"
        );
        assert!(
            matches!(
                run(vec!["emit-ts".to_owned(), input.display().to_string()]),
                Err(ref error) if *error == usage()
            ),
            "缺输出参数必须报 usage"
        );
        Ok(())
    }

    #[test]
    fn format_writes_only_when_needed() -> Result<(), String> {
        let dir = TempDir::new("format").map_err(|e| e.to_string())?;
        let dirty = dir
            .write("dirty.pa", LANGUAGE_NEEDS_FORMAT)
            .map_err(|e| e.to_string())?;
        run(vec!["format".to_owned(), dirty.display().to_string()])?;
        assert_eq!(
            dir.read("dirty.pa").map_err(|e| e.to_string())?,
            LANGUAGE_CANONICAL
        );

        let clean = dir
            .write("clean.pa", LANGUAGE_CANONICAL)
            .map_err(|e| e.to_string())?;
        run(vec!["format".to_owned(), clean.display().to_string()])?;
        assert_eq!(
            dir.read("clean.pa").map_err(|e| e.to_string())?,
            LANGUAGE_CANONICAL
        );
        Ok(())
    }

    #[test]
    fn format_check_distinguishes_clean_and_dirty() -> Result<(), String> {
        let dir = TempDir::new("format-check").map_err(|e| e.to_string())?;
        let clean = dir
            .write("clean.pa", LANGUAGE_CANONICAL)
            .map_err(|e| e.to_string())?;
        run(vec![
            "format".to_owned(),
            "--check".to_owned(),
            clean.display().to_string(),
        ])?;

        let dirty = dir
            .write("dirty.pa", LANGUAGE_NEEDS_FORMAT)
            .map_err(|e| e.to_string())?;
        assert!(
            matches!(
                run(vec![
                    "format".to_owned(),
                    "--check".to_owned(),
                    dirty.display().to_string(),
                ]),
                Err(ref error) if error.contains("requires formatting")
            ),
            "--check 对脏文件必须非零"
        );
        Ok(())
    }

    #[test]
    fn read_source_reports_missing_and_invalid_utf8() -> Result<(), String> {
        let dir = TempDir::new("read").map_err(|e| e.to_string())?;
        assert!(
            matches!(
                run(vec![
                    "check".to_owned(),
                    dir.path.join("absent.pa").display().to_string(),
                ]),
                Err(ref error) if error.starts_with("read ")
            ),
            "缺失文件必须报 read 诊断"
        );

        let invalid = dir.path.join("invalid.pa");
        fs::write(&invalid, [0xffu8, 0xfe]).map_err(|e| e.to_string())?;
        assert!(
            matches!(
                run(vec!["check".to_owned(), invalid.display().to_string()]),
                Err(ref error) if error.starts_with("pa.invalid_utf8 at byte 0")
            ),
            "非法 UTF-8 必须带字节位置"
        );
        Ok(())
    }

    #[test]
    fn required_path_reports_usage_without_argument() {
        assert!(
            matches!(
                required_path(&["check".to_owned()], 1),
                Err(ref error) if *error == usage()
            ),
            "缺参必须报 usage"
        );
    }

    #[test]
    fn write_atomically_replaces_content_and_covers_extensionless_target() -> Result<(), String> {
        let dir = TempDir::new("write").map_err(|e| e.to_string())?;
        let target = dir.write("a.ts", "old").map_err(|e| e.to_string())?;
        write_atomically(&target, b"new")?;
        assert_eq!(dir.read("a.ts").map_err(|e| e.to_string())?, "new");
        assert!(!dir.path.join("a.ts.tmp").exists());

        let extensionless = dir.path.join("plain");
        write_atomically(&extensionless, b"x")?;
        assert_eq!(dir.read("plain").map_err(|e| e.to_string())?, "x");
        Ok(())
    }

    #[test]
    fn write_atomically_reports_create_and_replace_failures() -> Result<(), String> {
        let dir = TempDir::new("write-fail").map_err(|e| e.to_string())?;
        assert!(
            matches!(
                write_atomically(&dir.path.join("absent-dir").join("a.ts"), b"x"),
                Err(ref error) if error.starts_with("write ")
            ),
            "不存在目录必须报 write 诊断"
        );

        // 目标是目录时 rename 失败：覆盖 replace 错误面。
        let as_directory = dir.path.join("as-dir.ts");
        fs::create_dir_all(&as_directory).map_err(|e| e.to_string())?;
        assert!(
            matches!(
                write_atomically(&as_directory, b"x"),
                Err(ref error) if error.starts_with("replace ")
            ),
            "目录目标必须报 replace 诊断"
        );
        Ok(())
    }
}
