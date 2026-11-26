//! Content's accessors for items: objects, item classes, slots, traps and chests, materials and
//! stock, recipes, uniques, parts, labels, and affixes.

use super::{
    Affixes, ChestRules, Content, Gems, ItemClassDef, ItemClassId, ItemKind, MAX_ITEM_LEVEL,
    MaterialDef, MaterialId, ObjectDef, ObjectId, Part, PartDef, PartId, Reading, RecipeDef,
    RecipeId, Recipes, ScrollLabelDef, SlotDef, SlotId, SpellId, Stock, TrapAvoid, TrapDef,
    TrapDisarm, TrapHarm, TrapId, UniqueDef, part,
};

use crate::object::Stack;

impl Content {
    pub fn object(&self, id: ObjectId) -> &ObjectDef {
        &self.objects[usize::from(id.0)]
    }

    pub fn object_id(&self, name: &str) -> Option<ObjectId> {
        self.object_ids.get(name).copied()
    }

    /// Every object type, in file order (by [`ObjectId`]).
    pub fn object_defs(&self) -> &[ObjectDef] {
        &self.objects
    }

    /// The item classes, in the order item lists group by (by [`ItemClassId`]).
    pub fn item_classes(&self) -> &[ItemClassDef] {
        &self.item_classes
    }

    pub fn item_class(&self, id: ItemClassId) -> &ItemClassDef {
        &self.item_classes[usize::from(id.0)]
    }

    pub fn item_class_id(&self, name: &str) -> Option<ItemClassId> {
        (self.item_classes.iter().position(|c| c.id == name)).map(|i| ItemClassId(i as u8))
    }

    /// The classes a loot pool's `name` stands for: the class of that id, or every class of that
    /// `group`, in list order. Empty for neither.
    pub fn pool_classes(&self, name: &str) -> Vec<ItemClassId> {
        (self.item_classes.iter().enumerate())
            .filter(|(_, c)| c.id == name || c.group.as_deref() == Some(name))
            .map(|(i, _)| ItemClassId(i as u8))
            .collect()
    }

    /// The equipment slots, in display order (by [`SlotId`]; empty without `data/slots.toml`).
    pub fn slots(&self) -> &[SlotDef] {
        &self.slots
    }

    pub fn slot(&self, id: SlotId) -> &SlotDef {
        &self.slots[usize::from(id.0)]
    }

    pub fn slot_id(&self, name: &str) -> Option<SlotId> {
        let i = self.slots.iter().position(|s| s.id == name)?;
        u8::try_from(i).ok().map(SlotId)
    }

    /// Every slot, with its id, in display order.
    pub fn slot_ids(&self) -> impl Iterator<Item = (SlotId, &SlotDef)> {
        (0u8..).zip(&self.slots).map(|(i, s)| (SlotId(i), s))
    }

