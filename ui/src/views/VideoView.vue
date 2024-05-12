<template>
  <div style="max-width: 1000px; margin: 0 auto;">
    <Breadcrumb />
    <n-card :title="name">
      <div id="dplayer" />
    </n-card>
    <n-flex justify="center" style="margin-top: 10px;">
      <n-button type="primary" size="small" icon="cloud-download" :onClick="() => triggerDownload(videoUrl, name)">
        Download
      </n-button>
    </n-flex>
  </div>
</template>

<script setup lang="ts">

import DPlayer from 'dplayer';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';



const { p } = useRoute().params;
const path = (p as string[]).join('/');
const videoUrl = ref("");
const thumbUrl = ref("");

const name = ref('');



onMounted(async () => {
  console.log(p);
  try {
    const response = await fetch(`/api/info/${path}`);
    const data = await response.json();
    name.value = data.file.name;
    videoUrl.value = `/api/download/${data.file.id}`;
    thumbUrl.value = `/api/thumb/large/${data.file.id}`;
  } catch (error) {
    console.error(error);
  }

  new DPlayer({
    container: document.getElementById('dplayer'),
    screenshot: true,
    video: {
      url: videoUrl.value,
      pic: thumbUrl.value,
    },
  });
});

function triggerDownload(url: string, fileName: string) {
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;  // 指定下载文件名
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
}

function create_new_window(uri: string) {
  // 拼接前缀
  let url = document.location.protocol + "//" + document.location.host + uri;
  window.open(url);
}

</script>
