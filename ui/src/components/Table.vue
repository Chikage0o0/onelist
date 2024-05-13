<template>
    <n-data-table ref="table" :columns="columns" :data="data" :pagination="paginationReactive" :row-props="rowProps" />
</template>

<script lang="ts" setup>
import { NIcon, useLoadingBar } from 'naive-ui';
import { type RendererElement, type RendererNode, type VNode, h, onMounted, onUnmounted, ref, reactive, onBeforeMount } from 'vue';
import { CloudDownloadSharp, FolderOpenOutline } from '@vicons/ionicons5';

import { useRouter } from 'vue-router';
const router = useRouter();
const loadingBar = useLoadingBar()
const columns = ref([
    {
        title: 'Name',
        key: 'name',
        width: 300,
        ellipsis: {
            tooltip: true
        },
        sorter: 'default'
    },
    {
        title: 'Size',
        key: 'size',
        width: 150,
        sorter: 'default',
        render: (row: any) => {
            return bytesToSize(row.size)
        }
    },
    {
        title: 'Creation Date',
        key: 'creationDate',
        sorter: 'default',
        render: (row: any) => {
            return timestampToDateTime(row.lastModified)
        }
    },
    {
        title: 'Last Modified',
        key: 'lastModified',
        sorter: 'default',
        render: (row: any) => {
            return timestampToDateTime(row.lastModified)
        }
    }
]);



const rowProps = (row: any) => {
    // 修改光标
    return {
        style: {
            cursor: 'pointer',
            // 不可编辑
            userSelect: 'none',
            webkitUserSelect: 'none',
        },
        onclick: () => {
            if (row.type === 'Folder') {

                router.push(`/list${row.path}`)

            } else if (row.type === 'Video') {
                router.push(`/video${row.path}`)
            } else {
                triggerDownload(`/api/download/${row.id}`, row.name)
            }
        }
    }
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



const paginationReactive = reactive({
    page: 1,
    pageSize: 20,
    showSizePicker: true,
    pageSizes: [10, 15, 20, 50, 100],
    onChange: (page: number) => {
        paginationReactive.page = page
    },
    onUpdatePageSize: (pageSize: number) => {
        paginationReactive.pageSize = pageSize
        paginationReactive.page = 1
    }
})

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
        files.forEach((file: { type: string; name: any; id: any; size: number; created_date_time: number; last_modified_date_time: number; full_path: string }) => {
            data.value.push({
                name: file.name,
                size: file.size ? file.size : 0,
                creationDate: file.created_date_time ? file.created_date_time : 0,
                lastModified: file.last_modified_date_time ? file.last_modified_date_time : 0,
                type: file.type,
                id: file.id,
                path: file.full_path
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



const unrefresh = router.afterEach(async () => {
    if (router.currentRoute.value.path.startsWith('/list') || router.currentRoute.value.path === '/') {
        await refreshData()
    }
})

onBeforeMount(async () => {
    // 如果是手机，只显示文件名
    if (window.innerWidth < 768) {
        columns.value = [
            {
                title: 'Name',
                key: 'name',
                width: window.innerWidth - 150,
                ellipsis: {
                    tooltip: true
                },
                sorter: 'default'
            }
        ]
    }
})

onBeforeMount(async () => {
    // get now uri path
    await refreshData()
})

onUnmounted(() => {
    unrefresh()
})



</script>
