<script>
  import Download from "@lucide/svelte/icons/download";
  import List from "@lucide/svelte/icons/list";
  import Clock from "@lucide/svelte/icons/clock";
  import Rss from "@lucide/svelte/icons/rss";
  import SlidersVertical from "@lucide/svelte/icons/sliders-vertical";
  import Sun from "@lucide/svelte/icons/sun";
  import Moon from "@lucide/svelte/icons/moon";
  import tinta from "../assets/logotipo-tinta.png";
  import acento from "../assets/logotipo-acento.png";
  import Icono from "./Icono.svelte";
  import { version } from "../../package.json";
  import { ajustes, app, temaEfectivo } from "./estado.svelte.js";

  let { activas = 0, hayActualizacion = false } = $props();

  const vistas = $derived([
    ["dl", "descargar", Download],
    ["queue", "cola", List],
    ...(ajustes.historial ? [["hist", "historial", Clock]] : []),
    ...(ajustes.suscripciones ? [["feeds", "suscripciones", Rss]] : []),
    ["settings", "ajustes", SlidersVertical],
  ]);
  const oscuro = $derived(temaEfectivo() === "dark");
  const plegado = $derived(ajustes.menuPlegado);
</script>

<!-- Estilo Fluent: pulsar el logotipo pliega la barra a solo iconos. -->
<nav class="lateral" class:plegado>
  <button
    type="button" class="logo"
    title={plegado ? "desplegar menú" : "plegar menú"} aria-expanded={!plegado}
    onclick={() => (ajustes.menuPlegado = !plegado)}
  >
    {#if plegado}
      <Icono tam={28} />
    {:else}
      <span class="marca" role="img" aria-label="tuna-dlp">
        <span class="capa tinta" style="--m: url({tinta})"></span>
        <span class="capa acento" style="--m: url({acento})"></span>
      </span>
    {/if}
  </button>

  {#each vistas as [id, etiqueta, Icon]}
    <button
      type="button" class="nav" class:on={app.vista === id}
      title={plegado ? etiqueta : null} aria-label={etiqueta}
      onclick={() => (app.vista = id)}
    >
      <span class="icono-nav">
        <Icon size={18} strokeWidth={1.75} />
        {#if id === "settings" && hayActualizacion && plegado}<span class="punto encima"></span>{/if}
      </span>
      {#if !plegado}
        <span class="nombre">{etiqueta}</span>
        {#if id === "queue" && activas > 0}<span class="cuenta">{activas}</span>{/if}
        {#if id === "settings" && hayActualizacion}<span class="punto"></span>{/if}
      {/if}
    </button>
  {/each}

  <div class="hueco"></div>

  <div class="pie">
    {#if app.ytv && !plegado}
      <div class="motor"><span class="verde"></span><span>yt-dlp {app.ytv}</span></div>
    {/if}
    <div class="fila">
      {#if !plegado}<span class="version">tuna-dlp {version}</span>{/if}
      <button type="button" class="tema" aria-label="cambiar tema" onclick={() => (ajustes.tema = oscuro ? "light" : "dark")}>
        <span class="icono" class:visible={oscuro}><Sun size={15} strokeWidth={1.75} /></span>
        <span class="icono" class:visible={!oscuro}><Moon size={15} strokeWidth={1.75} /></span>
      </button>
    </div>
  </div>
</nav>

<style>
  .lateral {
    width: 224px; flex: 0 0 auto; overflow: hidden;
    display: flex; flex-direction: column; gap: 4px;
    padding: 22px 14px 16px;
    border-right: 1px solid var(--line);
    background: var(--side);
    transition: width 0.28s cubic-bezier(0.2, 0.8, 0.2, 1), padding 0.28s ease, background-color 0.45s ease;
  }
  .lateral.plegado { width: 68px; padding-left: 12px; padding-right: 12px; }

  .logo {
    display: flex; align-items: center; height: 44px; margin: 0 0 18px; padding: 0 8px;
    border: none; border-radius: 4px; background: transparent; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .logo:hover { background: var(--hover); }
  .plegado .logo { justify-content: center; padding: 0; }
  .marca { position: relative; display: block; width: 142px; height: 40px; flex: 0 0 auto; }
  .capa {
    position: absolute; inset: 0;
    -webkit-mask: var(--m) no-repeat left center / contain;
    mask: var(--m) no-repeat left center / contain;
  }
  .tinta { background: var(--fg); transition: background-color 0.45s ease; }
  .acento { background: var(--accent); }

  .nav {
    display: flex; align-items: center; gap: 11px; width: 100%;
    padding: 10px 12px; border-radius: 4px; border: none;
    background: transparent; color: var(--dim);
    font-size: 14.5px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease, color 0.2s ease;
  }
  .nav:hover { color: var(--fg); }
  .nav.on { background: var(--hover); color: var(--fg); box-shadow: inset 2px 0 0 var(--accent); }
  .plegado .nav { justify-content: center; padding: 10px 0; }
  .icono-nav { position: relative; display: flex; }
  .nombre { flex: 1 1 auto; text-align: left; white-space: nowrap; }
  .cuenta {
    font-family: var(--mono); font-size: 10.5px; font-weight: 600;
    min-width: 20px; padding: 2px 6px; border-radius: 3px;
    background: var(--accent); color: var(--accent-fg); text-align: center;
  }
  .punto { width: 7px; height: 7px; border-radius: 50%; background: var(--accent); }
  .punto.encima { position: absolute; top: -3px; right: -4px; box-shadow: 0 0 0 2px var(--side); }

  .hueco { flex: 1 1 auto; }
  .pie {
    display: flex; flex-direction: column; gap: 10px;
    padding: 14px 10px 4px; border-top: 1px solid var(--line);
    transition: padding 0.28s ease;
  }
  .plegado .pie { padding: 14px 0 4px; }
  .motor {
    white-space: nowrap; display: flex; align-items: center; gap: 8px;
    font-family: var(--mono); font-size: 10.5px; color: var(--dim);
  }
  .verde { width: 7px; height: 7px; border-radius: 50%; background: var(--ok); }
  .fila { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .plegado .fila { justify-content: center; }
  .version { font-family: var(--mono); font-size: 10.5px; color: var(--faint); white-space: nowrap; }
  .tema {
    position: relative; width: 32px; height: 32px; padding: 0; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    border-radius: 4px; border: 1px solid var(--line);
    background: transparent; color: var(--fg); cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .tema:hover { background: var(--hover); }
  .icono {
    position: absolute; display: flex;
    opacity: 0; transform: rotate(-180deg) scale(0.6);
    transition: transform 0.45s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.3s ease;
  }
  .icono.visible { opacity: 1; transform: none; }
</style>
