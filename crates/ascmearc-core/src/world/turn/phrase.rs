//! Who a message speaks of, as the player perceives them (`docs/dev/localization.md`
//! "Phrases"): the player ([`World::you`]) and any actor ([`World::phrase_of`]), built at the
//! `say` site by value, so a line reads the same after the actor is gone.

use crate::lang::{Gender, Person, Phrase};
use crate::world::{ActorId, World};

impl World {
    /// The player: their name, said "you" (`Person::You`), with their gender.
    pub fn you(&self) -> Phrase {
        let (name, gender) = match self.character() {
            Some(c) => (c.name.as_str(), c.spec.gender.spoken()),
            None => ("", Gender::Other),
        };
        Phrase::literal(name).person(Person::You).gender(gender)
    }

    /// Actor `id` as the player knows it when they see it: the player ([`World::you`]), their
    /// companion by its name ("Snel"), one of their allies as theirs ("your wolf"), anyone else
    /// by its creature type ("the kobold", a unique's proper name "Black Wystan"). An actor gone
    /// from the level is "something".
    pub fn phrase_of(&self, id: ActorId) -> Phrase {
        let player = self.state.player;
        if id == player {
            return self.you();
        }
        let Some(actor) = self.actor_or_fallen(id) else {
            return something();
        };
        let creature = Phrase::id(format!(
            "creature:{}",
            self.content.creature(actor.creature).id
        ));
        if actor.master() != Some(player) {
            return creature;
        }
        match actor.kept() {
            Some(data) => Phrase::literal(&data.name),
            None => creature.owned_by(self.you()),
        }
    }
}

/// What the player cannot see: "something", said without an article.
pub(in crate::world) fn something() -> Phrase {
    Phrase::id("word:something").proper(true)
}

#[cfg(test)]
mod tests {
    use crate::lang::{Msg, Phrase};
    use crate::msg;
    use crate::test_support::{english_of, world};
    use crate::world::ActorId;

    #[test]
    fn the_player_and_others_as_a_line_names_them() {
        let w = world("showcase");
        let content = w.content().clone();
        let line = |who: Phrase| {
            let msg = Msg::new(msg::combat::DRIVEN_BACK).role("who", who);
            english_of(&content, &msg)
        };
        let player = w.state.player;
        assert_eq!(line(w.you()), "You are driven back.");
        assert_eq!(line(w.phrase_of(player)), "You are driven back.");
        let other = (w.state.actors.values())
            .find(|a| a.id != player && a.master().is_none())
            .expect("someone else");
        let name = &content.creature(other.creature).name;
        assert_eq!(
            line(w.phrase_of(other.id)),
            format!("{} is driven back.", crate::content::the_start(name))
        );
        assert_eq!(
            line(w.phrase_of(ActorId(u32::MAX))),
            "Something is driven back."
        );
    }
}
