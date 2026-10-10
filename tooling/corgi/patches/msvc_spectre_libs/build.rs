//! Local patch of `msvc_spectre_libs 0.1.3` (see Cargo.toml and the workspace
//! `[patch.crates-io]` entry for rationale). Identical to upstream except the
//! `error` feature no longer panics when the VS Spectre-mitigated libs are
//! absent. And unlike the first revision of this patch, it no longer warns
//! unconditionally either: the warning fires only when `/Qspectre` is actually
//! requested, which is the only situation where absent libs matter.

fn main() {
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    add_spectre_link_search();
}

/// Windows requires additional steps to find the spectre-mitigated CRT libs.
/// More info: https://docs.microsoft.com/en-us/cpp/build/reference/qspectre
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn add_spectre_link_search() {
    use cc::windows_registry;
    use std::env;

    let target = env::var("TARGET").expect("missing TARGET");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("missing CARGO_CFG_TARGET_ARCH");
    let arch = match arch.as_str() {
        "x86_64" => "x64",
        "x86" => "x86",
        // The spectre\arm64ec directory doesn't have any libs in it, instead the spectre arm64 libs
        // contain both arm64 and arm64ec objects.
        "aarch64" | "arm64ec" => "arm64",
        "arm" => "arm32",
        _ => panic!("unsupported arch: {arch}"),
    };

    let tool = windows_registry::find_tool(&target, "cl.exe").expect("couldn't find cl.exe");
    let spectre_libs = tool.path().join(format!(r"..\..\..\..\lib\spectre\{arch}"));

    if spectre_libs.exists() {
        println!(
            "cargo:rustc-link-search=native={}",
            spectre_libs.into_os_string().into_string().unwrap()
        );
    } else {
        // Upstream panics here when the `error` feature is enabled (which the
        // pinned python-environment-tools rev requests unconditionally via
        // pet-*/Cargo.toml). That turns an optional VS component into a hard
        // Windows build failure, so this patch degrades instead — but only
        // shouts when it matters: if nobody passes `/Qspectre`, absent libs
        // change nothing about the link and warning every build is pure noise.
        // If `/Qspectre` IS requested without the libs installed, the link
        // fails cryptically, so that case keeps the warning.
        let spectre_requested = env::var("CARGO_ENCODED_RUSTFLAGS")
            .map(|flags| flags.to_ascii_lowercase().contains("qspectre"))
            .unwrap_or(false);
        if spectre_requested {
            println!("cargo:warning=No spectre-mitigated libs were found, but /Qspectre was requested. Linking proceeds with the regular CRT libs and will likely fail.");
        }
    }
}
