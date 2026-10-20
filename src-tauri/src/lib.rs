mod aviso;
mod bandeja;
mod barra;
mod binarios;
mod canales;
mod cola;
mod historial;
mod inicio;
mod motor;
mod recorte;

use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_dialog::DialogExt;

pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn err(e: impl ToString) -> String {
    e.to_string()
}

pub(crate) fn datos_app(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_local_data_dir().map_err(err)
}

fn archivo_ajustes(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(err)?.join("ajustes.json"))
}

/// yt-dlp listo para lanzar: sin ventana de consola y con la salida en UTF-8.
/// (--encoding hace falta: el yt-dlp.exe empaquetado ignora PYTHONUTF8 y
/// escribe en cp1252, lo que rompe títulos y rutas con caracteres raros.)
pub(crate) fn yt_dlp(app: &AppHandle) -> Result<Command, String> {
    let bin = binarios::carpeta(datos_app(app)?)?;
    let mut cmd = Command::new(bin.join("yt-dlp.exe"));
    cmd.args(["--encoding", "utf-8", "--ffmpeg-location"])
        .arg(&bin)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        // Sin consola no hay stdin que heredar: mejor uno vacío que uno roto.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);
    Ok(cmd)
}

/// Solo las líneas ERROR: de yt-dlp si las hay; si no, todo stderr tal cual.
pub(crate) fn error_de(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let errores: Vec<&str> = stderr.lines().filter(|l| l.starts_with("ERROR:")).collect();
    if errores.is_empty() { stderr.trim().to_string() } else { errores.join("\n") }
}

pub(crate) async fn en_segundo_plano<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(err)?
}

#[derive(Serialize)]
struct Info {
    titulo: String,
    canal: String,
    sitio: String,
    miniatura: Option<String>,
    /// Idiomas de los subtítulos que tiene de verdad, subidos por el autor.
    subtitulos: Vec<String>,
    /// Los automáticos de YouTube que no están ya arriba: primero el idioma
    /// original del vídeo, luego las traducciones a máquina.
    automaticos: Vec<String>,
    /// Campos de yt-dlp para la vista previa de la plantilla de nombres.
    campos: HashMap<String, String>,
}

#[derive(Serialize)]
struct Lista {
    titulo: String,
    entradas: Vec<Entrada>,
}

#[derive(Serialize)]
struct Entrada {
    titulo: String,
    miniatura: Option<String>,
}

/// yt-dlp -J --flat-playlist: rápido incluso con listas enormes.
fn json_de(app: &AppHandle, url: &str, lista: bool) -> Result<Value, String> {
    let salida = yt_dlp(app)?
        .args(["-J", "--flat-playlist", "--no-warnings", "--socket-timeout", "15"])
        .arg(if lista { "--yes-playlist" } else { "--no-playlist" })
        .args(["--", url.trim()])
        .output()
        .map_err(err)?;
    if !salida.status.success() {
        return Err(error_de(&salida.stderr));
    }
    serde_json::from_slice(&salida.stdout).map_err(err)
}

/// La miniatura más pequeña que aún se vea bien a `min` px de ancho.
pub(crate) fn miniatura(v: &Value, min: u64) -> Option<String> {
    v["thumbnails"]
        .as_array()
        .and_then(|ts| {
            ts.iter()
                .filter(|t| t["width"].as_u64().is_some_and(|w| w >= min))
                .min_by_key(|t| t["width"].as_u64())
                .or_else(|| ts.last())
                .and_then(|t| t["url"].as_str().map(str::to_string))
        })
        .or_else(|| v["thumbnail"].as_str().map(str::to_string))
}

/// Título, canal, miniatura y subtítulos para la vista previa.
#[tauri::command]
async fn info(app: AppHandle, url: String) -> Result<Info, String> {
    en_segundo_plano(move || {
        let v = json_de(&app, &url, false)?;
        let texto = |k: &str| v[k].as_str().map(str::to_string);
        let mut subtitulos: Vec<String> = v["subtitles"]
            .as_object()
            .map(|m| m.keys().filter(|k| *k != "live_chat").cloned().collect())
            .unwrap_or_default();
        subtitulos.sort();
        let auto = v["automatic_captions"].as_object();
        // YouTube marca el idioma original con "-orig" (p. ej. "es-orig" junto a "es").
        let original: Vec<String> = auto
            .map(|m| m.keys().filter_map(|k| k.strip_suffix("-orig").map(str::to_string)).collect())
            .unwrap_or_default();
        let mut automaticos: Vec<String> = auto
            .map(|m| m.keys().filter(|k| !k.ends_with("-orig") && !subtitulos.contains(k)).cloned().collect())
            .unwrap_or_default();
        automaticos.sort_by_key(|k| !original.contains(k));
        let campos = v
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, x)| match x {
                        Value::String(t) => Some((k.clone(), t.clone())),
                        Value::Number(n) => Some((k.clone(), n.to_string())),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Info {
            titulo: texto("title").unwrap_or_default(),
            canal: texto("channel").or_else(|| texto("uploader")).unwrap_or_default(),
            sitio: texto("extractor_key").unwrap_or_default().to_lowercase(),
            // 96×54 en pantallas de alta densidad.
            miniatura: miniatura(&v, 240),
            subtitulos,
            automaticos,
            campos,
        })
    })
    .await
}

