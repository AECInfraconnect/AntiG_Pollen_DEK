use std::fs;
use std::path::PathBuf;

fn main() {
    let log_dir = PathBuf::from("C:\\NonExistentDirThatRequiresAdmin");
    let file_appender = tracing_appender::rolling::daily(&log_dir, "test.log");
    println!("Appender created.");
}
