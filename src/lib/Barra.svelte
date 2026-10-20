<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { enTauri } from "./estado.svelte.js";

  const ventana = enTauri ? getCurrentWindow() : null;
  let maximizada = $state(false);

  $effect(() => {
    if (!ventana) return;
    const leer = () => ventana.isMaximized().then((m) => (maximizada = m));
    leer();
    const quitar = ventana.onResized(leer);
    return () => quitar.then((f) => f());
  });
</script>

<header class="barra" data-tauri-drag-region>
  <span class="titulo" data-tauri-drag-region>tuna-dlp</span>
  <div class="botones">
    <button type="button" aria-label="minimizar" onclick={() => ventana?.minimize()}>
      <svg width="11" height="11" viewBox="0 0 12 12"><line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.1" /></svg>
    </button>
    <button type="button" aria-label={maximizada ? "restaurar" : "maximizar"} onclick={() => ventana?.toggleMaximize()}>
      {#if maximizada}
        <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.1">
          <rect x="1.5" y="3.5" width="7" height="7" rx="1.2" /><path d="M3.5 3.5V2.7c0-.7.5-1.2 1.2-1.2h4.6c.7 0 1.2.5 1.2 1.2v4.6c0 .7-.5 1.2-1.2 1.2h-.8" />
        </svg>
      {:else}
        <svg width="11" height="11" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.1" /></svg>
      {/if}
    </button>
    <button type="button" class="cerrar" aria-label="cerrar" onclick={() => ventana?.close()}>
      <svg width="11" height="11" viewBox="0 0 12 12"><path d="M1.5 1.5l9 9M10.5 1.5l-9 9" stroke="currentColor" stroke-width="1.1" /></svg>
    </button>
  </div>
</header>

<style>
  .barra {
    height: 40px; flex: 0 0 auto;
    display: flex; align-items: center; justify-content: space-between;
    padding-left: 14px;
    border-bottom: 1px solid var(--line);
    background: var(--side);
    transition: background-color 0.45s ease;
  }
  .titulo { font-size: 12.5px; font-weight: 600; color: var(--dim); }
  .botones { display: flex; height: 100%; }
  .botones button {
    width: 46px; height: 100%; padding: 0;
    display: flex; align-items: center; justify-content: center;
    border: none; background: transparent; color: var(--dim);
    transition: background-color 0.15s ease, color 0.15s ease;
  }
  .botones button:hover { background: var(--hover); color: var(--fg); }
  .botones .cerrar:hover { background: #c42b1c; color: #fff; }
</style>
