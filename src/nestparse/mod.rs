//! Core parsing and execution modules for Nest.
//!
//! This module contains all the functionality for:
//! - Parsing Nestfile configuration files
//! - Building dynamic CLI interfaces
//! - Executing commands and scripts
//! - Managing environment variables and templates

pub mod args;
pub mod ast;
pub mod cli;
pub mod command_handler;
pub mod display;
pub mod env;
pub mod executor;
pub mod file;
pub mod help;
pub mod include;
pub mod json;
pub mod output;
pub mod parser;
pub mod path;
pub mod template;
pub mod type_validator;
pub mod validator;
