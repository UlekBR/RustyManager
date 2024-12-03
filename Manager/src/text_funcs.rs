pub fn text_to_bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}