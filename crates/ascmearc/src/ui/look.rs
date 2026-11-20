//! Look mode: a cursor the player moves over the map to read what is there. It is a
//! free action — no game time passes — and it only describes what the player perceives.
//!
//! The line is said from `assets/lang/en/ui/look.toml`: a sentence entry per shape of the cell,
//! the clauses it holds nested in it as messages (`docs/dev/localization.md` "Nested messages"),
//! an empty slot for a clause that does not apply. Creatures, things, and the ground are
//! phrases, so the language gives their articles and numbers.

use super::text::{UiText, look as T, ui_with};
use crate::render::console::{Cell, Console};
use ascmearc_core::Pos;
use ascmearc_core::color::Rgb;
use ascmearc_core::content::{
    ActionKind, Content, CreatureId, MaterialId, Noun, ServiceKind, Stance, TerrainId, TrapId,
    ZoneKind,
};
use ascmearc_core::damage::ResistanceWord;
use ascmearc_core::lang::{Arg, MessageId, Modifier, Msg, Person, Phrase, Word};
use ascmearc_core::magic::Numbers;
use ascmearc_core::object::Stack;
use ascmearc_core::perception::{BelowView, CellView, PerceptionView, SeenActor};
use ascmearc_core::world::{ActorId, ActorKind};

/// Cursor colors: dark glyph on a warm highlight, so it reads on any terrain and under any light.
pub const CURSOR_FG: Rgb = Rgb::new(0x14, 0x12, 0x10);
pub const CURSOR_BG: Rgb = Rgb::new(0xe0, 0xc0, 0x70);

/// One line describing what the player knows about `p`: who is there, what lies there (as seen
/// or remembered), on what (a known trap on its terrain: "on a pit on stone floor"), in which
/// special room (once they have been in it), and what was heard there; over the air of a floor
/// above the ground, what is far below ([`far_below`]). Blood they know of there
/// reads before the terrain ("on blood-stained stone floor"), and char before that ("on charred
/// stone floor"), if `blood` is shown. A named place
/// with a note adds it after: "A mountain pass, Beorggeat. The way you came; snow has closed it
/// behind you."; so does terrain with a description (a forgotten altar's), and then a
/// gravestone's inscription.
pub fn describe(view: &PerceptionView<'_>, p: Pos, blood: bool) -> String {
    let mut lines = vec![describe_cell(view, p, blood)];
    if let Some(below) = view.below(p) {
        lines.push(clause(T::FAR_BELOW, [("below", far_below(view, below))]));
    }
    // The data's own words after it, as written: a place's note, a terrain's description (a
    // forgotten altar's) or a chest's band's, and a gravestone's inscription last ("A
    // gravestone. Here lies Ælfric Wudu, 1381–1427.").
    lines.extend((view.place(p)).and_then(|place| place.note.clone().map(Arg::Text)));
    let content = view.content();
    let band = (view.chest_at(p)).and_then(|c| content.chests().bands.get(c.band));
    let described = match (band, view.cell(p)) {
        (Some(band), _) => Some(band.description.clone()),
        (None, CellView::Visible(t) | CellView::Remembered(t)) => {
            content.terrain(t).description.clone()
        }
        (None, CellView::Unknown) => None,
    };
    lines.extend(described.map(Arg::Text));
    lines.extend(
        view.epitaph(p)
            .map(|epitaph| Arg::Text(epitaph.to_string())),
    );
    ui_with(T::LINES, &[("lines", Arg::List(lines))])
}

/// What the player's Knowledge tells of the creature at `p`, if it tells anything: "You know the
/// skeleton is immune to poison, resistant to piercing, and vulnerable to bludgeoning." Words of
/// the `stats` layer's resistances, never a number (Knowledge).
pub fn known(view: &PerceptionView<'_>, p: Pos) -> Option<String> {
    let content = view.content();
    let actor = view.actor_at(p)?;
    let known = view.known_resistances(actor);
    let parts: Vec<Arg> = [
        (ResistanceWord::Immune, T::IMMUNE_TO),
        (ResistanceWord::Resistant, T::RESISTANT_TO),
        (ResistanceWord::Vulnerable, T::VULNERABLE_TO),
    ]
    .into_iter()
    .filter_map(|(word, text)| {
        // A damage type's name is the data's text until core's words are converted (#1949).
        let kinds: Vec<Arg> = (known.iter())
            .filter(|(_, w)| *w == word)
            .map(|(k, _)| Arg::Text(content.damage_type(*k).name.clone()))
            .collect();
        (!kinds.is_empty()).then(|| clause(text, [("kinds", Arg::List(kinds))]))
    })
    .collect();
    if parts.is_empty() {
        return None;
    }
    let who = match proper_name(content, actor) {
        Some(proper) => proper,
        None if actor.ally => made_of(content, actor, kind(content, actor)).owned_by(you()),
        None => made_of(content, actor, kind(content, actor)),
    };
    Some(ui_with(
        T::KNOWN,
        &[("who", who.into()), ("parts", Arg::List(parts))],
    ))
}

