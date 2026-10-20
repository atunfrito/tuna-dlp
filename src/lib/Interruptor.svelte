<script>
  let { etiqueta, pista = "", valor = $bindable(), alCambiar = () => {}, borde = "abajo" } = $props();
</script>

<button type="button" class="fila {borde}" role="switch" aria-checked={valor} onclick={() => { valor = !valor; alCambiar(); }}>
  <span class="textos">
    <span class="etiqueta">{etiqueta}</span>
    {#if pista}<span class="pista">{pista}</span>{/if}
  </span>
  <span class="pista-track" class:on={valor}><span class="bola"></span></span>
</button>

<style>
  .fila {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    width: 100%; padding: 8px 0; border: none; background: transparent;
    color: var(--paper-fg); text-align: left; cursor: pointer;
  }
  .abajo { border-bottom: 1px solid var(--paper-rule); }
  .arriba { border-top: 1px solid var(--paper-rule); padding: 9px 0; }
  .textos { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
  .etiqueta { font-size: 14px; font-weight: 600; }
  .pista { font-size: 12px; color: var(--paper-dim); }
  .pista-track {
    flex: 0 0 auto; position: relative; width: 36px; height: 20px; border-radius: 10px;
    background: rgba(37, 31, 25, 0.18); transition: background-color 0.2s ease;
  }
  .pista-track.on { background: var(--accent); }
  .bola {
    position: absolute; top: 3px; left: 3px; width: 14px; height: 14px; border-radius: 50%;
    background: var(--paper); box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    transition: transform 0.22s ease;
  }
  .on .bola { transform: translateX(16px); }
</style>
