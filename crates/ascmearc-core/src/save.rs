//! Save format: an envelope around [`WorldState`], as gzip-compressed JSON.
//!
//! This module only turns a world into bytes and back; where the bytes live is the app's concern.
//! A save is pinned to the game version that wrote it ([`crate::GAME_VERSION`], the workspace
//! version): a save from another version, a build with different content, or a damaged file is
//! rejected with a clear error and never misread. Nothing carries a save across versions.

use crate::content::{Content, ModStamp, describe_mods};
use crate::world::{World, WorldState};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Arc;

/// Marker identifying a save of the game inside the envelope. It keeps the game's old name
/// (Warlock, until 0.18; #1680), so a save from before the rename still reads as one made by
/// another version rather than as no save at all.
const MAGIC: &str = "WARLOCK-SAVE";
/// Upper bound on the decompressed size, so a hostile file cannot exhaust memory.
const MAX_JSON_BYTES: u64 = 256 * 1024 * 1024;

/// Why a save could not be read. Messages are meant for the player.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SaveError {
    #[error("this is not an Ascmearc save file")]
    NotASave,
    /// Made by another version of the game (`found`; empty when the save does not say).
    #[error("this save was made by {}, and only that version can continue it", version_words(.found))]
    OtherVersion { found: String },
    /// Made with another set of mods (#1731): `found` and `active` in words
    /// ([`describe_mods`]).
    #[error(
        "this save was made {found}, and the game now runs {active}: turn on the same mods, in the same order, to continue it"
    )]
    OtherMods { found: String, active: String },
    #[error("this save was made with different game content and cannot be loaded")]
    ContentMismatch,
    #[error("the save file is damaged: {0}")]
    Corrupt(String),
    /// The state could not be turned into bytes (#1580): nothing is written, so the save
    /// already there is kept.
    #[error("the game could not be written to a save: {0}")]
    Unwritable(String),
}

/// `version 0.8.0`, or for a save that does not say, `an older version`.
fn version_words(found: &str) -> String {
    if found.is_empty() {
        "an older version".into()
    } else {
        format!("version {found}")
    }
}

#[derive(Serialize)]
struct EnvelopeOut<'a> {
    magic: &'static str,
    game_version: &'a str,
    content_fingerprint: u64,
    /// The content mods the run was played with, in order (#1731); left out without any. Text
    /// mods are never listed (#1892): a save loads with or without them.
    #[serde(skip_serializing_if = "<[ModStamp]>::is_empty")]
    mods: &'a [ModStamp],
    state: &'a WorldState,
}

/// The envelope with the payload still undecoded, so the header is checked before the state.
#[derive(Deserialize)]
struct EnvelopeIn {
    magic: String,
    #[serde(default)]
    game_version: String,
    #[serde(default)]
    content_fingerprint: u64,
    #[serde(default)]
    mods: Vec<ModStamp>,
    state: serde_json::Value,
}

/// Serializes the world's state into save bytes.
pub fn try_encode(world: &World) -> Result<Vec<u8>, SaveError> {
    try_encode_state(
        world.state(),
        world.fingerprint(),
        world.content().content_mods(),
    )
}

/// Serializes a state (for example a clone taken earlier) with the content fingerprint and the
/// mods of the content it was played with. An error (a map keyed by a struct, say: JSON keys are strings)
/// is returned, never an empty save that would be written over a good one (#1580).
pub fn try_encode_state(
    state: &WorldState,
    content_fingerprint: u64,
    mods: &[ModStamp],
) -> Result<Vec<u8>, SaveError> {
    compress(&EnvelopeOut {
        magic: MAGIC,
        game_version: crate::GAME_VERSION,
        content_fingerprint,
        mods,
        state,
    })
}

/// [`try_encode`] for tests, which may panic.
#[cfg(test)]
pub fn encode(world: &World) -> Vec<u8> {
    try_encode(world).expect("the state serializes")
}

/// [`try_encode_state`] for tests, which may panic.
#[cfg(test)]
pub fn encode_state(state: &WorldState, content_fingerprint: u64) -> Vec<u8> {
    try_encode_state(state, content_fingerprint, &[]).expect("the state serializes")
}

