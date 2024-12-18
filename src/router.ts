import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import { routes } from 'vue-router/auto-routes'
import { setupLayouts } from 'virtual:generated-layouts'

export const router = createRouter({
    history: createWebHashHistory(),
    routes: setupLayouts(routes),
})