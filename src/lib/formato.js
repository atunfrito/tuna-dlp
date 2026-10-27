// Tamaños, fechas y resúmenes en el idioma de la app, para la cola y el historial.
import { t, locale } from "./i18n.svelte.js";

// 8,2 MiB · 1,2 GiB
export function tam(bytes) {
  if (bytes == null) return "";
  const mib = bytes / 1048576;
  if (mib >= 1024) return (mib / 1024).toLocaleString(locale(), { maximumFractionDigits: 1 }) + " GiB";
  return mib.toLocaleString(locale(), { maximumFractionDigits: mib < 100 ? 1 : 0 }) + " MiB";
}

// hoy · ayer · hace 3 días · 12 mar 2026 (por días de calendario, no de 24 h)
export function cuando(segundos) {
  const dia = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const fecha = new Date(segundos * 1000);
  const dias = Math.round((dia(new Date()) - dia(fecha)) / 86400000);
  if (dias <= 0) return t("fecha.hoy");
  if (dias === 1) return t("fecha.ayer");
  if (dias < 30) return t("fecha.hace", { n: dias });
  return fecha.toLocaleDateString(locale(), { day: "numeric", month: "short", year: "numeric" }).replace(".", "");
}

// "mp4 1080p", "mp3", "subtítulos": lo que se va a bajar, en corto.
export function queEs(o) {
  if (o.mode === "audio") return o.afmt;
  if (o.mode === "subs") return t("modo.subs");
  return o.container + " " + (o.quality === "best" ? t("dl.maxima") : o.quality + "p");
}