/// A UI entry said inside another, with its slots filled.
fn clause<const N: usize>(text: UiText, args: [(&'static str, Arg); N]) -> Arg {
    let msg = (args.into_iter()).fold(Msg::new(MessageId::new(text.id)), |msg, (role, arg)| {
        msg.role(role, arg)
    });
    msg.into()
}

/// An empty slot: a clause that does not apply.
fn none() -> Arg {
    Arg::Text(String::new())
}

/// `items` joined by `joiner` (`listed`, `tags`), a nested entry per join; none for none.
fn joined(joiner: UiText, items: Vec<Arg>) -> Option<Arg> {
    let mut items = items.into_iter().rev();
    let last = items.next()?;
    Some(items.fold(last, |rest, first| {
        clause(joiner, [("first", first), ("rest", rest)])
    }))
}

/// The player, as an owner ("your wolf").
fn you() -> Phrase {
    Phrase::literal("").person(Person::You)
}

/// A `word:` phrase: a word the data names without an id of its own (core's words, #1949).
fn word(text: &str) -> Phrase {
    Phrase::id(format!("word:{text}"))
}

/// A creature type as a phrase.
fn creature_phrase(content: &Content, creature: CreatureId) -> Phrase {
    Phrase::id(format!("creature:{}", content.creature(creature).id))
}

/// A seen creature's type as a phrase.
fn kind(content: &Content, actor: &SeenActor) -> Phrase {
    creature_phrase(content, actor.creature)
}

/// A terrain as a phrase.
fn terrain_phrase(content: &Content, t: TerrainId) -> Phrase {
    Phrase::id(format!("terrain:{}", content.terrain(t).id))
}

/// A known trap as a phrase: its name, until traps have lexicon ids.
fn trap_phrase(content: &Content, trap: TrapId) -> Phrase {
    word(&content.trap(trap).name)
}

/// What look mode says of a zone on a cell (`world/magic/zone.rs`).
fn zone_words(kind: ZoneKind) -> Arg {
    let text = match kind {
        ZoneKind::Fire => T::ZONE_FIRE,
        ZoneKind::Fouled => T::ZONE_FOULED,
        ZoneKind::Slick => T::ZONE_SLICK,
        ZoneKind::Dark => T::ZONE_DARK,
        ZoneKind::Smoke => T::ZONE_SMOKE,
    };
    clause(text, [])
}

/// [`describe`] without a place's note: who is there ([`creature`]), what lies there
/// ([`pile`]), on what ground ([`ground`], a known trap), and what was heard there.
fn describe_cell(view: &PerceptionView<'_>, p: Pos, blood: bool) -> Arg {
    let content = view.content();
    // The ground after "on", with the room it lies in.
    let on = |t| ground(view, p, t, blood, false, room_after(view, p));
    // A known trap is part of the ground: "a fireball trap on stone floor".
    let trap = (view.trap(p).filter(|_| view.chest_at(p).is_none()))
        .map(|t| Arg::from(trap_phrase(content, t)));
    let terrain = |t| match &trap {
        Some(trap) => clause(T::TRAP_ON, [("trap", trap.clone()), ("ground", on(t))]),
        None => on(t),
    };
    // The ground named alone, with its article unless it is named in the plural ("webs") or as
    // a mass noun ("open air").
    let named = |t| match &trap {
        Some(_) => terrain(t),
        None => ground(view, p, t, blood, true, room_after(view, p)),
    };
    // Nothing but a trap there: the trap first ("A pit, on stone floor.").
    if let Some(trap) = &trap
        && view.actor_at(p).is_none()
        && view.objects(p).is_empty()
    {
        return trap_alone(view, p, trap.clone(), on);
    }
    // A sound heard there (never on a cell the player sees): "You hear footsteps here."
    let heard = view.sound_at(p).map(|s| Arg::Text(s.you_hear()));
    let things = pile(view, p);
    let here = (things.clone()).map_or_else(none, |o| clause(T::HERE, [("things", o)]));
    // What the player remembers at a cell out of sight.
    let memory = |t| match &things {
        Some(things) => clause(
            T::THINGS_ON,
            [("things", things.clone()), ("ground", terrain(t))],
        ),
        None => named(t),
    };
    match (view.cell(p), heard) {
        (CellView::Visible(t), _) => match (view.actor_at(p), &things) {
            (Some(_), _) if p == view.viewer() => {
                clause(T::YOU, [("ground", terrain(t)), ("here", here)])
            }
            // The player's own image in a hall of mirrors.
            (Some(actor), _) if actor.kind == ActorKind::Player => {
                clause(T::REFLECTION, [("ground", terrain(t)), ("here", here)])
            }
            (Some(actor), _) => clause(
                T::SEEN_CREATURE,
                [
                    ("who", creature(view, actor)),
                    ("ground", terrain(t)),
                    ("here", here),
                ],
            ),
            (None, Some(things)) => clause(
                T::SEEN_THINGS,
                [("things", things.clone()), ("ground", terrain(t))],
            ),
            (None, None) => clause(T::SEEN_GROUND, [("ground", named(t))]),
        },
        (CellView::Remembered(t), Some(heard)) => clause(
            T::HEARD_WHERE_REMEMBERED,
            [("heard", heard), ("memory", memory(t))],
        ),
        (CellView::Remembered(t), None) => match &things {
            Some(things) => clause(
                T::REMEMBER_THINGS,
                [("things", things.clone()), ("ground", terrain(t))],
            ),
            None => {
                let room = (room(view, p))
                    .map_or_else(none, |room| clause(T::ROOM_AFTER_COMMA, [("room", room)]));
                let ground = ground(view, p, t, blood, true, none());
                clause(T::REMEMBER_GROUND, [("ground", ground), ("room", room)])
            }
        },
        (CellView::Unknown, Some(heard)) => clause(T::HEARD_HERE, [("heard", heard)]),
        (CellView::Unknown, None) => clause(T::UNKNOWN, []),
    }
}

/// What is far below an air cell of a floor above the ground, as the player sees or remembers it
/// (docs/dev/towers.md): the creature standing there as it stood ("a wolf, on grass"; it is not
/// examined further: how it stands toward them, how hurt it is, and what it wears stay unknown
/// from this height), else what lies there ("a dagger, on grass"), else the ground ("grass");
/// remembered, the ground as last seen from above.
fn far_below(view: &PerceptionView<'_>, below: BelowView<'_>) -> Arg {
    let content = view.content();
    match below {
        BelowView::Visible(cell) => {
            let ground = Arg::from(terrain_phrase(content, cell.terrain));
            match (&cell.actor, &cell.pile) {
                (Some(actor), _) => clause(
                    T::BELOW_CREATURE,
                    [
                        ("who", creature_phrase(content, actor.creature).into()),
                        ("ground", ground),
                    ],
                ),
                (None, Some(top)) => clause(
                    T::BELOW_THING,
                    [("what", view.lore().phrase(top).into()), ("ground", ground)],
                ),
                (None, None) => clause(T::BELOW_GROUND, [("ground", ground)]),
            }
        }
        BelowView::Remembered(t) => clause(
            T::BELOW_GROUND,
            [("ground", terrain_phrase(content, t).into())],
        ),
    }
}

/// The ground of `p`, of terrain `t`, as look mode names it, a clause: said after "on"
/// (`ground`), or alone with its article (`ground_named`, by the terrain's noun) if `named`;
/// with `room` after it ([`room_after`], or none).
fn ground(
    view: &PerceptionView<'_>,
    p: Pos,
    t: TerrainId,
    blood: bool,
    named: bool,
    room: Arg,
) -> Arg {
    let (phrase, of, place) = fixture(view, p, t, blood);
    let text = match (named, view.content().terrain(t).noun) {
        (false, _) => T::GROUND,
        (true, Noun::One) => T::GROUND_NAMED,
        (true, Noun::Plural) => T::GROUND_NAMED_PLURAL,
        (true, Noun::Mass) => T::GROUND_NAMED_MASS,
    };
    clause(
        text,
        [
            ("ground", phrase.into()),
            ("of", of),
            ("where", place),
            ("room", room),
        ],
    )
}

/// What the ground of `p`, of terrain `t`, is called with what stands on it, as look mode names
/// it: the terrain, or a chest or a statue there, a wall found hollow, with blood and char
/// before it if `blood` is shown; its `of` clause (a statue's, a chest's known trap); and its
/// `where` clause: where it leads, that it is partly dug, or the named place.
fn fixture(view: &PerceptionView<'_>, p: Pos, t: TerrainId, blood: bool) -> (Phrase, Arg, Arg) {
    let content = view.content();
    // A chest goes by its band: "a coffer", "an open coffer", "a locked coffer" (Chest-wise);
    // its known trap after it: "a coffer, trapped: an arrow trap".
    let chest = view.chest_at(p);
    let chest_trap = view.trap(p).filter(|_| chest.is_some());
    let def = content.terrain(t);
    let (phrase, of) = match (chest, view.statue_at(p)) {
        (Some(chest), _) => {
            // The band's name with its state: core's words, until they are converted (#1949).
            let name = word(&chest.name(content.chests(), &def.name));
            let trapped = chest_trap.map_or_else(none, |trap| {
                clause(T::TRAPPED, [("trap", trap_phrase(content, trap).into())])
            });
            (name, trapped)
        }
        // A statue of the stoned goes by what it was: "a statue of a dwarf, mouth open", or
        // by the past character the graveyard names it for: "a statue of Brakka the dwarf".
        (None, Some(was)) => {
            let of = match view.statue_name(p) {
                Some(name) => Phrase::literal(name),
                None => creature_phrase(content, was),
            };
            let of = clause(T::STATUE_OF, [("of", of.into())]);
            (terrain_phrase(content, t), of)
        }
        (None, None) => (terrain_phrase(content, t), none()),
    };
    // Char reads before the blood: "charred blood-stained stone floor"; a wall the player found
    // sounds hollow (a sealed room's breach) says so: "a hollow stone wall".
    let char = (view.scorch(p).filter(|_| blood)).map(|c| ascmearc_core::scorch::word(c));
    let stain = (view.blood(p).filter(|_| blood)).map(|stain| stain.word(!def.transparent));
    let hollow =
        (view.hollow(p) && chest.is_none() && view.statue_at(p).is_none()).then_some("hollow");
    let phrase = [char, stain, hollow]
        .into_iter()
        .flatten()
        .fold(phrase, |phrase, adjective| {
            phrase.with(Modifier::Adjective(Word::id(format!("word:{adjective}"))))
        });
    // Stairs and locations name where they lead: "a staircase down, leading to The Depths 1";
    // a wall in sight that was dug into says so: "a stone wall, partly dug"; a named place its
    // name: "a volcano, Fyrbeorg".
    let place = match view.leads_to(p) {
        Some(to) => clause(T::LEADING_TO, [("to", Arg::Text(to))]),
        None if view.is_dug(p) => clause(T::PARTLY_DUG, []),
        None => match view.place(p) {
            Some(place) => clause(T::PLACE, [("place", Arg::Text(place.name.clone()))]),
            None => none(),
        },
    };
    (phrase, of, place)
}

/// The room clause after the ground's name ([`room`]): after a comma where the ground leads
/// somewhere ("a staircase down, leading to The Vast Caverns, in a sanctuary"); none for none.
fn room_after(view: &PerceptionView<'_>, p: Pos) -> Arg {
    match (room(view, p), view.leads_to(p)) {
        (Some(room), Some(_)) => clause(T::ROOM_AFTER_COMMA, [("room", room)]),
        (Some(room), None) => clause(T::ROOM, [("room", room)]),
        (None, _) => none(),
    }
}

/// Where the walkable ground of `p` lies, as it reads after the ground's name: the special room
/// the player has been in ("in an echoing hall"), ground hallowed by Sanctuary ("on hallowed
/// ground", the dark powers' "on dreaded ground"), and a zone lying on it ("under a foul
/// reek"), a list; none for none of them.
fn room(view: &PerceptionView<'_>, p: Pos) -> Option<Arg> {
    let walkable = match view.cell(p) {
        CellView::Visible(t) | CellView::Remembered(t) => view.content().terrain(t).walkable,
        CellView::Unknown => false,
    };
    // The room's name with its article is core's words until they are converted (#1949).
    let room = (view.room(p).filter(|_| walkable))
        .map(|r| clause(T::IN_ROOM, [("room", Arg::Text(r.with_article()))]));
    let hallow = (view.hallow(p).filter(|_| walkable)).map(|dark| match dark {
        true => clause(T::DREADED, []),
        false => clause(T::HALLOWED, []),
    });
    let zone = view.zone_at(p).map(zone_words);
    joined(
        T::LISTED,
        [room, hallow, zone].into_iter().flatten().collect(),
    )
}

/// A cell that holds nothing but a known trap `trap`: the trap first ("A pit, on stone floor."),
/// on its ground (`on`).
fn trap_alone(view: &PerceptionView<'_>, p: Pos, trap: Arg, on: impl Fn(TerrainId) -> Arg) -> Arg {
    match (view.cell(p), view.sound_at(p)) {
        (CellView::Visible(t), _) => clause(T::SEEN_TRAP, [("trap", trap), ("ground", on(t))]),
        (CellView::Remembered(t), Some(heard)) => {
            let memory = clause(T::TRAP_ON, [("trap", trap), ("ground", on(t))]);
            clause(
                T::HEARD_WHERE_REMEMBERED,
                [("heard", Arg::Text(heard.you_hear())), ("memory", memory)],
            )
        }
        (CellView::Remembered(t), None) => {
            clause(T::REMEMBER_TRAP, [("trap", trap), ("ground", on(t))])
        }
        (CellView::Unknown, _) => clause(T::UNKNOWN, []),
    }
}

/// Stacks as the line names them: one its phrase, two or three a list, more a count of objects
/// ("17 objects"); `item` makes each of a list. None for none.
fn stacks(stacks: &[Stack], item: impl Fn(&Stack) -> Arg) -> Option<Arg> {
    match stacks {
        [] => None,
        [one] => Some(item(one)),
        [_, _] | [_, _, _] => Some(Arg::List(stacks.iter().map(item).collect())),
        _ => {
            let count: u32 = stacks.iter().map(|s| s.count).sum();
            Some(word("object").count(count).into())
        }
    }
}

/// What lies on `p` as look mode names it: "a dagger", "a dagger and 12 arrows", "17 objects";
/// none for nothing. Each says what it gives in numbers (`magic::numbers`), and goods for sale in a
/// shop their price: "a dagger (+0 hit, 1d4, range 5; for sale, 45 gold)".
fn pile(view: &PerceptionView<'_>, p: Pos) -> Option<Arg> {
    let content = view.content();
    let seen = matches!(view.cell(p), CellView::Visible(_));
    let item = |stack: &Stack| {
        let what = Arg::from(view.lore().phrase(stack));
        // The numbers are core's words until they are converted (#1949).
        let numbers = Numbers::of(stack, content).text(content).map(Arg::Text);
        // A trained eye (Appraise, Survival; #1031) tells food's freshness of what it sees.
        let fresh = (seen.then(|| view.judge(stack)).flatten())
            .map(|w| clause(T::LOOKS, [("fresh", Arg::Text(w.to_string()))]));
        let price = (view.price_tag(p, stack))
            .map(|g| clause(T::FOR_SALE, [("gold", Arg::Number(i64::from(g)))]));
        let tags = [numbers, fresh, price].into_iter().flatten().collect();
        match joined(T::TAGS, tags) {
            Some(tags) => clause(T::TAGGED, [("what", what), ("tags", tags)]),
            None => what,
        }
    };
    stacks(view.objects(p), item)
}

/// A creature the player sees (not themselves), as it reads before ", on" its ground: its name,
/// how it stands toward them, how hurt it looks and the effects that show on it, and what it
/// wears and wields.
fn creature(view: &PerceptionView<'_>, actor: &SeenActor) -> Arg {
    let content = view.content();
    let proper = proper_name(content, actor);
    // A named ally of the player's is theirs: "(your ally, unhurt)"; any other ally is said
    // as theirs: "Your wolf (unhurt)".
    let ally = (actor.ally && proper.is_some()).then(|| clause(T::ALLY, []));
    // Its type tells the player how it stands toward them (its faction).
    let hostile = match view.stance(actor) {
        _ if actor.ally => None,
        Stance::Hostile => Some(clause(T::HOSTILE, [])),
        Stance::Neutral => None,
    };
    // They see how hurt it is: in words, never in hit points.
    let health = Some(Arg::from(actor.health.word()));
    // The status effects that show on it: ", bleeding" (the data's words, #1949).
    let effects = (view.effects_of(actor.id))
        .filter_map(|s| content.status(s).seen.clone())
        .map(Arg::Text);
    // Stalker's eye: it has not noticed the player (never said without it).
    let unaware = actor.unaware.then(|| clause(T::UNAWARE, []));
    // The creature the player's openings are in (`world/classes/mark.rs`).
    let marked = actor.marked.then(|| clause(T::YOUR_MARK, []));
    let state: Vec<Arg> = [ally, hostile, health]
        .into_iter()
        .flatten()
        .chain(effects)
        .chain([unaware, marked].into_iter().flatten())
        .collect();
    let state = joined(T::LISTED, state).unwrap_or_else(none);
    // What lives in it shows (docs/design/monster-dread.md §5.5): "writhing".
    let writhing = match content.creature(actor.creature).dies_into {
        Some(_) => clause(T::WRITHING, []),
        None => none(),
    };
    let moves = (moves(content, actor)).map_or_else(none, |m| clause(T::MOVES, [("moves", m)]));
    let [wielding, wearing, ready] = gear(view, actor.id);
    let after = |who: Arg, services: Arg, waits: Arg| {
        [
            ("who", who),
            ("state", state.clone()),
            ("services", services),
            ("waits", waits),
            ("writhing", writhing.clone()),
            ("moves", moves.clone()),
            ("wielding", wielding.clone()),
            ("wearing", wearing.clone()),
            ("ready", ready.clone()),
        ]
    };
    match proper {
        // A unique character the player knows by name: "Sister Hild (unhurt), who
        // cures and mends for gold, on plank floor."; or a named monster by its type's
        // proper name: "Black Wystan (hostile, unhurt), ...".
        Some(proper) => clause(
            T::CREATURE,
            after(proper.into(), services(content, actor), none()),
        ),
        // The player's kept beast by its name: "Snel, your hound (unhurt), ...".
        None if actor.companion => {
            let name = (view.kept()).find(|(a, _)| a.id == actor.id).map_or_else(
                || clause(T::YOUR_COMPANION, []),
                |(_, c)| Arg::Text(c.name.clone()),
            );
            let who = made_of(content, actor, kind(content, actor)).owned_by(you());
            clause(
                T::COMPANION,
                [
                    ("name", name),
                    ("who", who.into()),
                    ("state", state),
                    ("writhing", writhing),
                    ("moves", moves),
                    ("wielding", wielding),
                    ("wearing", wearing),
                    ("ready", ready),
                ],
            )
        }
        // Its grade, which anyone sees: "A big kobold"; messages name its type alone.
        None => {
            let def = content.creature(actor.creature);
            let who = kind(content, actor);
            let who = match ascmearc_core::grade::word(def, actor.grade) {
                Some(grade) => who.with(Modifier::Adjective(Word::id(format!("word:{grade}")))),
                None => who,
            };
            let who = made_of(content, actor, who);
            let who = match actor.ally {
                true => who.owned_by(you()),
                false => who,
            };
            clause(
                T::CREATURE,
                after(who.into(), none(), waits(content, actor)),
            )
        }
    }
}

/// The proper name a seen creature goes by, if any: a unique character's or a named monster's.
fn proper_name(content: &Content, actor: &SeenActor) -> Option<Phrase> {
    let name = &content.creature(actor.creature).name;
    match (actor.dialogue).and_then(|d| content.dialogue(d).name.as_deref()) {
        Some(name) => Some(Phrase::literal(name)),
        None => ascmearc_core::content::is_proper(name).then(|| kind(content, actor)),
    }
}

/// What a seen creature sells as a service, as it reads after its name: ", who cures and mends
/// for gold"; nothing if it sells none (campaign design §6.1).
fn services(content: &Content, actor: &SeenActor) -> Arg {
    let Some(dialogue) = actor.dialogue else {
        return none();
    };
    let mut verbs: Vec<UiText> = Vec::new();
    for service in &content.dialogue(dialogue).services {
        let verb = match service.does {
            ServiceKind::Cure => T::CURES,
            ServiceKind::Decurse => T::FREES_THE_CURSED,
            ServiceKind::Identify => T::IDENTIFIES,
            ServiceKind::Mend => T::MENDS,
            ServiceKind::TrainSkill => T::TEACHES,
        };
        if !verbs.contains(&verb) {
            verbs.push(verb);
        }
    }
    if verbs.is_empty() {
        return none();
    }
    let verbs = verbs.into_iter().map(|verb| clause(verb, [])).collect();
    clause(T::SERVICES, [("verbs", Arg::List(verbs))])
}

/// ", who waits" after a seen creature's name when its dialogue's only topics are callings'
/// (`calls`), with no service or wares (the Strange Spirit: "a strange spirit (unhurt), who
/// waits"; docs/design/prestige-classes.md §7); nothing for any other.
fn waits(content: &Content, actor: &SeenActor) -> Arg {
    let Some(dialogue) = actor.dialogue else {
        return none();
    };
    let d = content.dialogue(dialogue);
    let callings = !d.topics.is_empty() && d.topics.iter().all(|t| t.calls.is_some());
    match callings && d.services.is_empty() && d.offer.is_none() {
        true => clause(T::WAITS, []),
        false => none(),
    }
}

/// What a seen animated thing's body is made of, if it is one.
fn animated(content: &Content, actor: &SeenActor) -> Option<MaterialId> {
    // A household thing's body; a blade's weapon is told by the gear line instead.
    (actor.shape)
        .filter(|_| content.animated_body(actor.creature).is_some())
        .map(|shape| shape.material)
}

/// `who`, a seen creature's phrase, with what it is made of if it is an animated thing:
/// "animated candlestick of silver".
fn made_of(content: &Content, actor: &SeenActor, who: Phrase) -> Phrase {
    match animated(content, actor) {
        Some(material) => {
            let material = format!("material:{}", content.material(material).id);
            who.with(Modifier::Of(Word::id(material)))
        }
        None => who,
    }
}

/// How a seen animated thing moves, the data's word ("rocking"); none for any other creature.
fn moves(content: &Content, actor: &SeenActor) -> Option<Arg> {
    animated(content, actor)?;
    let body = content.animated_body(actor.creature)?;
    body.spec.moves.clone().map(Arg::Text)
}

/// What a seen creature wields, wears, and has ready, as each reads after its name: ", wielding
/// a spear", ", wearing a leather jerkin", ", with a short bow and 20 arrows ready"; each none if
/// it has none. That is all the player sees of what it carries.
fn gear(view: &PerceptionView<'_>, actor: ActorId) -> [Arg; 3] {
    let content = view.content();
    let mut groups: [Vec<Stack>; 3] = Default::default();
    for g in view.gear(actor) {
        let group = match content.slot(g.slot).action {
            ActionKind::Wield => 0,
            ActionKind::Ready => 2,
            _ => 1,
        };
        groups[group].push(g.stack.clone());
    }
    let said = |text: UiText, group: &[Stack]| {
        stacks(group, |stack| view.lore().phrase(stack).into())
            .map_or_else(none, |what| clause(text, [("what", what)]))
    };
    [
        said(T::WIELDING, &groups[0]),
        said(T::WEARING, &groups[1]),
        said(T::READY, &groups[2]),
    ]
}

/// Draws the cursor at console cell `(x, y)`, showing what the player perceives there.
pub fn draw_cursor(console: &mut Console, x: i32, y: i32, view: &PerceptionView<'_>, p: Pos) {
    draw_cursor_on(console, (x, y), view, p, CURSOR_BG);
}

/// [`draw_cursor`] on background `bg` (targeting marks a target out of reach with its own).
pub fn draw_cursor_on(
    console: &mut Console,
    (x, y): (i32, i32),
    view: &PerceptionView<'_>,
    p: Pos,
    bg: Rgb,
) {
    let content = view.content();
    let top = view.objects(p).first();
    // A chest keeps its glyph over its known trap.
    let trap = view.trap(p).filter(|_| view.chest_at(p).is_none());
    let ch = match (view.cell(p), view.actor_at(p), top) {
        (CellView::Visible(_), Some(actor), _) => actor.glyph(content),
        _ if view.sound_at(p).is_some() => super::sounds::MARKER,
        (CellView::Visible(_) | CellView::Remembered(_), None, _) if trap.is_some() => {
            trap.map_or(' ', |t| content.trap(t).glyph)
        }
        (CellView::Visible(_) | CellView::Remembered(_), None, Some(top)) => {
            content.object(top.object).glyph
        }
        (CellView::Visible(t) | CellView::Remembered(t), _, _) => content.terrain(t).glyph,
        (CellView::Unknown, _, _) => ' ',
    };
    console.put(
        x,
        y,
        Cell {
            ch,
            fg: CURSOR_FG,
            bg,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascmearc_core::content::Content;
    use ascmearc_core::{Action, Dir, World};
    use std::sync::Arc;

    fn world() -> World {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        World::new(content, "showcase", None).expect("world")
    }

    #[test]
    fn a_companion_is_seen_wearing_its_collar() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let spec = "human,male,ranger".parse().expect("a character");
        let mut w = World::start_run(content, "showcase", None, Some(&spec), None, true)
            .expect("a ranger's run");
        // To the darkness chamber's amulet and out to the hound, in the light of the lantern.
        let walk = "move:N*3 open:N move:N*2 move:E*30 open:N move:N*3 move:W*6 move:N*5 move:E*6 move:S*2 move:W*2 pickup move:E*2 move:N*2 wait*2 move:W use_tool";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        // The amulet picked up, wherever the kit left room for it (a ranger's random scrolls).
        let amulet = (w.perception().inventory().iter())
            .position(|s| w.content().object(s.object).id == "bone_amulet")
            .expect("the amulet");
        let give = format!("give:{amulet}");
        for action in ascmearc_core::action::parse_actions(&give).expect("actions") {
            w.apply(&action);
        }
        let view = w.perception();
        let (ally, data) = view.companion().expect("a companion");
        assert_eq!(ally.wears.len(), 1, "it took the amulet");
        let seen = (view.actors().iter()).find(|a| a.id == ally.id);
        let at = seen.expect("the hound is in sight").pos;
        let line = describe(&view, at, false);
        assert!(
            line.starts_with(&format!("{}, your hound (unhurt), wearing a ", data.name)),
            "{line}"
        );
    }

    #[test]
    fn named_places_say_their_name_and_note() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "ascmearc", None).expect("the Ascmearc");
        let pass = w.player_pos();
        assert_eq!(
            describe(&w.perception(), pass, true),
            "You, on mountain pass, Beorggeat. The way you came; snow has closed it behind you."
        );
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::E));
        }
        let town = Pos::new(18, 28);
        assert_eq!(
            describe(&w.perception(), town, true),
            "A town, leading to Cēaptūn."
        );
        assert_eq!(
            describe(&w.perception(), pass, true),
            "A mountain pass, Beorggeat. The way you came; snow has closed it behind you."
        );
    }

    #[test]
    fn a_gravestone_reads_its_inscription() {
        // Līctūn, Cēaptūn's churchyard: four cells down the south road.
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "ascmearc", None).expect("the Ascmearc");
        let walk = [(Dir::E, 6), (Dir::S, 4), (Dir::E, 1)];
        for (dir, n) in walk {
            for _ in 0..n {
                w.apply(&Action::Move(dir));
            }
        }
        w.apply(&Action::Descend);
        assert_eq!(w.perception().level_id(), "lictun");
        // One of the town's own, by the chapel (a fixed grave, `docs/dev/graves.md`).
        let stone = Pos::new(21, 19);
        let cut = w.ground_truth().0.graves()[&stone].clone();
        assert_eq!(
            describe(&w.perception(), stone, true),
            format!("A gravestone. {cut}")
        );
    }

    #[test]
    fn leavings_are_named_for_what_they_are() {
        // Webs, acid-pitted floor, and torn cloth (`content/leaving.rs`), never what left them.
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/leavings.toml",
            "name = \"Leavings\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#@wac.#\n\
             #######\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"w\" = { terrain = \"web\" }\n\"a\" = { terrain = \"acid_pit\" }\n\
             \"c\" = { terrain = \"floor\", objects = [\"torn_cloth\"] }\n",
        );
        files.insert(
            "scenarios/leavings.toml",
            "name = \"Leavings\"\nstart_level = \"leavings\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "leavings", None).expect("world");
        let view = w.perception();
        let at = |x| describe(&view, Pos::new(x, 1), false);
        assert!(at(2).starts_with("Webs."), "{}", at(2));
        // A mass noun takes no article (as open air, #1781).
        assert!(at(3).starts_with("Acid-pitted floor."), "{}", at(3));
        assert!(at(4).contains("scrap of torn cloth"), "{}", at(4));
    }

    #[test]
    fn a_forgotten_altar_says_what_it_is_and_a_spent_one_too() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "shrine", None).expect("the shrine");
        let walk = "move:N open:N move:N*3";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        let altar = Pos::new(13, 3);
        assert_eq!(
            describe(&w.perception(), altar, true),
            "A forgotten altar. An altar of worn stone. Its carvings have been rubbed smooth by \
             hands or by time, and whatever god it served has no name left. Something here still \
             listens."
        );
        for _ in 0..8 {
            w.apply(&Action::Pray);
        }
        assert_eq!(
            describe(&w.perception(), altar, true),
            "A spent altar. An altar of worn stone, cold and dark. Whatever listened here has \
             stopped.",
            "eight prayers at one altar spend it, but for one chance in 256"
        );
    }

    /// A chest goes by its band, with the band's description; open, it is the pile's ground.
    #[test]
    fn a_chest_goes_by_its_band() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/look_chests.toml",
            "name = \"Chests\"\nambient = [255, 255, 255, 0]\nitem_level = 9\nmap = '''\n\
             #####\n#C.K#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"C\" = { terrain = \"chest\", chest = \"coffer\", objects = [\"dagger\"] }\n\
             \"K\" = { terrain = \"chest\", locked = true, trap = \"arrow\", known = true }\n",
        );
        files.insert(
            "scenarios/look_chests.toml",
            "name = \"C\"\nstart_level = \"look_chests\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "look_chests", None).expect("world");
        let (coffer, banded) = (Pos::new(1, 1), Pos::new(3, 1));
        assert_eq!(
            describe(&w.perception(), coffer, true),
            "A coffer. A small wooden box with an iron hasp."
        );
        assert_eq!(
            describe(&w.perception(), banded, true),
            "A banded chest, trapped: an arrow trap. A heavy chest banded with black iron, its \
             lid studded with nails.",
            "the level's own: 9; a known trap after the name; whether it is locked is not shown"
        );
        w.apply(&Action::Open(Dir::N));
        assert_eq!(
            describe(&w.perception(), coffer, true),
            "A dagger (+0 hit, 1d4, range 5), on open coffer. A small wooden box with an iron hasp."
        );
    }

    /// A disguised mimic is a coffer in look mode, word for word; found out, it is the creature.
    /// What a wizard's Knowledge tells, in the stats layer's words; nothing of a creature above
    /// it, nor of what the player cannot see as a creature (a disguised mimic). The core tests
    /// hold the level's edge (`knowledge_tests.rs`).
    #[test]
    fn knowledge_says_what_a_creature_resists_in_words() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/look_lore.toml",
            "name = \"Lore\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             #######\n#s.M.d#\n#@.C..#\n#######\n'''\n[legend]\n\".\" = \"floor\"\n\
             \"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"s\" = { terrain = \"floor\", creature = \"skeleton\" }\n\
             \"d\" = { terrain = \"floor\", creature = \"rotting_drake\" }\n\
             \"M\" = { terrain = \"floor\", chest = 1, mimic = true }\n\
             \"C\" = { terrain = \"chest\", chest = 1 }\n",
        );
        files.insert(
            "scenarios/look_lore.toml",
            "name = \"L\"\nstart_level = \"look_lore\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "human,male,wizard".parse().expect("a character");
        let w = World::start_run(content, "look_lore", None, Some(&spec), None, true)
            .expect("a wizard's run");
        let view = w.perception();
        assert_eq!(
            known(&view, Pos::new(1, 1)).as_deref(),
            Some(
                "You know the skeleton is immune to poison and necrotic, resistant to piercing, \
                 and vulnerable to bludgeoning."
            )
        );
        assert_eq!(known(&view, Pos::new(5, 1)), None, "above their Knowledge");
        assert_eq!(
            known(&view, Pos::new(3, 1)),
            None,
            "a chest, as they see it"
        );
        assert_eq!(known(&view, w.player_pos()), None);
        assert_eq!(known(&view, Pos::new(2, 1)), None, "nobody there");
    }

    #[test]
    fn a_disguised_mimic_looks_like_a_chest() {
        let mut files = ascmearc_assets::files();
        files.insert(
            "levels/look_mimic.toml",
            "name = \"Mimic\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
             #####\n#M.C#\n#@..#\n#####\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
             \"M\" = { terrain = \"floor\", chest = 1, mimic = true }\n\
             \"C\" = { terrain = \"chest\", chest = 1 }\n",
        );
        files.insert(
            "scenarios/look_mimic.toml",
            "name = \"M\"\nstart_level = \"look_mimic\"\nhp = 1000\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "look_mimic", None).expect("world");
        let (mimic, coffer) = (Pos::new(1, 1), Pos::new(3, 1));
        assert_eq!(
            describe(&w.perception(), mimic, true),
            "A coffer. A small wooden box with an iron hasp."
        );
        assert_eq!(
            describe(&w.perception(), mimic, true),
            describe(&w.perception(), coffer, true)
        );
        // Touched, it is found out: a creature.
        w.apply(&Action::Move(Dir::N));
        let seen = describe(&w.perception(), mimic, true);
        assert!(seen.starts_with("A mimicling (hostile"), "{seen}");
    }

    #[test]
    fn people_are_named_with_the_services_they_sell() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "ceaptun", None).expect("Cēaptūn");
        let walk = "move:SW*4 move:W*15 open:SW move:SW move:S*2";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        assert_eq!(w.player_pos(), Pos::new(26, 24), "inside the Mynster");
        assert_eq!(
            describe(&w.perception(), Pos::new(27, 25), true),
            "Sister Hild (unhurt), who cures, mends, and frees the cursed for gold, on plank floor in a \
             sanctuary."
        );
    }

    /// One who knows words and has nothing else to offer waits (the Strange Spirit,
    /// docs/design/prestige-classes.md §7); the encounter is forced, whoever the player is.
    #[test]
    fn one_with_words_and_nothing_to_ask_waits() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content.clone(), "ascmearc", None).expect("the Ascmearc");
        let walk = "move:E*7 skip:4h meet:strange_spirit_roads encounter:hail";
        for action in ascmearc_core::action::parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
        let spirit = content.creature_id("strange_spirit").expect("the spirit");
        let view = w.perception();
        let at = (view.actors().iter())
            .find(|a| a.creature == spirit)
            .map(|a| a.pos)
            .expect("hailed, and in sight");
        let seen = describe(&w.perception(), at, true);
        assert!(
            seen.starts_with("A strange spirit (unhurt), who waits, on "),
            "{seen}"
        );
    }

    #[test]
    fn a_zone_says_what_lies_on_the_ground() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "showcase", None).expect("the hub");
        let me = w.player_pos();
        let spot = ascmearc_core::geom::Pos::new(me.x + 1, me.y);
        let words: Vec<(ZoneKind, String)> = (ZoneKind::ALL.iter())
            .map(|&kind| {
                let lay = format!("zone:{}@{},{},0", kind.id(), spot.x, spot.y);
                w.apply(&lay.parse().expect("parses"));
                (kind, describe(&w.perception(), spot, true))
            })
            .collect();
        insta::assert_debug_snapshot!(words);
    }

    #[test]
    fn hallowed_ground_says_so() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let spec: ascmearc_core::character::CharacterSpec =
            "human,female,priest".parse().expect("a character");
        let mut w = World::start_run(
            content.clone(),
            "showcase",
            Some(7),
            Some(&spec),
            None,
            true,
        )
        .expect("world");
        let factor = ascmearc_core::content::Factor::of(&content, Some(&spec));
        let need = content.experience().threshold_for(50, factor);
        let actions = format!("xp:{need} levelup:auto use:sanctuary");
        for action in ascmearc_core::action::parse_actions(&actions).expect("actions") {
            w.apply(&action);
        }
        let me = w.player_pos();
        assert_eq!(w.hallows().len(), 1, "hallowed");
        assert_eq!(
            describe(&w.perception(), me, true),
            "You, on stone floor on hallowed ground."
        );
    }

    #[test]
    fn describes_the_player_terrain_memory_and_the_unknown() {
        let mut w = world();
        let me = w.player_pos();
        assert_eq!(describe(&w.perception(), me, true), "You, on stone floor.");
        let wall = Pos::new(0, me.y);
        assert_eq!(describe(&w.perception(), wall, true), "A stone wall.");
        let door = Pos::new(me.x, me.y - 4);
        assert_eq!(describe(&w.perception(), door, true), "A closed door.");
        assert_eq!(
            describe(&w.perception(), Pos::new(70, 3), true),
            "You don't know what is there."
        );
        // Walk out of the start room: the room's floor is remembered, not seen.
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        for _ in 0..8 {
            w.apply(&Action::Move(Dir::E));
        }
        let text = describe(&w.perception(), me, true);
        assert!(text.starts_with("You remember"), "{text}");
    }

    #[test]
    fn stairs_name_where_they_lead() {
        let mut w = world();
        // Walk to the stairs nook (see the golden manifest's `stairs-nook` walk).
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 67))
            .chain([Action::Open(Dir::S)])
            .chain(steps(Dir::S, 4))
            .collect();
        for action in &walk {
            w.apply(action);
        }
        let view = w.perception();
        let (level, _) = w.ground_truth();
        let down = (0..level.height())
            .flat_map(|y| (0..level.width()).map(move |x| Pos::new(x, y)))
            .find(|&p| view.leads_to(p).as_deref() == Some("The Vast Caverns"))
            .expect("the staircase to the vast level is in view");
        let text = describe(&view, down, true);
        // The stairs nook is a sanctuary, which the player knows once they have been in it.
        assert_eq!(
            text,
            "A staircase down, leading to The Vast Caverns, in a sanctuary."
        );
        let here = describe(&view, w.player_pos(), true);
        assert_eq!(here, "You, on stone floor in a sanctuary.");
        let wall = Pos::new(79, 15);
        assert_eq!(
            describe(&view, wall, true),
            "A stone wall.",
            "walls name no room"
        );
        // Walking back out, the nook is remembered as a sanctuary.
        for action in [Action::Move(Dir::N), Action::Move(Dir::N)] {
            w.apply(&action);
        }
        w.apply(&Action::Move(Dir::N));
        w.apply(&Action::Move(Dir::N));
        w.apply(&Action::Move(Dir::W));
        w.apply(&Action::Move(Dir::W));
        w.apply(&Action::Move(Dir::W));
        let view = w.perception();
        let floor = Pos::new(76, 18);
        let text = describe(&view, floor, true);
        assert_eq!(text, "You remember a stone floor here, in a sanctuary.");
    }

    #[test]
    fn special_rooms_are_named_only_once_entered() {
        let mut w = world();
        // Up the corridor to the stairs nook's door: the nook is seen, but not yet known.
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 67))
            .chain([Action::Open(Dir::S)])
            .collect();
        for action in &walk {
            w.apply(action);
        }
        let inside = Pos::new(74, 13);
        let view = w.perception();
        assert!(view.is_visible(inside));
        assert_eq!(describe(&view, inside, true), "A stone floor.");
        w.apply(&Action::Move(Dir::S));
        w.apply(&Action::Move(Dir::S));
        assert_eq!(
            describe(&w.perception(), inside, true),
            "A stone floor in a sanctuary."
        );
    }

    #[test]
    fn heard_sounds_are_described_where_they_were_heard() {
        let mut w = world();
        // Along the corridor to the patrol gallery's closed door: the kobold is heard behind it.
        let steps = |dir, n| std::iter::repeat_n(Action::Move(dir), n);
        let walk: Vec<Action> = steps(Dir::N, 3)
            .chain([Action::Open(Dir::N)])
            .chain(steps(Dir::N, 2))
            .chain(steps(Dir::E, 36))
            .collect();
        for action in &walk {
            w.apply(action);
        }
        while w.perception().sounds().is_empty() && w.turn() < 200 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let sound = view.sounds().first().expect("footsteps behind the door");
        let text = describe(&view, sound.pos, true);
        assert!(
            text == "You hear footsteps here." || text == "You faintly hear footsteps here.",
            "{text}"
        );
    }

    #[test]
    fn objects_are_named_where_they_lie_and_where_they_are_remembered() {
        let mut w = world();
        let gold = Pos::new(10, 14);
        let pile = Pos::new(5, 17);
        assert_eq!(
            describe(&w.perception(), gold, true),
            "12 gold coins, on stone floor."
        );
        assert_eq!(
            describe(&w.perception(), pile, true),
            "A leather jerkin (Dv +1, Pv 1), a dagger (+0 hit, 1d4, range 5), and 2 wax candles, \
             on stone floor."
        );
        w.apply(&Action::Move(Dir::SW));
        w.apply(&Action::Move(Dir::SW));
        assert_eq!(
            describe(&w.perception(), pile, true),
            "You, on stone floor; here: a leather jerkin (Dv +1, Pv 1), a dagger (+0 hit, 1d4, \
             range 5), and 2 wax candles."
        );
        // Out of the room, the gold is remembered.
        for dir in [Dir::NE, Dir::NE, Dir::N, Dir::N, Dir::N] {
            w.apply(&Action::Move(dir));
        }
        w.apply(&Action::Open(Dir::N));
        for _ in 0..3 {
            w.apply(&Action::Move(Dir::N));
        }
        w.apply(&Action::Move(Dir::E));
        for _ in 0..8 {
            w.apply(&Action::Move(Dir::E));
        }
        let text = describe(&w.perception(), gold, true);
        assert_eq!(text, "You remember 12 gold coins here, on stone floor.");
        let mut console = Console::default();
        console.resize(3, 1);
        draw_cursor(&mut console, 1, 0, &w.perception(), gold);
        assert_eq!(console.get(1, 0).map(|c| c.ch), Some('$'));
    }

    #[test]
    fn hostile_creatures_are_marked_as_such() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        w.apply(&Action::Wait);
        let view = w.perception();
        let content = view.content();
        let described: Vec<(String, String)> = view
            .actors()
            .iter()
            .filter(|a| a.pos != view.viewer())
            .map(|a| {
                let id = content.creature(a.creature).id.clone();
                (id, describe(&view, a.pos, true))
            })
            .collect();
        let of = |id: &str| described.iter().find(|(c, _)| c == id).map(|(_, d)| d);
        let rat = of("rat").expect("a giant rat in view");
        assert!(
            rat.starts_with("A giant rat (hostile, unhurt), on "),
            "{rat}"
        );
        let bat = of("bat").expect("a cave bat in view");
        // Its grade word, if it has a grade, comes first.
        assert!(
            ["A cave bat", "A big cave bat", "A small cave bat"]
                .iter()
                .any(|start| bat.starts_with(&format!("{start} (unhurt), on "))),
            "{bat}"
        );
    }

    #[test]
    fn a_greater_or_lesser_creature_is_named_with_its_grade() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        w.apply(&Action::Wait);
        let view = w.perception();
        let content = view.content();
        let mut graded = 0;
        for a in view.actors().iter().filter(|a| a.pos != view.viewer()) {
            let def = content.creature(a.creature);
            let text = describe(&view, a.pos, true);
            match ascmearc_core::grade::word(def, a.grade) {
                Some(word) => {
                    graded += 1;
                    let named = format!("{word} {} (", def.name);
                    assert!(text.contains(&named), "{text}");
                }
                None => assert!(text.contains(&format!(" {} (", def.name)), "{text}"),
            }
        }
        assert!(graded > 0, "some creature in view is graded");
    }

    #[test]
    fn creatures_look_as_hurt_as_they_are_in_words() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The brawl at the fire pit goes on until someone in view is hurt.
        let hurt = (0..60).find_map(|_| {
            w.apply(&Action::Wait);
            let view = w.perception();
            let actor = view.actors().iter().find(|a| {
                a.pos != view.viewer() && a.health != ascmearc_core::combat::Health::Unhurt
            })?;
            Some((actor.health, describe(&view, actor.pos, true)))
        });
        let (health, text) = hurt.expect("someone in view was hurt");
        // The health word, then what shows of its statuses (", bleeding") if any.
        let word = crate::ui::text::word(health.word(), "bare");
        let named = [format!(", {word})"), format!(", {word}, ")];
        assert!(named.iter().any(|n| text.contains(n)), "{text}");
        // Never a number of hit points.
        assert!(!text.chars().any(|c| c.is_ascii_digit()), "{text}");
    }

    #[test]
    fn a_bleeding_creature_looks_it() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The brawl at the fire pit wounds someone in view before long.
        let bleeding = (0..60).find_map(|_| {
            w.apply(&Action::Wait);
            let view = w.perception();
            let actor = (view.actors().iter()).find(|a| view.effects_of(a.id).next().is_some())?;
            Some(describe(&view, actor.pos, true))
        });
        let text = bleeding.expect("someone in view bled");
        assert!(text.contains(", bleeding)"), "{text}");
        assert!(!text.chars().any(|c| c.is_ascii_digit()), "{text}");
    }

    #[test]
    fn blood_is_named_before_the_terrain_unless_hidden() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        for _ in 0..40 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let stained = (0..view.height())
            .flat_map(|y| (0..view.width()).map(move |x| Pos::new(x, y)))
            .find(|&p| {
                view.blood(p).is_some() && view.actor_at(p).is_none() && view.objects(p).is_empty()
            })
            .expect("the brawl left blood in view");
        let text = describe(&view, stained, true);
        let word = view.blood(stained).expect("blood").word(false);
        assert_eq!(text, format!("A {word} stone floor."));
        assert_eq!(describe(&view, stained, false), "A stone floor.");
    }

    #[test]
    fn creatures_are_seen_with_what_they_wield_and_wear() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "arena", None).expect("arena");
        // The war band comes into the torchlight of the fire pit, all of it still standing.
        for _ in 0..8 {
            w.apply(&Action::Wait);
        }
        let view = w.perception();
        let content = view.content();
        let described: Vec<(String, String)> = (view.actors().iter())
            .map(|a| {
                let id = content.creature(a.creature).id.clone();
                (id, describe(&view, a.pos, true))
            })
            .collect();
        let of = |id: &str| {
            (described
                .iter()
                .find(|(c, _)| c == id)
                .map(|(_, d)| d.as_str()))
            .unwrap_or_else(|| panic!("a {id} in view: {described:#?}"))
        };
        // How hurt each looks comes before what it wields: "(hostile, wounded), wielding...".
        let kobold = of("kobold");
        assert!(
            kobold.starts_with("A kobold (hostile, ")
                && kobold.contains("), wielding a spear, on "),
            "{kobold}"
        );
        let archer = of("kobold_archer");
        assert!(
            [
                "A kobold archer",
                "A big kobold archer",
                "A small kobold archer"
            ]
            .iter()
            .any(|start| archer.starts_with(&format!("{start} (hostile, ")))
                && archer.contains("), with a short bow and ")
                && archer.contains(" arrows ready, on "),
            "{archer}"
        );
        let shaman = of("kobold_shaman");
        assert!(
            shaman.starts_with("A kobold shaman (hostile, ")
                && shaman.contains("), wearing a bone amulet, on "),
            "{shaman}"
        );
        // Nothing else of what they carry; and nothing of what the rats do not have.
        let rat = of("rat");
        assert!(rat.starts_with("A giant rat (hostile, ") && !rat.contains("wield"));
    }

    #[test]
    fn known_traps_are_named_as_part_of_the_ground() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "traps", None).expect("the trap hall");
        // A hidden trap (the fireball in the crossing) is just floor.
        assert_eq!(
            describe(&w.perception(), Pos::new(27, 11), true),
            "A stone floor."
        );
        let actions = ascmearc_core::action::parse_actions("move:NW*14 move:W*7 move:N*2").unwrap();
        for action in actions {
            w.apply(&action);
        }
        assert_eq!(
            describe(&w.perception(), Pos::new(16, 2), true),
            "An arrow trap, on stone floor."
        );
        // On the trap with the player: part of the ground they stand on.
        w.apply(&Action::Move(Dir::N));
        let view = w.perception();
        let here = view.viewer();
        assert!(describe(&view, here, true).starts_with("You, on an arrow trap on stone floor"));
    }

    #[test]
    fn animated_things_are_seen_by_what_they_are_made_of() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.c@r.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"c" = { terrain = "floor", creature = "animated_candlestick", animated = { material = "silver" } }
