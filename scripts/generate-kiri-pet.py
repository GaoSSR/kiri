#!/usr/bin/env python3
"""Generate the Codex pet package for Kiri.

This script keeps the pet reproducible: it draws a compact cloud mascot into
the Codex pet atlas geometry instead of relying on checked-in intermediate
editor files.

Requires Pillow:
    python3 -m pip install Pillow
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError as exc:
    raise SystemExit(
        "Pillow is required. Install it with: python3 -m pip install Pillow"
    ) from exc


CELL_W = 192
CELL_H = 208
FRAMES = 8
ROWS = 9
SCALE = 4

PET_DIR = Path("assets/codex-pet/kiri")
SPRITESHEET = PET_DIR / "spritesheet.webp"
MANIFEST = PET_DIR / "pet.json"

BODY = (232, 249, 255, 255)
BODY_SHADOW = (193, 230, 247, 255)
OUTLINE = (78, 164, 210, 255)
INK = (27, 42, 58, 255)
EYE = (28, 62, 84, 255)
CHEEK = (255, 174, 186, 255)
ACCENT = (112, 226, 96, 255)
SOFT_BLUE = (163, 219, 247, 255)


def scaled_box(box: tuple[float, float, float, float]) -> tuple[int, int, int, int]:
    return tuple(round(value * SCALE) for value in box)  # type: ignore[return-value]


def draw_ellipse(
    draw: ImageDraw.ImageDraw,
    box: tuple[float, float, float, float],
    fill: tuple[int, int, int, int],
    outline: tuple[int, int, int, int] | None = None,
    width: int = 1,
) -> None:
    draw.ellipse(scaled_box(box), fill=fill, outline=outline, width=width * SCALE)


def draw_line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[float, float]],
    fill: tuple[int, int, int, int],
    width: int = 2,
) -> None:
    draw.line([(round(x * SCALE), round(y * SCALE)) for x, y in points], fill=fill, width=width * SCALE)


def draw_arc(
    draw: ImageDraw.ImageDraw,
    box: tuple[float, float, float, float],
    start: int,
    end: int,
    fill: tuple[int, int, int, int],
    width: int = 2,
) -> None:
    draw.arc(scaled_box(box), start=start, end=end, fill=fill, width=width * SCALE)


def draw_cloud_body(
    draw: ImageDraw.ImageDraw,
    x: float,
    y: float,
    *,
    sad: bool = False,
    blink: bool = False,
    focused: bool = False,
    waiting: bool = False,
    wave: float = 0.0,
    direction: int = 1,
) -> None:
    parts = [
        (37 + x, 71 + y, 80 + x, 117 + y),
        (58 + x, 45 + y, 132 + x, 119 + y),
        (117 + x, 64 + y, 158 + x, 117 + y),
        (44 + x, 88 + y, 149 + x, 131 + y),
    ]
    for box in parts:
        draw_ellipse(draw, box, OUTLINE)

    inner_parts = [
        (40 + x, 74 + y, 77 + x, 114 + y),
        (62 + x, 49 + y, 128 + x, 115 + y),
        (120 + x, 68 + y, 154 + x, 114 + y),
        (48 + x, 91 + y, 145 + x, 128 + y),
    ]
    for box in inner_parts:
        draw_ellipse(draw, box, BODY)

    draw_ellipse(draw, (52 + x, 107 + y, 139 + x, 133 + y), BODY_SHADOW)
    draw_ellipse(draw, (53 + x, 102 + y, 140 + x, 127 + y), BODY)

    left_hand = (43 + x, 111 + y, 68 + x, 132 + y)
    right_base_y = 111 + y - wave * 26
    right_hand = (127 + x, right_base_y, 152 + x, right_base_y + 21)
    if direction < 0:
        left_hand, right_hand = (
            (40 + x, right_base_y, 65 + x, right_base_y + 21),
            (124 + x, 111 + y, 149 + x, 132 + y),
        )

    for hand in [left_hand, right_hand]:
        draw_ellipse(draw, hand, OUTLINE)
        draw_ellipse(draw, (hand[0] + 3, hand[1] + 3, hand[2] - 3, hand[3] - 3), BODY)

    if blink:
        draw_line(draw, [(75 + x, 87 + y), (88 + x, 87 + y)], EYE, 3)
        draw_line(draw, [(106 + x, 87 + y), (119 + x, 87 + y)], EYE, 3)
    elif focused:
        draw_line(draw, [(74 + x, 83 + y), (90 + x, 87 + y)], EYE, 3)
        draw_line(draw, [(105 + x, 87 + y), (121 + x, 83 + y)], EYE, 3)
    else:
        eye_h = 28 if waiting else 22
        draw_ellipse(draw, (73 + x, 76 + y, 91 + x, 76 + y + eye_h), EYE)
        draw_ellipse(draw, (107 + x, 76 + y, 125 + x, 76 + y + eye_h), EYE)
        draw_ellipse(draw, (80 + x, 80 + y, 85 + x, 86 + y), (255, 255, 255, 255))
        draw_ellipse(draw, (114 + x, 80 + y, 119 + x, 86 + y), (255, 255, 255, 255))

    draw_ellipse(draw, (57 + x, 96 + y, 71 + x, 106 + y), CHEEK)
    draw_ellipse(draw, (127 + x, 96 + y, 141 + x, 106 + y), CHEEK)

    if sad:
        draw_arc(draw, (84 + x, 102 + y, 112 + x, 122 + y), 200, 340, INK, 2)
        draw_ellipse(draw, (135 + x, 84 + y, 146 + x, 99 + y), SOFT_BLUE)
        draw_ellipse(draw, (137 + x, 84 + y, 146 + x, 93 + y), SOFT_BLUE)
    elif focused:
        draw_line(draw, [(86 + x, 105 + y), (112 + x, 105 + y)], INK, 2)
    else:
        draw_arc(draw, (83 + x, 92 + y, 115 + x, 114 + y), 25, 155, INK, 2)


def draw_badge(draw: ImageDraw.ImageDraw, x: float, y: float) -> None:
    draw_ellipse(draw, (116 + x, 51 + y, 142 + x, 77 + y), OUTLINE)
    draw_ellipse(draw, (120 + x, 55 + y, 138 + x, 73 + y), ACCENT)


def frame_for_state(row: int, frame: int) -> Image.Image:
    canvas = Image.new("RGBA", (CELL_W * SCALE, CELL_H * SCALE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    phase = frame / FRAMES
    bob = math.sin(phase * math.tau) * 3
    x = 0.0
    y = 0.0
    kwargs: dict[str, object] = {}

    if row == 0:
        y = bob
        kwargs["blink"] = frame in {3, 4}
    elif row == 1:
        x = math.sin(phase * math.tau) * 6
        y = abs(math.sin(phase * math.tau)) * 4
        kwargs["direction"] = 1
        kwargs["focused"] = frame % 2 == 0
    elif row == 2:
        x = -math.sin(phase * math.tau) * 6
        y = abs(math.sin(phase * math.tau)) * 4
        kwargs["direction"] = -1
        kwargs["focused"] = frame % 2 == 0
    elif row == 3:
        y = bob
        kwargs["wave"] = (math.sin(phase * math.tau) + 1) / 2
    elif row == 4:
        y = -abs(math.sin(phase * math.tau)) * 20
    elif row == 5:
        y = 2
        kwargs["sad"] = True
    elif row == 6:
        y = bob / 2
        kwargs["waiting"] = True
    elif row == 7:
        x = math.sin(phase * math.tau) * 2
        y = bob / 2
        kwargs["focused"] = True
    elif row == 8:
        y = bob / 2
        kwargs["focused"] = frame in {1, 2, 5, 6}

    draw_cloud_body(draw, x, y, **kwargs)

    if row == 6 and frame % 2 == 0:
        draw_badge(draw, x, y)
    if row == 7:
        draw_badge(draw, x - 22, y + 52)
    if row == 8 and frame in {2, 3, 4}:
        draw_ellipse(draw, (132 + x, 55 + y, 143 + x, 66 + y), ACCENT)

    return canvas.resize((CELL_W, CELL_H), Image.Resampling.LANCZOS)


def write_pet_package() -> None:
    PET_DIR.mkdir(parents=True, exist_ok=True)
    sheet = Image.new("RGBA", (CELL_W * FRAMES, CELL_H * ROWS), (0, 0, 0, 0))

    for row in range(ROWS):
        for frame in range(FRAMES):
            sheet.paste(frame_for_state(row, frame), (frame * CELL_W, row * CELL_H))

    sheet.save(SPRITESHEET, "WEBP", lossless=True, quality=100, method=6)
    MANIFEST.write_text(
        json.dumps(
            {
                "id": "kiri",
                "displayName": "Kiri",
                "description": "A cheerful cloud companion that watches local development ports.",
                "spritesheetPath": "spritesheet.webp",
            },
            indent=2,
        )
        + "\n"
    )


def main() -> int:
    write_pet_package()
    print(f"wrote {SPRITESHEET}")
    print(f"wrote {MANIFEST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
