//! Progreso en el botón de la barra de tareas, pintado sobre el propio icono.
//!
//! La barra nativa de Windows (ITaskbarList3) sale en Windows 11 con el color
//! de énfasis del sistema, que puede ser gris. Aquí siempre es verde al bajar,
//! amarilla en pausa y roja si algo falla.

use std::sync::Mutex;
use tauri::{image::Image, AppHandle, Manager};

use crate::err;

const VERDE: [u8; 3] = [0x3f, 0xb9, 0x5f];
const AMARILLO: [u8; 3] = [0xe0, 0xa8, 0x2e];
const ROJO: [u8; 3] = [0xe5, 0x20, 0x3c];
const TINTA: [u8; 3] = [0x14, 0x10, 0x0d];
const PAPEL: [u8; 3] = [0xf7, 0xf1, 0xe6];

/// Lo último pintado, para no rehacer el icono si no ha cambiado.
#[derive(Default)]
pub struct Ultimo(Mutex<Option<(String, u8)>>);

/// estado: none | normal | paused | error. progreso: de 0 a 1.
#[tauri::command]
pub fn progreso_icono(app: AppHandle, estado: String, progreso: f64) -> Result<(), String> {
    let Some(ventana) = app.get_webview_window("main") else { return Ok(()) };
    let Some(base) = app.default_window_icon() else { return Ok(()) };
    let pct = (progreso.clamp(0.0, 1.0) * 100.0).round() as u8;
    {
        let ultimo = app.state::<Ultimo>();
        let mut u = ultimo.0.lock().unwrap();
        if u.as_ref().is_some_and(|(e, p)| *e == estado && *p == pct) {
            return Ok(());
        }
        *u = Some((estado.clone(), pct));
    }
    let color = match estado.as_str() {
        "normal" => VERDE,
        "paused" => AMARILLO,
        "error" => ROJO,
        // Sin nada en marcha, el icono de siempre.
        _ => return ventana.set_icon(base.clone()).map_err(err),
    };

    let (an, al) = (base.width(), base.height());
    let mut px = base.rgba().to_vec();
    let mut rect = |x0: u32, y0: u32, x1: u32, y1: u32, c: [u8; 3]| {
        for y in y0..y1.min(al) {
            for x in x0..x1.min(an) {
                let i = ((y * an + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
    };
    // Barra ancha en la parte de abajo, con un borde claro para que se vea
    // tanto sobre el fondo oscuro del icono como sobre la barra de tareas.
    let f = |v: f64, lado: u32| (v * lado as f64).round() as u32;
    let (x0, x1, y0, y1) = (f(0.06, an), f(0.94, an), f(0.74, al), f(0.94, al));
    let borde = (an / 32).max(1);
    rect(x0 - borde.min(x0), y0 - borde.min(y0), x1 + borde, y1 + borde, PAPEL);
    rect(x0, y0, x1, y1, TINTA);
    let lleno = x0 + ((x1 - x0) as f64 * pct as f64 / 100.0).round() as u32;
    rect(x0, y0, lleno.max(x0 + borde), y1, color);

    ventana.set_icon(Image::new_owned(px, an, al)).map_err(err)
}
