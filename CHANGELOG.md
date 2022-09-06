# Changelog
Date format: `DD/MM/YYYY`

<!-- maintainers -->
[@genoxalmighty]: https://github.com/genoxalmighty
[@slyker]: https://github.com/slyker
<!-- /maintainers -->
**0.13.0**
> Released on XX.XX.2022

- Improves image detection :
  - Improves mob detection and attack by detecting cursor
  - Changes minimum name's length detection (Mias and others were not detected)
  - Add custom mobs detection values. Default values Passive: R234 G234 B149 Tolerence: 2 / Aggressive: R179 G23 B23 Tolerence: 9
- Adds stop mobs detection feature
- Add HP/FP/MP restoration based on threshold and cooldown
- UI enhancements :
  - Updated slots selection
- Changes rotation values so only the camera rotate
- Removes useless settings 'Use skill to attack', 'Use pickup pet'
- Adds a feature to avoid obstacles (experimental) it'll only works if there's an attack slot to use (advice: add attack motion with a low cooldown)
- Adds a feature to avoid already attacked monster (disable if you play in party)

**0.12.0**
> Released on 24.07.2022

- Adds experimental unsupervised mode
- Adds pickup support without pickup pet
- Improves user interface
  - Includes updated slot configurator with labels
  - Includes better user interface interactions
  - Adds footer with quick access to Discord and GitHub
- Improves movement backend and code architecture

**0.11.0**
> Released on 20.07.2022

- Adds support for pill usage on critical health ([@genoxalmighty])
- Adds auto-shout mode ([@slyker])

**0.10.1**
> Released on 18.07.2022

- Adds privacy-friendly error logging to Sentry

**0.10.0**
> Released on 17.07.2022

- Adds HP detection and HP-based food consumption
- Fixes issue with mobs not getting detected

**0.9.0**
> Released on 04.07.2022

- Fixes crash related to message pumping on windows
- Vastly improves killed mob avoidance

**0.8.0**
> Released on 03.07.2022

- Adds the foundation for modes of operation (Farming, Support, AutoShout)
- Updates launcher UI

## Unreleased
> These changes took place before the repository went public.

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
