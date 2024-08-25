<script setup>
    import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
    import { faRefresh, faStar, faChartSimple } from '@fortawesome/free-solid-svg-icons';
</script>
<script>
import { ref } from 'vue';

export default {
    data() {
        return {
            searchServer: "",
            searchPid: "",
            searchParent: "",
            searchName: "",
            searchUmask: "",
            searchState: "",
            searchThreads: "",
            processes: null
        }
    },
    mounted() {
        this.refreshProcesses();        
    },
    methods: {        
        refreshProcesses() {
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
                            if (element.name === undefined) {
                                element.name = "";
                            }
                        });
                        processes.value.push(...json)
                    })
                    .catch(error => console.error('Error:', error));
            }
            this.processes = processes.value;
        },
        filter(process) {
            return this.checkAllNull() ||
                this.check_includes(process.url, this.searchServer) &&
                this.check_includes(process.pid.toString(), this.searchPid) &&
                this.check_includes(process.parentPid.toString(), this.searchParent) &&
                this.check_includes(process.name, this.searchName) &&
                this.check_includes(process.umask, this.searchUmask) &&
                this.check_includes(process.processState, this.searchState) &&
                this.check_includes(process.numThreads.toString(), this.searchThreads);
        },
        check_includes(value, search) {
            if (value === undefined) {
                return true;
            }
            if (search === "") {
                return true;
            }
            return value.includes(search);
        },
        checkAllNull() {
            return this.searchServer === "" &&
                this.searchPid === "" &&
                this.searchParent === "" &&
                this.searchName === "" &&
                this.searchUmask === "" &&
                this.searchState === "" &&
                this.searchThreads === "";
        },
        isMonitored(process) {
            if (process.monitored) {
                return "monitored";
            } else {
                return "";
            }
        }
    }
};
</script>
<template>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="collapse navbar-collapse">
            <ul class="navbar-nav mr-auto mt-2 mt-lg-0">
                <li class="nav-item">
                    <button class="btn btn-info small toolbar-item" @click="this.refreshProcesses()"><FontAwesomeIcon :icon="faRefresh" />&nbsp;Refresh</button>
                </li>                
            </ul>
        </div>
    </nav>    
    <div class="container-fluid" v-if="processes">
        <div class="row">
            <table class="table table-striped table-dark table-responsive">
                <thead class="thead-dark">
                    <tr>
                        <th scope="col">Server</th>
                        <th scope="col">Pid</th>
                        <th scope="col">Parent</th>
                        <th scope="col">Name</th>
                        <th scope="col">Umask</th>
                        <th scope="col">State</th>
                        <th scope="col">Threads</th>
                        <th scope="col">Details</th>
                    </tr>
                    <tr>
                        <th scope="col"><input id="idSearchServer" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchServer" minlength="10" maxLength="10"
                                size="10" /></th>
                        <th scope="col"><input id="idSearchPid" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchPid" minlength="5" maxLength="5"
                                size="5" /></th>
                        <th scope="col"><input id="idSearchParent" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchParent" minlength="5" maxLength="5"
                                size="5" /></th>
                        <th scope="col"><input id="idSearchName" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchName" minlength="16" maxLength="16"
                                size="16" /></th>
                        <th scope="col"><input id="idSearchUmask" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchUmask" minlength="10" maxLength="10"
                                size="10" /></th>
                        <th scope="col"><input id="isSearchState" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchState" minlength="10" maxLength="10"
                                size="10" /></th>
                        <th scope="col"><input id="isSearchThreads" type="text" class="form-control"
                                aria-describedby="basic-addon2" v-model="searchThreads" minlength="3" maxLength="3"
                                size="3" /></th>
                        <th scope="col"></th>                                
                    </tr>
                </thead>
                <tbody>
                    <template v-for="process in processes">
                        <tr v-if="filter(process)">
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{ process.url
                                    }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{ process.pid
                                    }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{
                                    process.parentPid }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{ process.name
                                    }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{
                                    process.umask }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{
                                    process.processState }}</label></td>
                            <td class="vertical-align-middle"><label class="form-check-label text-light" v-bind:class="isMonitored(process)">{{
                                    process.numThreads }}</label></td>
                            <td class="vertical-align-middle">
                                <button class="btn btn-info small"><FontAwesomeIcon :icon="faChartSimple" /></button>
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

.toolbar-item {
    margin-left: 10px;
    margin-right: 5px;
}
table {
    width: 100%;
    margin: 5px;
}
.monitored {
    color: #d8ffab !important
}
</style>
