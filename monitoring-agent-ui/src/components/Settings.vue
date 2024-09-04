<script setup>
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faListCheck, faCircleMinus } from '@fortawesome/free-solid-svg-icons';
</script>
<script>
const URL_NAME = 'apiUrls';

export default {
  data() {
    return {
      servers: this.getServers(),
      new_url: null,
      remove_url: null
    };
  },
  methods: {
    // Add a new URL to the list of URLs
    async addUrl(url) {
      this.servers = this.getServers();
      try {
        let fetch_result = await fetch(url);
        let response = await fetch_result.json();
        if (this.check_ping(response)) {
          let server = { url: url, name: response.name };
          this.servers.push(server);
          this.saveServers(this.servers);
          this.servers = this.getServers();
          return true;
        } else { return false; }
      } catch (error) {
        return false;
      }
    },
    // Get the list of URLs from local storage
    getServers() {
      let currentUrls = localStorage.getItem(URL_NAME);
      if (!currentUrls) {
        this.saveServers([]);
      }
      let servers = JSON.parse(currentUrls);
      return servers;
    },
    // Save the list of URLs to local storage
    saveServers(servers) {
      localStorage.setItem(URL_NAME, JSON.stringify(servers));
    },
    // Remove a URL from the list of URLs
    removeServer(url) {
      let servers = this.getServers();
      servers = servers.filter(u => u.url !== url);
      this.saveServers(servers);
      this.servers = this.getServers();
    },
    // Ping a URL to check if it is reachable
    async ping(url) {
      try {
        let fetch_result = await fetch(url);
        let response = await fetch_result.json();
        if (this.check_ping(response)) { return true } else { return false; }
      } catch (error) {
        return false;
      }
    },
    // Check if the ping response is valid
    check_ping(response) {
      if (response.status === 'Ok' && response.system === 'monitoring-agent-daemon') {
        return true;
      } else {
        return false;
      }
    },
    // Check a single URL
    async check(url) {
      this.ping(url).then(success => {
        if (success) {
          alert('Ping successful');
        } else {
          alert('Ping failed for: ' + url);
        }
      });
    },
    // Check all URLs
    async checkAll() {
      let failed = [];
      for (let url of this.urls) {
        this.ping(url).then(success => {
          if (!success) {
            failed.push(url);
          }
        });
      }
      if (failed.length > 0) {
        alert('Ping failed for: ' + failed.join(', '));
      } else {
        alert('Ping successful for all URLs');
      }
    }
  }
}
</script>
<template>
  <div class="container-fluid">
    <div class="row">
      <nav class="navbar navbar-dark bg-dark">
        <div class="container-fluid">
          <button class="btn btn-info small" @click="this.checkAll()">
            <FontAwesomeIcon :icon="faListCheck" />&nbsp;Check all
          </button>
          <form class="d-flex">
            <div class="input-group mb-3">
              <input type="text" class="form-control" aria-describedby="basic-addon2" v-model="new_url" size="50" />
              <div class="input-group-append">
                <button class="btn btn-primary" @click="addUrl(new_url)">Add</button>
              </div>
            </div>
          </form>
        </div>
      </nav>
      <table class="table table-striped-self table-dark">
        <thead class="thead-dark">
          <tr>
            <th scope="col"></th>
            <th scope="col">URL</th>
            <th scope="col">Name</th>
            <th scope="col">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="server in servers" id="{{ servers }}">
            <th scope="row"></th>
            <td class="vertical-align-middle"><label class="form-check-label text-light">{{ server.url }}</label></td>
            <td class="vertical-align-middle"><label class="form-check-label text-light">{{ server.name }}</label></td>
            <td>
              <button class="btn btn-danger btn-sm function" @click="removeServer(server.url)">
                <FontAwesomeIcon :icon="faCircleMinus" />&nbsp;Remove
              </button>
              <button class="btn btn-info btn-sm function" @click="this.check(server.url)">
                <FontAwesomeIcon :icon="faListCheck" />&nbsp;Check
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.function {
  margin-right: 5px;
}

.vertical-align-middle {
  padding: 0px;
  margin: auto;
  vertical-align: middle;
}

.table-striped-self>tbody>tr:nth-child(odd)>td,
.table-striped-self>tbody>tr:nth-child(odd)>th {
  background-color: rgb(90, 90, 90);
}

.table-striped-self>tbody>tr:nth-child(even)>td,
.table-striped-self>tbody>tr:nth-child(even)>th {
  background-color: rgb(65, 65, 65);
}
</style>
