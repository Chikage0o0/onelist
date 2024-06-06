<template>
    <n-data-table ref="table" :columns="columns" :data="data" :pagination="paginationReactive" :row-props="rowProps" />
</template>

<script lang="ts" setup>
import { NFlex, NIcon, useLoadingBar, useMessage } from 'naive-ui';
import { h, onMounted, onUnmounted, ref, reactive, onBeforeMount } from 'vue';
import { CloudDownloadSharp, CopyOutline, FolderOpenOutline, VideocamOutline } from '@vicons/ionicons5';

import { useRouter } from 'vue-router';
const router = useRouter();
const loadingBar = useLoadingBar()
const message = useMessage()
const columns = ref([
    {
        title: 'Name',
        key: 'name',
        ellipsis: {
            tooltip: true
        },
        sorter: 'default'
    },
    {
        title: 'Size',
        key: 'size',
        width: 120,
        sorter: 'default',
        render: (row: any) => {
            return bytesToSize(row.size)
        }
    },
    {
        title: 'Last Modified',
        key: 'lastModified',
        width: 200,
        sorter: 'default',
        render: (row: any) => {
            return timestampToDateTime(row.lastModified)
        },
        disabled: true
    },
    {
        title: 'Action',
        key: 'action',
        width: 80,
        render: (row: any) => {
            if (row.type === 'Folder') {
                return h(NFlex, { justifyContent: 'center' }, {
                    default: () => [
                        h(NIcon, { onClick: () => { router.push(`/list${encodeURI(row.path)}`) } }, { default: () => h(FolderOpenOutline) })
                    ]
                })
            }
            else if (row.type === 'Video') {
                return h(NFlex, { justifyContent: 'center' }, {
                    default: () => [h(NIcon, { onClick: () => { router.push(`/video${encodeURI(row.path)}`) } }, { default: () => h(VideocamOutline) })]
                })

            }
            else {
                return h(NFlex, { justifyContent: 'center' }, {
                    default: () => [
                        h(NIcon, { onClick: () => { triggerDownload(`/api/download/${row.id}`, row.name) } }, { default: () => h(CloudDownloadSharp) }),
                        // 复制链接到剪贴板
                        h(NIcon, { onClick: () => { copyToClipboard(`/api/download/${row.id}`) } }, { default: () => h(CopyOutline) }),
                    ]
                })

            }
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
        // onclick: () => {
        //     if (row.type === 'Folder') {

        //         router.push(`/list${row.path}`)

        //     } else if (row.type === 'Video') {
        //         router.push(`/video${row.path}`)
        //     } else {
        //         triggerDownload(`/api/download/${row.id}`, row.name)
        //     }
        // }
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

        //save to localstorage
        let path = window.location.pathname
        localStorage.setItem('page:' + path, paginationReactive.page.toString())
    },
    onUpdatePageSize: (pageSize: number) => {
        paginationReactive.pageSize = pageSize
        paginationReactive.page = 1

        //save to localstorage
        let path = window.location.pathname
        localStorage.setItem('page:' + path, paginationReactive.page.toString())
        localStorage.setItem('pageSize', paginationReactive.pageSize.toString())
    },

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
        files.forEach((file: { type: string; name: any; id: any; size: number; last_modified_date_time: number; full_path: string }) => {
            data.value.push({
                name: file.name,
                size: file.size ? file.size : 0,
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
    message.success('Download started' + fileName,
        {
            duration: 2000
        });

}

function copyToClipboard(uri: string) {
    // 拼接得出url
    let url = document.location.protocol + "//" + document.location.host + uri;

    navigator.clipboard.writeText(url).then(() => {
        message.success('Copied to clipboard',
            {
                duration: 2000
            });
    }, () => {
        message.error('Failed to copy to clipboard',
            {
                duration: 2000
            });
    });
}



const unrefresh = router.afterEach(async () => {
    if (router.currentRoute.value.path.startsWith('/list') || router.currentRoute.value.path === '/') {
        await refreshData()
    }
})





onMounted(async () => {
    let path = window.location.pathname
    const page = localStorage.getItem('page:' + path)
    const pageSize = localStorage.getItem('pageSize')
    if (page) {
        paginationReactive.page = parseInt(page)
    }
    if (pageSize) {
        paginationReactive.pageSize = parseInt(pageSize)
    }

    if (window.innerWidth < 768) {
        columns.value = columns.value.filter((column: { key: string; }) => column.key !== 'size' && column.key !== 'lastModified')
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
