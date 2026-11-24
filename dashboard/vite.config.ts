import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// `npm run dev` stuurt /ws en /api door naar een server op de eigen machine.
const server = process.env.UITSLAGAVOND_SERVER ?? "127.0.0.1:7400";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": `http://${server}`,
      "/ws": { target: `ws://${server}`, ws: true },
    },
  },
});
