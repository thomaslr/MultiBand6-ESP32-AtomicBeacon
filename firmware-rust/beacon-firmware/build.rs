// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    embuild::espidf::sysenv::output();

    // Ensure src/secrets.rs exists for fresh clones & CI builds (ignored in git)
    let secrets_path = Path::new("src/secrets.rs");
    if !secrets_path.exists() {
        let _ = fs::write(
            secrets_path,
            "pub const WIFI_SSID: &str = \"\";\npub const WIFI_PASSWORD: &str = \"\";\n",
        );
    }

    // Pre-compress web UI assets with Gzip at build time (~80% payload reduction)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let data_dir = Path::new(&manifest_dir).parent().unwrap().parent().unwrap().join("data");
    println!("cargo:rerun-if-changed={}", data_dir.display());

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let assets = [
        "index.html",
        "setup.html",
        "style.css",
        "app.js",
        "favicon.svg",
        "manifest.json",
        "sw.js",
        "icon-192.png",
        "icon-512.png",
    ];
    for asset in assets {
        let src_path = data_dir.join(asset);
        println!("cargo:rerun-if-changed={}", src_path.display());
        let output = Command::new("gzip")
            .args(["-9", "-n", "-c", src_path.to_str().unwrap()])
            .output()
            .unwrap_or_else(|e| panic!("Failed to run gzip on {:?}: {}", src_path, e));
        assert!(
            output.status.success(),
            "gzip failed for {:?}: {}",
            src_path,
            String::from_utf8_lossy(&output.stderr)
        );
        let dst_path = Path::new(&out_dir).join(format!("{}.gz", asset));
        fs::write(&dst_path, &output.stdout)
            .unwrap_or_else(|e| panic!("Failed to write {:?}: {}", dst_path, e));
    }
}
