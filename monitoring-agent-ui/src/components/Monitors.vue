<script>
import { ref } from 'vue';
const products = ref(null);
fetch('http://greyhawk.forgottendonkey.com:64999/monitors/status')
    .then(response => response.json())
    .then(data => data.monitors);
</script>

<template>
    <div class="card m-3">
        <h5 class="card-header text-center">Vue 3 fetch data from API</h5>
        <div class="card-body">
            <div v-if="products">
                <h5>Products</h5>
                <ul class="mb-0">
                    <li v-for="product in products" :key="product.id">{{product.name}}</li>
                </ul>
            </div>
            <div v-if="!products" class="text-center">
                <div class="spinner-border spinner-border-sm"></div>
            </div>
        </div>
    </div>
</template>