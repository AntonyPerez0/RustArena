// `sqlx::migrate!` copies the files in migrations/ into the program when it compiles.
// This line tells Cargo to recompile whenever a migration is added or changed;
// without it, a new migration file can be silently left out of the build.
fn main() {
    println!("cargo::rerun-if-changed=migrations");
}
