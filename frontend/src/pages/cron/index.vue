<template>
  <el-card class="page-card">
    <template #header>
      <span class="card-header">{{ t('page.cron') }}</span>
    </template>
    <el-empty :description="t('page.cron')" />
  </el-card>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
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
</style>
