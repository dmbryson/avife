use anyhow::{Context, Result};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(Display, EnumString, Debug, PartialEq, PartialOrd, Clone, Copy)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
#[repr(u8)]
pub enum Instruction {
    NopA = 0,
    NopB = 1,
    NopC = 2,

    IfNEqu = 3,
    IfLess = 4,
    IfLabel = 5,
    MovHead = 6,
    JmpHead = 7,
    GetHead = 8,
    SetFlow = 9,

    ShiftR = 10,
    ShiftL = 11,
    Inc = 12,
    Dec = 13,
    Push = 14,
    Pop = 15,
    SwapStk = 16,
    Swap = 17,

    Add = 18,
    Sub = 19,
    Nand = 20,

    HCopy = 21,
    HAlloc = 22,
    HDivide = 23,

    IO = 24,
    HSearch = 25,

    Error = 255,
}

#[allow(dead_code)]
impl Instruction {
    pub fn is_nop(self) -> bool {
        self <= Instruction::NopC
    }

    pub fn get_nop_mod(self) -> usize {
        self as usize
    }

    pub fn to_letter(self) -> char {
        (b'a' + self as u8) as char
    }

    pub fn from_letter(c: char) -> Option<Self> {
        match c {
            'a' => Some(Instruction::NopA),
            'b' => Some(Instruction::NopB),
            'c' => Some(Instruction::NopC),
            'd' => Some(Instruction::IfNEqu),
            'e' => Some(Instruction::IfLess),
            'f' => Some(Instruction::IfLabel),
            'g' => Some(Instruction::MovHead),
            'h' => Some(Instruction::JmpHead),
            'i' => Some(Instruction::GetHead),
            'j' => Some(Instruction::SetFlow),
            'k' => Some(Instruction::ShiftR),
            'l' => Some(Instruction::ShiftL),
            'm' => Some(Instruction::Inc),
            'n' => Some(Instruction::Dec),
            'o' => Some(Instruction::Push),
            'p' => Some(Instruction::Pop),
            'q' => Some(Instruction::SwapStk),
            'r' => Some(Instruction::Swap),
            's' => Some(Instruction::Add),
            't' => Some(Instruction::Sub),
            'u' => Some(Instruction::Nand),
            'v' => Some(Instruction::HCopy),
            'w' => Some(Instruction::HAlloc),
            'x' => Some(Instruction::HDivide),
            'y' => Some(Instruction::IO),
            'z' => Some(Instruction::HSearch),
            _ => None,
        }
    }
}

pub struct InstructionSet {
    instructions: Vec<Instruction>
}

impl InstructionSet {
    pub fn default_set() -> Self {
        InstructionSet {
            instructions: vec![
                Instruction::NopA,
                Instruction::NopB,
                Instruction::NopC,
                Instruction::IfNEqu,
                Instruction::IfLess,
                Instruction::IfLabel,
                Instruction::MovHead,
                Instruction::JmpHead,
                Instruction::GetHead,
                Instruction::SetFlow,
                Instruction::ShiftR,
                Instruction::ShiftL,
                Instruction::Inc,
                Instruction::Dec,
                Instruction::Push,
                Instruction::Pop,
                Instruction::SwapStk,
                Instruction::Swap,
                Instruction::Add,
                Instruction::Sub,
                Instruction::Nand,
                Instruction::HCopy,
                Instruction::HAlloc,
                Instruction::HDivide,
                Instruction::IO,
                Instruction::HSearch,
            ],
        }
    }

    pub fn random(&self, rng: &mut ChaCha8Rng) -> Instruction {
        let idx = rng.random_range(0..self.instructions.len());
        self.instructions[idx]
    }
}

#[derive(Clone)]
pub struct Genome {
    instructions: Vec<Instruction>,
}

impl Genome {
    pub const MIN_SIZE: usize = 8;
    pub const MAX_SIZE: usize = 2048;

    pub fn new(instructions: Vec<Instruction>) -> Genome {
        Genome{instructions}
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Genome> {
        let contents = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
        let instructions = contents
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let cleaned = line.split('#').next().unwrap_or("").trim();
                (i + 1, cleaned)
            })
            .filter(|(_, line)| !line.is_empty())
            .map(|(line_num, token)| {
                Instruction::from_str(token)
                    .with_context(|| format!("line {}: invalid instruction {:?}", line_num, token))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Genome { instructions })
    }

    pub fn as_short_string(&self) -> String {
        self.instructions.iter().map(|i| i.to_letter()).collect()
    }
}

impl Deref for Genome {
    type Target = Vec<Instruction>;
    fn deref(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}

impl DerefMut for Genome {
    fn deref_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }
}
