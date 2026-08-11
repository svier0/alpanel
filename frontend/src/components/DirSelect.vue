<template>
    <div class="dir-select">
        <input v-model="display" :readonly="true" class="dir-select-input" placeholder="/" />
        <button class="dir-select-btn" @click="open">浏览</button>
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
    display: flex;
    align-items: stretch;
    flex: 0 0 auto;
    width: 440px;
}

.dir-select-input {
    flex: 1;
    min-width: 0;
    padding: 5px 10px;
    border: 1px solid #555;
    border-right: none;
    border-radius: 3px 0 0 3px;
    background: #1a1a1a;
    color: #ccc;
    font-size: 13px;
    outline: none;
    box-sizing: border-box;
}

.dir-select-input:focus {
    border-color: #409eff;
}

.dir-select-input:focus + .dir-select-btn {
    border-color: #409eff;
}

.dir-select-btn {
    flex: 0 0 auto;
    padding: 5px 14px;
    border: 1px solid #555;
    border-radius: 0 3px 3px 0;
    background: #141414;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
}

.dir-select-btn:hover {
    color: #fff;
    border-color: #888;
}
</style>