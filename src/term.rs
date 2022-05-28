use crate::{Decode, DecodeError, Encode, EncodeError, USIZE_BYTES};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::BigInt;
use std::io::{Read, Write};

#[derive(Debug, thiserror::Error)]
pub enum ConvertTermError {
    #[error("expected a literal, but got {term:?}")]
    NotLiteral { term: Term },

    #[error("expected an integer, but got {term:?}")]
    NotInteger { term: Term },

    #[error("expected an atom, but got {term:?}")]
    NotAtom { term: Term },

    #[error("expected a label, but got {term:?}")]
    NotLabel { term: Term },

    #[error("expected a x-register, but got {term:?}")]
    NotXRegister { term: Term },

    #[error("expected a y-register, but got {term:?}")]
    NotYRegister { term: Term },

    #[error("expected a register, but got {term:?}")]
    NotRegister { term: Term },

    #[error("expected a list, but got {term:?}")]
    NotList { term: Term },

    #[error("expected an extended literal, but got {term:?}")]
    NotExtendedLiteral { term: Term },
}

// From beam_opcodes.hrl file.
const TAG_U: u8 = 0; // Literal
const TAG_I: u8 = 1; // Integer
const TAG_A: u8 = 2; // Atom
const TAG_X: u8 = 3; // X regsiter
const TAG_Y: u8 = 4; // Y register
const TAG_F: u8 = 5; // Label
const TAG_H: u8 = 6; // Character
const TAG_Z: u8 = 7; // Extended

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode)]
pub enum Term {
    Literal(Literal),
    Integer(Integer),
    Atom(Atom),
    XRegister(XRegister),
    YRegister(YRegister),
    Label(Label),
    List(List),
    ExtendedLiteral(ExtendedLiteral),
    // TODO: Alloc List, etc
}

impl Term {
    fn decode_extended<R: Read>(tag: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match tag >> 4 {
            0b0001 => {
                // TODO: List::decode
                let size = Literal::decode(reader)?;
                (0..size.value)
                    .map(|_| Self::decode(reader))
                    .collect::<Result<_, _>>()
                    .map(|elements| Self::List(List { elements }))
            }
            0b0010 => {
                todo!("floating piont register");
            }
            0b0011 => {
                todo!("allocation list");
            }
            0b0100 => ExtendedLiteral::decode(reader).map(Self::ExtendedLiteral),
            0b0101 => Register::decode(&mut once(tag).chain(reader)).map(Self::from),
            _ => Err(DecodeError::UnknownTermTag { tag }),
        }
    }
}

impl Decode for Term {
    fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        match tag & 0b111 {
            TAG_U => Literal::decode(&mut once(tag).chain(reader)).map(Self::Literal),
            TAG_I => Integer::decode(tag, reader).map(Self::Integer),
            TAG_A => Atom::decode(tag, reader).map(Self::Atom),
            TAG_X => XRegister::decode(&mut once(tag).chain(reader)).map(Self::XRegister),
            TAG_Y => YRegister::decode(&mut once(tag).chain(reader)).map(Self::YRegister),
            TAG_F => Label::decode(tag, reader).map(Self::Label),
            TAG_H => todo!(),
            TAG_Z => Self::decode_extended(tag, reader),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode)]
pub enum Register {
    X(XRegister),
    Y(YRegister),
}

impl TryFrom<Term> for Register {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        match term {
            Term::XRegister(t) => Ok(Self::X(t)),
            Term::YRegister(t) => Ok(Self::Y(t)),
            _ => Err(ConvertTermError::NotRegister { term }),
        }
    }
}

impl From<Register> for Term {
    fn from(v: Register) -> Self {
        match v {
            Register::X(v) => Term::XRegister(v),
            Register::Y(v) => Term::YRegister(v),
        }
    }
}

impl Decode for Register {
    fn decode<R: Read>(mut reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        match tag & 0b111 {
            TAG_X => XRegister::decode(&mut once(tag).chain(reader)).map(Self::X),
            TAG_Y => YRegister::decode(&mut once(tag).chain(reader)).map(Self::Y),
            TAG_Z if tag >> 4 == 0b0101 => {
                let tag = reader.read_u8()?;
                let mut register = match tag & 0b111 {
                    TAG_X => XRegister::decode(&mut once(tag).chain(&mut reader)).map(Self::X)?,
                    TAG_Y => YRegister::decode(&mut once(tag).chain(&mut reader)).map(Self::Y)?,
                    _ => return Err(DecodeError::UnknownTermTag { tag }),
                };
                let ty = Literal::decode(&mut reader)?;
                match &mut register {
                    Self::X(r) => r.ty = Some(ty.value),
                    Self::Y(r) => r.ty = Some(ty.value),
                };
                Ok(register)
            }
            _ => Err(DecodeError::UnknownTermTag { tag }),
        }
    }
}

// TODO: move
#[derive(Debug)]
struct Once {
    byte: u8,
    read: bool,
}

