import { createApp } from "vue";
import './assets/index.css';
import App from "./App.vue";
import {router} from "@/router.ts";
import { createPinia } from "pinia";
import { useKeyboard, createKeyboardNavigation } from './composables/KeyboardPlugin'
import { setupLogger } from './lib/Logger.ts';
const app = createApp(App)
app.use(router)
const pinia = createPinia();
app.use(pinia);
app.use(createKeyboardNavigation({ debug: false }))
useKeyboard()
app.mount("#app");
setupLogger();