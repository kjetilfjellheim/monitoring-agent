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
export default {
  data() {
    return {
      data: [],
      tooltip_statistics: "Statistics\nUser: normal processes executing in user mode\nNice: niced processes executing in user mode\nSystem: processes executing in kernel mode\nIdle: twiddling thumbs\nIOwait: waiting for I/O to complete\nIrq: servicing interrupts\nsoftirq: servicing softirqs\nSteal: It counts the ticks spent executing other virtual hosts.",
      tooltip_load_average: "Load average figures giving the number of jobs in the run queue (state R) or waiting for disk I/O (state D) aver‐ aged over 1, 5, and 15 minutes. They are the same as the load average numbers given by uptime(1) and other programs.",
      tooltip_meminfo: "Memory information\nTotal: total usable RAM\nUsed: The total amount of RAM used by the system\nAvailable: An estimate of how much memory is available for starting new applications, without swapping.",
      tooltip_swap: "Swap information\nTotal: total swap space in bytes\nFree: free swap space in bytes",
      tooltip_processes: "Process information\nTotal processes: total number of processes\nRunning processes: number of running processes",
      loadAvgHistOptions: {
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
              callback: function(val, index) {
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
      freeMemHistOptions: {
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
              callback: function(val, index) {
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
        }
      }      
    }
  },
  mounted() {
    this.refresh();
  },
  methods: {
    refresh() {
      const data = ref(null);
      const URL_NAME = 'apiUrls';
      let currentServers = localStorage.getItem(URL_NAME);
      let servers = JSON.parse(currentServers);
      data.value = [];
      let index = 0;
      for (let server of servers) {
        let server_data = {
          url: server.url,
          name: server.name,
          loadavg: ref(null),
          meminfo: ref(null),
          cpuinfo: ref(null),
          stat: ref(null),
          loadAvgHistData: ref(null),
          freeMemHistData: ref(null),
          id: index++
        };
        this.get_loadavg(server_data, server.url);
        this.get_meminfo(server_data, server.url);
        this.get_cpuinfo(server_data, server.url);
        this.get_stat(server_data, server.url);
        this.get_load_avg_hist(server_data, server.url);
        this.get_free_mem_hist(server_data, server.url);
        data.value.push(server_data);
      }
      this.data = data.value;
    },
    get_load_avg_hist(server, url) {
      fetch(url + "/loadavg/historical")
        .then(response => response.json())
        .then(json => {
          let labels = [];
          let values = [];
          json.loadavg1min.forEach(element => {
            labels.push(element.timestamp);
            values.push(element.value);
          });
          if (labels !== null) {
            server.loadAvgHistData.value = {
              labels: labels,
              datasets: [{
                backgroundColor: 'rgba(255,0,0,1)',
                borderColor: 'rgba(255,0,0,1)',
                label: 'Load Average',
                data: values,
                fill: false
              }]
            };
          }
        })
        .catch(error => console.error('Error:', error));
    },
    get_free_mem_hist(server, url) {
      fetch(url + "/meminfo/historical")
        .then(response => response.json())
        .then(json => {
          let labels = [];
          let values = [];
          json.freemem.forEach(element => {
            labels.push(element.timestamp);
            values.push(element.value);
          });
          server.freeMemHistData.value = {
            labels: labels,
            datasets: [{
              backgroundColor: 'rgba(255,0,0,1)',
              borderColor: 'rgba(255,0,0,1)',
              label: 'Free memory',
              data: values,
              fill: false
            }]
          };
        })
        .catch(error => console.error('Error:', error));
    },    
    get_loadavg(server, url) {
      fetch(url + "/loadavg/current")
        .then(response => response.json())
        .then(json => {
          server.loadavg.value = json;
        })
        .catch(error => console.error('Error:', error));
    },
    get_meminfo(server, url) {
      fetch(url + "/meminfo/current")
        .then(response => response.json())
        .then(json => {
          server.meminfo.value = json;
        })
        .catch(error => console.error('Error:', error));
    },
    get_cpuinfo(server, url) {
      fetch(url + "/cpuinfo/current")
        .then(response => response.json())
        .then(json => {
          server.cpuinfo.value = json;
        })
        .catch(error => console.error('Error:', error));
    },
    get_stat(server, url) {
      fetch(url + "/stat/current")
        .then(response => response.json())
        .then(json => {
          server.stat.value = json;
        })
        .catch(error => console.error('Error:', error));
    }
  }
};
</script>
<template>
  <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
    <div class="collapse navbar-collapse">
      <ul class="navbar-nav mr-auto mt-2 mt-lg-0">
        <li class="nav-item">
          <button class="btn btn-info small toolbar-item" @click="refresh()">
            <FontAwesomeIcon :icon="faRefresh" />&nbsp;Refresh
          </button>
        </li>
      </ul>
    </div>
  </nav>
  <div class="container-fluid">
    <br />
    <div class="accordion" id="accordion">
      <template v-for="server in data">
        <div class="accordion-item">
          <h2 class="accordion-header">
            <button class="accordion-button bg-primary" type="button" data-bs-toggle="collapse"
              v-bind:data-bs-target="'#' + server?.id" aria-expanded="true" v-bind:aria-controls="'#' + server?.id">
              {{ server?.name }}&nbsp;({{ server?.url }})
            </button>
          </h2>
          <div v-bind:id="server?.id" class="accordion-collapse collapse bg-dark" data-bs-parent="#accordion">
            <div class="accordion-body">
              <div class="jumbotron">              
                <div class="row">
                  <div class="col col-sm-12 col-md-12 col-lg-12">
                    <div class="card">
                      <div class="card-body">
                        <div class="container-fluid">
                          <table>
                            <thead>
                              <tr>
                                <th scope="col" colspan="2" data-bs-toggle="tooltip" data-bs-placement="bottom" v-bind:title="tooltip_load_average">
                                  Load Average
                                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" class="bi bi-info-circle" viewBox="0 0 16 16">
                                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14m0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16" />
                                    <path d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0" />
                                  </svg>                                
                                </th>
                                <th scope="col" colspan="2" data-bs-toggle="tooltip" data-bs-placement="bottom" v-bind:title="tooltip_meminfo">
                                  Memory
                                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" class="bi bi-info-circle" viewBox="0 0 16 16">
                                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14m0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16" />
                                    <path d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0" />
                                  </svg>                                
                                </th>
                                <th scope="col" colspan="2" data-bs-toggle="tooltip" data-bs-placement="bottom" v-bind:title="tooltip_swap">
                                  Swap
                                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" class="bi bi-info-circle" viewBox="0 0 16 16">
                                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14m0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16" />
                                    <path d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0" />
                                  </svg>                                                                
                                </th>   
                                <th scope="col" colspan="2">
                                  Cpu                                                             
                                </th>
                              </tr>                                                         
                            </thead>
                            <tbody>
                              <tr>
                                <td>1 min</td>
                                <td>{{ server?.loadavg?.loadAvg1Min }}</td>
                                <td>Total</td>
                                <td>{{ server?.meminfo?.totalMem }} bytes</td> 
                                <td>Total</td>   
                                <td>{{ server?.meminfo?.swapTotal }} bytes</td>
                                <td>Model name</td>
                                <td v-if="server?.cpuinfo?.length > 0">{{ server?.cpuinfo[0]?.modelName }}</td>
                              </tr>
                              <tr>
                                <td>5 min</td>
                                <td>{{ server?.loadavg?.loadAvg5Min }}</td>
                                <td>Free</td>
                                <td>{{ server?.meminfo?.freeMem }} bytes</td>    
                                <td>Free</td>
                                <td>{{ server?.meminfo?.swapFree }} bytes</td> 
                                <td>Vendor id</td>
                                <td v-if="server?.cpuinfo?.length > 0">{{ server?.cpuinfo[0]?.vendorId }}</td>                                                            
                              </tr>
                              <tr>
                                <td>15 min</td>
                                <td>{{ server?.loadavg?.loadAvg15Min }}</td>
                                <td>Available</td>
                                <td>{{ server?.meminfo?.availableMem }} bytes</td>                                
                                <td></td>
                                <td></td>
                                <td>Cores</td>
                                <td v-if="server?.cpuinfo?.length > 0 && server?.cpuinfo[0]?.cpuCores">{{ server?.cpuinfo[0]?.cpuCores }}</td>
                              </tr>
                            </tbody>
                          </table>
                        </div>                        
                      </div>
                    </div>
                  </div>                  
                  <div class="col col-sm-12 col-md-12 col-lg-12">
                    <div class="card-body chart" v-if="server?.loadAvgHistData && server?.loadAvgHistData.labels?.length > 0">
                      <Line :data="server?.loadAvgHistData" :options="loadAvgHistOptions" />
                    </div>
                  </div>
                  <div class="col col-sm-12 col-md-12 col-lg-12">
                    <div class="card-body chart" v-if="server?.loadAvgHistData && server?.freeMemHistData?.labels?.length > 0">
                      <Line :data="server?.freeMemHistData" :options="freeMemHistOptions" />
                    </div>
                  </div>
                  <div class="col col-12">
                    <div class="card">                      
                      <div class="card-body">
                        <dl class="row">                      
                          <dt class="col-sm-2 small no-margin" data-bs-toggle="tooltip" data-bs-placement="bottom" v-bind:title="tooltip_statistics">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-info-circle" viewBox="0 0 16 16">
                              <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14m0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16" />
                              <path d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0" />
                            </svg>                            
                          </dt>
                        </dl>
                        <dl class="row">
                          <dt class="col-sm-2 small no-margin">Number of processes</dt>
                          <dd class="col-sm-2 small text-truncate no-margin">{{ server?.stat?.numProcesses }}</dd>
                        </dl>
                        <dl class="row">
                          <dt class="col-sm-2 small no-margin">Processes running</dt>
                          <dd class="col-sm-2 small text-truncate no-margin">{{ server?.stat?.processesRunning }}</dd>
                        </dl>
                        <dl class="row">
                          <dt class="col-sm-2 small no-margin">Processes blocked</dt>
                          <dd class="col-sm-2 small text-truncate no-margin">{{ server?.stat?.processesBlocked }}</dd>
                        </dl>
                        <dl class="row">
                          <dt class="col-sm-2 small no-margin">Number of interrupts</dt>
                          <dd class="col-sm-2 small text-truncate no-margin">{{ server?.stat?.numInterrupts }}</dd>
                        </dl>
                        <dl class="row">
                          <table table table-responsive>
                            <thead>
                              <tr>
                                <th scope="col">Cpu</th>
                                <th scope="col">User</th>
                                <th scope="col">System</th>
                                <th scope="col">Nice</th>
                                <th scope="col">Idle</th>
                                <th scope="col">Iowait</th>
                                <th scope="col">Irq</th>
                                <th scope="col">Softirq</th>
                                <th scope="col">Steal</th>
                              </tr>
                            </thead>
                            <tbody>
                              <tr v-for="cpu in server?.stat?.cpus">
                                <td>{{ cpu.name }}</td>
                                <td>{{ cpu.user }}</td>
                                <td>{{ cpu.system }}</td>
                                <td>{{ cpu.nice }}</td>
                                <td>{{ cpu.idle }}</td>
                                <td>{{ cpu.iowait }}</td>
                                <td>{{ cpu.irq }}</td>
                                <td>{{ cpu.softirq }}</td>
                                <td>{{ cpu.steal }}</td>
                              </tr>
                            </tbody>
                          </table>
                        </dl>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>
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

.card-body {
  background-color: #cfc3c3;
}

td {
  background-color: #cfc3c3;
  padding-left: 30px;
  padding-right: 5px;
}

th {
  background-color: #cfc3c3;
  padding-left: 30px;
  padding-right: 5px;
}

.no-margin {
  margin: 0px;
}

.toolbar-item {
  margin-left: 10px;
  margin-right: 5px;
}

.title {
  padding-left: 6px;
}

.chart {
  height: 300px;
}
</style>
