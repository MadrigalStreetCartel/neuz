# Neuz
> Flyff Universe Client and Bot

## Bot Usage

**Requirements**

1. Use default theme
2. Enable single-click to attack in options
3. Put food item of your choice into slot 1

## Changelog

**0.2.0**
- Improved user interface (added bot starts and pause/resume button)
- Improved retargeting behavior when target marker is gone
- Added automatic foodie consumption

**0.1.0**
- Initial release

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

**Regeneration Logic**

1. Foodie is consumed every 5 seconds after an attack started
2. New attacks reset the counter, so no reg between attacks

Notes:

- The algorithm assumes that it's easy enough to kill the enemy if it takes less than 5 seconds
- In case the attack takes longer, it assumes that consuming one food item every 5 seconds is enough

