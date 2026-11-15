<script>
  let { options = [], value = $bindable(), onchange } = $props();
</script>

<div class="pills">
  {#each options as o (o.value)}
    <button
      type="button"
      class:on={o.value === value}
      onclick={(e) => { e.stopPropagation(); value = o.value; onchange?.(o.value); }}
    >{o.label}</button>
  {/each}
</div>

<style>
  .pills {
    display: flex;
    gap: 3px;
    padding: 3px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.28);
    border: 1px solid rgba(233, 200, 115, 0.12);
  }
  button {
    flex: 1;
    padding: 5px 2px;
    border-radius: 7px;
    font-size: 10.5px;
    color: var(--ink-dim);
    white-space: nowrap;
    transition: background 0.2s, color 0.2s;
  }
  button:hover { color: var(--ink); }
  button.on {
    color: #1a1406;
    font-weight: 600;
    background: linear-gradient(135deg, var(--gold-soft), var(--gold) 55%, var(--gold-deep));
    box-shadow: 0 2px 6px rgba(185, 141, 60, 0.35);
  }
</style>
