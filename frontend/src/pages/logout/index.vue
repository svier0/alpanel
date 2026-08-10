<template>
  <div></div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

onMounted(() => {
  localStorage.removeItem('token')
  router.replace('/login')
})
</script>
