//! The stems of a Finnish nominal and the case forms built on them (`docs/dev/localization.md`
//! "The `fi` module"). Everything here works on `Vec<char>`: ä and ö are two bytes, so a byte
//! slice could split one and panic, and a char vector cannot. Nothing here fails: a stem the
//! class rules could not derive never reaches these functions ([`super::classes::stems`] refuses
//! the entry first), and every lookup has a default.

use std::fmt;

/// The suffix vowel series a word takes: `a`/`o`/`u` after a back vowel, `ä`/`ö`/`y` otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Harmony {
    Back,
    Front,
}

impl Harmony {
    /// The harmony of a word by its last back or front vowel; a word with neither (only `e`
    /// and `i`, or no vowel at all: "Hild") takes the front series, as Finnish does.
    pub(super) fn of(word: &[char]) -> Harmony {
        for &c in word.iter().rev() {
            if is_back(c) {
                return Harmony::Back;
            }
            if is_front(c) {
                return Harmony::Front;
            }
        }
        Harmony::Front
    }

    /// `A`, `O`, `U` in an ending as this harmony says them.
    fn vowel(self, archiphoneme: char) -> char {
        match (self, archiphoneme) {
            (Harmony::Back, 'A') => 'a',
            (Harmony::Front, 'A') => 'ä',
            (Harmony::Back, 'O') => 'o',
            (Harmony::Front, 'O') => 'ö',
            (Harmony::Back, 'U') => 'u',
            (Harmony::Front, 'U') => 'y',
            (_, other) => other,
        }
    }

    /// An ending written with `A`/`O`/`U` for its harmony vowels, resolved.
    pub(super) fn ending(self, pattern: &str) -> String {
        pattern.chars().map(|c| self.vowel(c)).collect()
    }
}

/// The vowels, with the accented ones a foreign name carries and the Old English letters (the
/// long `ā ē ī ō ū ȳ ǣ` and `æ`: `Līgetdrȳ`, `Lytelscræf`; owner review 3). Non-ASCII capitals
/// are listed, as `to_ascii_lowercase` leaves them.
pub(super) fn is_vowel(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'ä'
            | 'ö'
            | 'å'
            | 'é'
            | 'è'
            | 'á'
            | 'à'
            | 'ó'
            | 'ú'
            | 'ü'
            | 'æ'
    ) || matches!(c, 'Ä' | 'Ö' | 'Å' | 'É' | 'Ü' | 'Æ')
        || is_long_vowel(c)
}

/// A long vowel written as one letter: the Old English macron vowels and `ǣ`.
pub(super) fn is_long_vowel(c: char) -> bool {
    matches!(
        c,
        'ā' | 'ē' | 'ī' | 'ō' | 'ū' | 'ȳ' | 'ǣ' | 'Ā' | 'Ē' | 'Ī' | 'Ō' | 'Ū' | 'Ȳ' | 'Ǣ'
    )
}

fn is_back(c: char) -> bool {
    matches!(
        c,
        'a' | 'o'
            | 'u'
            | 'å'
            | 'á'
            | 'à'
            | 'ó'
            | 'ú'
            | 'ā'
            | 'ō'
            | 'ū'
            | 'A'
            | 'O'
            | 'U'
            | 'Å'
            | 'Ā'
            | 'Ō'
            | 'Ū'
    )
}

fn is_front(c: char) -> bool {
    matches!(
        c,
        'ä' | 'ö' | 'y' | 'ü' | 'ȳ' | 'æ' | 'ǣ' | 'Ä' | 'Ö' | 'Y' | 'Ü' | 'Ȳ' | 'Æ' | 'Ǣ'
    )
}

/// Whether `word` ends in a long vowel: two equal vowels, or one letter with a macron (`ȳ`).
pub(super) fn ends_in_long_vowel(word: &[char]) -> bool {
    match word {
        [.., c] if is_long_vowel(*c) => true,
        [.., a, b] => is_vowel(*a) && a.eq_ignore_ascii_case(b),
        _ => false,
    }
}

/// The number of vowel runs: a rough syllable count ("maa" 1, "vapaa" 2).
pub(super) fn vowel_runs(word: &[char]) -> usize {
    let mut runs = 0;
    let mut in_run = false;
    for &c in word {
        let v = is_vowel(c);
        if v && !in_run {
            runs += 1;
        }
        in_run = v;
    }
    runs
}

/// Whether `word` ends with `suffix` (ASCII case-insensitive, so a name in capitals still fits).
pub(super) fn ends_with(word: &[char], suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    word.len() >= suffix.len()
        && word[word.len() - suffix.len()..]
            .iter()
            .zip(&suffix)
            .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
}

