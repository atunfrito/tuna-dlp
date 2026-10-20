import { invoke } from "@tauri-apps/api/core";
import { SvelteSet } from "svelte/reactivity";
import { ajustes, rutas, enTauri, CAMPOS_PRESET } from "./estado.svelte.js";
import { cola, encolar } from "./cola.svelte.js";
import { argumentos, PLANTILLA, PLANTILLA_VIEJA } from "./comando.js";
import { queEs } from "./formato.js";
import { videosNuevos } from "./avisos.js";

// Canales que se están comprobando ahora mismo (no se guarda).
export const ocupados = new SvelteSet();

// Nombre de carpeta válido en Windows.
const carpetaDe = (nombre) => nombre.replace(/[<>:"/\\|?*\x00-\x1f]/g, "_").replace(/[. ]+$/, "") || "canal";

// Cada canal se queda con lo que estaba elegido en descargar al añadirlo.
export async function seguir(url, form) {
  const c = await invoke("seguir_canal", { url });
  const opciones = Object.fromEntries(CAMPOS_PRESET.map((k) => [k, form[k]]));
  opciones.plantilla = ajustes.plantilla ? form.plantilla : PLANTILLA;
  opciones.playlist = false;
  ajustes.canales.push({ ...c, opciones, nuevos: 0, error: "" });
}

export function quitar(canal) {
  invoke("dejar_canal", { id: canal.id }).catch(() => {});
  ajustes.canales.splice(ajustes.canales.indexOf(canal), 1);
}

export async function comprobar(canal) {
  if (ocupados.has(canal.id)) return;
  ocupados.add(canal.id);
  try {
    const nuevos = await invoke("novedades", { id: canal.id, url: canal.url });
    // Lo que ya está en la cola (de una comprobación anterior) no se repite.
    const pendientes = nuevos.filter((n) => !cola.some((d) => d.url === n.url && d.estado !== "err"));
    Object.assign(canal, { nuevos: nuevos.length, error: "" });
    if (!pendientes.length) return;

    const opciones = { ...canal.opciones, start: "", end: "" };
    if (opciones.plantilla === PLANTILLA_VIEJA) opciones.plantilla = PLANTILLA;
    const suyos = { ...ajustes, carpeta: ajustes.carpeta + "\\" + carpetaDe(canal.nombre), plantilla: true, archivo: false };
    const args = [...argumentos(opciones, suyos, rutas), "--download-archive", canal.archivo];
    const previas = Object.fromEntries(pendientes.map((n) => [n.url, { info: n }]));
    encolar(pendientes.map((n) => ({ url: n.url, args })), null, queEs(opciones), previas);
    videosNuevos(canal.nombre, pendientes.length);
  } catch (e) {
    canal.error = String(e);
  } finally {
    ocupados.delete(canal.id);
  }
}

export const comprobarTodos = () => Promise.all(ajustes.canales.map(comprobar));

// "comprobar al abrir la app"
if (enTauri && ajustes.suscripciones && ajustes.comprobarAlAbrir) comprobarTodos();
