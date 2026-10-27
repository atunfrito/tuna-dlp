<script>
  import { invoke } from "@tauri-apps/api/core";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import { ajustes } from "../lib/estado.svelte.js";
  import { historial } from "../lib/historial.svelte.js";
  import { normal } from "../lib/idiomas.js";
  import { tam, cuando } from "../lib/formato.js";
  import { t } from "../lib/i18n.svelte.js";

  let busca = $state("");
  const encontradas = $derived.by(() => {
    const q = normal(busca.trim());
    return q ? historial.entradas.filter((e) => normal(e.titulo).includes(q)) : historial.entradas;
  });
  // Con miles de descargas, pintar todas no aporta nada: se busca.
  const MAX = 300;

  const abrir = (e) => invoke("mostrar_en_carpeta", { ruta: e.ruta, carpeta: ajustes.carpeta });
</script>

<div class="vista">
  <div class="cabecera">
    <h1>{t("hist.titulo")}<span class="acento">.</span></h1>
    {#if historial.entradas.length}
      <label class="buscador">
        <Search size={16} strokeWidth={2} />
        <input bind:value={busca} placeholder={t("hist.buscar")} spellcheck="false" />
        {#if busca}
          <button type="button" class="borrar" aria-label={t("hist.borrar")} onclick={() => (busca = "")}>
            <X size={14} strokeWidth={2.25} />
          </button>
        {/if}
      </label>
    {/if}
  </div>

  {#if !historial.entradas.length}
    <div class="vacio">{t("hist.vacio")}</div>
  {:else}
    <section class="papel">
      {#each encontradas.slice(0, MAX) as e (e.fecha + e.ruta)}
        <div class="fila">
          {#if e.miniatura}<img src={e.miniatura} alt="" loading="lazy" />{:else}<span class="hueco-img"></span>{/if}
          <div class="textos">
            <span class="titulo" title={e.titulo}>{e.titulo}</span>
            <span class="meta">{[cuando(e.fecha), e.formato, tam(e.tamano)].filter(Boolean).join(" · ")}</span>
          </div>
          <button type="button" class="boton" onclick={() => abrir(e)}>{t("abrir_carpeta")}</button>
        </div>
      {:else}
        <div class="vacia">{t("hist.nada")}</div>
      {/each}
    </section>
  {/if}
</div>

<style>
  .vista { position: relative; padding: 30px 36px 36px; display: flex; flex-direction: column; gap: 18px; }
  .cabecera { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; flex-wrap: wrap; }
  h1 { margin: 0; font-family: var(--serif); font-weight: 400; font-size: 44px; line-height: 1; letter-spacing: -0.022em; }
  .acento { color: var(--accent); }
  /* Una tira de papel como las tarjetas, con la lupa dentro. */
  .buscador {
    width: 320px; height: 40px; display: flex; align-items: center; gap: 9px;
    padding: 0 10px 0 13px; border-radius: var(--r);
    background: var(--paper); color: var(--paper-dim);
    box-shadow: var(--shadow), 0 0 0 1px transparent;
    cursor: text; transition: box-shadow 0.2s ease, color 0.2s ease;
  }
  .buscador:focus-within { color: var(--accent); box-shadow: var(--shadow), 0 0 0 1.5px var(--accent); }
  .buscador input {
    flex: 1 1 auto; min-width: 0; height: 100%; padding: 0;
    border: none; outline: none; background: transparent; color: var(--paper-fg);
    font-size: 14px;
  }
  .borrar {
    flex: 0 0 auto; width: 22px; height: 22px; padding: 0;
    display: flex; align-items: center; justify-content: center;
    border: none; border-radius: 50%; background: rgba(37, 31, 25, 0.08); color: var(--paper-dim);
    cursor: pointer; transition: background-color 0.15s ease, color 0.15s ease;
  }
  .borrar:hover { background: var(--accent); color: var(--accent-fg); }
  .vacio { padding: 48px 0; text-align: center; font-family: var(--hand); font-size: 24px; color: var(--dim); }

  .papel {
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); box-shadow: var(--shadow); padding: 6px 18px;
  }
  .fila { display: flex; align-items: center; gap: 14px; padding: 12px 0; border-bottom: 1px solid var(--paper-rule); }
  .fila:last-child { border-bottom: none; }
  img, .hueco-img {
    width: 80px; height: 45px; flex: 0 0 auto; display: block;
    object-fit: cover; border-radius: 3px; background: rgba(37, 31, 25, 0.08);
  }
  .textos { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .titulo { font-size: 14.5px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .boton {
    flex: 0 0 auto; padding: 8px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.18); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .boton:hover { background: rgba(37, 31, 25, 0.05); }
  .vacia { padding: 28px 0; text-align: center; font-family: var(--hand); font-size: 23px; color: rgba(37, 31, 25, 0.55); }
</style>
