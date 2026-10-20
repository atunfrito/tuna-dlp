<script>
  import Barra from "./lib/Barra.svelte";
  import Lateral from "./lib/Lateral.svelte";
  import Descargar from "./vistas/Descargar.svelte";
  import Cola from "./vistas/Cola.svelte";
  import Ajustes from "./vistas/Ajustes.svelte";
  import Suscripciones from "./vistas/Suscripciones.svelte";
  import Historial from "./vistas/Historial.svelte";
  import "./lib/canales.svelte.js";
  import "./lib/bandeja.svelte.js";
  import { ajustes, app, form, enTauri } from "./lib/estado.svelte.js";
  import { activas } from "./lib/cola.svelte.js";

  let main;
  // Cada vista empieza arriba del todo.
  $effect(() => {
    app.vista;
    main?.scrollTo(0, 0);
  });
  // --- arrastrar enlaces desde el navegador ------------------------------------
  // dragleave no siempre llega (p. ej. si se suelta fuera): el aviso se apaga
  // solo si pasan 150 ms sin dragover, que llega sin parar mientras se arrastra.
  let arrastrando = $state(false);
  let apagar;
  const esEnlace = (e) => [...(e.dataTransfer?.types ?? [])].some((t) => t === "text/uri-list" || t === "text/plain");
  function encima(e) {
    if (!esEnlace(e)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
    arrastrando = true;
    clearTimeout(apagar);
    apagar = setTimeout(() => (arrastrando = false), 150);
  }
  function soltar(e) {
    if (!esEnlace(e)) return;
    e.preventDefault();
    clearTimeout(apagar);
    arrastrando = false;
    const texto = e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text/plain") || "";
    const urls = texto.split(/\s+/).filter((x) => /^https?:\/\//i.test(x));
    if (!urls.length) return;
    const ya = new Set(form.urls.split("\n").map((l) => l.trim()));
    form.urls = [form.urls.trim(), ...urls.filter((u) => !ya.has(u))].filter(Boolean).join("\n");
    app.vista = "dl";
  }

  // Si se apaga una función opcional estando en su vista, de vuelta a descargar.
  $effect(() => {
    if (!ajustes.suscripciones && app.vista === "feeds") app.vista = "dl";
    if (!ajustes.historial && app.vista === "hist") app.vista = "dl";
  });
</script>

<div class="ventana" role="application" ondragover={encima} ondrop={soltar}>
  <Barra />
  <div class="cuerpo">
    <Lateral activas={activas()} hayActualizacion={app.upd === "available"} />
    <main bind:this={main}>
      <div class="brillo"></div>
      {#if arrastrando}
        <div class="soltar">
          <span class="grande">suéltalo aquí</span>
          <span class="pequeno">SE AÑADE A LA LISTA DE URLS</span>
        </div>
      {/if}
      {#if !enTauri}
        <p class="fuera">esto es solo la interfaz. para usarla, arranca la app con <code>npm run dev</code>.</p>
      {/if}
      {#if app.vista === "dl"}
        <Descargar />
      {:else if app.vista === "queue"}
        <Cola />
      {:else if app.vista === "hist"}
        <Historial />
      {:else if app.vista === "feeds"}
        <Suscripciones />
      {:else}
        <Ajustes />
      {/if}
    </main>
  </div>
</div>

<style>
  .ventana {
    height: 100vh; display: flex; flex-direction: column; overflow: hidden;
    background: var(--bg);
    background-image: radial-gradient(var(--dots) 1px, transparent 1.3px);
    background-size: 22px 22px;
    color: var(--fg);
    transition: background-color 0.45s ease, color 0.45s ease;
  }
  .cuerpo { flex: 1 1 auto; min-height: 0; display: flex; }
  main { flex: 1 1 auto; min-width: 0; overflow-y: auto; position: relative; }
  .brillo {
    position: absolute; top: -260px; left: 50%; transform: translateX(-50%);
    width: 900px; height: 520px; pointer-events: none;
    background: radial-gradient(ellipse at 50% 50%, var(--glow), rgba(0, 0, 0, 0) 66%);
  }
  .soltar {
    position: absolute; inset: 14px; z-index: 5; pointer-events: none;
    display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px;
    border: 2px dashed var(--accent); border-radius: 8px; background: rgba(200, 16, 46, 0.06);
  }
  .grande { font-family: var(--hand); font-size: 40px; line-height: 1; color: var(--accent); }
  .pequeno { font-family: var(--mono); font-size: 11px; letter-spacing: 0.14em; color: var(--dim); }
  .fuera {
    position: relative; margin: 20px 36px 0; padding: 10px 14px; border-radius: 4px;
    background: var(--accent); color: var(--accent-fg); font-size: 13px;
  }
</style>
