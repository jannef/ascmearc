//! Screen snapshot cases (`super`): equipment, the inventory, consuming, flasks, and what
//! items' descriptions say.

use super::*;

/// The showcase's storeroom, with the first stacks picked up (the arrows go into the quiver by
/// themselves) and armour on (equipment cases).
const ARMED: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
    pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup equip:head:g \
    equip:body:f equip:main_hand:c equip:missile_weapon:d equip:feet:j";
/// Enough stacks carried for a second page.
const HOARD: &str = "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup move:SW move:W*2 \
    move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N pickup move:S*2 \
    pickup move:NW move:W*2 move:N pickup move:SW pickup";

cases! {
    // Items.
    equipment_screen: Case {
        name: "equipment-screen",
        actions: ARMED,
        screen: Some("equipment"),
        ..BASE
    },
    equip_item_screen: Case {
        name: "equip-item-screen",
        actions: ARMED,
        screen: Some("equip-item"),
        ..BASE
    },
    kit_equipment: Case {
        name: "kit-equipment",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("equipment"),
        ..BASE
    },
    // An aristocrat's three extra ring slots (#1404), after the hands' rings.
    aristocrat_equipment: Case {
        name: "aristocrat-equipment",
        scenario: "world",
        character: Some("human,female,aristocrat"),
        run: true,
        screen: Some("equipment"),
        ..BASE
    },
    // Profane gifts' slots (#1628): Horned bars the head (its helm fell as it took), and
    // Many-fingered opens the first extra ring slot to a warrior.
    equipment_horned: Case {
        name: "equipment-horned",
        actions: concat!(
            "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N ",
            "pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup equip:head:g ",
            "equip:body:f gift:horned gift:many_fingered"
        ),
        screen: Some("equipment"),
        ..BASE
    },
    // The grown limbs (#1808): Three-armed's third hand holding a shield beside a staff,
    // and Prehensile-tailed's tail, empty.
    equipment_extra_arm: Case {
        name: "equipment-extra-arm",
        actions: concat!(
            "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N ",
            "pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup ",
            "equip:main_hand:c gift:extra_arm equip:third_hand:b"
        ),
        screen: Some("equipment"),
        ..BASE
    },
    equipment_prehensile_tail: Case {
        name: "equipment-prehensile-tail",
        actions: concat!(
            "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N ",
            "pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup ",
            "equip:main_hand:c gift:prehensile_tail"
        ),
        screen: Some("equipment"),
        ..BASE
    },
    kit_inventory: Case {
        name: "kit-inventory",
        scenario: "world",
        kit: true,
        run: true,
        screen: Some("inventory"),
        ..BASE
    },
    inventory_screen: Case {
        name: "inventory-screen",
        actions: "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup",
        screen: Some("inventory"),
        ..BASE
    },
    inventory_page_2: Case {
        name: "inventory-page-2",
        actions: HOARD,
        screen: Some("inventory"),
        keys: Some(">"),
        size: (80, 24),
        ..BASE
    },
    // The inventory's drop mode (#1196): `D` there, marks kept across pages; and as `d` in
    // play opens it.
    inventory_drop_mode: Case {
        name: "inventory-drop-mode",
        actions: HOARD,
        screen: Some("inventory"),
        keys: Some("D a > b"),
        size: (80, 24),
        ..BASE
    },
    drop_mode_from_play: Case {
        name: "drop-mode-from-play",
        actions: "move:SW*2 pickup move:NE*2 move:E*2 move:NE pickup",
        screen: Some("drop"),
        keys: Some("b"),
        ..BASE
    },
    burdened_inventory: Case {
        name: "burdened-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
            pickup move:S*2 pickup move:NW move:W*2 move:N pickup move:SW pickup",
        screen: Some("inventory"),
        ..BASE
    },
    magic_inventory: Case {
        name: "magic-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*5 pickup \
            move:E pickup equip:head:d",
        screen: Some("inventory"),
        ..BASE
    },
    gem_inventory: Case {
        name: "gem-inventory",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*7 move:W*4 move:S move:W*3 \
            pickup",
        screen: Some("inventory"),
        ..BASE
    },
    pick_up_screen: Case {
        name: "pick-up-screen",
        actions: "move:SW*2",
        screen: Some("pick-up"),
        ..BASE
    },
    eat_screen: Case {
        name: "eat-screen",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 kick:E*4 move:E*2 move:N \
            pickup move:E*2",
        screen: Some("eat"),
        ..BASE
    },
    // Born under the Barghest (seed 21): Hound's nose says how fresh the food smells.
    eat_screen_smell: Case {
        name: "eat-screen-smell",
        seed: Some(21),
        character: Some("human,male,warrior"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // A thief is trained in Appraise (#1031): the eat list says how fresh the food looks.
    eat_screen_judged: Case {
        name: "eat-screen-judged",
        seed: Some(6),
        character: Some("human,male,thief"),
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Nothing to consume (the arena's pack holds no food).
    consume_list_none: Case {
        name: "consume-list-none",
        scenario: "arena",
        run: true,
        screen: Some("eat"),
        ..BASE
    },
    // Potions: the Consume list with food (the bread carried) and potions
    // (the hub's pile underfoot) together, unknown ones by their word and colour.
    consume_list: Case {
        name: "consume-list",
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:NE move:E pickup move:SW",
        screen: Some("eat"),
        ..BASE
    },
    potion_description_unidentified: Case {
        name: "potion-description-unidentified",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup",
        screen: Some("describe"),
        ..BASE
    },
    potion_description: Case {
        name: "potion-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E pickup eat:a",
        screen: Some("describe"),
        ..BASE
    },
    // Flasks (#1223): the throw choice lists potions beside
    // throwing weapons, and a known flask says what its throw does.
    throw_list_flasks: Case {
        name: "throw-list-flasks",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E move:W*3 pickup identify",
        keys: Some("T"),
        ..BASE
    },
    potion_description_flask: Case {
        name: "potion-description-flask",
        actions: "drop move:N*3 open:N move:N*2 move:E*16 open:S move:S*3 open:S move:S*4 move:E*4 \
            open:N move:N*2 move:E move:W*3 pickup:c identify",
        screen: Some("describe"),
        ..BASE
    },
    item_description: Case {
        name: "item-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup \
            equip:head:a",
        screen: Some("describe"),
        ..BASE
    },
    ornament_description: Case {
        name: "ornament-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*4 pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    variant_description: Case {
        name: "variant-description",
        actions: "drop:a move:N*3 open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*3 move:N \
            pickup:b",
        screen: Some("describe"),
        ..BASE
    },
    // A unique item (the quarry's Stonebiter): unidentified, a rare dagger; identified, its
    // name, affixes, power, and lore.
    unique_description_unidentified: Case {
        name: "unique-description-unidentified",
        scenario: "quarry",
        actions: "drop move:W pickup",
        screen: Some("describe"),
        ..BASE
    },
    unique_description: Case {
        name: "unique-description",
        scenario: "quarry",
        actions: "drop move:W pickup identify",
        screen: Some("describe"),
        ..BASE
    },
    // The system affixes (#635), in words: the trap hall's staff (spells and mana), mail coat
    // (stamina, rage, eased casting), and amulet (hunger, disease, potions).
    item_description_spell_affix: Case {
        name: "item-description-spell-affix",
        scenario: "traps",
        actions: "drop:a move:E*5 pickup",
        screen: Some("describe"),
        ..BASE
    },
    item_description_stamina_affix: Case {
        name: "item-description-stamina-affix",
        scenario: "traps",
        actions: "drop:a move:E*6 pickup",
        screen: Some("describe"),
        ..BASE
    },
    item_description_utility_affix: Case {
        name: "item-description-utility-affix",
        scenario: "traps",
        actions: "drop:a move:E*7 pickup",
        screen: Some("describe"),
        ..BASE
    },
}

/// A cell holding a cursed iron ring of Hunger (#1754), unidentified.
const CURSED_CELL: &str = r##"
name = "Cursed cell"
ambient = [255, 255, 255, 0]
map = '''
#####
#@..#
#####
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }

[[item]]
at = [1, 1]
object = "iron_ring"
affixes = ["of_hunger"]
"##;

/// #1754: a cursed ring put on shows its curse but not its affix, in its name and description;
/// identified, it names the affix and says what it hinders.
#[test]
fn a_cursed_ring_as_text() {
    let mut files = ascmearc_assets::files();
    files.insert("levels/cursed_cell.toml", CURSED_CELL);
    files.insert(
        "scenarios/cursed_cell.toml",
        "name = \"cursed cell\"\nstart_level = \"cursed_cell\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let cases = [
        ("cursed-ring-description", "pickup equip:left_ring:a"),
        (
            "cursed-ring-identified",
            "pickup equip:left_ring:a identify",
        ),
    ];
    for (name, actions) in cases {
        let case = Case {
            name,
            scenario: "cursed_cell",
            actions,
            screen: Some("describe"),
            ..BASE
        };
        check_with(&case, content.clone());
    }
}
