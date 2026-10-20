//! Recorte de vídeos. No se usa --download-sections (da archivos rotos): se
//! descarga entero y luego se corta con ffmpeg. Si el inicio cae en un
//! fotograma clave basta con copiar; si no, se recodifica para que el corte
//! sea exacto.

use serde::Deserialize;
use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{err, CREATE_NO_WINDOW};

#[derive(Deserialize)]
pub struct Recorte {
    desde: String,
    hasta: String,
}

/// "90", "1:30", "01:02:03.5" → segundos. Vacío → None.
fn segundos(t: &str) -> Result<Option<f64>, String> {
    let t = t.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let partes: Vec<&str> = t.split(':').collect();
    if partes.len() > 3 {
        return Err(format!("tiempo raro: {t}"));
    }
    let mut total = 0.0;
    for p in partes {
        let n: f64 = p.trim().parse().map_err(|_| format!("tiempo raro: {t}"))?;
        if n < 0.0 || !n.is_finite() {
            return Err(format!("tiempo raro: {t}"));
        }
        total = total * 60.0 + n;
    }
    Ok(Some(total))
}

/// Tramo en segundos (inicio, fin opcional), ya validado.
pub fn tramo(r: &Recorte) -> Result<(f64, Option<f64>), String> {
    let desde = segundos(&r.desde)?.unwrap_or(0.0);
    let hasta = segundos(&r.hasta)?;
    if hasta.is_some_and(|h| h <= desde) {
        return Err("el final del recorte va antes del principio".into());
    }
    Ok((desde, hasta))
}

fn herramienta(bin: &Path, nombre: &str) -> Command {
    let mut cmd = Command::new(bin.join(nombre));
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// ¿Hay un fotograma clave justo en `t`? Sin vídeo (solo audio) cuenta como sí.
fn cae_en_clave(bin: &Path, archivo: &Path, t: f64) -> bool {
    if t == 0.0 {
        return true;
    }
    let salida = herramienta(bin, "ffprobe.exe")
        .args(["-v", "error", "-select_streams", "v:0", "-skip_frame", "nokey"])
        .args(["-show_entries", "frame=pts_time", "-of", "csv=p=0"])
        .arg("-read_intervals")
        .arg(format!("{t}%+5"))
        .arg(archivo)
        .output();
    let Ok(salida) = salida else { return false };
    let texto = String::from_utf8_lossy(&salida.stdout);
    let claves: Vec<f64> = texto.lines().filter_map(|l| l.trim().trim_end_matches(',').parse().ok()).collect();
    if claves.is_empty() {
        // Sin fotogramas de vídeo: si hay vídeo pero no se pudo leer, mejor recodificar.
        return !tiene_video(bin, archivo);
    }
    claves.iter().any(|k| (k - t).abs() < 0.05)
}

fn tiene_video(bin: &Path, archivo: &Path) -> bool {
    herramienta(bin, "ffprobe.exe")
        .args(["-v", "error", "-select_streams", "V", "-show_entries", "stream=index", "-of", "csv=p=0"])
        .arg(archivo)
        .output()
        .is_ok_and(|s| !s.stdout.trim_ascii().is_empty())
}

/// Corta `archivo` en su sitio.
pub fn cortar(bin: &Path, archivo: &Path, (desde, hasta): (f64, Option<f64>)) -> Result<(), String> {
    let ext = archivo.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let temporal: PathBuf = archivo.with_extension(format!("recorte.{ext}"));
    let copiar = cae_en_clave(bin, archivo, desde);

    let mut cmd = herramienta(bin, "ffmpeg.exe");
    cmd.args(["-hide_banner", "-nostdin", "-y", "-ss"]).arg(desde.to_string()).arg("-i").arg(archivo);
    // Con -ss antes de -i los tiempos empiezan en 0: el final va como duración.
    if let Some(h) = hasta {
        cmd.arg("-t").arg((h - desde).to_string());
    }
    cmd.args(["-map", "0", "-map_metadata", "0", "-map_chapters", "0", "-c", "copy"]);
    if !copiar {
        // Solo el vídeo principal; la miniatura incrustada se copia tal cual.
        if ext == "webm" {
            cmd.args(["-c:v:0", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-row-mt", "1", "-c:a", "libopus"]);
        } else {
            cmd.args(["-c:v:0", "libx264", "-crf", "18", "-preset", "medium", "-c:a", "aac", "-b:a", "192k"]);
        }
    }
    cmd.arg(&temporal);

    let salida = cmd.output().map_err(err)?;
    if !salida.status.success() {
        let _ = fs::remove_file(&temporal);
        let e = String::from_utf8_lossy(&salida.stderr);
        let ultima = e.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
        return Err(format!("ffmpeg no pudo recortar: {ultima}"));
    }
    fs::rename(&temporal, archivo).map_err(err)
}
