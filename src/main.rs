use pom::char_class::alpha;
use pom::parser::{call, is_a, none_of, not_a, one_of, seq, sym};
use pom::parser::{list, Parser};

#[derive(Debug, PartialEq)]
pub enum LispVal {
    Atom(String),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Number(i64),
    String(String),
    Bool(bool),
}

pub fn letter<'a>() -> Parser<'a, u8, u8> {
    is_a(|c: u8| alpha(c))
}

pub fn digit<'a>() -> Parser<'a, u8, u8> {
    one_of(b"0123456789")
}

pub fn symbol<'a>(b: ()) -> Parser<'a, u8, u8> {
    one_of(b"!#$%&|*+-/:<=>?@^_~")
}

pub fn spaces<'a>() -> Parser<'a, u8, ()> {
    one_of(b" ").repeat(0..).discard()
}

pub fn string<'a>() -> Parser<'a, u8, LispVal> {
    let special_char = sym(b'"');
    let escaped_seq = sym(b'\\') * special_char;
    let string_matcher =
        one_of(b"\"") * ((none_of(b"\\\"") | escaped_seq).repeat(0..)) - one_of(b"\"").collect();
    string_matcher
        .convert(|p| String::from_utf8(p))
        .map(|s| LispVal::String(s))
}

pub fn atom<'a>() -> Parser<'a, u8, LispVal> {
    let first_matcher = letter() | symbol(());
    let rest_matcher = (letter() | digit() | symbol(())).repeat(0..);
    (first_matcher + rest_matcher)
        .collect()
        .map(|matched| match matched {
            b"#t" => LispVal::Bool(true),
            b"#f" => LispVal::Bool(false),
            _ => LispVal::Atom(String::from_utf8(matched.to_vec()).unwrap()),
        })
}

pub fn number<'a>() -> Parser<'a, u8, LispVal> {
    octal_number() | binary_number() | hex_number() | decimal_number()
}
pub fn decimal_number<'a>() -> Parser<'a, u8, LispVal> {
    digit().repeat(1..).collect().map(|parsed| {
        let as_string = String::from_utf8(parsed.to_vec()).unwrap();
        LispVal::Number(i64::from_str_radix(&as_string, 10).unwrap())
    })
}

pub fn binary_number<'a>() -> Parser<'a, u8, LispVal> {
    let prefix = sym(b'#') * sym(b'b');
    (prefix.discard() * (one_of(b"01").repeat(1..)))
        .collect()
        .map(|parsed| {
            let as_string = String::from_utf8_lossy(&parsed[2..]);
            LispVal::Number(i64::from_str_radix(&as_string, 2).unwrap())
        })
}

pub fn octal_number<'a>() -> Parser<'a, u8, LispVal> {
    let prefix = sym(b'#') * sym(b'o');
    (prefix.discard() * (one_of(b"01234567").repeat(1..)))
        .collect()
        .map(|parsed| {
            let as_string = String::from_utf8_lossy(&parsed[2..]);
            LispVal::Number(i64::from_str_radix(&as_string, 8).unwrap())
        })
}

pub fn hex_number<'a>() -> Parser<'a, u8, LispVal> {
    let prefix = sym(b'#') * sym(b'x');
    (prefix.discard() * (one_of(b"0123456789abcdefABCDEF").repeat(1..)))
        .collect()
        .map(|parsed| {
            let mut as_string = String::from_utf8_lossy(&parsed[2..]);
            as_string.to_lowercase();
            LispVal::Number(i64::from_str_radix(&as_string, 16).unwrap())
        })
}

// Taken from a pom example
fn whitespace<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t\r\n").repeat(0..).discard()
}

pub fn parse_list<'a>() -> Parser<'a, u8, Vec<LispVal>> {
    (sym(b'(') * list(call(parse_expr), whitespace())) - sym(b')')
}

pub fn parse_expr<'a>() -> Parser<'a, u8, LispVal> {
    number() | atom() | string()
}

pub fn read_expr(input: &[u8]) -> LispVal {
    parse_expr().parse(input).unwrap()
}

pub fn read_list(input: &[u8]) -> LispVal {
    LispVal::List(parse_list().parse(input).unwrap())
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_number() {
        assert_eq!(read_expr(b"123"), LispVal::Number(123));
    }

    #[test]
    fn read_binary_number() {
        assert_eq!(read_expr(b"#b11"), LispVal::Number(3));
    }

    #[test]
    fn read_octal_number() {
        assert_eq!(read_expr(b"#o321"), LispVal::Number(209));
    }
    #[test]
    fn read_hex_number() {
        assert_eq!(read_expr(b"#xFF"), LispVal::Number(255));
    }

    #[test]
    fn read_string() {
        assert_eq!(read_expr(b"\"123\""), LispVal::String("123".to_owned()));
    }

    #[test]
    fn read_atom() {
        assert_eq!(read_expr(b"symbol"), LispVal::Atom("symbol".to_owned()));
    }
    #[test]
    fn read_string_with_quote() {
        assert_eq!(
            read_expr(b"\"1\\\"23\""),
            LispVal::String("1\"23".to_owned())
        );
    }

    #[test]
    fn read_list_test() {
        assert_eq!(
            read_list(b"(1 2 3)"),
            LispVal::List(vec![
                LispVal::Number(1),
                LispVal::Number(2),
                LispVal::Number(3)
            ])
        );
    }
    #[test]
    fn read_list_strings() {
        assert_eq!(
            read_list(b"(1 2 \"Hello World\")"),
            LispVal::List(vec![
                LispVal::Number(1),
                LispVal::Number(2),
                LispVal::String("Hello World".to_string())
            ])
        );
    }
}
