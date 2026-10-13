// Exercise the same filesystem finalizer used by production create rollback.
// This is cleanup fault/retry evidence, not kernel isolation or concurrency.
#[path = "../vendor/libcontainer/src/container/cleanup.rs"]
mod cleanup;
