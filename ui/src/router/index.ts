import { createRouter, createWebHistory } from "vue-router";
import HomeView from "@/views/FolderView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },

    {
      path: "/list/:p*",
      name: "list",
      component: HomeView,
    },
  ],
});

export default router;
