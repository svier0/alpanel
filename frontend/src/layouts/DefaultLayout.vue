<template>
  <div class="layout-container">
    <aside
      class="layout-aside"
      :style="{ width: isCollapse ? '64px' : '210px' }"
    >
      <div class="logo-area">
        <span v-if="!isCollapse" class="logo-text">{{ settings.title }}</span>
        <span v-else class="logo-text-short">A</span>
      </div>
      <div class="menu-wrap">
        <el-menu
          :default-active="activeMenu"
          :collapse="isCollapse"
          :router="true"
          class="layout-menu"
        >
          <el-menu-item index="/">
            <el-icon><HomeFilled /></el-icon>
            <template #title>{{ t('menu.home') }}</template>
          </el-menu-item>
          <el-menu-item index="/website">
            <el-icon><Monitor /></el-icon>
            <template #title>{{ t('menu.website') }}</template>
          </el-menu-item>
          <el-menu-item index="/file">
            <el-icon><Document /></el-icon>
            <template #title>{{ t('menu.file') }}</template>
          </el-menu-item>
          <el-menu-item index="/database">
            <el-icon><Coin /></el-icon>
            <template #title>{{ t('menu.database') }}</template>
          </el-menu-item>
          <el-menu-item index="/cron">
            <el-icon><Timer /></el-icon>
            <template #title>{{ t('menu.cron') }}</template>
          </el-menu-item>
          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <template #title>{{ t('menu.settings') }}</template>
          </el-menu-item>
          <el-menu-item index="/logout">
            <el-icon><SwitchButton /></el-icon>
            <template #title>{{ t('menu.logout') }}</template>
          </el-menu-item>
        </el-menu>
      </div>
      <div class="collapse-area" @click="isCollapse = !isCollapse">
        <el-icon class="collapse-icon">
          <Fold v-if="!isCollapse" />
          <Expand v-else />
        </el-icon>
      </div>
    </aside>
    <div class="layout-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  HomeFilled, Monitor, Document, Coin, Timer, Setting,
  Fold, Expand, SwitchButton
} from '@element-plus/icons-vue'
import { settings, fetchSettings } from '@/stores/settings'

const { t } = useI18n()
const route = useRoute()
const isCollapse = ref(false)

const activeMenu = computed(() => route.path)

onMounted(fetchSettings)
</script>

<style scoped>
.layout-container {
  display: flex;
  width: 100vw;
  height: 100vh;
}

.layout-aside {
  background-color: var(--alpanel-aside-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.logo-area {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 18px;
  font-weight: 600;
  border-bottom: 1px solid var(--alpanel-aside-border);
  white-space: nowrap;
  overflow: hidden;
  flex-shrink: 0;
}

.logo-text {
  padding: 0 12px;
}

.logo-text-short {
  font-size: 20px;
}

.menu-wrap {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.layout-menu {
  border-right: none;
}

.layout-menu:not(.el-menu--collapse) {
  width: 210px;
}

.collapse-area {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-top: 1px solid var(--alpanel-aside-border);
  cursor: pointer;
  color: var(--el-menu-text-color);
  flex-shrink: 0;
}

.collapse-area:hover {
  color: var(--el-menu-active-color);
  background-color: rgba(255, 255, 255, 0.05);
}

.collapse-icon {
  font-size: 20px;
}

.layout-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  padding: 20px;
  background: var(--el-fill-color-light);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

.layout-content > * {
  flex: 1;
}
</style>
