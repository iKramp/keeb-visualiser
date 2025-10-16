use ab_glyph::{FontArc, PxScale};
use tiny_skia::{Color, Pixmap};

use crate::draw_text;


pub fn parse_c_source(path: &str) -> Vec<Layer> {
    const START_KEYMAP: &str = "const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {";
    let keymap_string = std::fs::read_to_string(path).expect("Failed to read keymap.c");
    //remove new lines
    let keymap_string = keymap_string.replace("\n", " ");
    let keymap_start = keymap_string
        .find(START_KEYMAP)
        .expect("Failed to find keymaps");
    let keymap_string = &keymap_string[(keymap_start + START_KEYMAP.len())..];
    let keymap_end = keymap_string
        .find("};")
        .expect("Failed to find end of keymaps");
    let keymap_string = &keymap_string[..keymap_end];

    //remove /* */ comments
    let mut keymap_string = keymap_string.to_string();
    while let Some(start) = keymap_string.find("/*") {
        if let Some(end) = keymap_string[start..].find("*/") {
            keymap_string.replace_range(start..(start + end + 2), "");
        } else {
            break;
        }
    }

    //find all instances of [ ... ]
    let mut layer_names = Vec::new();
    let mut rest = keymap_string.as_str();
    while let Some(start) = rest.find('[') {
        if let Some(end) = rest[start..].find(']') {
            let layer = &rest[(start + 1)..(start + end)];
            layer_names.push(layer.trim().to_string());
            rest = &rest[(start + end + 1)..];
        } else {
            break;
        }
    }

    let mut layer_keys = Vec::new();

    let mut out_rest = keymap_string.as_str();
    while let Some(start) = out_rest.find("LAYOUT_40_macro(") {
        rest = &out_rest[(start + "LAYOUT_40_macro(".len())..];
        let mut curr_nesting = 1;
        loop {
            let next_open = rest.find('(');
            let next_close = rest.find(')');
            match (next_open, next_close) {
                (Some(o), Some(c)) => {
                    if o < c {
                        curr_nesting += 1;
                        rest = &rest[(o + 1)..];
                    } else {
                        curr_nesting -= 1;
                        if curr_nesting == 0 {
                            let layer = out_rest[(start + "LAYOUT_40_macro(".len())
                                ..(out_rest.len() - rest.len() + c)]
                                .trim();
                            layer_keys.push(layer.to_string());

                            out_rest = &rest[(c + 1)..];
                            break;
                        } else {
                            rest = &rest[(c + 1)..];
                        }
                    }
                }
                (Some(_), None) => {
                    panic!("Unmatched parentheses");
                }
                (None, Some(c)) => {
                    curr_nesting -= 1;
                    if curr_nesting == 0 {
                        let layer = out_rest
                            [(start + "LAYOUT_40_macro(".len())..(out_rest.len() - rest.len() + c)]
                            .trim();
                        layer_keys.push(layer.to_string());
                        out_rest = &rest[(c + 1)..];
                        break;
                    } else {
                        rest = &rest[(c + 1)..];
                    }
                }
                (None, None) => panic!("Unmatched parentheses"),
            }
        }
    }

    let parsed_layers = layer_keys.iter().map(|layer| {
        let mut keys = layer.split(',').map(|k| k.trim());
        let mut parsed_arr = Vec::new();
        while let Some(key) = keys.next() {
            if key.starts_with("MT(") || key.starts_with("LT(") {
                let second_part = keys.next().expect("Unmatched MT( or LT(");
                let parsed_key = format!("{},{}", key, second_part);
                parsed_arr.push(Keycode::parse_str(&parsed_key));
            } else {
                parsed_arr.push(Keycode::parse_str(key));
            }
        }
        parsed_arr

    }).collect::<Vec<_>>();

    layer_names.into_iter().zip(parsed_layers).map(|(name, keys)| {
        Layer { name, keys }
    }).collect::<Vec<_>>()
}

