import { createApp } from "vue";
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import App from "./App.vue";
import emitter from './services/emitter'
const app = createApp(App);
app.provide('emitter', emitter);
app.mount("#app");