    /// The slots one of class `class` has (`None`: no character, a creature), with their ids, in
    /// display order: every slot but those of another class ([`SlotDef::belongs_to`]).
    pub fn slots_for<'a>(
        &'a self,
        class: Option<&'a str>,
    ) -> impl Iterator<Item = (SlotId, &'a SlotDef)> + 'a {
        self.slot_ids().filter(move |(_, s)| s.belongs_to(class))
    }

    /// Whether `object` goes in `slot`: it fits there properly ([`Content::fits_properly`]), or
    /// the slot takes weapons (the missile weapon slot) and it is one or can be thrown.
    pub fn fits(&self, object: ObjectId, slot: SlotId) -> bool {
        let def = self.object(object);
        self.fits_properly(object, slot)
            || (self.slot(slot).takes_weapons
                && (def.can_throw() || (def.attack.is_some() && def.ammo.is_none())))
    }

    /// Whether `object` goes in `slot` by its own `slot`, not as a weapon some slot takes too:
    /// its `slot` is what the slot fits, or it is a light weapon or a shield (anything it can
    /// bash with) and the slot is a hand (two-weapon and two-shield fighting); a `light_only`
    /// slot (the tail) a light weapon alone.
    pub fn fits_properly(&self, object: ObjectId, slot: SlotId) -> bool {
        let (def, slot) = (self.object(object), self.slot(slot));
        if slot.light_only {
            return def.light_weapon && def.bash.is_none() && !def.two_handed;
        }
        def.slot.as_ref() == Some(&slot.fits)
            || ((def.light_weapon || def.bash.is_some()) && slot.hand)
    }

    /// The hand slots everyone has, the pair whose hold is the grip (a two-handed object takes
    /// them all), in display order: never a grown one ([`Content::extra_hands`]).
    pub fn hand_slots(&self) -> impl Iterator<Item = SlotId> + '_ {
        (self.slot_ids())
            .filter(|(_, s)| s.hand && !s.grown)
            .map(|(id, _)| id)
    }

    /// The grown hand slots (Three-armed's third hand, Prehensile-tailed's tail, #1808), in
    /// display order: what they hold is extra to the grip (`world/combat/extra_limbs.rs`).
    pub fn extra_hands(&self) -> impl Iterator<Item = SlotId> + '_ {
        (self.slot_ids())
            .filter(|(_, s)| s.hand && s.grown)
            .map(|(id, _)| id)
    }

    /// Whether missile weapon `launcher` shoots `ammo`: they pair by kind (`launches` and
    /// `ammo` in `data/objects.toml`).
    pub fn fires(&self, launcher: ObjectId, ammo: ObjectId) -> bool {
        let kind = &self.object(launcher).launches;
        kind.is_some() && *kind == self.object(ammo).ammo
    }

    /// The plain ammunition of `kind` (`ammo = "arrow"`): the first by object id, so "shoots
    /// arrows" names the arrow before the flaming arrow whatever order the files load in.
    pub fn ammunition(&self, kind: &str) -> Option<&ObjectDef> {
        (self.objects.iter())
            .filter(|o| o.ammo.as_deref() == Some(kind))
            .min_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn trap(&self, id: TrapId) -> &TrapDef {
        &self.traps[usize::from(id.0)]
    }

    /// The chest bands (empty without `data/chests.toml`).
    pub fn chests(&self) -> &ChestRules {
        &self.chests
    }

    pub fn trap_id(&self, name: &str) -> Option<TrapId> {
        self.trap_ids.get(name).copied()
    }

    /// Every trap type, in file order (by [`TrapId`]; none without `data/traps.toml`).
    pub fn trap_defs(&self) -> &[TrapDef] {
        &self.traps
    }

    /// The chance of stepping onto a known trap without setting it off (`[avoid]` in
    /// `data/traps.toml`, else [`TrapAvoid::default`]).
    pub fn trap_avoid(&self) -> &TrapAvoid {
        &self.trap_avoid
    }

    /// Disarming a known trap (`[disarm]` in `data/traps.toml`, else
    /// [`TrapDisarm::default`]).
    pub fn trap_disarm(&self) -> &TrapDisarm {
        &self.trap_disarm
    }

    /// What a trap's damage may be cut by (`[harm]` in `data/traps.toml`, else
    /// [`TrapHarm::default`]).
    pub fn trap_harm(&self) -> &TrapHarm {
        &self.trap_harm
    }

    /// Whether `object` is a knife, to fletch with (`data/fletch.toml`'s `knives`, a weapon
    /// skill: "Daggers and knives", so a dagger, a stiletto, or a knife; a short sword is not).
    pub fn is_knife(&self, object: ObjectId) -> bool {
        let knives = self.fletch().map(|f| f.knives);
        knives.is_some() && self.object(object).skill == knives
    }

    pub fn material(&self, id: MaterialId) -> &MaterialDef {
        &self.materials[usize::from(id.0)]
    }

    pub fn material_id(&self, name: &str) -> Option<MaterialId> {
        self.material_ids.get(name).copied()
    }

    /// Every material, in file order (by [`MaterialId`]).
    pub fn materials(&self) -> &[MaterialDef] {
        &self.materials
    }

    /// Gem sizes, cuts, and ornaments (`data/gems.toml`); `None` without the
    /// file: no gems are found or set.
    pub fn gems(&self) -> Option<&Gems> {
        self.gems.as_ref()
    }

    /// Material stock's form words and professions (`data/stock.toml`; docs/dev/objects.md
    /// "Stock"); empty without the file.
    pub fn stock(&self) -> &Stock {
        &self.stock
    }

    /// The professions' recipes (`data/recipes/<skill>/`; docs/dev/crafting.md), in path order.
    pub fn recipes(&self) -> &Recipes {
        &self.recipes
    }

    pub fn recipe(&self, id: RecipeId) -> &RecipeDef {
        &self.recipes.list[usize::from(id.0)]
    }

    /// The unique items, in table order.
    pub fn uniques(&self) -> impl Iterator<Item = &UniqueDef> {
        self.uniques.iter()
    }

    /// Unique item `id`.
    pub fn unique(&self, id: &str) -> Option<&UniqueDef> {
        self.uniques.get(id)
    }

    /// The kind `stack`'s affixes work as: its base's, or for a unique on a base of none
    /// (clothing, #2141) the kind its table names ([`UniqueDef::kind`]).
    pub fn item_kind(&self, stack: &Stack) -> Option<ItemKind> {
        let own = self.object(stack.object).kind;
        own.or_else(|| self.unique(stack.magic()?.unique.as_deref()?)?.kind)
    }

    /// Unique item `id` as a placement makes it (`unique:ID`): refused for an unknown id, and
    /// for a second placement anywhere in the content.
    pub(super) fn place_unique(&self, id: &str) -> Result<crate::object::Stack, String> {
        self.uniques.place(id)
    }

    /// The part names of `data/parts.toml`, in file order (by [`PartId`]).
    pub fn part_defs(&self) -> &[PartDef] {
        &self.parts
    }

    pub fn part_id(&self, name: &str) -> Option<PartId> {
        let i = self.parts.iter().position(|p| p.id == name)?;
        Some(PartId(u8::try_from(i).ok()?))
    }

    /// `parts` in words, every part, the main one first: "steel blade, leather grip", "iron".
    pub fn describe_parts(&self, parts: &[Part]) -> String {
        part::phrase(parts, &self.materials, &self.parts)
    }

    /// Unidentified scrolls' labels (`data/scroll_labels.toml`), if scrolls have them.
    pub fn scroll_labels(&self) -> Option<&ScrollLabelDef> {
        self.scroll_labels.as_ref()
    }

    /// Unidentified wands' labels (`data/wand_labels.toml`), if wands have them.
    pub fn wand_labels(&self) -> Option<&ScrollLabelDef> {
        self.wand_labels.as_ref()
    }

    /// Unidentified rods' labels (`data/rod_labels.toml`), if rods have them.
    pub fn rod_labels(&self) -> Option<&ScrollLabelDef> {
        self.rod_labels.as_ref()
    }

    /// Unidentified potions' labels (`data/potion_labels.toml`), if potions have them.
    pub fn potion_labels(&self) -> Option<&ScrollLabelDef> {
        self.potion_labels.as_ref()
    }

    /// The scroll of `spell`: the first object, by id, whose `read` casts it
    /// (`scroll_of_ID`; every shipped spell has one). What the wizard's Scribe writes.
    pub fn scroll_of(&self, spell: SpellId) -> Option<ObjectId> {
        let i = (self.objects.iter()).position(
            |o| matches!(&o.read, Some(Reading::Cast(scroll)) if scroll.spell == spell),
        )?;
        Some(ObjectId(u16::try_from(i).ok()?))
    }

    /// The blank scroll (`blank_scroll`): what a scroll is written on, and
    /// nothing to read.
    pub fn blank_scroll(&self) -> Option<ObjectId> {
        self.object_id("blank_scroll")
    }

    /// Prefixes and suffixes, and how loot rolls them; empty without
    /// `data/affixes.toml`.
    pub fn affixes(&self) -> &Affixes {
        &self.affixes
    }

    /// The item level of loot on level `id`: a dungeon branch level's danger
    /// ([`Content::floor_danger`], its fixed part where it grows with the player; still rising on
    /// an endless branch; #1850); a hand-made level's `item_level` (default 1); else 1. Never the
    /// depth (#2067): every branch floor has a danger (`content/dungeon.rs` refuses one without),
    /// and depth reaches loot only through it (`per_depth`). Kept within 1 to [`MAX_ITEM_LEVEL`].
    pub fn item_level(&self, id: &str) -> u8 {
        if let Some(template) = self.levels.get(id) {
            return template.item_level;
        }
        u8::try_from(self.floor_danger(id, 0))
            .unwrap_or(MAX_ITEM_LEVEL)
            .clamp(1, MAX_ITEM_LEVEL)
    }
}
