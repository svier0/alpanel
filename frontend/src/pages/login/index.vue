<template>
  <div class="login-container">
    <el-card class="login-card">
      <h2 class="login-title">{{ t('login.title') }}</h2>
      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="0"
        @submit.prevent="handleLogin"
      >
        <el-form-item prop="username">
          <el-input
            v-model="form.username"
            :placeholder="t('login.username')"
            :prefix-icon="User"
            size="large"
            autocomplete="username"
          />
        </el-form-item>
        <el-form-item prop="password">
          <el-input
            v-model="form.password"
            type="password"
            :placeholder="t('login.password')"
            :prefix-icon="Lock"
            size="large"
            show-password
            autocomplete="current-password"
            @keyup.enter="handleLogin"
          />
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            size="large"
            :loading="loading"
            class="login-btn"
            @click="handleLogin"
          >
            {{ loading ? t('login.loggingIn') : t('login.submit') }}
          </el-button>
        </el-form-item>
      </el-form>
      <p v-if="error" class="login-error">{{ error }}</p>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { User, Lock } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'

const { t } = useI18n()
const router = useRouter()

const formRef = ref<FormInstance>()
const loading = ref(false)
const error = ref('')

const form = reactive({
  username: '',
  password: '',
})

const rules: FormRules = {
  username: [{ required: true, message: () => t('login.username'), trigger: 'blur' }],
  password: [{ required: true, message: () => t('login.password'), trigger: 'blur' }],
}

async function handleLogin() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    error.value = ''
    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: form.username, password: form.password })
      })
      if (!res.ok) {
        error.value = t('login.invalid')
        return
      }
      const data = await res.json()
      localStorage.setItem('token', data.token)
      router.push('/')
    } catch {
      error.value = t('login.error')
    } finally {
      loading.value = false
    }
  })
}
</script>

<style scoped>
.login-container {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--el-fill-color-light);
}

.login-card {
  width: 420px;
  padding: 20px 0;
}

.login-title {
  text-align: center;
  margin-bottom: 24px;
  color: var(--el-text-color-primary);
  font-size: 22px;
  font-weight: 600;
}

.login-btn {
  width: 100%;
}

.login-error {
  color: var(--el-color-danger);
  font-size: 13px;
  text-align: center;
  margin-top: 8px;
}
</style>
