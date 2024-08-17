<script setup>
    import { ref } from 'vue';
    const data = ref(null);
    fetch('http://localhost:64999/monitors/status')
        .then(response => response.json())
        .then(response => data.value = response)
        .catch(error => alert(error));
</script>

<script>
export default {
    data() {
        return {
            showErrorStatus: true, 
            showOkStatus: true, 
        };
    },
    methods: {
        handleCheckboxChange() {
            // This method will be called when the checkbox state changes
            console.log("Checkbox state changed. Checked:", this.isChecked);
        },
    },
};
</script>

<template>
    <div class="form-check toolbar-item">
        <input class="form-check-input" type="checkbox" id="showOErrorStatus" v-model="showErrorStatus">
        <label class="form-check-label" for="showOErrorStatus">
            Show Error status
        </label>
    </div>
    <div class="form-check toolbar-item">
        <input class="form-check-input" type="checkbox" id="showOkStatus" v-model="showOkStatus">
        <label class="form-check-label" for="showOkStatus">
            Show Ok status
        </label>
    </div>
    <div v-if="data">
        <div class="container-fluid">
            <div class="row row-cols-1 row-cols-sm-1 row-cols-md-4 row-cols-lg-6">
                <template v-for="monitor in data">  

                <div class="col" v-if="(monitor.status === 'Ok' && showOkStatus) || (monitor.status === 'Error' && showErrorStatus)">
                    <div class="card">
                        <div v-if="monitor.status === 'Ok'" class="card-header bg-success">
                            <h5>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor"
                                    class="bi bi-shield-check" viewBox="0 0 16 16">
                                    <path
                                        d="M5.338 1.59a61 61 0 0 0-2.837.856.48.48 0 0 0-.328.39c-.554 4.157.726 7.19 2.253 9.188a10.7 10.7 0 0 0 2.287 2.233c.346.244.652.42.893.533q.18.085.293.118a1 1 0 0 0 .101.025 1 1 0 0 0 .1-.025q.114-.034.294-.118c.24-.113.547-.29.893-.533a10.7 10.7 0 0 0 2.287-2.233c1.527-1.997 2.807-5.031 2.253-9.188a.48.48 0 0 0-.328-.39c-.651-.213-1.75-.56-2.837-.855C9.552 1.29 8.531 1.067 8 1.067c-.53 0-1.552.223-2.662.524zM5.072.56C6.157.265 7.31 0 8 0s1.843.265 2.928.56c1.11.3 2.229.655 2.887.87a1.54 1.54 0 0 1 1.044 1.262c.596 4.477-.787 7.795-2.465 9.99a11.8 11.8 0 0 1-2.517 2.453 7 7 0 0 1-1.048.625c-.28.132-.581.24-.829.24s-.548-.108-.829-.24a7 7 0 0 1-1.048-.625 11.8 11.8 0 0 1-2.517-2.453C1.928 10.487.545 7.169 1.141 2.692A1.54 1.54 0 0 1 2.185 1.43 63 63 0 0 1 5.072.56" />
                                    <path
                                        d="M10.854 5.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 7.793l2.646-2.647a.5.5 0 0 1 .708 0" />
                                </svg>
                                {{ monitor.name }}

                            </h5>
                        </div>
                        <div v-else="monitor.status === 'Ok'" class="card-header bg-danger">
                            <h5>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor"
                                    class="bi bi-shield-x" viewBox="0 0 16 16">
                                    <path
                                        d="M5.338 1.59a61 61 0 0 0-2.837.856.48.48 0 0 0-.328.39c-.554 4.157.726 7.19 2.253 9.188a10.7 10.7 0 0 0 2.287 2.233c.346.244.652.42.893.533q.18.085.293.118a1 1 0 0 0 .101.025 1 1 0 0 0 .1-.025q.114-.034.294-.118c.24-.113.547-.29.893-.533a10.7 10.7 0 0 0 2.287-2.233c1.527-1.997 2.807-5.031 2.253-9.188a.48.48 0 0 0-.328-.39c-.651-.213-1.75-.56-2.837-.855C9.552 1.29 8.531 1.067 8 1.067c-.53 0-1.552.223-2.662.524zM5.072.56C6.157.265 7.31 0 8 0s1.843.265 2.928.56c1.11.3 2.229.655 2.887.87a1.54 1.54 0 0 1 1.044 1.262c.596 4.477-.787 7.795-2.465 9.99a11.8 11.8 0 0 1-2.517 2.453 7 7 0 0 1-1.048.625c-.28.132-.581.24-.829.24s-.548-.108-.829-.24a7 7 0 0 1-1.048-.625 11.8 11.8 0 0 1-2.517-2.453C1.928 10.487.545 7.169 1.141 2.692A1.54 1.54 0 0 1 2.185 1.43 63 63 0 0 1 5.072.56" />
                                    <path
                                        d="M6.146 5.146a.5.5 0 0 1 .708 0L8 6.293l1.146-1.147a.5.5 0 1 1 .708.708L8.707 7l1.147 1.146a.5.5 0 0 1-.708.708L8 7.707 6.854 8.854a.5.5 0 1 1-.708-.708L7.293 7 6.146 5.854a.5.5 0 0 1 0-.708" />
                                </svg>
                                {{ monitor.name }}
                            </h5>
                        </div>
                        <div class="card-body">
                            <dl class="row">
                                <dt class="col-sm-5 small bg-light no-margin">Current status</dt>
                                <dd class="col-sm-7 small text-truncate bg-light no-margin">{{ monitor.status }}</dd>
                                <dt class="col-sm-5 small no-margin">Server</dt>
                                <dd class="col-sm-7 small text-truncate no-margin">Localhost</dd>
                                <dt class="col-sm-5 small no-margin" v-if="monitor.lastSuccessfulTime != null">Last
                                    successful time</dt>
                                <dd class="col-sm-7 small no-margin text-truncate"
                                    v-if="monitor.lastSuccessfulTime != null">{{ monitor.lastSuccessfulTime }}</dd>
                                <dt class="col-sm-5 small no-margin" v-if="monitor.lastErrorTime != null">Last eror time
                                </dt>
                                <dd class="col-sm-7 small no-margin text-truncate" v-if="monitor.lastErrorTime != null">
                                    {{ monitor.lastErrorTime }}</dd>
                                <dt class="col-sm-12 small no-margin" v-if="monitor.lastError != null">Last error</dt>
                                <dd class="col-sm-12 small no-margin xtext-truncate" v-if="monitor.lastError != null"
                                    :key="monitor._id" data-bs-toggle="tooltip" data-bs-placement="top"
                                    v-bind:title="monitor.lastError">{{ monitor.lastError }}</dd>
                            </dl>
                        </div>
                    </div>
                </div>
                </template>
            </div>
        </div>
    </div>
    <div v-if="!data" class="text-center">
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
}
</style>