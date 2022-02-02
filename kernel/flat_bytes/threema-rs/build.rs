use std::env;
use std::fs;
use std::path;
use webpki::trust_anchor_util::cert_der_as_trust_anchor;
use webpki::trust_anchor_util::generate_code_for_trust_anchors;

fn main() {
    println!("cargo:rerun-if-changed=src/ca.der");
    let ca = fs::read("src/ca.der").expect("Couldn't open ca.der");
    let trust_anchor = cert_der_as_trust_anchor(&ca).expect("Couldn't parse ca.der");
    let src = generate_code_for_trust_anchors("THREEMA_CA", &[trust_anchor]);

    let target = path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not found")).join("src");
    fs::create_dir_all(&target).expect("Couldn't create target dir");
    let fname = target.join("ca.rs");
    fs::write(fname, src.as_bytes()).expect("Couldn't write ca.rs");
}
