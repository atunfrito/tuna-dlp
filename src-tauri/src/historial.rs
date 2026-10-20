//! Historial de descargas: un JSON en %LOCALAPPDATA%\es.atunfrito.tunadlp,
//! una entrada por archivo bajado.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::{datos_app, err};

#[derive(Clone, Serialize, Deserialize)]
pub struct Entrada {
    pub titulo: String,
    pub miniatura: Option<String>,
    /// Segundos desde 1970.
    pub fecha: u64,
    pub formato: String,
    pub tamano: Option<u64>,
    pub ruta: String,
}

/// Para que dos descargas que acaban a la vez no se pisen al escribir.
#[derive(Default)]
pub struct Cerrojo(Mutex<()>);

fn archivo(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(datos_app(app)?.join("historial.json"))
}

fn leer_de(ruta: &PathBuf) -> Vec<Entrada> {
    fs::read(ruta).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Añade las entradas y avisa al frontend para que salgan sin recargar.
pub fn anotar(app: &AppHandle, nuevas: Vec<Entrada>) -> Result<(), String> {
    if nuevas.is_empty() {
        return Ok(());
    }
    let cerrojo = app.state::<Cerrojo>();
    let _cerrado = cerrojo.0.lock().unwrap();
    let ruta = archivo(app)?;
    let mut todas = leer_de(&ruta);
    todas.extend(nuevas.iter().cloned());
    fs::create_dir_all(ruta.parent().unwrap()).map_err(err)?;
    // Se escribe aparte y se renombra: si la app se cierra a medias, el
    // historial viejo sigue entero.
    let temporal = ruta.with_extension("json.part");
    fs::write(&temporal, serde_json::to_vec(&todas).map_err(err)?).map_err(err)?;
    fs::rename(&temporal, &ruta).map_err(err)?;
    let _ = app.emit("historial", nuevas);
    Ok(())
}

#[tauri::command]
pub fn historial(app: AppHandle) -> Result<Vec<Entrada>, String> {
    let cerrojo = app.state::<Cerrojo>();
    let _cerrado = cerrojo.0.lock().unwrap();
    Ok(leer_de(&archivo(&app)?))
}
