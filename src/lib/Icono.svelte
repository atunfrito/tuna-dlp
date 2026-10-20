<script>
  // El icono de la app (brand/tunadlp-icono.svg) en pequeño. En modo oscuro va
  // en negativo: fondo crema y círculo oscuro, para que no se pierda contra el
  // fondo de la app y la lata siga destacando.
  let { tam = 18 } = $props();
</script>

<svg width={tam} height={tam} viewBox="0 0 100 100" aria-hidden="true">
  <rect class="fondo" width="100" height="100" rx="22" />
  <circle class="disco" cx="50" cy="50" r="36" />
  <circle class="aro" cx="50" cy="50" r="29.5" fill="none" stroke-width="2" />
  <rect x="37" y="27" width="26" height="36" rx="13" fill="#c8102e" />
  <rect x="43.5" y="33" width="13" height="15" rx="6.5" fill="#f7f1e6" />
  <circle cx="50" cy="56" r="3.6" fill="#8f0a20" />
</svg>

<style>
  .fondo { fill: #14100d; transition: fill 0.45s ease; }
  .disco { fill: #f7f1e6; transition: fill 0.45s ease; }
  .aro { stroke: rgba(37, 31, 25, 0.16); transition: stroke 0.45s ease; }
  :global(:root[data-theme="dark"]) .fondo { fill: #f7f1e6; }
  :global(:root[data-theme="dark"]) .disco { fill: #14100d; }
  :global(:root[data-theme="dark"]) .aro { stroke: rgba(247, 241, 230, 0.16); }
</style>
