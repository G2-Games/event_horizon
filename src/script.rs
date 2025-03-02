use log::debug;
use macroquad::math::Vec2;
use std::{collections::HashMap, fmt::Display, fs, iter::Peekable, path::{Path, PathBuf}, str::Chars, time::{Duration, Instant}};

use crate::{load_image, CharacterSprite, NovelState, SelectMenu, TextBox};

const COMMENT: &str = "#";
const DEFINE: &str = "DEFINE";
const LABEL: &str = "LABEL";
const SELECT: &str = "SELECT";
const IMAGE: &str = "IMG";
const TEXTBOX: &str = "TEXTBOX";
const JUMP: &str = "JUMP";
const IF: &str = "IF";
const LOAD: &str = "LOAD";
const DELAY: &str = "DELAY";

#[derive(Debug)]
pub struct Script {
    variable_map: HashMap<String, String>,
    jump_map: HashMap<String, (usize, PathBuf)>,
    lines: Vec<String>,
    index: usize,
}

impl Script {
    pub fn load_script<P: AsRef<Path>>(path: P) -> Script {
        let path_ref = path.as_ref();
        let script_txt = fs::read_to_string(path_ref).unwrap();

        let jump_map = pre_parse(&script_txt, path_ref);

        let lines = script_txt.lines().map(|s| s.to_string()).collect();

        Script {
            variable_map: HashMap::new(),
            jump_map,
            lines,
            index: 0,
        }
    }

    /// Load a new script and append it to the end of the current one in memory
    pub fn load_into(&mut self, path: &Path) {
        let script_txt = fs::read_to_string(path).unwrap();

        let jump_map = pre_parse(&script_txt, path);

        let lines: Vec<String> = script_txt.lines().map(|s| s.to_string()).collect();

        self.lines.extend_from_slice(&lines);
        self.jump_map.extend(jump_map);
    }

    pub fn insert_variable(&mut self, var_name: String, value: String) {
        self.variable_map.insert(var_name, value);
    }

    pub fn next_instruction(&mut self, state: &mut NovelState) {
        loop {
            if self.index == self.lines.len() {
                break;
            }

            let line = self.lines[self.index].clone();
            self.index += 1;

            let line = read_tokens(&line);

            if let Some(Token::Expression(c)) = line.tokens.get(0) {
                // Skip comments

                if c.as_str().starts_with(COMMENT) {
                    continue;
                }
            } else {
                // Line is blank, skip it
                continue;
            }

            match line.tokens.first().unwrap().to_string().as_str() {
                DEFINE => {
                    debug!(
                        "{}: {:>8} = {} (\"{}\")",
                        DEFINE,
                        line.tokens[1].to_string(),
                        line.tokens[2],
                        line.tokens[2].load_self_map(&self.variable_map).unwrap().to_string().as_str(),
                    );

                    self.variable_map.insert(
                        line.tokens[1].to_string(),
                        line.tokens[2].load_self_map(&self.variable_map).unwrap(),
                    );
                }
                DELAY => {
                    let delay = line.tokens[1].to_f32().unwrap();
                    let dur = Duration::from_secs_f32(delay);
                    let future = Instant::now();

                    debug!("DELAY: {:?}", dur);

                    state.delay = Some((future, dur));

                    return;
                }
                SELECT => {
                    let mut options = Vec::new();
                    loop {
                        let newline = read_tokens(&self.lines[self.index]);
                        if newline.indent <= line.indent {
                            break;
                        }

                        options.push((newline.tokens[0].to_string(), newline.tokens[1].to_string()));

                        self.index += 1;
                    }

                    debug!("{}: {:?}", SELECT, options);

                    state.select_menu = Some(SelectMenu {
                        selected: 0,
                        variable: line.tokens[1].to_string().clone(),
                        options,
                    });

                    return
                }
                IF => {
                    let condition_1 = line.tokens[1].load_self_map(&self.variable_map);
                    let condition_2 = line.tokens[2].load_self_map(&self.variable_map);

                    if condition_2 != condition_1 {
                        // Skip all the stuff inside the IF
                        loop {
                            let newline = read_tokens(&self.lines[self.index]);
                            if newline.indent <= line.indent {
                                break;
                            }

                            self.index += 1;
                        }
                    }
                }
                IMAGE => {
                    let path = line.tokens[2].load_self_map(&self.variable_map).unwrap();
                    match line.tokens[1].to_string().as_str() {
                        "BG" => {
                            debug!("IMG BG: {:>8}", line.tokens[2].clone());
                            state.background = load_image(&path.to_string());
                        }
                        "CHAR" => {
                            debug!("IMG CHAR: {:>8}", path.to_string());

                            state.characters.push(CharacterSprite {
                                name: path.to_string(),
                                texture: load_image(&path.to_string()).unwrap(),
                                position: Vec2::new(700., 100.),
                                saturation: 1.0,
                                flip: false,
                            });
                        }
                        "CHAR2" => {
                            debug!("IMG CHAR2: {:>8}", path.to_string());

                            state.characters.push(CharacterSprite {
                                name: path.to_string(),
                                texture: load_image(&path.to_string()).unwrap(),
                                position: Vec2::new(200., 100.),
                                saturation: 1.0,
                                flip: true,
                            });
                        },
                        "CLEAR" => {
                            if let Some(pos) = state.characters.iter().position(|c| c.name == path.to_string()) {
                                state.characters.remove(pos);
                            }
                        }
                        _ => panic!("Invalid option"),
                    }
                }
                TEXTBOX => {
                    if line.tokens.len() == 1 {
                        state.textbox = None;
                        continue;
                    }

                    let character_name = line.tokens[1].load_self_map(&self.variable_map).unwrap();
                    debug!("TEXTBOX: {}, \"{}\"", character_name, line.tokens[2]);

                    state.textbox = Some(TextBox {
                        name: character_name,
                        dialogue: line.tokens[2].to_string().clone().lines().map(|l| l.to_string()).collect(),
                        current_line: 0,
                        current_char: 0,
                    });

                    if !line.tokens.get(3).is_some_and(|t| t.to_string().as_str() == "SELECT") {
                        return
                    }
                }
                JUMP => {
                    debug!("JUMP: {}", line.tokens[1]);
                    let label_point = line.tokens[1].to_string();
                    self.index = self.jump_map.get(&label_point).unwrap().0 - 1;
                }
                LOAD => {
                    let path = line.tokens[1].to_string();
                    debug!("LOAD: {}", path);
                    self.load_into(&PathBuf::from(path));
                }
                LABEL => (),
                _ => (),
            }
        }
    }
}

