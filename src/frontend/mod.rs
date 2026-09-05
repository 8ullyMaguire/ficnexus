//! Frontend module - serves the SvelteKit built SPA
//!
//! The SvelteKit build output is served as static files by
//! tower-http ServeDir in server.rs.
//! Static file serving is handled in server.rs via:
//! `.fallback_service(ServeDir::new(&frontend_dir).append_index_html_on_directories(true))`
//!
//! This module only provides constants and helpers.

//! Default location for the SvelteKit build output
pub const DEFAULT_FRONTEND_DIR: &str = "./frontend/build";

pub mod cache_headers;
