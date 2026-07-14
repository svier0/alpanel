<template>
  <el-card class="page-card">
    <template #header>
      <span class="card-header">{{ t('page.settings') }}</span>
    </template>
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="120px"
      label-position="right"
      style="max-width: 500px"
    >
      <el-form-item :label="t('settings.port')" prop="port">
        <el-input-number v-model="form.port" :min="1" :max="65535" controls-position="right" />
      </el-form-item>
      <el-form-item :label="t('settings.user')" prop="user">
        <el-input v-model="form.user" :placeholder="t('settings.user')" />
      </el-form-item>
      <el-form-item :label="t('settings.password')">
        <el-input
          v-model="form.password"
          type="password"
          placeholder="••••••••"
          show-password
        />
      </el-form-item>
      <el-form-item :label="t('settings.title')">
        <el-input v-model="form.title" :placeholder="t('settings.title')" />
      </el-form-item>
      <el-form-item :label="t('settings.theme')">
        <el-select v-model="form.theme" style="width: 100%">
          <el-option value="auto" :label="t('settings.themeAuto')" />
          <el-option value="light" :label="t('settings.themeLight')" />
          <el-option value="dark" :label="t('settings.themeDark')" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" :loading="saving" @click="handleSave">
          {{ saving ? t('settings.saving') : t('settings.save') }}
        </el-button>
      </el-form-item>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { saveThemeLocally, saveTitleLocally } from '@/stores/settings'
import { apiFetch } from '@/utils/api'

const { t } = useI18n()

const formRef = ref<FormInstance>()

const form = reactive({
  port: 5555,
  user: '',
  password: '',
  title: '',
  theme: 'auto',
})

const rules: FormRules = {
  port: [{ required: true, trigger: 'blur' }],
  user: [{ required: true, trigger: 'blur' }],
}

const saving = ref(false)

onMounted(async () => {
  try {
    const data = await apiFetch('/api/settings')
    form.port = data.port
    form.user = data.user
    form.title = data.title
    form.theme = data.theme
  } catch {}
})

async function handleSave() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    saving.value = true
    try {
      const data = await apiFetch('/api/settings', {
        method: 'PUT',
        body: JSON.stringify({
          port: form.port,
          user: form.user || undefined,
          password: form.password || undefined,
          title: form.title || undefined,
          theme: form.theme,
        }),
      })
      saveTitleLocally(data.title)
      saveThemeLocally(data.theme)
      ElMessage.success(t('settings.saved'))
      form.password = ''
    } catch {
      ElMessage.error(t('settings.failed'))
    } finally {
      saving.value = false
    }
  })
}
</script>

<style scoped>
.page-card {
  flex: 1;
}
</style>
