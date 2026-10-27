<script>
  import { cola, cancelar, reintentar, reanudarTodo, abrir, limpiarTerminadas } from "../lib/cola.svelte.js";
  import { tam } from "../lib/formato.js";
  import Fallo from "../lib/Fallo.svelte";
  import { t } from "../lib/i18n.svelte.js";

  // [color del texto, fondo] de la etiqueta de cada estado.
  const COLORES = {
    cola: ["var(--paper-dim)", "rgba(37, 31, 25, 0.07)"],
    dl: ["#c8102e", "#fbe3e6"],
    proc: ["#8a5a12", "#f6e6b8"],
    pausa: ["#8a5a12", "#f6e6b8"],
    done: ["#2f7a4c", "#dcefe2"],
    err: ["#fff", "#c8102e"],
  };

  function tiempo(s) {
    s = Math.round(s);
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), seg = s % 60;
    const dos = (n) => String(n).padStart(2, "0");
    return (h ? h + ":" + dos(m) : dos(m)) + ":" + dos(seg);
  }
  const pct = (d) => (d.total ? Math.min(100, (d.bajado / d.total) * 100) : 0);

  function meta(d) {
    if (d.estado === "done") return [t("estado.done"), tam(d.tamano), d.que].filter(Boolean).join(" · ");
    if (d.estado === "proc") {
      return d.paso === "Merger" ? t("cola.uniendo") : t("cola.procesando");
    }
    if (d.estado === "dl" && d.total) {
      return [
        Math.floor(pct(d)) + " %",
        d.velocidad != null && tam(d.velocidad) + "/s",
        d.eta != null && t("cola.quedan", { tiempo: tiempo(d.eta) }),
        tam(d.total),
      ].filter(Boolean).join(" · ");
    }
    return d.que;
  }

  function barra(d) {
    const ancho = d.estado === "done" || d.estado === "proc" ? 100 : d.estado === "err" ? 0 : pct(d);
    const color = d.estado === "done" ? "var(--ok)" : "var(--accent)";
    return `width: ${ancho}%; background: ${color}`;
  }

  function accion(d) {
    if (d.estado === "done") return [t("abrir_carpeta"), () => abrir(d)];
    if (d.estado === "err") return [t("cola.reintentar"), () => reintentar(d)];
    if (d.estado === "pausa") return [t("cola.reanudar"), reanudarTodo];
    return [t("cola.cancelar"), () => cancelar(d)];
  }
</script>

<div class="vista">
  <div class="cabecera">
    <h1>{t("cola.titulo")}<span class="acento">.</span></h1>
    <button type="button" class="limpiar" onclick={limpiarTerminadas}>{t("cola.limpiar")}</button>
  </div>

  {#each cola as d (d.id)}
    {@const [fg, bg] = COLORES[d.estado]}
    {@const tag = t("estado." + d.estado)}
    {@const [etiqueta, hacer] = accion(d)}
    <div class="fila">
      {#if d.miniatura}
        <img src={d.miniatura} alt="" />
      {:else}
        <span class="hueco-img"></span>
      {/if}
      <div class="centro">
        <div class="arriba">
          <span class="titulo" title={d.titulo}>{d.titulo}</span>
          <span class="tag" style="color: {fg}; background: {bg}">{tag}</span>
        </div>
        <div class="pista"><div class="relleno" style={barra(d)}></div></div>
        {#if d.estado === "err"}
          <Fallo error={d.error} />
        {:else}
          <span class="meta">{meta(d)}</span>
        {/if}
      </div>
      <button type="button" class="accion" onclick={hacer}>{etiqueta}</button>
    </div>
  {:else}
    <div class="vacia">{t("cola.vacia")}</div>
  {/each}
</div>

<style>
  .vista { position: relative; padding: 30px 36px 36px; display: flex; flex-direction: column; gap: 18px; }
  .cabecera { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; }
  h1 { margin: 0; font-family: var(--serif); font-weight: 400; font-size: 44px; line-height: 1; letter-spacing: -0.022em; }
  .acento { color: var(--accent); }
  .limpiar {
    padding: 8px 12px; border-radius: 4px;
    border: 1px solid var(--line); background: transparent; color: var(--fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .limpiar:hover { background: var(--hover); }

  .fila {
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); box-shadow: var(--shadow);
    padding: 14px 16px; display: flex; align-items: center; gap: 16px;
  }
  img, .hueco-img {
    width: 112px; height: 63px; flex: 0 0 auto; display: block;
    object-fit: cover; border-radius: 4px; background: rgba(37, 31, 25, 0.08);
  }
  .centro { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 7px; }
  .arriba { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .titulo {
    flex: 1 1 auto; min-width: 0;
    font-size: 15px; font-weight: 700;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .tag {
    flex: 0 0 auto; padding: 3px 7px; border-radius: 3px;
    font-family: var(--mono); font-size: 10.5px; font-weight: 600;
  }
  .pista { height: 3px; border-radius: 2px; background: rgba(37, 31, 25, 0.1); overflow: hidden; }
  .relleno { height: 100%; border-radius: 2px; transition: width 0.4s ease; }
  .meta { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .accion {
    flex: 0 0 auto; padding: 8px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.18); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .accion:hover { background: rgba(37, 31, 25, 0.05); }

  .vacia { padding: 40px 0; text-align: center; font-family: var(--hand); font-size: 24px; color: var(--dim); }
</style>
