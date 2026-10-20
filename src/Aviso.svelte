<script>
  // La ventanita de avisos (ver src-tauri/src/aviso.rs). Los textos llegan ya
  // montados desde lib/avisos.js.
  import { invoke } from "@tauri-apps/api/core";

  const a = window.__TUNA__.aviso;
  const DURA = 7000;

  let visible = $state(false);
  let temporizador;

  function cerrar() {
    clearTimeout(temporizador);
    visible = false;
    setTimeout(() => invoke("cerrar_aviso"), 220);
  }
  // Con el ratón encima no se va.
  const esperar = () => (temporizador = setTimeout(cerrar, DURA));
  const quieto = () => clearTimeout(temporizador);

  function abrirApp() {
    invoke("mostrar_ventana");
    cerrar();
  }
  function abrirCarpeta(e) {
    e.stopPropagation();
    invoke("mostrar_en_carpeta", { ruta: a.ruta ?? null, carpeta: a.carpeta });
    cerrar();
  }

  $effect(() => {
    // La ventana es transparente: solo se ve la tarjeta.
    document.documentElement.style.background = document.body.style.background = "transparent";
    requestAnimationFrame(() => (visible = true));
    esperar();
  });
</script>

<div class="marco">
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <div
    class="aviso" class:visible class:grande={a.tipo === "bandeja"}
    role="alert" onclick={abrirApp} onmouseenter={quieto} onmouseleave={esperar}
  >
    <span class="cinta"></span>
    <div class="fila">
      {#if a.miniatura}<img src={a.miniatura} alt="" />{/if}
      <div class="textos">
        <span class="etiqueta"><span class="punto" class:verde={a.tipo === "descarga"}></span>{a.etiqueta}</span>
        <span class="titulo">{a.titulo}</span>
        {#if a.texto}<span class="texto">{a.texto}</span>{/if}
      </div>
      <button type="button" class="cerrar" aria-label="cerrar" onclick={(e) => { e.stopPropagation(); cerrar(); }}>×</button>
    </div>
    {#if a.carpeta}
      <div class="pie">
        <button type="button" class="boton" onclick={abrirCarpeta}>abrir carpeta</button>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Pegada abajo: la ventana es más alta que la tarjeta para que quepa la sombra. */
  .marco { height: 100vh; display: flex; align-items: flex-end; justify-content: flex-end; padding: 16px; }
  .aviso {
    position: relative; width: 100%;
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); padding: 14px 14px 14px 16px;
    box-shadow: 0 10px 26px rgba(20, 14, 8, 0.28), 0 0 0 1px rgba(37, 31, 25, 0.08);
    cursor: pointer;
    opacity: 0; transform: translateX(28px);
    transition: opacity 0.22s ease, transform 0.28s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .aviso.visible { opacity: 1; transform: none; }
  .cinta {
    position: absolute; top: -9px; right: 38px; width: 72px; height: 20px;
    background: var(--tape); opacity: 0.9; transform: rotate(3deg);
  }
  .fila { display: flex; align-items: flex-start; gap: 12px; }
  img {
    width: 88px; height: 50px; flex: 0 0 auto; display: block;
    object-fit: cover; border-radius: 3px; background: rgba(37, 31, 25, 0.08);
  }
  .textos { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .etiqueta {
    display: flex; align-items: center; gap: 6px;
    font-family: var(--mono); font-size: 10px; letter-spacing: 0.16em; color: var(--paper-dim);
  }
  .punto { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
  .punto.verde { background: var(--ok); }
  .titulo {
    font-size: 14.5px; font-weight: 700; line-height: 1.25;
    display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
  }
  .texto { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .grande .titulo { font-family: var(--serif); font-weight: 400; font-size: 22px; line-height: 1.1; }
  .grande .texto { font-family: var(--hand); font-size: 19px; line-height: 1.1; color: rgba(37, 31, 25, 0.6); }

  .cerrar {
    flex: 0 0 auto; width: 24px; height: 24px; margin: -4px -4px 0 0; padding: 0;
    border: none; border-radius: 4px; background: transparent;
    color: var(--paper-dim); font-size: 17px; line-height: 1; cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }
  .cerrar:hover { background: rgba(37, 31, 25, 0.07); color: var(--accent); }
  .pie { display: flex; justify-content: flex-end; margin-top: 10px; }
  .boton {
    padding: 6px 11px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.18); background: transparent; color: var(--paper-fg);
    font-size: 12.5px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .boton:hover { background: rgba(37, 31, 25, 0.05); }
</style>
