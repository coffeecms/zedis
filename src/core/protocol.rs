use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::{crlf, i64 as parse_i64},
    combinator::map,
    multi::count,
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum RespFrame {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespFrame>>),
    #[allow(dead_code)]
    Null,
}

impl RespFrame {
    pub fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            RespFrame::SimpleString(s) => {
                buf.extend_from_slice(b"+");
                buf.extend_from_slice(s.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
            RespFrame::Error(s) => {
                buf.extend_from_slice(b"-");
                buf.extend_from_slice(s.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
            RespFrame::Integer(i) => {
                buf.extend_from_slice(b":");
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
            RespFrame::BulkString(Some(s)) => {
                buf.extend_from_slice(b"$");
                buf.extend_from_slice(s.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf.extend_from_slice(s.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
            RespFrame::BulkString(None) => {
                buf.extend_from_slice(b"$-1\r\n");
            }
            RespFrame::Array(Some(frames)) => {
                buf.extend_from_slice(b"*");
                buf.extend_from_slice(frames.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                for frame in frames {
                    frame.encode(buf);
                }
            }
            RespFrame::Array(None) => {
                buf.extend_from_slice(b"*-1\r\n");
            }
            RespFrame::Null => {
                 buf.extend_from_slice(b"_\r\n");
            }
        }
    }
}


pub fn parse_frame(input: &[u8]) -> IResult<&[u8], RespFrame> {
    alt((
        parse_simple_string,
        parse_error,
        parse_integer,
        parse_bulk_string,
        parse_array,
    ))(input)
}


fn parse_simple_string(input: &[u8]) -> IResult<&[u8], RespFrame> {
    map(
        delimited(tag("+"), take_until("\r\n"), crlf),
        |s: &[u8]| RespFrame::SimpleString(String::from_utf8_lossy(s).to_string()),
    )(input)
}

fn parse_error(input: &[u8]) -> IResult<&[u8], RespFrame> {
    map(
        delimited(tag("-"), take_until("\r\n"), crlf),
        |s: &[u8]| RespFrame::Error(String::from_utf8_lossy(s).to_string()),
    )(input)
}

fn parse_integer(input: &[u8]) -> IResult<&[u8], RespFrame> {
    map(
        delimited(tag(":"), parse_i64, crlf),
        RespFrame::Integer,
    )(input)
}

fn parse_bulk_string(input: &[u8]) -> IResult<&[u8], RespFrame> {
    let (input, len) = delimited(tag("$"), parse_i64, crlf)(input)?;

    if len == -1 {
        return Ok((input, RespFrame::BulkString(None)));
    }

    let len = len as usize;
    map(terminated(take(len), crlf), |s: &[u8]| {
        RespFrame::BulkString(Some(String::from_utf8_lossy(s).to_string()))
    })(input)
}

fn parse_array(input: &[u8]) -> IResult<&[u8], RespFrame> {
    let (input, len) = delimited(tag("*"), parse_i64, crlf)(input)?;

    if len == -1 {
        return Ok((input, RespFrame::Array(None)));
    }

    let len = len as usize;
    map(count(parse_frame, len), |frames| {
        RespFrame::Array(Some(frames))
    })(input)
}
