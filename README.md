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

Planned Features for now:
- Giant detection and avoidance
- Scripting support (realized through a DSL) for custom movement and other behaviors
- Full Mac & Linux cross-platform support
- Automatic deployment script with Docker containers for easy parallelism

Join our Discord: https://discord.gg/cZr3X3mCnq

# Usage

1. Start Neuz as an admin
2. Choose or create your profile
3. Press `Play`
4. Select a playstyle
5. Adjust settings to your liking
6. Press `ENGAGE`

## Engagement Behavior

- Automation will start as soon as you `ENGAGE`.
- Pressing `DISENGAGE` will fully stop the automation.
- Fully workable in background since 0.15.0

## Farming Automation

Use farming automation if you're trying to level up your character or farm sets, quest items, penya, etc.

Works best if you're in a densely populated farming area.

### Requirements

No settings have to be changed by default.

1. Use default theme -> Gold (used by default) 
2. Enable auto-attack (enabled by default)

For optimal performance (optional and not necessarily needed):

1. Disable weather, event effects.
3. Press <kbd>ESC</kbd> a few times before engaging to clear all UI elements that are in the way. 
3. Take a look at #How-To channel on discord for a full setup tutorial.

## Support Automation
- Fill the slots as desired (Heal spell for the target you wanna heal, Food/Pills for yourself).
- Target the character you want to follow in game.
- Engage

## Slot Configuration

| Slot Symbol | Flyff Equivalent |   Description    |
| ----------- | ---------------- | ---------------- |
| 🍔         | Food             |   Heals you fast and has a low cooldown, will trigger when hp are lower than the threshold
| 💊         | Pill             |  Heals you fast but has a long cooldown, same trigger
| ![](./src/assets/heal_spell_16x16.png) | Heal Spell | Only support, heal followed character same except it belongs to the target hp
| 🐶         | Pickup Pet       |  Summon you're pet when needed 
| ![](./src/assets/icon_motion_pickup_16x16.png) | Pickup Motion | Grab items on the ground
| ![](./src/assets/icon_refresher_16x16.png) | MP restorer   | Restore you're mp fast, low cooldown, will trigger when mp are lower than the threshold
| ![](./src/assets/icon_vitaldrink_16x16.png) | FP restorer   | Same for fp
| 🗡️         | Attack Skill     |   Attack skill or **motion**
| 🪄         | Buff Skill       | We waited a long time for this one
| ✈️         | Board/Mount      |   Maybe in the sky

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
[download_msi]: https://github.com/MadrigalStreetCartel/neuz/raw/main/releases/Neuz_0.16.0_x64_en-US.msi
[changelog]: https://github.com/MadrigalStreetCartel/neuz/blob/main/CHANGELOG.md

<!-- Disclaimer -->
<small>Disclaimer: We are NOT affiliated with Gala Lab Corp., Sniegu Technologies SAS or Flyff Universe.</small>
