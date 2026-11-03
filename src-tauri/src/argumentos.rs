//! Los argumentos de yt-dlp llegan montados desde la interfaz (src/lib/comando.js).
//! Aquí se comprueba que solo sean las opciones que la app usa de verdad: si
//! alguien consiguiera ejecutar código en la interfaz, no podría pasar --exec,
//! --batch-file, --config-locations ni nada parecido para lanzar programas o
//! leer y escribir archivos donde no toca.

use std::path::{Component, Path};

/// Opciones sin valor.
const SUELTAS: &[&str] = &[
    "-x",
    "--skip-download",
    "--write-subs",
    "--write-auto-subs",
    "--embed-thumbnail",
    "--embed-metadata",
    "--embed-chapters",
    "--embed-subs",
    "--yes-playlist",
    "--no-playlist",
    "--windows-filenames",
];

/// Opciones que llevan un valor detrás.
const CON_VALOR: &[&str] = &[
    "--audio-format",
    "--audio-quality",
    "--sub-langs",
    "-f",
    "-S",
    "--merge-output-format",
    "--sponsorblock-remove",
    "-I",
    "--download-archive",
    "-r",
    "--cookies-from-browser",
    "-P",
    "-o",
];

const NAVEGADORES: &[&str] = &["firefox", "brave", "chrome", "edge", "chromium", "opera", "vivaldi"];

/// La plantilla de nombre (-o) es relativa a la carpeta (-P): nada de rutas
/// absolutas ni de subir con "..".
fn plantilla_valida(o: &str) -> bool {
    let p = Path::new(o);
    !o.is_empty()
        && !p.is_absolute()
        && !o.starts_with(['\\', '/'])
        && !p.components().any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
}

/// `datos` es la carpeta de la app: los --download-archive solo pueden vivir ahí.
pub fn validar(args: &[String], datos: &Path) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        let op = args[i].as_str();
        if SUELTAS.contains(&op) {
            i += 1;
            continue;
        }
        if !CON_VALOR.contains(&op) {
            return Err(format!("opción de yt-dlp no permitida: {op}"));
        }
        let Some(valor) = args.get(i + 1) else {
            return Err(format!("a {op} le falta el valor"));
        };
        let bien = match op {
            "--cookies-from-browser" => NAVEGADORES.contains(&valor.as_str()),
            "--download-archive" => {
                let p = Path::new(valor);
                p.is_absolute() && p.starts_with(datos) && !p.components().any(|c| c == Component::ParentDir)
            }
            "-o" => plantilla_valida(valor),
            _ => true,
        };
        if !bien {
            return Err(format!("valor no permitido para {op}: {valor}"));
        }
        i += 2;
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }
    const DATOS: &str = r"C:\Users\x\AppData\Local\es.atunfrito.tunadlp";

    #[test]
    fn lo_que_monta_la_app_vale() {
        let video = v(&[
            "-f", "bv*+ba/b", "-S", "res:1080,vcodec:h264", "--merge-output-format", "mp4",
            "--embed-thumbnail", "--embed-metadata", "--embed-chapters", "--sponsorblock-remove", "sponsor,selfpromo",
            "--no-playlist", "--download-archive", r"C:\Users\x\AppData\Local\es.atunfrito.tunadlp\descargados.txt",
            "-r", "2M", "--cookies-from-browser", "firefox", "--windows-filenames",
            "-P", r"C:\Users\x\Downloads\tuna-dlp", "-o", "%(title)s.%(ext)s",
        ]);
        assert!(validar(&video, Path::new(DATOS)).is_ok());
        let audio = v(&["-x", "--audio-format", "mp3", "--audio-quality", "0", "--yes-playlist", "-I", "1:3,5", "-P", "D:\\m", "-o", "%(uploader)s/%(title)s.%(ext)s"]);
        assert!(validar(&audio, Path::new(DATOS)).is_ok());
        let subs = v(&["--skip-download", "--write-subs", "--write-auto-subs", "--sub-langs", "es,en", "--no-playlist", "-P", "D:\\s", "-o", "%(title)s.%(ext)s"]);
        assert!(validar(&subs, Path::new(DATOS)).is_ok());
    }

    #[test]
    fn opciones_peligrosas_no() {
        for mala in ["--exec", "--exec-before-download", "--batch-file", "-a", "--config-locations", "--plugin-dirs", "--update", "--netrc-cmd", "--load-info-json", "--print-to-file"] {
            assert!(validar(&v(&[mala, "calc.exe"]), Path::new(DATOS)).is_err(), "{mala} debería rechazarse");
        }
    }

    #[test]
    fn valores_peligrosos_no() {
        let d = Path::new(DATOS);
        assert!(validar(&v(&["-o", r"C:\Windows\System32\x.%(ext)s"]), d).is_err());
        assert!(validar(&v(&["-o", "../../fuera.%(ext)s"]), d).is_err());
        assert!(validar(&v(&["-o", r"\\servidor\x"]), d).is_err());
        assert!(validar(&v(&["--download-archive", r"C:\Windows\win.ini"]), d).is_err());
        assert!(validar(&v(&["--download-archive", r"C:\Users\x\AppData\Local\es.atunfrito.tunadlp\..\..\otro.txt"]), d).is_err());
        assert!(validar(&v(&["--cookies-from-browser", "firefox:../../perfil"]), d).is_err());
        assert!(validar(&v(&["-P"]), d).is_err());
    }
}
