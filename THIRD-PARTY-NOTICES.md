# Third-party software

*[Versión en español](THIRD-PARTY-NOTICES.es.md)*

tuna-dlp's own code is released under the [MIT License](LICENSE). The installer and
the `.exe` also include the programs and libraries below, which **keep their own
licenses**. The full license texts are in the [`licenses/`](licenses) folder.

## Programs bundled inside the .exe

tuna-dlp doesn't modify these programs: they're compressed inside the `.exe`,
extracted as they are to `%LOCALAPPDATA%\es.atunfrito.tunadlp\bin`, and run as
separate processes.

### yt-dlp

- **What it does:** the actual downloading.
- **Version bundled:** 2026.08.19 (the official `yt-dlp.exe`; it can be updated from inside the app).
- **License:** [Unlicense](licenses/yt-dlp-LICENSE.txt) (public domain).
- **Source code:** https://github.com/yt-dlp/yt-dlp

### FFmpeg (`ffmpeg.exe` and `ffprobe.exe`)

- **What it does:** merges audio and video, converts formats, embeds thumbnails and trims.
- **Version bundled:** FFmpeg 9.0.2, the *release essentials* build by
  [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), unmodified.
- **License:** **GNU GPL v3**. That build is compiled with
  `--enable-gpl --enable-version3` and includes GPL libraries (x264, x265 and others),
  so the whole is distributed under the [GPL v3](licenses/GPL-3.0.txt).
  FFmpeg's licensing details: [licenses/FFmpeg-LICENSE.md](licenses/FFmpeg-LICENSE.md).
- **Corresponding source code:**
  - FFmpeg 9.0.2: https://ffmpeg.org/releases/ and https://github.com/FFmpeg/FFmpeg/tree/n9.0.2
  - Exact configuration and libraries of each gyan.dev build: https://www.gyan.dev/ffmpeg/builds/
    (`ffmpeg -version` also prints the full configuration).
  - If you need the source code of that build and can't get it from these links,
    [open an issue](https://github.com/atunfrito/tuna-dlp/issues) and I'll provide it,
    for at least three years after the last tuna-dlp release that includes it.

FFmpeg is a trademark of Fabrice Bellard, originator of the FFmpeg project.

## Libraries used to build the app

| Project | License | Copyright |
| --- | --- | --- |
| [Tauri](https://github.com/tauri-apps/tauri) | MIT or Apache-2.0 | © Tauri Programme within The Commons Conservancy |
| [Svelte](https://github.com/sveltejs/svelte) | MIT | © Svelte contributors |
| [Lucide](https://github.com/lucide-icons/lucide) (icons) | ISC | © Lucide Contributors; parts of Feather © Cole Bemis (MIT) |
| [zstd](https://github.com/facebook/zstd) (through the `zstd` crate) | BSD-3-Clause or GPL-2.0 | © Meta Platforms, Inc. |

The other Rust and npm dependencies, with their exact versions, are listed in
`src-tauri/Cargo.lock` and `package-lock.json`. They all use permissive licenses
(MIT, Apache-2.0, BSD, ISC or similar), and each package includes its own.

## Fonts

All under the [SIL Open Font License 1.1](licenses/OFL-1.1.txt), packaged by
[Fontsource](https://fontsource.org):

- **Instrument Serif**: © The Instrument Serif Project Authors
- **Archivo**: © The Archivo Project Authors
- **JetBrains Mono**: © The JetBrains Mono Project Authors
- **Caveat**: © The Caveat Project Authors
