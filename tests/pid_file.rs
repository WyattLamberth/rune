// Production adapter helpers: filesystem atomicity and owned-process rollback.
// These tests do not prove complete kernel container cleanup or isolation.
#[path = "../vendor/libcontainer/src/process/owned_process.rs"]
mod owned_process;
#[path = "../vendor/libcontainer/src/process/pid_file.rs"]
mod pid_file;
