# Changelog
Date format: `DD/MM/YYYY`

<!-- maintainers -->
[@genoxalmighty]: https://github.com/genoxalmighty
[@slyker]: https://github.com/slyker
<!-- /maintainers -->
**0.13.0**
> Released on XX.09.2022

- image detection
  - Improves mob detection and attack by detecting cursor
  - Changes min & max name's length detection (Mias, Bang, Master Mage Prankster  and others were not detected)
  - Adds PNJ/FP/MP detection
  - IGNORE_AREA_BOTTOM decreased to 110px instead of 150px
- UI enhancements
  - Removes settings 'Use skill to attack', 'Use pickup pet' (functionnalities still implemented)
  - Displays stats
  - Slot
    - Adds the F1 to F9 slot bars (90 slots availible) - use shortcut like in game to navigate
    - Reworked to fit with new threshold and cooldown restoration
    - Can be disabled instead of removed
- Changes rotation keys so only the camera rotate
- Removes unsupervised mode
- Farming behavior
  - Avoid obstacles (experimental) you'll need at least one attack slot to make it works (advice: add attack motion with a low cooldown like 500)
  - Avoid already attacked monster (disable if you play in party)
  - Avoid npc/pets selection (rare)
  - New mob search movements pattern, moves in circle
  - PickupPet waiting time is now non blocking and can be changed
  - Adds stop mobs detection feature, it disable the mob detection but keeps slots usage
  - Adds a minimum HP % to attack only for passive (will wait until this value is reached)
- AutoShout behaviour
  - Fixes an AutoShout behavior bug where spaces were automatically deleted
- Misc
  - Adds buffs support !
  - Add HP/FP/MP restoration based on threshold and cooldown
  - Removes games news on launcher
  - Adds version + update display
  - Debug options (click 3 times on stats to show, click again to hide)
    - Adds customisable mobs detection values
    - Customisable obstacle avoidance values and behavior
    - Reset slot bars button
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
