<script>
  // Un error de yt-dlp. Los que se entienden llevan su frase y el error
  // técnico queda en "ver detalles"; el resto se enseña tal cual.
  import { t, tError } from "./i18n.svelte.js";

  let { error } = $props();

  const FRASES = [
    [/private video|video is private/i, "fallo.privado"],
  ];
  const frase = $derived(FRASES.find(([patron]) => patron.test(error))?.[1]);
  const texto = $derived(tError(error));
  let detalles = $state(false);
</script>

{#if frase}
  <span class="fallo">
    <span class="frase">{t(frase)}</span>
    <button type="button" class="ver" onclick={() => (detalles = !detalles)}>{detalles ? t("fallo.ocultar") : t("fallo.ver")}</button>
    {#if detalles}<span class="tecnico" data-seleccionable>{texto}</span>{/if}
  </span>
{:else}
  <span class="tecnico" data-seleccionable>{texto}</span>
{/if}

<style>
  .fallo { display: flex; flex-direction: column; align-items: flex-start; gap: 4px; }
  .frase { font-size: 13px; font-weight: 600; color: var(--accent); }
  .ver {
    padding: 0; border: none; background: transparent; cursor: pointer;
    font-family: var(--mono); font-size: 10.5px; color: var(--paper-dim);
    text-decoration: underline; text-underline-offset: 2px;
  }
  .ver:hover { color: var(--accent); }
  .tecnico { font-family: var(--mono); font-size: 11.5px; color: var(--accent); white-space: pre-wrap; word-break: break-word; }
</style>
