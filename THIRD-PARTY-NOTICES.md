# Software de terceros

El código de tuna-dlp se publica con [licencia MIT](LICENSE). El instalador y el
`.exe` incluyen además los programas y librerías de abajo, que **mantienen sus
propias licencias**. Los textos completos están en la carpeta [`licenses/`](licenses).

## Programas incluidos dentro del .exe

tuna-dlp no modifica estos programas: van comprimidos dentro del `.exe`, se extraen
tal cual a `%LOCALAPPDATA%\es.atunfrito.tunadlp\bin` y se ejecutan como procesos
aparte.

### yt-dlp

- **Qué es:** el programa que hace las descargas.
- **Versión en tuna-dlp 1.0.0:** 2026.08.19 (el `yt-dlp.exe` oficial; se puede actualizar desde la app).
- **Licencia:** [Unlicense](licenses/yt-dlp-LICENSE.txt) (dominio público).
- **Código fuente:** https://github.com/yt-dlp/yt-dlp

### FFmpeg (`ffmpeg.exe` y `ffprobe.exe`)

- **Qué es:** une audio y vídeo, convierte formatos, incrusta miniaturas y recorta.
- **Versión en tuna-dlp 1.0.0:** FFmpeg 9.0.2, build *release essentials* de
  [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), sin modificar.
- **Licencia:** **GNU GPL v3**. Ese build está compilado con
  `--enable-gpl --enable-version3` e incluye librerías GPL (x264, x265, entre otras),
  así que el conjunto se distribuye bajo la [GPL v3](licenses/GPL-3.0.txt).
  Detalles de la licencia de FFmpeg: [licenses/FFmpeg-LICENSE.md](licenses/FFmpeg-LICENSE.md).
- **Código fuente correspondiente:**
  - FFmpeg 9.0.2: https://ffmpeg.org/releases/ y https://github.com/FFmpeg/FFmpeg/tree/n9.0.2
  - Configuración exacta y librerías de cada build de gyan.dev: https://www.gyan.dev/ffmpeg/builds/
    (`ffmpeg -version` también muestra la configuración completa).
  - Si necesitas el código fuente de ese build y no puedes conseguirlo con estos
    enlaces, [abre un issue](https://github.com/atunfrito/tuna-dlp/issues) y te lo
    facilito, durante al menos tres años desde la última versión publicada de tuna-dlp
    que lo incluya.

FFmpeg es una marca de Fabrice Bellard, creador del proyecto FFmpeg.

## Librerías usadas para construir la app

| Proyecto | Licencia | Copyright |
| --- | --- | --- |
| [Tauri](https://github.com/tauri-apps/tauri) | MIT o Apache-2.0 | © Tauri Programme within The Commons Conservancy |
| [Svelte](https://github.com/sveltejs/svelte) | MIT | © Svelte contributors |
| [Lucide](https://github.com/lucide-icons/lucide) (iconos) | ISC | © Lucide Contributors; partes de Feather © Cole Bemis (MIT) |
| [zstd](https://github.com/facebook/zstd) (vía el crate `zstd`) | BSD-3-Clause o GPL-2.0 | © Meta Platforms, Inc. |

Las demás dependencias de Rust y de npm, con sus versiones exactas, están en
`src-tauri/Cargo.lock` y `package-lock.json`. Todas usan licencias permisivas
(MIT, Apache-2.0, BSD, ISC o similares) y cada paquete incluye la suya.

## Fuentes

Todas con [SIL Open Font License 1.1](licenses/OFL-1.1.txt), empaquetadas desde
[Fontsource](https://fontsource.org):

- **Instrument Serif**: © The Instrument Serif Project Authors
- **Archivo**: © The Archivo Project Authors
- **JetBrains Mono**: © The JetBrains Mono Project Authors
- **Caveat**: © The Caveat Project Authors
