<script module>
  // Fuera del componente: al volver de otra vista siguen ahí la vista previa
  // y los vídeos marcados de la lista, sin volver a preguntar a yt-dlp.
  const previas = $state({});
  const listas = $state({});
  // La última url que se ha pegado sola: no se vuelve a pegar aunque borres el
  // campo (ni al volver de otra vista). Solo una url nueva en el portapapeles.
  let ultimaDelPortapapeles = null;
</script>

<script>
  import { invoke } from "@tauri-apps/api/core";
  import { untrack } from "svelte";
  import Download from "@lucide/svelte/icons/download";
  import Folder from "@lucide/svelte/icons/folder";
  import Chips from "../lib/Chips.svelte";
  import Interruptor from "../lib/Interruptor.svelte";
  import Fallo from "../lib/Fallo.svelte";
  import { ajustes, rutas, form, app, PRESETS, CAMPOS_PRESET } from "../lib/estado.svelte.js";
  import { encolar } from "../lib/cola.svelte.js";
  import { argumentos, comoTexto, recorte, indices, vistaPrevia, PLANTILLAS } from "../lib/comando.js";
  import { nombresDe } from "../lib/idiomas.js";
  import SelectorIdiomas from "../lib/SelectorIdiomas.svelte";
  import { queEs } from "../lib/formato.js";

  const esUrl = (t) => /^https?:\/\/\S+$/i.test(t);
  const sinPreset = () => (form.preset = null);

  const lineas = $derived([...new Set(form.urls.split("\n").map((l) => l.trim()).filter(Boolean))]);
  // Con la función apagada, lo que quedara escrito en el recorte no cuenta.
  const corte = $derived(ajustes.recortar ? recorte(form) : null);
  const comando = $derived(comoTexto(argsDe(lineas[0]), lineas[0], corte));
  const que = $derived(queEs(form));
  const resumen = $derived(lineas.length + (lineas.length === 1 ? " url" : " urls") + " · " + que);

  function aplicarPreset(nombre) {
    Object.assign(form, PRESETS[nombre], { preset: nombre });
  }

  // --- presets propios ------------------------------------------------------
  function guardarPreset() {
    const usados = new Set(ajustes.presets.map((p) => p.nombre));
    let nombre = "mi preset", n = 1;
    while (usados.has(nombre)) nombre = "mi preset " + ++n;
    const valores = Object.fromEntries(CAMPOS_PRESET.map((k) => [k, form[k]]));
    ajustes.presets.push({ nombre, valores });
    form.preset = nombre;
  }
  function aplicarPropio(p) {
    Object.assign(form, p.valores, { preset: p.nombre });
  }
  function borrarPreset(p) {
    ajustes.presets.splice(ajustes.presets.indexOf(p), 1);
    if (form.preset === p.nombre) form.preset = null;
  }

  // --- portapapeles ---------------------------------------------------------
  let pegada = $state("");
  async function mirarPortapapeles() {
    if (!ajustes.portapapeles) return;
    const texto = (await invoke("leer_portapapeles").catch(() => null))?.trim();
    if (!texto || !esUrl(texto) || texto === ultimaDelPortapapeles) return;
    ultimaDelPortapapeles = texto;
    if (!form.urls.trim()) form.urls = pegada = texto;
  }
  $effect(() => {
    // untrack: si el efecto leyera form.urls, se relanzaría cada vez que cambia
    // el campo y, al vaciarlo, volvería a pegar la url.
    untrack(mirarPortapapeles);
    window.addEventListener("focus", mirarPortapapeles);
    return () => window.removeEventListener("focus", mirarPortapapeles);
  });

  // --- vista previa (yt-dlp -J) ---------------------------------------------
  $effect(() => {
    const pendientes = lineas.filter((u) => esUrl(u) && !previas[u]);
    if (!pendientes.length) return;
    const t = setTimeout(() => {
      for (const u of pendientes) {
        previas[u] = { cargando: true };
        invoke("info", { url: u }).then(
          (info) => (previas[u] = { info }),
          (error) => (previas[u] = { error: String(error) })
        );
      }
    }, 450);
    return () => clearTimeout(t);
  });

  // --- carpeta ---------------------------------------------------------------
  async function cambiarCarpeta() {
    const elegida = await invoke("elegir_carpeta", { actual: ajustes.carpeta });
    if (elegida) ajustes.carpeta = elegida;
  }

  // --- la lista (playlist) de la primera url --------------------------------
  const verLista = $derived(form.playlist && form.mode !== "subs" && esUrl(lineas[0] ?? ""));
  const lista = $derived(verLista ? listas[lineas[0]] : null);
  const marcadas = $derived(lista?.entradas?.filter((e) => e.on).length ?? 0);
  $effect(() => {
    const u = verLista && lineas[0];
    if (!u || listas[u]) return;
    const t = setTimeout(() => {
      listas[u] = { cargando: true };
      invoke("lista", { url: u }).then(
        (l) => (listas[u] = { titulo: l.titulo, entradas: l.entradas.map((e) => ({ ...e, on: true })) }),
        (error) => (listas[u] = { error: String(error) })
      );
    }, 450);
    return () => clearTimeout(t);
  });
  const marcarTodos = (on) => lista.entradas.forEach((e) => (e.on = on));

  // Solo la url de la lista previsualizada lleva -I.
  function argsDe(url) {
    const l = verLista && url === lineas[0] ? lista : null;
    const sel = l?.entradas?.length ? indices(l.entradas.map((e) => e.on)) : null;
    return argumentos(form, ajustes, rutas, sel);
  }

  // --- subtítulos: los que tiene de verdad el vídeo ----------------------------
  const primera = $derived(previas[lineas[0]]?.info);

  // --- plantilla de nombres ---------------------------------------------------
  const quedara = $derived(
    vistaPrevia(form.plantilla, {
      ...(primera?.campos ?? {}),
      ext: form.mode === "audio" ? form.afmt : form.mode === "subs" ? "vtt" : form.container,
      ...(lista?.titulo
        ? { playlist_title: lista.titulo, playlist_index: "1".padStart(String(lista.entradas.length).length, "0") }
        : {}),
    })
  );

  // --- descargar: a la cola ------------------------------------------------
  let cmdAbierto = $state(false);
  // Con la lista a la vista y nada marcado no hay nada que bajar.
  const bloqueado = $derived(!lineas.length || (lista?.entradas && marcadas === 0));

  function descargar() {
    if (bloqueado) return;
    const trabajos = lineas.map((url) => ({ url, args: $state.snapshot(argsDe(url)) }));
    encolar(trabajos, $state.snapshot(corte), que, previas);
    app.vista = "queue";
  }
