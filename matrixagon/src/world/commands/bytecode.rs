/*
The World Command Bytecode

This is the internal command language for world commands.
 */

use std::fs;
use std::path::Path;
use std::char;

type TokenErrorRes = Result<(), TokenError>;

const COMMENT_LITERAL: char = ';';
const STRING_LITERAL: char = '"';
const STATIC_VAR: char = '$';
const MARKER: char = '#';


#[derive(PartialEq, Debug)]
pub enum TokenError {
    // Theres part of a string identified as a arguments in a command section
    ArgumentInCommandSection(u32),  // line number
    // Theres a missing space either before a starting double-quote mark or after a ending double-quote mark
    MissingSpaceAroundStringQuotes(u32, bool),  // line number, false: before; true: after
    // A string must be properly closed before the end of line (EOL)
    StringsNotEnclosedAfterEOL(u32),  // line number
    // There must not be any whitespaces before any command
    WhitespacesBeforeCommands(u32),  // line number
    // The command name must be valid; read the documentation on what commands are available and what are their usage
    InvalidCommandName(u32, String),  // line number, the invalid command name
    // The command name can and must only contain A-Z and _ (underscore), anything else is an error
    CommandNameInvalidCharacters(u32, char),  // line number, the invalid character
    // The argument values contains some invalid characters, mainly those of '#', '$', and ':'
    ArgumentValInvalidCharacters(u32, char),  // line number, the invalid character
    // The namespace argument value contains empty namespaces
    NamespaceEmpty(u32), // line number
    // There are only a limited number of argument types; The argument type was invalid or invalid usage of the argument
    InvalidArgumentTypes(u32),  // line number
    // The number had some problems
    InvalidNumber(u32),  // line number
    // The decimal had some problems
    InvalidDecimal(u32),  // line number
    // TODO: remove it later? should theoretically not happen.
    EmptyArguments(u32),  // line number
}

