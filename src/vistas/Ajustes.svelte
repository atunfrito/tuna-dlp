<script>
  import { invoke } from "@tauri-apps/api/core";
  import Chips from "../lib/Chips.svelte";
  import Interruptor from "../lib/Interruptor.svelte";
  import { version } from "../../package.json";
  import { ajustes, app, comprobar, actualizar } from "../lib/estado.svelte.js";
  import { t, tError } from "../lib/i18n.svelte.js";

  // [título, pista, botón, fondo] por estado, tal cual el prototipo.
  const U = $derived({
    idle: [t("upd.idle"), t("upd.idle_pista"), t("upd.comprobar"), "rgba(37, 31, 25, 0.04)"],
    checking: [t("upd.checking"), t("upd.checking_pista"), "", "rgba(37, 31, 25, 0.04)"],
    available: [t("upd.available", { version: app.ultima }), t("upd.available_pista"), t("upd.actualizar"), "#fbe3e6"],
    updating: [t("upd.updating"), t("upd.updating_pista"), "", "#fbe3e6"],
    ok: [t("upd.ok", { version: app.ytv }), t("upd.ok_pista"), t("upd.otra_vez"), "#dcefe2"],
    error: ["", "", t("upd.comprobar"), "#fbe3e6"],
  }[app.upd]);

  const accion = () => (app.upd === "available" ? actualizar() : comprobar());
</script>

