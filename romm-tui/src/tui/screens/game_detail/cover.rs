use ratatui_image::picker::ProtocolType;

pub(crate) fn detect_cover_protocol() -> Option<ProtocolType> {
    detect_cover_protocol_from_env(
        std::env::var("TERM_PROGRAM").ok(),
        std::env::var("TERM").ok(),
        std::env::var("KITTY_WINDOW_ID").ok(),
    )
}

pub fn detect_cover_protocol_from_env(
    term_program: Option<String>,
    term: Option<String>,
    kitty_window_id: Option<String>,
) -> Option<ProtocolType> {
    let term_program = term_program.unwrap_or_default();
    if kitty_window_id.is_some_and(|v| !v.is_empty()) {
        return Some(ProtocolType::Kitty);
    }
    if term_program.contains("iTerm")
        || term_program.contains("WezTerm")
        || term_program.contains("mintty")
        || term_program.contains("vscode")
        || term_program.contains("Tabby")
        || term_program.contains("Hyper")
        || term_program.contains("rio")
        || term_program.contains("WarpTerminal")
    {
        return Some(ProtocolType::Iterm2);
    }
    if term.is_some_and(|v| v.to_ascii_lowercase().contains("sixel")) {
        return Some(ProtocolType::Sixel);
    }
    None
}
