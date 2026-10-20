// Avisos propios (ventanita abajo a la derecha, ver Aviso.svelte) en lugar
// de las notificaciones de Windows.
import { invoke } from "@tauri-apps/api/core";
import { ajustes, temaEfectivo, enTauri } from "./estado.svelte.js";
import { tam } from "./formato.js";

function avisar(datos) {
  if (enTauri) invoke("aviso", { datos: { ...datos, tema: temaEfectivo() } }).catch(() => {});
}

// Si estás mirando la app no hace falta avisar de nada.
const mirando = () => document.visibilityState === "visible" && document.hasFocus();

export function descargaHecha(d) {
  if (!ajustes.avisar || mirando()) return;
  avisar({
    tipo: "descarga",
    etiqueta: "HECHO",
    titulo: d.titulo,
    texto: [d.que, tam(d.tamano)].filter(Boolean).join(" · "),
    miniatura: d.miniatura,
    ruta: d.ruta,
    carpeta: ajustes.carpeta,
  });
}

export function videosNuevos(canal, n) {
  if (!ajustes.avisarNuevos) return;
  avisar({
    tipo: "nuevos",
    etiqueta: "SUSCRIPCIONES",
    titulo: canal,
    texto: n + (n === 1 ? " vídeo nuevo, ya en la cola" : " vídeos nuevos, ya en la cola"),
  });
}

export function enLaBandeja() {
  avisar({
    tipo: "bandeja",
    etiqueta: "TUNA-DLP",
    titulo: "sigo aquí, en la bandeja",
    texto: "las descargas siguen. para salir del todo, clic derecho en mi icono.",
  });
}
