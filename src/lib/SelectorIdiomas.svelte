<script>
  // Idiomas de subtítulos: los elegidos como chips y un campo con desplegable
  // que reconoce lo que escribes ("ingl", "japo", "fr"…). valor es la lista de
  // códigos para yt-dlp ("es,en"). del vídeo: lo que tiene de verdad, si hay.
  import { COMUNES, TODOS, nombre, normal } from "./idiomas.js";
  import { t } from "./i18n.svelte.js";

  let { valor = $bindable(), manuales = null, automaticos = null, alCambiar = () => {} } = $props();

  let texto = $state("");
  let abierto = $state(false);
  let activo = $state(0);
  let campo;

  const elegidos = $derived(valor.split(",").map((x) => x.trim()).filter(Boolean));
  const hayVideo = $derived(manuales != null);

  // Cada opción: [código, nombre, "autor" | "auto" | ""]. Primero lo que tiene el vídeo.
  const opciones = $derived.by(() => {
    const q = normal(texto.trim());
    const tipo = (c) => (manuales?.includes(c) ? "autor" : automaticos?.includes(c) ? "auto" : "");
    const todos = [...new Set([...(manuales ?? []), ...(automaticos ?? []), ...(q ? TODOS : COMUNES)])];
    const orden = { autor: 0, auto: 1, "": 2 };
    return todos
      .filter((c) => !elegidos.includes(c))
      .map((c) => [c, nombre(c), tipo(c)])
      .filter(([c, n]) => !q || normal(n).includes(q) || normal(c).startsWith(q))
      .sort((a, b) =>
        // Si se ha escrito algo, primero los que empiezan así ("in" → inglés antes que "finés").
        (q ? Number(!normal(a[1]).startsWith(q)) - Number(!normal(b[1]).startsWith(q)) : 0) ||
        orden[a[2]] - orden[b[2]] ||
        (q ? a[1].localeCompare(b[1], "es") : 0)
      )
      .slice(0, 60);
  });
  // Un código que no está en la lista también vale: yt-dlp sabrá qué hacer con él.
  const codigoSuelto = $derived(/^[a-z]{2,3}(-[a-z0-9]{2,8})*$/i.test(texto.trim()) && !opciones.some(([c]) => c === texto.trim()));

  function elegir(c) {
    valor = [...elegidos, c].join(",");
    texto = "";
    activo = 0;
    alCambiar();
    campo?.focus();
  }
  function quitar(c) {
    valor = elegidos.filter((x) => x !== c).join(",");
    alCambiar();
  }

  function tecla(e) {
    const n = opciones.length;
    if (e.key === "ArrowDown") { e.preventDefault(); abierto = true; activo = n ? (activo + 1) % n : 0; }
    else if (e.key === "ArrowUp") { e.preventDefault(); activo = n ? (activo - 1 + n) % n : 0; }
    else if (e.key === "Enter") {
      e.preventDefault();
      if (opciones[activo]) elegir(opciones[activo][0]);
      else if (codigoSuelto) elegir(texto.trim());
    }
    else if (e.key === "Escape") abierto = false;
    else if (e.key === "Backspace" && !texto && elegidos.length) quitar(elegidos.at(-1));
  }

  // La opción activa siempre a la vista al moverse con las flechas.
  let lista = $state();
  $effect(() => {
    lista?.children[activo]?.scrollIntoView({ block: "nearest" });
  });
</script>

