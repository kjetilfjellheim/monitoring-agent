<script setup>
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faRefresh } from '@fortawesome/free-solid-svg-icons';
import { Line } from 'vue-chartjs'

</script>
<script>
import { ref } from 'vue';
import {
    Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend
} from 'chart.js';
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

const processStatus = ref(null);
const processStatm = ref(null);
const threads = ref(null);
const memUseHistData = ref(null);
const haveUseHistData = ref(false);

export default {
    data() {
        return {
            processStatus: null,
            processStatm: null,
            threads: null,
            memUseHistData: null,
            haveUseHistData: null,
            memUseHistOptions: {
                responsive: true,
                maintainAspectRatio: false,
                borderWidth: 1,
                scales: {
                    y: {
                        beginAtZero: true
                    },
                    x: {
                        ticks: {
                            display: true,
                            callback: function (val, index) {
                                return index % 2 === 0 ? this.getLabelForValue(val) : '';
                            },
                            font: {
                                size: 6
                            }
                        }
                    },
                },
                elements: {
                    point: {
                        radius: 0
                    }
                },
                tooltip: {
                    usePointStyle: true,
                }
            },
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
            this.get_memuse_hist();
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
        },
        get_memuse_hist() {
            fetch(decodeURIComponent(this.$props.server) + "/processes/" + this.$props.pid + "/statm/historical")
                .then(response => response.json())
                .then(json => {
                    haveUseHistData.value = false;
                    let labels = [];
                    let residentMemory = [];
                    let sharedMemory = [];
                    let trsMemory = [];
                    let lrsMemory = [];
                    let drsMemory = [];
                    let dtMemory = [];
                    json.usedMemory.forEach(element => {
                        haveUseHistData.value = true;
                        labels.push(element.timestamp);
                        residentMemory.push(element.residentSize / 1024);
                        sharedMemory.push(element.sharedSize / 1024);
                        trsMemory.push(element.trsSize / 1024);
                        lrsMemory.push(element.lrsSize / 1024);
                        drsMemory.push(element.drsSize / 1024);
                        dtMemory.push(element.dtSize / 1024);
                    });
                    memUseHistData.value = {
                        labels: labels,
                        datasets: [{
                            backgroundColor: 'rgba(0,255,255,1)',
                            borderColor: 'rgba(0,255,255,1)',
                            label: 'Resident',
                            data: residentMemory,
                            fill: false
                        },
                        {
                            backgroundColor: 'rgba(255,0,255,1)',
                            borderColor: 'rgba(255,0,255,1)',
                            label: 'Shared',
                            data: sharedMemory,
                            fill: false,
                            hidden: true
                        },
                        {
                            backgroundColor: 'rgba(255,0,0,1)',
                            borderColor: 'rgba(255,0,0,1)',
                            label: 'Trs',
                            data: trsMemory,
                            fill: false,
                            hidden: true
                        },
                        {
                            backgroundColor: 'rgba(0,255,0,1)',
                            borderColor: 'rgba(0,255,0,1)',
                            label: 'Lrs',
                            data: lrsMemory,
                            fill: false,
                            hidden: true
                        },
                        {
                            backgroundColor: 'rgba(0,0,255,1)',
                            borderColor: 'rgba(0,0,255,1)',
                            label: 'Drs',
                            data: drsMemory,
                            fill: false,
                            hidden: true
                        },
                        {
                            backgroundColor: 'rgba(255,255,0,1)',
                            borderColor: 'rgba(255,255,0,1)',
                            label: 'Dt',
                            data: dtMemory,
                            fill: false,
                            hidden: true
                        }]
                    };
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
        <div class="card">
            <div class="card-body">
                <div class="container-fluid" v-if="processStatus">
                    <div class="row">
                        <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-4">
                            <dl class="row">
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Process id</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.pid }}</dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Parent id</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.parentPid }}
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Name</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.name }}</dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Umask</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.umask }}</dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">State</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.processState }}
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Number of threads</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatus?.numThreads }}
                                </dd>
                            </dl>
                        </div>
                        <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-4">
                            <dl class="row">
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Total</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.totalSize }} Kb
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Resident</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.residentSize }}
                                    Kb</dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Shared size</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.sharedSize }} Kb
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Trs size</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.trsSize }} Kb
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Drs size</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.drsSize }} Kb
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Lrs size</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.lrsSize }} Kb
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">Dt size</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-6">{{ processStatm?.dtSize }} Kb
                                </dd>
                            </dl>
                        </div>
                        <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-4">
                            <dl class="row">
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-4">Groups</dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-8">
                                    <span v-for="group in processStatus?.groups"
                                        class="badge rounded-pill bg-primary group">{{ group }}</span>
                                </dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-12">&nbsp</dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-4"></dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-2">Real</dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-2">Eff</dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-2">Saved</dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-2">File</dt>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-4" v-if="processStatus?.uid">Uid
                                </dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-2"
                                    v-for="uid in processStatus?.uid">{{ uid }}</dd>
                                <dt class="col col-sm-12 col-md-12 col-lg-12 col-xl-4" v-if="processStatus?.gid">Gid
                                </dt>
                                <dd class="col col-sm-12 col-md-12 col-lg-12 col-xl-2"
                                    v-for="gid in processStatus?.gid">{{ gid }}</dd>
                            </dl>
                        </div>
                        <div class="col col-sm-12 col-md-12 col-lg-12 col-xl-12 chart" v-if="haveUseHistData">
                            <Line :data="memUseHistData" :options="memUseHistOptions" />
                        </div>
                        <div class="col col-sm-12 col-md-6 col-lg-6 col-xl-6">
                            <table class="table table-responsive">
                                <thead class="">
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
            <tr>
                <th colspan="4">Process limitations</th>
            </tr>
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
    margin-top: 5px;
}

.card {
    height: 100%;
    margin: 2px;
}

.card-body {
    background-color: #cfc3c3;
}

.table thead th {
    background-color: #cfc3c3 !important;
}

.table tbody td {
    background-color: #cfc3c3 !important;
}

tr {
    border-style: none;
}

thead {
    border-bottom: 1px solid black;
}

.chart {
    height: 300px;
    width: 100%;
}
</style>
