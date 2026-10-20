<script>
  // Un error de yt-dlp. Los que se entienden llevan su frase y el error
  // técnico queda en "ver detalles"; el resto se enseña tal cual.
  let { error } = $props();

  const FRASES = [
    [/private video|video is private/i, "youtube dice que es privado. si no lo es, se ha puesto tonto: prueba a actualizar yt-dlp en ajustes."],
  ];
  const frase = $derived(FRASES.find(([patron]) => patron.test(error))?.[1]);
  let detalles = $state(false);
</script>

{#if frase}
  <span class="fallo">
    <span class="frase">{frase}</span>
    <button type="button" class="ver" onclick={() => (detalles = !detalles)}>{detalles ? "ocultar detalles" : "ver detalles"}</button>
    {#if detalles}<span class="tecnico" data-seleccionable>{error}</span>{/if}
  </span>
{:else}
  <span class="tecnico" data-seleccionable>{error}</span>
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
