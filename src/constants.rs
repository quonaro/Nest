//! Constants used throughout the Nest application.

/// Valid configuration file names that Nest will search for.
///
/// The search is case-sensitive, so both lowercase and uppercase variants
/// are included.
pub const CONFIG_NAMES: [&str; 4] = ["nestfile", "Nestfile", "nest", "Nest"];

/// The number of spaces used for indentation in the configuration file.
///
/// Commands are nested using this indentation level.
pub const INDENT_SIZE: u8 = 4;

/// Reserved words that cannot be used as command or parameter names.
///
/// These words have special meaning in the Nestfile syntax.
#[allow(dead_code)]
pub const RESERVED_WORDS: [&str; 2] = ["nest", "default"];

/// Valid data types for command parameters.
///
/// Supported types:
/// - `str` - String values
/// - `bool` - Boolean values (true/false)
/// - `num` - Numeric values
/// - `arr` - Array of strings
#[allow(dead_code)]
pub const DATA_TYPES: [&str; 4] = ["str", "bool", "num", "arr"];
