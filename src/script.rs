use log::debug;
use std::{collections::HashMap, fmt::Display, fs, iter::{Enumerate, Peekable}, path::{Path, PathBuf}, str::{Chars, Lines}};

use macroquad::prelude::*;

const COMMENT: &str = "#";
const DEFINE: &str = "DEFINE";
const LABEL: &str = "LABEL";
const SELECT: &str = "SELECT";
const IMAGE: &str = "IMG";
const TEXTBOX: &str = "TEXTBOX";
const JUMP: &str = "JUMP";

#[derive(Debug)]
pub struct Script<P: AsRef<Path>> {
    path: P,
    variable_map: HashMap<String, String>,
    jump_map: HashMap<String, (usize, PathBuf)>,
    lines: Vec<String>,
    index: usize,
}

impl<P: AsRef<Path>> Script<P> {
    pub fn load_script(path: P) -> Script<P> {
        let path_ref = path.as_ref();
        let script_txt = fs::read_to_string(path_ref).unwrap();

        let jump_map = pre_parse(&script_txt, path_ref);

        let lines = script_txt.lines().map(|s| s.to_string()).collect();

        Script {
            path,
            variable_map: HashMap::new(),
            jump_map,
            lines,
            index: 0,
        }
    }

    pub fn next_instruction(&mut self) {
        let line = self.lines[self.index].clone();
        self.index += 1;

        let line = read_tokens(&line);

        if let Some(Token::Expression(c)) = line.tokens.get(0) {
            // Skip comments

            if c.as_str().starts_with(COMMENT) {
                return;
            }
        } else {
            // Line is blank, skip it
            return;
        }

        clear_background(Color::from_hex(0x230000));

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
            SELECT => {
                let mut options = HashMap::new();
                loop {
                    let newline = read_tokens(&self.lines[self.index + 1]);
                    if newline.indent <= line.indent {
                        break;
                    }

                    options.insert(newline.tokens[0].to_string(), newline.tokens[1].to_string());

                    self.index += 1;
                }
                debug!("{}: {:#?}", SELECT, options);
            }
            IMAGE => {
                let path = line.tokens[2].clone();
                match line.tokens[1].to_string().as_str() {
                    "BG" => debug!("IMG BG: {:>8}", path.to_string()),
                    "CHAR" => debug!("IMG CHAR: {:>8}", path.to_string()),
                    _ => panic!(),
                }
            }
            TEXTBOX => {
                let character_name = line.tokens[1].load_self_map(&self.variable_map).unwrap();
                debug!("TEXTBOX: {}, \"{}\"", character_name, line.tokens[2]);
            }
            JUMP => {
                let label_point = line.tokens[1].to_string();
                self.index = self.jump_map.get(&label_point).unwrap().0 - 1;
            }
            LABEL => (),
            _ => (),
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

pub trait Runnable {
    fn run(&self);
}
