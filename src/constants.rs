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

// CLI Application Constants

/// The name of the CLI application.
pub const APP_NAME: &str = "nest";

/// Description of the CLI application.
#[allow(dead_code)]
pub const APP_DESCRIPTION: &str = "Nest task runner";

// CLI Special Flags

/// Flag name for version information.
pub const FLAG_VERSION: &str = "version";

/// Flag name for showing commands in different formats.
pub const FLAG_SHOW: &str = "show";

/// Flag name for copying example nestfile.
pub const FLAG_EXAMPLE: &str = "example";

/// Flag name for updating Nest CLI.
pub const FLAG_UPDATE: &str = "update";

/// Flag name for specifying custom config file path.
pub const FLAG_CONFIG: &str = "config";

/// Flag name for dry-run mode.
pub const FLAG_DRY_RUN: &str = "dry-run";

/// Flag name for verbose output.
pub const FLAG_VERBOSE: &str = "verbose";

/// Flag name for generating shell completion.
pub const FLAG_COMPLETE: &str = "complete";

/// Flag name for showing standard command help.
pub const FLAG_STD: &str = "std";

/// Format option for JSON output.
pub const FORMAT_JSON: &str = "json";

/// Format option for AST output.
pub const FORMAT_AST: &str = "ast";

/// Short option for version flag.
pub const SHORT_VERSION: char = 'V';

// Command Structure Constants

/// Name of the default subcommand in group commands.
pub const DEFAULT_SUBCOMMAND: &str = "default";

// Standard Commands (Lifecycle & Diagnostics)
pub const CMD_LIST: &str = "list";
pub const CMD_CHECK: &str = "check";
pub const CMD_DOCTOR: &str = "doctor";
pub const CMD_CLEAN: &str = "clean";
pub const CMD_UNINSTALL: &str = "uninstall";

// Boolean Values

/// String representation of boolean true.
pub const BOOL_TRUE: &str = "true";

/// String representation of boolean false.
pub const BOOL_FALSE: &str = "false";

// Reserved Short Options

/// Reserved short option for help.
pub const RESERVED_SHORT_HELP: char = 'h';

/// Reserved short option for version.
pub const RESERVED_SHORT_VERSION: char = 'V';

/// Reserved short options that cannot be used as parameter aliases.
pub const RESERVED_SHORT_OPTIONS: &[char] = &[RESERVED_SHORT_HELP, RESERVED_SHORT_VERSION];

/// Name of the help flag (reserved).
/// Note: Currently not used, kept for future reference
#[allow(dead_code)]
pub const RESERVED_FLAG_HELP: &str = "help";

/// Name of the version flag (reserved).
/// Note: Currently not used, kept for future reference
#[allow(dead_code)]
pub const RESERVED_FLAG_VERSION: &str = "version";

// Template Variables

/// Template variable for current UTC time.
pub const TEMPLATE_VAR_NOW: &str = "{{now}}";

/// Template variable for current user.
pub const TEMPLATE_VAR_USER: &str = "{{user}}";

/// Template variable for system error message (available in fallback scripts).
pub const TEMPLATE_VAR_ERROR: &str = "{{SYSTEM_ERROR_MESSAGE}}";

// Environment Variables

/// Environment variable name for current user.
pub const ENV_VAR_USER: &str = "USER";

/// Default value for user when USER environment variable is not set.
pub const DEFAULT_USER: &str = "unknown";