/// Los vídeos de una playlist, en orden, para elegir cuáles bajar.
#[tauri::command]
async fn lista(app: AppHandle, url: String) -> Result<Lista, String> {
    en_segundo_plano(move || {
        let v = json_de(&app, &url, true)?;
        let Some(entradas) = v["entries"].as_array() else {
            return Err("esa url no es una lista".into());
        };
        let entradas = entradas
            .iter()
            .map(|e| Entrada {
                titulo: e["title"].as_str().or(e["url"].as_str()).unwrap_or_default().to_string(),
                miniatura: miniatura(e, 120),
            })
            .collect();
        Ok(Lista { titulo: v["title"].as_str().unwrap_or_default().to_string(), entradas })
    })
    .await
}

#[tauri::command]
async fn elegir_carpeta(app: AppHandle, actual: String) -> Option<String> {
    app.dialog()
        .file()
        .set_directory(actual)
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string())
}

#[tauri::command]
fn leer_portapapeles() -> Option<String> {
    clipboard_win::get_clipboard_string().ok()
}

#[tauri::command]
fn guardar_ajustes(app: AppHandle, ajustes: Value) -> Result<(), String> {
    let ruta = archivo_ajustes(&app)?;
    fs::create_dir_all(ruta.parent().unwrap()).map_err(err)?;
    fs::write(ruta, serde_json::to_vec_pretty(&ajustes).map_err(err)?).map_err(err)
}

fn leer_ajustes(app: &AppHandle) -> Value {
    archivo_ajustes(app)
        .ok()
        .and_then(|r| fs::read(r).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(|| json!({}))
}

/// Ajustes y tema antes de pintar, para que no haya parpadeo. Se ejecuta antes
/// de que exista <html>: espera a que aparezca.
pub(crate) fn script_inicio(inicio: &Value) -> String {
    format!(
        "window.__TUNA__ = {inicio};
        (function (t) {{
          if (t !== 'light' && t !== 'dark')
            t = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
          var d = document, s = function () {{ d.documentElement.dataset.theme = t; }};
          if (d.documentElement) s();
          else new MutationObserver(function (m, o) {{
            if (d.documentElement) {{ s(); o.disconnect(); }}
          }}).observe(d, {{ childList: true }});
        }})(window.__TUNA__.ajustes.tema);"
    )
}

pub fn run() {
    tauri::Builder::default()
        // Tiene que ir el primero: si ya hay un tuna-dlp abierto (p. ej. en la
        // bandeja), se enseña ese en vez de abrir otro con un segundo icono.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| bandeja::mostrar(app)))
        .plugin(tauri_plugin_dialog::init())
        .manage(cola::Procesos::default())
        .manage(historial::Cerrojo::default())
        .manage(barra::Ultimo::default())
        .setup(|app| {
            let handle = app.handle().clone();
            motor::registrar_panicos(handle.clone());
            motor::registrar(&handle, &format!("arranca tuna-dlp {}", app.package_info().version));

            // Extrae yt-dlp y ffmpeg en segundo plano nada más arrancar, para
            // que estén listos cuando se pulse "descargar".
            if let Ok(base) = datos_app(&handle) {
                std::thread::spawn(move || {
                    let _ = binarios::carpeta(base);
                });
            }

            let ajustes = leer_ajustes(&handle);
            let descargas = dirs::download_dir().unwrap_or_default();
            let inicio = json!({
                "ajustes": ajustes,
                "rutas": {
                    "descargas": descargas.join("tuna-dlp"),
                    "archivo": datos_app(&handle)?.join("descargados.txt"),
                },
            });
            let oscuro = ajustes["tema"] == "dark";
            let fondo = if oscuro { (0x14, 0x10, 0x0d, 255) } else { (0xe9, 0xdf, 0xcd, 255) };

            // Al arrancar con Windows empieza escondida en la bandeja.
            let escondida_al_empezar = std::env::args().any(|a| a == inicio::EN_BANDEJA);
            let ventana = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .visible(!escondida_al_empezar)
                .title("tuna-dlp")
                .inner_size(1180.0, 760.0)
                .min_inner_size(900.0, 600.0)
                .decorations(false)
                .shadow(true)
                .zoom_hotkeys_enabled(false)
                .background_color(fondo.into())
                // El manejador de Tauri solo entiende archivos: con él, un enlace
                // arrastrado desde el navegador no llega a la página (y el aviso
                // de "suéltalo aquí" se quedaba pegado). Sin él, lo hace WebView2.
                .disable_drag_drop_handler()
                .initialization_script(script_inicio(&inicio))
                .build()?;

            // Cerrar la ventana la esconde en la bandeja: las descargas siguen.
            let escondida = ventana.clone();
            ventana.on_window_event(move |ev| {
                if let WindowEvent::CloseRequested { api, .. } = ev {
                    api.prevent_close();
                    let _ = escondida.hide();
                    let _ = escondida.emit("oculta", ());
                }
            });
            bandeja::crear(&handle, ajustes["tema"].as_str().unwrap_or("system"))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cola::iniciar,
            cola::cancelar,
            cola::mostrar_en_carpeta,
            aviso::aviso,
            aviso::cerrar_aviso,
            bandeja::bandeja,
            barra::progreso_icono,
            bandeja::mostrar_ventana,
            inicio::abrir_con_windows,
            bandeja::estado_bandeja,
            bandeja::accion_bandeja,
            historial::historial,
            canales::seguir_canal,
            canales::novedades,
            canales::dejar_canal,
            motor::versiones,
            motor::ultima_version,
            motor::actualizar_ytdlp,
            motor::abrir_registros,
            motor::abrir_web,
            info,
            lista,
            elegir_carpeta,
            leer_portapapeles,
            guardar_ajustes
        ])
        .build(tauri::generate_context!())
        .expect("error al arrancar tuna-dlp")
        .run(|app, evento| {
            if let tauri::RunEvent::Exit = evento {
                cola::cancelar_todas(app);
            }
        });
}
