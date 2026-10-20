//! Avisos con el estilo de la app, en vez de las notificaciones de Windows
//! (que, sin instalador, salen firmadas por "Windows PowerShell"). Es una
//! ventanita sin bordes abajo a la derecha que no roba el foco.

use serde_json::{json, Value};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{err, script_inicio};

const ANCHO: f64 = 400.0;
const ALTO: f64 = 170.0;

/// Cada aviso nuevo sustituye al anterior.
#[tauri::command]
pub async fn aviso(app: AppHandle, datos: Value) -> Result<(), String> {
    if let Some(v) = app.get_webview_window("aviso") {
        let _ = v.destroy();
    }
    let pantalla = app.primary_monitor().map_err(err)?.ok_or("no hay pantalla")?;
    let escala = pantalla.scale_factor();
    // La zona de trabajo ya descuenta la barra de tareas.
    let zona = pantalla.work_area();
    let x = (zona.position.x as f64 + zona.size.width as f64) / escala - ANCHO;
    let y = (zona.position.y as f64 + zona.size.height as f64) / escala - ALTO;

    let tema = datos["tema"].clone();
    let inicio = json!({ "ajustes": { "tema": tema }, "rutas": {}, "aviso": datos });
    WebviewWindowBuilder::new(&app, "aviso", WebviewUrl::default())
        .title("tuna-dlp")
        .inner_size(ANCHO, ALTO)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(false)
        .zoom_hotkeys_enabled(false)
        .initialization_script(script_inicio(&inicio))
        .build()
        .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn cerrar_aviso(app: AppHandle) {
    if let Some(v) = app.get_webview_window("aviso") {
        let _ = v.destroy();
    }
}
