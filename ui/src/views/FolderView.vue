<script setup lang="ts">
import {  h, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

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
  maxHeight.value = window.innerHeight - 210
}

function bytesToSize(bytes: number) {
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  if (bytes === 0) return '0 Byte'
  const i = parseInt(String(Math.floor(Math.log(bytes) / Math.log(1024))))
  return Math.round(bytes / Math.pow(1024, i)) + ' ' + sizes[i]
}

function timestampToDateTime(timestamp: number) {
  const date = new Date(timestamp * 1000)
  return date.toLocaleString()
}



const pagination = {
  pageSize: 10,
}

const data:any= ref([])

const refreshData = async () => {
  let path = window.location.pathname
  if (path === '/') {
    path = ''
  }
  if (path.startsWith('/list')) {
    path = path.slice(5)
  }
  let url = `/api/list${path}`

  try {
    const res = await fetch(url)
    const json = await res.json()
    const files = json.files
    data.value = []
    files.forEach((file: { type: string; name: any; id: any; size: number; created_date_time: number; last_modified_date_time: number; }) => {
      let action;
      if (file.type === 'Folder') {
        if (path === '') {
          action = h('a', { onClick: () => router.push(`/list/${file.name}`) }, 'Open');
        } else {
          action = h('a', { onClick: () => router.push(`/list${path}/${file.name}`) }, 'Open');
        }
      } else {
        action = h('a', { onClick: () => window.open(`/api/download/proxy/${file.id}`) }, 'Download');
      }

      data.value.push({
        name: file.name,
        size: file.size ? bytesToSize(file.size) : '',
        creationDate: file.created_date_time ? timestampToDateTime(file.created_date_time) : '',
        lastModified: file.last_modified_date_time ? timestampToDateTime(file.last_modified_date_time) : '',
        action: action
      })
    })
  } catch (e) {
    console.error(e)
  }
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

<template>
  <n-data-table ref="table" :columns="columns" :data="data" :pagination="pagination" :max-height="maxHeight" />
</template>
