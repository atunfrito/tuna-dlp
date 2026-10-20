import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { untrack } from "svelte";
import { ajustes, enTauri } from "./estado.svelte.js";
import { descargaHecha } from "./avisos.js";

// estado: cola | dl | proc | pausa | done | err
export const cola = $state([]);
export const control = $state({ pausado: false });
let siguienteId = 1;

// Las descargas desde que la cola estuvo vacía por última vez: con ellas se
// calcula el progreso total de la bandeja y la barra de tareas.
let lote = $state([]);

const enMarcha = (d) => d.estado === "dl" || d.estado === "proc";
const pendiente = (d) => enMarcha(d) || d.estado === "cola" || d.estado === "pausa";
export const activas = () => cola.filter(enMarcha).length;

// trabajos: [{ url, args }] (cada url puede llevar sus propios argumentos).
export function encolar(trabajos, recorte, que, previas) {
  if (!cola.some(pendiente)) lote = [];
  for (const { url, args } of trabajos) {
    const info = previas[url]?.info;
    const id = siguienteId++;
    cola.push({
      id, url, args, recorte, que,
      titulo: info?.titulo || url, miniatura: info?.miniatura ?? null,
      estado: "cola", bajado: null, total: null, velocidad: null, eta: null, paso: "",
      ruta: null, tamano: null, error: "",
    });
    lote.push(id);
  }
  bombear();
}

// Arranca lo que haya en cola hasta llenar las descargas en paralelo.
function bombear() {
  if (control.pausado) return;
  let libres = ajustes.paralelo - activas();
  for (const d of cola) {
    if (libres <= 0) break;
    if (d.estado !== "cola") continue;
    Object.assign(d, { estado: "dl", velocidad: null, eta: null, paso: "", error: "" });
    libres--;
    invoke("iniciar", { id: d.id, url: d.url, args: d.args, recorte: d.recorte, formato: d.que }).catch((e) => terminar(d, { ok: false, error: String(e) }));
  }
}

// formato: lo que se ha bajado de verdad ("mp4 240p"), que puede no ser lo pedido.
function terminar(d, { ok, ruta = null, tamano = null, formato = null, error = "" }) {
  Object.assign(d, { estado: ok ? "done" : "err", ruta, tamano, error });
  if (formato) d.que = formato;
  if (ok && ajustes.abrirAlTerminar) abrir(d);
  if (ok) descargaHecha(d);
  bombear();
}

export function cancelar(d) {
  invoke("cancelar", { id: d.id });
  cola.splice(cola.indexOf(d), 1);
  bombear();
}

export function reintentar(d) {
  Object.assign(d, { estado: "cola", bajado: null, total: null });
  bombear();
}

// yt-dlp no sabe pausar: se para y, al reanudar, sigue desde los .part.
export function pausarTodo() {
  control.pausado = true;
  for (const d of cola) {
    if (!enMarcha(d)) continue;
    invoke("cancelar", { id: d.id });
    Object.assign(d, { estado: "pausa", velocidad: null, eta: null });
  }
}

export function reanudarTodo() {
  control.pausado = false;
  for (const d of cola) if (d.estado === "pausa") d.estado = "cola";
  bombear();
}

export function abrir(d) {
  invoke("mostrar_en_carpeta", { ruta: d.ruta, carpeta: ajustes.carpeta });
}

export function limpiarTerminadas() {
  for (let i = cola.length - 1; i >= 0; i--) if (cola[i].estado === "done") cola.splice(i, 1);
}

// Progreso total del lote, de 0 a 1 (null si no hay nada).
export function progreso() {
  const ds = lote.map((id) => cola.find((d) => d.id === id)).filter(Boolean);
  if (!ds.length) return null;
  const parte = (d) =>
    d.estado === "done" || d.estado === "err" || d.estado === "proc" ? 1
      : d.total ? Math.min(1, d.bajado / d.total)
      : 0;
  return ds.reduce((s, d) => s + parte(d), 0) / ds.length;
}

// Si cambian las descargas en paralelo, que entren las que esperan.
$effect.root(() => {
  $effect(() => {
    ajustes.paralelo;
    untrack(bombear);
  });
});

if (enTauri) {
  listen("cola", ({ payload: ev }) => {
    const d = cola.find((x) => x.id === ev.id);
    // Tras pausar aún pueden llegar líneas de un proceso que se está cerrando.
    if (!d || d.estado === "pausa") return;
    if (ev.tipo === "titulo") {
      if (d.titulo === d.url) d.titulo = ev.titulo;
      d.miniatura ??= ev.miniatura;
    } else if (ev.tipo === "progreso") {
      Object.assign(d, { estado: "dl", bajado: ev.bajado, total: ev.total, velocidad: ev.velocidad, eta: ev.eta });
    } else if (ev.tipo === "proceso") {
      Object.assign(d, { estado: "proc", paso: ev.paso });
    } else if (ev.tipo === "fin") {
      terminar(d, ev);
    }
  });
}
