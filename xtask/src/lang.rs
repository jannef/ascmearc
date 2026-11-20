//! `cargo xtask lang`: the translator's tools for the `fi` morphology (`docs/dev/localization.md`
//! "The `fi` module"; the checker, `--check`, is L7).
//!
//! - `--forms fi [WORD...]`: every form of every word in `assets/lang/fi/lexicon/*.toml`, and of
//!   each WORD given on the command line (`"pitkä|miekka/9*A"`), for a native speaker's review.
//! - `--suggest fi <KOTUS_XML> [WORD...]`: for every lexicon word without a class, and for each
//!   WORD given, the class and gradation the Kotus word list gives (a local copy of
//!   `kotus-sanalista_v1.xml`, its path on the command line; never embedded, never shipped), a
//!   compound by its last part; words the list lacks are named for the translator to decide.

use ascmearc_core::lang::fi::{CASES, Fi};
use ascmearc_core::lang::{LexWord, Lexicon, MapLexicon, Morphology, Phrase, Register};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub enum Mode {
    Forms { words: Vec<String> },
    Suggest { kotus: PathBuf, words: Vec<String> },
}

const USAGE: &str =
    "usage: cargo xtask lang <--forms fi [WORD...] | --suggest fi <KOTUS_XML> [WORD...]>";

pub fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [flag, code, rest @ ..] if flag == "--forms" && code == "fi" => Ok(Mode::Forms {
            words: rest.to_vec(),
        }),
        [flag, code, kotus, rest @ ..] if flag == "--suggest" && code == "fi" => {
            Ok(Mode::Suggest {
                kotus: PathBuf::from(kotus),
                words: rest.to_vec(),
            })
        }
        [_, code, ..] if code != "fi" => Err(format!("only `fi` has these tools ({USAGE})")),
        _ => Err(USAGE.to_string()),
    }
}

pub fn run(mode: Mode) -> Result<(), String> {
    match mode {
        Mode::Forms { words } => forms(&words),
        Mode::Suggest { kotus, words } => suggest(&kotus, &words),
    }
}

fn lexicon_dir() -> PathBuf {
    crate::workspace_root().join("assets/lang/fi/lexicon")
}

/// Every `id = word` of the Finnish lexicon files (prefixed by the file's name, `objects:`),
/// then the words given on the command line under their own text.
fn words(extra: &[String]) -> Result<Vec<(String, LexWord)>, String> {
    let mut out: Vec<(String, LexWord)> = Vec::new();
    let dir = lexicon_dir();
    if dir.is_dir() {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();
        for file in files {
            let text =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let lexicon = MapLexicon::parse(&format!("{stem}:"), &text)
                .map_err(|e| format!("{}: {e}", file.display()))?;
            for id in lexicon.ids() {
                if let Some(word) = lexicon.word(id) {
                    out.push((id.to_string(), word));
                }
            }
        }
    }
    for word in extra {
        out.push((word.clone(), LexWord::new(word.clone())));
    }
    Ok(out)
}

fn forms(extra: &[String]) -> Result<(), String> {
    let words = words(extra)?;
    if words.is_empty() {
        return Err(format!(
            "no words: none in {} and none given ({USAGE})",
            lexicon_dir().display()
        ));
    }
    let fi = Fi::default();
    let reg = Register::default();
    for (id, word) in words {
        println!("{id} = {}", word.base);
        if let Err(e) = fi.parse_word(&word.base) {
            if !(word.proper && !word.base.contains('/')) {
                println!("  refused: {e}; renders as written in every form");
                println!();
                continue;
            }
            println!("  a proper name: inflected as a foreign name");
        }
        let mut lexicon = MapLexicon::default();
        lexicon.insert("w", word.clone());
        let phrase = Phrase::id("w");
        let form = |name: &str| {
            fi.phrase(&phrase, name, &lexicon, &reg)
                .unwrap_or_else(|e| e.to_string())
        };
        for case in CASES {
            if *case == "com" || *case == "ins" {
                println!("  {case:<6} {:<24} {}", "", form(case));
            } else {
                println!(
                    "  {case:<6} {:<24} {}",
                    form(case),
                    form(&format!("pl_{case}"))
                );
            }
        }
        let poss = [
            "nom_ni",
            "nom_si",
            "nom_nsa",
            "ade_si",
            "ade_nsa",
            "ill_nsa",
            "pl_gen_nsa",
            "com_nsa",
        ]
        .map(|f| format!("{f}={}", form(f)));
        println!("  poss   {}", poss.join(" "));
        println!(
            "  3×     {}",
            fi.phrase(&Phrase::id("w").count(3), "counted", &lexicon, &reg)
                .unwrap_or_default()
        );
        println!();
    }
    Ok(())
}

