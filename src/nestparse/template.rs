use std::collections::HashMap;
use std::env;
use chrono::Utc;

pub struct TemplateProcessor;

impl TemplateProcessor {
    pub fn process(script: &str, args: &HashMap<String, String>) -> String {
        let mut processed = script.to_string();

        // Replace parameter placeholders {{param}}
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Replace special variables
        processed = processed.replace("{{now}}", &Utc::now().to_rfc3339());
        processed = processed.replace(
            "{{user}}",
            &env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
        );

        processed
    }
}

