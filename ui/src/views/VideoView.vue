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
      <n-button type="primary" size="small" icon="copy" :onClick="() => copyToClipboard(videoUrl)">
        Copy Link
      </n-button>
    </n-flex>
  </div>
</template>

<script setup lang="ts">

import DPlayer, { DPlayerEvents } from 'dplayer';
import { useMessage } from 'naive-ui';
import { onMounted, onUnmounted, ref } from 'vue';
import { useRoute } from 'vue-router';



const { p } = useRoute().params;
const path = (p as string[]).join('/');
const videoUrl = ref("");
const thumbUrl = ref("");

const name = ref('');
const message = useMessage();
const dp = ref<DPlayer>();




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

  dp.value = new DPlayer({
    container: document.getElementById('dplayer'),
    video: {
      url: videoUrl.value,
      pic: thumbUrl.value,
    },
  });




  // 添加监听事件
  dp.value.on('loadedmetadata' as DPlayerEvents, () => {
    const lastTime = localStorage.getItem('video:' + videoUrl.value);
    if (lastTime) {
      dp.value && dp.value.seek(parseFloat(lastTime));
    }
  });

  dp.value.on('timeupdate' as DPlayerEvents, () => {
    if (dp.value && dp.value.video && dp.value.video.currentTime !== 0) {
      localStorage.setItem('video:' + videoUrl.value, dp.value.video.currentTime.toString());
    }
  });

  dp.value.on('ended' as DPlayerEvents, () => {
    localStorage.removeItem('video:' + videoUrl.value);
  });

});



onUnmounted(() => {
  if (dp.value) {
    dp.value.destroy();
  }
});

function triggerDownload(url: string, fileName: string) {
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;  // 指定下载文件名
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);

  dp.value && dp.value.pause();

  message.success('Download started',
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

function create_new_window(uri: string) {
  // 拼接前缀
  let url = document.location.protocol + "//" + document.location.host + uri;
  window.open(url);
}

</script>
