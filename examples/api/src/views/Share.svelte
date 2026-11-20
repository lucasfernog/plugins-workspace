<script>
  import { share } from '@tauri-apps/plugin-share'
  import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'
  import { appDataDir, join } from '@tauri-apps/api/path'

  export let onMessage

  let title = 'Tauri'
  let text = 'Build smaller, faster and more secure apps'
  let url = 'https://tauri.app'

  // anchors the share sheet to the clicked button (used on macOS and iPad)
  function anchor(event) {
    return { anchor: event.currentTarget.getBoundingClientRect() }
  }

  function shareText(event) {
    share(
      {
        title: title || undefined,
        text: text || undefined,
        url: url || undefined
      },
      anchor(event)
    )
      .then(() => onMessage('share sheet opened'))
      .catch(onMessage)
  }

  async function shareFile(event) {
    const options = anchor(event)
    try {
      // the example's share scope allows $APPDATA/**
      const name = 'share-example.txt'
      await writeTextFile(name, text || 'Hello from Tauri', {
        baseDir: BaseDirectory.AppData
      })
      await share(
        {
          title: title || undefined,
          files: [await join(await appDataDir(), name)]
        },
        options
      )
      onMessage('share sheet opened')
    } catch (e) {
      onMessage(e)
    }
  }
</script>

<div class="flex flex-col gap-2">
  <input
    class="input"
    id="share-title"
    placeholder="Title"
    bind:value={title}
  />
  <input class="input" id="share-text" placeholder="Text" bind:value={text} />
  <input class="input" id="share-url" placeholder="URL" bind:value={url} />
  <div class="flex flex-row gap-2">
    <button class="btn" id="share-text-button" on:click={shareText}
      >Share text and URL</button
    >
    <button class="btn" id="share-file-button" on:click={shareFile}
      >Share a file</button
    >
  </div>
</div>
