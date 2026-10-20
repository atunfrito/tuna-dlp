//! Suscripciones: cada canal tiene su propio --download-archive en
//! %LOCALAPPDATA%\es.atunfrito.tunadlp\suscripciones\<id>.txt.

use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use crate::{datos_app, en_segundo_plano, err, error_de, miniatura, yt_dlp};

/// Cuántos vídeos recientes se miran en cada comprobación.
const RECIENTES: &str = "30";

#[derive(Serialize)]
pub struct Canal {
    id: String,
    nombre: String,
    url: String,
    archivo: String,
}

#[derive(Serialize)]
pub struct Nuevo {
    url: String,
    titulo: String,
    miniatura: Option<String>,
}

/// "youtube.com/@canal" → "https://youtube.com/@canal/videos". En YouTube, la
/// raíz del canal devuelve pestañas (vídeos, shorts, directos) y no vídeos.
fn normalizar(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    let es_youtube = url.contains("youtube.com/");
    let tramos: Vec<&str> = url.split('/').collect();
    // https: / "" / youtube.com / @canal  → 4 tramos; channel/UC… → 5
    let raiz = match tramos.get(3) {
        Some(t) if t.starts_with('@') => tramos.len() == 4,
        Some(&("channel" | "c" | "user")) => tramos.len() == 5,
        _ => false,
    };
    if es_youtube && raiz { format!("{url}/videos") } else { url }
}

fn carpeta(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(datos_app(app)?.join("suscripciones"))
}

/// El id viene del frontend: solo cifras, para que no pueda salirse de la carpeta.
fn archivo_de(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("id de canal raro: {id}"));
    }
    Ok(carpeta(app)?.join(format!("{id}.txt")))
}

fn listar(app: &AppHandle, url: &str, limite: Option<&str>) -> Result<Value, String> {
    let mut cmd = yt_dlp(app)?;
    cmd.args(["-J", "--flat-playlist", "--yes-playlist", "--no-warnings", "--socket-timeout", "15"]);
    if let Some(n) = limite {
        cmd.args(["--playlist-end", n]);
    }
    let salida = cmd.args(["--", url]).output().map_err(err)?;
    if !salida.status.success() {
        return Err(error_de(&salida.stderr));
    }
    serde_json::from_slice(&salida.stdout).map_err(err)
}

/// Lo mismo que escribe yt-dlp en el --download-archive: "youtube dQw4w9WgXcQ".
fn clave(e: &Value) -> Option<String> {
    let ie = e["ie_key"].as_str().or(e["extractor_key"].as_str())?;
    Some(format!("{} {}", ie.to_lowercase(), e["id"].as_str()?))
}

fn entradas(v: &Value) -> &[Value] {
    v["entries"].as_array().map_or(&[], Vec::as_slice)
}

/// Da de alta un canal. Todo lo que ya tiene cuenta como visto: si no, la
/// primera comprobación bajaría el canal entero.
#[tauri::command]
pub async fn seguir_canal(app: AppHandle, url: String) -> Result<Canal, String> {
    en_segundo_plano(move || {
        let url = normalizar(&url);
        let v = listar(&app, &url, None)?;
        if v["_type"] != "playlist" {
            return Err("esa url no es un canal".into());
        }
        let texto = |k: &str| v[k].as_str().filter(|t| !t.is_empty()).map(str::to_string);
        let nombre = texto("channel").or_else(|| texto("uploader")).or_else(|| texto("title")).unwrap_or_else(|| url.clone());

        let id = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis()).to_string();
        let archivo = archivo_de(&app, &id)?;
        fs::create_dir_all(carpeta(&app)?).map_err(err)?;
        let vistos: Vec<String> = entradas(&v).iter().filter_map(clave).collect();
        fs::write(&archivo, vistos.join("\n") + "\n").map_err(err)?;
        crate::motor::registrar(&app, &format!("sigue {nombre} ({url}): {} vídeos ya vistos", vistos.len()));

        Ok(Canal { id, nombre, url, archivo: archivo.display().to_string() })
    })
    .await
}

/// Los vídeos recientes que aún no están en el archivo del canal, del más
/// antiguo al más nuevo (el orden en que conviene bajarlos).
#[tauri::command]
pub async fn novedades(app: AppHandle, id: String, url: String) -> Result<Vec<Nuevo>, String> {
    en_segundo_plano(move || {
        let archivo = archivo_de(&app, &id)?;
        let vistos: HashSet<String> =
            // Sin la marca BOM que añaden algunos editores al guardar: yt-dlp no la pone.
            fs::read_to_string(&archivo).unwrap_or_default().lines().map(|l| l.trim_start_matches('\u{feff}').trim().to_string()).collect();
        let v = listar(&app, &url, Some(RECIENTES))?;
        let mut nuevos: Vec<Nuevo> = entradas(&v)
            .iter()
            .filter(|e| clave(e).is_some_and(|k| !vistos.contains(&k)))
            .filter_map(|e| {
                Some(Nuevo {
                    url: e["url"].as_str()?.to_string(),
                    titulo: e["title"].as_str().unwrap_or_default().to_string(),
                    miniatura: miniatura(e, 240),
                })
            })
            .collect();
        nuevos.reverse();
        Ok(nuevos)
    })
    .await
}

#[tauri::command]
pub fn dejar_canal(app: AppHandle, id: String) -> Result<(), String> {
    let archivo = archivo_de(&app, &id)?;
    if archivo.exists() {
        fs::remove_file(archivo).map_err(err)?;
    }
    Ok(())
}