<div class="selector">
  {#if elegidos.length}
    <div class="elegidos">
      {#each elegidos as c (c)}
        <span class="elegido">
          {nombre(c)}
          <button type="button" aria-label={t("idiomas.quitar", { nombre: nombre(c) })} onclick={() => quitar(c)}>×</button>
        </span>
      {/each}
    </div>
  {/if}

  <div class="buscador">
    <input
      bind:this={campo}
      bind:value={texto}
      class="campo"
      placeholder={t("idiomas.buscar")}
      spellcheck="false"
      role="combobox"
      aria-expanded={abierto}
      aria-controls="idiomas-lista"
      onfocus={() => (abierto = true)}
      onblur={() => (abierto = false)}
      oninput={() => ((abierto = true), (activo = 0))}
      onkeydown={tecla}
    />
    {#if abierto && (opciones.length || texto.trim())}
      <!-- mousedown con preventDefault: elegir sin que el campo pierda el foco y cierre la lista. -->
      <div class="desplegable" id="idiomas-lista" role="listbox" bind:this={lista}>
        {#each opciones as [c, n, tipo], i (c)}
          <button
            type="button" role="option" class="opcion" class:activa={i === activo} aria-selected={i === activo}
            onmousedown={(e) => { e.preventDefault(); elegir(c); }}
            onmouseenter={() => (activo = i)}
          >
            <span class="nombre">{n}</span>
            <span class="codigo">{c}</span>
            {#if tipo === "autor"}<span class="marca autor">{t("idiomas.en_video")}</span>
            {:else if tipo === "auto"}<span class="marca">{t("idiomas.auto")}</span>{/if}
          </button>
        {:else}
          {#if codigoSuelto}
            <button type="button" role="option" class="opcion activa" aria-selected="true"
              onmousedown={(e) => { e.preventDefault(); elegir(texto.trim()); }}>
              <span class="nombre">{nombre(texto.trim())}</span><span class="codigo">{texto.trim()}</span>
            </button>
          {:else}
            <span class="nada">{t("idiomas.nada")}</span>
          {/if}
        {/each}
      </div>
    {/if}
  </div>
  {#if hayVideo && !manuales.length && !automaticos?.length}
    <span class="nota">{t("idiomas.sin_subs")}</span>
  {/if}
</div>

<style>
  .selector { display: flex; flex-direction: column; gap: 8px; }
  .elegidos { display: flex; gap: 6px; flex-wrap: wrap; }
  .elegido {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 5px 6px 5px 11px; border-radius: 4px;
    background: var(--accent); color: var(--accent-fg);
    font-size: 13px; font-weight: 600;
  }
  .elegido button {
    width: 18px; height: 18px; padding: 0; border: none; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    background: transparent; color: inherit; font-size: 14px; line-height: 1; cursor: pointer;
    transition: background-color 0.15s ease;
  }
  .elegido button:hover { background: rgba(0, 0, 0, 0.2); }

  .buscador { position: relative; }
  .campo {
    width: 100%; font-family: var(--mono); font-size: 13px; padding: 8px 11px;
    border-radius: 4px; border: 1px solid var(--paper-line);
    background: var(--paper-field); color: var(--paper-fg); outline: none;
    transition: border-color 0.2s ease;
  }
  .campo:focus { border-color: var(--accent); }
  .desplegable {
    position: absolute; left: 0; right: 0; top: calc(100% + 4px); z-index: 10;
    max-height: 244px; overflow-y: auto; padding: 4px;
    background: var(--paper); border: 1px solid var(--paper-line); border-radius: var(--r);
    box-shadow: 0 14px 30px rgba(37, 31, 25, 0.18);
  }
  .opcion {
    display: flex; align-items: baseline; gap: 8px; width: 100%;
    padding: 7px 9px; border: none; border-radius: 3px;
    background: transparent; color: var(--paper-fg); text-align: left; cursor: pointer;
  }
  .opcion.activa { background: rgba(37, 31, 25, 0.06); box-shadow: inset 2px 0 0 var(--accent); }
  .nombre { font-size: 13.5px; font-weight: 600; }
  .codigo { font-family: var(--mono); font-size: 11px; color: var(--paper-dim); }
  .marca {
    margin-left: auto; padding: 2px 6px; border-radius: 3px;
    font-family: var(--mono); font-size: 10px; color: var(--paper-dim); background: rgba(37, 31, 25, 0.06);
  }
  .marca.autor { color: #2f7a4c; background: #dcefe2; }
  .nada, .nota { display: block; font-family: var(--hand); font-size: 19px; color: rgba(37, 31, 25, 0.5); }
  .nada { padding: 6px 9px; }
</style>
