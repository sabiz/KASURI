<script lang="ts">
    // Tauriウィンドウ操作APIのimport（Tauri環境のみ有効）
    let tauriWindow: any = undefined;
    if (typeof window !== "undefined" && "__TAURI__" in window) {
        // @ts-ignore
        tauriWindow = window.__TAURI__?.window?.getCurrent
            ? window.__TAURI__.window.getCurrent()
            : undefined;
    }

    async function minimizeWindow() {
        if (tauriWindow && tauriWindow.minimize) {
            await tauriWindow.minimize();
        }
    }
    async function maximizeWindow() {
        if (
            tauriWindow &&
            tauriWindow.isMaximized &&
            tauriWindow.maximize &&
            tauriWindow.unmaximize
        ) {
            const isMax = await tauriWindow.isMaximized();
            if (isMax) {
                await tauriWindow.unmaximize();
            } else {
                await tauriWindow.maximize();
            }
        }
    }
    async function closeWindow() {
        if (tauriWindow && tauriWindow.close) {
            await tauriWindow.close();
        }
    }
    import { onMount } from "svelte";

    let application_search_path_list: string[] = [];
    let application_search_interval_on_startup_minute: number = 0;
    let log_level: string = "";
    let width: number = 0;
    let auto_startup: boolean = false;
    let shortcut_key: string = "";
    let application_name_aliases: { path: string; alias: string }[] = [];

    onMount(async () => {
        // TODO: Load settings from backend
    });

    function saveSettings() {
        // TODO: Save settings to backend
    }

    // Fallback for folder selection (Electron/Tauri/legacy browsers)
    async function selectFolderFallback(): Promise<string | undefined> {
        // @ts-ignore
        if (window.__TAURI__ && window.__TAURI__.dialog) {
            // Tauri dialog
            const { open } = window.__TAURI__.dialog;
            return await open({ directory: true });
        }
        // fallback: prompt
        return prompt("Enter folder path:") || undefined;
    }
</script>

