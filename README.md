# typtaps

> A minimal Typst editor with live preview.

## Required Dependencies

- The Typst CLI.

## Usage

```bash
cargo run --release
```

## Features

- **Live Watch**: Auto-reloads on file changes (Not super consistent).
- **Integrated Preview**: PDF viewer with zoom and pan (Buggy at best).

## Contributing

I need help. Please. I don't know what to add. Thanks....

### Note

I removed the File Tree and the Directory, since I pivoted towards a more simplistic use case. A person who just wants to get a live local rendering of typst directly integrated in their app.

## Future Plans

1. Being able to switch between horizontal and vertical splits.
2. Being able to zoom into the Viewer (This was possible originally, but since remade everything this feature is lost LOL)
3. Being able to resize the viewer and the code editor.


## Acknowledgements:
- Iced-rs GUI Library.
- iced_code_editor by LuDog71FR.
---

