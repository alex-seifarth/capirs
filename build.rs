/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
extern crate bindgen;
extern crate cmake;

use std::env;
use std::path::PathBuf;

fn main() {
    // output directory
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // build library vsomeipc - the C-wrapper for vsomeip3
    let _lib_vsomeipc = cmake::Config::new("libvsomeipc")
        .define("CMAKE_INSTALL_PREFIX", out_path.join("libvsomeipc").to_str().unwrap())
        .build();
    println!("cargo:rerun-if-changed=libvsomeip/CMakeLists.txt");
    println!("cargo:rerun-if-changed=libvsomeip/vsomeipc.h");
    println!("cargo:rerun-if-changed=libvsomeip/vsomeipc.cpp");

    let vsomeipc_include = out_path.join("libvsomeipc").join("include");
    let vsomeipc_lib = out_path.join("libvsomeipc").join("lib");

    // via bindgen generate the C/Rust FFI for vsomeipc
    let bindings = bindgen::Builder::default()
        .header(vsomeipc_include.join("vsomeipc.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("vsomeipc.rs"))
        .expect("Couldn't write bindings!");

    // link vsomeipc
    println!("cargo:rustc-link-search=native={}", vsomeipc_lib.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=vsomeipc");
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rustc-link-search=native=/usr/local");
    println!("cargo:rustc-link-lib=vsomeip3");
}