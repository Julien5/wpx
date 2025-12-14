pub fn shorten_name(name: &str) -> String {
    if name.len() < 10 {
        return name.to_string();
    }
    let mut result_parts: Vec<&str> = Vec::new();
    let parts = name.split(|c: char| c.is_whitespace() || c == '-');
    for part in parts {
        if part.is_empty() {
            continue;
        }
        result_parts.push(part);
        let ret = result_parts.join(" ");
        if ret.len() > 5 {
            return ret;
        }
    }

    result_parts.join(" ")
}
