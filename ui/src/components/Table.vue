<template>
    <n-data-table ref="table" :columns="columns" :data="data" :pagination="pagination" :max-height="maxHeight" />
</template>

<script lang="ts" setup>
import { NIcon, useLoadingBar } from 'naive-ui';
import { type RendererElement, type RendererNode, type VNode, h, onMounted, onUnmounted, ref } from 'vue';
import { CloudDownloadSharp, FolderOpenOutline } from '@vicons/ionicons5';

import { useRouter } from 'vue-router';
const router = useRouter();
const loadingBar = useLoadingBar()
const columns = [
    {
        title: 'Name',
        key: 'name',
        width: 200,
        ellipsis: {
            tooltip: true
        }
    },

    {
        title: 'Size',
        key: 'size',
    },
    {
        title: 'Creation Date',
        key: 'creationDate',
    },
    {
        title: 'Last Modified',
        key: 'lastModified',
    },
    {
        title: 'Action',
        key: 'action',
    }
]

const maxHeight = ref(250)  // 初始高度设置为 250px

// 计算 max-height
function updateMaxHeight() {
    // 示例：设置 max-height 为视窗高度的 75%
    maxHeight.value = window.innerHeight - 275
}

function bytesToSize(bytes: number) {
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
    if (bytes === 0) return '0 Byte'
    const i = parseInt(String(Math.floor(Math.log(bytes) / Math.log(1024))))
    // 保留两位小数
    return (bytes / Math.pow(1024, i)).toFixed(2) + ' ' + sizes[i]
}

function timestampToDateTime(timestamp: number) {
    const date = new Date(timestamp * 1000)
    return date.toLocaleString()
}



const pagination = {
    pageSize: 15,
}
const icon_size = 20
const path = ref("")

const data: any = ref([])

const refreshData = async () => {
    let p = window.location.pathname
    if (p === '/') {
        p = ''
    }
    if (p.startsWith('/list')) {
        p = p.slice(5)
    }
    let url = `/api/list${p}`


    try {
        loadingBar.start()
        const res = await fetch(url)
        const json = await res.json()
        const files = json.files
        data.value = []
        files.forEach((file: { type: string; name: any; id: any; size: number; created_date_time: number; last_modified_date_time: number; }) => {
            let action: VNode<RendererNode, RendererElement, { [key: string]: any; }>;
            if (file.type === 'Folder') {
                if (p === '') {
                    action = h(NIcon, { onClick: () => router.push(`/list/${file.name}`), style: 'cursor: pointer;', size: icon_size }, { default: () => h(FolderOpenOutline) });
                } else {
                    action = h(NIcon, { onClick: () => router.push(`/list${p}/${file.name}`), style: 'cursor: pointer;', size: icon_size }, { default: () => h(FolderOpenOutline) });
                }
            } else {
                action = h(NIcon, { onClick: () => triggerDownload(`/api/download/${file.id}`, file.name), style: 'cursor: pointer;', size: icon_size }, { default: () => h(CloudDownloadSharp) });
            }

            data.value.push({
                name: file.name,
                size: file.size ? bytesToSize(file.size) : '',
                creationDate: file.created_date_time ? timestampToDateTime(file.created_date_time) : '',
                lastModified: file.last_modified_date_time ? timestampToDateTime(file.last_modified_date_time) : '',
                action: action
            })
        })
        loadingBar.finish()
    } catch (e) {
        loadingBar.error()
        console.error(e)
    }
}

function triggerDownload(url: string, fileName: string) {
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;  // 指定下载文件名
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}



router.afterEach(async () => {
    await refreshData()
})

onMounted(async () => {
    updateMaxHeight()
    window.addEventListener('resize', updateMaxHeight);
    // get now uri path
    await refreshData()
})


onUnmounted(() => {
    window.removeEventListener('resize', updateMaxHeight);
})
</script>
