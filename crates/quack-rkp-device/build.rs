use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=bridge/RkpV1Bridge.java");
    for name in [
        "ANDROID_HOME",
        "ANDROID_SDK_ROOT",
        "ANDROID_BUILD_TOOLS_VERSION",
        "JAVA_HOME",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android")
        || env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("aarch64")
    {
        return;
    }
    let sdk = env::var_os("ANDROID_HOME")
        .filter(|root| !root.is_empty())
        .or_else(|| env::var_os("ANDROID_SDK_ROOT").filter(|root| !root.is_empty()))
        .map(PathBuf::from)
        .expect("V1 bridge build requires ANDROID_HOME or ANDROID_SDK_ROOT");
    let build_tools_dir = sdk.join("build-tools");
    let (d8, _version) = if let Ok(v) = env::var("ANDROID_BUILD_TOOLS_VERSION") {
        let p = build_tools_dir.join(&v).join("lib/d8.jar");
        (p, v)
    } else {
        let mut candidates = Vec::new();
        if let Ok(entries) = fs::read_dir(&build_tools_dir) {
            for entry in entries.flatten() {
                let p = entry.path().join("lib/d8.jar");
                if p.is_file() {
                    candidates.push((p, entry.file_name().to_string_lossy().to_string()));
                }
            }
        }
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        candidates.into_iter().next().unwrap_or_else(|| {
            let v = "36.0.0".to_string();
            (build_tools_dir.join(&v).join("lib/d8.jar"), v)
        })
    };
    assert!(
        d8.is_file(),
        "install Android SDK Build Tools (expected {})",
        d8.display()
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let classes = out.join("bridge-classes");
    fs::create_dir_all(&classes).expect("create bridge classes directory");
    let java_tool = |name: &str| {
        let name = if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.into()
        };
        env::var_os("JAVA_HOME")
            .filter(|root| !root.is_empty())
            .map(|root| PathBuf::from(root).join("bin").join(&name))
            .unwrap_or_else(|| PathBuf::from(name))
    };
    run(Command::new(java_tool("javac"))
        .args(["--release", "8", "-encoding", "UTF-8", "-d"])
        .arg(&classes)
        .arg("bridge/RkpV1Bridge.java"));
    run(Command::new(java_tool("java"))
        .arg("-cp")
        .arg(&d8)
        .args([
            "com.android.tools.r8.D8",
            "--release",
            "--no-desugaring",
            "--min-api",
            "26",
            "--output",
        ])
        .arg(out.join("rkp-v1.jar"))
        .arg(classes.join("duck/rkp/RkpV1Bridge.class")));
}

fn run(command: &mut Command) {
    let result = command
        .output()
        .unwrap_or_else(|error| panic!("run {command:?}: {error}"));
    assert!(
        result.status.success(),
        "{command:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
