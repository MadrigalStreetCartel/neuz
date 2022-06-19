# Neuz
> Flyff Universe Client and Bot

Binaries are downloadable from here: [MSI Builds](https://github.com/SplittyDev/Neuz/tree/main/src-tauri/target/release/bundle/msi)

## Bot Usage

**Requirements**

1. Use default theme
2. Enable single-click to attack in options
3. Put food item of your choice into slot 1

## Changelog

**0.4.0**
- Adds patch notes and events to launcher
- Improve dead mob avoidance
- Improve initial movement pattern when no mob is found
- Improve and extend movement patterns to make the bot seem more life-like 

**0.3.0**
- Avoids members of the Violet Magician Troupe
- Ignores small targets (like buffs and UI icons)
- Tries to avoid attacking an enemy that has already been killed

**0.2.0**
- Improves user interface
- Improves retargeting behavior when target marker is gone
- Adds automatic foodie consumption

**0.1.0**
- Initial release

## Ideas

- AFK Protection Mode: Attack aggro mobs close to player, but do not actively search for mobs

## Mechanics

**Attack Logic**

1. Detects targets (passive and aggro)
2. Finds closest target, preferring aggros even if they're further away
3. Clicks target to initiate attack
4. Detects target marker to confirm attack status
5. Keeps attacking till the target marker is gone (enemy or player is dead)
6. Back to step 1

If no enemy is in range:

1. Initiates one of multiple predefined movement patterns
2. Checks for targets again after movement pattern is complete

If the enemy is killed:

1. Initiates short idle pattern with a certain probability
2. Checks for targets again after idle pattern is complete
    - Tries to avoid attacking an enemy that has already been killed

**Regeneration Logic**

1. Foodie is consumed every 5 seconds after an attack started
2. New attacks reset the counter, so no reg between attacks

Notes:

- The algorithm assumes that it's easy enough to kill the enemy if it takes less than 5 seconds
- In case the attack takes longer, it assumes that consuming one food item every 5 seconds is enough

