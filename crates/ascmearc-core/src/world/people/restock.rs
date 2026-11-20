//! Shops restocking as the player gains levels (#1735; the numbers are
//! `data/rules/restock.toml`, `content/restock.rs`; a town's bonuses its `[town.restock]`,
//! `content/town.rs`).
//!
//! - **When** ([`World::restock_shops`], one line in `experience.rs::rise_from` and one after each
//!   level is first made in `travel.rs`): every shop on every level made so far catches up to the
//!   player's level, one restock for each level from the one after its `stocked` (or 2) to the
//!   player's, then `stocked` is the player's level. Every level made is kept in the world state,
//!   so a level-up restocks the shops of levels the player is away from at once, with the
//!   bonuses of that moment; a level made later catches up as it is made. A level lost and gained
//!   again restocks nothing: each level counts once. Only a shop whose keeper trades restocks.
//! - **A restock** ([`World::restock`]): from its own stream, `restock/<level>/<shop>/<player
//!   level>`, so what comes does not depend on when or whether the player looks: `gone` stacks
//!   of goods lying on the shop floor (what the shop trades, never its kind's `always`) leave,
//!   chosen evenly; then the kind's `always` objects missing from the floor come back, and `come`
//!   stacks more are drawn from what the kind stocks (`Shops::goods`), each on a free cell of the
//!   shop floor (plain ground, nothing lying there, no trap or fire, not the keeper's
//!   post; who stands where does not count, so goods may come to lie under a creature), made at
//!   the restock's item level (`Shops::lay_out`).
//! - **The item level** ([`crate::content::RestockRules::item_level`]): the player's level, the
//!   level's own item level for a shop in no town (`content.item_level`: a floor's depth), and the
//!   town's bonus ([`World::restock_bonus`]).
//! - **The talk** ([`World::economy_lines`], from `talk.rs::one_liner`): in a town, a speaker's
//!   `[dialogue.economy]` `better` lines while its bonus is above 0, else its `hard` ones.

use crate::content::{Content, Dialogue, DialogueLine, TownDef};
use crate::geom::Pos;
use crate::magic::loot::pick_weighted;
use crate::rng::Rng;
use crate::world::World;

impl World {
    /// What town `town` adds to its shops' restocking now, in hundredths of an item level: each
    /// quest its `[town.restock]` names that is done, and its share of a merchant's stake's bonus
    /// once its `after` quest is done.
    pub(crate) fn restock_bonus(&self, town: &TownDef) -> u32 {
        let Some(restock) = &town.restock else {
            return 0;
        };
        let done =
            |quest: &str| (self.state.quests.records.get(quest)).is_some_and(|r| r.completed > 0);
        let quests: u32 = (restock.quests.iter())
            .filter(|(q, _)| done(q))
            .map(|(_, &b)| b)
            .sum();
        let invest = match (&restock.invest, self.content.restock()) {
            (Some(share), Some(rules)) if share.after.as_deref().is_none_or(done) => {
                rules.invest_bonus(self.state.fund.gold()) * share.share / 100
            }
            _ => 0,
        };
        quests.saturating_add(invest)
    }

    /// Every shop's goods catch up to the player's level (the module docs).
    pub(in crate::world) fn restock_shops(&mut self) {
        let content = self.content.clone();
        let Some(rules) = content.restock() else {
            return;
        };
        let player = self.player_level();
        if player <= 1 {
            return;
        }
        let ids: Vec<String> = self.state.levels.keys().cloned().collect();
        for id in ids {
            let town = content.town_at(&id);
            let bonus = town.map_or(0, |t| self.restock_bonus(t));
            let danger = if town.is_some() {
                0
            } else {
                content.item_level(&id)
            };
            for i in 0..self.state.levels[&id].shops.len() {
                let shop = &self.state.levels[&id].shops[i];
                let from = shop.stocked.max(1);
                if from >= player {
                    continue;
                }
                if shop.is_open() {
                    for level in from + 1..=player {
                        let made_at = rules.item_level(level, danger, bonus);
                        self.restock(&content, (&id, i), level, made_at);
                    }
                }
                if let Some(shop) =
                    (self.state.levels.get_mut(&id)).and_then(|l| l.shops.get_mut(i))
                {
                    shop.stocked = player;
                }
            }
        }
    }

