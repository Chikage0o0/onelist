<template>
  <div id="dplayer"></div>
</template>

<style scoped>
#dplayer {
  width: 50%;
  height: 50%;
}
</style>

<script setup lang="ts">
import DPlayer from 'dplayer';
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const { id } = route.params;


onMounted(() => {
  new DPlayer({
    container: document.getElementById('dplayer'),
    screenshot: true,
    video: {
      url: '/api/download/' + id,
      pic: '/api/thumb/large/' + id,
    },
  });
});

</script>