<main class="container w-screen h-full">
    <!-- Custom Window Frame -->
    <div
        class="w-screen flex items-center justify-between bg-gray-800 text-white px-2 py-1 select-none"
        style="-webkit-app-region: drag; border-top-left-radius: 0.5rem; border-top-right-radius: 0.5rem;"
    >
        <div class="font-bold text-lg pl-1" style="letter-spacing:0.05em;">
            KASURI
        </div>
        <div
            class="flex items-center gap-1"
            style="-webkit-app-region: no-drag;"
        >
            <button
                type="button"
                aria-label="Minimize"
                class="w-8 h-8 flex items-center justify-center hover:bg-gray-700 rounded transition"
                on:click={minimizeWindow}
            >
                <svg width="16" height="16" fill="currentColor"
                    ><rect y="12" width="16" height="2" rx="1" /></svg
                >
            </button>
            <button
                type="button"
                aria-label="Maximize"
                class="w-8 h-8 flex items-center justify-center hover:bg-gray-700 rounded transition"
                on:click={maximizeWindow}
            >
                <svg width="16" height="16" fill="currentColor"
                    ><rect
                        x="2"
                        y="2"
                        width="12"
                        height="12"
                        rx="2"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    /></svg
                >
            </button>
            <button
                type="button"
                aria-label="Close"
                class="w-8 h-8 flex items-center justify-center hover:bg-red-600 rounded transition"
                on:click={closeWindow}
            >
                <svg width="16" height="16" fill="currentColor"
                    ><rect
                        x="3"
                        y="3"
                        width="10"
                        height="10"
                        rx="2"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    /><line
                        x1="5"
                        y1="5"
                        x2="11"
                        y2="11"
                        stroke="currentColor"
                        stroke-width="2"
                    /><line
                        x1="11"
                        y1="5"
                        x2="5"
                        y2="11"
                        stroke="currentColor"
                        stroke-width="2"
                    /></svg
                >
            </button>
        </div>
    </div>
    <div class="kasuri-content-scroll">
        <h1 class="text-3xl mb-6">Settings</h1>
        <form class="space-y-6" on:submit|preventDefault={saveSettings}>
            <!-- Application Search Path List -->
            <div>
                <label
                    class="block font-bold mb-1"
                    for="application_search_path_list_0"
                    >Application Search Path List</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    List of directories to search for applications. Enter
                    "WindowsStoreApp" to include Windows Store apps.
                </p>
                {#each application_search_path_list as path, i}
                    <div class="flex items-center mb-1">
                        <input
                            class="input input-bordered flex-1"
                            id={`application_search_path_list_${i}`}
                            type="text"
                            bind:value={application_search_path_list[i]}
                            placeholder="Enter folder path or WindowsStoreApp"
                        />
                        {#if application_search_path_list[i] !== "WindowsStoreApp"}
                            <button
                                type="button"
                                class="ml-2 btn btn-sm btn-secondary"
                                on:click={async () => {
                                    // Try Tauri/Electron fallback first
                                    let selectedPath =
                                        await selectFolderFallback();
                                    if (
                                        !selectedPath &&
                                        typeof window !== "undefined" &&
                                        "showDirectoryPicker" in window
                                    ) {
                                        // @ts-ignore
                                        const dirHandle =
                                            await window.showDirectoryPicker();
                                        // Try to get full path (Tauri injects .path, web only has .name)
                                        if (dirHandle && "path" in dirHandle) {
                                            // @ts-ignore
                                            application_search_path_list[i] =
                                                dirHandle.path;
                                        } else if (
                                            dirHandle &&
                                            "name" in dirHandle
                                        ) {
                                            // Web fallback: only name available
                                            // (You may want to show a warning in production)
                                            // @ts-ignore
                                            application_search_path_list[i] =
                                                dirHandle.name;
                                        }
                                    } else if (selectedPath) {
                                        application_search_path_list[i] =
                                            selectedPath;
                                    }
                                }}>Select Folder</button
                            >
                        {/if}
                        <button
                            type="button"
                            class="ml-2 btn btn-sm btn-error"
                            on:click={() =>
                                application_search_path_list.splice(i, 1)}
                            >Remove</button
                        >
                    </div>
                {/each}
                <button
                    type="button"
                    class="btn btn-sm btn-primary mt-1"
                    on:click={() =>
                        (application_search_path_list = [
                            ...application_search_path_list,
                            "",
                        ])}>Add Path</button
                >
            </div>

            <!-- Application Search Interval On Startup (minutes) -->
            <div>
                <label
                    class="block font-bold mb-1"
                    for="application_search_interval_on_startup_minute"
                    >Application Search Interval On Startup (minutes)</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    Interval in minutes to search for applications at startup.
                </p>
                <input
                    class="input input-bordered"
                    id="application_search_interval_on_startup_minute"
                    type="number"
                    min="0"
                    bind:value={application_search_interval_on_startup_minute}
                />
            </div>

            <!-- Log Level -->
            <div>
                <label class="block font-bold mb-1" for="log_level"
                    >Log Level</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    Specifies the log output level (error, warn, info, debug).
                </p>
                <select
                    class="input input-bordered"
                    id="log_level"
                    bind:value={log_level}
                >
                    <option value="error">error</option>
                    <option value="warn">warn</option>
                    <option value="info">info</option>
                    <option value="debug">debug</option>
                </select>
            </div>

            <!-- Window Width -->
            <div>
                <label class="block font-bold mb-1" for="width"
                    >Window Width</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    Width of the main application window (in pixels).
                </p>
                <input
                    class="input input-bordered"
                    id="width"
                    type="number"
                    min="0"
                    bind:value={width}
                />
            </div>

            <!-- Auto Startup -->
            <div class="flex items-center">
                <input
                    id="auto_startup"
                    type="checkbox"
                    bind:checked={auto_startup}
                    class="mr-2"
                />
                <label for="auto_startup" class="font-bold">Auto Startup</label>
                <span class="text-xs text-gray-500 ml-2"
                    >Automatically start the application when the system boots.</span
                >
            </div>

            <!-- Shortcut Key -->
            <div>
                <label class="block font-bold mb-1" for="shortcut_key"
                    >Shortcut Key</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    Global shortcut key to toggle the application visibility.
                </p>
                <input
                    class="input input-bordered"
                    id="shortcut_key"
                    type="text"
                    bind:value={shortcut_key}
                />
            </div>

            <!-- Application Name Aliases -->
            <div>
                <label
                    class="block font-bold mb-1"
                    for="application_name_aliases_0"
                    >Application Name Aliases</label
                >
                <p class="text-xs text-gray-500 mb-1">
                    List of application paths and their aliases.
                </p>
                {#each application_name_aliases as alias, i}
                    <div class="flex items-center mb-1">
                        <input
                            class="input input-bordered mr-2"
                            id={`application_name_aliases_path_${i}`}
                            type="text"
                            placeholder="Path"
                            bind:value={application_name_aliases[i].path}
                        />
                        <input
                            class="input input-bordered mr-2"
                            id={`application_name_aliases_alias_${i}`}
                            type="text"
                            placeholder="Alias"
                            bind:value={application_name_aliases[i].alias}
                        />
                        <button
                            type="button"
                            class="btn btn-sm btn-error"
                            on:click={() =>
                                application_name_aliases.splice(i, 1)}
                            >Remove</button
                        >
                    </div>
                {/each}
                <button
                    type="button"
                    class="btn btn-sm btn-primary mt-1"
                    on:click={() =>
                        (application_name_aliases = [
                            ...application_name_aliases,
                            { path: "", alias: "" },
                        ])}>Add Alias</button
                >
            </div>

            <div>
                <button type="submit" class="btn btn-success"
                    >Save Settings</button
                >
            </div>
        </form>
    </div>
</main>

<style lang="postcss">
    @reference "tailwindcss";
    :global(html),
    :global(body) {
        background-color: var(--color-bg);
        overflow: hidden;
        height: 100%;
        width: 100%;
    }

    main.container {
        height: 100vh;
        width: 100vw;
        display: flex;
        flex-direction: column;
        padding: 0;
    }

    /* 疑似ウィンドウフレームの下にスクロール領域 */
    .kasuri-content-scroll {
        flex: 1 1 0%;
        overflow-y: auto;
        padding-top: 2rem;
        padding-bottom: 2rem;
        padding-left: 0;
        padding-right: 0;
        width: 100vw;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .kasuri-content-scroll form {
        max-width: 640px;
        width: 100%;
        margin: 0 auto;
    }
</style>
