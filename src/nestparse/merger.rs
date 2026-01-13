//! Command merging logic for Nest CLI.
//!
//! This module handles merging of duplicate commands (e.g., from includes and overrides).
//! When a command with the same name exists multiple times at the same level,
//! they are merged into a single command.

use super::ast::Command;
use std::collections::HashMap;

/// Merges a list of commands, combining duplicates.
pub fn merge_commands(commands: Vec<Command>) -> Vec<Command> {
    let mut merged_map: HashMap<String, Command> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for cmd in commands {
        if let Some(existing) = merged_map.get_mut(&cmd.name) {
            merge_command(existing, cmd);
        } else {
            order.push(cmd.name.clone());
            merged_map.insert(cmd.name.clone(), cmd);
        }
    }

    // Reconstruction preserving order of first appearance
    order
        .into_iter()
        .map(|name| merged_map.remove(&name).unwrap())
        .collect()
}

/// Merges `source` command into `target` command.
fn merge_command(target: &mut Command, source: Command) {
    // 1. Merge parameters (source overrides target by name)
    // We want to keep target parameters that are not in source,
    // and replace target parameters that ARE in source,
    // and add new parameters from source.
    // However, for parameters, usually the new definition replaces the old valid signature.
    // If I have `run(force: bool)` and override with `run(debug: bool)`, what happens?
    // Usually in this context (configuration override), we probably want to *extend* or *replace*?
    // The user said: "if I write ... I change the description ... to change subcommands ... write exactly the same".
    // For parameters, if they are re-declared, it's safer to Assume the user wants to *replace* the parameter list 
    // OR merge them. 
    // Given the "override" nature, if parameters are present in the override, they might be adding to them? 
    // But since positional arguments matter, merging lists is tricky.
    // STRATEGY: If source has parameters, PREPEND/APPEND? No, parameters define the signature.
    // Let's assume: If source has parameters, they REPLACE target parameters entirely if the user re-defines the signature.
    // Wait, the parser handles `cmd:` as empty params.
    // If I have:
    // cmd:
    //   > desc: A
    //
    // And I do:
    // cmd:
    //   > desc: B
    //
    // The second `cmd` has empty parameters. Should it wipe strict parameters of first `cmd`?
    // Probably NOT. 
    // Logic: If source parameters are empty, keep target parameters.
    // If source parameters are NOT empty, REPLACE target parameters.
    if !source.parameters.is_empty() {
        target.parameters = source.parameters;
    }

    // 2. Merge directives
    // Some directives are "unique" (desc, cwd), others are accumulative (env? depends?).
    // For simplicity and typical override behavior: overwrite all of same type?
    // Or just append and let the runner handle it?
    // The runner usually takes the *last* directive if multiple are present (e.g. desc).
    // `env` might be additive.
    // Let's just append source directives to target directives. 
    // The runner logic usually iterates and sets state. If we append, the later ones (source) will come after.
    // If the runner respects "last wins", this works.
    target.directives.extend(source.directives);

    // 3. Merge children (recursively)
    target.children.extend(source.children);
    target.children = merge_commands(target.children.clone());

    // 4. Merge variables and constants
    // Variables: allow redef. Append to end.
    target.local_variables.extend(source.local_variables);
    
    // Constants: strictly generally don't allow redef in same scope, but here we are merging scopes.
    // We'll append. The validator might complain later if we don't deduplicate, 
    // OR the runner will use the first/last. 
    // The parser checks for constant redefinition *within a single parse*.
    // But here we are post-parse. 
    // Let's append.
    target.local_constants.extend(source.local_constants);

    // 5. Wildcard flag
    target.has_wildcard = target.has_wildcard || source.has_wildcard;
    
    // 6. Source file - keep target or update to source? 
    // Maybe keep target (original definition) or source (latest override).
    // Let's keep source to point to where the final override happened?
    // actually, purely for debugging, list of sources would be better, but we have one field.
    // Let's update to source to show where the "active" part might be coming from, or keep target.
    // Usually "where is this defined" implies the base. Let's keep target.
}
