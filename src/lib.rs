use beam_file::chunk::CodeChunk;
use byteorder::ReadBytesExt as _;
use std::io::Read;

pub const INSTRUCTION_SET_VERSION: u32 = 0;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("supported instruction set version is {INSTRUCTION_SET_VERSION}, but got {version}")]
    UnsupportedInstructionSetVersion { version: u32 },

    #[error("expected a {expected:?} arg, but got {actual:?}")]
    UnexpectedArg {
        expected: &'static str,
        actual: CompactTerm,
    },

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl ParseError {
    fn unexpected_arg(expected: &'static str, actual: CompactTerm) -> Self {
        Self::UnexpectedArg { expected, actual }
    }
}

pub fn parse_code_chunk(chunk: &CodeChunk) -> Result<(), ParseError> {
    if chunk.version != INSTRUCTION_SET_VERSION {
        return Err(ParseError::UnsupportedInstructionSetVersion {
            version: chunk.version,
        });
    }

    dbg!(chunk.info_size);
    dbg!(chunk.opcode_max);
    dbg!(chunk.label_count);
    dbg!(chunk.function_count);
    dbg!(chunk.bytecode.len());
    let mut reader = &mut &chunk.bytecode[..];
    while !reader.is_empty() {
        let op = Op::decode(&mut reader)?;
        dbg!(op);
    }
    todo!()
}

// https://blog.stenmans.org/theBeamBook/#SEC-BeamModulesCTE
pub fn decode_compact_term() {}

pub type DecodeError = ParseError;

#[derive(Debug, Clone)]
pub enum CompactTerm {
    Literal(Literal),
    Atom(Atom),
    Todo,
}

impl CompactTerm {
    // TODO: impl `TryFrom` trait
    pub fn try_into_literal(self) -> Result<Literal, DecodeError> {
        match self {
            CompactTerm::Literal(literal) => Ok(literal),
            term => Err(DecodeError::unexpected_arg("literal", term)),
        }
    }

    pub fn try_into_atom(self) -> Result<Atom, DecodeError> {
        match self {
            CompactTerm::Atom(atom) => Ok(atom),
            term => Err(DecodeError::unexpected_arg("atom", term)),
        }
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        match tag & 0b111 {
            0b000 => {
                if (tag & 0b1000) != 0 {
                    todo!();
                }
                let index = (tag >> 4) as usize;
                Ok(Self::Literal(Literal { index }))
            }
            0b001 => {
                todo!();
            }
            0b010 => {
                if (tag & 0b1000) != 0 {
                    todo!();
                }
                let index = (tag >> 4) as usize;
                Ok(Self::Atom(Atom { index }))
            }
            0b011 => {
                todo!();
            }
            0b100 => {
                todo!();
            }
            0b101 => {
                todo!()
            }
            0b110 => {
                todo!();
            }
            _ => {
                todo!();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom {
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct LabelOp {
    pub literal: Literal,
}

impl LabelOp {
    pub const CODE: u8 = 1;
    pub const ARITY: usize = 1;

    pub fn decode_args<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let literal = CompactTerm::decode(reader)?.try_into_literal()?;
        Ok(Self { literal })
    }
}

#[derive(Debug, Clone)]
pub struct FuncInfoOp {
    pub module: Atom,
    pub function: Atom,
    pub arity: Literal,
}

impl FuncInfoOp {
    pub const CODE: u8 = 2;
    pub const ARITY: usize = 3;

    pub fn decode_args<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let module = CompactTerm::decode(reader)?.try_into_atom()?;
        let function = CompactTerm::decode(reader)?.try_into_atom()?;
        let arity = CompactTerm::decode(reader)?.try_into_literal()?;
        Ok(Self {
            module,
            function,
            arity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LineOp {
    pub literal: Literal,
}

impl LineOp {
    pub const CODE: u8 = 153;
    pub const ARITY: usize = 1;

    pub fn decode_args<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let literal = CompactTerm::decode(reader)?.try_into_literal()?;
        Ok(Self { literal })
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Label(LabelOp),
    FuncInfo(FuncInfoOp),
    Line(LineOp),
}

impl Op {
    pub fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        match reader.read_u8()? {
            LabelOp::CODE => LabelOp::decode_args(reader).map(Self::Label),
            FuncInfoOp::CODE => FuncInfoOp::decode_args(reader).map(Self::FuncInfo),
            LineOp::CODE => LineOp::decode_args(reader).map(Self::Line),
            op => todo!("{op}"),
        }
    }

    pub fn opcode(&self) -> u8 {
        match self {
            Self::Label { .. } => LabelOp::CODE,
            Self::FuncInfo { .. } => FuncInfoOp::CODE,
            Self::Line { .. } => LineOp::CODE,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Self::Label { .. } => LabelOp::ARITY,
            Self::FuncInfo { .. } => FuncInfoOp::ARITY,
            Self::Line { .. } => LineOp::ARITY,
        }
    }
}
