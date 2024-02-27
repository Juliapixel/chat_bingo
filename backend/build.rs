use vergen::EmitBuilder;

fn main() {
    println!("cargo:rerun-if-changed=./migrations");

    EmitBuilder::builder()
        .all_git()
        .build_timestamp()
        .emit_at("../.git".into())
        .unwrap();
}
