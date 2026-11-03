// Actualizar la propia app con tauri-plugin-updater. Solo se instalan paquetes
// firmados con la clave cuya parte pública está en tauri.conf.json: si la firma
// no cuadra, el plugin no instala nada.
import { invoke } from "@tauri-apps/api/core";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { ajustes, enTauri } from "./estado.svelte.js";

// estado: idle | available | downloading | installing | error
export const nueva = $state({ estado: "idle", version: "", bajado: 0, total: 0, error: "", portable: false, cerrado: false });

let pendiente = null;

export async function buscarApp() {
  try {
    nueva.portable = await invoke("es_portable");
    pendiente = await check();
    if (pendiente) {
      nueva.version = pendiente.version;
      nueva.estado = "available";
    }
  } catch {
    // Sin internet, o la release aún no trae latest.json: no se molesta a nadie.
  }
}

export async function instalarApp() {
  // La portable no tiene instalador al lado: se baja a mano de la release.
  if (nueva.portable) return invoke("abrir_release");
  if (!pendiente) return;
  nueva.estado = "downloading";
  nueva.bajado = 0;
  nueva.total = 0;
  try {
    await pendiente.downloadAndInstall((e) => {
      if (e.event === "Started") nueva.total = e.data.contentLength ?? 0;
      else if (e.event === "Progress") nueva.bajado += e.data.chunkLength;
      else if (e.event === "Finished") nueva.estado = "installing";
    });
    await relaunch();
  } catch (e) {
    nueva.error = String(e);
    nueva.estado = "error";
  }
}

if (enTauri && ajustes.buscarActualizaciones) buscarApp();