    /// Shop `i` of level `id` restocks for the player's level `level`, its new goods made at
    /// item level `made_at` (the module docs).
    fn restock(&mut self, content: &Content, (id, i): (&str, usize), level: u8, made_at: u8) {
        let (Some(shops), Some(rules)) = (content.shops(), content.restock()) else {
            return;
        };
        let mut rng = Rng::stream(self.state.seed, &format!("restock/{id}/{i}/{level}"));
        let now = self.state.time;
        let Some(state) = self.state.levels.get_mut(id) else {
            return;
        };
        let shop = state.shops[i].clone();
        let def = shops.get(shop.kind);
        // What goes: goods lying on the floor, by cell and then top first.
        let mut lying: Vec<(Pos, usize)> = (state.objects.iter())
            .filter(|(p, _)| shop.contains(**p))
            .flat_map(|(&p, pile)| {
                (pile.iter().enumerate())
                    .filter(|(_, s)| shops.trades(content, s) && !def.always.contains(&s.object))
                    .map(move |(k, _)| (p, k))
            })
            .collect();
        let gone = rng.range(i32::from(rules.gone.0), i32::from(rules.gone.1));
        let mut chosen: Vec<(Pos, usize)> = Vec::new();
        for _ in 0..usize::try_from(gone).unwrap_or(0).min(lying.len()) {
            chosen.push(lying.remove(rng.index(lying.len())));
        }
        for (p, _) in &chosen {
            if let Some(pile) = state.objects.get_mut(p) {
                let mut k = 0;
                pile.retain(|_| {
                    k += 1;
                    !chosen.contains(&(*p, k - 1))
                });
            }
        }
        state.objects.retain(|_, pile| !pile.is_empty());
        // What comes: what the kind always has and is missing, then goods drawn.
        let on_floor = |o| {
            (state.objects.iter())
                .any(|(p, pile)| shop.contains(*p) && pile.iter().any(|s| s.object == o))
        };
        let missing: Vec<_> = (def.always.iter().copied())
            .filter(|&o| !on_floor(o))
            .collect();
        let mut free: Vec<Pos> = (shop.min.y..=shop.max.y)
            .flat_map(|y| (shop.min.x..=shop.max.x).map(move |x| Pos::new(x, y)))
            .filter(|&p| {
                p != shop.post
                    && (state.terrain.get(p)).is_some_and(|&t| content.terrain(t).is_plain_ground())
                    && !state.objects.contains_key(&p)
                    && !state.traps.contains_key(&p)
                    && !state.fires.contains_key(&p)
            })
            .collect();
        rng.shuffle(&mut free);
        let come = rng.range(i32::from(rules.come.0), i32::from(rules.come.1));
        let goods = shops.goods(content, shop.kind);
        let mut cells = free.into_iter();
        let drawn = (0..usize::try_from(come).unwrap_or(0)).map(|_| None);
        for always in missing.into_iter().map(Some).chain(drawn) {
            let Some(p) = cells.next() else {
                break;
            };
            let object = match always {
                Some(object) => object,
                None => match pick_weighted(&mut rng, &goods) {
                    Some(&object) => object,
                    None => break,
                },
            };
            let mut stack = shops.lay_out(content, &mut rng, object, made_at);
            crate::rot::made_at([&mut stack], content, now);
            state.objects.insert(p, vec![stack]);
        }
    }

    /// The one-liners about the town's trade `d` adds in the town the player is in (the module
    /// docs); none outside a town.
    pub(in crate::world) fn economy_lines<'d>(&self, d: &'d Dialogue) -> &'d [DialogueLine] {
        let (Some(economy), Some(town)) =
            (&d.economy, self.content.town_at(&self.state.current_level))
        else {
            return &[];
        };
        match self.restock_bonus(town) > 0 {
            true => &economy.better,
            false => &economy.hard,
        }
    }
}
