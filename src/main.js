import { mount } from "svelte";
import "@fontsource/instrument-serif/latin-400.css";
import "@fontsource/archivo/latin-400.css";
import "@fontsource/archivo/latin-600.css";
import "@fontsource/archivo/latin-700.css";
import "@fontsource/jetbrains-mono/latin-400.css";
import "@fontsource/jetbrains-mono/latin-600.css";
import "@fontsource/caveat/latin-500.css";
import "./app.css";

// Que parezca una app y no una web: sin menú contextual del navegador
// (salvo en los campos de texto, para poder pegar) ni atajos de recarga.
document.addEventListener("contextmenu", (e) => {
  if (!e.target.closest("input, textarea")) e.preventDefault();
});
if (import.meta.env.PROD) {
  document.addEventListener("keydown", (e) => {
    const k = e.key.toLowerCase();
    if (k === "f5" || k === "f7" || (e.ctrlKey && ["r", "p", "f", "g", "u", "j"].includes(k))) e.preventDefault();
  });
}

// Las ventanitas de avisos y del menú de la bandeja cargan solo lo suyo: si importara App, también
// cargaría el estado y acabaría pisando los ajustes guardados.
const t = window.__TUNA__;
const vista = t?.aviso ? import("./Aviso.svelte") : t?.menu ? import("./Menu.svelte") : import("./App.svelte");
vista.then(({ default: Vista }) => mount(Vista, { target: document.getElementById("app") }));
