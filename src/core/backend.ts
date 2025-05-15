import { invoke } from "@tauri-apps/api/core";

const INVOKE_SEARCH_APPLICATION = "search_application";
const INVOKE_CHANGED_CONTENT_SIZE = "changed_content_size";
const INVOKE_CLOSE_WINDOW = "close_window";
const INVOKE_LAUNCH_APPLICATION = "launch_application";

export interface Application {
    name: string;
    app_id: string;
    icon_path: string;
}

export class Backend {

    lastContentSize: number = 0;

    /**
     * The constructor for the Backend class.
     * It initializes the backend instance.
     */
    public constructor() {

    }

    /**
     * Searches for applications based on the provided query.
     * @param query The search query string.
     * @returns A promise that resolves to an array of Application objects.
     */
    public async searchApplication(
        query: string,
    ): Promise<Application[]> {

        return invoke(INVOKE_SEARCH_APPLICATION, {
            query,
        });
    }

    /**
     * Sends the content size to the backend.
     * @param contentSize The size of the content to be sent.
     * @returns A promise that resolves when the content size is sent.
     */
    public async sendContentSize(
        contentSize: number,
    ): Promise<void> {
        if (this.lastContentSize === contentSize) {
            return;
        }
        this.lastContentSize = contentSize;
        await invoke(INVOKE_CHANGED_CONTENT_SIZE, {
            contentHeight: contentSize
        });
    }

    /**
     * Closes the window.
     * @returns A promise that resolves when the window is closed.
     */
    public async close() {
        await invoke(INVOKE_CLOSE_WINDOW);
    }

    /**
     * Launches an application based on the provided app ID.
     * @param appId The ID of the application to be launched.
     * @returns A promise that resolves when the application is launched.
     */
    public async launch(application: Application) {
        await invoke(INVOKE_LAUNCH_APPLICATION, {
            appId: application.app_id,
        });
    }
}