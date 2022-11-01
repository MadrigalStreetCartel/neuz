![banner]

- [Download](#download)
- [Community](#community)
- [Usage](#usage)
  - [Engagement Behavior](#engagement-behavior)
  - [Farming Automation](#farming-automation)
    - [Requirements](#requirements)
    - [Slot Configuration](#slot-configuration)
  - [Support Automation / AutoShout](#support-automation--autoshout)
- [FAQ](#faq)

# Download

- Latest version: [Download for Windows][download_msi]
- Older versions: [Release Archive](./releases)

Take a look at the [changelog][changelog]!

# Community
**v0.12.1 is the last maintained and published release from the initial founders. Further versions are solely community developed**

Planned Features for now (as of v0.12.1):
- FS mode for assist/rm with auto-buff, auto-heal, auto-follow and auto-mount
- Giant detection and avoidance
- Scripting support (realized through a DSL) for custom movement and other behaviors
- Full Mac & Linux cross-platform support
- Automatic deployment script with Docker containers for easy parallelism

Join our Discord: https://discord.gg/cZr3X3mCnq

# Usage

1. Start Neuz
2. Press `Play`
3. Select a playstyle
4. Adjust settings to your liking
5. Press `ENGAGE`

## Engagement Behavior

- Automation will take over your cursor as soon as the game window is focused.
- Unfocusing the game window (e.g. <kbd>ALT+TAB</kbd>) will pause the automation until the game window is focused again.
- Pressing `DISENGAGE` will fully stop the automation.

## Farming Automation

Use farming automation if you're trying to level up your character or farm sets, quest items, penya, etc.

Works best if you're in a densely populated farming area.

### Requirements

No settings have to be changed by default.

1. Use default theme (used by default)
2. Enable auto-attack (enabled by default)

For optimal performance (optional and not necessarily needed):

1. Disable weather effects
3. Press <kbd>ESC</kbd> a few times before engaging to clear all UI elements that are in the way. 

## Support Automation
- Fill the slots as desired (Heal spell for the target you wanna heal, Food/Pills for yourself).
- Target the character you want to follow in game.
- Engage

## Slot Configuration

| Slot Symbol | Flyff Equivalent |
| ----------- | ---------------- |
| 🍔         | Food             |
| 💊         | Pill             |
| ![](./src/assets/heal_spell_16x16.png) | Heal Spell |
| 🐶         | Pickup Pet       |
| ![](./src/assets/icon_motion_pickup_16x16.png) | Pickup Motion |
| ![](./src/assets/icon_refresher_16x16.png) | MP restorer   |
| ![](./src/assets/icon_vitaldrink_16x16.png) | FP restorer   |
| 🗡️         | Attack Skill     |
| 🪄         | Buff Skill       |
| ✈️         | Board/Mount      |

## AutoShout
- Write your messages (1 per line press enter to return line).
- Enter wanted interval.
- Engage and flood chat!

# FAQ

**Is this safe?**<br>
Yes. If you don't trust us, compile it yourself or GTFO.

**Is this a bot?**<br>
It's a client with semi-autonomous automation features.

<!-- Links -->
[banner]: ./banner.png
[download_msi]: https://github.com/MadrigalStreetCartel/neuz/raw/main/releases/Neuz_0.13.0_x64_en-US.msi
[changelog]: https://github.com/MadrigalStreetCartel/neuz/blob/main/CHANGELOG.md

<!-- Disclaimer -->
<small>Disclaimer: We are NOT affiliated with Gala Lab Corp., Sniegu Technologies SAS or Flyff Universe.</small>
