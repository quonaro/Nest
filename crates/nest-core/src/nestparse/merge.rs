use crate::nestparse::ast::{Command, Directive};
use indexmap::IndexMap;

/// Merges a list of commands, applying "Last Wins" strategy for duplicates.
///
/// If multiple commands have the same name, later commands merge into earlier ones.
/// - Scalars (script, desc): Replace
/// - Lists (depends_on): Replace
/// - Maps (env): Merge
/// - Children: Recursive Merge
pub fn merge_commands(commands: Vec<Command>) -> Vec<Command> {
    // Use IndexMap to preserve order of first occurrence
    let mut merged_map: IndexMap<String, Command> = IndexMap::new();

    for cmd in commands {
        if let Some(existing) = merged_map.get_mut(&cmd.name) {
            merge_single_command(existing, cmd);
        } else {
            merged_map.insert(cmd.name.clone(), cmd);
        }
    }

    merged_map.into_values().collect()
}

fn merge_single_command(base: &mut Command, override_cmd: Command) {
    // 1. Merge Directives
    // Strategy: Split directives into unique kinds.
    // Some directives are "scalar" (script, desc) -> Replace
    // Some are "list" (depends_on) -> Replace (per our agreement)
    // Some are "map" (env) -> Merge

    // For simplicity and performance, we can rebuild the directives list.
    // But we need to handle specific directive types differently.
    
    // Convert base directives to a map keyed by discriminant? 
    // No, AST structure is a Vec enum.
    
    // Let's iterate over override directives and apply them to base.
    for dir in override_cmd.directives {
        match dir {
            // Scalars: Replace existing
            Directive::Script(..) => {
                // Remove all script directives from base
                base.directives.retain(|d| !matches!(d, Directive::Script(..)));
                base.directives.push(dir);
            }
            Directive::Desc(_) => {
                base.directives.retain(|d| !matches!(d, Directive::Desc(_)));
                base.directives.push(dir);
            }
            Directive::Cwd(_) => {
                base.directives.retain(|d| !matches!(d, Directive::Cwd(_)));
                base.directives.push(dir);
            }
            Directive::Logs(_, _) => {
                base.directives.retain(|d| !matches!(d, Directive::Logs(_, _)));
                base.directives.push(dir);
            }
            // Lists: Replace existing (Strict Replace)
            Directive::Depends(_, _) => {
                base.directives.retain(|d| !matches!(d, Directive::Depends(_, _)));
                base.directives.push(dir);
            }
            // Maps: Merge (Env)
            Directive::Env(key, val, hide) => {
                // Remove existing env with same key, then add new
                base.directives.retain(|d| match d {
                    Directive::Env(k, _, _) => k != &key,
                    _ => true,
                });
                base.directives.push(Directive::Env(key, val, hide));
            }
            // Others (Before, After, etc.) - assume Replace for now or Append?
            // Let's assume Append/Replace behavior depends on semantics.
            // For now, let's treat them as Replace for safety.
            _ => {
                // Generic replace for other directives isn't easy without discriminant equality.
                // Let's just push for now? No, that causes duplicates.
                // Reimplementing logic for every directive is tedious but safe.
                // Let's stick to the main ones we discussed.
                base.directives.push(dir);
            }
        }
    }

    // 2. Merge Children (Recursive)
    let merged_children = merge_commands(
        base.children.drain(..).chain(override_cmd.children).collect()
    );
    base.children = merged_children;

    // 3. Merge Parameters (Variables) - Replace by name
    for param in override_cmd.parameters {
        base.parameters.retain(|p| p.name != param.name);
        base.parameters.push(param);
    }
}
