//! Screen snapshot cases (`super`): talk, quests, and messages.

use super::*;

/// Next to the hub's hermit.
macro_rules! hermit {
    () => {
        "move:N*3 open:N move:N*2 move:E*67 open:S move:S*4 move:SE*3"
    };
}

cases! {
    // Talk, quests, messages.
    dialogue_screen: Case {
        name: "dialogue-screen",
        actions: concat!(hermit!(), " chat:SE ask:SE:rooms"),
        screen: Some("dialogue"),
        ..BASE
    },
    // The test quarrel's parting (#1785): both rivals refused and killed, the quest handed in to
    // the scholar, and the one dialog asks Leave or Betray; Esc does not close it.
    quarrel_parting: Case {
        name: "quarrel-parting",
        scenario: "quarrel",
        actions: "ask:E:quest-quarrel move:N move:W*2 move:N ascend move:N ask:N:refuse-quarrel \
                  move:N*8 move:S*3 descend move:E*6 ascend move:N ask:N:refuse-quarrel \
                  move:N*8 move:S*3 descend move:SW move:W ask:SW:quest-quarrel levelup:auto",
        keys: Some("Esc"),
        ..BASE
    },
    // A rival's offer on the first arrival at her top (#1785): the dialogue screen opens on it,
    // with the two topics.
    quarrel_offer: Case {
        name: "quarrel-offer",
        scenario: "quarrel",
        actions: "ask:E:quest-quarrel move:N move:W*2 move:N ascend",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Sister Hild: a proper name, topics, and two services with their prices (#336).
    dialogue_services: Case {
        name: "dialogue-services",
        scenario: "ceaptun",
        actions: "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Factor Wulfrun: an appraisal, priced per thing (#432).
    dialogue_appraisal: Case {
        name: "dialogue-appraisal",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 open:S move:S*4 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's Wat One-Eye (#425): the guild's chain, one quest topic at a time, before his
    // own two.
    dialogue_guild: Case {
        name: "dialogue-guild",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 move:E*9 open:S move:S*5 chat:S",
        screen: Some("dialogue"),
        ..BASE
    },
    // Cēaptūn's smith, Eadric, the smithy's keeper (#1394): his topics, then Metalcraft taught
    // for 2000 gold.
    dialogue_smith: Case {
        name: "dialogue-smith",
        scenario: "ceaptun",
        actions: "move:NW*7 move:N chat:N",
        screen: Some("dialogue"),
        ..BASE
    },
    // An aristocrat's squire at the Gæsthūs (#1405): the stipend of a few levels handed over,
    // then a flattering one-liner, in the message history.
    servant_stipend: Case {
        name: "servant-stipend",
        scenario: "ceaptun",
        character: Some("human,male,aristocrat"),
        actions: "xp:3334 levelup:auto move:SW move:S*3 open:S move:S*2 move:SW*4 chat:W chat:W",
        screen: Some("messages"),
        ..BASE
    },
    // A merchant's business partner in Wulfrun's office (#1407): 100 gold put in, a month gone,
    // and the dialogue screen on what the stake is worth and the deals.
    partner_invest: Case {
        name: "partner-invest",
        scenario: "ceaptun",
        character: Some("human,male,merchant"),
        actions: "move:SE*4 move:E*22 open:S move:S*2 move:E*2 chat:E ask:E:invest_100 \
                  skip:24h*30 chat:E",
        screen: Some("dialogue"),
        ..BASE
    },
    // A craftsperson's patron at the Gæsthūs (#1421): a commission given with its stock, then
    // the reminder while it waits, in the message history.
    patron_commission: Case {
        name: "patron-commission",
        scenario: "ceaptun",
        character: Some("human,male,craftsperson"),
        talents: Some("smith,wright,healthy,lean,wortcunner"),
        actions: "xp:40000 levelup:auto move:SW move:S*3 open:S move:S*2 move:E*4 chat:E chat:E",
        screen: Some("messages"),
        ..BASE
    },
    quest_offer: Case {
        name: "quest-offer",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("dialogue"),
        ..BASE
    },
    // A rolled quest (#422): the courtyard's gardener, whose offer names what was drawn as it
    // was given.
    quest_offer_rolled: Case {
        name: "quest-offer-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("dialogue"),
        ..BASE
    },
    quest_log: Case {
        name: "quest-log",
        actions: concat!(hermit!(), " ask:SE:quest-rats"),
        screen: Some("quests"),
        ..BASE
    },
    // A rolled quest's first breadcrumb, filled.
    quest_log_rolled: Case {
        name: "quest-log-rolled",
        scenario: "courtyard",
        actions: "move:E*9 move:NE*6 wait*12 move:E ask:N:quest-slug_charm",
        screen: Some("quests"),
        ..BASE
    },
    message_history: Case {
        name: "message-history",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        screen: Some("messages"),
        ..BASE
    },
    more_prompt: Case {
        name: "more-prompt",
        actions: concat!(hermit!(), " chat:SE ask:SE:quest-rats"),
        ..BASE
    },
}

/// Factor Wulfrun asked about the lost shaft before level 5: the topic is listed, her reply
/// says the player is not ready yet, and nothing is given or marked. The content lifts the
/// quest's prerequisite (the barrow's heart, deep in Cealdhlāw) so a short walk reaches her.
#[test]
fn quest_refused_as_too_early() {
    let mut files = ascmearc_assets::files();
    let path = "campaigns/ascmearc/data/quests/wulfrun.toml";
    let quests = include_str!("../../../../../assets/campaigns/ascmearc/data/quests/wulfrun.toml");
    let lifted = quests.replacen("requires = \"barrow_heart\"\n", "", 1);
    assert_ne!(lifted, quests, "the lost shaft requires the barrow's heart");
    files.insert(path, &lifted);
    let content = Arc::new(Content::load(&files).expect("content"));
    let case = Case {
        name: "quest-refused-early",
        scenario: "ceaptun",
        actions: "move:E*26 move:S*4 open:S move:S*4 ask:S:quest-lost_shaft",
        screen: Some("dialogue"),
        ..BASE
    };
    check_with(&case, content);
}

/// The hermit's oath (#1174): a calling's topic, offered to a bard whose class is full (the dev
/// experience and level-ups bring the showcase's bard to 30). Asking it is the core's
/// (`world/campaigns/strange_spirit_tests.rs`): the level-up it brings would cover this screen.
#[test]
fn the_oath_topic() {
    let content = Arc::new(ascmearc_assets::content().expect("content"));
    let case = Case {
        name: "dialogue-calling-topic",
        character: Some("human,male,bard"),
        actions: concat!(hermit!(), " xp:363777 levelup:auto chat:SE"),
        screen: Some("dialogue"),
        ..BASE
    };
    check_with(&case, content);
}

/// The test quarrel's parting (#1785) waits for the thanks: asked on the dialogue screen, the
/// hand-in reopens the screen on the scholar's words, and the one dialog asks the parting only
/// once every screen is closed; then it must be answered.
#[test]
fn the_parting_opens_once_the_thanks_are_read() {
    let content = Arc::new(ascmearc_assets::content().expect("content"));
    let case = Case {
        scenario: "quarrel",
        // A level bought first, so the hand-in brings none to choose over the dialog.
        actions: "xp:1500 levelup:auto ask:E:quest-quarrel move:N move:W*2 move:N ascend move:N \
                  ask:N:refuse-quarrel move:N*8 move:S*3 descend move:E*6 ascend move:N \
                  ask:N:refuse-quarrel move:N*8 move:S*3 descend move:SW move:W chat:SW",
        screen: Some("dialogue"),
        ..BASE
    };
    let mut app = start(&case, content);
    press_keys(&mut app, "1");
    let parting = |app: &App| {
        let game = app.world().resource::<Game>();
        game.world.perception().parting().is_some()
    };
    assert!(parting(&app), "the hand-in leaves the parting pending");
    assert!(!app.world().resource::<ScreenStack>().asks_required());
    assert!(
        !app.world().resource::<ScreenStack>().is_empty(),
        "the thanks first"
    );
    // Closing the thanks opens it.
    press_keys(&mut app, "Esc");
    assert!(app.world().resource::<ScreenStack>().asks_required());
    press_keys(&mut app, "Esc");
    assert!(
        app.world().resource::<ScreenStack>().asks_required(),
        "it must be answered"
    );
    press_keys(&mut app, "Enter");
    assert!(!parting(&app), "Leave answers it");
    assert!(!app.world().resource::<ScreenStack>().asks_required());
}
