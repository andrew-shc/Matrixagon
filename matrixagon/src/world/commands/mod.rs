/*
The World Command Executor

A place where all the command bytecodes gets executed.
 */

use std::fs;
use std::path::Path;
use std::char;

pub mod bytecode;
mod tokenizer;

use crate::world::commands::bytecode::{Tokens, TokenError};


type CommandProgRes = Result<ProgramSuccess, ProgramError>;

pub enum ProgramSuccess {
    Success,  // the program executed flawlessly
    Interrupt,  // program ended with a user key interrupt
}

pub enum ProgramError {
    TokenErr(TokenError),
    ExecErr(ProgExecutionError),
}

pub enum ProgExecutionError {

}

pub struct WorldCommandExecutor {
    glob_nmspc: u8,
    exec_tkn: Vec<Vec<Tokens>>,
}

impl WorldCommandExecutor {
    pub fn new() -> Self {
        Self {
            glob_nmspc: 0,
            exec_tkn: Vec::new(),
        }
    }

    pub fn update(&mut self) {

    }

    // directly adds the script commands tokens to the executor tokens
    pub fn load_commands(&mut self) {

    }

    // directly adds the script files tokens to the executor tokens
    pub fn load_file(&mut self) {

    }

    // directly adds the bytecode command tokens to the executor tokens
    pub fn load_commands_bytc(&mut self, char_stream: Vec<char>) {

    }

    // directly adds the bytecode file tokens to the executor tokens
    pub fn load_file_bytc(&mut self, fname: String) {
        match bytecode::compile_file(fname) {
            Ok(mut tokens) => self.exec_tkn.append(&mut tokens),
            Err(err) => println!("Loading bytecode file error: {:?}", err),
        }
    }
}
