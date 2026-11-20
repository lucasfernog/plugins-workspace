<script>
  import {
    getItem,
    setItem,
    removeItem
  } from '@tauri-apps/plugin-secure-storage'

  let { onMessage } = $props()

  let key = $state('access-token')
  let value = $state('')

  async function read() {
    try {
      const stored = await getItem(key)
      onMessage(stored === null ? `"${key}" is not set` : `${key} = ${stored}`)
    } catch (error) {
      onMessage(error)
    }
  }

  async function write() {
    try {
      await setItem(key, value)
      onMessage(`Stored "${key}"`)
    } catch (error) {
      onMessage(error)
    }
  }

  async function remove() {
    try {
      await removeItem(key)
      onMessage(`Removed "${key}"`)
    } catch (error) {
      onMessage(error)
    }
  }
</script>

<div class="flex flex-col childre:grow gap-1">
  <div class="flex flex-col flex-row-md gap-4">
    <div class="flex items-center gap-1">
      Key:
      <input class="grow input" bind:value={key} />
    </div>

    <div class="flex items-center gap-1">
      Value:
      <input class="grow input" bind:value />
    </div>

    <div>
      <button class="btn" onclick={read}>Read</button>
      <button class="btn" onclick={write}>Write</button>
      <button class="btn" onclick={remove}>Remove</button>
    </div>
  </div>
  <div>
    Items are kept in the operating system's secure storage (Keychain,
    Credential Manager, Secret Service or Android Keystore).
  </div>
</div>
