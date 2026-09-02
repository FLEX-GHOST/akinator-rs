pub mod client;
pub mod enums;
pub mod error;
pub mod models;
pub mod session_manager;

pub use client::{Akinator, AkinatorBuilder};
pub use enums::{Answer, Language, Theme};
pub use error::{Error, Result};
pub use models::{Guess, SessionData, StepResult};
pub use session_manager::SessionManager;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static _MALLOC_CONF: &[u8] = b"background_thread:true,dirty_decay_ms:0,muzzy_decay_ms:0\0";
