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
pub mod codegen;
pub mod command_handler;
pub mod completion;
pub mod standard_commands;

pub mod display;
pub mod env;
pub mod executor;
pub mod file;
pub mod handlers;
pub mod help;
pub mod include;
pub mod input;
pub mod json;
pub mod logging;
pub mod merge;
pub mod output;
pub mod parser;
pub mod path;
pub mod runtime;
pub mod template;
pub mod type_validator;
pub mod validator;
pub mod watcher;
