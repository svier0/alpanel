<template>
    <el-dialog
        v-model="visible"
        title="选择目录"
        width="500px"
        append-to-body
        @opened="focusInput"
    >
        <div style="margin-bottom:8px;color:var(--el-text-color-secondary);font-size:12px">{{ currentPath }}</div>
        <div style="display:flex;gap:4px;margin-bottom:8px">
            <el-input v-model="newDir" placeholder="新建子目录名称" size="small" @keyup.enter="createDir" />
            <el-button size="small" type="primary" @click="createDir" :loading="creating">新建</el-button>
        </div>
        <div style="max-height:300px;overflow-y:auto;border:1px solid var(--el-border-color-lighter);border-radius:4px">
            <div
                v-for="item in items"
                :key="item.path"
                style="padding:6px 12px;cursor:pointer;font-size:13px;display:flex;align-items:center;gap:6px"
                @click="enterDir(item)"
            >
                <span style="color:#e6a23c">📁</span>
                <span>{{ item.name }}</span>
            </div>
            <div v-if="items.length === 0" style="padding:12px;color:var(--el-text-color-secondary);font-size:12px;text-align:center">无子目录</div>
        </div>
        <template #footer>
            <el-button v-if="showUp" @click="goUp">返回上级</el-button>
            <el-button @click="visible = false">取消</el-button>
            <el-button type="primary" @click="confirm">选择当前目录</el-button>
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { apiFetch } from '@/utils/api'

const props = defineProps<{
    modelValue: boolean
    initialPath?: string
    showUp?: boolean
}>()

const emit = defineEmits<{
    (e: 'update:modelValue', v: boolean): void
    (e: 'confirm', path: string): void
}>()

const visible = computed({
    get: () => props.modelValue,
    set: (v: boolean) => emit('update:modelValue', v),
})

const currentPath = ref('/')
const parentPath = ref('')
const items = ref<{ name: string; path: string; is_dir: boolean }[]>([])
const newDir = ref('')
const creating = ref(false)

watch(() => props.modelValue, (v) => {
    if (v) {
        load(props.initialPath || '/')
        nextTick(focusInput)
    }
})

async function load(path: string) {
    try {
        const data = await apiFetch('/api/files/list?path=' + encodeURIComponent(path))
        items.value = (data.items || []).filter((i: any) => i.is_dir)
        currentPath.value = data.path
        parentPath.value = data.parent || ''
    } catch {}
}

function enterDir(item: { name: string; path: string; is_dir: boolean }) {
    if (item.is_dir) load(item.path)
}

function goUp() {
    if (parentPath.value) load(parentPath.value)
}

function confirm() {
    const dir = currentPath.value.endsWith('/') ? currentPath.value : currentPath.value + '/'
    emit('confirm', dir)
    visible.value = false
}

async function createDir() {
    const name = newDir.value.trim()
    if (!name) return
    creating.value = true
    try {
        const p = currentPath.value.endsWith('/') ? currentPath.value + name : currentPath.value + '/' + name
        await apiFetch('/api/files/create', {
            method: 'POST',
            body: JSON.stringify({ path: p, type: 'dir' }),
        })
        newDir.value = ''
        await load(currentPath.value)
    } finally {
        creating.value = false
    }
}

function focusInput() {
    const el = document.querySelector<HTMLInputElement>('.el-dialog input[placeholder="新建子目录名称"]')
    if (el) el.focus()
}
</script>