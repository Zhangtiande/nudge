//! Nudge - LLM-powered CLI auto-completion library
//!
//! This library provides the core functionality for Nudge, including:
//! - Configuration management
//! - Context gathering (history, CWD, git, plugins)
//! - LLM API integration
//! - Sensitive data sanitization
//! - Dangerous command detection
//!
//! # FFI Support
//!
//! When compiled with the `ffi` feature, this library exports C-compatible
//! functions that can be loaded by shell integration scripts for lower-latency
//! completion.

pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod paths;
pub mod protocol;

#[cfg(all(unix, feature = "ffi"))]
pub mod ffi;
