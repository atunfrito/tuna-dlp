// Avisos propios (ventanita abajo a la derecha, ver Aviso.svelte) en lugar
// de las notificaciones de Windows.
import { invoke } from "@tauri-apps/api/core";
import { ajustes, temaEfectivo, enTauri } from "./estado.svelte.js";
import { tam } from "./formato.js";
import { t, idioma } from "./i18n.svelte.js";

function avisar(datos) {
  if (enTauri) invoke("aviso", { datos: { ...datos, tema: temaEfectivo(), idioma: idioma() } }).catch(() => {});
}

// Si estás mirando la app no hace falta avisar de nada.
const mirando = () => document.visibilityState === "visible" && document.hasFocus();

export function descargaHecha(d) {
  if (!ajustes.avisar || mirando()) return;
  avisar({
    tipo: "descarga",
    etiqueta: t("aviso.hecho"),
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
    etiqueta: t("aviso.subs"),
    titulo: canal,
    texto: n === 1 ? t("aviso.nuevo") : t("aviso.nuevos", { n }),
  });
}

export function enLaBandeja() {
  avisar({
    tipo: "bandeja",
    etiqueta: "TUNA-DLP",
    titulo: t("aviso.bandeja"),
    texto: t("aviso.bandeja_texto"),
  });
}
