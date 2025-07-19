/**
 * Settings interface.
 */
export type Settings = {
    applicationSearchPathList: string[];
    applicationSearchIntervalOnStartup: number;
    logLevel: LogLevel;
    width: number;
    autoStartup: boolean;
    shortcutKey: string;
    applicationNameAliases: ApplicationNameAlias[];
}

/**
 * Log levels
 */
export enum LogLevel {
    Debug = 'debug',
    Info = 'info',
    Warn = 'warn',
    Error = 'error'
}

/**
 * Application name alias.
 */
type ApplicationNameAlias = {
    path: string;
    alias: string;
};