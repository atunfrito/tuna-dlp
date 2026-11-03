//! Actualizar yt-dlp sin fiarse de nadie más que de su release oficial.
//!
//! No se usa `yt-dlp --update-to`: comprueba el SHA-256, pero si no encuentra
//! el archivo de sumas reemplaza el binario igualmente. Aquí la descarga solo
//! viene de github.com por HTTPS y el binario nuevo solo sustituye al viejo si
//! su SHA-256 coincide con el de SHA2-256SUMS de esa misma release.

use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path, time::Duration};
use tauri::{AppHandle, Emitter};

use crate::{binarios, cola, datos_app, err, motor::registrar, yt_dlp};

const BINARIO: &str = "yt-dlp.exe";
const SUMAS: &str = "SHA2-256SUMS";

fn repo_de(canal: &str) -> Result<&'static str, String> {
    match canal {
        "stable" => Ok("yt-dlp/yt-dlp"),
        "nightly" => Ok("yt-dlp/yt-dlp-nightly-builds"),
        _ => Err(format!("canal desconocido: {canal}")),
    }
}

/// Solo GitHub: la API, la web de releases y el almacén donde redirige las descargas.
fn host_permitido(host: Option<&str>) -> bool {
    matches!(host, Some("api.github.com" | "github.com" | "objects.githubusercontent.com" | "release-assets.githubusercontent.com"))
}

fn cliente() -> Result<reqwest::Client, String> {
    // reqwest va sin proveedor criptográfico propio: se usa ring, el mismo que el plugin de actualización.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .https_only(true)
        .user_agent(concat!("tuna-dlp/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|intento| {
            if intento.previous().len() > 5 {
                intento.error("demasiadas redirecciones")
            } else if intento.url().scheme() == "https" && host_permitido(intento.url().host_str()) {
                intento.follow()
            } else {
                intento.error("redirección a un sitio no permitido")
            }
        }))
        .build()
        .map_err(err)
}

async fn json(cliente: &reqwest::Client, url: &str) -> Result<Value, String> {
    let r = cliente.get(url).header("Accept", "application/vnd.github+json").send().await.map_err(err)?;
    if !r.status().is_success() {
        return Err(format!("github respondió {}", r.status()));
    }
    let bytes = r.bytes().await.map_err(err)?;
    serde_json::from_slice(&bytes).map_err(err)
}

async fn ultima_release(cliente: &reqwest::Client, repo: &str) -> Result<Value, String> {
    json(cliente, &format!("https://api.github.com/repos/{repo}/releases/latest")).await
}

/// URL de descarga de un archivo de la release, comprobando que sea de ese repo.
fn url_de(release: &Value, repo: &str, nombre: &str) -> Result<String, String> {
    let url = release["assets"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["name"] == nombre))
        .and_then(|x| x["browser_download_url"].as_str())
        .ok_or_else(|| format!("la release no trae {nombre}: no se actualiza"))?;
    if !url.starts_with(&format!("https://github.com/{repo}/releases/download/")) {
        return Err(format!("dirección inesperada para {nombre}: no se actualiza"));
    }
    Ok(url.to_string())
}

#[tauri::command]
pub async fn ultima_version(canal: String) -> Result<String, String> {
    let repo = repo_de(&canal)?;
    let release = ultima_release(&cliente()?, repo).await?;
    release["tag_name"].as_str().map(str::to_string).ok_or_else(|| "github no devolvió versión".into())
}

#[derive(Clone, Serialize)]
struct Progreso {
    bajado: u64,
    total: Option<u64>,
}

#[tauri::command]
pub async fn actualizar_ytdlp(app: AppHandle, canal: String) -> Result<String, String> {
    let repo = repo_de(&canal)?;
    // Con yt-dlp en marcha, Windows no deja reemplazar el .exe.
    if cola::hay_descargas(&app) {
        return Err("hay descargas en marcha: espera a que terminen para actualizar yt-dlp".into());
    }
    let bin = binarios::carpeta(datos_app(&app)?)?;
    registrar(&app, &format!("actualizando yt-dlp desde {repo}"));
    match descargar_y_cambiar(&app, repo, &bin).await {
        Ok(v) => {
            registrar(&app, &format!("yt-dlp actualizado a {v}"));
            Ok(v)
        }
        Err(e) => {
            registrar(&app, &format!("no se actualizó yt-dlp: {e}"));
            Err(e)
        }
    }
}

