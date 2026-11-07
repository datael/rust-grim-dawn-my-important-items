use std::io::Write;

use serde::Deserialize;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("I/O Error")]
    IO(#[from] std::io::Error),
    #[error("error in line processing")]
    ProcessLine(#[from] ProcessLineError),
}

#[derive(Error, Debug)]
pub enum ProcessLineError {
    #[error("I/O Error")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Category {
    /// The replacement template.
    ///
    /// The token `%` is replaced with the matched item name (or the current
    /// value, when several categories match the same line). If `%` is absent
    /// the original name is dropped and the template is used verbatim.
    pub label: String,

    /// Item names to match. Matching is a substring test against the portion of
    /// the line after the first `=`.
    #[serde(default)]
    pub item_names: Vec<String>,
}

/// Top-level config: a list of categories applied in declaration order.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Each `[[category]]` in the config file.
    #[serde(default, rename = "category")]
    pub categories: Vec<Category>,
}

pub fn process(
    input: &mut impl Iterator<Item = Result<std::string::String, std::io::Error>>,
    output: &mut impl std::io::Write,
    config: &Config,
    verbose: bool,
) -> Result<(), ProcessError> {
    for line in input {
        let line = line?;
        let line = line.as_str();
        process_line(line, config, output, verbose)?;
    }

    Ok(())
}

fn process_line(
    line: &str,
    config: &Config,
    output: &mut impl std::io::Write,
    verbose: bool,
) -> Result<(), ProcessLineError> {
    macro_rules! as_is {
        () => {
            Ok(output.write_fmt(format_args!("{}\n", line)).map(|_| ())?)
        };
    }

    if line.trim_start().starts_with('#') {
        return as_is!();
    }

    let Some((prefix, original_item_name)) = line.split_once('=') else {
        return as_is!();
    };

    let mut new_name: Option<String> = None;

    for category in config.categories.iter() {
        for category_item_name in category.item_names.iter() {
            if original_item_name.contains(category_item_name) {
                let replace_with = &category.label.replace('%', category_item_name);

                new_name = Some(match new_name {
                    Some(new_name) => new_name.replace(category_item_name, replace_with),
                    None => original_item_name.replace(category_item_name, replace_with),
                });

                // Assuming an item isn't named doubly and that config is spelling out the entire name.
                break;
            }
        }
    }

    let Some(updated_name) = new_name else {
        return as_is!();
    };

    if verbose {
        std::io::stderr()
            .lock()
            .write_fmt(format_args!("{prefix}={updated_name}\n"))?;
    }

    output.write_fmt(format_args!("{prefix}={updated_name}\n"))?;

    Ok(())
}
