// Idioma de la ventana principal: el elegido en ajustes, o el de Windows.
import { ajustes } from "./estado.svelte.js";
import { idiomaDe, traducir, traducirError } from "./textos.js";

export const idioma = () => idiomaDe(ajustes.idioma);

// t("clave", { var }) en cualquier sitio de la ventana principal. Lee
// ajustes.idioma, así que lo que la use se repinta al cambiar de idioma.
export const t = (clave, vars) => traducir(idioma(), clave, vars);
export const tError = (mensaje) => traducirError(idioma(), mensaje);

// Formato de números y fechas del idioma.
export const locale = () => (idioma() === "es" ? "es-ES" : "en-US");

$effect.root(() => {
  $effect(() => {
    document.documentElement.lang = idioma();
  });
});
