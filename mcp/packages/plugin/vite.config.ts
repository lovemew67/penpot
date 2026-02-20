import { defineConfig } from "vite";
import livePreview from "vite-live-preview";

export default defineConfig({
    plugins: [
        livePreview({
            reload: true,
            config: {
                build: {
                    sourcemap: true,
                },
            },
        }),
    ],
    build: {
        rollupOptions: {
            input: {
                plugin: "src/plugin.ts",
                index: "./index.html",
            },
            output: {
                entryFileNames: "[name].js",
            },
        },
    },
    preview: {
        host: "0.0.0.0",
        port: 4400,
        cors: true,
        allowedHosts: [],
    },
    define: {
        IS_MULTI_USER_MODE: JSON.stringify(process.env.MULTI_USER_MODE === "true"),
    },
});
