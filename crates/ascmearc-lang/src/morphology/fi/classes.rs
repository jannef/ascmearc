//! The Kotus inflection classes of nominals (Kotus, "Nykysuomen sanalista", types 1–49): the word
//! shape each accepts and how its stems come from the lemma (`docs/dev/localization.md` "The
//! `fi` module"). A class is one row of [`SPECS`] and one arm of [`stems`]; the forms themselves
//! are built from the stems in [`super::inflect`]. Verified against the Wiktionary oracle in
//! `oracle.toml`, class by class.

use super::inflect::{
    Grad, Grade, Harmony, Ill, Par, PlGen, PlIll, Spec, Stems, cut, ends_in_long_vowel, ends_with,
    is_vowel, strengthen, text, vowel_runs, weaken, with,
};

/// The classes this module inflects. Not here: 31 (`kaksi`), 36 (`sisin`), 37 (`vasen`), 45
/// (`kahdeksas`), 46 (`tuhat`), and the compound types 50 and 51; a word of those is refused with
/// "not implemented", and `forms` can spell it out.
pub(super) static SPECS: &[Spec] = &[
    spec(
        1,
        "valo",
        "a word ending in o, ö, u, or y",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        2,
        "palvelu",
        "a word ending in o, ö, u, or y",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        3,
        "valtio",
        "a word ending in o, ö, u, or y",
        Grade::Direct,
        Par::Ta,
        Ill::Vn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        4,
        "laatikko",
        "a word ending in o, ö, u, or y",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        5,
        "risti",
        "a word ending in i or in a consonant",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::SgEn,
        PlIll::Hin,
    ),
    spec(
        6,
        "paperi",
        "a word ending in i or in a consonant",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        true,
        PlGen::SgEn,
        PlIll::Hin,
    ),
    spec(
        7,
        "ovi",
        "a word ending in a consonant and i",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        8,
        "nalle",
        "a word ending in e",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        9,
        "kala",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        10,
        "koira",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        11,
        "omena",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        12,
        "kulkija",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        13,
        "katiska",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        14,
        "solakka",
        "a word ending in a or ä",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::Jen,
        PlIll::Hin,
    ),
    spec(
        15,
        "korkea",
        "a word ending in a vowel and a or ä (ea, eä, oa)",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        16,
        "vanhempi",
        "a comparative ending in mpi",
        Grade::Direct,
        Par::A,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        17,
        "vapaa",
        "a word of two or more syllables ending in a long vowel",
        Grade::None,
        Par::Ta,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        18,
        "maa",
        "a word of one syllable ending in a long vowel",
        Grade::None,
        Par::Ta,
        Ill::HVn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        19,
        "suo",
        "a word of one syllable ending in uo, ie, or yö",
        Grade::None,
        Par::Ta,
        Ill::HVn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        20,
        "filee",
        "a word of two or more syllables ending in a long vowel",
        Grade::None,
        Par::Ta,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        21,
        "rosé",
        "a foreign word ending in a vowel",
        Grade::None,
        Par::Ta,
        Ill::HVn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        22,
        "parfait",
        "a foreign word ending in a silent consonant",
        Grade::None,
        Par::Ta,
        Ill::HVn,
        true,
        PlGen::Den,
        PlIll::Hin,
    ),
    spec(
        23,
        "tiili",
        "a word ending in a consonant and i",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        24,
        "uni",
        "a word ending in a consonant and i",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        25,
        "toimi",
        "a word ending in mi",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        26,
        "pieni",
        "a word ending in a consonant and i",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        27,
        "käsi",
        "a word ending in si",
        Grade::Direct,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        28,
        "kynsi",
        "a word ending in nsi, rsi, or lsi",
        Grade::Direct,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        29,
        "lapsi",
        "a word ending in psi",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::Ten,
        PlIll::In,
    ),
    spec(
        30,
        "veitsi",
        "a word ending in tsi",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        32,
        "sisar",
        "a word ending in a consonant",
        Grade::Inverse,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        33,
        "kytkin",
        "a word ending in a vowel and n",
        Grade::Inverse,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        34,
        "onneton",
        "a word ending in ton or tön",
        Grade::Inverse,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        35,
        "lämmin",
        "a word ending in a vowel and n",
        Grade::Inverse,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        38,
        "nainen",
        "a word ending in nen",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::Ten,
        PlIll::In,
    ),
    spec(
        39,
        "vastaus",
        "a word ending in a vowel and s",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::Ten,
        PlIll::In,
    ),
    spec(
        40,
        "kalleus",
        "a word ending in us or ys",
        Grade::Direct,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
    spec(
        41,
        "vieras",
        "a word ending in a vowel and s",
        Grade::Inverse,
        Par::Cons,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        42,
        "mies",
        "mies, or a compound ending in it",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::Ten,
        PlIll::In,
    ),
    spec(
        43,
        "ohut",
        "a word ending in ut or yt",
        Grade::None,
        Par::Cons,
        Ill::Vn,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        44,
        "kevät",
        "a word ending in a vowel and t",
        Grade::None,
        Par::Cons,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        47,
        "kuollut",
        "a participle ending in a consonant and ut or yt",
        Grade::None,
        Par::Cons,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        48,
        "hame",
        "a word ending in e",
        Grade::Inverse,
        Par::Cons,
        Ill::Seen,
        true,
        PlGen::Den,
        PlIll::Siin,
    ),
    spec(
        49,
        "askel",
        "a word ending in a consonant",
        Grade::Inverse,
        Par::Cons,
        Ill::Vn,
        false,
        PlGen::PlEn,
        PlIll::In,
    ),
];

#[allow(clippy::too_many_arguments)]
const fn spec(
    class: u8,
    example: &'static str,
    shape: &'static str,
    grade: Grade,
    par: Par,
    ill: Ill,
    pl_par_t: bool,
    pl_gen: PlGen,
    pl_ill: PlIll,
) -> Spec {
    Spec {
        class,
        example,
        shape,
        grade,
        par,
        ill,
        pl_par_t,
        pl_gen,
        pl_ill,
    }
}

pub(super) fn spec_of(class: u8) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.class == class)
}

