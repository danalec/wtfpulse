pub fn map_key_id_to_name(id: i64) -> String {
    match id {
        // Qt Key Codes (WhatPulse uses these)
        16777216 => "ESCAPE".to_string(),
        16777217 => "TAB".to_string(),
        16777219 => "BACKSPACE".to_string(),
        16777220 => "RETURN".to_string(),
        16777221 => "ENTER".to_string(),
        16777222 => "INSERT".to_string(),
        16777223 => "DELETE".to_string(),
        16777232 => "HOME".to_string(),
        16777233 => "END".to_string(),
        16777234 => "LEFT".to_string(),
        16777235 => "UP".to_string(),
        16777236 => "RIGHT".to_string(),
        16777237 => "DOWN".to_string(),
        16777238 => "PAGEUP".to_string(),
        16777239 => "PAGEDOWN".to_string(),
        16777252 => "CAPSLOCK".to_string(),
        16777264..=16777275 => format!("F{}", id - 16777263), // F1-F12

        // Legacy / ASCII mappings
        8 => "BACKSPACE".to_string(),
        9 => "TAB".to_string(),
        13 => "RETURN".to_string(),
        20 => "CAPSLOCK".to_string(),
        27 => "ESCAPE".to_string(),
        32 => "SPACE".to_string(),
        33 => "1".to_string(),     // ! -> 1
        34 => "QUOTE".to_string(), // "
        35 => "3".to_string(),     // # -> 3
        36 => "4".to_string(),     // $ -> 4
        37 => "5".to_string(),     // % -> 5
        38 => "7".to_string(),     // & -> 7
        39 => "QUOTE".to_string(), // '
        40 => "9".to_string(),     // ( -> 9
        41 => "0".to_string(),     // ) -> 0
        42 => "8".to_string(),     // * -> 8
        43 => "EQUAL".to_string(), // + -> =
        44 => "COMMA".to_string(),
        45 => "MINUS".to_string(),
        46 => "PERIOD".to_string(),
        47 => "SLASH".to_string(),
        48..=57 => ((id as u8) as char).to_string(), // 0-9
        58 => "SEMICOLON".to_string(),               // : -> ;
        59 => "SEMICOLON".to_string(),
        60 => "COMMA".to_string(), // < -> ,
        61 => "EQUAL".to_string(),
        62 => "PERIOD".to_string(),                  // > -> .
        63 => "SLASH".to_string(),                   // ? -> /
        64 => "2".to_string(),                       // @ -> 2
        65..=90 => ((id as u8) as char).to_string(), // A-Z
        91 => "BRACKETLEFT".to_string(),
        92 => "BACKSLASH".to_string(),
        93 => "BRACKETRIGHT".to_string(),
        94 => "6".to_string(),     // ^ -> 6
        95 => "MINUS".to_string(), // _ -> -
        96 => "GRAVE".to_string(),
        97..=122 => ((id as u8 - 32) as char).to_string(), // a-z -> A-Z
        123 => "BRACKETLEFT".to_string(),                  // { -> [
        124 => "BACKSLASH".to_string(),                    // | -> \
        125 => "BRACKETRIGHT".to_string(),                 // } -> ]
        126 => "GRAVE".to_string(),                        // ~ -> `
        // Function keys (approximate mapping if WhatPulse stores them as such)
        // Modifiers?
        // WhatPulse usually stores modifiers separately or not in keypress_frequency same way?
        // Assuming standard ASCII mapping for now.
        other => {
            // Try to convert to char if printable
            if let Some(c) = std::char::from_u32(other as u32) {
                let s = c.to_string().to_uppercase();
                // Special mappings for international keys to match API naming
                match s.as_str() {
                    "Ç" => return "CEDILLA".to_string(),
                    "Ñ" => return "NTILDE".to_string(),
                    _ => {}
                }

                if c.is_alphanumeric() {
                    return s;
                }
            }
            format!("UNKNOWN_{}", other)
        }
    }
}
