import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { Tooltip } from 'bootstrap'

import "bootstrap/dist/css/bootstrap.min.css"
import "bootstrap"
import 'bootstrap-icons/font/bootstrap-icons';

import './assets/base.css';

export default {
  mounted() {
    new Tooltip(document.body, {
      selector: "[data-bs-toggle='tooltip']",
    })
  }
}

const app = createApp(App)

app.use(router)

app.mount('#app')