pub struct Layer {
    pub name: String,
    pub keys: Vec<Keycode>,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Keycode {
    NONE,
    TRANSPARENT,

    KC_A,
    KC_B,
    KC_C,
    KC_D,
    KC_E,
    KC_F,
    KC_G,
    KC_H,
    KC_I,
    KC_J,
    KC_K,
    KC_L,
    KC_M,
    KC_N,
    KC_O,
    KC_P,
    KC_Q,
    KC_R,
    KC_S,
    KC_T,
    KC_U,
    KC_V,
    KC_W,
    KC_X,
    KC_Y,
    KC_Z,

    KC_0,
    KC_1,
    KC_2,
    KC_3,
    KC_4,
    KC_5,
    KC_6,
    KC_7,
    KC_8,
    KC_9,

    KC_F1,
    KC_F2,
    KC_F3,
    KC_F4,
    KC_F5,
    KC_F6,
    KC_F7,
    KC_F8,
    KC_F9,
    KC_F10,

    KC_KB_VOLUME_UP,
    KC_KB_VOLUME_DOWN,
    KC_KB_MUTE,
    KC_MEDIA_PLAY,
    KC_MEDIA_PREV,
    KC_MEDIA_NEXT,

    KC_MINUS,
    KC_GRV,
    KC_QUOT,
    KC_BSLS,
    KC_LBRC,
    KC_RBRC,
    KC_EQUAL,
    KC_COMM,
    KC_DOT,
    KC_SLASH,

    MOD_LCTL,
    KC_BSPC,
    KC_LGUI,
    KC_SPACE,
    MOD_LALT,
    KC_TAB,
    KC_PGUP,
    KC_PGDN,
    KC_HOME,
    KC_END,
    KC_ENT,
    KC_ESC,
    KC_PSCR,

    MO(String), // layer
    MT(Box<Keycode>, Box<Keycode>), //hold/tap
    LT(String, Box<Keycode>), // layer, keycode
    S(Box<Keycode>), // shift
}

impl Keycode {
    fn parse_str(key: &str) -> Self {
        match key {
            "KC_NO" => { Keycode::NONE }
            "KC_TRNS" => { Keycode::TRANSPARENT }

            "KC_A" => { Keycode::KC_A }
            "KC_B" => { Keycode::KC_B }
            "KC_C" => { Keycode::KC_C }
            "KC_D" => { Keycode::KC_D }
            "KC_E" => { Keycode::KC_E }
            "KC_F" => { Keycode::KC_F }
            "KC_G" => { Keycode::KC_G }
            "KC_H" => { Keycode::KC_H }
            "KC_I" => { Keycode::KC_I }
            "KC_J" => { Keycode::KC_J }
            "KC_K" => { Keycode::KC_K }
            "KC_L" => { Keycode::KC_L }
            "KC_M" => { Keycode::KC_M }
            "KC_N" => { Keycode::KC_N }
            "KC_O" => { Keycode::KC_O }
            "KC_P" => { Keycode::KC_P }
            "KC_Q" => { Keycode::KC_Q }
            "KC_R" => { Keycode::KC_R }
            "KC_S" => { Keycode::KC_S }
            "KC_T" => { Keycode::KC_T }
            "KC_U" => { Keycode::KC_U }
            "KC_V" => { Keycode::KC_V }
            "KC_W" => { Keycode::KC_W }
            "KC_X" => { Keycode::KC_X }
            "KC_Y" => { Keycode::KC_Y }
            "KC_Z" => { Keycode::KC_Z }

            "KC_0" => { Keycode::KC_0 }
            "KC_1" => { Keycode::KC_1 }
            "KC_2" => { Keycode::KC_2 }
            "KC_3" => { Keycode::KC_3 }
            "KC_4" => { Keycode::KC_4 }
            "KC_5" => { Keycode::KC_5 }
            "KC_6" => { Keycode::KC_6 }
            "KC_7" => { Keycode::KC_7 }
            "KC_8" => { Keycode::KC_8 }
            "KC_9" => { Keycode::KC_9 }

            "KC_F1" => { Keycode::KC_F1 }
            "KC_F2" => { Keycode::KC_F2 }
            "KC_F3" => { Keycode::KC_F3 }
            "KC_F4" => { Keycode::KC_F4 }
            "KC_F5" => { Keycode::KC_F5 }
            "KC_F6" => { Keycode::KC_F6 }
            "KC_F7" => { Keycode::KC_F7 }
            "KC_F8" => { Keycode::KC_F8 }
            "KC_F9" => { Keycode::KC_F9 }
            "KC_F10" => { Keycode::KC_F10 }

            "KC_KB_VOLUME_UP" => { Keycode::KC_KB_VOLUME_UP }
            "KC_KB_VOLUME_DOWN" => { Keycode::KC_KB_VOLUME_DOWN }
            "KC_KB_MUTE" => { Keycode::KC_KB_MUTE }
            "KC_MPLY" => { Keycode::KC_MEDIA_PLAY }
            "KC_MPRV" => { Keycode::KC_MEDIA_PREV }
            "KC_MNXT" => { Keycode::KC_MEDIA_NEXT }

            "KC_MINUS" => { Keycode::KC_MINUS }
            "KC_GRV" => { Keycode::KC_GRV }
            "KC_QUOT" => { Keycode::KC_QUOT }
            "KC_BSLS" => { Keycode::KC_BSLS }
            "KC_LBRC" => { Keycode::KC_LBRC }
            "KC_RBRC" => { Keycode::KC_RBRC }
            "KC_EQUAL" => { Keycode::KC_EQUAL }
            "KC_COMM" => { Keycode::KC_COMM }
            "KC_DOT" => { Keycode::KC_DOT }
            "KC_SLASH" => { Keycode::KC_SLASH }

            "MOD_LCTL" => { Keycode::MOD_LCTL }
            "KC_BSPC" => { Keycode::KC_BSPC }
            "KC_LGUI" => { Keycode::KC_LGUI }
            "KC_SPACE" => { Keycode::KC_SPACE }
            "MOD_LALT" => { Keycode::MOD_LALT }
            "KC_TAB" => { Keycode::KC_TAB }
            "KC_PGUP" => { Keycode::KC_PGUP }
            "KC_PGDN" => { Keycode::KC_PGDN }
            "KC_HOME" => { Keycode::KC_HOME }
            "KC_END" => { Keycode::KC_END }
            "KC_ENT" => { Keycode::KC_ENT }
            "KC_ESC" => { Keycode::KC_ESC }
            "KC_PSCR" => { Keycode::KC_PSCR }

            k if k.starts_with("S(") && k.ends_with(')') => {
                let inner = &k[2..(k.len() - 1)];
                Keycode::S(Box::new(Keycode::parse_str(inner)))
            }
            k if k.starts_with("MO(") && k.ends_with(')') => {
                let layer = &k[3..(k.len() - 1)];
                Keycode::MO(layer.to_string())
            }
            k if k.starts_with("MT(") && k.ends_with(')') => {
                let inner = &k[3..(k.len() - 1)];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    panic!("Invalid MT() format");
                }
                let hold = Keycode::parse_str(parts[0]);
                let tap = Keycode::parse_str(parts[1]);
                Keycode::MT(Box::new(hold), Box::new(tap))
            }
            k if k.starts_with("LT(") && k.ends_with(')') => {
                let inner = &k[3..(k.len() - 1)];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    panic!("Invalid LT() format");
                }
                let layer = parts[0].to_string();
                let keycode = Keycode::parse_str(parts[1]);
                Keycode::LT(layer, Box::new(keycode))
            }
            _ => { panic!("Unknown keycode: {}", key); }
        }
    }

    pub fn render(&self, px: f32, py: f32, pixmap: &mut Pixmap, font: &FontArc, scale: PxScale, color: Color) -> bool {
        match self {
            Self::TRANSPARENT => {
                return false;
            }
            Self::NONE => {},

            Self::KC_A => Self::render_simple(px, py, "a", pixmap, font, scale, color),
            Self::KC_B => Self::render_simple(px, py, "b", pixmap, font, scale, color),
            Self::KC_C => Self::render_simple(px, py, "c", pixmap, font, scale, color),
            Self::KC_D => Self::render_simple(px, py, "d", pixmap, font, scale, color),
            Self::KC_E => Self::render_simple(px, py, "e", pixmap, font, scale, color),
            Self::KC_F => Self::render_simple(px, py, "f", pixmap, font, scale, color),
            Self::KC_G => Self::render_simple(px, py, "g", pixmap, font, scale, color),
            Self::KC_H => Self::render_simple(px, py, "h", pixmap, font, scale, color),
            Self::KC_I => Self::render_simple(px, py, "i", pixmap, font, scale, color),
            Self::KC_J => Self::render_simple(px, py, "j", pixmap, font, scale, color),
            Self::KC_K => Self::render_simple(px, py, "k", pixmap, font, scale, color),
            Self::KC_L => Self::render_simple(px, py, "l", pixmap, font, scale, color),
            Self::KC_M => Self::render_simple(px, py, "m", pixmap, font, scale, color),
            Self::KC_N => Self::render_simple(px, py, "n", pixmap, font, scale, color),
            Self::KC_O => Self::render_simple(px, py, "o", pixmap, font, scale, color),
            Self::KC_P => Self::render_simple(px, py, "p", pixmap, font, scale, color),
            Self::KC_Q => Self::render_simple(px, py, "q", pixmap, font, scale, color),
            Self::KC_R => Self::render_simple(px, py, "r", pixmap, font, scale, color),
            Self::KC_S => Self::render_simple(px, py, "s", pixmap, font, scale, color),
            Self::KC_T => Self::render_simple(px, py, "t", pixmap, font, scale, color),
            Self::KC_U => Self::render_simple(px, py, "u", pixmap, font, scale, color),
            Self::KC_V => Self::render_simple(px, py, "v", pixmap, font, scale, color),
            Self::KC_W => Self::render_simple(px, py, "w", pixmap, font, scale, color),
            Self::KC_X => Self::render_simple(px, py, "x", pixmap, font, scale, color),
            Self::KC_Y => Self::render_simple(px, py, "y", pixmap, font, scale, color),
            Self::KC_Z => Self::render_simple(px, py, "z", pixmap, font, scale, color),

            Self::KC_0 => Self::render_simple(px, py, "0", pixmap, font, scale, color),
            Self::KC_1 => Self::render_simple(px, py, "1", pixmap, font, scale, color),
            Self::KC_2 => Self::render_simple(px, py, "2", pixmap, font, scale, color),
            Self::KC_3 => Self::render_simple(px, py, "3", pixmap, font, scale, color),
            Self::KC_4 => Self::render_simple(px, py, "4", pixmap, font, scale, color),
            Self::KC_5 => Self::render_simple(px, py, "5", pixmap, font, scale, color),
            Self::KC_6 => Self::render_simple(px, py, "6", pixmap, font, scale, color),
            Self::KC_7 => Self::render_simple(px, py, "7", pixmap, font, scale, color),
            Self::KC_8 => Self::render_simple(px, py, "8", pixmap, font, scale, color),
            Self::KC_9 => Self::render_simple(px, py, "9", pixmap, font, scale, color),

            Self::KC_F1 => Self::render_two_chars(px, py, "F1", pixmap, font, scale, color),
            Self::KC_F2 => Self::render_two_chars(px, py, "F2", pixmap, font, scale, color),
            Self::KC_F3 => Self::render_two_chars(px, py, "F3", pixmap, font, scale, color),
            Self::KC_F4 => Self::render_two_chars(px, py, "F4", pixmap, font, scale, color),
            Self::KC_F5 => Self::render_two_chars(px, py, "F5", pixmap, font, scale, color),
            Self::KC_F6 => Self::render_two_chars(px, py, "F6", pixmap, font, scale, color),
            Self::KC_F7 => Self::render_two_chars(px, py, "F7", pixmap, font, scale, color),
            Self::KC_F8 => Self::render_two_chars(px, py, "F8", pixmap, font, scale, color),
            Self::KC_F9 => Self::render_two_chars(px, py, "F9", pixmap, font, scale, color),
            Self::KC_F10 => Self::render_three_chars(px, py, "F10", pixmap, font, scale, color),

            Self::KC_KB_VOLUME_UP => Self::render_two_chars(px, py, "V+", pixmap, font, scale, color),
            Self::KC_KB_VOLUME_DOWN => Self::render_two_chars(px, py, "V-", pixmap, font, scale, color),
            Self::KC_KB_MUTE => Self::render_three_chars(px, py, "mut", pixmap, font, scale, color),
            Self::KC_MEDIA_PLAY => Self::render_two_chars(px, py, "▶", pixmap, font, scale, color),
            Self::KC_MEDIA_PREV => Self::render_three_chars(px, py, "prev", pixmap, font, scale, color),
            Self::KC_MEDIA_NEXT => Self::render_three_chars(px, py, "next", pixmap, font, scale, color),

            Self::KC_MINUS => Self::render_simple(px, py, "-", pixmap, font, scale, color),
            Self::KC_GRV => Self::render_simple(px, py, "`", pixmap, font, scale, color),
            Self::KC_QUOT => Self::render_simple(px, py, "'", pixmap, font, scale, color),
            Self::KC_BSLS => Self::render_simple(px, py, "\\", pixmap, font, scale, color),
            Self::KC_LBRC => Self::render_simple(px, py, "[", pixmap, font, scale, color),
            Self::KC_RBRC => Self::render_simple(px, py, "]", pixmap, font, scale, color),
            Self::KC_EQUAL => Self::render_simple(px, py, "=", pixmap, font, scale, color),
            Self::KC_COMM => Self::render_simple(px, py, ",", pixmap, font, scale, color),
            Self::KC_DOT => Self::render_simple(px, py, ".", pixmap, font, scale, color),
            Self::KC_SLASH => Self::render_simple(px, py, "/", pixmap, font, scale, color),
            
            Self::MOD_LCTL => Self::render_three_chars(px, py, "Ctl", pixmap, font, scale, color),
            Self::KC_BSPC =>  Self::render_three_chars(px, py, "Bsp", pixmap, font, scale, color),
            Self::KC_LGUI =>  Self::render_three_chars(px, py, "Mod", pixmap, font, scale, color),
            Self::KC_SPACE => Self::render_simple(px, py, "␣", pixmap, font, scale, color),
            Self::MOD_LALT => Self::render_three_chars(px, py, "Alt", pixmap, font, scale, color),
            Self::KC_TAB =>   Self::render_three_chars(px, py, "Tab", pixmap, font, scale, color),
            Self::KC_PGUP =>  Self::render_three_chars(px, py, "PgU", pixmap, font, scale, color),
            Self::KC_PGDN =>  Self::render_three_chars(px, py, "PgD", pixmap, font, scale, color),
            Self::KC_HOME =>  Self::render_three_chars(px, py, "Hom", pixmap, font, scale, color),
            Self::KC_END =>   Self::render_three_chars(px, py, "End", pixmap, font, scale, color),
            Self::KC_ENT =>   Self::render_three_chars(px, py, "Ent", pixmap, font, scale, color),
            Self::KC_ESC =>   Self::render_three_chars(px, py, "Esc", pixmap, font, scale, color),
            Self::KC_PSCR =>  Self::render_three_chars(px, py, "PSc", pixmap, font, scale, color),


            Self::S(inner) => {
                match **inner {
                    Self::KC_A => Self::render_simple(px, py, "A", pixmap, font, scale, color),
                    Self::KC_B => Self::render_simple(px, py, "B", pixmap, font, scale, color),
                    Self::KC_C => Self::render_simple(px, py, "C", pixmap, font, scale, color),
                    Self::KC_D => Self::render_simple(px, py, "D", pixmap, font, scale, color),
                    Self::KC_E => Self::render_simple(px, py, "E", pixmap, font, scale, color),
                    Self::KC_F => Self::render_simple(px, py, "F", pixmap, font, scale, color),
                    Self::KC_G => Self::render_simple(px, py, "G", pixmap, font, scale, color),
                    Self::KC_H => Self::render_simple(px, py, "H", pixmap, font, scale, color),
                    Self::KC_I => Self::render_simple(px, py, "I", pixmap, font, scale, color),
                    Self::KC_J => Self::render_simple(px, py, "J", pixmap, font, scale, color),
                    Self::KC_K => Self::render_simple(px, py, "K", pixmap, font, scale, color),
                    Self::KC_L => Self::render_simple(px, py, "L", pixmap, font, scale, color),
                    Self::KC_M => Self::render_simple(px, py, "M", pixmap, font, scale, color),
                    Self::KC_N => Self::render_simple(px, py, "N", pixmap, font, scale, color),
                    Self::KC_O => Self::render_simple(px, py, "O", pixmap, font, scale, color),
                    Self::KC_P => Self::render_simple(px, py, "P", pixmap, font, scale, color),
                    Self::KC_Q => Self::render_simple(px, py, "Q", pixmap, font, scale, color),
                    Self::KC_R => Self::render_simple(px, py, "R", pixmap, font, scale, color),
                    Self::KC_S => Self::render_simple(px, py, "S", pixmap, font, scale, color),
                    Self::KC_T => Self::render_simple(px, py, "T", pixmap, font, scale, color),
                    Self::KC_U => Self::render_simple(px, py, "U", pixmap, font, scale, color),
                    Self::KC_V => Self::render_simple(px, py, "V", pixmap, font, scale, color),
                    Self::KC_W => Self::render_simple(px, py, "W", pixmap, font, scale, color),
                    Self::KC_X => Self::render_simple(px, py, "X", pixmap, font, scale, color),
                    Self::KC_Y => Self::render_simple(px, py, "Y", pixmap, font, scale, color),
                    Self::KC_Z => Self::render_simple(px, py, "Z", pixmap, font, scale, color),

                    Self::KC_0 => Self::render_simple(px, py, ")", pixmap, font, scale, color),
                    Self::KC_1 => Self::render_simple(px, py, "!", pixmap, font, scale, color),
                    Self::KC_2 => Self::render_simple(px, py, "@", pixmap, font, scale, color),
                    Self::KC_3 => Self::render_simple(px, py, "#", pixmap, font, scale, color),
                    Self::KC_4 => Self::render_simple(px, py, "$", pixmap, font, scale, color),
                    Self::KC_5 => Self::render_simple(px, py, "%", pixmap, font, scale, color),
                    Self::KC_6 => Self::render_simple(px, py, "^", pixmap, font, scale, color),
                    Self::KC_7 => Self::render_simple(px, py, "&", pixmap, font, scale, color),
                    Self::KC_8 => Self::render_simple(px, py, "*", pixmap, font, scale, color),
                    Self::KC_9 => Self::render_simple(px, py, "(", pixmap, font, scale, color),

                    Self::KC_MINUS => Self::render_simple(px, py, "_", pixmap, font, scale, color),
                    Self::KC_GRV => Self::render_simple(px, py, "~", pixmap, font, scale, color),
                    Self::KC_QUOT => Self::render_simple(px, py, "\"", pixmap, font, scale, color),
                    Self::KC_BSLS => Self::render_simple(px, py, "|", pixmap, font, scale, color),
                    Self::KC_LBRC => Self::render_simple(px, py, "{", pixmap, font, scale, color),
                    Self::KC_RBRC => Self::render_simple(px, py, "}", pixmap, font, scale, color),
                    Self::KC_EQUAL => Self::render_simple(px, py, "+", pixmap, font, scale, color),
                    Self::KC_COMM => Self::render_simple(px, py, "<", pixmap, font, scale, color),
                    Self::KC_DOT => Self::render_simple(px, py, ">", pixmap, font, scale, color),
                    Self::KC_SLASH => Self::render_simple(px, py, "?", pixmap, font, scale, color),

                    _ => Self::render_two_chars(px, py, "<>", pixmap, font, scale, color),
                }
            },
            Self::MO(layer) => {
                match layer.as_str() {
                    "_SHIFT" => Self::render_simple(px, py, "↑", pixmap, font, scale, color),
                    "_NUMBERS" => Self::render_three_chars(px, py, "123", pixmap, font, scale, color),
                    "_SYMBOLS" => Self::render_three_chars(px, py, "#+=", pixmap, font, scale, color),
                    _ => {}
                };
            }

            Self::LT(layer, key) => {
                match layer.as_str() {
                    "_SHIFT" => Self::render_simple(px, py - 10.0, "↑", pixmap, font, scale, color),
                    "_NUMBERS" => Self::render_three_chars(px, py - 10.0, "123", pixmap, font, scale, color),
                    "_SYMBOLS" => Self::render_three_chars(px, py - 10.0, "#+=", pixmap, font, scale, color),
                    _ => {}
                };
                key.render(px, py + 15.0, pixmap, font, scale, color);
            }
            Self::MT(hold, tap) => {
                hold.render(px, py - 10.0, pixmap, font, scale, color);
                tap.render(px, py + 15.0, pixmap, font, scale, color);
            }
        };
        return true;
    }

    //for single character keys
    fn render_simple(px: f32, py: f32, text: &str, pixmap: &mut Pixmap, font: &FontArc, scale: PxScale, color: Color) {
        draw_text(pixmap, text, font, scale, px + 17.0, py + 17.0, color);
    }

    fn render_two_chars(px: f32, py: f32, text: &str, pixmap: &mut Pixmap, font: &FontArc, scale: PxScale, color: Color) {
        draw_text(pixmap, text, font, scale, px + 5.0, py + 17.0, color);
    }

    fn render_three_chars(px: f32, py: f32, text: &str, pixmap: &mut Pixmap, font: &FontArc, scale: PxScale, color: Color) {
        let new_scale = PxScale { x: scale.x * 0.8, y: scale.y * 0.8 };
        draw_text(pixmap, text, font, new_scale, px + 1.0, py + 17.0, color);
    }
}
