use log::{error, info};
use std::io;

/// unnecessary due to a u8 could present base pair!
// enum Base {
//     A,
//     C,
//     G,
//     T,
//     N,
//     Gap,
// }

// degine a strand enum: +: positive, -: negative
#[derive(Debug)]
pub enum Strand {
    Positive,
    Negative,
}
// a maf line start with 's'
#[derive(Debug)]
pub struct BlockSequence {
    // define fileds by order of a line
    pub seqname: String,
    pub start: u64,
    pub alignsize: u64,
    pub strand: Strand,
    pub seqsize: u64,
    pub alignment: Vec<u8>, // just store base pair --> bytes using u8
}

// a maf block start with 'a'
#[derive(Debug)]
pub struct Block {
    pub aline: String, // store a line loos like: score=2333 => {"score":"2333"}
    pub sequences: Vec<BlockSequence>,
}
pub enum MAFItem {
    Block(Block),
    Comment(String),
}
// make reader lines as a reference
#[derive(Debug)]
pub struct LinesRef<'a, B: 'a> {
    linebuf: &'a mut B,
}

// rewrite the iterator for LinesRef
impl<'a, B: io::BufRead> Iterator for LinesRef<'a, B> {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<io::Result<String>> {
        let mut buf = String::new();
        match self.linebuf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => {
                error!("read line error: {}", buf);
                Some(Err(e))
            }
        }
    }
}

// define parse error
#[derive(Debug)]
pub enum ParseError {
    IOError(io::Error),
    UnexpectedLine(String),
    BadMetadata,
    BadLineType(String),
    Misc(&'static str),
    EOF,
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IOError(err)
    }
}

pub fn get_maf_item<T: io::BufRead + ?Sized>(mut input: &mut T) -> Result<MAFItem, ParseError> {
    let mut header: Option<String> = None;
    {
        let lines = LinesRef {
            linebuf: &mut input,
        };
        for line_res in lines {
            let line: String = line_res?;
            info!("line: {}", line);
            if line.trim().is_empty() {
                // skip blank line
                continue;
            }
            if line.starts_with('#') {
                // MAF comment
                return Ok(MAFItem::Comment(line.chars().skip(1).collect()));
            } else if line.starts_with('a') {
                // consume a a-line
                header = Some(line);
                break; // break the loop and start read s-lines
            } else {
                // Shouldn't see this.
                error!("unexpected line: {}", line);
                return Err(ParseError::UnexpectedLine(line));
            }
        }
    }
    let block = parse_block(
        // read the rest s-lines
        header.ok_or(ParseError::EOF)?,
        LinesRef {
            linebuf: &mut input,
        },
    )?;
    Ok(MAFItem::Block(block))
}

fn parse_block(
    header: String,
    iter: impl Iterator<Item = Result<String, io::Error>>,
) -> Result<Block, ParseError> {
    // init a block pair for storing two BlockSequences
    let mut block_pair: Vec<BlockSequence> = vec![];

    for line_res in iter {
        let line: String = line_res?;
        info!("line: {}", line);
        if line.is_empty() {
            // jump out the loop and return the block
            break;
        }
        let mut fields: Vec<_> = line.split_whitespace().collect(); // split it
        match fields[0] {
            "s" => update_from_s_line(&mut fields, &mut block_pair)?,
            _ => {
                error!("not s-lines!!");
                return Err(ParseError::BadLineType(fields[0].to_string()));
            }
        };
    }
    Ok(Block {
        aline: header,
        sequences: block_pair,
    })
}

// parse strand string to Strand enum
fn parse_strand(strand: &str) -> Result<Strand, ParseError> {
    match strand {
        "+" => Ok(Strand::Positive),
        "-" => Ok(Strand::Negative),
        _ => {
            error!("Strand {} not valid, only for +/-", strand);
            Err(ParseError::Misc("Strand not valid"))
        }
    }
}

fn update_from_s_line(
    fields: &mut Vec<&str>,
    block_pair: &mut Vec<BlockSequence>,
) -> Result<(), ParseError> {
    let alignment = fields.pop().ok_or(ParseError::Misc("s line incomplete"))?;
    let sequence_size = fields
        .pop()
        .ok_or(ParseError::Misc("s line incomplete"))
        .and_then(|s| {
            s.parse::<u64>()
                .map_err(|_| ParseError::Misc("invalid sequence size"))
        })?;
    let strand = fields
        .pop()
        .ok_or(ParseError::Misc("s line incomplete"))
        .and_then(parse_strand)?;
    let aligned_length = fields
        .pop()
        .ok_or(ParseError::Misc("s line incomplete"))
        .and_then(|s| {
            s.parse::<u64>()
                .map_err(|_| ParseError::Misc("invalid aligned length"))
        })?;
    let start = fields
        .pop()
        .ok_or(ParseError::Misc("s line incomplete"))
        .and_then(|s| {
            s.parse::<u64>()
                .map_err(|_| ParseError::Misc("invalid start"))
        })?;
    let seqname = fields.pop().ok_or(ParseError::Misc("s line incomplete"))?;
    block_pair.push(BlockSequence {
        alignment: alignment.as_bytes().to_vec(),
        seqsize: sequence_size,
        strand,
        alignsize: aligned_length,
        start,
        seqname: seqname.to_string(),
    });
    Ok(())
}
