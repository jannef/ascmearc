# Ascmearc design — 5. Input and interface

Part of the design document; the index is [`specification.md`](../specification.md).

The game is played from the keyboard, ADOM-style. Every screen tells the player what their character knows, and nothing more.

## Keys

- Keyboard first. Default keys are ADOM's: an ADOM command takes ADOM's key. A command ADOM lacks, or keeps behind a prefix, takes a key ADOM leaves free.
- Movement is the numpad and the arrows. The vi keys are never bound in play by default.
- Only play commands are rebindable. The keys of menus, menu lists, screens, info pages, prompts, dialogs, text fields, and aiming are fixed. The owner: "do not fucking make menus rebindable. that just gets too messy"; "at least menu lists like that. it just won't work".
- Esc is fixed. It always cancels a prompt, backs out of a screen, or opens the pause menu. No binding, settings file, or keymap can lock the player out.
- Letters and digits on a list always name rows, never commands.
- Rebinding a key that another command in the same context uses moves it, and the screen says so. The table never holds a conflict.
- Play keys may carry Ctrl, Alt, Super, or Shift. Matching is exact. Shortcuts the OS or browser keeps are refused.
- Keymaps export to and import from a hand-editable file. An import changes only the commands it lists.
- The `?` help is generated from the bindings, so it is always right. Hints name the play keys as bound.

## Abilities

- Everything a character can do because of who they are (race, birth sign, class, talent, skill, weapon skill, style, item, place) is in one abilities menu, not a key each.
- Basic roguelike commands keep their own keys. Digging is using a tool, not an ability.
- An ability may have a key of its own for players who want one. It is unbound by default.
- The menu says why an ability cannot be used now. It opens on the last ability used, and one key repeats it.

## Screens and prompts

- Screens are modal and stacked. Esc returns to the screen below, as it was left.
- While a screen is open no game time passes, except for what is chosen on it (pick up, drop, wear, eat), which takes its normal time.
- Direction prompts, look mode, aiming, rest, and `--more--` are input modes, not screens.
- One parameterized dialog asks every question: a title, the text, and options with roles. The safe option is selected first, so Enter alone is never regretted. Held-key repeats never open or answer a dialog.
- The interface asks before a risky step: onto a known trap, into deep water, into a feature that offers an action, out of a shop while owing, on while fainting, or a burst read so close it may catch the reader. The rules never ask; they do what they are told.
- An action on a cell beside the player (open, chat, dig, a directional ability) counts the targets the player knows of there. With none, the rules refuse it at once, taking no time. With one, it acts on it. With several, it asks the direction. Kick always asks.
- Every way to quit asks first. Closing the window saves without asking.
- The info pages (Character, Inventory, Equipment, Skills, Weapons, Magic, Quests, Talents, Story) are switched by number keys and Tab, each replacing the last. They share one frame.
- The level-up screen opens by itself and cannot be left until its choices are made.
- The quest log is read-only. Quests are never abandoned.
- The death screen replaces every other screen. The save is deleted at once.

## Menus and runs

- The main menu offers Continue (the last character played), Characters, Graveyard (once anyone has died), New run, Settings, Controls, and Quit.
- The pause menu offers Resume, Settings, Controls, Save and quit to menu, and Quit (which saves). It carries one line of prose on the world: the date, the moon, the weather under the open sky, and the temperature.
- A new run's seed is drawn fresh. Only developer builds show it or let it be typed.
- Character creation goes race, birth-sign reveal, gender, class, talents, name, summary. Every step has a Random row, and Esc steps back with earlier choices kept.

## What is shown

- Every screen reads the perception snapshot, never ground truth (a developer reveal excepted).
- Hidden numbers stay hidden: no modifiers, potentials, speed, chances, or a creature's to-hit or hit points. Effects are told in words.
- The owner's exception: "player just needs to know how much +hit and damage, dv and pv equipment has". Item lists, descriptions, and the equipment screen give an item's own to-hit, damage, range, Defence, and Protection.
- The character screen shows the player's own standing attacks (to-hit, damage, range) and their Defence and Protection as totals.
- The level-up skill-roll step shows the dice each roll throws.
- A spell's row shows its roll, damage type, and mana cost.
- An unidentified item's magic counts for nothing in shown numbers until it is identified.
- Item lists are one paged list. Stacks are grouped by item class, at most 26 to a page, with letters restarting on each page. There is no scrolling.
- Actions name stacks by position, never by a page's letter.

## Status bar

- The owner: "status bar should really not convey that much information. It all should naturally be from player PV... but just don't put useless stuff there. Just basically a hud to track character status."
- It shows only the character's status, from the perception snapshot: where they are, hit points, mana, resources, status effects, hunger, burden, level, stance, kept beasts, and the time of day.
- No dots between the segments.
- The turn, position, light, vision, and seed are developer segments, never in a release.
- When space runs out, the least important segment gives way first. Being Hungry or worse is kept to the last.
- The character screen lists every condition the bar may have given up, so the two never disagree.

## Messages

- Messages never carry a number.
- An attack is reported only if the player perceives it. An end they do not see is "something".
- A plural noun takes a plural verb: "The flames burn the goblin.", "Your shards glance off its hide."
- The same message repeated folds into one with a count.
- When a turn's messages overflow, `--more--` pages them; what was only noted during a long action, without stopping it (chatter, footsteps), is passed over, kept in the log. It is presentation only: the rules, replays, and dumps never see pages.
- Long actions (rest, dig, run, a meal, a long search) stop for whatever would stop a rest.

## Language

- The game is shown in the language the player chooses in Settings, English by default, never guessed from the system. Switching takes effect at once, mid-run included: what was already said reads in the new language too.
- Every word the player reads is the language's data, never English written in code. What the player typed (a name), made-up words (a scroll's label), and notation (dice, signed numbers) read alike in every language.
- The log holds what was said, not its wording: a word changed in a language never changes a run, a save, or a replay.
- The narrator uses the language's full grammar. Creatures' speech uses none: it is whole lines, chosen by gender and number, written in the speaker's register (a dialect inside the language, by race or by the speaker's own).
- A line a language lacks is read in English. English is complete; Finnish is being translated.
- A language is data, so a mod can add one.
- The wiki is English only.
- How it is built: `docs/dev/localization.md`; the design and what is still to come: `docs/design/localization.md`.
