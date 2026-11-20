//! The plural dictionary (`assets/data/plurals.toml`, and a campaign's `data/plurals.toml`):
//! [`Plurals`] lives in `ascmearc_lang::en` (`docs/dev/localization.md`); `Content::load_plurals`
//! reads the files into it, and [`super::Content::plural`] is the one place a plural is made when
//! content gives none.

pub use ascmearc_lang::en::Plurals;

#[cfg(test)]
mod tests {
    #[test]
    fn the_builtin_dictionary() {
        let content = super::super::Content::builtin().expect("content");
        assert_eq!(content.plural("undead"), "undead");
        assert_eq!(content.plural("vermin"), "vermin");
        assert_eq!(content.plural("dwarf"), "dwarves");
        assert_eq!(content.plural("wolf"), "wolves");
        assert_eq!(content.plural("goblin"), "goblins");
    }
}
