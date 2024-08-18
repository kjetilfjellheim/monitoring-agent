<script>
const URL_NAME = 'apiUrls';

export default {
  data() {
    return {
      urls: this.getUrls(),
      new_url: null,
      remove_url: null
    };
  },
  methods: {
    // Add a new URL to the list of URLs
    addUrl(url) {
      this.urls = this.getUrls();
      this.urls.push(url);
      this.saveUrls(this.urls);
    },
    // Get the list of URLs from local storage
    getUrls() {
      let currentUrls = localStorage.getItem(URL_NAME);
      if (!currentUrls) {
        currentUrls = [];
        this.saveUrls(currentUrls);
      }
      let urls = JSON.parse(currentUrls);
      return urls;
    },
    // Save the list of URLs to local storage
    saveUrls(urls) {
      localStorage.setItem(URL_NAME, JSON.stringify(urls));
    },
    // Remove a URL from the list of URLs
    removeUrl(url) {
      let urls = this.getUrls();
      urls = urls.filter(u => u !== url);
      this.saveUrls(urls);
      this.urls = this.getUrls();
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
      for (let url of this.urls) {
        this.ping(url).then(success => {
          if (!success) {
            alert('Ping failed for: ' + url);
          }
        });
      }      
    }
  }
}
</script>
<template>
  <div class="container">
    <div class="row">
      <nav class="navbar navbar-expand-lg navbar-dark bg-dark ">
        <div class="container-fluid">
          <div class="collapse navbar-collapse" id="navbarSupportedContent">
            <div class="input-group mb-3">
              <button class="btn btn-info small" @click="checkAll()">Check all</button>
            </div>
            <ul class="navbar-nav me-auto mb-2 mb-lg-0">
            </ul>
            <form class="d-flex">
              <div class="input-group mb-3">
                <input type="text" class="form-control" aria-describedby="basic-addon2" v-model="new_url" size="50" />
                <div class="input-group-append">
                  <button class="btn btn-primary" @click="addUrl(new_url)">Add</button>
                </div>
              </div>
            </form>
          </div>
        </div>
      </nav>
      <table class="table table-striped table-dark">
        <thead class="thead-dark">
          <tr>
            <th scope="col"></th>
            <th scope="col">URL</th>
            <th scope="col">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="url in urls" id="{{ url }}">
            <th scope="row"></th>
            <td class="vertical-align-middle"><label class="form-check-label text-light">{{ url }}</label></td>
            <td><button class="btn btn-danger function" @click="removeUrl(url)">Remove</button><button
                class="btn btn-info function" @click="check(url)">Check</button></td>
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
</style>