/// The Kotus list: word → its `class[*letter]` marks (several for a word with several
/// inflections), read from the 2006 XML (`<st><s>miekka</s><t><tn>9</tn><av>A</av></t></st>`).
fn read_kotus(path: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut list: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for record in text.split("<st>").skip(1) {
        let record = record.split("</st>").next().unwrap_or("");
        let Some(word) = between(record, "<s>", "</s>") else {
            continue;
        };
        let mut marks: Vec<String> = Vec::new();
        for t in record.split("<tn>").skip(1) {
            let Some(class) = t.split("</tn>").next() else {
                continue;
            };
            let mark = match between(t, "<av>", "</av>") {
                Some(letter) => format!("{}*{letter}", class.trim()),
                None => class.trim().to_string(),
            };
            marks.push(mark);
        }
        list.entry(word.to_string()).or_default().extend(marks);
    }
    if list.is_empty() {
        return Err(format!(
            "{}: no `<st><s>…</s>…</st>` records; is it the Kotus word list XML?",
            path.display()
        ));
    }
    Ok(list)
}

fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = text.find(open)? + open.len();
    let end = text[start..].find(close)? + start;
    Some(text[start..end].trim())
}

/// `word` without its `/class` mark, and the mark.
fn split_mark(word: &str) -> (&str, Option<&str>) {
    match word.rfind('/') {
        Some(slash) => (&word[..slash], Some(&word[slash + 1..])),
        None => (word, None),
    }
}

fn suggest(kotus: &Path, extra: &[String]) -> Result<(), String> {
    let list = read_kotus(kotus)?;
    let words = words(extra)?;
    if words.is_empty() {
        return Err(format!(
            "no words: none in {} and none given ({USAGE})",
            lexicon_dir().display()
        ));
    }
    let mut missing = 0;
    for (id, word) in words {
        if word.proper && !word.base.contains('/') {
            continue;
        }
        for token in word.base.split_whitespace() {
            let (base, mark) = split_mark(token);
            let has_class =
                mark.is_some_and(|m| m.chars().next().is_some_and(|c| c.is_ascii_digit()));
            let base = base.replace('|', "");
            let lookup = lookup(&list, &base);
            match (has_class, lookup) {
                (true, Some((found, marks))) => {
                    let mark = mark.unwrap_or("");
                    if !marks.iter().any(|m| m == mark) {
                        println!(
                            "{id}: {token} differs from the list, which says {}{}",
                            marks.join(" or "),
                            by(&base, found)
                        );
                    }
                }
                (true, None) => {}
                (false, Some((found, marks))) => {
                    println!("{id}: {base} → {}{}", marks.join(" or "), by(&base, found));
                }
                (false, None) => {
                    missing += 1;
                    println!(
                        "{id}: {base} is not in the Kotus list (nor is any ending of it): decide its class"
                    );
                }
            }
        }
    }
    if missing > 0 {
        println!("{missing} word(s) the list lacks");
    }
    Ok(())
}

fn by(base: &str, found: &str) -> String {
    if found == base {
        String::new()
    } else {
        format!(" (by its last part, {found})")
    }
}

/// The list's entry for `base`, else for its longest ending of three letters or more that is a
/// word of the list (a compound by its last part), lower-cased.
fn lookup<'a>(
    list: &'a BTreeMap<String, Vec<String>>,
    base: &str,
) -> Option<(&'a str, &'a [String])> {
    let lower = base.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    for start in 0..chars.len() {
        if chars.len() - start < 3 && start > 0 {
            break;
        }
        let tail: String = chars[start..].iter().collect();
        if let Some((word, marks)) = list.get_key_value(&tail)
            && !marks.is_empty()
        {
            return Some((word.as_str(), marks.as_slice()));
        }
    }
    None
}
