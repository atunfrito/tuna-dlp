// Montaje de los argumentos de yt-dlp. Es el build() del prototipo, pero
// devuelve un array: Rust se lo pasa al proceso tal cual, sin shell.

// Plantilla de nombres cuando la función está apagada: solo el título.
export const PLANTILLA = "%(title)s.%(ext)s";
// La de antes, con el id entre corchetes. Los canales guardados con ella pasan a la nueva.
export const PLANTILLA_VIEJA = "%(title)s [%(id)s].%(ext)s";

export const PLANTILLAS = [
  ["%(title)s.%(ext)s", "plantilla.titulo"],
  ["%(uploader)s/%(title)s.%(ext)s", "plantilla.canal"],
  ["%(upload_date)s - %(title)s.%(ext)s", "plantilla.fecha"],
  ["%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s", "plantilla.lista"],
];

// Qué vídeos de la playlist bajar: [true, false, true, true] → "1,3:4".
// null si van todos (no hace falta -I).
export function indices(marcadas) {
  if (marcadas.every(Boolean)) return null;
  const tramos = [];
  marcadas.forEach((on, i) => {
    if (!on) return;
    const ultimo = tramos.at(-1);
    if (ultimo && ultimo[1] === i) ultimo[1] = i + 1;
    else tramos.push([i + 1, i + 1]);
  });
  return tramos.map(([a, b]) => (a === b ? a : a + ":" + b)).join(",");
}

// seleccion: la de indices(), solo para la url cuya lista se ha previsualizado.
export function argumentos(o, ajustes, rutas, seleccion = null) {
  const a = [];
  if (o.mode === "audio") {
    a.push("-x", "--audio-format", o.afmt, "--audio-quality", o.abr === "best" ? "0" : o.abr + "K");
  } else if (o.mode === "subs") {
    // Con los dos, yt-dlp prefiere los del autor y tira de los automáticos si no hay.
    a.push("--skip-download", "--write-subs", "--write-auto-subs", "--sub-langs", o.langs);
  } else {
    const sort = [];
    if (o.quality !== "best") sort.push("res:" + o.quality);
    if (o.codec !== "any") sort.push("vcodec:" + o.codec);
    a.push("-f", "bv*+ba/b");
    if (sort.length) a.push("-S", sort.join(","));
    a.push("--merge-output-format", o.container);
  }
  if (o.thumb) a.push("--embed-thumbnail");
  if (o.meta) a.push("--embed-metadata");
  if (o.chapters && o.mode !== "subs") a.push("--embed-chapters");
  if (o.sponsor) a.push("--sponsorblock-remove", "sponsor,selfpromo");
  if (o.subs && o.mode === "video") a.push("--embed-subs", "--write-auto-subs", "--sub-langs", o.langs);
  a.push(o.playlist ? "--yes-playlist" : "--no-playlist");
  if (o.playlist && seleccion) a.push("-I", seleccion);
  if (ajustes.archivo) a.push("--download-archive", rutas.archivo);
  if (ajustes.velocidad !== "sin límite") a.push("-r", ajustes.velocidad.replace(" MB/s", "M"));
  if (ajustes.navegador !== "ninguno") a.push("--cookies-from-browser", ajustes.navegador);
  // La carpeta va en -P y el nombre en -o, sin mezclarlos.
  a.push("--windows-filenames", "-P", ajustes.carpeta, "-o", (ajustes.plantilla && o.plantilla.trim()) || PLANTILLA);
  return a;
}

// "quedará así →": la plantilla con los datos del vídeo. yt-dlp pone NA en lo
// que no sabe; aquí igual, para que la vista previa no engañe.
export function vistaPrevia(plantilla, campos) {
  return plantilla.replace(/%\((\w+)\)[-#0+ ]*\d*(?:\.\d+)?[sdf]/g, (_, k) => campos[k] ?? "NA");
}

// El recorte no va a yt-dlp (--download-sections da archivos rotos): Rust
// descarga entero y luego corta con ffmpeg.
export function recorte(o) {
  const desde = o.start.trim(), hasta = o.end.trim();
  return o.mode !== "subs" && (desde || hasta) ? { desde, hasta } : null;
}

// Para enseñarlo: comillas solo donde la terminal las necesitaría.
export function comoTexto(args, url, corte = null, archivo = "ARCHIVO") {
  const q = (v) => (/^[\w.,:/=+@%-]+$/.test(v) ? v : '"' + v.replace(/"/g, '\\"') + '"');
  const texto = ["yt-dlp", ...args, url || "URL"].map(q).join(" ");
  if (!corte) return texto;
  const tramo = ["-ss", corte.desde || "0", ...(corte.hasta ? ["-to", corte.hasta] : [])];
  return texto + " && ffmpeg " + tramo.join(" ") + " -i " + archivo + " …";
}