/// `word` without its last `n` chars (all of it gone when it is shorter).
pub(super) fn cut(word: &[char], n: usize) -> Vec<char> {
    word[..word.len().saturating_sub(n)].to_vec()
}

pub(super) fn with(mut word: Vec<char>, tail: &str) -> Vec<char> {
    word.extend(tail.chars());
    word
}

pub(super) fn text(word: &[char]) -> String {
    word.iter().collect()
}

/// The vowel an illative `-hVn` or a lengthening `-Vn` copies: the stem's last vowel, with an
/// accented or long letter read as its plain one (`rosé` → `roséhen`, `Līgetdrȳ` →
/// `Līgetdrȳhyn`); `i` when the stem has none.
pub(super) fn last_vowel(word: &[char]) -> char {
    word.iter()
        .rev()
        .find(|c| is_vowel(**c))
        .map_or('i', |c| match c.to_ascii_lowercase() {
            'é' | 'è' | 'ē' | 'Ē' => 'e',
            'á' | 'à' | 'ā' | 'Ā' => 'a',
            'å' | 'ó' | 'ō' | 'Ō' => 'o',
            'ú' | 'ū' | 'Ū' => 'u',
            'ü' | 'ȳ' | 'Ȳ' => 'y',
            'ī' | 'Ī' => 'i',
            'æ' | 'ǣ' | 'Æ' | 'Ǣ' => 'ä',
            other => other,
        })
}

// ----- Consonant gradation -----

/// The Kotus gradation letters: the strong and weak grade of the consonant(s) before the last
/// vowel of the stem (`kk`~`k` for `miekka` : `miekan`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Grad {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
}

impl Grad {
    pub(super) fn parse(letter: char) -> Option<Grad> {
        Some(match letter {
            'A' => Grad::A,
            'B' => Grad::B,
            'C' => Grad::C,
            'D' => Grad::D,
            'E' => Grad::E,
            'F' => Grad::F,
            'G' => Grad::G,
            'H' => Grad::H,
            'I' => Grad::I,
            'J' => Grad::J,
            'K' => Grad::K,
            'L' => Grad::L,
            'M' => Grad::M,
            _ => return None,
        })
    }

    pub(super) fn letter(self) -> char {
        match self {
            Grad::A => 'A',
            Grad::B => 'B',
            Grad::C => 'C',
            Grad::D => 'D',
            Grad::E => 'E',
            Grad::F => 'F',
            Grad::G => 'G',
            Grad::H => 'H',
            Grad::I => 'I',
            Grad::J => 'J',
            Grad::K => 'K',
            Grad::L => 'L',
            Grad::M => 'M',
        }
    }

    /// (strong, weak).
    fn grades(self) -> (&'static str, &'static str) {
        match self {
            Grad::A => ("kk", "k"),
            Grad::B => ("pp", "p"),
            Grad::C => ("tt", "t"),
            Grad::D => ("k", ""),
            Grad::E => ("p", "v"),
            Grad::F => ("t", "d"),
            Grad::G => ("nk", "ng"),
            Grad::H => ("mp", "mm"),
            Grad::I => ("lt", "ll"),
            Grad::J => ("nt", "nn"),
            Grad::K => ("rt", "rr"),
            Grad::L => ("k", "j"),
            Grad::M => ("k", "v"),
        }
    }

    /// Clusters that look like this letter's strong grade but belong to another letter, so the
    /// letter was wrong for the word (`*F` on `ranta`, whose `nt` is `J`).
    fn not_strong(self) -> &'static [&'static str] {
        match self {
            Grad::D | Grad::L | Grad::M => &["kk", "nk"],
            Grad::E => &["pp", "mp"],
            Grad::F => &["tt", "nt", "lt", "rt"],
            _ => &[],
        }
    }

    fn not_weak(self) -> &'static [&'static str] {
        match self {
            Grad::A => &["kk"],
            Grad::B => &["pp"],
            Grad::C => &["tt"],
            _ => &[],
        }
    }
}

impl fmt::Display for Grad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (strong, weak) = self.grades();
        let weak = if weak.is_empty() { "∅" } else { weak };
        write!(f, "{} ({strong}~{weak})", self.letter())
    }
}

