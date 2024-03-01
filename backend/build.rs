use vergen::EmitBuilder;

fn main() {
    println!("cargo:rerun-if-changed=./migrations");

    EmitBuilder::builder()
        .all_git()
        .git_dirty(true)
        .build_timestamp()
        .emit_at("../.git".into())
        .unwrap();
}
