<script>
  import Interruptor from "../lib/Interruptor.svelte";
  import { ajustes, form } from "../lib/estado.svelte.js";
  import { ocupados, seguir, quitar, comprobar } from "../lib/canales.svelte.js";
  import { t, tError } from "../lib/i18n.svelte.js";

  let url = $state("");
  let anadiendo = $state(false);
  let fallo = $state("");

  async function anadir() {
    if (!url.trim() || anadiendo) return;
    anadiendo = true;
    fallo = "";
    try {
      await seguir(url, form);
      url = "";
    } catch (e) {
      fallo = String(e);
    } finally {
      anadiendo = false;
    }
  }

  const estado = (c) => (c.error ? t("subs.error") : c.nuevos === 1 ? t("subs.nuevo") : c.nuevos ? t("subs.nuevos", { n: c.nuevos }) : t("subs.al_dia"));
  const corta = (u) => u.replace(/^https?:\/\/(www\.)?/, "").replace(/\/videos$/, "");
</script>

<div class="vista">
  <div class="intro">
    <h1>{t("subs.titulo")}<span class="acento">.</span></h1>
    <span class="entradilla">{t("subs.entradilla")}</span>
  </div>

  <section class="papel">
    <span class="cinta"></span>
    <span class="etiqueta">{t("subs.anadir_canal")}</span>
    <form class="anadir" onsubmit={(e) => { e.preventDefault(); anadir(); }}>
      <input class="campo" bind:value={url} placeholder={t("subs.ejemplo")} spellcheck="false" disabled={anadiendo} />
      <button type="submit" class="primario" disabled={anadiendo || !url.trim()}>{t("subs.anadir")}</button>
    </form>
    {#if anadiendo}<span class="barra-carga"></span>{/if}
    {#if fallo}<span class="fallo" data-seleccionable>{tError(fallo)}</span>{/if}
  </section>

  <section class="papel lista">
    {#each ajustes.canales as c (c.id)}
      <div class="canal">
        <div class="textos">
          <span class="nombre">{c.nombre}</span>
          <span class="meta" class:error={c.error} title={tError(c.error) || null}>{corta(c.url)} · {estado(c)}</span>
        </div>
        <button type="button" class="boton" disabled={ocupados.has(c.id)} onclick={() => comprobar(c)}>{t("subs.comprobar")}</button>
        <button type="button" class="boton quitar" onclick={() => quitar(c)}>{t("subs.quitar")}</button>
      </div>
    {:else}
      <div class="vacia">{t("subs.vacia")}</div>
    {/each}
  </section>

  <section class="papel interruptores">
    <Interruptor borde="abajo" bind:valor={ajustes.comprobarAlAbrir} etiqueta={t("subs.al_abrir")} pista={t("subs.al_abrir_pista")} />
    <Interruptor borde="ninguno" bind:valor={ajustes.avisarNuevos} etiqueta={t("subs.avisar")} pista={t("aviso_pista")} />
  </section>
</div>

<style>
  .vista { position: relative; padding: 30px 36px 36px; display: flex; flex-direction: column; gap: 18px; max-width: 820px; }
  .intro { display: flex; flex-direction: column; gap: 8px; }
  h1 { margin: 0; font-family: var(--serif); font-weight: 400; font-size: 44px; line-height: 1; letter-spacing: -0.022em; }
  .acento { color: var(--accent); }
  .entradilla { font-size: 15px; color: var(--dim); }

  .papel {
    position: relative;
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); box-shadow: var(--shadow);
    padding: 18px 20px; display: flex; flex-direction: column; gap: 12px;
  }
  .cinta {
    position: absolute; top: -11px; right: 44px; width: 84px; height: 24px;
    background: var(--tape); opacity: 0.85; transform: rotate(3deg);
  }
  .etiqueta { font-family: var(--mono); font-size: 10.5px; letter-spacing: 0.16em; color: var(--paper-dim); }
  .anadir { display: flex; gap: 8px; }
  .campo {
    flex: 1 1 auto; min-width: 0;
    font-family: var(--mono); font-size: 13px; padding: 8px 11px;
    border-radius: 4px; border: 1px solid var(--paper-line);
    background: var(--paper-field); color: var(--paper-fg); outline: none;
    transition: border-color 0.2s ease;
  }
  .campo:focus { border-color: var(--accent); }
  .primario {
    padding: 9px 16px; border-radius: 4px; border: none;
    background: var(--accent); color: var(--accent-fg);
    font-size: 13.5px; font-weight: 700; cursor: pointer;
    transition: transform 0.2s ease, opacity 0.2s ease;
  }
  .primario:hover:not(:disabled) { transform: translateY(-1px); }
  .primario:disabled { opacity: 0.5; cursor: default; }
  .barra-carga { display: block; height: 3px; width: 40%; border-radius: 2px; background: var(--accent); animation: ir 1.1s ease-in-out infinite alternate; }
  @keyframes ir { to { transform: translateX(150%); } }
  .fallo { font-family: var(--mono); font-size: 11.5px; color: var(--accent); white-space: pre-wrap; }

  .lista { padding: 6px 20px; gap: 0; }
  .canal { display: flex; align-items: center; gap: 14px; padding: 13px 0; border-bottom: 1px solid var(--paper-rule); }
  .canal:last-child { border-bottom: none; }
  .textos { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .nombre { font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .meta.error { color: var(--accent); }
  .boton {
    flex: 0 0 auto; padding: 8px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.18); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease, color 0.2s ease, opacity 0.2s ease;
  }
  .boton:hover:not(:disabled) { background: rgba(37, 31, 25, 0.05); }
  .boton:disabled { opacity: 0.45; cursor: default; }
  .quitar { border-color: transparent; color: rgba(37, 31, 25, 0.5); }
  .quitar:hover:not(:disabled) { background: transparent; color: var(--accent); }
  .vacia { padding: 28px 0; text-align: center; font-family: var(--hand); font-size: 23px; color: rgba(37, 31, 25, 0.55); }

  .interruptores { padding: 8px 20px; gap: 0; }
</style>