<div class="vista">
  <h1>{t("aj.titulo")}<span class="acento">.</span></h1>

  <section class="papel">
    <span class="cinta"></span>
    <span class="etiqueta">{t("aj.motor")}</span>
    <div class="versiones">
      <div class="caja"><span class="nombre">tuna-dlp</span><span class="numero">{version}</span></div>
      <div class="caja"><span class="nombre">yt-dlp</span><span class="numero">{app.ytv || "…"}</span></div>
      <div class="caja"><span class="nombre">ffmpeg</span><span class="numero">{app.ffmpeg || "…"}</span></div>
    </div>
    <div class="actualizar" style="background: {U[3]}">
      <span class="textos">
        {#if app.upd === "error"}
          <span class="fallo" data-seleccionable>{tError(app.updError)}</span>
        {:else}
          <span class="upd-titulo">{U[0]}</span>
          <span class="upd-pista">{U[1]}</span>
        {/if}
      </span>
      {#if U[2]}
        <button type="button" class="boton" class:primario={app.upd === "available"} onclick={accion}>{U[2]}</button>
      {/if}
    </div>
    <div class="grupo">
      <span class="sub">{t("aj.canal")}</span>
      <Chips bind:valor={ajustes.canal} alCambiar={comprobar} opciones={[["stable", t("aj.estable")], ["nightly", "nightly"]]} />
    </div>
  </section>

  <section class="papel">
    <span class="etiqueta">{t("aj.descargas")}</span>
    <div class="opcion">
      <span class="textos"><span class="titulo-op">{t("aj.a_la_vez")}</span><span class="pista">{t("aj.a_la_vez_pista")}</span></span>
      <Chips bind:valor={ajustes.paralelo} opciones={[1, 2, 3, 4, 5].map((n) => [n, String(n)])} />
    </div>
    <div class="opcion">
      <span class="textos"><span class="titulo-op">{t("aj.velocidad")}</span><span class="pista">{t("aj.velocidad_pista")}</span></span>
      <Chips bind:valor={ajustes.velocidad} opciones={["sin límite", "10 MB/s", "5 MB/s", "2 MB/s"].map((v) => [v, v === "sin límite" ? t("aj.sin_limite") : v])} />
    </div>
    <div class="opcion">
      <span class="textos"><span class="titulo-op">{t("aj.cookies")}</span><span class="pista">{t("aj.cookies_pista")}</span></span>
      <Chips bind:valor={ajustes.navegador} opciones={["ninguno", "firefox", "brave", "chrome"].map((v) => [v, v === "ninguno" ? t("aj.ninguno") : v])} />
    </div>
    <div class="interruptores">
      <Interruptor borde="arriba" bind:valor={ajustes.abrirConWindows} etiqueta={t("aj.con_windows")} pista={t("aj.con_windows_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.portapapeles} etiqueta={t("aj.portapapeles")} pista={t("aj.portapapeles_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.avisar} etiqueta={t("aj.avisar")} pista={t("aviso_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.abrirAlTerminar} etiqueta={t("aj.abrir_al_terminar")} />
      <Interruptor borde="arriba" bind:valor={ajustes.archivo} etiqueta={t("aj.archivo")} pista={t("aj.archivo_pista")} />
    </div>
  </section>

  <section class="papel">
    <span class="etiqueta">{t("aj.opcionales")}</span>
    <span class="aviso">{t("aj.opcionales_pista")}</span>
    <div class="interruptores">
      <Interruptor borde="arriba" bind:valor={ajustes.verComando} etiqueta={t("aj.comando")} pista={t("aj.comando_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.recortar} etiqueta={t("aj.recortar")} pista={t("aj.recortar_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.plantilla} etiqueta={t("aj.plantilla")} pista={t("aj.plantilla_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.historial} etiqueta={t("aj.historial")} pista={t("aj.historial_pista")} />
      <Interruptor borde="arriba" bind:valor={ajustes.suscripciones} etiqueta={t("aj.suscripciones")} pista={t("aj.suscripciones_pista")} />
    </div>
  </section>

  <section class="papel">
    <span class="etiqueta">{t("aj.apariencia")}</span>
    <div class="opcion">
      <span class="titulo-op">{t("aj.tema")}</span>
      <Chips bind:valor={ajustes.tema} opciones={[["light", t("aj.claro")], ["dark", t("aj.oscuro")], ["system", t("aj.sistema")]]} />
    </div>
    <div class="opcion">
      <span class="titulo-op">{t("aj.idioma")}</span>
      <!-- Cada idioma con su propio nombre, para encontrarlo aunque no entiendas el otro. -->
      <Chips bind:valor={ajustes.idioma} opciones={[["es", "español"], ["en", "english"], ["system", t("aj.sistema")]]} />
    </div>
  </section>

  <section class="nota">
    <span class="textos">
      <span class="mano">{t("aj.cueva")}</span>
      <span class="explica">{t("aj.cueva_texto")}</span>
    </span>
    <div class="enlaces">
      <button type="button" class="enlace" onclick={() => invoke("abrir_web")}>atunfrito.es ↗</button>
      <button type="button" class="enlace" onclick={() => invoke("abrir_registros")}>{t("aj.registros")}</button>
    </div>
  </section>
</div>

<style>
  .vista { position: relative; padding: 30px 36px 36px; display: flex; flex-direction: column; gap: 18px; max-width: 820px; }
  h1 { margin: 0; font-family: var(--serif); font-weight: 400; font-size: 44px; line-height: 1; letter-spacing: -0.022em; }
  .acento { color: var(--accent); }

  .papel {
    position: relative;
    background: var(--paper); color: var(--paper-fg);
    border-radius: var(--r); box-shadow: var(--shadow);
    padding: 20px 22px; display: flex; flex-direction: column; gap: 14px;
  }
  .cinta {
    position: absolute; top: -11px; left: 36px; width: 84px; height: 24px;
    background: var(--tape); opacity: 0.85; transform: rotate(-3deg);
  }
  .etiqueta { font-family: var(--mono); font-size: 10.5px; letter-spacing: 0.16em; color: var(--paper-dim); }
  .sub { font-size: 12.5px; font-weight: 600; color: rgba(37, 31, 25, 0.62); }
  .grupo { display: flex; flex-direction: column; gap: 7px; }

  .versiones { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }
  .caja {
    padding: 12px 14px; border-radius: 4px; border: 1px solid rgba(37, 31, 25, 0.12);
    display: flex; flex-direction: column; gap: 4px;
  }
  .nombre { font-size: 12px; color: var(--paper-dim); }
  .numero { font-family: var(--mono); font-size: 15px; font-weight: 600; }

  .actualizar {
    display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap;
    padding: 12px 14px; border-radius: 4px; transition: background-color 0.3s ease;
  }
  .textos { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .upd-titulo { font-family: var(--serif); font-size: 22px; line-height: 1.1; }
  .upd-pista { font-family: var(--hand); font-size: 19px; line-height: 1.1; color: rgba(37, 31, 25, 0.6); }
  .pista { font-size: 12.5px; color: rgba(37, 31, 25, 0.6); }
  .fallo { font-family: var(--mono); font-size: 12px; color: var(--accent); white-space: pre-wrap; word-break: break-word; }
  .boton {
    flex: 0 0 auto; padding: 9px 14px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.2); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 700; cursor: pointer;
    transition: transform 0.2s ease, background-color 0.2s ease;
  }
  .boton:hover { transform: translateY(-1px); }
  .boton.primario { border: none; background: var(--accent); color: var(--accent-fg); }
  .boton.primario:hover { background: var(--accent-hot); }

  .opcion { display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap; }
  .titulo-op { font-size: 14px; font-weight: 600; }
  .opcion .pista { font-size: 12px; color: var(--paper-dim); }
  .interruptores { display: flex; flex-direction: column; gap: 4px; }
  .aviso { margin-top: -6px; font-size: 13px; color: rgba(37, 31, 25, 0.6); }

  .nota {
    background: var(--note); color: var(--paper-fg);
    border-radius: 4px; box-shadow: var(--shadow);
    padding: 18px 22px; display: flex; align-items: center; justify-content: space-between; gap: 16px; flex-wrap: wrap;
    transform: rotate(-0.4deg);
  }
  .nota .textos { gap: 4px; }
  .mano { font-family: var(--hand); font-size: 24px; color: var(--accent); line-height: 1; }
  .explica { font-size: 13px; color: rgba(37, 31, 25, 0.66); }
  .enlaces { display: flex; gap: 8px; flex-wrap: wrap; }
  .enlace {
    padding: 8px 12px; border-radius: 4px;
    border: 1px solid rgba(37, 31, 25, 0.2); background: transparent; color: var(--paper-fg);
    font-size: 13px; font-weight: 600; cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .enlace:hover { background: rgba(37, 31, 25, 0.05); }
</style>
