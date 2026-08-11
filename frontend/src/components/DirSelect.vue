<template>
    <div class="dir-select">
        <el-input v-model="display" size="small" readonly placeholder="/">
            <template #append>
                <el-button @click="open">浏览</el-button>
            </template>
        </el-input>
        <DirPicker
            v-model="visible"
            :initial-path="resolveOpenPath()"
            :base-path="basePath"
            :show-up="showUp"
            @confirm="picked"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import DirPicker from '@/components/DirPicker.vue'

const props = withDefaults(defineProps<{
    modelValue: string
    basePath?: string
    showUp?: boolean
}>(), {
    basePath: '',
    showUp: true,
})

const emit = defineEmits<{
    (e: 'update:modelValue', v: string): void
}>()

const visible = ref(false)

const display = computed(() => props.modelValue)

function resolveOpenPath(): string {
    const v = props.modelValue
    if (props.basePath) {
        if (v === '/' || v === '') return props.basePath
        return props.basePath.replace(/\/$/, '') + (v.startsWith('/') ? v : '/' + v)
    }
    return v || '/'
}

function open() {
    visible.value = true
}

function picked(path: string) {
    emit('update:modelValue', path)
}
</script>

<style scoped>
.dir-select {
    flex: 1;
}
</style>