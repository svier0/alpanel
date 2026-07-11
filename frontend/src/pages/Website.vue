<template>
  <div>
    <h1 class="text-2xl font-semibold">{{ t('page.website') }}</h1>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>
