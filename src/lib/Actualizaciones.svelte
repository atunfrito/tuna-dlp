<script>
  // Avisos de versión nueva arriba de la vista: uno para tuna-dlp y otro para
  // yt-dlp. Se cierran con "ahora no" hasta la próxima vez que se abra la app.
  import { app, actualizar } from "./estado.svelte.js";
  import { nueva, instalarApp } from "./actualizador.svelte.js";
  import { activas } from "./cola.svelte.js";
  import { t, tError } from "./i18n.svelte.js";

  let ytCerrado = $state(false);
  // Los avisos de "hecho" o "falló" de yt-dlp solo salen si se actualizó desde aquí.
  let ytDesdeAqui = $state(false);

  const ocupado = $derived(activas() > 0);
  const pct = (b, total) => (total > 0 ? Math.min(100, Math.round((b / total) * 100)) : 0);

  // El plugin dice "signature" si el paquete no está firmado con nuestra clave.
  const firmaMala = $derived(/signature/i.test(nueva.error));
  const verApp = $derived(!nueva.cerrado && nueva.estado !== "idle");
  const verYt = $derived(
    !ytCerrado && (app.upd === "available" || app.upd === "updating" || (ytDesdeAqui && (app.upd === "ok" || app.upd === "error"))),
  );

  function actualizarYt() {
    ytDesdeAqui = true;
    actualizar();
  }
</script>

{#if verApp || verYt}
  <div class="avisos">
    {#if verApp}
      {@const bajando = nueva.estado === "downloading" || nueva.estado === "installing"}
      <div class="tira">
        <span class="textos">
          <span class="etiqueta">TUNA-DLP</span>
          {#if nueva.estado === "error"}
            <span class="titulo">{t("act.app", { version: nueva.version })}</span>
            <span class="fallo" data-seleccionable>{firmaMala ? t("act.firma") : t("act.fallo", { error: nueva.error })}</span>
          {:else}
            <span class="titulo">{t("act.app", { version: nueva.version })}</span>
            <span class="pista">
              {#if nueva.estado === "installing"}{t("act.instalando")}
              {:else if nueva.estado === "downloading"}{t("act.bajando", { pct: pct(nueva.bajado, nueva.total) + " %" })}
              {:else if nueva.portable}{t("act.app_pista_portable")}
              {:else if ocupado}{t("act.esperar")}
              {:else}{t("act.app_pista")}{/if}
            </span>
          {/if}
        </span>
        {#if !bajando}
          <span class="botones">
            <button type="button" class="cerrar" onclick={() => (nueva.cerrado = true)}>{t("act.cerrar")}</button>
            <button type="button" class="boton" disabled={ocupado && !nueva.portable} onclick={instalarApp}>
              {nueva.portable ? t("act.descargar") : nueva.estado === "error" ? t("act.reintentar") : t("act.actualizar")}
            </button>
          </span>
        {/if}
        {#if bajando}
          <span class="barra"><span class="relleno" class:sin-total={!nueva.total} style="width: {nueva.total ? pct(nueva.bajado, nueva.total) : 100}%"></span></span>
        {/if}
      </div>
    {/if}

    {#if verYt}
      <div class="tira" class:hecho={app.upd === "ok"}>
        <span class="textos">
          <span class="etiqueta">YT-DLP</span>
          {#if app.upd === "error"}
            <span class="fallo" data-seleccionable>{tError(app.updError)}</span>
          {:else if app.upd === "ok"}
            <span class="titulo">{t("upd.ok", { version: app.ytv })}</span>
            <span class="pista">{t("upd.ok_pista")}</span>
          {:else}
            <span class="titulo">{t("act.ytdlp", { version: app.ultima })}</span>
            <span class="pista">
              {#if app.upd === "updating"}{t("act.bajando", { pct: pct(app.ytBajado, app.ytTotal) + " %" })}
              {:else if ocupado}{t("act.esperar")}
              {:else}{t("upd.available_pista")}{/if}
            </span>
          {/if}
        </span>
        {#if app.upd !== "updating"}
          <span class="botones">
            <button type="button" class="cerrar" onclick={() => (ytCerrado = true)}>{app.upd === "available" ? t("act.cerrar") : "×"}</button>
            {#if app.upd === "available"}
              <button type="button" class="boton" disabled={ocupado} onclick={actualizarYt}>{t("act.actualizar")}</button>
            {/if}
          </span>
        {:else}
          <span class="barra"><span class="relleno" class:sin-total={!app.ytTotal} style="width: {app.ytTotal ? pct(app.ytBajado, app.ytTotal) : 100}%"></span></span>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .avisos { position: relative; z-index: 1; padding: 22px 36px 0; display: flex; flex-direction: column; gap: 10px; max-width: 820px; }
  .tira {
    position: relative; overflow: hidden;
    display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap;
    padding: 12px 16px 12px 18px; border-radius: var(--r);
    background: var(--paper); color: var(--paper-fg); box-shadow: var(--shadow);
    border-left: 3px solid var(--accent);
    animation: entra 0.35s ease;
  }
  .tira.hecho { border-left-color: var(--ok); }
  .textos { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .etiqueta { font-family: var(--mono); font-size: 10.5px; letter-spacing: 0.16em; color: var(--paper-dim); }
  .titulo { font-family: var(--serif); font-size: 22px; line-height: 1.1; }
  .pista { font-family: var(--hand); font-size: 19px; line-height: 1.1; color: rgba(37, 31, 25, 0.6); }
  .fallo { font-family: var(--mono); font-size: 12px; color: var(--accent); white-space: pre-wrap; word-break: break-word; }
  .botones { display: flex; align-items: center; gap: 6px; flex: 0 0 auto; }
  .boton {
    padding: 9px 14px; border-radius: 4px; border: none;
    background: var(--accent); color: var(--accent-fg);
    font-size: 13px; font-weight: 700; cursor: pointer;
    transition: transform 0.2s ease, background-color 0.2s ease;
  }
  .boton:hover:not(:disabled) { transform: translateY(-1px); background: var(--accent-hot); }
  .boton:disabled { opacity: 0.45; cursor: default; }
  .cerrar {
    padding: 9px 10px; border: none; background: transparent; border-radius: 4px;
    color: var(--paper-dim); font-size: 13px; cursor: pointer;
  }
  .cerrar:hover { background: rgba(37, 31, 25, 0.06); color: var(--paper-fg); }
  .barra { position: absolute; left: 0; right: 0; bottom: 0; height: 3px; background: rgba(200, 16, 46, 0.12); }
  .relleno { display: block; height: 100%; background: var(--accent); transition: width 0.25s ease; }
  .relleno.sin-total { animation: late 1.2s ease-in-out infinite; }
  @keyframes entra { from { opacity: 0; transform: translateY(-6px); } }
  @keyframes late { 50% { opacity: 0.35; } }
</style>
