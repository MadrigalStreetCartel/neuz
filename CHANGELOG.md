# Changelog
Date format: `DD/MM/YYYY`

<!-- maintainers -->
[@genoxalmighty]: https://github.com/genoxalmighty
[@slyker]: https://github.com/slyker
<!-- /maintainers -->
**0.13.0**
> Released on XX.09.2022

- Adds buffs support !
- Improves image detection & target selection :
  - Improves mob detection and attack by detecting cursor
  - Changes minimum name's length detection (Mias and others were not detected)
  - Adds custom mobs detection values
  - Adds pnj detection and 'avoidance' (basically press escape)
- Adds stop mobs detection feature
- Add HP/FP/MP restoration based on threshold and cooldown
- UI enhancements :
  - Updated slots selection
  - Removes useless settings 'Use skill to attack', 'Use pickup pet' (functionnality's still implemented)
- Changes rotation values so only the camera rotate
- Removes unsupervised mode since the bot will now be more autonomous
- Farming behavior update :
  - Avoid obstacles (experimental) you'll need at least one attack slot to make it works (advice: add attack motion with a low cooldown like 500)
  - Avoid already attacked monster (disable if you play in party)
  - Avoid npc/pets selection (rare)
  - New mob search movements, moves in circle within the current area
  - PickupPet waiting time is now non blocking and can be changed
- Fixes an AutoShout behavior bug where spaces were automatically deleted
- Lot of various fixes
Big thanks to Moe who helped a lot to make this release perfect !

**0.12.1**
> Released on 23.08.2022

**INFO**:<br>
This is the last release supported by the founders.<br>
Project lead has been given to [@slyker].

- Fixes occasional clicks outside of window area
- Updates default window layout

**0.12.1**
> Released on 23.08.2022

**INFO**:<br>
This is the last release supported by the founders.<br>
Project lead has been given to [@slyker].

- Fixes occasional clicks outside of window area
- Updates default window layout

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