/// The Kotus classes this module leaves out, for the error message.
pub(super) fn is_kotus_class(class: u8) -> bool {
    (1..=51).contains(&class)
}

fn last(w: &[char]) -> char {
    w.last().copied().unwrap_or(' ')
}

fn nth_last(w: &[char], n: usize) -> char {
    w.len()
        .checked_sub(n)
        .and_then(|i| w.get(i))
        .copied()
        .unwrap_or(' ')
}

fn is_consonant(c: char) -> bool {
    c.is_alphabetic() && !is_vowel(c)
}

/// Whether `w` (the inflecting part of the lemma) has the shape class `class` accepts.
pub(super) fn fits(class: u8, w: &[char]) -> bool {
    let l = last(w);
    let l2 = nth_last(w, 2);
    let lower = l.to_lowercase().next().unwrap_or(l);
    match class {
        1..=4 => matches!(lower, 'o' | 'ö' | 'u' | 'y'),
        5 | 6 => w.len() >= 2 && (lower == 'i' || is_consonant(l)),
        7 | 23 | 24 | 26 => w.len() >= 2 && lower == 'i' && is_consonant(l2),
        8 | 48 => lower == 'e',
        9..=14 => matches!(lower, 'a' | 'ä'),
        15 => matches!(lower, 'a' | 'ä') && is_vowel(l2) && l2 != l,
        16 => ends_with(w, "mpi"),
        17 | 20 => ends_in_long_vowel(w) && vowel_runs(w) >= 2,
        18 => ends_in_long_vowel(w) && vowel_runs(w) == 1,
        19 => {
            vowel_runs(w) == 1 && (ends_with(w, "uo") || ends_with(w, "ie") || ends_with(w, "yö"))
        }
        21 => is_vowel(l),
        22 | 32 | 49 => is_consonant(l),
        25 => ends_with(w, "mi"),
        27 => w.len() >= 3 && ends_with(w, "si"),
        28 => w.len() >= 3 && ends_with(w, "si") && matches!(nth_last(w, 3), 'n' | 'r' | 'l'),
        29 => ends_with(w, "psi"),
        30 => ends_with(w, "tsi"),
        33 | 35 => w.len() >= 3 && lower == 'n' && is_vowel(l2),
        34 => ends_with(w, "ton") || ends_with(w, "tön"),
        38 => w.len() >= 4 && ends_with(w, "nen"),
        39 | 41 => w.len() >= 3 && lower == 's' && is_vowel(l2),
        40 => ends_with(w, "us") || ends_with(w, "ys"),
        42 => ends_with(w, "mies"),
        43 => ends_with(w, "ut") || ends_with(w, "yt"),
        44 => w.len() >= 3 && lower == 't' && is_vowel(l2),
        47 => {
            w.len() >= 4
                && (ends_with(w, "ut") || ends_with(w, "yt"))
                && is_consonant(nth_last(w, 3))
        }
        _ => false,
    }
}