"r" = { terrain = "floor", creature = "animated_rug" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "hall", None).expect("hall");
        let view = w.perception();
        assert_eq!(
            describe(&view, Pos::new(2, 1), true),
            "An animated candlestick of silver (hostile, unhurt), rocking, on stone floor."
        );
        assert_eq!(
            describe(&view, Pos::new(4, 1), true),
            "An animated rug of wool (hostile, unhurt), rippling, on stone floor."
        );
    }

    #[test]
    fn what_lives_in_a_wormhost_shows_writhing() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.w@h.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"w" = { terrain = "floor", creature = "wormhost" }
"h" = { terrain = "floor", creature = "charnel_heap" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "hall", None).expect("hall");
        let view = w.perception();
        // Either may be graded ("A withered charnel heap"): the grade word is not the point.
        let host = describe(&view, Pos::new(2, 1), true);
        assert!(
            host.ends_with("wormhost (hostile, unhurt), writhing, on stone floor."),
            "{host}"
        );
        let heap = describe(&view, Pos::new(4, 1), true);
        assert!(
            heap.ends_with("charnel heap (hostile, unhurt), on stone floor."),
            "{heap}"
        );
    }

    #[test]
    fn a_wall_found_to_sound_hollow_says_so() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#@.#....#
#..#....#
#########
'''
[[room]]
effect = "great_treasury"
area = [3, 0, 8, 3]
breach = [3, 1]
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "hall", None).expect("hall");
        let breach = Pos::new(3, 1);
        assert_eq!(describe(&w.perception(), breach, true), "A stone wall.");
        w.apply(&Action::Move(Dir::E));
        for _ in 0..20 {
            if w.perception().hollow(breach) {
                break;
            }
            for action in ascmearc_core::action::parse_actions("search:100").unwrap() {
                w.apply(&action);
            }
        }
        assert_eq!(
            describe(&w.perception(), breach, true),
            "A hollow stone wall."
        );
    }

    #[test]
    fn a_composite_is_seen_by_its_gear() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.a@p.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "animated_armour_12" }
"p" = { terrain = "floor", creature = "animated_panoply_70" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "dwarf,male,warrior".parse::<ascmearc_core::character::CharacterSpec>();
        let mut said = Vec::new();
        let talents = ["sworn_foe:kobold", "healthy", "lean"]
            .map(String::from)
            .to_vec();
        let spec = spec.expect("a character").with_talents(talents);
        let w = World::start_run(content.clone(), "hall", Some(1), Some(&spec), None, true)
            .expect("hall");
        let view = w.perception();
        // The gear line names what it wears and wields, unidentified.
        said.extend([Pos::new(2, 1), Pos::new(4, 1)].map(|p| describe(&view, p, true)));
        insta::assert_snapshot!(said.join("\n"));
    }

    #[test]
    fn an_ally_is_yours() {
        let content = Arc::new(ascmearc_assets::content().expect("content"));
        let mut w = World::new(content, "showcase", None).expect("showcase");
        for action in ascmearc_core::action::parse_actions("ally:dire_warg").expect("actions") {
            w.apply(&action);
        }
        let view = w.perception();
        let ally = (view.actors().iter())
            .find(|a| a.ally)
            .expect("the warg beside the player");
        assert_eq!(
            describe(&view, ally.pos, true),
            "Your dire warg (unhurt), on stone floor."
        );
    }

    #[test]
    fn stalkers_eye_tells_a_creature_unaware_of_you() {
        // A dark hall: the dwarf sees the deer by darkvision; the deer's eyes miss them.
        let hall = r##"
name = "Hall"
ambient = [0, 0, 0, 0]
map = '''
#######
#@..d.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", creature = "deer" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "dwarf,male,ranger"
            .parse::<ascmearc_core::character::CharacterSpec>()
            .expect("a character")
            .with_talents(
                ["sworn_foe:kobold", "healthy", "lean"]
                    .map(String::from)
                    .to_vec(),
            );
        // A ranger gains the talent at level 2. A run whose deer is big (its grade is the
        // run's, `rng.rs` "Actor keys"), so the grade word shows before the talent's.
        let start = |seed: u64, actions: &str| {
            let mut w =
                World::start_run(content.clone(), "hall", Some(seed), Some(&spec), None, true)
                    .expect("starts");
            for action in ascmearc_core::action::parse_actions(actions).expect("actions") {
                w.apply(&action);
            }
            describe(&w.perception(), Pos::new(4, 1), true)
        };
        let seed = (1..200u64)
            .find(|&seed| start(seed, "").starts_with("A big deer"))
            .expect("a big deer");
        let said: Vec<String> = ["", "xp:691 levelup:auto*2"]
            .into_iter()
            .map(|actions| start(seed, actions))
            .collect();
        assert_eq!(said[0], "A big deer (unhurt), on stone floor.");
        assert_eq!(
            said[1],
            "A big deer (unhurt, unaware of you), on stone floor."
        );
    }

    #[test]
    fn the_creature_your_openings_are_in_is_your_mark() {
        // A dark hall: the dwarf thief's sling stone finds the deer unaware, a sneak attack
        // that builds an opening in its guard.
        let hall = r##"
name = "Hall"
ambient = [0, 0, 0, 0]
map = '''
#######
#@..d.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"d" = { terrain = "floor", creature = "deer" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let spec = "dwarf,male,thief"
            .parse::<ascmearc_core::character::CharacterSpec>()
            .expect("a character")
            .with_talents(
                ["sworn_foe:kobold", "healthy", "lean"]
                    .map(String::from)
                    .to_vec(),
            );
        let marked = (1..40).find_map(|seed| {
            let mut w =
                World::start_run(content.clone(), "hall", Some(seed), Some(&spec), None, true)
                    .expect("starts");
            w.apply(&"fire:4,1".parse::<Action>().expect("action"));
            let view = w.perception();
            let deer = (view.actors().iter()).find(|a| a.marked)?;
            Some((describe(&view, deer.pos, true), view.own_mark()))
        });
        let (said, mark) = marked.expect("some stone lands a sneak attack");
        // "A deer (badly wounded, bleeding, your mark), on blood-spattered stone floor; ..."
        assert!(said.contains(", your mark), on "), "{said}");
        assert_eq!(mark.as_deref(), Some("the deer"));
    }

    #[test]
    fn a_blade_is_seen_by_the_weapon_it_wields() {
        let hall = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#######
#.b@c.#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"b" = { terrain = "floor", creature = "animated_blade_8" }
"c" = { terrain = "floor", creature = "animated_blade_19" }
"##;
        let mut files = ascmearc_assets::files();
        files.insert("levels/hall.toml", hall);
        files.insert(
            "scenarios/hall.toml",
            "name = \"Hall\"\nstart_level = \"hall\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("content"));
        let w = World::new(content, "hall", None).expect("hall");
        let view = w.perception();
        // No material word: the gear line names the weapon as anyone's, unidentified.
        let said = [Pos::new(2, 1), Pos::new(4, 1)].map(|p| describe(&view, p, true));
        insta::assert_snapshot!(said.join("\n"));
    }
}
