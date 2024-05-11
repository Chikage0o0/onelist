<template>
    <n-breadcrumb>
        <n-breadcrumb-item>
            <router-link to="/">Home</router-link>
        </n-breadcrumb-item>
        <n-breadcrumb-item v-for="item in breadcrumb" :key="item">
            <router-link :to="item.to">{{ item.label }}</router-link>
        </n-breadcrumb-item>
    </n-breadcrumb>
</template>

<script lang="ts" setup>
import { onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router';

const route = useRoute()

const breadcrumb = ref<{
    label: string;
    to: string;
}[]>(
    []
)

const update = (path: string) => {
    // Path转义
    breadcrumb.value = path.split('/').filter((item) => item !== '').map((item, index, arr) => {
        return {
            label: urldecode(item),
            to: "/list/" + arr.slice(0, index + 1).join("/"),
        }
    })
}

watch(() => route.fullPath, (path) => {

    let p = path
    if (p === '/') {
        p = ''
    }
    if (p.startsWith('/list')) {
        p = p.slice(5)
    }
    update(p)
})

onMounted(() => {
    let p = route.fullPath
    if (p === '/') {
        p = ''
    }
    if (p.startsWith('/list')) {
        p = p.slice(5)
    }
    update(p)
})



const urldecode = (str: string) => {
    return decodeURIComponent(str.replace(/\+/g, '%20'))
}
</script>