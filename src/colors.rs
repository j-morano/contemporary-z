const COLORS: [(&str, &str); 32] = [
    ("black_fg", "30"),
    ("red_fg", "31"),
    ("green_fg", "32"),
    ("yellow_fg", "33"),
    ("blue_fg", "34"),
    ("magenta_fg", "35"),
    ("cyan_fg", "36"),
    ("white_fg", "37"),
    ("bright_black_fg", "90"),
    ("bright_red_fg", "91"),
    ("bright_green_fg", "92"),
    ("bright_yellow_fg", "93"),
    ("bright_blue_fg", "94"),
    ("bright_magenta_fg", "95"),
    ("bright_cyan_fg", "96"),
    ("bright_white_fg", "97"),
    ("black_bg", "40"),
    ("red_bg", "41"),
    ("green_bg", "42"),
    ("yellow_bg", "43"),
    ("blue_bg", "44"),
    ("magenta_bg", "45"),
    ("cyan_bg", "46"),
    ("white_bg", "47"),
    ("bright_black_bg", "100"),
    ("bright_red_bg", "101"),
    ("bright_green_bg", "102"),
    ("bright_yellow_bg", "103"),
    ("bright_blue_bg", "104"),
    ("bright_magenta_bg", "105"),
    ("bright_cyan_bg", "106"),
    ("bright_white_bg", "107"),
];

const SGR: [(&str, &str); 2] = [
    ("normal", "0"),
    ("bold", "1"),
];

pub(crate) fn color_code(color: &str) -> &str {
    for c in COLORS {
        if c.0 == color {
            return c.1;
        }
    }
    return "30";
}

pub(crate) fn sgr_code(sgr: &str) -> &str{
    for c in SGR {
        if c.0 == sgr {
            return c.1;
        }
    }
    return "0";
}