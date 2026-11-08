use std::sync::OnceLock;

use regex::Regex;
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
    diffs_only: bool,
) -> Result<(), ProcessError> {
    for line in input {
        let line = line?;
        let line = line.as_str();
        process_line(line, config, output, diffs_only)?;
    }

    Ok(())
}

fn process_line(
    line: &str,
    config: &Config,
    output: &mut impl std::io::Write,
    diffs_only: bool,
) -> Result<(), ProcessLineError> {
    macro_rules! as_is {
        () => {{
            if !diffs_only {
                output.write_fmt(format_args!("{}\n", line)).map(|_| ())?;
            }

            Ok(())
        }};
    }

    if line.trim_start().starts_with('#') {
        return as_is!();
    }

    let Some((prefix, original_item_name)) = line.split_once('=') else {
        return as_is!();
    };

    static STRIP_METADATA_FROM_NAME: OnceLock<Regex> = OnceLock::new();
    let strip_metadata_from_name = STRIP_METADATA_FROM_NAME.get_or_init(|| {
        Regex::new(r"\{[^}]*\}|\([^)]*\)").expect("not runtime generated; should compile")
    });

    let stripped_item_name = strip_metadata_from_name.replace_all(original_item_name, "");
    let stripped_item_name = stripped_item_name.trim();

    let mut new_name: Option<String> = None;

    for category in config.categories.iter() {
        for category_item_name in category.item_names.iter() {
            if stripped_item_name.eq(category_item_name) {
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

    let rendered = format_args!("{prefix}={updated_name}\n");

    output.write_fmt(rendered)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category<const N: usize>(label: &str, item_names: [&str; N]) -> Category {
        Category {
            label: label.into(),
            item_names: item_names
                .iter()
                .map(|item_name| item_name.to_string())
                .collect(),
        }
    }

    fn single_category_config<const N: usize>(label: &str, item_names: [&str; N]) -> Config {
        Config {
            categories: vec![category(label, item_names)],
        }
    }

    #[derive(Debug, Default)]
    struct StringsOutput {
        inner: String,
    }

    impl std::io::Write for StringsOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            // VERY lossy conversion; drops invalid utf8 entirely.
            // But we aren't expecting invalid utf8 anyway.
            for chunk in buf.utf8_chunks() {
                self.inner.push_str(chunk.valid());
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn test_process(lines: &[&str], config: &Config, diffs_only: bool) -> Vec<String> {
        let mut lines = lines.iter().map(|line| Ok(line.to_string()));
        let mut output = StringsOutput::default();
        process(&mut lines, &mut output, config, diffs_only).expect("failed");

        output.inner.lines().map(str::to_string).collect()
    }

    #[test]
    fn lines_without_matches_are_unchanged() {
        let config = single_category_config("%!", ["A"]);

        for (input, expected) in [
            (&[][..], &[][..]), // Annoyingly, the [..] is required to force these to be inferred to be slices so that we can have multiple lenghts of input
            (&[""], &[""]),
            (&["a"], &["a"]),
            (&["a="], &["a="]),
            (&["a=="], &["a=="]),
            (&["=b"], &["=b"]),
            (&["==b"], &["==b"]),
            (&["a=b"], &["a=b"]),
            (&["a==b"], &["a==b"]),
            (&["a=b", "a=b"], &["a=b", "a=b"]),
            (&["a=b", "c=d", "e=f"], &["a=b", "c=d", "e=f"]),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn matches_before_eqals_sign_are_ignored() {
        let config = single_category_config("%!", ["A"]);

        for (input, expected) in [
            (&["A"][..], &["A"][..]),
            (&["A=a"], &["A=a"]),
            (&["A=A"], &["A=A!"]),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn meta_information_is_not_affected() {
        let config = single_category_config("(C) %", ["Spectral Arbalest"]);

        for (input, expected) in [
            (&["a=Spectral Arbalest"], &["a=(C) Spectral Arbalest"]),
            (
                &["a={^L}Spectral Arbalest"],
                &["a={^L}(C) Spectral Arbalest"],
            ),
            (
                &["a=(S) Spectral Arbalest"],
                &["a=(S) (C) Spectral Arbalest"],
            ),
            (
                &["a={^L}(S) Spectral Arbalest"],
                &["a={^L}(S) (C) Spectral Arbalest"],
            ),
            (
                &["a={^L}Spectral Arbalest (S)"],
                &["a={^L}(C) Spectral Arbalest (S)"],
            ),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn matches_that_arent_the_full_name_are_ignored() {
        let config = single_category_config("%!", ["A"]);

        for (input, expected) in [
            (&["a="], &["a="]),
            (&["a=A"], &["a=A!"]),
            (&["a=AA"], &["a=AA"]),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn meta_information_interspersed_means_item_is_skipped() {
        // I'd like to support this eventually, but it's a lot of parsing.

        let config = single_category_config("(C) %", ["Spectral Arbalest"]);

        for (input, expected) in [
            (&["a=Spectral{^L} Arbalest"], &["a=Spectral{^L} Arbalest"]),
            (
                &["a=(S) Spectral{^L} Arbalest"],
                &["a=(S) Spectral{^L} Arbalest"],
            ),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn diff_mode_outputs_diffs_only() {
        let config = single_category_config("%!", ["A"]);

        for (input, expected) in [
            (&["b=B"][..], &[][..]),
            (&["a=A", "b=B", "c=A"], &["a=A!", "c=A!"]),
        ] {
            assert_eq!(test_process(input, &config, true), expected);
        }
    }

    #[test]
    fn multiple_target_item_names_are_supported() {
        let config = single_category_config("%!", ["A", "B"]);

        for (input, expected) in [
            (&["b=B"][..], &["b=B!"][..]),
            (&["a=A", "b=B", "c=A"], &["a=A!", "b=B!", "c=A!"]),
            (&["a=A", "b=B", "c=C"], &["a=A!", "b=B!", "c=C"]),
            (
                &["a=A", "b=B", "c=C", "ab=AB"],
                &["a=A!", "b=B!", "c=C", "ab=AB"],
            ),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }

    #[test]
    fn multiple_categories_are_supported() {
        let config = Config {
            categories: vec![
                category("(C) %", ["Ellena's Necklace", "Living Ring"]),
                category("(B) %", ["Ellena's Necklace"]),
            ],
        };

        for (input, expected) in [
            (
                &["tagGDX1RingB101={^L}Living Ring"][..],
                &["tagGDX1RingB101={^L}(C) Living Ring"][..],
            ),
            (
                &[
                    "tagNecklaceB003={^L}Ellena's Necklace",
                    "tagGDX1RingB101={^L}Living Ring",
                ],
                &[
                    "tagNecklaceB003={^L}(C) (B) Ellena's Necklace",
                    "tagGDX1RingB101={^L}(C) Living Ring",
                ],
            ),
        ] {
            assert_eq!(test_process(input, &config, false), expected);
        }
    }
}
