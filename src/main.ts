import { createApp } from "vue";
import './assets/index.css';
import App from "./App.vue";
import {router} from "@/router.ts";
import { createPinia } from "pinia";
import { useKeyboard, createKeyboardNavigation } from './composables/KeyboardPlugin'
const app = createApp(App)
app.use(router)
const pinia = createPinia();
app.use(pinia);
app.use(createKeyboardNavigation({ debug: true }))
useKeyboard()
app.mount("#app");
