import { createApp } from 'vue';
import { createPinia } from 'pinia';
import naive from 'naive-ui';
import App from './App.vue';
import router from './router';
import { useSettingsStore } from './stores/settings';
import { useDownloadStore } from './stores/download';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(naive);

// Load settings and initialize download store before mounting
const settingsStore = useSettingsStore();
const downloadStore = useDownloadStore();

Promise.all([
  settingsStore.loadSettings(),
  downloadStore.init(),
]).then(() => {
  app.mount('#app');
});
