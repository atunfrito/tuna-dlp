// Bandeja del sistema y progreso en el icono de la barra de tareas (lo pinta
// Rust sobre el icono: la barra nativa sale con el color de énfasis, a veces gris).
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ajustes, app, form, rutas, enTauri, temaEfectivo } from "./estado.svelte.js";
import { cola, control, activas, progreso, encolar, pausarTodo, reanudarTodo } from "./cola.svelte.js";
import { argumentos, recorte } from "./comando.js";
import { queEs } from "./formato.js";
import { enLaBandeja } from "./avisos.js";

const esUrl = (t) => /^https?:\/\/\S+$/i.test(t);

// Con lo que esté elegido en descargar. Sin url en el portapapeles, abre la ventana.
async function pegarYDescargar() {
  const texto = (await invoke("leer_portapapeles").catch(() => null))?.trim();
  if (!texto || !esUrl(texto)) {
    app.vista = "dl";
    invoke("mostrar_ventana");
    return;
  }
  const corte = ajustes.recortar ? recorte(form) : null;
  encolar([{ url: texto, args: argumentos(form, ajustes, rutas) }], corte, queEs(form), {});
}

function resumen() {
  const n = activas();
  if (n) return n + " descargando · " + Math.floor((progreso() ?? 0) * 100) + " %";
  if (control.pausado && cola.some((d) => d.estado === "pausa")) return "en pausa";
  return "nada descargando";
}

if (enTauri) {
  listen("bandeja", ({ payload }) => {
    if (payload === "pegar") pegarYDescargar();
    else if (payload === "pausa") control.pausado ? reanudarTodo() : pausarTodo();
    else if (payload === "carpeta") invoke("mostrar_en_carpeta", { ruta: null, carpeta: ajustes.carpeta });
  });

  // La primera vez que se cierra la ventana, que se sepa que la app sigue ahí.
  listen("oculta", () => {
    if (ajustes.avisoBandeja) return;
    ajustes.avisoBandeja = true;
    enLaBandeja();
  });

  // Los eventos de progreso llegan muchas veces por segundo: se agrupan.
  let pendiente = null;
  $effect.root(() => {
    $effect(() => {
      const estado = resumen();
      const p = progreso();
      const errores = cola.some((d) => d.estado === "err");
      const status =
        activas() ? (errores ? "error" : "normal")
          : control.pausado && p != null && p < 1 ? "paused"
          : "none";
      const datos = { estado, pausado: control.pausado, status, p, activas: activas(), tema: temaEfectivo() };
      if (pendiente) return void (pendiente.datos = datos);
      pendiente = { datos };
      setTimeout(() => {
        const { datos: d } = pendiente;
        pendiente = null;
        invoke("bandeja", { estado: d.estado, pausado: d.pausado, progreso: d.p, activas: d.activas, tema: d.tema }).catch(() => {});
        invoke("progreso_icono", { estado: d.status, progreso: d.p ?? 0 }).catch(() => {});
      }, 500);
    });
  });
}