/// The stems of `w`, a word of `class` with the gradation `grad`, or why the entry is refused.
/// `w` has passed [`fits`].
pub(super) fn stems(
    class: u8,
    w: &[char],
    grad: Option<Grad>,
    harmony: Harmony,
) -> Result<Stems, String> {
    let Some(spec) = spec_of(class) else {
        return Err(if is_kotus_class(class) {
            format!("class {class} not implemented: spell its forms out with `forms`")
        } else {
            format!("class {class} is not a Kotus class (1–49)")
        });
    };
    let a = harmony.ending("A");
    let l = last(w);
    // The gradation letter a class has built in (`käsi` : `käden` is 27 whatever the entry says),
    // and whether the class takes one at all.
    let inherent: Option<Grad> = match class {
        16 => Some(Grad::H),
        27 | 40 => Some(Grad::F),
        28 => match nth_last(w, 3) {
            'n' => Some(Grad::J),
            'r' => Some(Grad::K),
            _ => Some(Grad::I),
        },
        _ => None,
    };
    let grad = match (grad, inherent) {
        (Some(given), Some(own)) if given != own => {
            return Err(format!(
                "class {class} gradates {own} by itself, not {given}"
            ));
        }
        (given, own) => given.or(own),
    };
    if grad.is_some() && spec.grade == Grade::None {
        return Err(format!("class {class} ({}) does not gradate", spec.example));
    }
    // Inverse classes: the lemma is weak; the stems come from its strong form.
    let strong_w: Vec<char> = match (spec.grade, grad) {
        (Grade::Inverse, Some(g)) => strengthen(w, g)?,
        _ => w.to_vec(),
    };
    let sw = strong_w.as_slice();
    let mut cons: Option<Vec<char>> = None;
    let (strong, pl): (Vec<char>, Vec<char>) = match class {
        1..=4 | 8 | 21 => (w.to_vec(), with(w.to_vec(), "i")),
        5 | 6 => {
            let stem = if is_vowel(l) {
                w.to_vec()
            } else {
                with(w.to_vec(), "i")
            };
            let pl = with(cut(&stem, 1), "ei");
            (stem, pl)
        }
        7 => (with(cut(w, 1), "e"), w.to_vec()),
        9 | 13 | 14 | 12 => {
            let o = if l == 'a' || l == 'A' { "oi" } else { "öi" };
            (w.to_vec(), with(cut(w, 1), o))
        }
        10 | 11 | 15 | 17 | 18 | 20 => (w.to_vec(), with(cut(w, 1), "i")),
        16 => (with(cut(w, 1), &a), w.to_vec()),
        19 => {
            let pl = match (nth_last(w, 2), l) {
                ('u', 'o') => "oi",
                ('y', 'ö') => "öi",
                _ => "ei",
            };
            (w.to_vec(), with(cut(w, 2), pl))
        }
        22 => (with(w.to_vec(), "'"), with(w.to_vec(), "'i")),
        23 | 24 | 26 => {
            cons = Some(cut(w, 1));
            (with(cut(w, 1), "e"), w.to_vec())
        }
        25 => {
            cons = Some(with(cut(w, 2), "n"));
            (with(cut(w, 1), "e"), w.to_vec())
        }
        27 | 28 => {
            cons = Some(with(cut(w, 2), "t"));
            (with(cut(w, 2), "te"), w.to_vec())
        }
        29 | 30 => {
            cons = Some(with(cut(w, 3), "s"));
            (with(cut(w, 1), "e"), w.to_vec())
        }
        32 | 49 => {
            cons = Some(w.to_vec());
            (with(sw.to_vec(), "e"), with(sw.to_vec(), "i"))
        }
        33..=35 => {
            cons = Some(w.to_vec());
            // 33 `kytkin` : `kytkime-`, `sydän` : `sydäme-`, but `hapan` : `happama-`; 34 and
            // 35 take the harmony vowel (`onnettoma-`, `lämpimä-`).
            let before = nth_last(sw, 2);
            let vowel = if class == 33 && !matches!(before, 'a' | 'A') {
                "e".to_string()
            } else {
                a.clone()
            };
            let base = cut(sw, 1);
            (with(with(base.clone(), "m"), &vowel), with(base, "mi"))
        }
        38 => {
            cons = Some(with(cut(w, 3), "s"));
            (with(cut(w, 3), "se"), with(cut(w, 3), "si"))
        }
        39 => {
            cons = Some(w.to_vec());
            (with(cut(w, 1), "kse"), with(cut(w, 1), "ksi"))
        }
        40 => {
            cons = Some(with(cut(w, 1), "t"));
            (with(cut(w, 1), "te"), with(cut(w, 1), "ksi"))
        }
        41 | 44 => {
            cons = Some(w.to_vec());
            let v = nth_last(sw, 2);
            let mut strong = cut(sw, 1);
            strong.push(v);
            (strong, with(cut(sw, 1), "i"))
        }
        42 => {
            cons = Some(w.to_vec());
            (with(cut(w, 1), "he"), with(cut(w, 1), "hi"))
        }
        43 => {
            cons = Some(w.to_vec());
            (with(cut(w, 1), "e"), with(cut(w, 1), "i"))
        }
        47 => {
            cons = Some(w.to_vec());
            (with(cut(w, 2), "ee"), with(cut(w, 2), "ei"))
        }
        48 => {
            cons = Some(with(w.to_vec(), "t"));
            (with(sw.to_vec(), "e"), with(sw.to_vec(), "i"))
        }
        _ => return Err(format!("class {class} not implemented")),
    };
    // Weak stems: direct gradation weakens the strong stems; the plural stem of 27, 28, and 40
    // (`käsi-`, `kynsi-`, `kalleuksi-`) has no cluster to gradate and stays as it is.
    let (weak, pl_weak) = match (spec.grade, grad) {
        (Grade::Direct, Some(g)) => {
            let weak = weaken(&strong, g)?;
            let pl_weak = if matches!(class, 27 | 28 | 40) {
                pl.clone()
            } else {
                weaken(&pl, g)?
            };
            (weak, pl_weak)
        }
        _ => (strong.clone(), pl.clone()),
    };
    if strong.is_empty() || pl.is_empty() {
        return Err(format!("`{}` is too short for class {class}", text(w)));
    }
    Ok(Stems {
        nom: w.to_vec(),
        strong,
        weak,
        cons,
        pl_strong: pl,
        pl_weak,
        harmony,
        spec,
    })
}
