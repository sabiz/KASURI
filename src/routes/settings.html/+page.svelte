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
    let temporarySettings: Settings = $state({
        applicationSearchPathList: [],
        applicationSearchIntervalOnStartupMinute: 0,
        logLevel: LogLevel.Info,
        width: 0,
        autoStartup: false,
        shortcutKey: "",
        applicationNameAliases: [],
    });
    let isRecordingShortcut = $state(false);
    let beforeRecordingShortcut: string = "";

    let applicationSearchIntervalOnStartupMinute = $derived(
        temporarySettings.applicationSearchIntervalOnStartupMinute / 60,
    );
    let auto_startup: string = $derived(
        temporarySettings.autoStartup ? "on" : "off",
    );
    let elementShortcut: HTMLInputElement | null = null;

    onMount(async () => {
        const settings = await backend.getSettings();
        temporarySettings = { ...settings };
        console.log("Settings loaded:", $state.snapshot(temporarySettings));
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
    async function openSearchPathFolderSelector(
        path: string = "",
        index: number | null = null,
    ) {
        const folder = await open({
            directory: true,
            multiple: false,
            defaultPath: path,
            title: "Select Application Search Path",
        });
        if (!folder) {
            return;
        }
        if (!index) {
            temporarySettings.applicationSearchPathList = [
                ...temporarySettings.applicationSearchPathList,
                folder,
            ];
        } else {
            temporarySettings.applicationSearchPathList[index] = folder;
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

    /**
     * Toggles the state of shortcut key recording.
     * If currently recording, it stops and removes the event listener.
     * If not recording, it adds the event listener and focuses the input.
     */
    function toggleShortcutRecordingState() {
        if (isRecordingShortcut) {
            elementShortcut?.removeEventListener(
                "keydown",
                handleShortcutKeydown,
                true,
            );
            isRecordingShortcut = false;
            elementShortcut?.blur();
            return;
        } else {
            beforeRecordingShortcut = temporarySettings.shortcutKey;
            elementShortcut?.focus();
            elementShortcut?.addEventListener(
                "keydown",
                handleShortcutKeydown,
                true,
            );
            isRecordingShortcut = true;
        }
    }

    /**
     * Handles the keydown event for recording a shortcut key.
     * It prevents the default action, constructs the shortcut key string,
     * and updates the temporary settings with the new shortcut key.
     * If Esc is pressed, it cancels the recording and restores the previous shortcut.
     * @param event The keyboard event.
     */
    function handleShortcutKeydown(event: KeyboardEvent) {
        event.preventDefault();
        if (event.key === "Escape") {
            // If Esc is pressed, cancel the recording and restore the previous shortcut
            temporarySettings.shortcutKey = beforeRecordingShortcut;
            if (isRecordingShortcut) {
                toggleShortcutRecordingState();
            }
            return;
        }
        const keys = [];
        if (event.ctrlKey) {
            keys.push("Ctrl");
        }
        if (event.altKey) {
            keys.push("Alt");
        }
        if (event.shiftKey) {
            keys.push("Shift");
        }
        if (event.metaKey) {
            keys.push("Meta");
        }
        if (event.key === " ") {
            keys.push("Space");
        }
        const key = event.key;
        if (!["Control", "Shift", "Alt", "Meta", " "].includes(key)) {
            keys.push(key.length === 1 ? key.toUpperCase() : key);
        }
        temporarySettings.shortcutKey = keys.join("+");
    }

    /**
     * Opens a file selector dialog to select an alias target application path.
     * Updates the settings with the selected path for the specified alias index.
     * If index is null, it adds a new alias to the settings.
     * @param path
     * @param index
     */
    async function openAliasTargetSelector(
        path: string = "",
        index: number | null = null,
    ) {
        const targetPath = await open({
            directory: false,
            multiple: false,
            defaultPath: path,
            title: "Select Alias Target Application Path",
        });
        if (!targetPath) {
            return;
        }
        if (index === null) {
            temporarySettings.applicationNameAliases = [
                ...temporarySettings.applicationNameAliases,
                { path: targetPath, alias: "" },
            ];
        } else {
            temporarySettings.applicationNameAliases[index].path = targetPath;
        }
    }

    /**
     * Removes an alias from the settings.
     * It filters out the alias at the specified index.
     * @param index
     */
    function removeAlias(index: number) {
        temporarySettings.applicationNameAliases =
            temporarySettings.applicationNameAliases.filter(
                (_, i) => i !== index,
            );
    }

    async function saveSettings() {
        throw new Error("Not implemented yet");
    }

    /**
     * Loads the default settings from the backend.
     * It fetches the default settings and updates the temporary settings.
     * @returns A promise that resolves when the default settings are loaded.
     */
    async function loadDefaultSettings() {
        temporarySettings = await backend.getDefaultSettings();
        console.log(
            "Default settings loaded:",
            $state.snapshot(temporarySettings),
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
                onclick={minimizeWindow}
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
                onclick={maximizeWindow}
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
                onclick={closeWindow}
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
                        disabled={path === WINDOWS_STORE_APP_ALIAS}
                        onclick={async () =>
                            await openSearchPathFolderSelector(path, i)}
                    >
                        <Icon
                            icon="uiw:folder-open"
                            width={24}
                            height={24}
                            class={path === WINDOWS_STORE_APP_ALIAS
                                ? "text-(--color-bg-lightx2)"
                                : ""}
                        />
                    </button>
                    <input class="flex-1" type="text" readonly value={path} />
                    <button
                        class="btn-ctl mx-2"
                        onclick={() => removeSearchPath(i)}
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
                    onclick={() => openSearchPathFolderSelector()}
                >
                    <Icon icon="uiw:folder-add" width={24} height={24} />
                </button>
                {#if !temporarySettings.applicationSearchPathList.includes(WINDOWS_STORE_APP_ALIAS)}
                    <button
                        class="btn-ctl ml-3"
                        aria-label="Add WindowsStoreApp"
                        title="Add WindowsStoreApp"
                        onclick={() =>
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

        <div>
            <span class="setting-title"
                >Application Search Interval On Startup (hour)</span
            >
            <p class="setting-explanation">
                If the elapsed time since the last application startup is less
                than the specified hour, automatic application search will be
                skipped. Set to 0 to always search at startup.<br />
            </p>
            <input
                class="mt-1 mr-2"
                type="number"
                min="0"
                max="8760"
                bind:value={applicationSearchIntervalOnStartupMinute}
                onchange={() =>
                    (temporarySettings.applicationSearchIntervalOnStartupMinute =
                        applicationSearchIntervalOnStartupMinute * 60)}
            />
            <span class="text">hours</span>
        </div>

        <div>
            <span class="setting-title">Log Level</span>
            <p class="setting-explanation">
                Specifies the log output level (error, warn, info, debug).
            </p>
            <select class="mt-1" bind:value={temporarySettings.logLevel}>
                <option value="error">error</option>
                <option value="warn">warn</option>
                <option value="info">info</option>
                <option value="debug">debug</option>
            </select>
        </div>

        <div>
            <span class="setting-title">Window Width</span>
            <p class="setting-explanation">
                Width of the main application window.
            </p>
            <input
                class="mt-1 mr-2"
                type="number"
                min="100"
                bind:value={temporarySettings.width}
            />
            <span class="text">pixels</span>
        </div>
        <div>
            <span class="setting-title">Auto Startup</span>
            <p class="setting-explanation">
                Automatically start the application when the system boots.
            </p>
            <select class="mt-1" bind:value={auto_startup}>
                <option value="off">off</option>
                <option value="on">on</option>
            </select>
        </div>

        <div>
            <span class="setting-title">Shortcut Key</span>
            <p class="setting-explanation">
                Global shortcut key to toggle the application visibility.<br />
            </p>
            <input
                class="w-40 text-center mr-2 {isRecordingShortcut
                    ? 'outline-(--color-accent-blue) outline-solid outline-2'
                    : ''}"
                type="text"
                value={temporarySettings.shortcutKey}
                readonly
                placeholder="Not set"
                bind:this={elementShortcut}
            />
            <button
                class="btn-ctl !px-2"
                onclick={toggleShortcutRecordingState}
            >
                {isRecordingShortcut ? "Stop Recording" : "Start Recording"}
            </button>
            {#if isRecordingShortcut}
                <div class="text-xs mt-1 text-(--color-accent-blue)">
                    Press the desired key combination to set the shortcut. Press
                    <strong>Esc</strong> to cancel.
                </div>
            {/if}
        </div>

        <div>
            <span class="setting-title">Application Name Aliases</span>
            <p class="setting-explanation">
                List of application paths and their aliases.<br />
            </p>
            <p class="text-xs border-(--color-text) border-1 p-2 rounded mb-3">
                <strong>Note:</strong><br /> The "path" must exactly match the path
                of an application discovered via "Application Search Path" (such
                as .lnk or .exe files). If the path does not match,the alias will
                not be applied.
            </p>
            {#each temporarySettings.applicationNameAliases as alias, i}
                <div
                    class="mb-2 p-2 rounded bg-(--color-bg-lightx2) flex items-center"
                >
                    <div class="flex flex-1">
                        <div class="mb-1 flex-3">
                            <span class="block text-xs font-semibold mb-1"
                                >Path</span
                            >
                            <div class="flex items-start mb-1">
                                <button
                                    class="btn-ctl mr-2"
                                    aria-label="Select Alias Target Application Path"
                                    title="Select Alias Target Application Path"
                                    onclick={async () =>
                                        await openAliasTargetSelector(
                                            alias.path,
                                            i,
                                        )}
                                >
                                    <Icon
                                        icon="uiw:folder-open"
                                        width={24}
                                        height={24}
                                    />
                                </button>
                                <textarea
                                    class="resize-y text-sm"
                                    readonly
                                    rows="2"
                                    bind:value={alias.path}
                                ></textarea>
                            </div>
                        </div>
                        <div class="ml-2 flex-1">
                            <span class="block text-xs font-semibold mb-1"
                                >Alias</span
                            >
                            <input
                                class="w-full {alias.alias
                                    ? ''
                                    : 'border-(--color-accent-red) border-2'}"
                                type="text"
                                placeholder="Alias"
                                bind:value={alias.alias}
                            />
                        </div>
                    </div>
                    <button
                        class="btn-ctl basis-auto ml-2"
                        onclick={() => removeAlias(i)}
                        aria-label="Remove Alias"
                        title="Remove Alias"
                    >
                        <Icon icon="uiw:delete" width={24} height={24} />
                    </button>
                </div>
            {/each}
            <button
                class="btn-ctl mt-1"
                aria-label="Add Alias"
                title="Add Alias"
                onclick={async () => await openAliasTargetSelector()}
                ><Icon
                    icon="basil:add-outline"
                    width={24}
                    height={24}
                /></button
            >
        </div>
        <hr />
        <div>
            <button
                class="btn-ctl text-(--color-accent-blue)"
                onclick={saveSettings}>Save</button
            >
            <button class="btn-ctl ml-2" onclick={loadDefaultSettings}
                >Load Defaults</button
            >
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

    input[type="text"],
    input[type="number"],
    select {
        @apply px-2 rounded bg-(--color-bg-light) h-8;
    }
    textarea {
        @apply px-2 py-1 rounded bg-(--color-bg-light) w-full resize-y;
        line-height: 1.4;
    }

    .setting-title {
        @apply block font-bold mb-1 text-lg border-b-1;
    }

    .setting-explanation {
        @apply text-xs mb-1;
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
