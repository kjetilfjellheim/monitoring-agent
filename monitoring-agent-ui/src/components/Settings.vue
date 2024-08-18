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
    addUrl(url) {
      this.urls = this.getUrls();
      this.urls.push(url);
      this.saveUrls(this.urls);
    },
    getUrls() {
      let currentUrls = localStorage.getItem(URL_NAME);
      if (!currentUrls) {
        currentUrls = [];
        this.saveUrls(currentUrls);
      }
      let urls = JSON.parse(currentUrls);
      return urls;
    },
    saveUrls(urls) {
      localStorage.setItem(URL_NAME, JSON.stringify(urls));
    },
    removeUrl(url) {
      let urls = this.getUrls();
      urls = urls.filter(u => u !== url);
      this.saveUrls(urls);
      this.urls = this.getUrls();
    },
    test(url) {
      fetch(url + "/monitors/status")
        .then(response => response.json())
        .then(response => alert("Got success response from " + url))
        .catch(error => alert(error));
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
            <ul class="navbar-nav me-auto mb-2 mb-lg-0">
            </ul>
            <form class="d-flex">
              <div class="input-group mb-3">
                <input type="text" class="form-control" aria-describedby="basic-addon2" v-model="new_url" />
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
            <th scope="col">#</th>
            <th scope="col">URL</th>
            <th scope="col">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="url in urls" id="{{ url }}">
            <th scope="row">1</th>
            <td><input type="text" class="form-control" aria-describedby="basic-addon2" v-bind:value="url" /></td>
            <td><button class="btn btn-danger function" @click="removeUrl(url)">Remove</button><button class="btn btn-info function" @click="test(url)">Test</button></td>
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
</style>
