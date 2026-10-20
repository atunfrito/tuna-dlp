// Tamaños, fechas y resúmenes a la española, para la cola y el historial.

// 8,2 MiB · 1,2 GiB
export function tam(bytes) {
  if (bytes == null) return "";
  const mib = bytes / 1048576;
  if (mib >= 1024) return (mib / 1024).toLocaleString("es-ES", { maximumFractionDigits: 1 }) + " GiB";
  return mib.toLocaleString("es-ES", { maximumFractionDigits: mib < 100 ? 1 : 0 }) + " MiB";
}

// hoy · ayer · hace 3 días · 12 mar 2026 (por días de calendario, no de 24 h)
export function cuando(segundos) {
  const dia = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const fecha = new Date(segundos * 1000);
  const dias = Math.round((dia(new Date()) - dia(fecha)) / 86400000);
  if (dias <= 0) return "hoy";
  if (dias === 1) return "ayer";
  if (dias < 30) return "hace " + dias + " días";
  return fecha.toLocaleDateString("es-ES", { day: "numeric", month: "short", year: "numeric" }).replace(".", "");
}

// "mp4 1080p", "mp3", "subtítulos": lo que se va a bajar, en corto.
export function queEs(o) {
  if (o.mode === "audio") return o.afmt;
  if (o.mode === "subs") return "subtítulos";
  return o.container + " " + (o.quality === "best" ? "máxima" : o.quality + "p");
}
