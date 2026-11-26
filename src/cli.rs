use crate::config::Mode;


#[derive(Debug, PartialEq)]
pub(crate) enum AliasArgs {
    /// `-a`: choose among the aliased dirs and go to it.
    Select,
    /// `-a DIR`: remove the alias of DIR.
    Remove(String),
    /// `-a ALIAS DIR`: set the alias of DIR.
    Set { alias: String, dir: String },
}


#[derive(Debug, PartialEq)]
pub(crate) enum Command {
    /// No arguments: choose among the stored dirs.
    Select,
    Version,
    Help,
    DatabasePath,
    Clear,
    Previous,
    Last,
    ListAll,
    Sync,
    RemoveAlias,
    /// `-l [NUMBER]`
    List(Option<usize>),
    /// `-r [SUBSTRING]...`
    Remove(Vec<String>),
    Alias(AliasArgs),
    /// `-f SUBSTRING...`
    Find(Vec<String>),
    /// A dir, an alias or substrings; `mode` overrides the configured one.
    Cd { patterns: Vec<String>, mode: Option<Mode> },
}


/// Parse the command-line arguments (without the program name). All the
/// validation happens here, so commands can rely on well-formed input.
pub(crate) fn parse(args: &[String]) -> Result<Command, String> {
    let (first, rest) = match args.split_first() {
        Some((first, rest)) => (first.as_str(), rest),
        None => return Ok(Command::Select),
    };

    // Shorthands for the two most common kinds of options
    let alone = |command| no_arguments(first, rest, command);
    let forced = |mode| Ok(Command::Cd { patterns: substrings(first, rest)?, mode: Some(mode) });

    match first {
        "-v" | "--version" => alone(Command::Version),
        "-h" | "--help" => alone(Command::Help),
        "--database-path" => alone(Command::DatabasePath),
        "--clear" => alone(Command::Clear),
        "-" => alone(Command::Previous),
        "=" => alone(Command::Last),
        "--list-all" => alone(Command::ListAll),
        "--sync" => alone(Command::Sync),
        "--remove-alias" => alone(Command::RemoveAlias),
        "-l" => match rest {
            [] => Ok(Command::List(None)),
            [number] => number
                .parse()
                .map(|n| Command::List(Some(n)))
                .map_err(|_| format!("invalid number '{}'", number)),
            _ => Err("'-l' takes at most one number".to_string()),
        },
        "-r" => Ok(Command::Remove(rest.to_vec())),
        "-a" => match rest {
            [] => Ok(Command::Alias(AliasArgs::Select)),
            [dir] => Ok(Command::Alias(AliasArgs::Remove(dir.clone()))),
            [alias, dir] => Ok(Command::Alias(AliasArgs::Set {
                alias: alias.clone(),
                dir: dir.clone(),
            })),
            _ => Err("'-a' takes at most an alias and a directory".to_string()),
        },
        "-f" => Ok(Command::Find(substrings(first, rest)?)),
        "-t" => forced(Mode::Shortest),
        "-e" => forced(Mode::Score),
        "-b" => forced(Mode::Basename),
        // Everything after '--' is a dir or substring, even if it looks like
        // an option
        "--" => match rest {
            [] => Err("expected a directory or substring after '--'".to_string()),
            _ => Ok(Command::Cd { patterns: rest.to_vec(), mode: None }),
        },
        option if option.starts_with('-') => Err(format!("unknown option '{}'", option)),
        _ => Ok(Command::Cd { patterns: args.to_vec(), mode: None }),
    }
}


/// For options that take no arguments.
fn no_arguments(option: &str, rest: &[String], command: Command) -> Result<Command, String> {
    match rest.first() {
        None => Ok(command),
        Some(extra) => Err(format!("unexpected argument '{}' after '{}'", extra, option)),
    }
}


/// For options that take one or more substrings.
fn substrings(option: &str, rest: &[String]) -> Result<Vec<String>, String> {
    if rest.is_empty() {
        Err(format!("'{}' requires at least one substring", option))
    } else {
        Ok(rest.to_vec())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(args: &str) -> Result<Command, String> {
        let args: Vec<String> = args.split_whitespace().map(String::from).collect();
        parse(&args)
    }

    fn cd(patterns: &[&str], mode: Option<Mode>) -> Result<Command, String> {
        Ok(Command::Cd {
            patterns: patterns.iter().map(|p| p.to_string()).collect(),
            mode,
        })
    }

    #[test]
    fn no_arguments_selects() {
        assert_eq!(parse_str(""), Ok(Command::Select));
    }

    #[test]
    fn plain_arguments_are_dirs_or_substrings() {
        assert_eq!(parse_str("foo"), cd(&["foo"], None));
        assert_eq!(parse_str("foo bar"), cd(&["foo", "bar"], None));
        assert_eq!(parse_str("2024"), cd(&["2024"], None));
    }

    #[test]
    fn forced_modes_need_substrings() {
        assert_eq!(parse_str("-t foo"), cd(&["foo"], Some(Mode::Shortest)));
        assert_eq!(parse_str("-e foo bar"), cd(&["foo", "bar"], Some(Mode::Score)));
        assert_eq!(parse_str("-b foo"), cd(&["foo"], Some(Mode::Basename)));
        for option in ["-t", "-e", "-b", "-f"] {
            assert!(parse_str(option).is_err(), "{} without substring", option);
        }
    }

    #[test]
    fn simple_options_reject_extra_arguments() {
        assert_eq!(parse_str("-v"), Ok(Command::Version));
        assert_eq!(parse_str("--help"), Ok(Command::Help));
        assert_eq!(parse_str("-"), Ok(Command::Previous));
        assert_eq!(parse_str("="), Ok(Command::Last));
        assert!(parse_str("--sync now").is_err());
        assert!(parse_str("- foo").is_err());
    }

    #[test]
    fn list_takes_optional_number() {
        assert_eq!(parse_str("-l"), Ok(Command::List(None)));
        assert_eq!(parse_str("-l 5"), Ok(Command::List(Some(5))));
        assert!(parse_str("-l five").is_err());
        assert!(parse_str("-l 1 2").is_err());
    }

    #[test]
    fn alias_forms() {
        assert_eq!(parse_str("-a"), Ok(Command::Alias(AliasArgs::Select)));
        assert_eq!(parse_str("-a d"), Ok(Command::Alias(AliasArgs::Remove("d".into()))));
        assert_eq!(
            parse_str("-a x d"),
            Ok(Command::Alias(AliasArgs::Set { alias: "x".into(), dir: "d".into() }))
        );
        assert!(parse_str("-a x d y").is_err());
    }

    #[test]
    fn remove_accepts_no_substrings() {
        assert_eq!(parse_str("-r"), Ok(Command::Remove(vec![])));
        assert_eq!(parse_str("-r foo"), Ok(Command::Remove(vec!["foo".into()])));
    }

    #[test]
    fn unknown_options_are_errors_unless_after_double_dash() {
        assert!(parse_str("--sycn").is_err());
        assert!(parse_str("-x foo").is_err());
        assert_eq!(parse_str("-- -x"), cd(&["-x"], None));
        assert!(parse_str("--").is_err());
    }
}
