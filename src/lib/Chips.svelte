<script>
  // Grupo de chips excluyentes. opciones: [[valor, etiqueta], ...]
  let { opciones, valor = $bindable(), alCambiar = () => {} } = $props();
</script>

<div class="chips">
  {#each opciones as [v, etiqueta]}
    <button type="button" class="chip" class:on={valor === v} onclick={() => { valor = v; alCambiar(); }}>{etiqueta}</button>
  {/each}
</div>

<style>
  .chips { display: flex; gap: 6px; flex-wrap: wrap; }
  .chip {
    padding: 7px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.16); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease;
  }
  .chip:hover { border-color: var(--accent); }
  .chip.on { border-color: var(--accent); background: var(--accent); color: var(--accent-fg); }
</style>
