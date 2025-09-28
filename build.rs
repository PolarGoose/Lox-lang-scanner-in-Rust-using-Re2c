use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let scanner_src_path = Path::new("src").join("lox_language_scanner.re2c.rs");
    println!("cargo::rerun-if-changed={}", scanner_src_path.display());

    println!("Run build.rs");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let re2c_exe_path = Path::new(&out_dir).join("re2c.exe");

    if !re2c_exe_path.exists() {
        powershell_execute_script(&format!(
            r#"$ErrorActionPreference='Stop';
               Invoke-WebRequest -Uri "https://github.com/PolarGoose/re2c-for-Windows/releases/download/4.3/re2c.zip" -OutFile "{dir}re2c.zip";
               Expand-Archive -Path "{dir}re2c.zip" -DestinationPath "{dir}" -Force;"#,
            dir = out_dir.to_str().unwrap()
        ));
    }

    let scanner_out_path = Path::new(&out_dir).join("lox_language_scanner.rs");
    let mut status = Command::new(&re2c_exe_path)
        .arg(&scanner_src_path)
        .arg("-o")
        .arg(&scanner_out_path)
        .arg("-W")
        .arg("-Werror")
        .arg("--lang")
        .arg("rust")
        .status()
        .expect("failed to run re2c.exe");

    if !status.success() {
        panic!("re2c generation failed");
    }

    status = Command::new("rustfmt")
        .arg(&scanner_out_path)
        .status()
        .expect("failed to run rustfmt");

    if !status.success() {
        panic!("rustfmt failed");
    }
}

fn powershell_execute_script(script: &str) {
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .unwrap();
}
