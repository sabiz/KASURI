import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../core/settings";

const INVOKE_SEARCH_APPLICATION = "search_application";
const INVOKE_CHANGED_CONTENT_SIZE = "changed_content_size";
const INVOKE_CLOSE_WINDOW = "close_window";
const INVOKE_LAUNCH_APPLICATION = "launch_application";
const INVOKE_GET_SETTINGS = "get_settings";
const INVOKE_GET_DEFAULT_SETTINGS = "get_default_settings";
const INVOKE_SAVE_SETTINGS = "save_settings";
const INVOKE_RESTART_APP = "restart_app";


/**
 * Represents an application that can be searched and launched.
 */
export interface Application {
    /** The display name of the application */
    name: string;
    /** The unique identifier for the application */
    app_id: string;
    /** The path to the icon file for the application */
    icon_path: string;
}

/**
 * Backend class for interacting with Tauri backend services.
 * Provides methods for searching applications, managing window state,
 * and launching applications.
 */
export class Backend {
    /** Tracks the last sent content size to avoid unnecessary updates */
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
        if (contentSize < 0) {
            throw new Error("Content size cannot be negative");
        }
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
    public async close(): Promise<void> {
        await invoke(INVOKE_CLOSE_WINDOW);
    }

    /**
     * Launches an application based on the provided application object.
     * @param application The application object to be launched.
     * @returns A promise that resolves when the application is launched.
     */
    public async launch(application: Application): Promise<void> {
        if (!application || !application.app_id) {
            throw new Error("Invalid application object");
        }
        await invoke(INVOKE_LAUNCH_APPLICATION, {
            appId: application.app_id,
        });
    }

    /**
     * Retrieves the settings from the backend.
     * @returns A promise that resolves to the settings object.
     */
    public async getSettings(): Promise<Settings> {
        const result = await invoke(INVOKE_GET_SETTINGS);
        if (typeof result !== "object" || result === null) {
            throw new Error("Invalid settings format received from backend");
        }
        return this.transformForFrontend<Settings>(result);
    }

    /**
     * Retrieves the default settings from the backend.
     * @returns A promise that resolves to the default settings object.
     */
    public async getDefaultSettings(): Promise<Settings> {
        const result = await invoke(INVOKE_GET_DEFAULT_SETTINGS);
        if (typeof result !== "object" || result === null) {
            throw new Error("Invalid default settings format received from backend");
        }
        return this.transformForFrontend<Settings>(result);
    }

    /**
     * Saves the settings to the backend.
     * @param settings The settings object to be saved.
     * @returns A promise that resolves when the settings are saved.
     */
    public async saveSettings(settings: Settings): Promise<boolean> {
        return await invoke(INVOKE_SAVE_SETTINGS, {
            settings: this.transformForBackend(settings)
        });
    }

    /**
     * Restarts the application.
     * @returns A promise that resolves when the application is restarted.
     */
    public async restartApp(): Promise<void> {
        await invoke(INVOKE_RESTART_APP);
    }

    /**
     * Transforms an object by converting its keys from snake_case to camelCase.
     * @param obj The object to be transformed.
     * @returns The transformed object with camelCase keys.
     */
    private transformForFrontend<T>(obj: any): T {
        const result: any = {};
        for (const key in obj) {
            if (!obj.hasOwnProperty(key)) {
                continue;
            }
            const value = obj[key];
            const camelCaseKey = key.replace(/_([a-z])/g, (match, letter) => letter.toUpperCase());
            if (Array.isArray(value)) {
                result[camelCaseKey] = value.map((item: any) => {
                    if (typeof item === "object" && item !== null) {
                        return this.transformForFrontend(item);
                    }
                    return item;
                });
            } else if (typeof value === "object" && value !== null) {
                result[camelCaseKey] = this.transformForFrontend(value);
            } else {
                result[camelCaseKey] = value;
            }
        }
        return result as T;
    }

    private transformForBackend<T>(obj: any): T {
        const result: any = {};
        for (const key in obj) {
            if (!obj.hasOwnProperty(key)) {
                continue;
            }
            const value = obj[key];
            const snakeCaseKey = key.replace(/([A-Z])/g, (match) => `_${match.toLowerCase()}`);
            if (Array.isArray(value)) {
                result[snakeCaseKey] = value.map((item: any) => {
                    if (typeof item === "object" && item !== null) {
                        return this.transformForBackend(item);
                    }
                    return item;
                });
            } else if (typeof value === "object" && value !== null) {
                result[snakeCaseKey] = this.transformForBackend(value);
            } else {
                result[snakeCaseKey] = value;
            }
        }
        return result as T;
    }
}