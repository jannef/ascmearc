//! Drȳtorr's tower folk (#1823; docs/dev/towers.md "The tower folk"): the trap floors draw only
//! the tower's roster (and a chest's mimic), 8 to 14 a floor, the wizard's creations among them; and none of the
//! new folk strays into another branch or level (`Content::fill_generic` leaves a tower's
//! population out, and they are `stray = false`).

use crate::mapgen::generate;
use crate::test_support::CONTENT;
use std::collections::BTreeSet;

/// The tower's own folk, made for it (assets/campaigns/ascmearc/data/creatures/).
const FOLK: [&str; 4] = ["gargoyle", "loam_man", "mannikin", "ward_stone"];

#[test]
fn the_tower_folk_keep_the_floors() {
    let def = CONTENT.dungeon("drytorr_tower").expect("Drȳtorr");
    let roster = &def.spawning.as_ref().expect("a population").population;
    let mut met = BTreeSet::new();
    for seed in 1..=12 {
        for depth in 1..=2 {
            let id = def.level_id(depth);
            let level = generate(&CONTENT, ("ascmearc", seed), &id, None).expect("generates");
            let at = format!("seed {seed} {id}");
            // A chest may be a mimic, as on any floor: not one of the tower's folk.
            let folk: Vec<_> = (level.creatures.iter())
                .filter(|c| CONTENT.creature(c.creature).mimic.is_none())
                .collect();
            assert!((8..=14).contains(&folk.len()), "{at}: {}", folk.len());
            for c in folk {
                let name = &CONTENT.creature(c.creature).id;
                assert!(roster.contains(&c.creature), "{at}: {name}");
                met.insert(name.as_str());
            }
        }
    }
    for id in FOLK {
        assert!(met.contains(id), "{id} never met: {met:?}");
    }
}

#[test]
fn the_tower_folk_stray_nowhere_else() {
    let folk: Vec<_> = (FOLK.iter())
        .map(|id| CONTENT.creature_id(id).expect(id))
        .collect();
    for &c in &folk {
        assert!(!CONTENT.may_stray(c), "{}", CONTENT.creature(c).id);
    }
    let lists = (CONTENT.dungeons())
        .filter_map(|d| Some((d.id.as_str(), d.spawning.as_ref()?)))
        .chain((CONTENT.levels()).filter_map(|l| Some((l.id.as_str(), &l.spawning.as_ref()?.1))));
    for (id, spawning) in lists {
        let theirs = id == "drytorr_tower";
        for &c in &folk {
            assert!(!spawning.generic.contains(&c), "{id} strays");
            assert_eq!(spawning.population.contains(&c), theirs, "{id}");
        }
    }
}
