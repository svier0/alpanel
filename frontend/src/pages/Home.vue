<template>
  <el-card class="page-card">
    <template #header>
      <div class="card-header-row">
        <div class="header-left">
          <img
            v-if="osLogo"
            :src="osLogo"
            class="os-logo"
            @error="osLogo = ''"
          />
          <el-icon v-else class="os-logo-icon"><Monitor /></el-icon>
          <span>{{ osInfo.os_name }} {{ osInfo.os_version }}</span>
        </div>
        <div class="header-right">
          <span class="header-version">v{{ VERSION }}</span>
          <el-button size="small" disabled>
            <el-icon><RefreshRight /></el-icon>
            {{ t('header.restart') }}
          </el-button>
        </div>
      </div>
    </template>
    <el-empty :description="t('page.home')" />
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Monitor, RefreshRight } from '@element-plus/icons-vue'
import { apiFetch } from '@/utils/api'

const VERSION = '0.1.0'
const { t } = useI18n()

interface OsInfo {
  os_name: string
  os_version: string
  os_id: string
}

const osInfo = ref<OsInfo>({ os_name: '', os_version: '', os_id: '' })
const osLogo = ref('')

const logoMap: Record<string, string> = {
  debian: 'https://www.debian.org/logos/openlogo-nd.svg',
  alpine: 'https://alpinelinux.org/alpinelogo.svg',
  ubuntu: 'https://ubuntu.com/static/images/logos/logo-ubuntu.svg',
}

onMounted(async () => {
  try {
    const data = await apiFetch('/api/system/info')
    osInfo.value = data
    osLogo.value = logoMap[data.os_id] || ''
  } catch {}
})
</script>

<style scoped>
.page-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.page-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.card-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}
.os-logo {
  width: 22px;
  height: 22px;
  object-fit: contain;
}
.os-logo-icon {
  font-size: 22px;
  color: var(--el-color-primary);
}
.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}
.header-version {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
</style>
