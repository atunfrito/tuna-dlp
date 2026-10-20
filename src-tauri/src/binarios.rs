//! yt-dlp y ffmpeg viajan comprimidos dentro del .exe. La primera vez que
//! hacen falta se extraen a %LOCALAPPDATA%\es.atunfrito.tunadlp\bin\<firma>.

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::OnceLock,
};

include!(concat!(env!("OUT_DIR"), "/indice.rs"));
const COMPRIMIDO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/binarios.zst"));

static CARPETA: OnceLock<Result<PathBuf, String>> = OnceLock::new();

/// Carpeta con los binarios listos para usar. Bloquea mientras se extraen.
pub fn carpeta(base: PathBuf) -> Result<PathBuf, String> {
    CARPETA
        .get_or_init(|| extraer(base.join("bin").join(FIRMA)).map_err(|e| e.to_string()))
        .clone()
}

fn extraer(dir: PathBuf) -> io::Result<PathBuf> {
    if BINARIOS.iter().all(|(nombre, _)| dir.join(nombre).exists()) {
        return Ok(dir);
    }
    fs::create_dir_all(&dir)?;
    let mut flujo = zstd::stream::Decoder::with_buffer(COMPRIMIDO)?;
    flujo.window_log_max(27)?;
    for (nombre, tamano) in BINARIOS {
        // Se escribe aparte y se renombra al final: si la app se cierra a
        // medias no queda un .exe cortado que parezca bueno.
        let temporal = dir.join(format!("{nombre}.part"));
        let mut salida = fs::File::create(&temporal)?;
        let copiados = io::copy(&mut (&mut flujo).take(tamano as u64), &mut salida)?;
        if copiados != tamano as u64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("{nombre} incompleto")));
        }
        salida.sync_all()?;
        drop(salida);
        fs::rename(&temporal, dir.join(nombre))?;
    }
    limpiar_antiguas(&dir);
    Ok(dir)
}

/// Borra las carpetas de versiones anteriores de los binarios.
fn limpiar_antiguas(actual: &Path) {
    let Some(padre) = actual.parent() else { return };
    let Ok(entradas) = fs::read_dir(padre) else { return };
    for e in entradas.flatten() {
        if e.path() != actual {
            let _ = fs::remove_dir_all(e.path());
        }
    }
}