/// The consonant cluster before the stem's last vowel run (`miek|ka`, `ren|g|as`): where
/// gradation happens. Returns (start, end) of the cluster in `stem`, skipping any consonants
/// after the last vowel (`rengas`: the `s`).
fn cluster(stem: &[char]) -> (usize, usize) {
    let mut end = stem.len();
    while end > 0 && !is_vowel(stem[end - 1]) {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_vowel(stem[start - 1]) {
        start -= 1;
    }
    let vowel_start = start;
    while start > 0 && !is_vowel(stem[start - 1]) {
        start -= 1;
    }
    (start, vowel_start)
}

/// `stem` in the weak grade (`miekka` → `mieka`), or why the letter does not fit the word.
pub(super) fn weaken(stem: &[char], grad: Grad) -> Result<Vec<char>, String> {
    let (start, end) = cluster(stem);
    let cluster: Vec<char> = stem[start..end].to_vec();
    let (strong, weak) = grad.grades();
    let fits =
        ends_with(&cluster, strong) && !grad.not_strong().iter().any(|c| ends_with(&cluster, c));
    if !fits {
        return Err(format!(
            "gradation {grad} needs {strong} before the last vowel of `{}`",
            text(stem)
        ));
    }
    let mut out = stem[..end - strong.chars().count()].to_vec();
    out.extend(weak.chars());
    out.extend_from_slice(&stem[end..]);
    Ok(out)
}

/// `stem` in the strong grade (`lääke` → `lääkke`, `varas` → `varkas`), or why not.
pub(super) fn strengthen(stem: &[char], grad: Grad) -> Result<Vec<char>, String> {
    let (start, end) = cluster(stem);
    let cluster: Vec<char> = stem[start..end].to_vec();
    let (strong, weak) = grad.grades();
    let fits = ends_with(&cluster, weak) && !grad.not_weak().iter().any(|c| ends_with(&cluster, c));
    if !fits {
        let weak = if weak.is_empty() {
            "no consonant"
        } else {
            weak
        };
        return Err(format!(
            "gradation {grad} needs {weak} before the last vowel of `{}`",
            text(stem)
        ));
    }
    let mut out = stem[..end - weak.chars().count()].to_vec();
    out.extend(strong.chars());
    out.extend_from_slice(&stem[end..]);
    Ok(out)
}

// ----- Stems and forms -----

/// How the singular partitive is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Par {
    /// The vowel stem and `A` (`miekkaa`).
    A,
    /// The vowel stem and `tA` (`maata`, `vapaata`).
    Ta,
    /// The consonant stem and `tA` (`kättä`, `naista`).
    Cons,
}

/// How the singular illative is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ill {
    /// The last vowel lengthened and `n` (`miekkaan`).
    Vn,
    /// `h`, the last vowel, `n` (`maahan`, `tiehen`).
    HVn,
    /// `seen` (`vapaaseen`, `kirveeseen`).
    Seen,
}

/// How the plural genitive is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlGen {
    /// The plural stem's `i` as `j` and `en` (`valojen`).
    Jen,
    /// The plural stem and `en` (`koirien`).
    PlEn,
    /// The singular strong stem and `en` (`ristien`).
    SgEn,
    /// The plural stem and `den` (`maiden`).
    Den,
    /// The consonant stem and `ten` (`naisten`).
    Ten,
}

/// How the plural illative is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlIll {
    Hin,
    In,
    Siin,
}

/// Which forms take the weak grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Grade {
    /// No gradation in this class.
    None,
    /// The lemma is strong; the genitive and the like are weak (`miekka` : `miekan`).
    Direct,
    /// The lemma is weak; the vowel stem is strong (`lääke` : `lääkkeen`).
    Inverse,
}

/// A class's rules beyond its stems.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Spec {
    pub class: u8,
    pub example: &'static str,
    pub shape: &'static str,
    pub grade: Grade,
    pub par: Par,
    pub ill: Ill,
    pub pl_par_t: bool,
    pub pl_gen: PlGen,
    pub pl_ill: PlIll,
}

/// The stems a word inflects from, with its rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Stems {
    /// The written singular nominative (`parfait`, `kuningas`).
    pub nom: Vec<char>,
    /// The singular vowel stem, strong grade (`miekka`, `käte`, `kuninkaa`, `parfait'`).
    pub strong: Vec<char>,
    /// The singular vowel stem, weak grade (`mieka`, `käde`); the strong one where nothing
    /// gradates.
    pub weak: Vec<char>,
    /// The consonant stem of the partitive and the `-ten` genitive (`kät`, `nais`, `kuningas`).
    pub cons: Option<Vec<char>>,
    /// The plural `i`-stem, strong (`miekkoi`, `käsi`) and weak (`miekoi`).
    pub pl_strong: Vec<char>,
    pub pl_weak: Vec<char>,
    pub harmony: Harmony,
    pub spec: &'static Spec,
}

