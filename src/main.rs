use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn usage() {
    eprintln!("Cinder compiler prototype\n\nUsage:\n  cinder check <file.cind>\n  cinder build <file.cind> [-o output]\n  cinder run <file.cind>\n  cinder emit-ir <file.cind>\n  cinder emit-cpp <file.cind> [-o output.cpp]");
}
fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}
fn build(source_path: &Path, output: &Path) -> Result<(), String> {
    let source = read_source(source_path)?;
    let cpp = cinder::compile_to_cpp(&source)?;
    let temp_cpp = env::temp_dir().join(format!("cinder-{}.cpp", std::process::id()));
    fs::write(&temp_cpp, cpp).map_err(|e| format!("cannot write temporary C++ file: {e}"))?;
    let compiler = env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    let optimization = env::var("CINDER_OPT_LEVEL").unwrap_or_else(|_| "3".to_string());
    if !matches!(optimization.as_str(), "0" | "1" | "2" | "3" | "s" | "z") {
        let _ = fs::remove_file(&temp_cpp);
        return Err("CINDER_OPT_LEVEL must be one of 0, 1, 2, 3, s, or z".into());
    }
    let mut command = Command::new(&compiler);
    command.arg("-std=c++20").arg(format!("-O{optimization}")).arg("-DNDEBUG").arg("-Wall").arg("-Wextra").arg("-pedantic");
    if env::var("CINDER_LTO").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true")) { command.arg("-flto"); }
    let result = command.arg(&temp_cpp).arg("-o").arg(output).output()
        .map_err(|e| format!("could not start native compiler '{compiler}': {e}"))?;
    let _ = fs::remove_file(&temp_cpp);
    if !result.status.success() {
        return Err(format!("native compiler failed:\n{}{}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)));
    }
    println!("Built native executable: {}", output.display());
    Ok(())
}
fn default_output(source: &Path) -> PathBuf {
    let mut path = source.to_path_buf();
    path.set_extension(if cfg!(windows) { "exe" } else { "" });
    path
}
fn real_main() -> Result<ExitCode, String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    // The cinderun executable is a convenience runner: cinderun file.cind.
    if env::current_exe().ok().and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned())).as_deref() == Some("cinderun") {
        args.insert(0, "run".to_string());
    }
    if args.len() < 2 { usage(); return Err("missing command or source file".into()); }
    let command = args[0].as_str();
    let source_path = Path::new(&args[1]);
    match command {
        "check" => {
            let source = read_source(source_path)?;
            cinder::compile_to_ir(&source)?;
            println!("Cinder check passed: {}", source_path.display());
            Ok(ExitCode::SUCCESS)
        }
        "emit-ir" => {
            let source = read_source(source_path)?;
            let program = cinder::compile_to_ir(&source)?;
            println!("{program:#?}");
            Ok(ExitCode::SUCCESS)
        }
        "emit-cpp" => {
            let source = read_source(source_path)?;
            let cpp = cinder::compile_to_cpp(&source)?;
            let output = if args.len() >= 4 && args[2] == "-o" {
                PathBuf::from(&args[3])
            } else {
                let mut path = source_path.to_path_buf(); path.set_extension("cpp"); path
            };
            fs::write(&output, cpp).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
            println!("Emitted C++: {}", output.display());
            Ok(ExitCode::SUCCESS)
        }
        "build" => {
            let output = if args.len() >= 4 && args[2] == "-o" { PathBuf::from(&args[3]) } else { default_output(source_path) };
            build(source_path, &output)?;
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let output = env::temp_dir().join(format!("cinder-run-{}{}", std::process::id(), if cfg!(windows) { ".exe" } else { "" }));
            build(source_path, &output)?;
            let result = Command::new(&output).status().map_err(|e| format!("could not run {}: {e}", output.display()));
            let _ = fs::remove_file(&output);
            let status = result?;
            Ok(ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8))
        }
        _ => { usage(); Err(format!("unknown command '{command}'")) }
    }
}
fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(error) => { eprintln!("cinder: {error}"); ExitCode::FAILURE }
    }
}
