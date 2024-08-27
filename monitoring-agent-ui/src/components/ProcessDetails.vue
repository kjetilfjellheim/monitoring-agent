<script setup>
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faRefresh } from '@fortawesome/free-solid-svg-icons';
</script>
<script>
import { ref } from 'vue';

const processStatus = ref(null);
const processStatm = ref(null);
const threads = ref(null);

export default {
    data() {
        return {
            processStatus: null,
            processStatm: null,
            threads: null
        }
    },
    props: {
        server: { required: true },
        pid: { required: true }
    },
    mounted() {
        this.refreshAll();    
    },
    methods: {       
        refreshAll() {
            this.refreshProcessData();
            this.refreshStatm();
            this.refreshThreads();
        }, 
        refreshProcessData() {
            var url = decodeURIComponent(this.$props.server) + "/processes/" + this.$props.pid;
            fetch(url)
                    .then(response => response.json())
                    .then(json => {
                        processStatus.value = json;
                    })
                    .catch(error => console.error('Error:', error));
        },
        refreshStatm() {
            var url = decodeURIComponent(this.$props.server) + "/processes/" + this.$props.pid + "/statm";
            fetch(url)
                    .then(response => response.json())
                    .then(json => {
                        json.totalSize = json.totalSize * json.pagesize / 1024;
                        json.residentSize = json.residentSize * json.pagesize / 1024;
                        json.sharedSize = json.sharedSize * json.pagesize / 1024;
                        json.trsSize = json.trsSize * json.pagesize / 1024;
                        json.drsSize = json.drsSize * json.pagesize / 1024;
                        json.lrsSize = json.lrsSize * json.pagesize / 1024;
                        json.dtSize = json.dtSize * json.pagesize / 1024;
                        processStatm.value = json;
                    })
                    .catch(error => console.error('Error:', error));
        },
        refreshThreads() {
            var url = decodeURIComponent(this.$props.server) + "/processes/" + this.$props.pid + "/threads";
            fetch(url)
                    .then(response => response.json())
                    .then(json => {
                        threads.value = json;
                    })
                    .catch(error => console.error('Error:', error));
        }
    }
};
</script>
<template>
    <div>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="collapse navbar-collapse">
            <ul class="navbar-nav mr-auto mt-2 mt-lg-0">
                <li class="nav-item">
                    <button class="btn btn-info small toolbar-item" @click="refreshAll()">
                        <FontAwesomeIcon :icon="faRefresh" />&nbsp;Refresh
                    </button>
                </li>
            </ul>
        </div>
    </nav>
    <div class="container-fluid" v-if="processStatus">
        <div class="row">
            <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-4">
                <dl class="row">
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Process id</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.pid }}</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Parent id</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.parentPid }}</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Name</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.name }}</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Umask</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.umask }}</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">State</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.processState }}</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Number of threads</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatus?.numThreads }}</dd>
                </dl>
            </div>
            <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-3">
                <dl class="row">
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Total</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.totalSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Resident</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.residentSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Shared size</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.sharedSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Trs size</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.trsSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Drs size</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.drsSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Lrs size</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.lrsSize }} Kb</dd>
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Dt size</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">{{ processStatm?.dtSize }} Kb</dd>
                </dl>
            </div>
            <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-3">
                <dl class="row">                   
                    <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">Groups</dt>
                    <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6 text-light">
                        <span v-for="group in processStatus?.groups" class="badge rounded-pill bg-primary group">{{ group }}</span>
                    </dd>
                </dl>
            </div>
            <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-6">
                <table class="table table-striped-self table-dark table-responsive">
                    <thead class="thead-dark">
                        <tr>
                            <th scope="col">Thread id</th>
                            <th scope="col">Name</th>
                            <th scope="col">State</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="thread in threads">
                            <td>{{ thread.pid }}</td>
                            <td>{{ thread.name }}</td>
                            <td>{{ thread.processState }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <!--
            <div class="col col-sm-12 col-md-12 col-lg-6 col-xl-6 text-light">
                Resident memory over time
                <Chart :data="data" :margin="margin" :direction="direction">
                    <template #layers>
                        <Grid strokeDasharray="2,2" />
                        <Line :dataKeys="['name', 'pl']" />
                    </template>
                </Chart>
            </div>
            <div class="col col-sm-12 col-md-12 col-lg-6 col-xl-6 text-light">
                Max memory usage in the specified period
                <Chart :data="data" :margin="margin" :direction="direction">
                    <template #layers>
                        <Grid strokeDasharray="2,2" />
                        <Line :dataKeys="['name', 'pl']" />
                    </template>
                </Chart>
            </div>
            <div class="col col-sm-12 col-md-12 col-lg-12 col-xl-12">
                <table class="table table-striped-self table-dark table-responsive">
                    <thead class="thead-dark">
                        <tr><th colspan="4">Process limitations</th></tr>
                        <tr>
                            <th scope="col">Resource</th>
                            <th scope="col">Soft Limit</th>
                            <th scope="col">Hard Limit</th>
                            <th scope="col">Units</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Max cpu time</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>seconds</td>
                        </tr>
                        <tr>
                            <td>Max file size</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max data size</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max stack size</td>
                            <td>8388608</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max core file size</td>
                            <td>0</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max resident set</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max processes</td>
                            <td>124811</td>
                            <td>124811</td>
                            <td>processes</td>
                        </tr>
                        <tr>
                            <td>Max open files</td>
                            <td>1048576</td>
                            <td>1048576</td>
                            <td>files</td>
                        </tr>
                        <tr>
                            <td>Max locked memory</td>
                            <td>4099702784</td>
                            <td>4099702784</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max address space</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max file locks</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>locks</td>
                        </tr>
                        <tr>
                            <td>Max pending signals</td>
                            <td>124811</td>
                            <td>124811</td>
                            <td>signals</td>
                        </tr>
                        <tr>
                            <td>Max msgqueue size</td>
                            <td>819200</td>
                            <td>819200</td>
                            <td>bytes</td>
                        </tr>
                        <tr>
                            <td>Max nice priority</td>
                            <td>0</td>
                            <td>0</td>
                            <td></td>
                        </tr>
                        <tr>
                            <td>Max realtime priority</td>
                            <td>0</td>
                            <td>0</td>
                            <td></td>
                        </tr>
                        <tr>
                            <td>Max realtime timeout</td>
                            <td>unlimited</td>
                            <td>unlimited</td>
                            <td>us</td>
                        </tr>
                    </tbody>
                </table>
            </div>-->
        </div>
    </div>
    </div>
</template>
<style scoped>
.toolbar-item {
    margin-left: 10px;
    margin-right: 5px;
}

.group {
    margin-left: 2px;
    margin-right: 2px;
}

#app {
    color: #2ecc71
}

.col {
    margin: 0px;
}

.table { 
    margin-top: 50px;
}

.table-striped-self>tbody>tr:nth-child(odd)>td, 
.table-striped-self>tbody>tr:nth-child(odd)>th {
   background-color: rgba(var(--bs-dark-rgb),var(--bs-bg-opacity))!important;
 }

 .table-striped-self>tbody>tr:nth-child(even)>td, 
.table-striped-self>tbody>tr:nth-child(even)>th {
   background-color: rgb(65, 65, 65); 
 }
</style>
