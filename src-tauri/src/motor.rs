//! Todo lo de la tarjeta "el motor" de ajustes: versiones, comprobar y
//! actualizar yt-dlp, y el registro en %LOCALAPPDATA%\es.atunfrito.tunadlp\logs.

use serde::Serialize;
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use crate::{binarios, datos_app, en_segundo_plano, err, error_de, yt_dlp, CREATE_NO_WINDOW};

// --- registro ------------------------------------------------------------------

fn carpeta_registros(app: &AppHandle) -> Option<PathBuf> {
    datos_app(app).ok().map(|d| d.join("logs"))
}

/// Fecha UTC legible sin traer una dependencia solo para esto.
fn ahora() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (dias, resto) = (s.div_euclid(86_400), s.rem_euclid(86_400));
    // Días desde 1970 a fecha civil (algoritmo de Howard Hinnant).
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}Z", resto / 3600, resto % 3600 / 60, resto % 60)
}

pub fn registrar(app: &AppHandle, linea: &str) {
    let Some(dir) = carpeta_registros(app) else { return };
    let _ = fs::create_dir_all(&dir);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(dir.join("tuna-dlp.log")) {
        let _ = writeln!(f, "{}  {}", ahora(), linea.replace('\n', "\n                      "));
    }
}

/// Si la app revienta, que quede escrito antes de cerrarse.
pub fn registrar_panicos(app: AppHandle) {
    std::panic::set_hook(Box::new(move |info| registrar(&app, &format!("PÁNICO {info}"))));
}

#[tauri::command]
pub fn abrir_registros(app: AppHandle) {
    if let Some(dir) = carpeta_registros(&app) {
        let _ = fs::create_dir_all(&dir);
        let _ = Command::new("explorer").arg(dir).spawn();
    }
}

#[tauri::command]
pub fn abrir_web() {
    // Dirección fija: el frontend no puede pedir abrir otra.
    let _ = Command::new("explorer").arg("https://atunfrito.es").spawn();
}

// --- versiones -------------------------------------------------------------------

#[derive(Serialize)]
pub struct Versiones {
    ytdlp: String,
    ffmpeg: String,
}

#[tauri::command]
pub async fn versiones(app: AppHandle) -> Result<Versiones, String> {
    en_segundo_plano(move || {
        let ytdlp = yt_dlp(&app)?.arg("--version").output().map_err(err)?;
        let bin = binarios::carpeta(datos_app(&app)?)?;
        let ffmpeg = Command::new(bin.join("ffmpeg.exe"))
            .arg("-version")
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(err)?;
        // "ffmpeg version 9.0.2-essentials_build-www.gyan.dev Copyright…" → "9.0.2"
        let ffmpeg = String::from_utf8_lossy(&ffmpeg.stdout)
            .split_whitespace()
            .nth(2)
            .and_then(|v| v.split('-').next())
            .unwrap_or_default()
            .to_string();
        Ok(Versiones { ytdlp: String::from_utf8_lossy(&ytdlp.stdout).trim().to_string(), ffmpeg })
    })
    .await
}

// --- actualizar yt-dlp --------------------------------------------------------------

fn canal_valido(canal: &str) -> Result<&'static str, String> {
    match canal {
        "stable" => Ok("stable"),
        "nightly" => Ok("nightly"),
        _ => Err(format!("canal desconocido: {canal}")),
    }
}

/// Última versión publicada en el canal, preguntando a la API de GitHub con el
/// curl que trae Windows (así no hace falta un cliente HTTP en la app).
#[tauri::command]
pub async fn ultima_version(canal: String) -> Result<String, String> {
    let repo = match canal_valido(&canal)? {
        "nightly" => "yt-dlp/yt-dlp-nightly-builds",
        _ => "yt-dlp/yt-dlp",
    };
    en_segundo_plano(move || {
        let salida = Command::new("curl.exe")
            .args(["-sfL", "--max-time", "15", "-H", "User-Agent: tuna-dlp"])
            .arg(format!("https://api.github.com/repos/{repo}/releases/latest"))
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(err)?;
        if !salida.status.success() {
            return Err(format!("curl: {}", String::from_utf8_lossy(&salida.stderr).trim()));
        }
        let v: Value = serde_json::from_slice(&salida.stdout).map_err(err)?;
        v["tag_name"].as_str().map(str::to_string).ok_or_else(|| "github no devolvió versión".into())
    })
    .await
}

/// yt-dlp se actualiza a sí mismo dentro de la carpeta de binarios extraídos.
#[tauri::command]
pub async fn actualizar_ytdlp(app: AppHandle, canal: String) -> Result<String, String> {
    let canal = canal_valido(&canal)?;
    en_segundo_plano(move || {
        registrar(&app, &format!("actualizando yt-dlp (canal {canal})"));
        let salida = yt_dlp(&app)?.args(["--update-to", canal]).output().map_err(err)?;
        if !salida.status.success() {
            let e = error_de(&salida.stderr);
            registrar(&app, &format!("fallo al actualizar yt-dlp: {e}"));
            return Err(e);
        }
        let v = yt_dlp(&app)?.arg("--version").output().map_err(err)?;
        let v = String::from_utf8_lossy(&v.stdout).trim().to_string();
        registrar(&app, &format!("yt-dlp actualizado a {v}"));
        Ok(v)
    })
    .await
}