impl Read for Once {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read || buf.is_empty() {
            Ok(0)
        } else {
            buf[0] = self.byte;
            self.read = true;
            Ok(1)
        }
    }
}

fn once(byte: u8) -> Once {
    Once { byte, read: false }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    XRegister(XRegister),
    YRegister(YRegister),
    Literal(Literal),
    Integer(Integer),
    Atom(Atom),
}

// TODO: impl Decode
// TODO(?): s/Literal/Usize/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Literal {
    pub value: usize,
}

impl Decode for Literal {
    fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag & 0b111 != TAG_U {
            return Err(DecodeError::UnknownTermTag { tag });
        }
        let value = decode_usize(tag, reader)?;
        Ok(Self { value })
    }
}

impl Encode for Literal {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_usize(TAG_U, self.value, writer)
    }
}

impl TryFrom<Term> for Literal {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::Literal(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotLiteral { term })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExtendedLiteral {
    pub value: usize,
}

impl ExtendedLiteral {
    fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let literal: Literal = Term::decode(reader)?.try_into()?;
        Ok(Self {
            value: literal.value,
        })
    }
}

impl Encode for ExtendedLiteral {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u8(TAG_Z | 0b0100_0000)?;
        let literal = Literal { value: self.value };
        literal.encode(writer)
    }
}

impl TryFrom<Term> for ExtendedLiteral {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::ExtendedLiteral(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotExtendedLiteral { term })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Integer {
    pub value: BigInt,
}

impl Integer {
    fn decode<R: Read>(tag: u8, reader: &mut R) -> Result<Self, DecodeError> {
        let value = decode_integer(tag, reader)?;
        Ok(Self { value })
    }
}

impl Encode for Integer {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_integer(TAG_I, &self.value, writer)
    }
}

impl TryFrom<Term> for Integer {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::Integer(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotInteger { term })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom {
    pub value: usize,
}

impl Atom {
    fn decode<R: Read>(tag: u8, reader: &mut R) -> Result<Self, DecodeError> {
        let value = decode_usize(tag, reader)?;
        Ok(Self { value })
    }
}

impl Encode for Atom {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_usize(TAG_A, self.value, writer)
    }
}

impl TryFrom<Term> for Atom {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::Atom(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotAtom { term })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct XRegister {
    pub value: usize,
    pub ty: Option<usize>,
}

impl Decode for XRegister {
    fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag & 0b111 != TAG_X {
            return Err(DecodeError::UnknownTermTag { tag });
        }
        let value = decode_usize(tag, reader)?;
        Ok(Self { value, ty: None })
    }
}

impl Encode for XRegister {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_usize(TAG_X, self.value, writer)
    }
}

impl TryFrom<Term> for XRegister {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::XRegister(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotXRegister { term })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct YRegister {
    pub value: usize,
    pub ty: Option<usize>,
}

impl Decode for YRegister {
    fn decode<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = reader.read_u8()?;
        if tag & 0b111 != TAG_Y {
            return Err(DecodeError::UnknownTermTag { tag });
        }
        let value = decode_usize(tag, reader)?;
        Ok(Self { value, ty: None })
    }
}

impl Encode for YRegister {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_usize(TAG_Y, self.value, writer)
    }
}

impl TryFrom<Term> for YRegister {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::YRegister(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotYRegister { term })
        }
    }
}

impl Encode for Vec<YRegister> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let list = List {
            elements: self.iter().copied().map(Term::YRegister).collect(),
        };
        list.encode(writer)
    }
}

