//! Descargas de la cola: cada una es un proceso de yt-dlp cuya salida se lee
//! línea a línea y se manda al frontend como eventos.

use serde::Serialize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    sync::Mutex,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    binarios, datos_app, error_de,
    historial::{self, Entrada},
    motor::registrar,
    recorte::{self, Recorte},
    yt_dlp, CREATE_NO_WINDOW,
};

/// PIDs de las descargas en marcha, para poder cancelarlas.
#[derive(Default)]
pub struct Procesos(Mutex<HashMap<u32, u32>>);

#[derive(Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "lowercase")]
enum Evento {
    Titulo { id: u32, titulo: String, miniatura: Option<String> },
    Progreso { id: u32, bajado: Option<f64>, total: Option<f64>, velocidad: Option<f64>, eta: Option<f64> },
    Proceso { id: u32, paso: String },
    Fin { id: u32, ok: bool, ruta: Option<String>, tamano: Option<u64>, formato: Option<String>, error: String },
}

// Plantillas con marcas propias para separar lo nuestro del resto de la salida.
const PLANTILLAS: [&str; 10] = [
    "--newline",
    "--progress",
    "--progress-template",
    "download:TUNAD %(progress.downloaded_bytes)s %(progress.total_bytes)s %(progress.total_bytes_estimate)s %(progress.speed)s %(progress.eta)s",
    "--progress-template",
    "postprocess:TUNAP %(progress.status)s %(progress.postprocessor)s",
    "--print",
    "before_dl:TUNAT %(thumbnail)s %(title)s",
    "--print",
    // La altura real (NA en audio) y, al final por si lleva espacios, la ruta.
    "after_move:TUNAF %(height)s %(filepath)s",
];

fn numero(s: Option<&str>) -> Option<f64> {
    s.and_then(|v| v.parse().ok())
}

/// Un archivo terminado, con el título y la miniatura del vídeo del que salió.
struct Archivo {
    ruta: String,
    titulo: String,
    miniatura: Option<String>,
    /// Lo que se ha bajado de verdad, no lo que se pidió: "mp4 240p", "mp3".
    formato: String,
}

/// "mp4 240p" a partir de la ruta final y la altura que da yt-dlp. La
/// extensión sale del archivo porque tras convertir (mp3, recorte…) es la buena.
fn formato_real(ruta: &str, altura: &str) -> String {
    let ext = std::path::Path::new(ruta).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match altura.parse::<u32>() {
        Ok(h) if h > 0 => format!("{ext} {h}p"),
        _ => ext,
    }
}

/// Lo que se va sabiendo de una descarga (en una lista, vídeo a vídeo).
#[derive(Default)]
struct Seguimiento {
    titulo: String,
    miniatura: Option<String>,
    archivos: Vec<Archivo>,
}

/// Traduce una línea de yt-dlp a un evento, si es una de las nuestras.
fn interpretar(id: u32, linea: &str, seg: &mut Seguimiento) -> Option<Evento> {
    let (marca, resto) = linea.split_once(' ')?;
    match marca {
        "TUNAT" => {
            let (miniatura, titulo) = resto.split_once(' ')?;
            let miniatura = (miniatura != "NA").then(|| miniatura.to_string());
            seg.titulo = titulo.to_string();
            seg.miniatura = miniatura.clone();
            Some(Evento::Titulo { id, titulo: titulo.to_string(), miniatura })
        }
        "TUNAF" => {
            let (altura, ruta) = resto.split_once(' ')?;
            seg.archivos.push(Archivo {
                ruta: ruta.to_string(),
                titulo: seg.titulo.clone(),
                miniatura: seg.miniatura.clone(),
                formato: formato_real(ruta, altura),
            });
            None
        }
        "TUNAD" => {
            // bajado, total, total estimado, velocidad, eta ("NA" si no se sabe)
            let c: Vec<Option<f64>> = resto.split(' ').map(|v| numero(Some(v))).collect();
            let campo = |i: usize| c.get(i).copied().flatten();
            Some(Evento::Progreso {
                id,
                bajado: campo(0),
                total: campo(1).or(campo(2)),
                velocidad: campo(3),
                eta: campo(4),
            })
        }
        "TUNAP" => {
            let (estado, paso) = resto.split_once(' ')?;
            // MoveFiles solo mueve el archivo final: no merece un aviso.
            (estado == "started" && paso != "MoveFiles").then(|| Evento::Proceso { id, paso: paso.to_string() })
        }
        _ => None,
    }
}