/// The fourteen cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Case {
    Nom,
    Gen,
    Par,
    Ine,
    Ela,
    Ill,
    Ade,
    Abl,
    All,
    Ess,
    Tra,
    Abe,
    /// Plural only (`miekkoine-`, said with a possessive suffix: `miekkoineen`).
    Com,
    /// Plural only (`miekoin`), no possessive.
    Ins,
}

impl Case {
    pub(super) const ALL: [Case; 14] = [
        Case::Nom,
        Case::Gen,
        Case::Par,
        Case::Ine,
        Case::Ela,
        Case::Ill,
        Case::Ade,
        Case::Abl,
        Case::All,
        Case::Ess,
        Case::Tra,
        Case::Abe,
        Case::Com,
        Case::Ins,
    ];

    /// The twelve cases with a singular.
    pub(super) const SINGULAR: [Case; 12] = [
        Case::Nom,
        Case::Gen,
        Case::Par,
        Case::Ine,
        Case::Ela,
        Case::Ill,
        Case::Ade,
        Case::Abl,
        Case::All,
        Case::Ess,
        Case::Tra,
        Case::Abe,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Case::Nom => "nom",
            Case::Gen => "gen",
            Case::Par => "par",
            Case::Ine => "ine",
            Case::Ela => "ela",
            Case::Ill => "ill",
            Case::Ade => "ade",
            Case::Abl => "abl",
            Case::All => "all",
            Case::Ess => "ess",
            Case::Tra => "tra",
            Case::Abe => "abe",
            Case::Com => "com",
            Case::Ins => "ins",
        }
    }

    pub(super) fn parse(name: &str) -> Option<Case> {
        Case::ALL.into_iter().find(|c| c.name() == name)
    }

    pub(super) fn plural_only(self) -> bool {
        matches!(self, Case::Com | Case::Ins)
    }
}

/// The possessive suffixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Poss {
    /// -ni, mine.
    Ni,
    /// -si, yours.
    Si,
    /// -nsa/-nsä or -Vn, his, hers, its, theirs.
    Nsa,
    /// -mme, ours.
    Mme,
    /// -nne, yours (plural).
    Nne,
}

impl Poss {
    pub(super) const ALL: [Poss; 5] = [Poss::Ni, Poss::Si, Poss::Nsa, Poss::Mme, Poss::Nne];

    pub(super) fn name(self) -> &'static str {
        match self {
            Poss::Ni => "ni",
            Poss::Si => "si",
            Poss::Nsa => "nsa",
            Poss::Mme => "mme",
            Poss::Nne => "nne",
        }
    }

    pub(super) fn parse(name: &str) -> Option<Poss> {
        Poss::ALL.into_iter().find(|p| p.name() == name)
    }
}

impl Stems {
    fn e(&self, pattern: &str) -> String {
        self.harmony.ending(pattern)
    }

    fn cons(&self) -> &[char] {
        self.cons.as_deref().unwrap_or(&self.strong)
    }

    /// A case form, singular or plural (`com` and `ins` are plural whatever is asked).
    pub(super) fn form(&self, plural: bool, case: Case) -> String {
        if plural || case.plural_only() {
            self.plural(case)
        } else {
            self.singular(case)
        }
    }

    fn singular(&self, case: Case) -> String {
        let (stem, ending): (&[char], String) = match case {
            Case::Nom => (&self.nom, String::new()),
            Case::Gen => (&self.weak, "n".into()),
            Case::Par => match self.spec.par {
                Par::A => (&self.strong, self.e("A")),
                Par::Ta => (&self.strong, self.e("tA")),
                Par::Cons => (self.cons(), self.e("tA")),
            },
            Case::Ine => (&self.weak, self.e("ssA")),
            Case::Ela => (&self.weak, self.e("stA")),
            Case::Ill => {
                let v = last_vowel(&self.strong);
                match self.spec.ill {
                    Ill::Vn if self.strong.last().is_some_and(|c| is_vowel(*c)) => {
                        (&self.strong, format!("{v}n"))
                    }
                    Ill::Vn => (&self.strong, "iin".into()),
                    Ill::HVn => (&self.strong, format!("h{v}n")),
                    Ill::Seen => (&self.strong, "seen".into()),
                }
            }
            Case::Ade => (&self.weak, self.e("llA")),
            Case::Abl => (&self.weak, self.e("ltA")),
            Case::All => (&self.weak, "lle".into()),
            Case::Ess => (&self.strong, self.e("nA")),
            Case::Tra => (&self.weak, "ksi".into()),
            Case::Abe => (&self.weak, self.e("ttA")),
            Case::Com | Case::Ins => return self.plural(case),
        };
        let mut out = text(stem);
        out.push_str(&ending);
        out
    }

