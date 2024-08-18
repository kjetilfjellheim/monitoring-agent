<script setup>
import { ref } from 'vue';
const processes = ref(null);
const URL_NAME = 'apiUrls';
processes.value = [];
let currentUrls = localStorage.getItem(URL_NAME);
let urls = JSON.parse(currentUrls);
for (let url of urls) {
    fetch(url + "/processes")
        .then(response => response.json())
        .then(json => {
            json.forEach(element => {
                element.url = url;
            });
            processes.value.push(...json)
        })
        .catch(error => alert(error));
}                
</script>
<script>
export default {
    data() {
        return {
            search: ""
        }
    },
    methods: {
        filter(process) {
            return this.check_includes(process.url, this.search) || 
                this.check_includes(process.pid.toString(), this.search) || 
                this.check_includes(process.parentPid.toString(), this.search) || 
                this.check_includes(process.name, this.search) ||
                process.umask.includes(this.search) || 
                process.processState.includes(this.search) || 
                process.numThreads.toString().includes(this.search) || 
                process.groups.join(", ").includes(this.search);            
        },
        check_includes(value, search) {
            if (value === undefined) {
                return false;
            }
            return value.includes(search);
        }
    }
};
</script>
<template>
    <div class="container" v-if="processes">
      <div class="row">
        <nav class="navbar navbar-expand-lg navbar-dark bg-dark ">
          <div class="container-fluid">
            <div class="collapse navbar-collapse" id="navbarSupportedContent">      
                <ul class="navbar-nav me-auto mb-2 mb-lg-0"></ul>
                <form class="d-flex">
                    <div class="form-floating">
                        <input id="searchInput" type="text" class="form-control" aria-describedby="basic-addon2" v-model="search" size="50" />
                        <label for="searchInput">Search</label>
                    </div>
                </form>
            </div>
          </div>
        </nav>
        <table class="table table-striped table-dark">
          <thead class="thead-dark">
            <tr>
              <th scope="col">Server</th>
              <th scope="col">Pid</th>
              <th scope="col">Parent</th>
              <th scope="col">Name</th>
              <th scope="col">Umask</th>
              <th scope="col">State</th>
              <th scope="col">Threads</th>
              <th scope="col">Groups</th>
              <th scope="col">Functions</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="process in processes">
                <tr v-if="filter(process)">
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.url }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.pid }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.parentPid }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.name }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.umask }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.processState }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.numThreads }}</label></td>
                    <td class="vertical-align-middle"><label class="form-check-label text-light">{{ process.groups.join(", ") }}</label></td>
                    <td class="vertical-align-middle">
                        <button class="btn btn-info btn-sm tools">Details</button>
                        <button class="btn btn-info btn-sm tools" disabled>Threads</button>
                    </td>
                </tr>
            </template>
          </tbody>
        </table>
      </div>
    </div>
    <div v-if="!processes" class="text-center">
        <div class="spinner-border spinner-border-sm"></div>
    </div>    
  </template>

<style scoped>
.col {
    padding: 5px;
    margin: 5px;
}

.card {
    height: 100%;
    margin: 2px;
}

.no-margin {
    margin: 0px;
}

.toolbar-item {
    margin-left: 10px;
    margin-right: 5px;
}

.tools {
    margin-right: 4px;
}
</style>