async fn descargar_y_cambiar(app: &AppHandle, repo: &str, bin: &Path) -> Result<String, String> {
    let c = cliente()?;
    let release = ultima_release(&c, repo).await?;

    // 1. El SHA-256 esperado. Sin él no se sigue.
    let sumas = c.get(url_de(&release, repo, SUMAS)?).send().await.map_err(err)?;
    if !sumas.status().is_success() {
        return Err(format!("no se pudo bajar {SUMAS}: no se actualiza"));
    }
    let sumas = sumas.text().await.map_err(err)?;
    let esperado = sumas
        .lines()
        .find_map(|l| {
            let mut partes = l.split_whitespace();
            let (hash, nombre) = (partes.next()?, partes.next()?);
            (nombre.trim_start_matches('*') == BINARIO).then(|| hash.to_ascii_lowercase())
        })
        .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| format!("{SUMAS} no trae la suma de {BINARIO}: no se actualiza"))?;

    // 2. El binario, a un archivo aparte, calculando el SHA-256 por el camino.
    let r = c.get(url_de(&release, repo, BINARIO)?).send().await.map_err(err)?;
    if !r.status().is_success() {
        return Err(format!("no se pudo bajar {BINARIO} ({})", r.status()));
    }
    let total = r.content_length();
    let nuevo = bin.join("yt-dlp.exe.descarga");
    let mut archivo = fs::File::create(&nuevo).map_err(err)?;
    let mut hash = Sha256::new();
    let (mut bajado, mut avisado) = (0u64, 0u64);
    let mut trozos = r.bytes_stream();
    while let Some(trozo) = trozos.next().await {
        let trozo = match trozo {
            Ok(t) => t,
            Err(e) => {
                drop(archivo);
                let _ = fs::remove_file(&nuevo);
                return Err(format!("la descarga se cortó: {e}"));
            }
        };
        hash.update(&trozo);
        archivo.write_all(&trozo).map_err(err)?;
        bajado += trozo.len() as u64;
        if bajado - avisado > 256 * 1024 {
            avisado = bajado;
            let _ = app.emit("ytdlp-progreso", Progreso { bajado, total });
        }
    }
    archivo.sync_all().map_err(err)?;
    drop(archivo);
    let _ = app.emit("ytdlp-progreso", Progreso { bajado, total });

    // 3. Solo si coincide.
    let real = format!("{:x}", hash.finalize());
    if real != esperado {
        let _ = fs::remove_file(&nuevo);
        return Err(format!("el SHA-256 de {BINARIO} no coincide con el de la release: no se actualiza"));
    }

    // 4. Cambio con vuelta atrás: el viejo se aparta y se recupera si algo falla.
    let actual = bin.join(BINARIO);
    let viejo = bin.join("yt-dlp.exe.viejo");
    let _ = fs::remove_file(&viejo);
    fs::rename(&actual, &viejo).map_err(|e| format!("no se pudo apartar el yt-dlp actual: {e}"))?;
    if let Err(e) = fs::rename(&nuevo, &actual) {
        let _ = fs::rename(&viejo, &actual);
        return Err(format!("no se pudo poner el yt-dlp nuevo: {e}"));
    }
    let salida = yt_dlp(app)?.arg("--version").output().map_err(err)?;
    if !salida.status.success() {
        let _ = fs::remove_file(&actual);
        let _ = fs::rename(&viejo, &actual);
        return Err("el yt-dlp nuevo no arranca: se deja el anterior".into());
    }
    let _ = fs::remove_file(&viejo);
    Ok(String::from_utf8_lossy(&salida.stdout).trim().to_string())
}

// --- la app ----------------------------------------------------------------------

/// La versión portable no se puede actualizar sola (el updater instalaría la app):
/// en ella el aviso lleva a la página de la release. La instalada tiene su desinstalador al lado.
#[tauri::command]
pub fn es_portable() -> bool {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| !d.join("uninstall.exe").is_file())).unwrap_or(true)
}

#[tauri::command]
pub fn abrir_release() {
    // Dirección fija: el frontend no puede pedir abrir otra.
    let _ = std::process::Command::new("explorer").arg("https://github.com/atunfrito/tuna-dlp/releases/latest").spawn();
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn solo_github() {
        assert!(host_permitido(Some("github.com")));
        assert!(host_permitido(Some("objects.githubusercontent.com")));
        assert!(!host_permitido(Some("github.com.malo.es")));
        assert!(!host_permitido(Some("evil.com")));
        assert!(!host_permitido(None));
    }

    #[test]
    fn urls_de_la_release() {
        let r: Value = serde_json::json!({ "assets": [
            { "name": "yt-dlp.exe", "browser_download_url": "https://github.com/yt-dlp/yt-dlp/releases/download/2026.10.26/yt-dlp.exe" },
            { "name": "SHA2-256SUMS", "browser_download_url": "https://evil.com/SHA2-256SUMS" }
        ]});
        assert!(url_de(&r, "yt-dlp/yt-dlp", "yt-dlp.exe").is_ok());
        assert!(url_de(&r, "yt-dlp/yt-dlp", "SHA2-256SUMS").is_err());
        assert!(url_de(&r, "yt-dlp/yt-dlp", "otro.exe").is_err());
        assert!(url_de(&r, "otro/repo", "yt-dlp.exe").is_err());
    }
}
