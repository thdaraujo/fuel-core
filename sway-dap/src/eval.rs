use std::io;
use std::str::FromStr;

use fuel_vm::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Expression {
    Word(Word),
    Register(RegisterId),
    Memory(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    Print(Expression),
    Exec(Opcode),
    Quit,
}

impl Instruction {
    fn parse_word(token: &str) -> io::Result<Word> {
        if let Some(token) = token.strip_prefix("0x") {
            Word::from_str_radix(token, 16)
        } else if let Some(token) = token.strip_prefix("0b") {
            Word::from_str_radix(token, 2)
        } else {
            Word::from_str(token)
        }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn take_token<'a, T>(tokens: &mut T) -> io::Result<&'a str>
    where
        T: Iterator<Item = &'a str>,
    {
        tokens.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Failed to extract token from provided command",
        ))
    }

    fn take_word<'a, T>(tokens: &mut T) -> io::Result<Word>
    where
        T: Iterator<Item = &'a str>,
    {
        Self::take_token(tokens).and_then(Self::parse_word)
    }

    fn word<'a, T>(tokens: &mut T) -> io::Result<Expression>
    where
        T: Iterator<Item = &'a str>,
    {
        Self::take_word(tokens).map(|w| Expression::Word(w))
    }

    fn register<'a, T>(tokens: &mut T) -> io::Result<Expression>
    where
        T: Iterator<Item = &'a str>,
    {
        Self::take_token(tokens)
            .map(|t| t.split_at(1).1)
            .and_then(Self::parse_word)
            .map(|w| Expression::Register(w as RegisterId))
    }

    fn memory<'a, T>(tokens: &mut T) -> io::Result<Expression>
    where
        T: Iterator<Item = &'a str>,
    {
        let left = Self::take_token(tokens)?;
        let token = if left.contains(']') {
            left.to_owned()
        } else {
            let right = Self::take_token(tokens)?;
            format!("{}{}", left, right)
        };

        let parsed: Vec<&str> = token
            .as_str()
            .split(&['$', 'm', '[', ',', ']'][..])
            .collect();

        match (parsed.get(3), parsed.get(4)) {
            (Some(l), Some(r)) => {
                let l = Self::parse_word(l)? as usize;
                let r = Self::parse_word(r)? as usize;

                Ok(Expression::Memory(l, r))
            }

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The provided memory slice is invalid",
            )),
        }
    }

    fn expression<'a, T>(tokens: &mut T) -> io::Result<Expression>
    where
        T: Clone + Iterator<Item = &'a str>,
    {
        let token = tokens.clone().peekable().next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Failed to parse expression",
        ))?;

        if token.starts_with("$m") {
            Self::memory(tokens)
        } else if token.starts_with("$") {
            Self::register(tokens)
        } else {
            Self::word(tokens)
        }
    }

    #[rustfmt::skip]
    fn opcode<'a, T>(tokens: &mut T) -> io::Result<Opcode>
    where
        T: Iterator<Item = &'a str>,
    {
        let x = Self::take_token(tokens)?;

        let a = Self::take_token(tokens).and_then(Self::parse_word).map(|w| w as RegisterId).ok();
        let b = Self::take_token(tokens).and_then(Self::parse_word).map(|w| w as RegisterId).ok();
        let c = Self::take_token(tokens).and_then(Self::parse_word).map(|w| w as RegisterId).ok();
        let d = Self::take_token(tokens).and_then(Self::parse_word).map(|w| w as RegisterId).ok();

        match (x, a, b, c, d) {
            ("noop",    _, _, _, _) => Ok(Opcode::NOOP),
            ("ret",     Some(a), _, _, _) => Ok(Opcode::RET(  a)),
            ("aloc",    Some(a), _, _, _) => Ok(Opcode::ALOC( a)),
            ("bhei",    Some(a), _, _, _) => Ok(Opcode::BHEI( a)),
            ("burn",    Some(a), _, _, _) => Ok(Opcode::BURN( a)),
            ("mint",    Some(a), _, _, _) => Ok(Opcode::MINT( a)),
            ("rvrt",    Some(a), _, _, _) => Ok(Opcode::RVRT( a)),
            ("flag",    Some(a), _, _, _) => Ok(Opcode::FLAG( a)),
            ("cb",      Some(a), _, _, _) => Ok(Opcode::CB(   a)),
            ("ji",      Some(a), _, _, _) => Ok(Opcode::JI(   a as Immediate24)),
            ("cfei",    Some(a), _, _, _) => Ok(Opcode::CFEI( a as Immediate24)),
            ("cfsi",    Some(a), _, _, _) => Ok(Opcode::CFSI( a as Immediate24)),
            ("move",    Some(a), Some(b), _, _) => Ok(Opcode::MOVE( a, b)),
            ("not",     Some(a), Some(b), _, _) => Ok(Opcode::NOT(  a, b)),
            ("ctmv",    Some(a), Some(b), _, _) => Ok(Opcode::CTMV( a, b)),
            ("mcl",     Some(a), Some(b), _, _) => Ok(Opcode::MCL(  a, b)),
            ("bhsh",    Some(a), Some(b), _, _) => Ok(Opcode::BHSH( a, b)),
            ("croo",    Some(a), Some(b), _, _) => Ok(Opcode::CROO( a, b)),
            ("csiz",    Some(a), Some(b), _, _) => Ok(Opcode::CSIZ( a, b)),
            ("srw",     Some(a), Some(b), _, _) => Ok(Opcode::SRW(  a, b)),
            ("srwq",    Some(a), Some(b), _, _) => Ok(Opcode::SRWQ( a, b)),
            ("sww",     Some(a), Some(b), _, _) => Ok(Opcode::SWW(  a, b)),
            ("swwq",    Some(a), Some(b), _, _) => Ok(Opcode::SWWQ( a, b)),
            ("xil",     Some(a), Some(b), _, _) => Ok(Opcode::XIL(  a, b)),
            ("xis",     Some(a), Some(b), _, _) => Ok(Opcode::XIS(  a, b)),
            ("xol",     Some(a), Some(b), _, _) => Ok(Opcode::XOL(  a, b)),
            ("xos",     Some(a), Some(b), _, _) => Ok(Opcode::XOS(  a, b)),
            ("xwl",     Some(a), Some(b), _, _) => Ok(Opcode::XWL(  a, b)),
            ("xws",     Some(a), Some(b), _, _) => Ok(Opcode::XWS(  a, b)),
            ("mcli",    Some(a), Some(b), _, _) => Ok(Opcode::MCLI( a, b as Immediate18)),
            ("add",     Some(a), Some(b), Some(c), _) => Ok(Opcode::ADD(  a, b, c)),
            ("and",     Some(a), Some(b), Some(c), _) => Ok(Opcode::AND(  a, b, c)),
            ("div",     Some(a), Some(b), Some(c), _) => Ok(Opcode::DIV(  a, b, c)),
            ("eq",      Some(a), Some(b), Some(c), _) => Ok(Opcode::EQ(   a, b, c)),
            ("exp",     Some(a), Some(b), Some(c), _) => Ok(Opcode::EXP(  a, b, c)),
            ("gt",      Some(a), Some(b), Some(c), _) => Ok(Opcode::GT(   a, b, c)),
            ("mlog",    Some(a), Some(b), Some(c), _) => Ok(Opcode::MLOG( a, b, c)),
            ("mroo",    Some(a), Some(b), Some(c), _) => Ok(Opcode::MROO( a, b, c)),
            ("mod",     Some(a), Some(b), Some(c), _) => Ok(Opcode::MOD(  a, b, c)),
            ("mul",     Some(a), Some(b), Some(c), _) => Ok(Opcode::MUL(  a, b, c)),
            ("or",      Some(a), Some(b), Some(c), _) => Ok(Opcode::OR(   a, b, c)),
            ("sll",     Some(a), Some(b), Some(c), _) => Ok(Opcode::SLL(  a, b, c)),
            ("srl",     Some(a), Some(b), Some(c), _) => Ok(Opcode::SRL(  a, b, c)),
            ("sub",     Some(a), Some(b), Some(c), _) => Ok(Opcode::SUB(  a, b, c)),
            ("xor",     Some(a), Some(b), Some(c), _) => Ok(Opcode::XOR(  a, b, c)),
            ("cimv",    Some(a), Some(b), Some(c), _) => Ok(Opcode::CIMV( a, b, c)),
            ("mcp",     Some(a), Some(b), Some(c), _) => Ok(Opcode::MCP(  a, b, c)),
            ("ldc",     Some(a), Some(b), Some(c), _) => Ok(Opcode::LDC(  a, b, c)),
            ("tr",      Some(a), Some(b), Some(c), _) => Ok(Opcode::TR(   a, b, c)),
            ("sldc",    Some(a), Some(b), Some(c), _) => Ok(Opcode::SLDC( a, b, c)),
            ("ecr",     Some(a), Some(b), Some(c), _) => Ok(Opcode::ECR(  a, b, c)),
            ("k256",    Some(a), Some(b), Some(c), _) => Ok(Opcode::K256( a, b, c)),
            ("s256",    Some(a), Some(b), Some(c), _) => Ok(Opcode::S256( a, b, c)),
            ("modi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::MODI( a, b, c as Immediate12)),
            ("addi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::ADDI( a, b, c as Immediate12)),
            ("andi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::ANDI( a, b, c as Immediate12)),
            ("expi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::EXPI( a, b, c as Immediate12)),
            ("divi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::DIVI( a, b, c as Immediate12)),
            ("muli",    Some(a), Some(b), Some(c), _) => Ok(Opcode::MULI( a, b, c as Immediate12)),
            ("ori",     Some(a), Some(b), Some(c), _) => Ok(Opcode::ORI(  a, b, c as Immediate12)),
            ("slli",    Some(a), Some(b), Some(c), _) => Ok(Opcode::SLLI( a, b, c as Immediate12)),
            ("srli",    Some(a), Some(b), Some(c), _) => Ok(Opcode::SRLI( a, b, c as Immediate12)),
            ("subi",    Some(a), Some(b), Some(c), _) => Ok(Opcode::SUBI( a, b, c as Immediate12)),
            ("xori",    Some(a), Some(b), Some(c), _) => Ok(Opcode::XORI( a, b, c as Immediate12)),
            ("jnei",    Some(a), Some(b), Some(c), _) => Ok(Opcode::JNEI( a, b, c as Immediate12)),
            ("lb",      Some(a), Some(b), Some(c), _) => Ok(Opcode::LB(   a, b, c as Immediate12)),
            ("lw",      Some(a), Some(b), Some(c), _) => Ok(Opcode::LW(   a, b, c as Immediate12)),
            ("sb",      Some(a), Some(b), Some(c), _) => Ok(Opcode::SB(   a, b, c as Immediate12)),
            ("sw",      Some(a), Some(b), Some(c), _) => Ok(Opcode::SW(   a, b, c as Immediate12)),
            ("meq",     Some(a), Some(b), Some(c), Some(d)) => Ok(Opcode::MEQ(  a, b, c, d)),
            ("call",    Some(a), Some(b), Some(c), Some(d)) => Ok(Opcode::CALL( a, b, c, d)),
            ("ccp",     Some(a), Some(b), Some(c), Some(d)) => Ok(Opcode::CCP(  a, b, c, d)),
            ("log",     Some(a), Some(b), Some(c), Some(d)) => Ok(Opcode::LOG(  a, b, c, d)),
            ("tro",     Some(a), Some(b), Some(c), Some(d)) => Ok(Opcode::TRO(  a, b, c, d)),

            _ => Err(io::Error::new(io::ErrorKind::Other, "Malformed opcode"))
        }
    }
}

impl FromStr for Instruction {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let mut tokens = s.as_str().split_whitespace();

        let instruction = tokens.next().ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "The instruction could not be parsed",
        ))?;

        match instruction {
            "p" | "print" => Self::expression(&mut tokens).map(Self::Print),

            "x" | "exec" => Self::opcode(&mut tokens).map(Self::Exec),

            "q" | "quit" => Ok(Self::Quit),

            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unimplemented instruction",
            )),
        }
    }
}
