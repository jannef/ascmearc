# Ascmearc design — 8. Architecture, time, determinism

Part of the design document; the index is [`specification.md`](../specification.md).

## Architecture

- The core is plain Rust: no engine, no platform I/O in the rules, no wall-clock time. It takes its content as input.
- The app does rendering, input, UI, effects, and platform work. The ECS lives only there.
- A headless CLI on top of the core scripts, dumps, and verifies runs.
- Presentation reads only the per-turn perception snapshot. Hidden information cannot leak by construction (a developer reveal mode excepted).
- A turn runs: the player acts; the other actors act in time order until the player may act again; the environment updates; perception; presentation events.
- Gameplay never waits for animations, and never depends on frame rate, animation, or visual randomness.
- Keep modules small with clear responsibilities. No speculative abstractions, no premature optimization.

## Time and speed

- World time is whole seconds on a calendar of 12 months of 30 days. All arithmetic is integer.
- Every action has an energy cost in data; 1000 is normal and takes 6 s at normal speed.
- An overworld step instead takes the terrain's travel time.
- The actor's speed scales every action's time. Burden, Bloated, and statuses raise the price of every action in one place.
- An attack is priced as it begins: what it changes affects only the next attack.
- An energy scheduler runs everyone. Leftover energy carries over, so action counts stay exactly proportional to speed and cost, with no drift.
- Ties go to the player first, then by creation order.
- Creatures on other levels are frozen: returning sets off no burst of actions.
- Speed is hidden from the player everywhere.

## Rest

- Rest (an hour, until dawn, until dusk; at most a day) passes turns through the normal pipeline and ends exactly at its goal.
- A rest stops for: a hostile coming close in view (the owner: 5 steps; neutrals and allies never stop it), an attack, a missile or spell nearby, a message that is news, or death.
- Resting heals twice as fast.
- Skipping perception or coarsening far creatures' turns during a rest is an optimization. It never changes when the rest stops or what happens.
- The developer time skip passes no turn, but it is recorded.

## Dice and events

- Dice in data are `NdS±K` text or a fixed number. A damage roll may sum dice of several sizes.
- The core emits events. The app turns them into motion, particles, missiles, and spell effects.

## Determinism

- The same seed and actions give the same result within a game version. Seeds need not be stable across versions.
- Every random draw comes from a stream keyed by the seed and a label. There is no shared stream: adding or removing one draw moves no other.
- A level depends only on the seed and its id, never on the order levels are visited.
- Each optional layer of generation (traps, loot, chests, gear) draws from its own stream, so the rest of the level is the same with or without it.
- No hash-order iteration and no floating point in core decisions.
- Visual randomness in the app never touches the rules' streams.
- A seed is a number or a word.
