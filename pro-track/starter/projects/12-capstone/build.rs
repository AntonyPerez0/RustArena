// `sqlx::migrate!` copies the files in migrations/ into the program when it compiles.
// This line tells Cargo to recompile whenever a migration is added or changed.
fn main() {
    println!("cargo::rerun-if-changed=migrations");
}
