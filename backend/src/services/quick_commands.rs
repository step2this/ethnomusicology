// ST-007: Quick commands (shuffle, sort-by-bpm, reverse, undo)

/// Deterministic operations on a setlist that do not require LLM.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickCommand {
    Shuffle,
    SortByBpm,
    Reverse,
    Undo,
    RevertToVersion(i32),
}

/// Attempt to parse a user message as a quick command.
/// Returns None if it's not a recognized quick command (should go to LLM).
pub fn parse_quick_command(message: &str) -> Option<QuickCommand> {
    let s = message.trim().to_lowercase();
    match s.as_str() {
        "shuffle" => return Some(QuickCommand::Shuffle),
        "sort by bpm" | "sort-by-bpm" | "sort bpm" => return Some(QuickCommand::SortByBpm),
        "reverse" | "reverse order" => return Some(QuickCommand::Reverse),
        "undo" => return Some(QuickCommand::Undo),
        _ => {}
    }

    // "revert to version N"
    if let Some(rest) = s.strip_prefix("revert to version ") {
        if let Ok(n) = rest.trim().parse::<i32>() {
            return Some(QuickCommand::RevertToVersion(n));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse tests ---

    #[test]
    fn test_parse_shuffle() {
        assert_eq!(parse_quick_command("shuffle"), Some(QuickCommand::Shuffle));
        assert_eq!(parse_quick_command("SHUFFLE"), Some(QuickCommand::Shuffle));
        assert_eq!(
            parse_quick_command("  shuffle  "),
            Some(QuickCommand::Shuffle)
        );
    }

    #[test]
    fn test_parse_sort_by_bpm() {
        assert_eq!(
            parse_quick_command("sort by bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("sort-by-bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("sort bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("SORT BY BPM"),
            Some(QuickCommand::SortByBpm)
        );
    }

    #[test]
    fn test_parse_reverse() {
        assert_eq!(parse_quick_command("reverse"), Some(QuickCommand::Reverse));
        assert_eq!(
            parse_quick_command("reverse order"),
            Some(QuickCommand::Reverse)
        );
        assert_eq!(parse_quick_command("REVERSE"), Some(QuickCommand::Reverse));
    }

    #[test]
    fn test_parse_undo() {
        assert_eq!(parse_quick_command("undo"), Some(QuickCommand::Undo));
        assert_eq!(parse_quick_command("UNDO"), Some(QuickCommand::Undo));
    }

    #[test]
    fn test_parse_revert() {
        assert_eq!(
            parse_quick_command("revert to version 3"),
            Some(QuickCommand::RevertToVersion(3))
        );
        assert_eq!(
            parse_quick_command("revert to version 1"),
            Some(QuickCommand::RevertToVersion(1))
        );
        assert_eq!(
            parse_quick_command("revert to version 0"),
            Some(QuickCommand::RevertToVersion(0))
        );
        assert_eq!(
            parse_quick_command("REVERT TO VERSION 7"),
            Some(QuickCommand::RevertToVersion(7))
        );
    }

    #[test]
    fn test_parse_not_quick_command() {
        assert_eq!(parse_quick_command("swap track 5"), None);
        assert_eq!(parse_quick_command("add something darker"), None);
        assert_eq!(parse_quick_command("hello"), None);
        assert_eq!(parse_quick_command("revert to version"), None);
        assert_eq!(parse_quick_command("revert to version abc"), None);
    }
}