#[tauri::command]
pub async fn iniciar(
    app: AppHandle,
    id: u32,
    url: String,
    args: Vec<String>,
    recorte: Option<Recorte>,
    formato: String,
) -> Result<(), String> {
    let tramo = recorte.as_ref().map(recorte::tramo).transpose()?;
    // async: la primera vez puede tocar esperar a que se extraigan los binarios.
    let mut hijo = yt_dlp(&app)?
        .args(&args)
        .args(PLANTILLAS)
        .arg("--no-simulate")
        .args(["--", url.trim()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo lanzar yt-dlp: {e}"))?;
    app.state::<Procesos>().0.lock().unwrap().insert(id, hijo.id());
    registrar(&app, &format!("[{id}] empieza {url}"));

    let stdout = hijo.stdout.take().unwrap();
    let stderr = hijo.stderr.take().unwrap();

    // stderr: pasos de ffmpeg y errores. Se guarda entero para el mensaje final.
    let app_err = app.clone();
    let lector_err = thread::spawn(move || {
        let mut todo = Vec::new();
        let mut r = BufReader::new(stderr);
        let mut linea = Vec::new();
        while r.read_until(b'\n', &mut linea).unwrap_or(0) > 0 {
            let texto = String::from_utf8_lossy(&linea);
            if let Some(ev) = interpretar(id, texto.trim_end(), &mut Seguimiento::default()) {
                let _ = app_err.emit("cola", ev);
            }
            todo.extend_from_slice(&linea);
            linea.clear();
        }
        todo
    });

    thread::spawn(move || {
        let mut seg = Seguimiento::default();
        for linea in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(ev) = interpretar(id, &linea, &mut seg) {
                let _ = app.emit("cola", ev);
            }
        }
        let ok = hijo.wait().map(|s| s.success()).unwrap_or(false);
        let stderr = lector_err.join().unwrap_or_default();
        let cancelada = app.state::<Procesos>().0.lock().unwrap().remove(&id).is_none();
        if cancelada {
            registrar(&app, &format!("[{id}] cancelada"));
            return;
        }
        let mut ok = ok;
        let mut error = if ok { String::new() } else { error_de(&stderr) };
        let rutas: Vec<&str> = seg.archivos.iter().map(|a| a.ruta.as_str()).collect();
        if let (true, Some(tramo)) = (ok, tramo) {
            if let Err(e) = recortar(&app, id, &rutas, tramo) {
                ok = false;
                error = e;
            }
        }
        // Al historial va todo lo que llegó a bajarse, aunque la lista fallara a medias.
        let fecha = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let entradas = seg
            .archivos
            .iter()
            .map(|a| Entrada {
                titulo: a.titulo.clone(),
                miniatura: a.miniatura.clone(),
                fecha,
                formato: if a.formato.is_empty() { formato.clone() } else { a.formato.clone() },
                tamano: std::fs::metadata(&a.ruta).ok().map(|m| m.len()),
                ruta: a.ruta.clone(),
            })
            .collect();
        if let Err(e) = historial::anotar(&app, entradas) {
            registrar(&app, &format!("[{id}] no se pudo apuntar en el historial: {e}"));
        }
        let ultimo = seg.archivos.pop();
        let formato = ultimo.as_ref().map(|a| a.formato.clone()).filter(|f| !f.is_empty());
        let ruta = ultimo.map(|a| a.ruta);
        let tamano = ruta.as_ref().and_then(|r| std::fs::metadata(r).ok()).map(|m| m.len());
        match (&ruta, ok) {
            (_, false) => registrar(&app, &format!("[{id}] error: {error}")),
            (Some(r), true) => registrar(&app, &format!("[{id}] hecho: {r}")),
            (None, true) => registrar(&app, &format!("[{id}] hecho")),
        }
        let _ = app.emit("cola", Evento::Fin { id, ok, ruta, tamano, formato, error });
    });
    Ok(())
}

/// Corta con ffmpeg cada archivo bajado (varios si era una lista).
fn recortar(app: &AppHandle, id: u32, rutas: &[&str], tramo: (f64, Option<f64>)) -> Result<(), String> {
    let bin = binarios::carpeta(datos_app(app)?)?;
    for r in rutas {
        let _ = app.emit("cola", Evento::Proceso { id, paso: "Recorte".into() });
        registrar(app, &format!("[{id}] recortando {r}"));
        recorte::cortar(&bin, std::path::Path::new(r), tramo)?;
    }
    Ok(())
}

/// Mata yt-dlp y todo lo que haya lanzado (su propio intérprete y ffmpeg).
fn matar_arbol(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[tauri::command]
pub fn cancelar(procesos: State<Procesos>, id: u32) {
    // Se quita antes de matar: así el hilo lector sabe que no fue un error.
    let pid = procesos.0.lock().unwrap().remove(&id);
    if let Some(pid) = pid {
        matar_arbol(pid);
    }
}

/// Al cerrar la app no deben quedar descargas huérfanas.
pub fn cancelar_todas(app: &AppHandle) {
    let pids: Vec<u32> = app.state::<Procesos>().0.lock().unwrap().drain().map(|(_, p)| p).collect();
    for pid in pids {
        matar_arbol(pid);
    }
}

/// Abre el Explorador con el archivo seleccionado, o la carpeta si no hay archivo.
#[tauri::command]
pub fn mostrar_en_carpeta(ruta: Option<String>, carpeta: String) {
    let mut cmd = Command::new("explorer");
    // Si aún no se ha bajado nada, la carpeta puede no existir.
    let _ = std::fs::create_dir_all(&carpeta);
    match ruta.filter(|r| std::path::Path::new(r).exists()) {
        // explorer no sigue las reglas normales de comillas: hay que montarlas a mano.
        Some(r) => cmd.raw_arg(format!("/select,\"{r}\"")),
        None => cmd.arg(carpeta),
    };
    let _ = cmd.spawn();
}

