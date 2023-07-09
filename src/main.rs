mod handlers;

use std::{collections::HashMap, hash::Hash, io::stdout, io::Write, process::Command};

use anyhow::{Context, Result};
use pest::Parser;
use pest_derive::Parser;
use rustix::path::Arg;

use crate::handlers::{build_builtins, build_modifiers};

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct MotdTemplateParser;

fn run(template: &str) -> Result<()> {
    let file = {
        let mut pairs = MotdTemplateParser::parse(Rule::file, template)?;
        pairs.next().with_context(|| "Failed to parse file")?
    };
    assert!(file.as_rule() == Rule::file);

    let builtin_registry = build_builtins();
    let modifier_registry = build_modifiers();
    let mut custom_registry: HashMap<String, String> = HashMap::new();
    let mut environ_registry: HashMap<String, String> = HashMap::new();

    let mut cout = stdout().lock();
    for item in file.into_inner() {
        match item.as_rule() {
            Rule::anystr => {
                write!(cout, "{}", item.as_str())?;
            }
            Rule::escape => {
                write!(cout, "{}", item.as_str().get(..1).unwrap())?;
            }
            Rule::configs => {
                for subitem in item.into_inner() {
                    match subitem.as_rule() {
                        Rule::environ => {
                            let mut environ = subitem.into_inner();
                            let name = environ.next().expect("BUG: Expects an identifier");
                            let value = environ.next().expect("BUG: Expects an identifier");
                            if let Some(f) = builtin_registry.get(value.as_str()) {
                                let value = f()?;
                                environ_registry
                                    .entry(name.as_str().to_string())
                                    .and_modify(|e| *e = value.clone())
                                    .or_insert_with(|| value);
                            } else {
                                anyhow::bail!("Unknown builtin identifier {}", value.as_str());
                            }
                        }
                        Rule::config => {
                            let mut config = subitem.into_inner();
                            let custom = config.next().expect("BUG: Expects an identifier");
                            let command = config.next().expect("BUG: Expects a command");
                            custom_registry.insert(
                                custom.as_str().to_string_lossy().to_string(),
                                command.as_str().to_string_lossy().to_string(),
                            );
                            assert!(config.next().is_none());
                        }
                        _ => unreachable!("Unknown config expression"),
                    }
                }
            }
            Rule::template => {
                for subitem in item.into_inner() {
                    let buf = match subitem.as_rule() {
                        Rule::builtin => {
                            let (ident, modifiers) = parse_expression(subitem);
                            if let Some(f) = builtin_registry.get(ident.as_str()) {
                                apply_modifiers(f()?, modifiers, &modifier_registry)?
                            } else {
                                anyhow::bail!("Unknown builtin identifier {}", ident);
                            }
                        }
                        Rule::custom => {
                            let (ident, modifiers) = parse_expression(subitem);
                            if let Some(c) = custom_registry.get(ident.as_str()) {
                                let buf = Command::new("sh")
                                    .arg("-c")
                                    .arg(c)
                                    .envs(&environ_registry)
                                    .output()?
                                    .stdout
                                    .to_string_lossy()
                                    .trim()
                                    .to_owned();
                                apply_modifiers(buf, modifiers, &modifier_registry)?
                            } else {
                                anyhow::bail!("Undefined custom identifier '{}'", ident.as_str());
                            }
                        }

                        _ => unreachable!("Undefined command types"),
                    };
                    write!(cout, "{buf}")?;
                }
            }
            Rule::whitespace => {}
            Rule::EOI => break,
            Rule::file => anyhow::bail!("BUG: Multiple file templates"),
            _ => {
                let (line, col) = item.line_col();
                anyhow::bail!("Syntax error: Found '{}' outside of template or config expressions, starting at line {}, col {}", item.as_str(), line, col)
            }
        }
    }
    Ok(())
}

/// Parse template expression like { identifier :mod1 :mod2 }
fn parse_expression(
    subitem: pest::iterators::Pair<'_, Rule>,
) -> (
    pest::iterators::Pair<'_, Rule>,
    Option<pest::iterators::Pair<'_, Rule>>,
) {
    let mut expr = subitem.into_inner();
    let ident = expr.next().expect("BUG: Expects an identifier");
    let modifiers = expr.next();
    (ident, modifiers)
}

/// Apply optional modifiers for commands like { command :mod1 :mod2 }
fn apply_modifiers(
    source: String,
    modifiers: Option<pest::iterators::Pair<'_, Rule>>,
    modifier_registry: &HashMap<String, fn(&str) -> String>,
) -> Result<String> {
    let mut buf = source;
    if let Some(modifiers) = modifiers {
        for modifier in modifiers.into_inner() {
            if let Some(f2) = modifier_registry.get(modifier.as_str()) {
                buf = f2(&buf);
            } else {
                anyhow::bail!("Unknown modifier identifier '{}'", modifier.as_str());
            }
        }
    }

    Ok(buf)
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    colored::control::set_override(true);
    match args.len() {
        2 => {
            let template = std::fs::read_to_string(&args[1])
                .with_context(|| format!("Failed to read file '{}'", &args[1]))?;
            let result = run(&template);
            if let Err(e) = result {
                let mut cout = stdout().lock();
                writeln!(cout)?;

                eprintln!("An error occured during generating motd message:\n{e}");
            }
        }
        _ => eprintln!("Usage: {} <template file path>", args[0]),
    }
    Ok(())
}