</script>

<div class="vista">
  <div class="cabecera">
    <!-- Solo el punto de la "?" en rojo: encima, una copia roja recortada desde 0.72em. -->
    <h1>¿qué nos bajamos hoy<span class="interrogacion">?<span class="punto" aria-hidden="true"><span>?</span></span></span></h1>
    <div class="atajos">
      <span class="mano">atajos →</span>
      {#each Object.keys(PRESETS) as nombre}
        <button type="button" class="preset" class:on={form.preset === nombre} onclick={() => aplicarPreset(nombre)}>{nombre}</button>
      {/each}
      {#each ajustes.presets as p (p.nombre)}
        <span class="propio" class:on={form.preset === p.nombre}>
          <button type="button" class="preset" onclick={() => aplicarPropio(p)}>{p.nombre}</button>
          <button type="button" class="borrar" title="borrar este preset" aria-label="borrar {p.nombre}" onclick={() => borrarPreset(p)}>×</button>
        </span>
      {/each}
      <button type="button" class="guardar" title="guarda lo que tienes elegido como atajo" onclick={guardarPreset}>+ guardar</button>
    </div>
  </div>

  <section class="papel urls">
    <span class="cinta"></span>
    <div class="entre">
      <span class="etiqueta">LAS URLS · UNA POR LÍNEA</span>
      {#if pegada && form.urls === pegada}<span class="pegada">● pegada del portapapeles</span>{/if}
    </div>
    <textarea bind:value={form.urls} rows="2" spellcheck="false" placeholder="https://www.youtube.com/watch?v=..."></textarea>
    <span class="arrastra">o arrastra un enlace desde el navegador</span>

    {#each lineas.filter((u) => previas[u]) as u (u)}
      {@const p = previas[u]}
      <div class="previa">
        {#if p.info?.miniatura}
          <img src={p.info.miniatura} alt="" />
        {:else}
          <span class="hueco-img" class:cargando={p.cargando}></span>
        {/if}
        <div class="datos">
          {#if p.cargando}
            <span class="barra-carga" style="width: 60%"></span>
            <span class="barra-carga fina" style="width: 30%"></span>
          {:else if p.error}
            <Fallo error={p.error} />
          {:else}
            <div class="titulo">{p.info.titulo}</div>
            <div class="meta">{[p.info.canal, p.info.sitio].filter(Boolean).join(" · ")}</div>
          {/if}
        </div>
      </div>
    {/each}
  </section>

  {#if lista}
    <section class="papel lista">
      <div class="entre">
        <span class="etiqueta">LA LISTA{lista.entradas ? ` · ${marcadas} DE ${lista.entradas.length}` : ""}</span>
        {#if lista.entradas}
          <div class="botones-lista">
            <button type="button" class="secundario chico" onclick={() => marcarTodos(true)}>todos</button>
            <button type="button" class="secundario chico" onclick={() => marcarTodos(false)}>ninguno</button>
          </div>
        {/if}
      </div>
      {#if lista.cargando}
        {#each [60, 45, 52] as ancho}
          <div class="video"><span class="caja"></span><span class="mini cargando"></span><span class="barra-carga" style="width: {ancho}%"></span></div>
        {/each}
      {:else if lista.error}
        <Fallo error={lista.error} />
      {:else}
        <div class="videos">
          {#each lista.entradas as e, i}
            <button type="button" class="video" role="checkbox" aria-checked={e.on} onclick={() => (e.on = !e.on)}>
              <span class="caja" class:on={e.on}>{e.on ? "✓" : ""}</span>
              <span class="n">{String(i + 1).padStart(2, "0")}</span>
              {#if e.miniatura}<img class="mini" src={e.miniatura} alt="" loading="lazy" />{:else}<span class="mini"></span>{/if}
              <span class="nombre-video">{e.titulo}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  <div class="rejilla">
    <section class="papel">
      <span class="etiqueta">QUÉ QUIERES</span>
      <Chips bind:valor={form.mode} alCambiar={sinPreset} opciones={[["video", "vídeo"], ["audio", "solo audio"], ["subs", "subtítulos"]]} />
      {#if form.mode === "video"}
        <div class="grupos">
          <div class="grupo">
            <span class="sub">calidad</span>
            <Chips bind:valor={form.quality} alCambiar={sinPreset} opciones={[["best", "máxima"], ["2160", "4K"], ["1440", "1440p"], ["1080", "1080p"], ["720", "720p"], ["480", "480p"], ["360", "360p"]]} />
          </div>
          <div class="grupo">
            <span class="sub">contenedor</span>
            <Chips bind:valor={form.container} alCambiar={sinPreset} opciones={[["mp4", "mp4"], ["mkv", "mkv"], ["webm", "webm"]]} />
          </div>
          <div class="grupo">
            <span class="sub">códec</span>
            <Chips bind:valor={form.codec} alCambiar={sinPreset} opciones={[["any", "da igual"], ["h264", "h264"], ["vp9", "vp9"], ["av01", "av1"]]} />
          </div>
        </div>
      {:else if form.mode === "audio"}
        <div class="grupos">
          <div class="grupo">
            <span class="sub">formato</span>
            <Chips bind:valor={form.afmt} alCambiar={sinPreset} opciones={["mp3", "m4a", "opus", "flac", "wav", "vorbis"].map((f) => [f, f])} />
          </div>
          <div class="grupo">
            <span class="sub">bitrate</span>
            <Chips bind:valor={form.abr} alCambiar={sinPreset} opciones={[["best", "máxima"], ["320", "320k"], ["256", "256k"], ["192", "192k"], ["128", "128k"]]} />
          </div>
        </div>
      {:else}
        <div class="grupo">
          <span class="sub">idiomas</span>
          <SelectorIdiomas bind:valor={form.langs} manuales={primera?.subtitulos ?? null} automaticos={primera?.automaticos ?? null} alCambiar={sinPreset} />
        </div>
      {/if}
    </section>

    <section class="papel extras">
      <span class="etiqueta separada">EXTRAS</span>
      <Interruptor bind:valor={form.thumb} alCambiar={sinPreset} etiqueta="miniatura" pista="incrustada en el archivo" />
      <Interruptor bind:valor={form.meta} alCambiar={sinPreset} etiqueta="metadatos" pista="título, canal, fecha" />
      <Interruptor bind:valor={form.chapters} alCambiar={sinPreset} etiqueta="capítulos" pista="si el vídeo los tiene" />
      <Interruptor bind:valor={form.sponsor} alCambiar={sinPreset} etiqueta="quitar patrocinios" pista="vía sponsorblock" />
      <Interruptor bind:valor={form.subs} alCambiar={sinPreset} etiqueta="subtítulos" pista={"incrustados: " + (nombresDe(form.langs) || "ningún idioma")} />
      <Interruptor bind:valor={form.playlist} alCambiar={sinPreset} etiqueta="lista entera" pista="si la url es una playlist" />
    </section>
  </div>

  <div class="rejilla">
    {#if ajustes.recortar}
      <section class="papel">
        <span class="etiqueta">RECORTAR</span>
        <div class="recorte">
          <input class="campo tiempo" bind:value={form.start} placeholder="00:00" spellcheck="false" />
          <span class="flecha">→</span>
          <input class="campo tiempo" bind:value={form.end} placeholder="final" spellcheck="false" />
        </div>
      </section>
    {/if}
    <section class="papel">
      <span class="etiqueta">SE GUARDA EN</span>
      <div class="carpeta">
        <span class="ruta" title={ajustes.carpeta} data-seleccionable>{ajustes.carpeta}</span>
        <button type="button" class="secundario" onclick={cambiarCarpeta}>
          <Folder size={15} strokeWidth={1.75} />cambiar
        </button>
      </div>
      {#if ajustes.plantilla}
        <span class="etiqueta hueco-arriba">CÓMO SE LLAMA</span>
        <Chips bind:valor={form.plantilla} alCambiar={sinPreset} opciones={PLANTILLAS} />
        <input class="campo" bind:value={form.plantilla} oninput={sinPreset} spellcheck="false" />
        <div class="quedara">
          <span class="mano-roja">quedará así →</span>
          <span class="muestra" data-seleccionable>{quedara}</span>
        </div>
      {/if}
    </section>
  </div>

  {#if ajustes.verComando}
  <section class="terminal">
    <button type="button" class="plegar" onclick={() => (cmdAbierto = !cmdAbierto)}>
      <span>VER EL COMANDO</span>
      <span class="chevron" class:abierto={cmdAbierto}>▾</span>
    </button>
    <div class="panel" class:abierto={cmdAbierto}>
      <div class="interior">
        <div class="linea" data-seleccionable><span class="dolar">$</span>{comando}</div>
      </div>
    </div>
  </section>
  {/if}

  <div class="accion">
    <span class="resumen">{resumen}</span>
    <button type="button" class="descargar" onclick={descargar} disabled={bloqueado}>
      <Download size={17} strokeWidth={2} />descargar
    </button>
  </div>
</div>

<style>
  .vista { position: relative; padding: 30px 36px 36px; display: flex; flex-direction: column; gap: 18px; }

  .cabecera { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; flex-wrap: wrap; }
  h1 {
    margin: 0; font-family: var(--serif); font-weight: 400;
    font-size: 44px; line-height: 1; letter-spacing: -0.022em;
  }
  .interrogacion { position: relative; display: inline-block; line-height: 1; }
  .punto { position: absolute; inset: 0.72em 0 0 0; overflow: hidden; }
  .punto span { display: block; position: relative; top: -0.72em; color: var(--accent); }
  .atajos { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .mano { font-family: var(--hand); font-size: 20px; color: var(--dim); }
  .preset {
    padding: 6px 11px; border-radius: 3px;
    border: 1px solid var(--line); background: transparent; color: var(--fg);
    font-size: 12.5px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
  }
  .preset:hover { border-color: var(--accent); }
  .preset.on, .propio.on .preset { border-color: var(--accent); background: var(--accent); color: var(--accent-fg); }
  /* Preset propio: el mismo botón partido en dos, con la × pegada a la derecha. */
  .propio { display: inline-flex; align-items: stretch; }
  .propio .preset { border-top-right-radius: 0; border-bottom-right-radius: 0; border-right: none; }
  .borrar {
    padding: 0 8px; border-radius: 0 3px 3px 0;
    border: 1px solid var(--line); background: transparent; color: var(--dim);
    font-size: 14px; line-height: 1; cursor: pointer;
    transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
  }
  .borrar:hover { color: var(--accent); border-color: var(--accent); }
  .propio.on .borrar { border-color: var(--accent); background: #a50d26; color: var(--accent-fg); }
  .guardar {
    padding: 6px 10px; border-radius: 3px;
    border: 1px dashed var(--line); background: transparent; color: var(--dim);
    font-size: 12.5px; font-weight: 600; cursor: pointer;
    transition: color 0.2s ease, border-color 0.2s ease;
  }
  .guardar:hover { color: var(--accent); border-color: var(--accent); }

  .papel {
    position: relative;
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); box-shadow: var(--shadow);
    padding: 18px 20px;
    display: flex; flex-direction: column; gap: 14px;
  }
  .urls { padding: 20px 22px 18px; }
  .arrastra { margin-top: -6px; font-family: var(--hand); font-size: 19px; color: rgba(37, 31, 25, 0.5); }
  .cinta {
    position: absolute; top: -11px; right: 40px; width: 92px; height: 24px;
    background: var(--tape); opacity: 0.85; transform: rotate(3deg);
  }
  .entre { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .etiqueta { font-family: var(--mono); font-size: 10.5px; letter-spacing: 0.16em; color: var(--paper-dim); }
  .separada { margin-bottom: 8px; }
  .pegada { font-family: var(--mono); font-size: 10.5px; color: var(--ok); }
  .sub { font-size: 12.5px; font-weight: 600; color: rgba(37, 31, 25, 0.62); }

  textarea, .campo {
    border-radius: 4px; border: 1px solid var(--paper-line);
    background: var(--paper-field); color: var(--paper-fg);
    font-family: var(--mono); outline: none;
    transition: border-color 0.2s ease;
  }
  textarea:focus, .campo:focus { border-color: var(--accent); }
  textarea {
    width: 100%; resize: none; field-sizing: content; min-height: 66px; max-height: 180px;
    font-size: 13.5px; line-height: 1.6; padding: 11px 14px;
  }
  .campo { font-size: 13px; padding: 8px 11px; }
  .tiempo { width: 96px; }

  .previa {
    display: flex; align-items: center; gap: 14px;
    padding: 10px; border-radius: 4px; background: rgba(37, 31, 25, 0.04);
  }
  .previa img, .hueco-img {
    width: 96px; height: 54px; flex: 0 0 auto;
    border-radius: 3px; object-fit: cover; display: block;
    background: rgba(37, 31, 25, 0.08);
  }
  .datos { min-width: 0; flex: 1 1 auto; display: flex; flex-direction: column; gap: 3px; }
  .titulo { font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .barra-carga { display: block; height: 12px; border-radius: 3px; background: rgba(37, 31, 25, 0.08); }
  .barra-carga.fina { height: 9px; margin-top: 5px; }
  .cargando, .barra-carga { animation: latido 1.2s ease-in-out infinite; }
  @keyframes latido { 50% { opacity: 0.45; } }

  .rejilla { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px; align-items: start; }
  .grupos { display: flex; flex-direction: column; gap: 12px; }
  .grupo { display: flex; flex-direction: column; gap: 7px; }
  .extras { gap: 0; }

  .recorte { display: flex; align-items: center; gap: 10px; }

  .lista { gap: 10px; }
  .botones-lista { display: flex; gap: 6px; }
  .secundario.chico { padding: 5px 10px; font-size: 12px; }
  .videos { display: flex; flex-direction: column; max-height: 340px; overflow-y: auto; }
  .video {
    display: flex; align-items: center; gap: 12px; width: 100%;
    padding: 7px 4px; border: none; border-top: 1px solid var(--paper-rule);
    background: transparent; color: var(--paper-fg); text-align: left; cursor: pointer;
  }
  .caja {
    flex: 0 0 auto; width: 18px; height: 18px; border-radius: 3px;
    display: flex; align-items: center; justify-content: center;
    border: 1.5px solid rgba(37, 31, 25, 0.3); background: transparent;
    font-size: 12px; font-weight: 800; color: var(--accent-fg);
    transition: background-color 0.2s ease, border-color 0.2s ease;
  }
  .caja.on { border-color: var(--accent); background: var(--accent); }
  .n { width: 20px; flex: 0 0 auto; font-family: var(--mono); font-size: 11px; color: rgba(37, 31, 25, 0.45); }
  .mini {
    width: 64px; height: 36px; flex: 0 0 auto; display: block;
    border-radius: 3px; object-fit: cover; background: rgba(37, 31, 25, 0.08);
  }
  .nombre-video {
    flex: 1 1 auto; min-width: 0; font-size: 14px; font-weight: 600;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  .hueco-arriba { margin-top: 4px; }
  .quedara { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
  .mano-roja { flex: 0 0 auto; font-family: var(--hand); font-size: 19px; color: var(--accent); }
  .muestra { min-width: 0; font-family: var(--mono); font-size: 12px; word-break: break-all; }
  .flecha { color: rgba(37, 31, 25, 0.4); }
  .carpeta { display: flex; align-items: center; gap: 10px; }
  .ruta {
    flex: 1 1 auto; min-width: 0;
    font-family: var(--mono); font-size: 12.5px; padding: 9px 11px;
    border-radius: 4px; background: var(--paper-field); border: 1px solid var(--paper-line);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .secundario {
    flex: 0 0 auto; display: flex; align-items: center; gap: 6px;
    padding: 8px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.18); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: transform 0.2s ease, background-color 0.2s ease;
  }
  .secundario:hover { background: rgba(37, 31, 25, 0.05); transform: translateY(-1px); }

  .terminal { border-radius: var(--r); background: var(--term); color: var(--term-fg); overflow: hidden; }
  .plegar {
    width: 100%; display: flex; align-items: center; justify-content: space-between; gap: 10px;
    padding: 12px 18px; border: none; background: transparent;
    color: rgba(240, 235, 226, 0.7); font-family: var(--mono); font-size: 11px; letter-spacing: 0.14em;
    cursor: pointer;
  }
  .plegar:hover { color: var(--term-fg); }
  .chevron { display: inline-block; transition: transform 0.25s ease; }
  .chevron.abierto { transform: rotate(180deg); }
  .panel { display: grid; grid-template-rows: 0fr; transition: grid-template-rows 0.3s ease; }
  .panel.abierto { grid-template-rows: 1fr; }
  .interior { min-height: 0; overflow: hidden; }
  .linea {
    padding: 0 18px 16px;
    font-family: var(--mono); font-size: 12.5px; line-height: 1.7; word-break: break-all;
  }
  .dolar { color: #7fd8a4; margin-right: 0.6em; }

  /* Pegada abajo mientras se hace scroll, con un degradado para que no corte en seco. */
  .accion {
    position: sticky; bottom: 0; z-index: 5;
    display: flex; align-items: center; justify-content: flex-end; gap: 14px;
    margin: 0 -36px -36px; padding: 22px 36px 24px;
    background: linear-gradient(to bottom, transparent, var(--bg) 42%);
  }
  .resumen { font-size: 13px; color: var(--dim); }
  .descargar {
    display: flex; align-items: center; gap: 9px;
    padding: 13px 22px; border-radius: 4px; border: none;
    background: var(--accent); color: var(--accent-fg);
    font-size: 15px; font-weight: 700; cursor: pointer;
    box-shadow: 0 8px 18px rgba(200, 16, 46, 0.28);
    transition: transform 0.2s ease, background-color 0.2s ease, opacity 0.2s ease;
  }
  .descargar:hover:not(:disabled) { background: var(--accent-hot); transform: translateY(-2px); }
  .descargar:disabled { opacity: 0.5; cursor: default; box-shadow: none; }
</style>
