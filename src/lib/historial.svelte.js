import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { enTauri } from "./estado.svelte.js";

// Lo más nuevo primero. Rust avisa con el evento "historial" al terminar cada
// descarga, así que no hace falta volver a leer el archivo.
export const historial = $state({ entradas: [] });

if (enTauri) {
  invoke("historial").then((e) => (historial.entradas = e.reverse()), () => {});
  listen("historial", ({ payload }) => historial.entradas.unshift(...payload.reverse()));
}