impl TryFrom<Term> for Vec<YRegister> {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        List::try_from(term).and_then(|list| {
            list.elements
                .into_iter()
                .map(|x| YRegister::try_from(x))
                .collect()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Label {
    pub value: usize,
}

impl Label {
    fn decode<R: Read>(tag: u8, reader: &mut R) -> Result<Self, DecodeError> {
        let value = decode_usize(tag, reader)?;
        Ok(Self { value })
    }
}

impl Encode for Label {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        encode_usize(TAG_F, self.value, writer)
    }
}

impl TryFrom<Term> for Label {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::Label(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotLabel { term })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct List {
    pub elements: Vec<Term>,
}

impl Encode for List {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u8(TAG_Z | 0b0001_0000)?;
        let size = Literal {
            value: self.elements.len(),
        };
        size.encode(writer)?;
        for x in &self.elements {
            x.encode(writer)?;
        }
        Ok(())
    }
}

impl TryFrom<Term> for List {
    type Error = ConvertTermError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        if let Term::List(t) = term {
            Ok(t)
        } else {
            Err(ConvertTermError::NotList { term })
        }
    }
}

fn decode_usize<R: Read>(tag: u8, reader: &mut R) -> Result<usize, DecodeError> {
    if (tag & 0b1_000) == 0 {
        Ok((tag >> 4) as usize)
    } else if (tag & 0b10_000) == 0 {
        let v = reader.read_u8()? as usize;
        Ok((usize::from(tag & 0b111_00_000) << 3) | v)
    } else if (tag >> 5) != 0b111 {
        let byte_size = usize::from(tag >> 5) + 2;
        if byte_size > USIZE_BYTES as usize {
            Err(DecodeError::TooLargeUsizeValue { byte_size })
        } else {
            Ok(reader.read_uint::<BigEndian>(byte_size)? as usize)
        }
    } else {
        let byte_size = Literal::decode(reader)?.value;
        Err(DecodeError::TooLargeUsizeValue { byte_size })
    }
}

fn encode_usize<W: Write>(tag: u8, value: usize, writer: &mut W) -> Result<(), EncodeError> {
    if value < 16 {
        writer.write_u8((value << 4) as u8 | tag)?;
    } else if value < 0x800 {
        writer.write_u8(((value >> 3) as u8 & 0b1110_0000) | tag | 0b000_1000)?;
        writer.write_u8((value & 0xFF) as u8)?;
    } else {
        let bytes = value.to_be_bytes();
        encode_num_bytes(tag, &bytes, writer)?;
    }
    Ok(())
}

fn decode_integer<R: Read>(tag: u8, reader: &mut R) -> Result<BigInt, DecodeError> {
    if (tag & 0b1_000) == 0 {
        Ok(BigInt::from(tag >> 4))
    } else if (tag & 0b10_000) == 0 {
        let v = u64::from(reader.read_u8()?);
        Ok(BigInt::from((u64::from(tag) & 0b111_00_000) << 3 | v))
    } else if (tag >> 5) != 0b111 {
        let byte_size = usize::from(tag >> 5) + 2;
        let mut buf = vec![0; byte_size];
        reader.read_exact(&mut buf)?;
        Ok(BigInt::from_signed_bytes_be(&buf))
    } else {
        let byte_size = Literal::decode(reader)?.value;
        let mut buf = vec![0; byte_size];
        reader.read_exact(&mut buf)?;
        Ok(BigInt::from_signed_bytes_be(&buf))
    }
}

fn encode_integer<W: Write>(tag: u8, value: &BigInt, writer: &mut W) -> Result<(), EncodeError> {
    if let Ok(v) = usize::try_from(value.clone()) {
        encode_usize(tag, v, writer)
    } else if let Ok(v) = i16::try_from(value.clone()) {
        let bytes = v.to_be_bytes();
        encode_num_bytes(tag, &bytes, writer)
    } else {
        let bytes = value.to_signed_bytes_be();
        encode_num_bytes(tag, &bytes, writer)
    }
}

fn encode_num_bytes<W: Write>(tag: u8, bytes: &[u8], writer: &mut W) -> Result<(), EncodeError> {
    assert!(bytes.len() >= 2, "bug");

    if bytes.len() <= 8 {
        let mut n = bytes.len();
        for (i, b) in bytes.iter().copied().enumerate() {
            if b != 0 {
                if (b & 0b1000_0000) != 0 {
                    n += 1;
                }
                writer.write_u8(((n - 2) << 5) as u8 | 0b0001_1000 | tag)?;
                if (b & 0b1000_0000) != 0 {
                    writer.write_u8(0)?;
                }
                for &b in &bytes[i..] {
                    writer.write_u8(b)?;
                }
                break;
            }
            n -= 1;
        }
    } else {
        writer.write_u8(tag | 0b1111_1000)?;
        let size = Literal {
            value: bytes.len() - 8,
        };
        size.encode(writer)?;
        writer.write_all(bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_usize_works() {
        let data: &[(&[u8], usize)] = &[
            (&[0], 0),
            (&[16], 1),
            (&[8, 20], 20),
            (&[40, 144], 400),
            (&[24, 87, 28], 22300),
            (&[56, 15, 18, 6], 987654),
        ];
        for (input, expected) in data {
            let decoded = decode_usize(input[0], &mut &input[1..]).expect("decode failure");
            assert_eq!(decoded, *expected);
        }
    }

    #[test]
    fn too_large_usize_value() {
        let input = [248, 0, 0, 137, 16, 135, 184, 176, 52, 113, 21];
        assert!(matches!(
            decode_usize(input[0], &mut &input[1..]),
            Err(DecodeError::TooLargeUsizeValue { .. })
        ));
    }

    #[test]
    fn decode_integer_works() {
        let data: &[(&[u8], i64)] = &[
            (&[0], 0),
            (&[16], 1),
            (&[8, 20], 20),
            (&[40, 144], 400),
            (&[24, 87, 28], 22300),
            (&[56, 15, 18, 6], 987654),
            (&[24, 255, 255], -1),
            (&[24, 254, 189], -323),
            (&[88, 248, 164, 147, 83], -123432109),
        ];
        for (input, expected) in data {
            let decoded = decode_integer(input[0], &mut &input[1..]).expect("decode failure");
            assert_eq!(decoded, BigInt::from(*expected));
        }
    }
}
