<script module>
  let openId = $state(null);
  let seq = 0;
</script>

<script>
  import { fly } from 'svelte/transition';
  import { ICON } from '../../lib/icons.js';

  let { options = [], value = $bindable(), placeholder = 'Pilih', disabled = false, onchange } = $props();
  const id = ++seq;
  const open = $derived(openId === id);
  const current = $derived(options.find((o) => o.value === value));

  function toggle(e) {
    e.stopPropagation();
    if (disabled) return;
    openId = open ? null : id;
  }
  function pick(e, o) {
    e.stopPropagation();
    value = o.value;
    openId = null;
    onchange?.(o);
  }
</script>

<svelte:window onclick={() => open && (openId = null)} />

<div class="sel" class:open class:disabled>
  <button class="btn" {disabled} onclick={toggle}>
    {#if current?.icon}<span class="ic">{@html ICON[current.icon]}</span>{/if}
    <span class="lbl" class:ph={!current}>{current?.label ?? placeholder}</span>
    <span class="chev">{@html ICON.chevron}</span>
  </button>
  {#if open}
    <div class="menu" transition:fly={{ y: -4, duration: 160 }}>
      {#each options as o (o.value)}
        <button class="opt" class:on={o.value === value} onclick={(e) => pick(e, o)}>
          {#if o.icon}<span class="ic">{@html ICON[o.icon]}</span>{/if}
          <span>{o.label}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sel { position: relative; }
  .btn {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 7px 9px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(233, 200, 115, 0.18);
    font-size: 11.5px;
    text-align: left;
    transition: border-color 0.2s, background 0.2s;
  }
  .btn:hover:not(:disabled) { border-color: rgba(233, 200, 115, 0.4); }
  .open .btn { border-color: var(--gold); background: rgba(0, 0, 0, 0.4); }
  .disabled .btn { opacity: 0.4; cursor: default; }
  .lbl { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ph { color: var(--ink-faint); }
  .ic, .chev { width: 13px; height: 13px; display: grid; flex: none; color: var(--gold); }
  .chev { color: var(--ink-faint); transition: transform 0.2s; }
  .open .chev { transform: rotate(180deg); }
  .ic :global(svg), .chev :global(svg) { width: 100%; height: 100%; }

  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    max-height: 168px;
    overflow-y: auto;
    padding: 4px;
    border-radius: 10px;
    background: #0b1512;
    border: 1px solid rgba(233, 200, 115, 0.3);
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.6);
  }
  .opt {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 8px;
    border-radius: 7px;
    font-size: 11px;
    text-align: left;
    color: var(--ink-dim);
  }
  .opt:hover { background: rgba(233, 200, 115, 0.1); color: var(--ink); }
  .opt.on { color: var(--gold); background: rgba(233, 200, 115, 0.08); }
</style>
