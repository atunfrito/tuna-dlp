//! "abrir con windows": una entrada en HKCU\...\Run que arranca tuna-dlp
//! escondida en la bandeja (--bandeja). Sin permisos de administrador.

use std::{os::windows::process::CommandExt, process::Command};

use crate::{err, CREATE_NO_WINDOW};

const CLAVE: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NOMBRE: &str = "tuna-dlp";
/// Argumento con el que arranca Windows: la ventana empieza escondida.
pub const EN_BANDEJA: &str = "--bandeja";

#[tauri::command]
pub fn abrir_con_windows(activar: bool) -> Result<(), String> {
    let mut reg = Command::new("reg");
    if activar {
        // La ruta del .exe de ahora: si se mueve (versión portable), se reescribe al abrirla.
        let exe = std::env::current_exe().map_err(err)?;
        reg.args(["add", CLAVE, "/v", NOMBRE, "/t", "REG_SZ", "/f", "/d"])
            .arg(format!("\"{}\" {EN_BANDEJA}", exe.display()));
    } else {
        reg.args(["delete", CLAVE, "/v", NOMBRE, "/f"]);
    }
    let salida = reg.creation_flags(CREATE_NO_WINDOW).output().map_err(err)?;
    // Borrar algo que no existe también es "apagado".
    if activar && !salida.status.success() {
        return Err(String::from_utf8_lossy(&salida.stderr).trim().to_string());
    }
    Ok(())
}
