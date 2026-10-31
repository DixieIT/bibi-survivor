# Bibi Survivor

A small top-down pixel-art survival game written in Rust with [macroquad](https://macroquad.rs).
You play a sword-wielding chibi girl holding off endless waves of skeletons.

## Run

```
cargo run --release
```

Pick a map from the menu. Controls: **WASD/arrows** move, **Shift** sprint, **J/Space** attack
(in the held direction, chaining a 3-hit combo), **E** dash and **Q** sword vortex (once unlocked),
**1/2/3** pick a power-up, **F5** restart, **Esc** back to the menu.

## Features

- Three maps: "Ruined Hamlet" (a burnt-out village at night, in fog and firelight), a hand-painted village and "Tiny Town", built from tiles
- Finite waves: clear one, pick one of three power-ups (slash, shield, dash, life steal, sword vortex, stamina), catch your breath, face the next
- Skeletons that path around obstacles; melee ones lunge, archers keep their distance and shoot (in volleys from wave 4),
  and from wave 5 sturdy tanks and fireball-casting mages join in
- Stamina, health and heart drops

## Project layout

- `src/`: game code
- `assets/`: sprites, maps (`assets/maps/<name>/`) and the tileset. Files here override the copies built into the exe.
- `tools/`: Python scripts that generate the sprites, maps and collision masks

## Adding a map

Feel free to add maps! Create a folder `assets/maps/<name>/` with:

- `map.png`: the map picture (the only required file). The player is about 24 px tall, so size it accordingly.
- `collision.png`: same size as the map; any non-transparent pixel blocks movement. Without it, nothing blocks.
- `map.txt`: settings, one per line:
  ```
  title My Map
  spawn 320 240
  ```
  `spawn` is the player's start position, in map pixels. Optional atmosphere, all from 0 to 1:
  `night 0.4` (darkness), `fog 0.5` (drifting mist), `ash 0.6` (falling ash); and lights in the
  dark, `light <x> <y> <radius> [fire|torch|glow]` (fires and torches flicker and throw embers).
- `overlay.png` + `overlays.txt` (optional): bits of the map, such as tree crowns or roofs, drawn over
  characters standing behind them. See `assets/maps/town/` for an example; `tools/make_town_map.py`
  and `tools/make_ruins_map.py` write them.
