import { invoke } from "@tauri-apps/api/core";
import { PLANTILLA } from "./comando.js";

// Lo que Rust inyecta antes de cargar la página (ver src-tauri/src/lib.rs).
const inicio = window.__TUNA__ ?? { ajustes: {}, rutas: { descargas: "", archivo: "descargados.txt" } };

export const rutas = inicio.rutas;
export const enTauri = "__TAURI_INTERNALS__" in window;

const AJUSTES = {
  tema: "light",
  // "system" sigue al idioma de Windows (ver lib/textos.js).
  idioma: "system",
  carpeta: rutas.descargas,
  paralelo: 2,
  velocidad: "sin límite",
  navegador: "ninguno",
  canal: "stable",
  portapapeles: true,
  avisar: true,
  abrirAlTerminar: false,
  archivo: true,
  menuPlegado: false,
  // Funciones opcionales: apagadas por defecto para no liar.
  verComando: false,
  recortar: false,
  plantilla: false,
  suscripciones: false,
  historial: false,
  // Presets propios: [{ nombre, valores }], guardados con el resto de ajustes.
  presets: [],
  // Suscripciones: [{ id, nombre, url, archivo, opciones, nuevos, error }]
  canales: [],
  comprobarAlAbrir: true,
  avisarNuevos: true,
  // Arrancar con Windows, escondida en la bandeja.
  abrirConWindows: false,
  // Ya se avisó una vez de que cerrar la ventana la deja en la bandeja.
  avisoBandeja: false,
};

export const ajustes = $state({ ...AJUSTES, ...inicio.ajustes });
// upd: idle | checking | available | updating | ok | error
export const app = $state({ vista: "dl", ytv: "", ffmpeg: "", upd: "idle", ultima: "", updError: "" });

// Se guardan solos, agrupando cambios seguidos en una sola escritura.
let pendiente;
$effect.root(() => {
  $effect(() => {
    // JSON.stringify recorre el proxy y registra cada campo como dependencia;
    // $state.snapshot no lo hace y el efecto no volvería a saltar.
    const copia = JSON.parse(JSON.stringify(ajustes));
    clearTimeout(pendiente);
    pendiente = setTimeout(() => invoke("guardar_ajustes", { ajustes: copia }).catch(() => {}), 300);
  });
});

// La entrada de arranque de Windows sigue al ajuste. También al abrir la app,
// por si el .exe (versión portable) se ha movido de sitio.
if (enTauri) {
  $effect.root(() => {
    $effect(() => {
      invoke("abrir_con_windows", { activar: ajustes.abrirConWindows }).catch(() => {});
    });
  });
}

// "sistema" sigue al tema de Windows mientras la app está abierta.
const oscuroSistema = matchMedia("(prefers-color-scheme: dark)");
export function temaEfectivo() {
  if (ajustes.tema === "light" || ajustes.tema === "dark") return ajustes.tema;
  return oscuroSistema.matches ? "dark" : "light";
}
$effect.root(() => {
  $effect(() => {
    const aplicar = () => (document.documentElement.dataset.theme = temaEfectivo());
    aplicar();
    oscuroSistema.addEventListener("change", aplicar);
    return () => oscuroSistema.removeEventListener("change", aplicar);
  });
});

// --- yt-dlp: versión y actualizaciones -----------------------------------------
function leerVersiones() {
  return invoke("versiones").then((v) => { app.ytv = v.ytdlp; app.ffmpeg = v.ffmpeg; }, () => {});
}

export async function comprobar() {
  app.upd = "checking";
  try {
    const [ultima] = await Promise.all([invoke("ultima_version", { canal: ajustes.canal }), leerVersiones()]);
    app.ultima = ultima;
    app.upd = ultima !== app.ytv ? "available" : "idle";
  } catch (e) {
    app.updError = String(e);
    app.upd = "error";
  }
}

export async function actualizar() {
  app.upd = "updating";
  try {
    app.ytv = await invoke("actualizar_ytdlp", { canal: ajustes.canal });
    app.upd = "ok";
  } catch (e) {
    app.updError = String(e);
    app.upd = "error";
  }
}

// Al arrancar se mira si hay yt-dlp nuevo, para el punto rojo de "ajustes".
if (enTauri) leerVersiones().then(comprobar);

// Opciones de la vista descargar. Viven aquí para no perderse al cambiar de vista.
export const form = $state({
  urls: "", preset: "vídeo",
  mode: "video", quality: "1080", container: "mp4", codec: "h264",
  afmt: "mp3", abr: "best", langs: "es,en", start: "", end: "",
  plantilla: PLANTILLA,
  thumb: true, meta: true, chapters: true, sponsor: true, subs: false, playlist: false,
});

// Lo que guarda "+ guardar": modo, calidad, contenedor, códec, audio, extras, idiomas y plantilla.
export const CAMPOS_PRESET = [
  "mode", "quality", "container", "codec", "afmt", "abr",
  "thumb", "meta", "chapters", "sponsor", "subs", "playlist", "langs", "plantilla",
];

export const PRESETS = {
  música: { mode: "audio", afmt: "mp3", abr: "best", thumb: true, meta: true, chapters: false, sponsor: false, subs: false },
  vídeo: { mode: "video", quality: "1080", container: "mp4", codec: "h264", thumb: true, meta: true, chapters: true, sponsor: true },
  archivo: { mode: "video", quality: "best", container: "mkv", codec: "any", thumb: true, meta: true, chapters: true, subs: true, sponsor: false },
  max: { mode: "video", quality: "2160", container: "mkv", codec: "av01", thumb: true, meta: true, chapters: true },
};
