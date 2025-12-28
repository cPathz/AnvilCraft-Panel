/**
 * AnvilCraft Feature Flags
 * Use this file to toggle features across the application.
 * Ideally, import this object and use it in {#if} blocks or logic checks.
 */
export const FEATURES = {
    CONSOLE: {
        /**
         * Experimental Autocomplete System (Tree-based + Arguments)
         * Status: Beta
         * Default: false (for stable release)
         */
        AUTOCOMPLETE: true,

        /**
         * Command History Navigation (Up/Down arrows)
         * Status: Stable
         * Default: true
         */
        HISTORY: true,
    },
    SETTINGS: {
        /**
         * Advanced RAM Linking Logic (Min/Max constraints)
         * Status: Stable
         * Default: true
         */
        MEMORY_LINK: true,
    },
    UI: {
        /**
         * Show Beta Watermark in bottom right
         * Status: Formatting
         * Default: true
         */
        SHOW_WATERMARK: true,
    }
};
