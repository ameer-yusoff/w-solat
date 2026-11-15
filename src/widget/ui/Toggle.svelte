<script>
  let { checked = $bindable(false), onchange, label = '' } = $props();
</script>

<label class="tg" aria-label={label}>
  <input type="checkbox" bind:checked onchange={() => onchange?.(checked)} />
  <span class="track"><span class="thumb"></span></span>
</label>

<style>
  .tg { position: relative; display: inline-block; width: 34px; height: 19px; flex: none; cursor: pointer; }
  input { opacity: 0; width: 0; height: 0; position: absolute; }
  .track {
    position: absolute;
    inset: 0;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.12);
    transition: background 0.25s, border-color 0.25s;
  }
  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: #f4f1e8;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    transition: transform 0.3s var(--spring), background 0.25s;
  }
  input:checked + .track {
    background: linear-gradient(135deg, var(--gold-soft), var(--gold-deep));
    border-color: var(--gold);
  }
  input:checked + .track .thumb { transform: translateX(15px); background: #1a1406; }
</style>
