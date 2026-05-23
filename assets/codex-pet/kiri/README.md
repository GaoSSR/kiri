# Kiri Codex Pet

This directory contains the Codex custom pet package for Kiri.

Files:

- `pet.json` - Codex pet manifest.
- `spritesheet.webp` - 1536x1872 atlas, with 8 frames per row and 9 animation rows.

Animation row order follows the current Codex pet contract:

1. `idle`
2. `running-right`
3. `running-left`
4. `waving`
5. `jumping`
6. `failed`
7. `waiting`
8. `running`
9. `review`

To regenerate the transparent atlas from source:

```bash
python3 -m pip install Pillow
python3 scripts/generate-kiri-pet.py
```

If a hand-drawn or generated replacement is produced later, keep the same
`pet.json` shape and replace only `spritesheet.webp`. The atlas must remain
1536x1872, with 8 frames per row and 9 rows.