    /// Whether the plural stem ends in a vowel and `i` (`valoi`, where the `i` turns to `j`
    /// before a vowel), as against a consonant and `i` (`koiri`).
    fn pl_stem_vi(&self) -> bool {
        matches!(self.pl_strong.as_slice(), [.., a, b] if is_vowel(*a) && *b == 'i')
    }

    fn pl_j(&self) -> Vec<char> {
        let mut stem = cut(&self.pl_strong, 1);
        stem.push('j');
        stem
    }

    fn plural(&self, case: Case) -> String {
        let (stem, ending): (Vec<char>, String) = match case {
            Case::Nom => (self.weak.clone(), "t".into()),
            Case::Gen => match self.spec.pl_gen {
                PlGen::Jen => (self.pl_j(), "en".into()),
                PlGen::PlEn => (self.pl_strong.clone(), "en".into()),
                PlGen::SgEn => (self.strong.clone(), "en".into()),
                PlGen::Den => (self.pl_strong.clone(), "den".into()),
                PlGen::Ten => (self.cons().to_vec(), "ten".into()),
            },
            Case::Par => {
                if self.spec.pl_par_t {
                    (self.pl_strong.clone(), self.e("tA"))
                } else if self.pl_stem_vi() {
                    (self.pl_j(), self.e("A"))
                } else {
                    (self.pl_strong.clone(), self.e("A"))
                }
            }
            Case::Ine => (self.pl_weak.clone(), self.e("ssA")),
            Case::Ela => (self.pl_weak.clone(), self.e("stA")),
            Case::Ill => match self.spec.pl_ill {
                PlIll::Hin => (self.pl_strong.clone(), "hin".into()),
                PlIll::In => (self.pl_strong.clone(), "in".into()),
                PlIll::Siin => (self.pl_strong.clone(), "siin".into()),
            },
            Case::Ade => (self.pl_weak.clone(), self.e("llA")),
            Case::Abl => (self.pl_weak.clone(), self.e("ltA")),
            Case::All => (self.pl_weak.clone(), "lle".into()),
            Case::Ess => (self.pl_strong.clone(), self.e("nA")),
            Case::Tra => (self.pl_weak.clone(), "ksi".into()),
            Case::Abe => (self.pl_weak.clone(), self.e("ttA")),
            Case::Com => (self.pl_strong.clone(), "ne".into()),
            Case::Ins => (self.pl_weak.clone(), "n".into()),
        };
        let mut out = text(&stem);
        out.push_str(&ending);
        out
    }

    /// A case form with a possessive suffix (`miekassani`, `miekkansa`, `miekoillaan`), or None
    /// for the instructive, which takes none.
    pub(super) fn possessive(&self, plural: bool, case: Case, poss: Poss) -> Option<String> {
        if case == Case::Ins {
            return None;
        }
        let plural = plural || case.plural_only();
        // The stem the suffix attaches to: the nominatives and the genitive singular use the
        // strong vowel stem (the suffix opens the syllable: `miekkani`, `kätensä`); endings in
        // `-n` drop it (`miekkaa-ni`, `miekkoje-nsa`, `miekkoihi-ni`); the translative's `-ksi`
        // becomes `-kse-`; the rest take the suffix as they are.
        let (base, keeps_n): (Vec<char>, bool) = match (plural, case) {
            (false, Case::Nom | Case::Gen) | (true, Case::Nom) => (self.strong.clone(), true),
            (_, Case::Gen | Case::Ill) => {
                let form: Vec<char> = self.form(plural, case).chars().collect();
                (cut(&form, 1), true)
            }
            (_, Case::Tra) => {
                let stem = if plural { &self.pl_weak } else { &self.weak };
                (with(stem.clone(), "kse"), false)
            }
            _ => (self.form(plural, case).chars().collect(), false),
        };
        let suffix = match poss {
            Poss::Ni => "ni".to_string(),
            Poss::Si => "si".to_string(),
            Poss::Mme => "mme".to_string(),
            Poss::Nne => "nne".to_string(),
            Poss::Nsa => {
                // `-nsA` after the nominatives, the `-n` cases, and a long vowel; else the
                // final vowel lengthened and `n` (`miekassaan`, `miekalleen`).
                match base.last() {
                    Some(&v) if !keeps_n && is_vowel(v) && !ends_in_long_vowel(&base) => {
                        format!("{v}n")
                    }
                    _ => self.e("nsA"),
                }
            }
        };
        let mut out = text(&base);
        out.push_str(&suffix);
        Some(out)
    }
}