/// `value` as gzip-compressed JSON.
fn compress(value: &impl Serialize) -> Result<Vec<u8>, SaveError> {
    let unwritable = |e: &dyn std::fmt::Display| SaveError::Unwritable(e.to_string());
    let json = serde_json::to_vec(value).map_err(|e| unwritable(&e))?;
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&json).map_err(|e| unwritable(&e))?;
    gz.finish().map_err(|e| unwritable(&e))
}

/// The campaign of a saved state: its own, or for a state without one its scenario's.
fn campaign_of(state: &serde_json::Value, content: &Content) -> String {
    let field = |name: &str| state.get(name).and_then(serde_json::Value::as_str);
    match field("campaign").filter(|c| !c.is_empty()) {
        Some(campaign) => campaign.to_string(),
        None => (field("scenario").and_then(|s| content.scenario(s)))
            .and_then(|s| s.campaign.clone())
            .unwrap_or_default(),
    }
}

/// Restores a world from save bytes, validating it against `content`.
pub fn decode(bytes: &[u8], content: Arc<Content>) -> Result<World, SaveError> {
    if !bytes.starts_with(&[0x1f, 0x8b]) {
        return Err(SaveError::NotASave);
    }
    let mut json = Vec::new();
    GzDecoder::new(bytes)
        .take(MAX_JSON_BYTES)
        .read_to_end(&mut json)
        .map_err(|e| SaveError::Corrupt(e.to_string()))?;
    let envelope: EnvelopeIn = serde_json::from_slice(&json).map_err(|e| {
        if e.is_syntax() || e.is_eof() {
            SaveError::Corrupt(e.to_string())
        } else {
            SaveError::NotASave
        }
    })?;
    if envelope.magic != MAGIC {
        return Err(SaveError::NotASave);
    }
    if envelope.game_version != crate::GAME_VERSION {
        return Err(SaveError::OtherVersion {
            found: envelope.game_version,
        });
    }
    if envelope.mods != content.content_mods() {
        return Err(SaveError::OtherMods {
            found: describe_mods(&envelope.mods),
            active: describe_mods(content.content_mods()),
        });
    }
    if envelope.content_fingerprint
        != content.fingerprint_for(&campaign_of(&envelope.state, &content))
    {
        return Err(SaveError::ContentMismatch);
    }
    let state: WorldState =
        serde_json::from_value(envelope.state).map_err(|e| SaveError::Corrupt(e.to_string()))?;
    World::from_state(content, state).map_err(|e| SaveError::Corrupt(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Dir, GAME_VERSION};

    fn content() -> Arc<Content> {
        crate::test_support::builtin()
    }

    /// A value serde_json cannot write (a map keyed by a tuple) is an error, never an empty
    /// save written over a good one (#1580).
    #[test]
    fn a_state_that_cannot_be_serialized_is_an_error() {
        let map = std::collections::BTreeMap::from([((1u8, 2u8), 3u8)]);
        let err = compress(&map).expect_err("JSON keys are strings");
        assert!(matches!(err, SaveError::Unwritable(_)), "{err:?}");
    }

    fn world() -> World {
        let mut w = World::new(content(), "showcase", None).expect("world");
        for _ in 0..5 {
            w.apply(&Action::Move(Dir::E));
        }
        w
    }

    fn gz(json: &str) -> Vec<u8> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(json.as_bytes()).expect("write");
        e.finish().expect("finish")
    }

    #[test]
    fn round_trip_preserves_state() {
        let w = world();
        let restored = decode(&encode(&w), content()).expect("decode");
        assert_eq!(restored.state(), w.state());
    }

    #[test]
    fn encode_state_matches_encode() {
        let w = world();
        let parked = (w.state().clone(), w.fingerprint());
        assert_eq!(encode_state(&parked.0, parked.1), encode(&w));
    }

    #[test]
    fn restored_world_continues_identically() {
        let mut a = world();
        let mut b = decode(&encode(&a), content()).expect("decode");
        for _ in 0..20 {
            a.apply(&Action::Wait);
            b.apply(&Action::Wait);
        }
        assert_eq!(a.state(), b.state());
    }

    #[test]
    fn a_save_records_the_version_that_wrote_it() {
        let mut json = Vec::new();
        let bytes = encode(&world());
        GzDecoder::new(&bytes[..])
            .read_to_end(&mut json)
            .expect("gunzip");
        let envelope: serde_json::Value = serde_json::from_slice(&json).expect("json");
        assert_eq!(envelope["game_version"], GAME_VERSION);
    }

    /// A save of another version is refused before its state is read, however well it would
    /// read; one that names no version (older than the pin) too.
    #[test]
    fn rejects_another_version() {
        let w = world();
        let mut state = serde_json::to_value(w.state()).expect("state");
        let other = |version: Option<&str>, state: &mut serde_json::Value| {
            let mut envelope = serde_json::json!({
                "magic": MAGIC,
                "content_fingerprint": w.fingerprint(),
            });
            if let Some(version) = version {
                envelope["game_version"] = version.into();
            }
            envelope["state"] = state.take();
            decode(&gz(&envelope.to_string()), content()).err()
        };
        assert_eq!(
            other(Some("0.1.0"), &mut state.clone()),
            Some(SaveError::OtherVersion {
                found: "0.1.0".into()
            })
        );
        let unnamed = other(None, &mut state);
        assert_eq!(
            unnamed,
            Some(SaveError::OtherVersion {
                found: String::new()
            })
        );
        let text = unnamed.map(|e| e.to_string()).unwrap_or_default();
        assert!(text.contains("an older version"), "{text}");
    }

    #[test]
    fn rejects_content_mismatch() {
        let json = format!(
            r#"{{"magic":"WARLOCK-SAVE","game_version":"{GAME_VERSION}","content_fingerprint":1,"state":null}}"#
        );
        assert_eq!(
            decode(&gz(&json), content()).err(),
            Some(SaveError::ContentMismatch)
        );
    }

    /// The content with the repository's mods `ids` (`mods/<id>/`) laid over, in order; a mod
    /// given as `(id, files)` lays those files.
    fn with_mods(mods: &[(&str, Vec<(String, String)>)]) -> Arc<Content> {
        let mut files = crate::content::ContentFiles::builtin();
        for (id, mod_files) in mods {
            let stamp = ModStamp {
                id: (*id).into(),
                version: "1.0.0".into(),
            };
            files.overlay(
                stamp,
                mod_files.iter().map(|(p, t)| (p.as_str(), t.as_str())),
            );
        }
        Arc::new(Content::load(&files).expect("the mods load"))
    }

    /// The content with the repository's example mod, `mods/pride_month/`, laid over.
    fn with_pride_month() -> Arc<Content> {
        with_mods(&[("pride_month", crate::content::pride_month())])
    }

    fn creaky_doors() -> (&'static str, Vec<(String, String)>) {
        ("creaky_doors", crate::content::repo_mod("creaky_doors"))
    }

    /// A text mod (#1892) is never recorded: a save made without the example text mod loads
    /// with it, one made with it loads without it, and its log reads in either.
    #[test]
    fn a_text_mod_never_refuses_a_save() {
        let plain = encode(&world());
        assert!(decode(&plain, with_mods(&[creaky_doors()])).is_ok());
        let text = with_mods(&[creaky_doors()]);
        let mut w = World::new(text.clone(), "showcase", None).expect("world");
        w.apply(&Action::Wait);
        let bytes = encode(&w);
        let restored = decode(&bytes, content()).expect("loads without the text mod");
        assert_eq!(restored.state(), w.state());
        assert_eq!(restored.log(), w.log());
        // With a content mod beside it, only the content mod counts.
        let both = with_mods(&[
            ("pride_month", crate::content::pride_month()),
            creaky_doors(),
        ]);
        let mut w = World::new(both, "showcase", None).expect("world");
        w.apply(&Action::Wait);
        let bytes = encode(&w);
        assert!(decode(&bytes, with_pride_month()).is_ok());
        assert!(matches!(
            decode(&bytes, text),
            Err(SaveError::OtherMods { .. })
        ));
    }

    /// A mod with content and text pins the save by its content: refused without it, whatever
    /// text mods are on.
    #[test]
    fn a_mod_with_content_and_text_pins_the_save() {
        let mut files = crate::content::pride_month();
        files.push((
            "lang/en/messages/doors.toml".into(),
            "open = \"You open the door, proudly.\"\n".into(),
        ));
        let both = with_mods(&[("pride_month", files)]);
        assert_eq!(both.content_mods().len(), 1);
        let mut w = World::new(both.clone(), "showcase", None).expect("world");
        w.apply(&Action::Wait);
        let bytes = encode(&w);
        assert!(matches!(
            decode(&bytes, content()),
            Err(SaveError::OtherMods { .. })
        ));
        assert!(matches!(
            decode(&bytes, with_mods(&[creaky_doors()])),
            Err(SaveError::OtherMods { .. })
        ));
        assert!(matches!(
            decode(&encode(&world()), both.clone()),
            Err(SaveError::OtherMods { .. })
        ));
        assert!(decode(&bytes, both).is_ok());
    }

    /// A save records its mods (#1731): one made without the mod is refused with it, one made
    /// with it is refused without it, each saying why; with the same mods it loads.
    #[test]
    fn rejects_another_set_of_mods() {
        let plain = encode(&world());
        let err = decode(&plain, with_pride_month()).expect_err("refused");
        assert_eq!(
            err.to_string(),
            "this save was made without mods, and the game now runs with the mod pride_month \
             1.0.0: turn on the same mods, in the same order, to continue it"
        );
        let mut modded = World::new(with_pride_month(), "showcase", None).expect("world");
        modded.apply(&Action::Wait);
        let bytes = encode(&modded);
        assert!(matches!(
            decode(&bytes, content()),
            Err(SaveError::OtherMods { .. })
        ));
        let restored = decode(&bytes, with_pride_month()).expect("the same mods");
        assert_eq!(restored.state(), modded.state());
    }

    #[test]
    fn rejects_foreign_data() {
        assert_eq!(decode(b"hello", content()).err(), Some(SaveError::NotASave));
        let other = gz(r#"{"magic":"OTHER","game_version":"0.1.0","state":1}"#);
        assert_eq!(decode(&other, content()).err(), Some(SaveError::NotASave));
    }

    #[test]
    fn rejects_corrupt_and_truncated_data() {
        let bytes = encode(&world());
        let truncated = &bytes[..bytes.len() / 2];
        assert!(matches!(
            decode(truncated, content()),
            Err(SaveError::Corrupt(_))
        ));
        assert!(matches!(
            decode(&gz("{ not json"), content()),
            Err(SaveError::Corrupt(_))
        ));
        let bad_state = format!(
            r#"{{"magic":"WARLOCK-SAVE","game_version":"{GAME_VERSION}","content_fingerprint":{},"state":{{"seed":"x"}}}}"#,
            content().fingerprint_for("")
        );
        assert!(matches!(
            decode(&gz(&bad_state), content()),
            Err(SaveError::Corrupt(_))
        ));
    }

    /// The first JSON object (depth first) that has `key`.
    fn find_with<'a>(v: &'a mut serde_json::Value, key: &str) -> Option<&'a mut serde_json::Value> {
        if v.get(key).is_some() {
            return Some(v);
        }
        match v {
            serde_json::Value::Object(m) => m.values_mut().find_map(|c| find_with(c, key)),
            serde_json::Value::Array(a) => a.iter_mut().find_map(|c| find_with(c, key)),
            _ => None,
        }
    }

    /// `w`'s save with its state changed by `change`.
    fn tampered(w: &World, change: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
        let mut state = serde_json::to_value(w.state()).expect("state");
        change(&mut state);
        let envelope = serde_json::json!({
            "magic": MAGIC,
            "game_version": GAME_VERSION,
            "content_fingerprint": w.fingerprint(),
            "state": state,
        });
        gz(&envelope.to_string())
    }

    /// A grid whose cells do not fill it, or a memory not the terrain's size, is a damaged
    /// save, never a panic on Continue (#1585).
    #[test]
    fn rejects_grids_of_the_wrong_size() {
        let w = world();
        let short = tampered(&w, |state| {
            let grid = find_with(state, "cells").expect("a grid");
            grid["cells"].as_array_mut().expect("cells").pop();
        });
        assert!(matches!(
            decode(&short, content()),
            Err(SaveError::Corrupt(_))
        ));
        let small_memory = tampered(&w, |state| {
            let level = find_with(state, "memory").expect("a level");
            level["memory"] = serde_json::json!({"width": 1, "height": 1, "cells": [null]});
        });
        assert!(matches!(
            decode(&small_memory, content()),
            Err(SaveError::Corrupt(_))
        ));
    }
}