fn pre_parse<P: AsRef<Path>>(script: &str, path: P) -> HashMap<String, (usize, PathBuf)> {
    let mut jump_map = HashMap::new();

    for (index, line) in script.lines().enumerate() {
        let line = read_tokens(line);
        if line.tokens.is_empty() {
            continue;
        }

        if line.tokens[0].to_string() == "LABEL" {
            jump_map.insert(line.tokens[1].to_string(), (index, path.as_ref().to_path_buf()));
        }
    }

    jump_map
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token {
    String(String),
    Expression(String),
}

impl Token {
    /// Loads either a literal string or a variable
    fn load_self_map(&self, var_map: &HashMap<String, String>) -> Option<String> {
        match self {
            Token::String(s) => Some(s.clone()),
            Token::Expression(e) => var_map.get(e).cloned(),
        }
    }

    fn to_f32(&self) -> Option<f32> {
        self.to_string().parse::<f32>().ok()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Token::String(s) => s,
            Token::Expression(s) => s,
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug)]
struct ScriptLine {
    tokens: Vec<Token>,
    indent: u8,
}

fn read_tokens(line: &str) -> ScriptLine {
    let mut tokens = Vec::new();

    let line = line.replace('’', "\'").replace('…', "...");

    let mut char_indices = line.chars().peekable();
    let mut indent = 0;
    let mut encountered_non_empty = false;
    while let Some(ch) = char_indices.peek() {
        let token = match ch {
            '"' => {
                if let Some(t) = read_string(&mut char_indices) {
                    Token::String(t)
                } else {
                    continue;
                }
            }
            _ => Token::Expression(read_arguments(&mut char_indices)),
        };

        if token.to_string().is_empty() && !encountered_non_empty {
            indent += 1;
        } else if !token.to_string().is_empty() {
            encountered_non_empty = true;
            tokens.push(token);
        }
    }

    ScriptLine { tokens, indent }
}

fn read_string(line: &mut Peekable<Chars>) -> Option<String> {
    let mut final_string = String::new();

    let mut started = false;
    let mut escaped = None;
    let mut space_index = 0;
    let mut last_index = 0;
    for (index, c) in line.enumerate() {
        match c {
            '\n' => return None,
            '\\' if escaped.is_none() => escaped = Some(index),
            '\\' if escaped.is_some() => escaped = None,
            '"' if !started && escaped.is_none() => started = true,
            '"' if started && escaped.is_none() => break,
            _ if started => final_string.push(c),
            _ => (),
        }

        if c == ' ' {
            space_index = index
        }

        if index - last_index == 39 && space_index != 0 {
            //dbg!(space_index, final_string.len(), &final_string, &final_string[space_index - 1..space_index]);
            if final_string.is_char_boundary(space_index - 1) {
                final_string.replace_range(space_index - 1..space_index, "\n");
            }
            last_index = index
        }

        if let Some(e) = escaped {
            if index.abs_diff(e - 1) > 1 {
                escaped = None
            }
        }
    }

    if !final_string.is_empty() {
        Some(final_string)
    } else {
        None
    }
}

fn read_arguments(line: &mut Peekable<Chars>) -> String {
    let mut final_string = String::new();

    for c in line {
        if !c.is_whitespace() {
            final_string.push(c);
        } else {
            break;
        }
    }

    final_string
}
