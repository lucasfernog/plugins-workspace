<script>
  import { onMount } from 'svelte'
  import { listen } from '@tauri-apps/api/event'
  import {
    schedule,
    cancel,
    cancelAll,
    getScheduled
  } from '@tauri-apps/plugin-background-tasks'

  export let onMessage

  // registered in src-tauri/src/lib.rs and listed in Info.plist
  const TASK = 'com.tauri.api.background-task'

  let delaySeconds = 0
  let requiresNetwork = false
  let scheduled = null

  async function refresh() {
    try {
      scheduled = await getScheduled()
    } catch (e) {
      onMessage(e)
    }
  }

  function run(action) {
    return () =>
      action()
        .then((res) => onMessage(res ?? 'ok'))
        .catch(onMessage)
        .finally(refresh)
  }

  function request(extra) {
    return {
      name: TASK,
      input: { scheduledAt: new Date().toISOString() },
      delay: delaySeconds * 1000,
      requiresNetwork,
      ...extra
    }
  }

  onMount(() => {
    refresh()
    // emitted by the task when it runs while the app is alive
    const unlisten = listen('background-task', (event) => {
      onMessage({ ran: TASK, input: event.payload })
      refresh()
    })
    return () => unlisten.then((fn) => fn())
  })
</script>

<div>
  <p>
    Scheduled:
    <b id="background-tasks-scheduled"
      >{scheduled ? scheduled.map((t) => t.name).join(', ') || 'none' : '...'}</b
    >
  </p>

  <div class="flex gap-2 children:grow items-center">
    <label for="background-tasks-delay">Delay (seconds)</label>
    <input
      class="input"
      id="background-tasks-delay"
      type="number"
      min="0"
      bind:value={delaySeconds}
    />
  </div>
  <label>
    <input type="checkbox" bind:checked={requiresNetwork} />
    Requires network
  </label>

  <br />

  <button
    class="btn"
    id="background-tasks-schedule"
    on:click={run(() => schedule(request()))}>schedule once</button
  >
  <button
    class="btn"
    id="background-tasks-schedule-processing"
    on:click={run(() => schedule(request({ kind: 'processing' })))}
    >schedule processing</button
  >
  <button
    class="btn"
    id="background-tasks-schedule-periodic"
    on:click={run(() => schedule(request({ interval: 15 * 60 * 1000 })))}
    >schedule every 15 min</button
  >
  <button
    class="btn"
    id="background-tasks-cancel"
    on:click={run(() => cancel(TASK))}>cancel</button
  >
  <button class="btn" id="background-tasks-cancel-all" on:click={run(cancelAll)}
    >cancel all</button
  >
  <button class="btn" id="background-tasks-refresh" on:click={refresh}
    >refresh</button
  >
</div>

<br />

<p>
  Send the app to the background after scheduling. The system decides when the
  task runs: when it does while the app is alive, its input shows up in the
  console below, and it is always written to the device log.
</p>
