# Neuz
> Open Source Flyff Universe client with enhanced features.

We are NOT affiliated with Gala Lab Corp., Sniegu Technologies SAS or Flyff Universe.

Binaries are downloadable from here: [MSI Builds](https://github.com/SplittyDev/Neuz/tree/main/src-tauri/target/release/bundle/msi) | [DEB Builds](https://github.com/SplittyDev/Neuz/tree/main/src-tauri/target/release/bundle/deb) | [AppImage Builds](https://github.com/SplittyDev/Neuz/tree/main/src-tauri/target/release/bundle/appimage)

## Semi Autonomous Mode
> Right now the bot operates in semi-autonomous mode. Keep an eye on it.

**Requirements**
> No settings have to be changed by default.

1. Use default theme (used by default)
2. Enable auto-attack (enabled by default)

For best compatibility:

1. Enable right-click player menu for best compatibility (disabled by default)
2. Disable single-click to attack (disabled by default, not necessarily needed)

## Changelog

**0.8.0**
- Adds the foundation for modes of operation (Farming, Support, AutoShout)
- Updates launcher UI

**0.7.0**
- Adds support for Linux (incomplete)
- Fixes long standing bug with violet magician avoidance
- Switches to our own [libscreenshot](https://github.com/madrigalstreetcartel/libscreenshot) for huge performance gains
- Replaces copyrighted logo with MadrigalStreetCapital logo
- Improves launcher UI

**0.6.0**
- Adds option to stay in area in order to avoid giants etc.
- Tweaks algorithms for improved hit detection
- Extends dead-zone at bottom of screen

**0.5.0**
- Adds automatic saving/loading of bot configuration
- Adds configurable slots for foods, pets, skills and buffs
- Adds on-demand pickup-pet handling to prevent pet from interfering with algorithm
- Adds more configurable options to bot UI
- Further improves movement patterns and responsiveness
- Greatly improves detection performance by using a multi-threaded algorithm
- Relaunches application when client window is closed to prevent desync issues

**0.4.0**
- Adds patch notes and events to launcher
- Improves dead mob avoidance
- Improves initial movement pattern when no mob is found
- Improves and extends movement patterns to make the bot seem more life-like 

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

