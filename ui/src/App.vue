<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { defineComponent, ref } from 'vue'
import { darkTheme } from 'naive-ui'
import type { GlobalTheme } from 'naive-ui'

const theme = ref<GlobalTheme>(darkTheme)

const toggleTheme = () => {
  theme.value = theme.value === darkTheme ? undefined : darkTheme
}
import { defineComponent, onMounted, ref, type Ref } from "vue";
import { type GlobalTheme, darkTheme } from "naive-ui";
// locale & dateLocale
import { zhCN, dateZhCN } from "naive-ui";



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

<template>
  <n-config-provider :theme="uiTheme" :locale="zhCN" :date-locale="dateZhCN">
    <n-dialog-provider>
      <n-notification-provider>
        <n-message-provider>
          <n-loading-bar-provider>
            <n-layout position="absolute" embedded>
              <n-layout-header bordered>
                <n-flex justify="space-between">
                  <n-button>Oops!</n-button>
                  <n-button>Oops!</n-button>
                  <n-button>Oops!</n-button>
                </n-flex>
              </n-layout-header>
              <n-layout-content content-style="padding: 24px;" :native-scrollbar="false">
                <RouterView />
              </n-layout-content>
              <n-layout-footer position="absolute" bordered>
                <n-flex justify="center">
                  <n-button>Oops!</n-button>
                  <n-button>Oops!</n-button>
                  <n-button>Oops!</n-button>
                </n-flex>
              </n-layout-footer>
            </n-layout>
          </n-loading-bar-provider>
        </n-message-provider>
      </n-notification-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<style scoped></style>
