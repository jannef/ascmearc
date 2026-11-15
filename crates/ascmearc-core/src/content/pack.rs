//! Packs a content file (#1720): its TOML parsed once, at build time, into a compact tree that
//! `content/de.rs` reads back without a TOML parser. `ascmearc-assets/build.rs` includes this file
//! (the `packed` feature) to pack the built-in content, and the core's tests to check that the
//! packed content loads equal to the TOML. It needs nothing but `toml`, so the build script can
//! include it as it is.
//!
//! The form is text, so a packed file goes where a TOML file goes (a `&str` in `ContentFiles`):
//! [`MARK`], then one value. Every byte that is not inside a string is ASCII:
//! - a number (a length, a count, an integer, a float's bits) is a varint of 6-bit digits, least
//!   significant first, each byte `0x40` while more follow (so `0x00..=0x7f`); an integer is
//!   zigzagged first;
//! - a value is a tag byte and its body: `s` a string (its byte length, then its UTF-8), `i` an
//!   integer, `f` a float, `t` and `n` true and false, `a` an array (its length, then its
//!   values), `m` a table (its length, then each key as a string body and its value, the keys in
//!   sorted order).

/// What a packed file starts with: no TOML file starts with a NUL.
pub const MARK: char = '\0';

/// The packed form of the TOML `text`.
pub fn pack(text: &str) -> Result<String, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut out = String::from(MARK);
    value(&mut out, &toml::Value::Table(table))?;
    Ok(out)
}

fn value(out: &mut String, v: &toml::Value) -> Result<(), String> {
    match v {
        toml::Value::String(s) => {
            out.push('s');
            string(out, s);
        }
        toml::Value::Integer(i) => {
            out.push('i');
            number(out, ((i << 1) ^ (i >> 63)) as u64);
        }
        toml::Value::Float(f) => {
            out.push('f');
            number(out, f.to_bits());
        }
        toml::Value::Boolean(b) => out.push(if *b { 't' } else { 'n' }),
        toml::Value::Array(a) => {
            out.push('a');
            number(out, a.len() as u64);
            for v in a {
                value(out, v)?;
            }
        }
        toml::Value::Table(t) => {
            out.push('m');
            number(out, t.len() as u64);
            for (k, v) in t {
                string(out, k);
                value(out, v)?;
            }
        }
        toml::Value::Datetime(d) => return Err(format!("a date ({d}) is no content value")),
    }
    Ok(())
}

fn string(out: &mut String, s: &str) {
    number(out, s.len() as u64);
    out.push_str(s);
}

fn number(out: &mut String, mut n: u64) {
    loop {
        let digit = (n & 0x3f) as u8;
        n >>= 6;
        if n == 0 {
            out.push(char::from(digit));
            return;
        }
        out.push(char::from(digit | 0x40));
    }
}
