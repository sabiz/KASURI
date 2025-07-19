<script lang="ts">
    import { onMount } from "svelte";
    import Icon from "@iconify/svelte";
    import { Window } from "@tauri-apps/api/window";
    import { open } from "@tauri-apps/plugin-dialog";
    import type { Settings } from "../../core/settings";
    import { LogLevel } from "../../core/settings";
    import { Backend } from "../../core/backend";

    const THIS_WINDOW_LABEL = "settings";
    const WINDOWS_STORE_APP_ALIAS = "WindowsStoreApp";

    const backend = new Backend();
    let temporarySettings: Settings = {
        applicationSearchPathList: ["C:\\", WINDOWS_STORE_APP_ALIAS],
        applicationSearchIntervalOnStartup: 0,
        logLevel: LogLevel.Info,
        width: 0,
        autoStartup: false,
        shortcutKey: "",
        applicationNameAliases: [],
    };

    let application_search_interval_on_startup_minute: number = 0;
    let log_level: string = "";
    let width: number = 0;
    let auto_startup: boolean = false;
    let shortcut_key: string = "";
    let application_name_aliases: { path: string; alias: string }[] = [];

    onMount(async () => {
        const settings = await backend.getSettings();
        temporarySettings = settings;
    });

    /**
     * Minimizes the current window.
     */
    async function minimizeWindow() {
        (await Window.getByLabel(THIS_WINDOW_LABEL))?.minimize();
    }
    /**
     * Maximizes the current window.
     */
    async function maximizeWindow() {
        (await Window.getByLabel(THIS_WINDOW_LABEL))?.toggleMaximize();
    }
    /**
     * Closes the current window.
     */
    async function closeWindow() {
        (await Window.getByLabel(THIS_WINDOW_LABEL))?.close();
    }

    /**
     * Opens a folder selector dialog to select a directory.
     * Updates the settings with the selected path.
     * @param path The default path to show in the dialog.
     * @param index The index of the search path to update in the settings.
     */
    async function openFolderSelector(path: string, index: number) {
        const folder = await open({
            directory: true,
            multiple: false,
            defaultPath: path,
            title: "Select Application Search Path",
        });
        if (!folder) {
            return;
        }
        if (typeof folder === "string") {
            temporarySettings.applicationSearchPathList[index] = folder;
        }
    }

    async function addFolder() {
        const folder = await open({
            directory: true,
            multiple: false,
            title: "Select Application Search Path",
        });
        if (!folder) {
            return;
        }
        console.log("Selected folder:", folder);
        if (typeof folder === "string") {
            temporarySettings.applicationSearchPathList = [
                ...temporarySettings.applicationSearchPathList,
                folder,
            ];
        }
    }

    /**
     * Removes a search path from the settings.
     * @param i The index of the search path to remove.
     */
    function removeSearchPath(i: number) {
        temporarySettings.applicationSearchPathList =
            temporarySettings.applicationSearchPathList.filter(
                (_, index) => index !== i,
            );
    }
</script>

<main class="container w-screen h-screen p-0 flex flex-col">
    <div
        class="w-screen flex items-center justify-between px-2 py-1 select-none bg-(--color-bg-lightx2)"
        data-tauri-drag-region
    >
        <div
            class="font-bold text-lg pl-1 tracking-[0.12em]"
            data-tauri-drag-region
        >
            KASURI | Settings
        </div>
        <div class="flex items-center gap-1">
            <button
                type="button"
                title="Minimize"
                aria-label="Minimize"
                class="btn-window hover:bg-(--color-bg-lightx3)"
                on:click={minimizeWindow}
            >
                <svg width="16" height="16" fill="currentColor"
                    ><rect y="12" width="16" height="2" rx="1" /></svg
                >
            </button>
            <button
                type="button"
                title="Maximize"
                aria-label="Maximize"
                class="btn-window hover:bg-(--color-bg-lightx3)"
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
                title="Close"
                aria-label="Close"
                class="btn-window hover:bg-(--color-accent-red)"
                on:click={closeWindow}
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
    <div
        class="kasuri-content-scroll grow shrink basis-0 overflow-y-auto pt-[2rem] pb-[2rem] w-screen space-y-6 px-[3rem]"
    >
        <div>
            <span class="block font-bold mb-1 text-lg border-b-1"
                >Application Search Path</span
            >
            <p class="text-xs mb-1">
                List of directories to search for applications. Enter
                "WindowsStoreApp" to include Windows Store apps.
            </p>
            {#each temporarySettings.applicationSearchPathList as path, i}
                <div class="flex items-center mb-1 pt-1 pb-1">
                    <button
                        class="btn-ctl mr-2"
                        aria-label="Select Folder"
                        title="Select Folder"
                        disabled={temporarySettings.applicationSearchPathList[
                            i
                        ] === WINDOWS_STORE_APP_ALIAS}
                        on:click={() => openFolderSelector(path, i)}
                    >
                        <Icon
                            icon="uiw:folder-open"
                            width={24}
                            height={24}
                            class={temporarySettings.applicationSearchPathList[
                                i
                            ] === WINDOWS_STORE_APP_ALIAS
                                ? "text-(--color-bg-lightx2)"
                                : ""}
                        />
                    </button>
                    <input class="flex-1" type="text" readonly value={path} />
                    <button
                        class="btn-ctl mx-2"
                        on:click={() => removeSearchPath(i)}
                    >
                        <Icon icon="uiw:delete" width={24} height={24} />
                    </button>
                </div>
            {/each}
            <div class="mt-3">
                <button
                    class="btn-ctl"
                    aria-label="Add Folder"
                    title="Add Folder"
                    on:click={addFolder}
                >
                    <Icon icon="uiw:folder-add" width={24} height={24} />
                </button>
                {#if !temporarySettings.applicationSearchPathList.includes(WINDOWS_STORE_APP_ALIAS)}
                    <button
                        class="btn-ctl"
                        aria-label="Add WindowsStoreApp"
                        title="Add WindowsStoreApp"
                        on:click={() =>
                            (temporarySettings.applicationSearchPathList = [
                                ...temporarySettings.applicationSearchPathList,
                                WINDOWS_STORE_APP_ALIAS,
                            ])}
                    >
                        <Icon icon="uiw:appstore" width={24} height={24} />
                    </button>
                {/if}
            </div>
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
            <label class="block font-bold mb-1" for="log_level">Log Level</label
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
            <label class="block font-bold mb-1" for="width">Window Width</label>
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
            <label class="block font-bold mb-1" for="application_name_aliases_0"
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
                        on:click={() => application_name_aliases.splice(i, 1)}
                        >🗑️</button
                    >
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
            <button type="submit" class="btn btn-success">Save Settings</button>
        </div>
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
        color: var(--color-text);
    }

    button {
        transition: background-color 0.2s ease-in-out;
    }

    input[type="text"] {
        @apply px-2 rounded bg-(--color-bg-light) h-8;
    }

    .btn-window {
        @apply w-8 h-8 flex items-center justify-center rounded;
    }

    .btn-ctl {
        @apply bg-(--color-bg-light) p-[0.125rem] rounded cursor-pointer text-lg;
    }
    .btn-ctl:hover:not(:disabled) {
        @apply bg-(--color-bg-lightx3);
    }
    .btn-ctl:disabled {
        @apply cursor-not-allowed;
    }
</style>
