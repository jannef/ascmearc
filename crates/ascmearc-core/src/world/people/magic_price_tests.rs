//! Magic raises an item's shop price (#1709, `data/rules/magic_worth.toml`):
//! the worth by rarity and identified affixes, a price that tells nothing unidentified, the bill
//! keeping an item as it was taken, and selling.

use crate::action::{letter, parse_actions};
use crate::content::{Content, Rarity, Shops};
use crate::geom::Pos;
use crate::magic::{Affix, Magic};
use crate::object::Stack;
use crate::test_support::builtin as content;
use crate::world::*;

/// From the courtyard's start into Grimbold's General Store (as `shop_tests.rs` walks it).
const INTO_SHOP: &str = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                         move:S*5 move:E*5";
/// The dagger on the shop floor.
const DAGGER: Pos = Pos::new(56, 13);

/// A magic (or rare) dagger with Sharp's first tier (item level 1), identified or not.
fn magic_dagger(content: &Content, rarity: Rarity, identified: bool) -> Stack {
    let magic = Magic {
        rarity,
        level: 1,
        affixes: vec![Affix {
            id: "sharp".into(),
            tier: 1,
            value: 1,
        }],
        name: (rarity == Rarity::Rare).then(|| "Grim Bite".into()),
        identified,
        carried: 0,
        unique: None,
    };
    Stack::with_magic(content.object_id("dagger").expect("dagger"), 1, magic)
}

fn apply(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn shops(w: &World) -> &Shops {
    w.content().shops().expect("shops")
}

fn shop(w: &World) -> &Shop {
    &w.state.current().shops[0]
}

fn price(w: &World, stack: &Stack) -> u32 {
    let f = favour(w.content(), shops(w), w.state.player());
    shops(w).price(w.content(), shop(w).kind, stack, f)
}

#[test]
fn magic_adds_its_rarity_and_known_affixes_to_the_worth() {
    let c = content();
    let plain = Stack::one(c.object_id("dagger").expect("dagger"));
    assert_eq!(plain.worth(&c), 30, "a plain item is worth its value");
    // Unidentified, only the rarity is known: half again, twice.
    assert_eq!(magic_dagger(&c, Rarity::Magic, false).worth(&c), 45);
    assert_eq!(magic_dagger(&c, Rarity::Rare, false).worth(&c), 60);
    // Identified, each affix adds 50 + 25 × its tier's item level (1).
    assert_eq!(magic_dagger(&c, Rarity::Magic, true).worth(&c), 45 + 75);
    assert_eq!(magic_dagger(&c, Rarity::Rare, true).worth(&c), 60 + 75);
    // A stack counts each one.
    let mut two = magic_dagger(&c, Rarity::Magic, true);
    two.count = 2;
    assert_eq!(two.worth(&c), 240);
    // Value stays the make alone.
    assert_eq!(two.value(&c), 60);
}

#[test]
fn without_the_rules_file_magic_adds_nothing() {
    let mut files = crate::content::ContentFiles::builtin();
    files.remove("data/rules/magic_worth.toml");
    let c = Content::load(&files).expect("content");
    assert_eq!(magic_dagger(&c, Rarity::Rare, true).worth(&c), 30);
}

#[test]
fn the_bill_keeps_a_magic_item_as_it_was_taken() {
    let mut w = World::new(content(), "courtyard", None).expect("courtyard");
    apply(&mut w, INTO_SHOP);
    let unknown = magic_dagger(w.content(), Rarity::Magic, false);
    let known = magic_dagger(w.content(), Rarity::Magic, true);
    *w.state
        .current_mut()
        .objects
        .get_mut(&DAGGER)
        .expect("the dagger") = vec![unknown.clone()];
    // The tag tells the rarity, which shows anyway, and never the affixes.
    let asked = price(&w, &unknown);
    let plain = price(&w, &Stack::one(unknown.object));
    assert!(asked > plain, "{asked} over {plain}");
    assert_eq!(w.perception().price_tag(DAGGER, &unknown), Some(asked));
    assert!(price(&w, &known) > asked, "identified, its affixes count");

    apply(&mut w, "move:E move:NE pickup");
    assert_eq!(
        shop(&w).unpaid,
        std::slice::from_ref(&unknown),
        "the bill keeps its magic"
    );
    let (width, height) = (w.state.current().width(), w.state.current().height());
    assert!(shop(&w).is_valid_for(w.content(), width, height));
    let item = w.state.player().inventory.len() - 1;
    // Identified since, it is still the unpaid dagger, owed at the price it was taken at.
    assert!(w.player_mut().inventory[item].identify());
    let view = w.perception();
    assert_eq!(view.unpaid()[item], Some(asked));
    assert_eq!(view.shop_here().map(|s| s.owed()), Some(asked));
    // Put back, the bill is clear.
    apply(&mut w, &format!("drop:{}", letter(item).expect("a letter")));
    assert!(shop(&w).unpaid.is_empty());
}

#[test]
fn an_identified_magic_item_sells_for_more() {
    let mut w = World::new(content(), "courtyard", None).expect("courtyard");
    apply(&mut w, INTO_SHOP);
    let f = favour(w.content(), shops(&w), w.state.player());
    let offer = |w: &World, s: &Stack| shops(w).offer(w.content(), shop(w).kind, s, f);
    let plain = offer(
        &w,
        &Stack::one(w.content().object_id("dagger").expect("dagger")),
    );
    let unknown = offer(&w, &magic_dagger(w.content(), Rarity::Rare, false));
    let known = offer(&w, &magic_dagger(w.content(), Rarity::Rare, true));
    assert!(
        plain < unknown && unknown < known,
        "{plain} {unknown} {known}"
    );
    let item = w.state.player().inventory.len();
    let rare = magic_dagger(w.content(), Rarity::Rare, true);
    w.player_mut().inventory.push(rare);
    apply(&mut w, &format!("sell:{}", letter(item).expect("a letter")));
    let sold = crate::test_support::last_text(&w);
    assert!(sold.contains(&format!("for {known} gold")), "{sold}");
}
