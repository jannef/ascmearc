//! The message ids, generated at build time from `assets/lang/en/messages/*.toml` (`build.rs`;
//! `docs/dev/localization.md`): `msg::doors::LOCKED` is the id `doors.locked`, and exists only
//! while `doors.toml` has `locked`. `ALL` lists every id with the roles its English template names
//! (`key`, the host's keys, left out).

include!(concat!(env!("OUT_DIR"), "/message_ids.rs"));

#[cfg(test)]
mod tests {
    /// The build script's scan finds what the runtime parser finds in the content: the same ids,
    /// the same roles.
    #[test]
    fn the_generated_ids_and_roles_are_the_english_templates() {
        let content = crate::test_support::builtin();
        let templates = content.english().templates();
        let mut parsed: Vec<(String, Vec<String>)> = Vec::new();
        for id in templates.ids() {
            let mut roles = templates.get(id).expect("listed").roles();
            roles.retain(|r| r != "key");
            parsed.push((id.to_string(), roles));
        }
        let mut generated: Vec<(String, Vec<String>)> = (super::ALL.iter())
            .map(|(id, roles)| {
                (
                    id.to_string(),
                    roles.iter().map(|r| r.to_string()).collect(),
                )
            })
            .collect();
        parsed.iter_mut().for_each(|(_, r)| r.sort());
        generated.iter_mut().for_each(|(_, r)| r.sort());
        parsed.sort();
        generated.sort();
        assert!(!generated.is_empty());
        assert_eq!(generated, parsed);
        assert_eq!(super::doors::LOCKED.as_str(), "doors.locked");
    }
}
