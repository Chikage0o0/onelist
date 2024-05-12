<script setup lang="ts">


const theme = ref<any>(darkTheme)

const toggleTheme = () => {
  theme.value = theme.value === darkTheme ? undefined : darkTheme
}
import { onBeforeMount, onMounted, ref, type Ref } from "vue";
import { type GlobalTheme, darkTheme } from "naive-ui";
// locale & dateLocale
import { zhCN, dateZhCN } from "naive-ui";

const name = document.title;

// 自动明暗切换主题
const uiTheme: Ref<GlobalTheme | null> = ref<GlobalTheme | null>(null);
const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
onMounted(() => {

  uiTheme.value = isDark ? darkTheme : null;
  const mqList = window.matchMedia("(prefers-color-scheme: dark)");
  mqList.addEventListener("change", (event) => {
    // is dark mode
    if (event.matches) {
      uiTheme.value = darkTheme;
    } else {
      uiTheme.value = null;
    }
  });
});
</script>

<style scoped>
.link {
  color: #1890ff;
  text-decoration: none;
}
</style>

<template>
  <n-config-provider :theme="uiTheme" :locale="zhCN" :date-locale="dateZhCN">
    <n-loading-bar-provider>
      <n-layout position="absolute">
        <n-layout-header style="height: 64px; padding: 12px">
          <n-flex justify="center">
            <h1 style="margin: 0;">{{ name }}</h1>
          </n-flex>
        </n-layout-header>
        <n-layout position="absolute" style="top: 64px; bottom: 64px" :native-scrollbar="false">
          <n-layout content-style="padding: 24px;">
            <RouterView />
          </n-layout>
        </n-layout>
        <n-layout-footer position="absolute" style="height: 64px">
          <n-flex justify="center" style="padding-top: 12px;">
            <p>Powered by <a href="https://github.com/Chikage0o0/onelist" class="link" target="_blank">OneList</a>
              ,developed by <a href="https://github.com/Chikage0o0" class="link" target="_blank">Chikage</a>.</p>
          </n-flex>
        </n-layout-footer>
      </n-layout>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<style scoped></style>
