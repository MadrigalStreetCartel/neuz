# Neuz-Dev
> Flyff Universe client with enhanced features.

**This is a private development repository. Read the [LICENSE](./LICENSE.md)!**

Binaries are downloadable from here: [MSI](https://github.com/MadrigalStreetCartel/neuz-dev/tree/main/src-tauri/target/release/bundle/msi) | [DEB](https://github.com/MadrigalStreetCartel/neuz-dev/tree/main/src-tauri/target/release/bundle/deb) | [AppImage](https://github.com/MadrigalStreetCartel/neuz-dev/tree/main/src-tauri/target/release/bundle/appimage)

## Contributing

1. Fork the repository
2. Use a descriptive branch name (example: `feat/add-auto-loot`)
3. Implement your changes and test them properly
4. Run `cargo fmt` to format your code according to the Rust standard
5. Open a pull request and describe your changes in detail

## Changelog
> This is an internal changelog. DO NOT REPRODUCE.

**0.10.0**
- Adds HP detection and HP-based food consumption
- Fixes issue with mobs not getting detected

**0.9.0**
- Fixes crash related to message pumping on windows
- Vastly improves killed mob avoidance

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

