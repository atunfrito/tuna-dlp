<script>
  // El menú del clic derecho en la bandeja (ver src-tauri/src/bandeja.rs).
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import ClipboardPaste from "@lucide/svelte/icons/clipboard-paste";
  import AppWindow from "@lucide/svelte/icons/app-window";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Power from "@lucide/svelte/icons/power";
  import Icono from "./lib/Icono.svelte";

  let e = $state({ estado: "nada descargando", pausado: false, progreso: null, activas: 0 });
  let abierto = $state(0);

  const hacer = (accion) => invoke("accion_bandeja", { accion });
  const punto = $derived(e.activas ? "verde" : e.pausado ? "ambar" : "");

  $effect(() => {
    document.documentElement.style.background = document.body.style.background = "transparent";
    const aplicar = (x) => {
      e = x;
      if (x.tema) document.documentElement.dataset.theme = x.tema;
    };
    invoke("estado_bandeja").then(aplicar);
    const quitar = [
      listen("menu-estado", ({ payload }) => aplicar(payload)),
      // Cada vez que se abre, la entrada vuelve a animarse.
      listen("menu-abierto", () => abierto++),
    ];
    const tecla = (ev) => ev.key === "Escape" && hacer("cerrar");
    window.addEventListener("keydown", tecla);
    return () => {
      quitar.forEach((q) => q.then((f) => f()));
      window.removeEventListener("keydown", tecla);
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="marco" onclick={(ev) => ev.target === ev.currentTarget && hacer("cerrar")}>
  {#key abierto}
    <div class="menu" role="menu">
      <div class="cabecera">
        <Icono tam={22} />
        <div class="textos">
          <span class="nombre">tuna-dlp</span>
          <span class="estado"><span class="punto {punto}"></span>{e.estado}</span>
        </div>
      </div>
      {#if e.activas && e.progreso != null}
        <div class="barra"><div class="relleno" style="width: {Math.round(e.progreso * 100)}%"></div></div>
      {/if}

      <div class="separador"></div>
      <button type="button" role="menuitem" class="item" onclick={() => hacer("pegar")}>
        <ClipboardPaste size={16} strokeWidth={1.75} />pegar y descargar
      </button>
      <button type="button" role="menuitem" class="item" onclick={() => hacer("abrir")}>
        <AppWindow size={16} strokeWidth={1.75} />abrir tuna-dlp
      </button>
      <button type="button" role="menuitem" class="item" onclick={() => hacer("pausa")}>
        {#if e.pausado}<Play size={16} strokeWidth={1.75} />reanudar todo{:else}<Pause size={16} strokeWidth={1.75} />pausar todo{/if}
      </button>
      <button type="button" role="menuitem" class="item" onclick={() => hacer("carpeta")}>
        <FolderOpen size={16} strokeWidth={1.75} />abrir carpeta de descargas
      </button>
      <div class="separador"></div>
      <button type="button" role="menuitem" class="item salir" onclick={() => hacer("salir")}>
        <Power size={16} strokeWidth={1.75} />salir del todo
      </button>
    </div>
  {/key}
</div>

<style>
  /* Pegado abajo a la derecha: ahí está el cursor al abrirlo. */
  .marco { height: 100vh; display: flex; align-items: flex-end; justify-content: flex-end; padding: 8px; }
  .menu {
    width: 100%; padding: 6px;
    background: var(--side); color: var(--fg);
    border: 1px solid var(--line); border-radius: 8px;
    box-shadow: 0 12px 30px rgba(20, 14, 8, 0.3);
    animation: entra 0.16s cubic-bezier(0.2, 0.8, 0.2, 1);
    transform-origin: bottom right;
  }
  @keyframes entra { from { opacity: 0; transform: translateY(6px) scale(0.98); } }

  .cabecera { display: flex; align-items: center; gap: 10px; padding: 8px 10px 6px; }
  .textos { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .nombre { font-size: 13.5px; font-weight: 700; }
  .estado {
    display: flex; align-items: center; gap: 6px;
    font-family: var(--mono); font-size: 10.5px; color: var(--dim);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .punto { width: 6px; height: 6px; flex: 0 0 auto; border-radius: 50%; background: var(--faint); }
  .punto.verde { background: var(--ok); }
  .punto.ambar { background: #d49a2a; }
  .barra { height: 3px; margin: 4px 10px 6px; border-radius: 2px; background: var(--line); overflow: hidden; }
  .relleno { height: 100%; border-radius: 2px; background: var(--accent); transition: width 0.4s ease; }

  .separador { height: 1px; margin: 5px 6px; background: var(--line); }
  .item {
    display: flex; align-items: center; gap: 10px; width: 100%;
    padding: 8px 10px; border: none; border-radius: 4px;
    background: transparent; color: var(--fg); text-align: left;
    font-size: 13.5px; font-weight: 600; cursor: pointer;
    transition: background-color 0.12s ease, color 0.12s ease;
  }
  .item :global(svg) { color: var(--dim); transition: color 0.12s ease; }
  .item:hover, .item:focus-visible { background: var(--hover); box-shadow: inset 2px 0 0 var(--accent); outline: none; }
  .item:hover :global(svg) { color: var(--fg); }
  .salir { color: var(--dim); }
  .salir:hover, .salir:hover :global(svg) { color: var(--accent); }
</style>
