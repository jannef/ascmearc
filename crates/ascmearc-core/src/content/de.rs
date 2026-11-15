//! Reads a content file into its type (#1720): every loader calls [`from_str`] on the text
//! `ContentFiles` holds, which is TOML or the packed form `content/pack.rs` makes.
//!
//! With the `packed` feature (the web build) the TOML arm is compiled out (but for the core's own
//! tests), so no `toml` deserializer is instantiated for a content type and the build carries one
//! small reader instead: only the packed built-in content loads, and a TOML file (a mod, a
//! `--content-dir`) is refused (docs/dev/content.md, "Packed content").

use super::pack::MARK;
use super::value::{Error, Table, Value};
use serde::de::DeserializeOwned;

/// Reads `text`, a whole content file, as a `T`.
pub(crate) fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    if let Some(packed) = text.strip_prefix(MARK) {
        return T::deserialize(decode(packed.as_bytes())?);
    }
    toml_from_str(text)
}

#[cfg(any(test, not(feature = "packed")))]
fn toml_from_str<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    toml::from_str(text).map_err(|e| Error::new(e.to_string()))
}

#[cfg(all(not(test), feature = "packed"))]
fn toml_from_str<T: DeserializeOwned>(_: &str) -> Result<T, Error> {
    Err(Error::new(
        "this build reads only the packed built-in content, not TOML (the `packed` feature)",
    ))
}

/// The value packed in `bytes` (`content/pack.rs` has the form).
fn decode(bytes: &[u8]) -> Result<Value, Error> {
    let mut reader = Reader { bytes, at: 0 };
    let value = reader.value()?;
    match reader.at == bytes.len() {
        true => Ok(value),
        false => Err(corrupt()),
    }
}

fn corrupt() -> Error {
    Error::new("corrupt packed content")
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Result<u8, Error> {
        let b = *self.bytes.get(self.at).ok_or_else(corrupt)?;
        self.at += 1;
        Ok(b)
    }

    fn number(&mut self) -> Result<u64, Error> {
        let mut n = 0u64;
        for shift in (0..64).step_by(6) {
            let b = self.byte()?;
            n |= u64::from(b & 0x3f) << shift;
            if b & 0x40 == 0 {
                return Ok(n);
            }
        }
        Err(corrupt())
    }

    fn len(&mut self) -> Result<usize, Error> {
        usize::try_from(self.number()?).map_err(|_| corrupt())
    }

    fn string(&mut self) -> Result<String, Error> {
        let len = self.len()?;
        let end = self.at.checked_add(len).ok_or_else(corrupt)?;
        let s = self.bytes.get(self.at..end).ok_or_else(corrupt)?;
        self.at = end;
        String::from_utf8(s.to_vec()).map_err(|_| corrupt())
    }

    fn value(&mut self) -> Result<Value, Error> {
        Ok(match self.byte()? {
            b's' => Value::String(self.string()?),
            b'i' => {
                let z = self.number()?;
                Value::Integer((z >> 1) as i64 ^ -((z & 1) as i64))
            }
            b'f' => Value::Float(f64::from_bits(self.number()?)),
            b't' => Value::Boolean(true),
            b'n' => Value::Boolean(false),
            b'a' => {
                let len = self.len()?;
                let mut out = Vec::with_capacity(len.min(self.bytes.len()));
                for _ in 0..len {
                    out.push(self.value()?);
                }
                Value::Array(out)
            }
            b'm' => {
                let mut out = Table::new();
                for _ in 0..self.len()? {
                    let key = self.string()?;
                    out.insert(key, self.value()?);
                }
                Value::Table(out)
            }
            _ => return Err(corrupt()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::pack::pack;
    use super::*;

    #[test]
    fn a_packed_file_decodes_to_its_toml() {
        let text = "a = -3\nb = 9223372036854775807\nc = -9223372036854775808\nd = 1.5e-3\n\
                    e = \"ünï ✓\"\nf = [true, false, [1]]\n[g.h]\ni = { j = \"\" }\n";
        let packed = pack(text).expect("packs");
        let direct: Value = toml::from_str(text).expect("toml");
        let read: Value = from_str(&packed).expect("decodes");
        assert_eq!(read, direct);
        assert!(from_str::<Value>(&packed[..packed.len() - 1]).is_err());
    }

    /// The built-in content, every file packed, loads to exactly what its TOML loads to: the
    /// value reader agrees with `toml` on every type content has (#1720).
    #[test]
    fn the_packed_built_in_content_loads_equal_to_its_toml() {
        use super::super::{Content, ContentFiles};
        let toml = ContentFiles::builtin();
        let mut packed = ContentFiles::default();
        for (path, text) in &toml.files {
            let form = pack(text).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(form.starts_with(MARK), "{path}");
            packed.insert(path, &form);
        }
        let from_toml = Content::load(&toml).expect("the TOML loads");
        let from_packed = Content::load(&packed).expect("the packed form loads");
        let (a, b) = (format!("{from_toml:#?}"), format!("{from_packed:#?}"));
        if a != b {
            let line = (a.lines().zip(b.lines()))
                .position(|(x, y)| x != y)
                .unwrap_or(0);
            let around = |s: &str| {
                s.lines()
                    .skip(line.saturating_sub(8))
                    .take(12)
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            panic!(
                "differ at line {line}:\nTOML:\n{}\npacked:\n{}",
                around(&a),
                around(&b)
            );
        }
    }
}