#[derive(PartialEq)]
enum FirstLiterals {
    None,
    Comment,
    String,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ConstValType {
    None,
    StaticVar,
    Marker,
    Namespace,
    // Str,  // String is already handled by the tokenizer
    Int,
    Float,
}

// compiles a multiple lines of commands down to computer readable tokens
pub (super) fn compile_command(char_stream: Vec<char>) -> Result<Vec<Vec<Tokens>>, TokenError> {
    println!("File character stream: {:?}", char_stream);

    let tokens = bytecode_tokenizer(char_stream);

    tokens
}

// compiles the file down to computer readable tokens
pub (super) fn compile_file(fname: String) -> Result<Vec<Vec<Tokens>>, TokenError> {
    // Read File: reads each file into a character stream
    // numeric byte stream
    let byte_stream = fs::read(Path::new(&fname)).expect(&format!("Command file '{}' not found!", fname)[..]);
    // TODO: assume its a text file for now
    // convert it too a character stream
    let char_stream;
    unsafe {
        char_stream = byte_stream.iter().map(|c|char::from_u32_unchecked(*c as u32)).collect::<Vec<_>>();
    };

    println!("File character stream: {:?}", char_stream);

    let tokens = bytecode_tokenizer(char_stream);

    println!("File compilation successfully converted bytecode file to tokens!");

    tokens
}

// TODO: when tokenizing, save each line's line number in Vec so we have an accurate repr of the line # as the tokenizer eliminates lines and to improve error correction
// reads the file and returns a formatted tokens
fn bytecode_tokenizer(char_stream: Vec<char>) -> Result<Vec<Vec<Tokens>>, TokenError> {
    let mut tkn_err: TokenErrorRes = Result::Ok(());

    println!("\n ****** Raw file data ****** ");
    for c in char_stream.clone() { print!("{}", c); }

    // Pre-Processing:
    //      removes comments,
    //      then removes any trailing spaces,
    //      then remove any empty lines,
    //      then compress all whitespaces into a single space,

    // compressed chr stream
    let mut compr_char_stream: Vec<char> = Vec::new();

    let mut first_literals = FirstLiterals::None;
    let mut empty_space = false;  // checks for multiple empty spaces; compresses all the whitespace into a single space
    let mut empty_lines = true;  // checks if the current line contains nothing or just whitespaces and comments

    for c in char_stream {
        match c {
            COMMENT_LITERAL => {
                if first_literals == FirstLiterals::None {
                    first_literals = FirstLiterals::Comment;
                }
                if first_literals != FirstLiterals::Comment {
                    if empty_space {
                        compr_char_stream.push(' ');
                        empty_space = false;
                    }
                    compr_char_stream.push(c);
                    empty_lines = false;
                }
            },
            STRING_LITERAL => {
                if first_literals == FirstLiterals::None {
                    first_literals = FirstLiterals::String;
                } else if first_literals == FirstLiterals::String {
                    first_literals = FirstLiterals::None;
                }
                if first_literals != FirstLiterals::Comment {
                    if empty_space {
                        compr_char_stream.push(' ');
                        empty_space = false;
                    }
                    compr_char_stream.push(c);
                    empty_lines = false;
                }
            },
            '\n' => {
                if !empty_lines {
                    compr_char_stream.push('\n');
                }
                first_literals = FirstLiterals::None;
                empty_space = false;
                empty_lines = true;
            },
            ' ' | '\t' | '\r' => {
                empty_space = true;
            },
            _ => {
                if first_literals != FirstLiterals::Comment {
                    if empty_space {
                        compr_char_stream.push(' ');
                        empty_space = false;
                    }
                    compr_char_stream.push(c);
                    empty_lines = false;
                }
            },
        }
    }

    println!("\n ****** Compressed file data ****** ");
    for c in compr_char_stream.clone() { print!("{}", c); }

    // Tokenizer:
    //     tokenizes each parts of a text into valid rust type
    //     only tokenizes into: Commands, String Literal Arguments, and Pseudo-Arguments

    // [commands, arguments, arguments, arguments...]
    let mut raw_tokens: Vec<Vec<Tokens>> = Vec::new();
    let mut tknl_buf: Vec<Tokens> = Vec::new();  // buffer for the current line of tokens
    let mut tkni_buf = String::new();  // buffer for the current individual token

    let mut line_no = 1u32;  // line number
    let mut cmd_sect = true;  // true: currently in command section, false: currently in arguments section
    let mut in_string = false;  // currently within a string enclosure
    let mut first_char = true;  // first character of the file or the character after line feed
    let mut space_bef = false;  // is there a space before; checking for spaces around string
    let mut str_quote_bef = false;  // is there a double string quote ending before; checking for spaces around string

    for c in compr_char_stream {
        match c {
            STRING_LITERAL => {
                if cmd_sect {
                    tkn_err = Err(TokenError::ArgumentInCommandSection(line_no));
                    break;
                }
                if in_string {
                    tknl_buf.push(Tokens::Argument(Arguments::Values(ValType::Str(tkni_buf.clone()))));
                    tkni_buf.clear();
                    in_string = false;
                    str_quote_bef = true;
                    continue;
                } else {
                    // checks if there are spaces before
                    if space_bef {
                        in_string = true;
                    } else {
                        tkn_err = Err(TokenError::MissingSpaceAroundStringQuotes(line_no, false));
                        break;
                    }
                }
            },
            '\n' => {
                if in_string {
                    tkn_err = Err(TokenError::StringsNotEnclosedAfterEOL(line_no));
                    break;
                } else if cmd_sect {
                    // to add commands that are not added through space such as argumentless commands
                    if let Ok(cmd) = command_name_conv(tkni_buf.as_str(), line_no) {
                        tknl_buf.push(Tokens::Command(cmd));
                        tkni_buf.clear();
                    } else if let Err(err) = command_name_conv(tkni_buf.as_str(), line_no) {
                        tkn_err = Err(err);
                        break;
                    }
                    cmd_sect = false;
                } else if !cmd_sect {
                    // also to add arguments at EOL
                    if !tkni_buf.is_empty() {
                        if let Ok(arg) = arguments_str_eval(tkni_buf.as_str(), line_no) {
                            tknl_buf.push(Tokens::Argument(arg));
                            tkni_buf.clear();
                        } else if let Err(err) = arguments_str_eval(tkni_buf.as_str(), line_no) {
                            tkn_err = Err(err);
                            break;
                        }
                    }
                }
                raw_tokens.push(tknl_buf.clone());
                tknl_buf.clear();
                tkni_buf.clear();

                cmd_sect = true;
                first_char = true;
                space_bef = false;
                str_quote_bef = false;
                line_no += 1;
                continue;  // to prevent it reset the first_char
            },
            ' ' => {
                if first_char {
                    tkn_err = Err(TokenError::WhitespacesBeforeCommands(line_no));
                    break;
                }
                if cmd_sect {
                    if let Ok(cmd) = command_name_conv(tkni_buf.as_str(), line_no) {
                        tknl_buf.push(Tokens::Command(cmd));
                        tkni_buf.clear();
                    } else if let Err(err) = command_name_conv(tkni_buf.as_str(), line_no) {
                        tkn_err = Err(err);
                        break;
                    }
                    cmd_sect = false;
                } else {  // arguments section
                    if !in_string {
                        if let Ok(arg) = arguments_str_eval(tkni_buf.as_str(), line_no) {
                            tknl_buf.push(Tokens::Argument(arg));
                            tkni_buf.clear();
                        } else if let Err(err) = arguments_str_eval(tkni_buf.as_str(), line_no) {
                            tkn_err = Err(err);
                            break;
                        }
                    } else {
                        tkni_buf.push(' ');
                    }
                }
                space_bef = true;
                continue;
            },
            'A'..='Z' | '_' => {  // these are the all the valid characters of a command
                if str_quote_bef {
                    tkn_err = Err(TokenError::MissingSpaceAroundStringQuotes(line_no, true));
                    break;
                }
                if cmd_sect {
                    tkni_buf.push(c);
                } else {
                    tkni_buf.push(c);
                }
            },
            _ => {
                if str_quote_bef {
                    tkn_err = Err(TokenError::MissingSpaceAroundStringQuotes(line_no, true));
                    break;
                }
                if cmd_sect {
                    tkn_err = Err(TokenError::CommandNameInvalidCharacters(line_no, c));
                    break;
                }
                tkni_buf.push(c);
            },
        };
        space_bef = false;
        str_quote_bef = false;
        first_char = false;
    }

    println!("\n ****** Computer-readble Tokens ****** ");
    for tkn in raw_tokens.clone() { println!("{:?}", tkn); }

    match tkn_err {
        Ok(_) => Ok(raw_tokens),
        Err(e) => Err(e),
    }
}

fn arguments_str_eval(arg_name: &str, cur_line_no: u32) -> Result<Arguments, TokenError>{
    // Individual Arguments Value Type Identifier

    let mut interp_type = ConstValType::None;  // the interpreted type for the argument str
    let mut arg_str: Vec<String> = Vec::new();
    arg_str.push(String::new());

    let mut first_char = true;

    for c in arg_name.chars() {
        match c {
            STATIC_VAR => {  // static variable; globally defined
                if first_char {
                    interp_type = ConstValType::StaticVar;
                } else {
                    return Err(TokenError::ArgumentValInvalidCharacters(cur_line_no, c));
                }
            },
            MARKER => {  // marker tag; globally defined
                if first_char {
                    interp_type = ConstValType::Marker;
                } else {
                    return Err(TokenError::ArgumentValInvalidCharacters(cur_line_no, c));
                }
            },
            ':' => {  // external namespace operator/prefixes; prefix in this case
                if first_char {
                    interp_type = ConstValType::Namespace;
                } else if interp_type == ConstValType::Namespace {
                    // only an external namespace can contain a ':' within
                    if arg_str.last().unwrap().is_empty() {
                        return Err(TokenError::NamespaceEmpty(cur_line_no));
                    }

                    arg_str.push(String::new());
                } else {
                    return Err(TokenError::ArgumentValInvalidCharacters(cur_line_no, c));
                }
            },
            '-' | '0'..='9' => {  // 10-based integers with sign
                if first_char {
                    interp_type = ConstValType::Int;
                }
                arg_str.last_mut().unwrap().push(c);
            },
            '.' => {  // additional floats
                if first_char || interp_type == ConstValType::Int {
                    // upgrades the integer to floats once the tokenizer later finds it contains a dot '.' (decimal point)
                    interp_type = ConstValType::Float;
                }
                arg_str.last_mut().unwrap().push(c);
            },
            _ => {
                if first_char {
                    return Err(TokenError::InvalidArgumentTypes(cur_line_no));
                }
                arg_str.last_mut().unwrap().push(c);
            },
        }
        first_char = false;
    }

    if arg_str.is_empty() {
        return Err(TokenError::EmptyArguments(cur_line_no));
    }

    // Individual Arguments Value Parser
    // note: unwrapping for the first value of the arg_str is because there will always be a first value
    let arg_val: Option<Arguments> = match interp_type {
        ConstValType::None => None,
        ConstValType::StaticVar => Some(Arguments::StaticVar(arg_str.first().unwrap().clone())),
        ConstValType::Marker => Some(Arguments::Marker(arg_str.first().unwrap().clone())),
        ConstValType::Namespace => Some(Arguments::Namespace(arg_str.clone())),
        ConstValType::Int => {
            // parsing note: integer in the bytecode refers to isize, not usize
            // TODO: Handle big numbers?
            let val = arg_str.first().unwrap().clone();
            match val.parse::<i64>() {
                Result::Ok(val) => {
                    Some(Arguments::Values(ValType::Int(val)))
                },
                Result::Err(_e) => {
                    println!("Integer parser error: o-val: {:?}", val);
                    return Err(TokenError::InvalidNumber(cur_line_no));
                },
            }
        },
        ConstValType::Float => {
            // TODO: Handle precisions?
            let val = arg_str.first().unwrap().clone();
            match val.parse::<f64>() {
                Result::Ok(val) => {
                    Some(Arguments::Values(ValType::Float(val)))
                },
                Result::Err(_e) => {
                    println!("Float parser error: o-val: {:?}", val);
                    return Err(TokenError::InvalidDecimal(cur_line_no));
                },
            }
        },
    };

    Ok(arg_val.expect("The argument resulted a `None`"))
}

// converts a command text into a command name
fn command_name_conv(cmd_name: &str, cur_line_no: u32) -> Result<Commands, TokenError> {
    match cmd_name {
        "PUSH"      => Ok(Commands::Push),
        "STATIC"    => Ok(Commands::Static),
        "NAMESPACE" => Ok(Commands::Namespace),
        "INCL"      => Ok(Commands::Include),
        "POP"       => Ok(Commands::Pop),

        "COUT"      => Ok(Commands::COut),
        "CIN"       => Ok(Commands::CIn),
        "CMD_COPY"  => Ok(Commands::CmdCopy),
        "CMD_MOVE"  => Ok(Commands::CmdMove),
        "RET"       => Ok(Commands::Ret),
        "EVNT"      => Ok(Commands::Event),

        "PACK"      => Ok(Commands::Pack),
        "UNPK"      => Ok(Commands::Unpack),
        "ROT_TWO"   => Ok(Commands::RotTwo),
        "ROT_THREE" => Ok(Commands::RotThree),
        "ROT_FOUR"  => Ok(Commands::RotFour),
        "JMP_IF"    => Ok(Commands::JmpIf),
        "JMP"       => Ok(Commands::Jmp),
        "MRK"       => Ok(Commands::Mrk),

        "ADD"       => Ok(Commands::Add),
        "SUB"       => Ok(Commands::Sub),
        "MUL"       => Ok(Commands::Mul),
        "DIV"       => Ok(Commands::Div),
        "MOD"       => Ok(Commands::Mod),
        "SHL"       => Ok(Commands::ShL),
        "SHR"       => Ok(Commands::ShR),
        "AND"       => Ok(Commands::And),
        "OR"        => Ok(Commands::Or),
        "XOR"       => Ok(Commands::Xor),
        "NOT"       => Ok(Commands::Not),
        "NEG"       => Ok(Commands::Neg),

        "ANDL"      => Ok(Commands::AndL),
        "ORL"       => Ok(Commands::OrL),
        "XORL"      => Ok(Commands::XorL),
        "NOTL"      => Ok(Commands::NotL),

        "ADDI"      => Ok(Commands::AddI),
        "SUBI"      => Ok(Commands::SubI),
        "MULI"      => Ok(Commands::MulI),
        "DIVI"      => Ok(Commands::DivI),
        "MODI"      => Ok(Commands::ModI),
        "ANDI"      => Ok(Commands::AndI),
        "ORI"       => Ok(Commands::OrI),
        "XORI"      => Ok(Commands::XorI),

        _ => {
            Err(TokenError::InvalidCommandName(cur_line_no, String::from(cmd_name)))
        }
    }
}

// All the tokens after the pre-processing
#[derive(Clone, PartialEq, Debug)]
pub (super) enum Tokens {
    Command(Commands),
    Argument(Arguments),
    PseudoArguments(String),  // NOTE: DO NOT USE THIS FOR FINAL PRODUCT, THIS IS FOR INTERMEDIATE REPRESENTATION
}

// All the possible arguments
#[derive(Clone, PartialEq, Debug)]
pub (super) enum Arguments {
    Values(ValType),
    StaticVar(String),  // Static variables with its name
    Marker(String),  // Marker with its marker name
    Namespace(Vec<String>),  // Vectors of each namespace names ordered from left to right delimited by ':'
}

// Types that only exists on arguments immediate
#[derive(Clone, PartialEq, Debug)]
pub (super) enum ValType {
    Str(String), // String
    Int(i64), // Integer
    Float(f64), // Float
}

// Types that only exists on stacks
#[derive(Clone, PartialEq, Debug)]
pub (super) enum StackType {
    Str(String), // String
    Int(u64), // Integer
    Float(f64), // Float
    List(u16), // Length of the lists
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub (super) enum Commands {
    Push,
    Static,
    Namespace,
    Include,
    Pop,

    COut,
    CIn,
    CmdCopy,
    CmdMove,
    Ret,
    Event,

    Pack,
    Unpack,
    RotTwo,
    RotThree,
    RotFour,
    JmpIf,
    Jmp,
    Mrk,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    ShL,
    ShR,
    And,
    Or,
    Xor,
    Not,
    Neg,

    AndL,
    OrL,
    XorL,
    NotL,

    AddI,
    SubI,
    MulI,
    DivI,
    ModI,
    AndI,
    OrI,
    XorI,
}
