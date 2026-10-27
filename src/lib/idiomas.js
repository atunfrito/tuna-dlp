// Nombres de idioma (en el idioma de la app) para los subtítulos.
import { idioma } from "./i18n.svelte.js";

const cache = {};
const nombres = () => (cache[idioma()] ??= new Intl.DisplayNames([idioma()], { type: "language", fallback: "code" }));

// Los que salen primero cuando aún no se ha escrito nada.
export const COMUNES = ["es", "en", "es-419", "ca", "gl", "eu", "fr", "de", "it", "pt", "ja", "ko"];

// Todos los ISO 639-1, más las variantes que YouTube usa a menudo. Así se
// puede buscar cualquier idioma aunque el vídeo aún no esté pegado.
export const TODOS = [
  ...COMUNES,
  "pt-BR", "en-GB", "fr-CA", "zh-Hans", "zh-Hant",
  "aa", "ab", "af", "ak", "am", "an", "ar", "as", "av", "ay", "az", "ba", "be", "bg", "bi", "bm", "bn", "bo",
  "br", "bs", "ce", "ch", "co", "cr", "cs", "cu", "cv", "cy", "da", "dv", "dz", "ee", "el", "eo", "et", "fa",
  "ff", "fi", "fj", "fo", "fy", "ga", "gd", "gn", "gu", "gv", "ha", "he", "hi", "ho", "hr", "ht", "hu", "hy",
  "hz", "ia", "id", "ie", "ig", "ii", "ik", "io", "is", "iu", "jv", "ka", "kg", "ki", "kj", "kk", "kl", "km",
  "kn", "kr", "ks", "ku", "kv", "kw", "ky", "la", "lb", "lg", "li", "ln", "lo", "lt", "lu", "lv", "mg", "mh",
  "mi", "mk", "ml", "mn", "mr", "ms", "mt", "my", "na", "nb", "nd", "ne", "ng", "nl", "nn", "no", "nr", "nv",
  "ny", "oc", "oj", "om", "or", "os", "pa", "pi", "pl", "ps", "qu", "rm", "rn", "ro", "ru", "rw", "sa", "sc",
  "sd", "se", "sg", "si", "sk", "sl", "sm", "sn", "so", "sq", "sr", "ss", "st", "su", "sv", "sw", "ta", "te",
  "tg", "th", "ti", "tk", "tl", "tn", "to", "tr", "ts", "tt", "tw", "ty", "ug", "uk", "ur", "uz", "ve", "vi",
  "vo", "wa", "wo", "xh", "yi", "yo", "za", "zh", "zu",
];

export function nombre(codigo) {
  try {
    const n = nombres().of(codigo);
    return n === codigo ? codigo : n.toLowerCase();
  } catch {
    return codigo;
  }
}

// "es,en" → "español, inglés"
export const nombresDe = (langs) => langs.split(",").map((c) => c.trim()).filter(Boolean).map(nombre).join(", ");

// "japones" encuentra "japonés": sin tildes ni mayúsculas.
export const normal = (t) => t.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
